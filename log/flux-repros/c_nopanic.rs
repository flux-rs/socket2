#[cfg(flux)] extern crate flux_core;
pub fn f<T>(r: Result<u32, ()>) -> Result<usize, ()> {
    r.map(|n| {
        debug_assert_eq!(n as usize, core::mem::size_of::<T>());
        n as usize
    })
}
