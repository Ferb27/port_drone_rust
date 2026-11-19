// Ported from droneengage_common@5fb11c2: de_databus/chunk_protocol.hpp,
// udpClient.cpp (sendMSG, InternalReceiverEntry), unixDgramClient.cpp
//
// Databus framing: a message is split into datagrams of at most `chunk_size`
// payload bytes, each prefixed by a 2-byte little-endian chunk number. The
// last datagram always carries 0xFFFF instead of its number.

/// Chunk number used for the last datagram of a message.
pub const CHUNK_END_MARKER: u16 = 0xFFFF;
/// Size of the per-datagram header.
pub const CHUNK_HEADER_SIZE: usize = 2;
/// Largest chunk size accepted by the C++ UDP client (exclusive).
pub const MAX_UDP_DATABUS_PACKET_SIZE: usize = 0xFFFF;
/// Default chunk size on loopback.
pub const DEFAULT_UDP_DATABUS_PACKET_SIZE: usize = 8192;
/// Safe payload for a 1500-byte Ethernet MTU, used for non-loopback targets.
pub const SAFE_REMOTE_UDP_PAYLOAD: usize = 1472;

/// 2-byte header for chunk `n`.
pub fn chunk_header(n: u16, last: bool) -> [u8; 2] {
    if last {
        [0xFF, 0xFF]
    } else {
        n.to_le_bytes()
    }
}

/// Split `msg` into ready-to-send datagrams (header + payload).
///
/// Same as C++: an empty message produces no datagram at all, and chunk
/// numbers wrap at 16 bits.
pub fn split(msg: &[u8], chunk_size: usize) -> Vec<Vec<u8>> {
    if chunk_size == 0 {
        return Vec::new();
    }
    let count = msg.len().div_ceil(chunk_size);
    msg.chunks(chunk_size)
        .enumerate()
        .map(|(i, payload)| {
            let mut d = Vec::with_capacity(CHUNK_HEADER_SIZE + payload.len());
            d.extend_from_slice(&chunk_header(i as u16, i + 1 == count));
            d.extend_from_slice(payload);
            d
        })
        .collect()
}

/// Reassembles datagrams from ONE source into complete messages.
///
/// Mirrors the C++ receiver exactly:
/// - datagrams shorter than 2 bytes are ignored;
/// - chunk 0 clears whatever was pending (a lost tail is dropped);
/// - every other chunk is appended in arrival order (no reordering, no gap check);
/// - on 0xFFFF the concatenation is returned with a trailing NUL byte appended,
///   because C++ hands the buffer to code that treats it as a C string.
#[derive(Debug, Default)]
pub struct Reassembler {
    pending: Vec<u8>,
}

impl Reassembler {
    pub fn new() -> Self {
        Self::default()
    }

    /// Feed one datagram. Returns a complete message (with trailing NUL) when
    /// this datagram was the last chunk.
    pub fn push(&mut self, datagram: &[u8]) -> Option<Vec<u8>> {
        if datagram.len() < CHUNK_HEADER_SIZE {
            tracing::warn!(len = datagram.len(), "databus: packet too small");
            return None;
        }
        let n = u16::from_le_bytes([datagram[0], datagram[1]]);
        if n == 0 {
            self.pending.clear();
        }
        self.pending
            .extend_from_slice(&datagram[CHUNK_HEADER_SIZE..]);
        if n == CHUNK_END_MARKER {
            let mut msg = std::mem::take(&mut self.pending);
            msg.push(0);
            Some(msg)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_chunk_uses_end_marker() {
        let d = split(b"{}", 8192);
        assert_eq!(d, vec![vec![0xFF, 0xFF, b'{', b'}']]);
    }

    #[test]
    fn empty_message_sends_nothing() {
        assert!(split(b"", 8192).is_empty());
    }

    #[test]
    fn multi_chunk_numbers_little_endian() {
        let msg: Vec<u8> = (0..10u8).collect();
        let d = split(&msg, 4);
        assert_eq!(d.len(), 3);
        assert_eq!(&d[0][..2], &[0, 0]);
        assert_eq!(&d[1][..2], &[1, 0]);
        assert_eq!(&d[2][..2], &[0xFF, 0xFF]);
        assert_eq!(d[2].len(), 2 + 2);
    }

    #[test]
    fn roundtrip_appends_nul() {
        let msg: Vec<u8> = (0..=255u8).cycle().take(20_000).collect();
        let mut r = Reassembler::new();
        let mut out = None;
        for d in split(&msg, 8192) {
            out = r.push(&d);
        }
        let mut expect = msg.clone();
        expect.push(0);
        assert_eq!(out.unwrap(), expect);
    }

    #[test]
    fn chunk_zero_drops_incomplete_previous() {
        let mut r = Reassembler::new();
        assert!(r.push(&[0, 0, b'x']).is_none()); // tail never arrives
        assert!(r.push(&[0, 0, b'a']).is_none());
        assert_eq!(r.push(&[0xFF, 0xFF, b'b']).unwrap(), b"ab\0");
    }

    #[test]
    fn lost_tail_then_single_chunk_concatenates_like_cpp() {
        // C++ only clears on chunk 0, so a single-chunk message after a lost
        // tail gets the stale prefix. Kept on purpose for compatibility.
        let mut r = Reassembler::new();
        r.push(&[0, 0, b'x']);
        assert_eq!(r.push(&[0xFF, 0xFF, b'y']).unwrap(), b"xy\0");
    }

    #[test]
    fn tiny_packet_ignored() {
        let mut r = Reassembler::new();
        assert!(r.push(&[0xFF]).is_none());
    }
}
