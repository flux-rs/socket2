# Harness notes — socket2

Target: `rust-lang/socket2` master `8b8b96f` (0.6.5), imported as a tree copy
(`93e150b`; git-subtree isn't installed here). `all` is on by default so its API is
checked. See also `ABORT.md`, `BUGS.md`, `FLUX_ISSUES.md`, `TO_FILE.md`.

## Score (2026-10-30)

Driver and libs: upstream `bd5de27` (2026-10-29), nightly-2026-10-22. The same scores held on
`6c6bdf4a00` (2026-10-18). `cargo flux` is clean on both:

| target | processed | checked | trusted | ICE | ABORT | BUG |
|---|---:|---:|---:|---:|---:|---:|
| `aarch64-apple-darwin` | 307 | 299 | 8 | 3 | 4 | 1 |
| `x86_64-unknown-linux-gnu` | 389 | 380 | 9 | 3 | 5 | 1 |

`uninit_ifname` is the Linux-only ABORT. `grep` must recurse, the harness's `src/*.rs`
misses `src/sys/`: `grep -rho 'trusted(reason = "[A-Z]*' src | sort | uniq -c`.

Windows, and the other unix targets' `cfg`s, weren't checked.

To run the harness on Linux: `CARGO_BUILD_TARGET=x86_64-unknown-linux-gnu python3 ../../harness/run_flux.py --full`.
`log/*.json` are from the default (macOS) target.

## What is verified

- `SockAddr`: `len <= size_of::<sockaddr_storage>()`, preserved by every safe constructor;
  `new` and `set_length` take it as a precondition. So `as_bytes` (`Eq`, `Hash`) is in bounds.
- `MaybeUninitSlice`, both layers: `iovec` base and length agree, so `as_slice` and `as_mut_slice` are in bounds.
- Every call to `recv`, `recvfrom`, `send`, `sendto`, `bind`, `connect`, `accept`, `getsockname`,
  `getpeername`, `setsockopt` and `socketpair` passes a buffer at least as long as the length.
  `recv` and co return at most their length: that is an extern spec about the kernel, not a proof.
- `unix_sockaddr`: the path fits `sun_path`, and the length arithmetic.
- `path_len`, `path_bytes`, `as_pathname`, `as_abstract_namespace`, given `assume_unix_len`.
- `CcidEndpoints`: `length <= N`, but see "Stale locals".
- Strict overflow checking, crate-wide.

## Not modelled

- `sendmsg`, `recvmsg` and `msghdr`: the `MsgHdr` builders aren't specified. A spec on the syscall
  can't see through `*const msghdr` anyway.
- libc `getsockopt`: its length is behind a pointer, which an extern spec can't constrain.
- Windows `MaybeUninitSlice`: the `#[field]` in `lib.rs` wouldn't resolve under flux there.

## `checked` overstates what flux checks

Three things make a SAFE weaker than it looks. Each has a repro in `log/flux-repros/`.

**Unconstrained generics (`e_generic.rs`).** A value of a generic type that appears only in the
signature's result gets a kvar nothing constrains, which is solved to `false`. Everything after
binding it is silently dead: `proves_anything` there verifies `-> i32[1]` returning 2. In socket2
that hit `view_as`, `mem::zeroed`, `ptr.cast::<T>()` then a deref, and `getsockopt::<T>()?`.
This is flux-rs/flux#907. #1841 tried to fix it and was closed unmerged. #1856 is open, and HEAD still behaves this way.

**Stale locals (`d_rawwrite.rs`).** `&mut x` coerced to a raw pointer doesn't havoc `x`. So these
hold only because flux thinks the kernel didn't write `len`, not because anything checks it:
`SockAddr::try_init`'s `new` precondition (the `debug_assert` isn't what proves it, deleting it
changes nothing), `tcp_congestion`'s slice, `device`'s, and `dccp_available_ccids`'s `CcidEndpoints` invariant.

**Debug-only facts.** flux builds with `debug_assertions`, so a `debug_assert!` reads as a fact in release too.

### `flux_canary`

`../../harness/flux_canary.py` puts a must-fail statement before every statement of every
non-trusted fn and reports those flux doesn't flag. At the score above:

| target | canaries | fired | fully checked fns | fns with silent lines |
|---|---:|---:|---:|---:|
| macOS | 956 | 523 | 259 | 8 |
| Linux | 956 | 711 | 336 | 11 |

All the remaining silent lines were read. They are `cfg`'d out on that target, or follow
`getsockopt::<T>()` (`device_index_v{4,6}`, `tcp_user_timeout`) or `&buf[..]` (`device`), with nothing to prove.
The audit doesn't cover expression-bodied closures, macro-generated fns, or test code.

## Source changes

All are behaviour-preserving and marked `CENTRIFUGE`:

- `view_in`, `view_in6`, `view_in_mut`, `view_in6_mut`, `view_as_un`, `view_as_iucv`, `view_as_vm`, and
  `SockAddrStorage::zeroed().storage` in the `From` impls: monomorphic, for #1856. Revert once that lands.
- `bool::then` to `if` in `as_pathname`, `as_abstract_namespace`: flux can't carry the guard into the closure.
- `size_of::<T>()` out of `getsockopt`'s closure: `FLUX_ISSUES.md` 3.
- `copy_path`, `uninit_ifname`, `assume_unix_len`: one assumption each, so the callers stay checked.

## Setup

- `flux_thaw.py` mustn't be run directly from a crate: it looks for `.git` in the crate dir, so it
  would `git init` there, and its `git add -A` staged everything untracked in the repo. It was run
  through a wrapper scoping both to the crate. Commits here were likewise made without `checkpoint.sh`.
- `.cargo/config.toml` is local and excluded: the sandbox forbids flux's temp file next to the libs
  in `~/.cargo/git`, so a local copy of flux is patched in. Not needed outside the sandbox.
- `run_flux.py` used to say SAFE when `cargo flux` failed before running anything: now ERROR. Its
  `detect_result` still ignores the exit code otherwise, so check for a `summary.` line.
- `#![cfg_attr(flux, allow(unused_features))]` and `#[allow(unused_extern_crates)]`: the crate denies `rust_2018_idioms`.
