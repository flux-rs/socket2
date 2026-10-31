# Flux issues from socket2

Repros: `log/flux-repros/`, each confirmed standalone on upstream `bd5de27` (2026-10-29), and the
local branch rebased onto it, `da3a598c44`. Search: issue titles and bodies, #885 to #1898.

No ICE labels remain in the crate: each is worked around. See `NOTES.md`, "Source changes".

| # | repro | symptom | upstream |
|---|---|---|---|
| 1 | `a_ena.rs` | ICE: `ena snapshot_vec.rs:199`, index out of bounds | **#1828, open**: same root cause |
| 2 | `b_expr.rs` | ICE: `rty/expr.rs:606`, index out of bounds | no match; same panic site as #1747, different trigger |
| 3 | `c_nopanic.rs` | false `may panic: MightPanic(NotInCallGraph)` | no match |
| 4 | `d_rawwrite.rs` | **unsound**: proves a stale value | no match |
| 5 | `e_generic.rs` | **unsound**: result-only generics make code dead | **#907, #1841 (closed unmerged), #1856 (open)** |
| 6 | `g_alias.rs` | false error: a refinement in a type alias's argument | no match |

1. `*mut [IoSliceMut<'_>] as *mut [Wrapper<'_>]`. Backtrace: `Ty::is_sized` from
   `checker::raw_ptr_with_size`, on a type still carrying regions. #1828 has the same panic through
   the same function, and its thread proposes erasing regions first. Socket2 adds a smaller repro
   with no trait or closure. Workaround: `slice::from_raw_parts_mut`.
2. `&mut [MaybeUninit::uninit(); 8]`. Backtrace: `Expr::proj_and_reduce` under `Reducer::fold_expr`,
   from `Node::simplify` in `execute_fixpoint_query`. #1747 panics at the same line, from an
   associated type's sort being computed as `()` while `Enumerate::Item` is a pair. There is no
   associated type here, and a root cause wasn't established, so this may or may not be the same bug.
   Workaround: `[MaybeUninit::new(0); 8]`.
3. `assert_eq!(n, size_of::<usize>())`, one line, no generic and no closure. An earlier note blamed
   closures: wrong. Binding the `size_of` to a local avoids it, and `n == size_of::<T>()` is fine.
4. `let mut x = 128; write_it(&mut x); x` is proved `u32[128]`, and `write_it` stores 99999 through
   a `*mut u32`. #1837 and #1867 are about raw pointers but concern dereference validity, not this.
5. Known, and still live on HEAD: `cast_deref`, #1841's own example shape, stays silent there. #1841
   was closed unmerged, and its successor #1856, "Switch off parametricity-by-default", is open.
6. `fn(cap: u32) -> R<u32{v: v <= cap}>` with `type R<T> = Result<T, ()>` rejects `Ok(cap)`, and spelling
   out `Result<u32{v: v <= cap}, ()>` passes. `io::Result` shows it too, so `io::Result<…>` specs can't name a
   parameter. The same spec shape, with more parameters and `io::Result`, once panicked instead, at
   `rty/binder.rs:213`, `index out of bounds`, on `getsockopt_into`: a wrong binder index would explain both,
   but that's a guess, and that panic wasn't reproduced alone.

Also, not issues: `readable` here duplicates flux-core's `valid`, and `core::cmp::min` had no spec,
only `Ord::min`. Note #1897 asks for the `from_raw_parts` specs that `flux_specs.rs` supplies.
