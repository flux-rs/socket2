#[cfg(flux)] extern crate flux_core;
#[flux_rs::trusted]
fn write_it(p: *mut u32) { unsafe { *p = 99999 } }
// flux proves this, and it returns 99999.
#[flux_rs::spec(fn() -> u32[128])]
pub fn f() -> u32 {
    let mut x: u32 = 128;
    write_it(&mut x);
    x
}
pub fn main_check() { assert_eq!(f(), 99999); }
