#[cfg(flux)] extern crate flux_core;
pub fn f(n: usize) { assert_eq!(n, core::mem::size_of::<usize>()); }
