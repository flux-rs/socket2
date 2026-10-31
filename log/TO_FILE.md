# socket2: to file — all waiting on a decision; nothing has been filed

1. **socket2 bug 1** (`BUGS.md`), to `rust-lang/socket2`: a public issue is reasonable, it is a panic
   rather than memory unsafety.
2. **Flux issues 1–4** (`FLUX_ISSUES.md`): first retest on upstream HEAD, see below. 4 is unsound, and matters most.
3. **`core::cmp::min` spec and exporting `valid`**: small flux-core PRs.

## Retesting on a current driver

Your installed driver predates #1841, so every crate fluxed with it should be re-audited.

    cd <flux checkout at upstream HEAD>   # toolchain nightly-2026-10-22 is installed
    cargo xtask install                   # replaces the installed driver: back it up first
    cd crates/socket2
    python3 ../../harness/run_flux.py --full
    python3 ../../harness/flux_canary.py
    # then the five `log/flux-repros/`

After that, revert the `view_*` helpers (`NOTES.md`, "Source changes") and bump `rust-toolchain.toml` and the lib `rev`.
