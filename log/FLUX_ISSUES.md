# Flux issues from socket2

Repros: `log/flux-repros/`, each confirmed standalone on both the old driver, `6c6bdf4a00`
(2026-10-18), and upstream HEAD, `bd5de27` (2026-10-29). Issue text was searched over the 200
most recent issues and PRs, #1690 up: older ones weren't, the download failed past that.

| # | repro | symptom | upstream |
|---|---|---|---|
| 1 | `a_ena.rs` | ICE: `ena snapshot_vec.rs:199`, index out of bounds | **#1828, open**: same root cause |
| 2 | `b_expr.rs` | ICE: `rty/expr.rs:606`, index out of bounds | no match; same panic site as #1747, different trigger |
| 3 | `c_nopanic.rs` | false `may panic: MightPanic(NotInCallGraph)` | no match |
| 4 | `d_rawwrite.rs` | **unsound**: proves a stale value | no match |
| 5 | `e_generic.rs` | **unsound**: result-only generics make code dead | **#907, #1841 (closed unmerged), #1856 (open)** |

1. `*mut [IoSliceMut<'_>] as *mut [Wrapper<'_>]`. Backtrace: `Ty::is_sized` from
   `checker::raw_ptr_with_size`, on a type still carrying regions. #1828 has the same panic through
   the same function, and its thread proposes erasing regions first. Socket2 adds a smaller repro
   with no trait or closure. Both `Read::read_vectored`.
2. `&mut [MaybeUninit::uninit(); 8]`. Backtrace: `Expr::proj_and_reduce` under `Reducer::fold_expr`,
   from `Node::simplify` in `execute_fixpoint_query`. #1747 panics at the same line, from an
   associated type's sort being computed as `()` while `Enumerate::Item` is a pair. There is no
   associated type here, and a root cause wasn't established, so this may or may not be the same bug.
   `peek_sender`.
3. `size_of::<T>()` for a generic `T`, in a `Result::map` closure. Nothing in the issue text mentions
   `NotInCallGraph` or `MightPanic`. #1679 is about reporting panic causes, not this. Workaround: move the call out.
4. `let mut x = 128; write_it(&mut x); x` is proved `u32[128]`, and `write_it` stores 99999 through
   a `*mut u32`. #1837 and #1867 are about raw pointers but concern dereference validity, not this.
5. Known, and still live on HEAD: `cast_deref`, #1841's own example shape, stays silent there. #1841
   was closed unmerged, and its successor #1856, "Switch off parametricity-by-default", is open.

Also, not issues: `readable` here duplicates flux-core's `valid`, and `core::cmp::min` had no spec,
only `Ord::min`. Note #1897 asks for the `from_raw_parts` specs that `flux_specs.rs` supplies.
