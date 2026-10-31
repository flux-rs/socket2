#[cfg(flux)] extern crate flux_core;
use std::io::IoSliceMut;
pub struct Wrapper<'a>(&'a mut [u8]);
pub fn f(bufs: &mut [IoSliceMut<'_>]) -> usize {
    let bufs = unsafe { &mut *(bufs as *mut [IoSliceMut<'_>] as *mut [Wrapper<'_>]) };
    bufs.len()
}
