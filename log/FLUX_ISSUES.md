# Flux issues from socket2

Repros: `log/flux-repros/`, each confirmed standalone on the installed driver, `6c6bdf4a00`
(2026-10-18), with libs `2cacd2e`.

**None has been retested on upstream HEAD.** The side build was refused in this session, so
"not fixed" below means only "reproduces on this driver". Search was by keyword over issues and PRs.

| # | repro | symptom | status |
|---|---|---|---|
| 1 | `a_ena.rs` | ICE: `ena snapshot_vec.rs:199`, index out of bounds | no match found; retest on HEAD |
| 2 | `b_expr.rs` | ICE: `rty/expr.rs:606`, index out of bounds | no match found; retest on HEAD |
| 3 | `c_nopanic.rs` | false `may panic: MightPanic(NotInCallGraph)` | no match found, #1679 is related; retest on HEAD |
| 4 | `d_rawwrite.rs` | **unsound**: proves a stale value | no match found; retest on HEAD |
| 5 | `e_generic.rs` | **unsound**: result-only generics make code dead | known: **#1841**, fixed 2026-10-19 |

1. `*mut [IoSliceMut<'_>] as *mut [Wrapper<'_>]`, cast back through `&mut *`. The unsize-free fat
   pointer cast with region-generic elements. socket2: both `Read::read_vectored`.
2. `&mut [MaybeUninit::uninit(); 8]`. `proj_and_reduce` indexes an empty tuple: `MaybeUninit` has
   `init` in flux-core, and the repeat's element seems to be indexed by unit. socket2: `peek_sender`.
3. `size_of::<T>()` for a generic `T`, in a `Result::map` closure. `Result::map` is
   `no_panic_if(F::no_panic())`, and the call site isn't resolved in the call graph. Note the
   closure contains a `debug_assert_eq!`, so it can panic anyway. Workaround: move the call out.
4. `let mut x = 128; write_it(&mut x); x` is proved `u32[128]`, and `write_it` stores 99999 through
   a `*mut u32`. Passing `&mut x` to a raw-pointer parameter doesn't invalidate `x`. An escaping
   pointer should havoc it, or the local's type should be weakened at the coercion.
5. For this crate's record only. Do not file. Notes: `NOTES.md`.

Also, not issues: `readable` here duplicates flux-core's `valid`, which isn't exported, and
`core::cmp::min` had no spec, only `Ord::min`. Both could go in flux-core.
