//! CENTRIFUGE: flux-only refinement-level definitions and extern specs.
//! Nothing here affects compiled code.

flux_rs::defs! {
    // `p` can be read or written for `n` bytes (flux-core's `valid`, which isn't exported).
    fn readable(p: ptr, n: int) -> bool {
        0 < p.base && p.base <= p.addr && 0 <= p.size && n <= p.size
    }
}

#[flux_rs::extern_spec(core::slice)]
#[flux_rs::spec(fn(data: *const[@p] T, len: usize) -> &[T][len]
                requires readable(p, len * T::size_of()))]
unsafe fn from_raw_parts<'a, T>(data: *const T, len: usize) -> &'a [T];

#[flux_rs::extern_spec(core::slice)]
#[flux_rs::spec(fn(data: *mut[@p] T, len: usize) -> &mut [T][len]
                requires readable(p, len * T::size_of()))]
unsafe fn from_raw_parts_mut<'a, T>(data: *mut T, len: usize) -> &'a mut [T];
