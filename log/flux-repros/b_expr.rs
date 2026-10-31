#[cfg(flux)] extern crate flux_core;
use std::mem::MaybeUninit;
fn take(buf: &mut [MaybeUninit<u8>]) -> usize { buf.len() }
pub fn f() -> usize { take(&mut [MaybeUninit::uninit(); 8]) }
