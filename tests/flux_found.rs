//! CENTRIFUGE: reproductions of what fluxing socket2 turned up. See log/BUGS.md.
#![cfg(all(unix, not(target_os = "wasi")))]

use socket2::SockAddr;

/// BUGS.md bug 1. `SockAddr::unix` accepts an interior null, which
/// `SocketAddr::from_pathname` rejects, so `as_unix`'s `unwrap` panics.
#[test]
#[should_panic(expected = "paths must not contain interior null bytes")]
fn as_unix_panics_on_interior_null() {
    let addr = SockAddr::unix("a\0b").unwrap();
    let _ = addr.as_unix();
}
