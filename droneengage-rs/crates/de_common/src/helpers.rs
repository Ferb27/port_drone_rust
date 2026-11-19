// Ported from droneengage_common@5fb11c2: helpers/helpers.cpp
// (removeComments, isLocalhost, getUnixSocketBaseDir, read_env_flag, time helpers)

use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

/// Strip `//` and `/* */` comments. Byte-for-byte the naive C++ algorithm:
/// it does NOT know about string literals, so `"http://x"` loses `//x"`.
/// Config files written for C++ already live with that, so we keep it.
pub fn remove_comments(prgm: &str) -> String {
    let b = prgm.as_bytes();
    let n = b.len();
    let mut res = Vec::with_capacity(n);
    let (mut s_cmt, mut m_cmt) = (false, false);
    let mut i = 0;
    while i < n {
        if s_cmt && b[i] == b'\n' {
            s_cmt = false;
        } else if m_cmt && b[i] == b'*' && i + 1 < n && b[i + 1] == b'/' {
            m_cmt = false;
            i += 1;
        } else if s_cmt || m_cmt {
            // inside a comment
        } else if b[i] == b'/' && i + 1 < n && b[i + 1] == b'/' {
            s_cmt = true;
            i += 1;
        } else if b[i] == b'/' && i + 1 < n && b[i + 1] == b'*' {
            m_cmt = true;
            i += 1;
        } else {
            res.push(b[i]);
        }
        i += 1;
    }
    // Only ASCII bytes are ever removed, so UTF-8 stays valid.
    String::from_utf8(res).unwrap_or_default()
}

/// "127.0.0.1", "::1" or "localhost" (case-insensitive).
pub fn is_localhost(ip: &str) -> bool {
    let l = ip.to_ascii_lowercase();
    l == "127.0.0.1" || l == "::1" || l == "localhost"
}

/// True when env var `name` is 1/true/TRUE/yes/YES/on/ON.
pub fn read_env_flag(name: &str) -> bool {
    matches!(
        std::env::var(name).as_deref(),
        Ok("1" | "true" | "TRUE" | "yes" | "YES" | "on" | "ON")
    )
}

/// Wall clock in microseconds since the Unix epoch (C++ get_time_usec).
pub fn get_time_usec() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_micros() as u64)
        .unwrap_or(0)
}

/// Seconds since the Unix epoch (C++ std::time(nullptr)).
pub fn unix_time_sec() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// Resolve the unix socket base directory used by the module bus.
///
/// Order: `configured_dir`, then `$DE_UNIX_SOCKET_DIR`, then `/run/de_comm`,
/// then `/tmp/de_comm`. Returns `None` when an explicitly configured (or env)
/// directory cannot be created or is not writable; callers then fall back to UDP.
pub fn unix_socket_base_dir(configured_dir: &str) -> Option<String> {
    let mut dir = configured_dir.to_string();
    if dir.is_empty() {
        if let Ok(env) = std::env::var("DE_UNIX_SOCKET_DIR") {
            dir = env;
        }
    }
    if !dir.is_empty() {
        while dir.len() > 1 && dir.ends_with('/') {
            dir.pop();
        }
        if ensure_dir_path(&dir) {
            return Some(dir);
        }
        tracing::error!(%dir, "unix socket dir cannot be created or is not writable");
        return None;
    }
    for d in ["/run/de_comm", "/tmp/de_comm"] {
        if ensure_dir(d) {
            return Some(d.to_string());
        }
    }
    // Neither worked: return the preferred path so the bind error names it.
    Some("/run/de_comm".to_string())
}

fn writable(dir: &str) -> bool {
    // access(W_OK|X_OK) approximation: a directory we can create files in.
    let probe = Path::new(dir).join(format!(".de_probe_{}", std::process::id()));
    match std::fs::File::create(&probe) {
        Ok(_) => {
            let _ = std::fs::remove_file(&probe);
            true
        }
        Err(_) => false,
    }
}

fn ensure_dir(dir: &str) -> bool {
    use std::os::unix::fs::DirBuilderExt;
    match std::fs::DirBuilder::new().mode(0o775).create(dir) {
        Ok(()) => true,
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => writable(dir),
        Err(_) => false,
    }
}

fn ensure_dir_path(dir: &str) -> bool {
    use std::os::unix::fs::DirBuilderExt;
    match std::fs::DirBuilder::new()
        .recursive(true)
        .mode(0o775)
        .create(dir)
    {
        Ok(()) => writable(dir),
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn comments_removed_like_cpp() {
        let src = "{\n \"a\": 1, // one\n /* block\n */ \"b\": \"x\"\n}";
        // the newline that ends a // comment is swallowed too (C++ quirk)
        assert_eq!(remove_comments(src), "{\n \"a\": 1,   \"b\": \"x\"\n}");
    }

    #[test]
    fn comments_inside_strings_are_also_cut() {
        assert_eq!(remove_comments("\"http://x\""), "\"http:");
    }

    #[test]
    fn localhost_variants() {
        assert!(is_localhost("LOCALHOST"));
        assert!(is_localhost("::1"));
        assert!(!is_localhost("127.0.0.2"));
    }
}
