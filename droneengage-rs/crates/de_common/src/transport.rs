// Ported from droneengage_common@5fb11c2: de_databus/udpClient.cpp, udpClient.hpp,
// unixDgramClient.cpp, unixDgramClient.hpp
//
// Datagram transport between a module and de_comm. Both flavours use the
// chunk protocol from `chunk.rs`. The periodic ID sender of the C++ clients
// lives in `module.rs` (it is transport independent).

use std::collections::HashMap;
use std::io;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use tokio::net::{UdpSocket, UnixDatagram};
use tokio::sync::Mutex;

use crate::chunk::{self, Reassembler};

/// Max datagram we accept (C++ MAXLINE).
const MAX_DATAGRAM: usize = 0xFFFF;
/// Pause between chunks of one message ("fast sending causes packet loss").
const INTER_CHUNK_DELAY: Duration = Duration::from_millis(10);

enum Socket {
    Udp {
        socket: UdpSocket,
        target: SocketAddr,
    },
    Unix {
        socket: UnixDatagram,
        broker: PathBuf,
        own: PathBuf,
    },
}

/// A bound databus endpoint.
pub struct Transport {
    socket: Socket,
    chunk_size: usize,
    stopped: AtomicBool,
    /// Serialises whole messages so chunks of two messages never interleave
    /// (C++ `m_lock` in sendMSG).
    send_lock: Mutex<()>,
    rx: Mutex<RxState>,
}

#[derive(Default)]
struct RxState {
    udp: Reassembler,
    unix_by_source: HashMap<PathBuf, Reassembler>,
}

impl Transport {
    /// UDP: bind `bind` (module listening address) and send to `target` (de_comm).
    pub async fn bind_udp(
        target: SocketAddr,
        bind: SocketAddr,
        chunk_size: usize,
    ) -> io::Result<Self> {
        check_chunk_size(chunk_size)?;
        let socket = UdpSocket::bind(bind).await?;
        tracing::info!(%bind, %target, chunk_size, "databus: UDP listener");
        Ok(Self::new(Socket::Udp { socket, target }, chunk_size))
    }

    /// Unix datagram: bind `own` and send to `broker` (de_comm's socket).
    /// Parent directories of `own` are created (mode 0775) and a stale
    /// `own` socket file is removed first, like the C++ client.
    pub fn bind_unix(broker: &Path, own: &Path, chunk_size: usize) -> io::Result<Self> {
        if chunk_size == 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "chunk size must be > 0",
            ));
        }
        if let Some(parent) = own.parent() {
            if !parent.as_os_str().is_empty() {
                create_dirs_0775(parent);
            }
        }
        let _ = std::fs::remove_file(own);
        let socket = UnixDatagram::bind(own)?;
        tracing::info!(own = %own.display(), broker = %broker.display(), chunk_size, "databus: unix socket");
        Ok(Self::new(
            Socket::Unix {
                socket,
                broker: broker.to_path_buf(),
                own: own.to_path_buf(),
            },
            chunk_size,
        ))
    }

    fn new(socket: Socket, chunk_size: usize) -> Self {
        Self {
            socket,
            chunk_size,
            stopped: AtomicBool::new(false),
            send_lock: Mutex::new(()),
            rx: Mutex::new(RxState::default()),
        }
    }

    pub fn is_unix(&self) -> bool {
        matches!(self.socket, Socket::Unix { .. })
    }

    /// Local address (UDP only), mostly for tests.
    pub fn local_udp_addr(&self) -> Option<SocketAddr> {
        match &self.socket {
            Socket::Udp { socket, .. } => socket.local_addr().ok(),
            Socket::Unix { .. } => None,
        }
    }

    /// Stop sending; pending and future `send` calls return immediately.
    pub fn stop(&self) {
        self.stopped.store(true, Ordering::SeqCst);
    }

    /// Send one message, chunked. Errors are logged and end the message
    /// early, the same way C++ `break`s out of its loop.
    pub async fn send(&self, msg: &[u8]) {
        if self.stopped.load(Ordering::SeqCst) {
            return;
        }
        let _guard = self.send_lock.lock().await;
        let datagrams = chunk::split(msg, self.chunk_size);
        let last = datagrams.len().saturating_sub(1);
        for (i, d) in datagrams.iter().enumerate() {
            if self.stopped.load(Ordering::SeqCst) {
                return;
            }
            if let Err(e) = self.send_datagram(d, i == 0).await {
                tracing::warn!(error = %e, "databus: sendto failed");
                return;
            }
            if i != last {
                tokio::time::sleep(INTER_CHUNK_DELAY).await;
            }
        }
    }

    async fn send_datagram(&self, d: &[u8], first: bool) -> io::Result<()> {
        match &self.socket {
            Socket::Udp { socket, target } => socket.send_to(d, target).await.map(|_| ()),
            Socket::Unix { socket, broker, .. } => {
                let mut r = socket.send_to(d, broker).await;
                // de_comm may not have created its socket yet: back off
                // 100, 200, 400, 800, 1600 ms on the first chunk only.
                if first {
                    let mut retry = 0;
                    while let Err(e) = &r {
                        if e.kind() != io::ErrorKind::NotFound
                            || retry >= 5
                            || self.stopped.load(Ordering::SeqCst)
                        {
                            break;
                        }
                        tokio::time::sleep(Duration::from_millis(100 << retry)).await;
                        r = socket.send_to(d, broker).await;
                        retry += 1;
                    }
                }
                r.map(|_| ())
            }
        }
    }

    /// Wait for the next complete message. The returned buffer ends with an
    /// extra NUL byte (see `Reassembler`). Only one task should call this.
    pub async fn recv(&self) -> io::Result<Vec<u8>> {
        let mut buf = vec![0u8; MAX_DATAGRAM];
        loop {
            match &self.socket {
                Socket::Udp { socket, .. } => {
                    let (n, _from) = socket.recv_from(&mut buf).await?;
                    // C++ accepts datagrams from any sender into one buffer.
                    if let Some(msg) = self.rx.lock().await.udp.push(&buf[..n]) {
                        return Ok(msg);
                    }
                }
                Socket::Unix { socket, .. } => {
                    let (n, from) = socket.recv_from(&mut buf).await?;
                    let key = from
                        .as_pathname()
                        .map(Path::to_path_buf)
                        .unwrap_or_default();
                    let mut rx = self.rx.lock().await;
                    if let Some(msg) = rx.unix_by_source.entry(key).or_default().push(&buf[..n]) {
                        return Ok(msg);
                    }
                }
            }
        }
    }
}

impl Drop for Transport {
    fn drop(&mut self) {
        if let Socket::Unix { own, .. } = &self.socket {
            let _ = std::fs::remove_file(own);
        }
    }
}

fn check_chunk_size(chunk_size: usize) -> io::Result<()> {
    if chunk_size == 0 || chunk_size >= chunk::MAX_UDP_DATABUS_PACKET_SIZE {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("invalid UDP packet size: {chunk_size}"),
        ));
    }
    Ok(())
}

/// mkdir -p with mode 0775, logging but not failing (C++ behaviour).
fn create_dirs_0775(dir: &Path) {
    use std::os::unix::fs::DirBuilderExt;
    if let Err(e) = std::fs::DirBuilder::new()
        .recursive(true)
        .mode(0o775)
        .create(dir)
    {
        tracing::warn!(dir = %dir.display(), error = %e, "databus: cannot create socket dir");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn udp_roundtrip_multi_chunk() {
        let a = Transport::bind_udp(
            "127.0.0.1:9".parse().unwrap(),
            "127.0.0.1:0".parse().unwrap(),
            1000,
        )
        .await
        .unwrap();
        let a_addr = a.local_udp_addr().unwrap();
        let b = Transport::bind_udp(a_addr, "127.0.0.1:0".parse().unwrap(), 1000)
            .await
            .unwrap();
        let msg: Vec<u8> = (0..5000u32).map(|i| (i % 251) as u8).collect();
        b.send(&msg).await;
        let got = a.recv().await.unwrap();
        assert_eq!(&got[..got.len() - 1], &msg[..]);
        assert_eq!(*got.last().unwrap(), 0);
    }

    #[tokio::test]
    async fn unix_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let broker = dir.path().join("sub/broker.sock");
        let own = dir.path().join("sub/own.sock");
        let b = Transport::bind_unix(&own, &broker, 8192).unwrap();
        let a = Transport::bind_unix(&broker, &own, 8192).unwrap();
        a.send(b"{\"x\":1}").await;
        assert_eq!(b.recv().await.unwrap(), b"{\"x\":1}\0");
    }

    #[tokio::test]
    async fn invalid_chunk_size_rejected() {
        let r = Transport::bind_udp(
            "127.0.0.1:9".parse().unwrap(),
            "127.0.0.1:0".parse().unwrap(),
            0xFFFF,
        )
        .await;
        assert!(r.is_err());
    }
}
