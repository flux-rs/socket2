# Harness notes — socket2

Target: `rust-lang/socket2` master `8b8b96f` (0.6.5), imported as a tree copy
(`93e150b`; git-subtree isn't installed here). `all` is on by default so its API is
checked. See also `ABORT.md`, `BUGS.md`, `FLUX_ISSUES.md`, `TO_FILE.md`.

## Score (2026-10-31)

Driver: `da3a598c44`, the local `kris-gaudel/auto-strong-callgraph` rebased onto upstream `bd5de27`,
nightly-2026-10-22. Libs: `bd5de27`. `cargo flux` is clean on all of these:

| target | processed | checked | trusted | ICE | ABORT | BUG |
|---|---:|---:|---:|---:|---:|---:|
| `aarch64-apple-darwin` | 310 | 304 | 6 | 0 | 5 | 1 |
| `x86_64-unknown-linux-gnu` | 391 | 385 | 6 | 0 | 5 | 1 |
| `x86_64-unknown-linux-musl` | 386 | 380 | 6 | 0 | 5 | 1 |
| `aarch64-linux-android` | 352 | 346 | 6 | 0 | 5 | 1 |
| `x86_64-unknown-freebsd` | 313 | 307 | 6 | 0 | 5 | 1 |
| `x86_64-unknown-netbsd` | 291 | 285 | 6 | 0 | 5 | 1 |
| `x86_64-unknown-illumos` | 301 | 295 | 6 | 0 | 5 | 1 |
| `x86_64-pc-windows-msvc` | 282 | 281 | 2 | 0 | 2 | 0 |

It was 299 / 8 (3 ICE, 4 ABORT, 1 BUG) on macOS at the first pass. `checked` includes statics, as in
dlmalloc-rs, so it can exceed `processed - trusted`. `all` is on. Other `cfg`s of the crate weren't checked.
`grep` must recurse: `grep -rho 'trusted(reason = "[A-Z]*' src | sort | uniq -c`, which says 7 ABORT
across both backends, each of `offset_of_path` and `kernel_len` having one per backend.

To run on another target: `CARGO_BUILD_TARGET=<triple> python3 ../../harness/run_flux.py --full`.
`log/*.json` are from macOS. On Windows, `windows-sys` needs a `cargo fetch` outside the sandbox first.

## What is verified

- `SockAddr`: `0 <= len <= size_of::<sockaddr_storage>()`, preserved by every safe constructor;
  `new` and `set_length` take it as a precondition. So `as_bytes` (`Eq`, `Hash`) is in bounds.
- `MaybeUninitSlice`, both layers, on unix (`iovec`) and Windows (`WSABUF`): base and length agree.
- Every call to `recv`, `recvfrom`, `send`, `sendto`, `bind`, `connect`, `accept`, `getsockname`, `getpeername`,
  `setsockopt` and `socketpair` passes a buffer at least as long as the length; on Windows `recv`, `send`,
  `sendto`, `bind`, `connect` and `setsockopt` too. libc `getsockopt` goes through `getsockopt_into`, which is the same.
- Every pointer and length that goes into a `msghdr`, through `set_msghdr_name`, `_iov` and `_control`, both backends.
- On unix, `Socket::recv`, `recv_with_flags`, `send` and `send_with_flags` return at most the buffer's length,
  given the same bound on libc's, which is the kernel's promise.
- `unix_sockaddr` on both backends: the path fits `sun_path`, and the length arithmetic.
- `path_len`, `path_bytes`, `as_pathname`, `as_abstract_namespace`, given `assume_unix_len`.
- `CcidEndpoints`: `length <= N`. `device`, `tcp_congestion`: their slices are in bounds.
- Strict overflow checking, crate-wide.

## Testing

- `tests/socket.rs`, `--lib` and `flux_found`: all 67 pass on macOS, outside the sandbox, and on `140d233`,
  before these changes. 15 fail inside the sandbox on either, as socket creation is denied.
- Nothing was run on Windows or the other targets. They compile and are checked, that's all.

## Not modelled

- What's inside a `msghdr` at `sendmsg` and `recvmsg`: only what goes in is checked, a spec can't see through `*const msghdr`.
- `WSARecv`, `WSASend`, `WSARecvFrom` and `WSASendTo`: their buffers are `MaybeUninitSlice` lists, not specified.
- Windows `getsockopt`, its length is behind a pointer, and it has no wrapper like unix's.
- The `recv` bound on Windows: `syscall!` tests with an unrefined `PartialEq::eq`, so `-1` isn't ruled out.

## `checked` overstates what flux checks

Three things make a SAFE weaker than it looks. Each has a repro in `log/flux-repros/`.

**Unconstrained generics (`e_generic.rs`).** A value of a generic type that appears only in the
signature's result gets a kvar nothing constrains, which is solved to `false`. Everything after
binding it is silently dead: `proves_anything` there verifies `-> i32[1]` returning 2. In socket2
that hit `view_as`, `mem::zeroed`, `ptr.cast::<T>()` then a deref, and `getsockopt::<T>()?`.
This is flux-rs/flux#907. #1841 tried to fix it and was closed unmerged. #1856 is open, and HEAD still behaves this way.

**Stale locals (`d_rawwrite.rs`).** `&mut x` coerced to a raw pointer doesn't havoc `x`. So at the first pass
`try_init`, `tcp_congestion`, `device` and `dccp_available_ccids` held only because flux thought the kernel
didn't write `len`. No longer: `getsockopt_into` takes a capacity and returns the length, and `kernel_len`
states the kernel's promise, and forgets the old value. Any other out-parameter is still a risk.

**Debug-only facts.** flux builds with `debug_assertions`, so a `debug_assert!` reads as a fact in release too.

### `flux_canary`

| target | canaries | fired | fully checked fns | fns with silent lines |
|---|---:|---:|---:|---:|
| macOS | 975 | 540 | 264 | 8 |
| Linux | 975 | 732 | 342 | 10 |
| Windows | 975 | 523 | 248 | 3 |

All the remaining silent lines were read. They are `cfg`'d out on that target, or follow `getsockopt::<T>()`
(`device_index_v{4,6}`, `tcp_user_timeout`), or on Windows `*uninit.as_ptr().cast::<u8>()` in `assume_init`, with
nothing to prove. This found Windows' `unix_sockaddr` 13/17 silent, the same trap as unix, now fixed.
The audit doesn't cover expression-bodied closures, macro-generated fns, or test code.
FreeBSD, Android, musl, NetBSD and illumos were swept, not audited.

## Source changes

All are behaviour-preserving and marked `CENTRIFUGE`:

- `view_in`, `view_in6`, `view_in_mut`, `view_in6_mut`, `view_as_un`, `view_as_iucv`, `view_as_vm`, and
  `SockAddrStorage::zeroed().storage` in the `From` impls: monomorphic, for #1856. Revert once that lands.
- `bool::then` to `if` in `as_pathname`, `as_abstract_namespace`: flux can't carry the guard into the closure.
- `size_of::<T>()` out of `assert_eq!`: `FLUX_ISSUES.md` 3.
- `copy_path`, `assume_unix_len`, `kernel_usec`, `kernel_len`: one assumption each, so the callers stay checked.
- `getsockopt_into`: libc `getsockopt` with its in-out length split into a capacity and a result.
- `from_raw_parts_mut` in `read_vectored`, for a slice pointer cast, which is #1828. The same memory and length.
- `[MaybeUninit::new(0); N]` in `peek_sender` and `device`, for `uninit`, which crashes flux (`b_expr.rs`). **Not
  strictly the same**: these 8 and `IFNAMSIZ` bytes are now zeroed first. Nothing reads them before the kernel writes.
- `size_of::<MaybeUninit<T>>()` in unix `getsockopt`, the same as `size_of::<T>()`.
- Windows: `view_as_un`, `offset_of_path`, `Control` set as one `WSABUF`, `size_of` out of `debug_assert_eq!`.

## Setup

- `run_flux.py --make-targets` needs `-Fdump-call-graph`, which is only on the local flux branch, not upstream.

- `flux_thaw.py` mustn't be run directly from a crate: it looks for `.git` in the crate dir, so it
  would `git init` there, and its `git add -A` staged everything untracked in the repo. It was run
  through a wrapper scoping both to the crate. Commits here were likewise made without `checkpoint.sh`.
- `.cargo/config.toml` is local and excluded: the sandbox forbids flux's temp file next to the libs
  in `~/.cargo/git`, so a local copy of flux is patched in. Not needed outside the sandbox.
- `run_flux.py` used to say SAFE when `cargo flux` failed before running anything: now ERROR. Its
  `detect_result` still ignores the exit code otherwise, so check for a `summary.` line.
- `#![cfg_attr(flux, allow(unused_features))]` and `#[allow(unused_extern_crates)]`: the crate denies `rust_2018_idioms`.
