# socket2: to file — all waiting on a decision; nothing has been filed

1. **socket2 bug 1** (`BUGS.md`), to `rust-lang/socket2`: a public issue is reasonable, it is a panic
   rather than memory unsafety.
2. **Flux issues 2, 3, 4** (`FLUX_ISSUES.md`): no upstream match, reproduce on HEAD. 4 is unsound, and matters most.
3. **Flux issue 1**: comment on #1828 with `a_ena.rs`, a smaller repro.
4. **Flux issue 5**: nothing to file. #1856 is the one to watch, then revert the `view_*` helpers.
5. **`core::cmp::min` spec and exporting `valid`**: small flux-core PRs.

Crates fluxed before this may have the same dead code: run `harness/flux_canary.py` on them.
