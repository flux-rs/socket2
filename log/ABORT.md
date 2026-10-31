# ABORT log — socket2

Every entry is a `#[flux_rs::trusted(reason = "ABORT: …")]` on a small helper, so the callers stay checked.

| § | category | fn | status |
|---|---|---|---|
| 1 | flux can't relate addresses | `offset_of_path` | ABORT |
| 2 | `nonoverlapping` across allocations | `copy_path` | ABORT |
| 3 | fact about runtime data | `assume_unix_len` | ABORT |
| 4 | flux false positive | `getsockopt` | DONE, not trusted |
| 5 | fact about the kernel | `from_timeval` | ABORT |
| 6 | flux-core spec too strong | `uninit_ifname` (Linux only) | ABORT |

## §1 `sys::offset_of_path` (src/sys/unix.rs)

- Error: `arithmetic operation may underflow` on `path - base`.
- Cause: `base` and `path` are `addr_of!`s of a struct and its field, and flux gives each a fresh
  address. Nothing says a field's address is at or after its struct's.
- Tried: precondition (nothing to state, both are locals); `refined_by` (`sockaddr_un` is a libc
  struct, and offsets aren't indices); callee spec (`addr_of!` is a MIR rvalue, not a call).
  `mem::offset_of!` has no refinement either.
- Assumed: `-> usize[2]`, `flux_specs::sun_path_offset`. `tests/flux_found.rs` checks that on the
  host. It holds for Linux's `sa_family_t` and the BSDs' `sun_len` plus `sun_family`.

## §2 `sys::copy_path` (src/sys/unix.rs)

- Error: `a precondition cannot be proved`, flux-core's `nonoverlapping` on `ptr::copy_nonoverlapping`.
- Cause: the same as `crates/lz4_flex/log/ABORT.md`: flux compares addresses, and a slice's has
  no relation to a `&mut`'s. A `&[u8]` carries no address to state disjointness with.
- Tried: precondition (nothing to name); helper taking both references, which keeps `n <= m` checked at the
  one caller. `valid` and alignment hold there, only `nonoverlapping` is assumed.

## §3 `SockAddr::assume_unix_len` (src/sys/unix.rs)

- Error, before the helper: `a precondition cannot be proved`, `named_unix_len` at `path_bytes`,
  in `as_pathname` and `as_abstract_namespace`.
- Cause: `path_bytes` reads through a `&sockaddr_un`, so `len` must fit that, not just the storage.
  That follows from `ss_family == AF_UNIX`, which is in `sockaddr_storage`'s bytes, and written
  through `view_as` or by the kernel. It can't be an index.
- Tried: `refined_by` a family (no field to index, `storage` is opaque bytes); `requires` (both
  fns are safe and public); splitting `then` into an `if`, which fixed the guard half of this.
- Assumed: `len <= size_of::<sockaddr_un>()` for a named `AF_UNIX` address. This is `SockAddr::new`'s
  "the address family and length match the type of storage address". Only an unsafe caller breaking
  that can make it false, and then the read stays inside `storage`.

## §4 `sys::getsockopt` — DONE

- Error: `call to core::mem::size_of may panic: MightPanic(NotInCallGraph)` at the
  `debug_assert_eq!` in the `.map` closure.
- Fixed by moving `size_of::<T>()` out of the closure. See `FLUX_ISSUES.md` 3.

## §5 `sys::from_timeval` (src/sys/unix.rs)

- Error: `arithmetic operation may overflow` on `(tv_usec as u32) * 1000`.
- Cause: true only if `tv_usec < 1_000_000`, which comes from `getsockopt` through a raw pointer.
- Tried: precondition (`timeout_opt`'s `.map(from_timeval)` has nothing to discharge it with: libc
  `getsockopt`'s out-value can't be specified); refining `timeval` (same reason).
- Not a bug unless the kernel misbehaves. Overflow is a panic in debug and wraps in release.

## §6 `sys::uninit_ifname` (src/sys/unix.rs, Linux only)

- Error: `a precondition cannot be proved`, `MaybeUninit<T>[true]` at `MaybeUninit::uninit().assume_init()`.
- Cause: flux-core's `assume_init` requires `init`, but an uninitialised `[MaybeUninit<u8>; N]` is fine.
- Tried: `[MaybeUninit::uninit(); N]` would be correct and hits the ICE in `FLUX_ISSUES.md` 2.
