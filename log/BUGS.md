# socket2 bugs found while fluxing

## Bug 1: `SockAddr::as_unix` panics from safe code — CONFIRMED

`as_unix` assumes that a path from `as_pathname` is one `std` can represent, and `unwrap`s:

```rust
let path = self.as_pathname()?;
// SAFETY: we can represent this as a valid pathname, then so can the standard library.
Some(std::os::unix::net::SocketAddr::from_pathname(path).unwrap())
```

`SockAddr::unix` checks only the length, so it accepts an interior null, which `from_pathname` rejects:

```rust
SockAddr::unix("a\0b").unwrap().as_unix(); // panics
```

- Found by flux: `Result::unwrap` requires `Result[true]`. `as_unix` holds the only error of its kind.
- Repro: `tests/flux_found.rs`, `as_unix_panics_on_interior_null`, passes on macOS.
- Impact: a panic, not memory unsafety. A service that builds a path from untrusted input and calls `as_unix` can be made to panic.
- Fix: return `from_pathname(path).ok()`, or reject the null in `unix_sockaddr`.
- Upstream: no matching issue found in `rust-lang/socket2`; not reported.
- `trusted(reason = "BUG: …")` on `as_unix`.

## Unconfirmed

`path_len` always subtracts one for a trailing null. If the kernel returns a pathname that fills
`sun_path` with no null, which Linux can do for a 108-byte path, `as_pathname` would drop its last
byte. From reading the code and unix(7) only. It needs a Linux run, and isn't a flux finding.
