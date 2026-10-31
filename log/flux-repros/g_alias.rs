#[cfg(flux)] extern crate flux_core;
type R<T> = Result<T, ()>;
// `n` should be 1: a local alias shows the same failure.
#[flux_rs::spec(fn(cap: u32) -> R<u32{v: v <= cap}>)]
pub fn f(cap: u32) -> R<u32> { Ok(cap) }
