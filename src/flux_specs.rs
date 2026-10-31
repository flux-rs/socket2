//! CENTRIFUGE: flux-only refinement-level definitions and extern specs.
//! Nothing here affects compiled code.

flux_rs::defs! {
    // `p` can be read or written for `n` bytes (flux-core's `valid`, which isn't exported).
    fn readable(p: ptr, n: int) -> bool {
        p.addr != 0 && p.base <= p.addr && n <= p.size
    }

    // Bytes in a `sockaddr_storage`, which every address has to fit in.
    fn storage_size() -> int { <crate::sys::sockaddr_storage as Sized>::size_of() }

    fn mu8_size() -> int { <core::mem::MaybeUninit<u8> as Sized>::size_of() }

    // Where `sun_path` sits in a `sockaddr_un`: after a 2-byte family (Linux) or
    // length and family bytes (the BSDs).
    fn sun_path_offset() -> int { 2 }
    fn sun_path_size() -> int { <libc::sockaddr_un as Sized>::size_of() - sun_path_offset() }

    // `len` is the length of a named `sockaddr_un`: past the family, and the name,
    // with its leading (abstract) or trailing (pathname) null, fits `sun_path`.
    fn named_unix_len(len: int, abstract_name: bool) -> bool {
        sun_path_offset() < len && len - sun_path_offset() <= sun_path_size()
    }

    // `p` addresses a whole `sockaddr_storage`.
    fn covers_storage(p: ptr) -> bool { readable(p, storage_size()) }
}

#[flux_rs::extern_spec(core::slice)]
#[flux_rs::spec(fn(data: *const[@p] T, len: usize) -> &[T][len]
                requires readable(p, len * T::size_of()))]
unsafe fn from_raw_parts<'a, T>(data: *const T, len: usize) -> &'a [T];

#[flux_rs::extern_spec(core::slice)]
#[flux_rs::spec(fn(data: *mut[@p] T, len: usize) -> &mut [T][len]
                requires readable(p, len * T::size_of()))]
unsafe fn from_raw_parts_mut<'a, T>(data: *mut T, len: usize) -> &'a mut [T];

#[flux_rs::extern_spec]
#[flux_rs::assoc(fn cloned(old: int, new: int) -> bool { old == new })]
impl Clone for u32 {
    #[flux_rs::spec(fn(&u32[@x]) -> u32[x])]
    fn clone(&self) -> u32;
}

#[flux_rs::extern_spec(core::cmp)]
#[flux_rs::spec(fn(v1: T[@a], v2: T[@b]) -> T{v: <T as Ord>::min_res(a, b, v)})]
fn min<T: Ord>(v1: T, v2: T) -> T;

// `libc::iovec`, indexed by what it describes.
#[flux_rs::extern_spec(libc)]
#[flux_rs::refined_by(base: ptr, len: int)]
struct iovec {
    #[field(*mut[base] c_void)]
    iov_base: *mut c_void,
    #[field(usize[len])]
    iov_len: usize,
}

// Sockets API, per POSIX.1-2024 (`sys/socket.h`). Each spec states the buffer the
// call reads or writes. The bound on what `recv` and friends return is what the
// kernel promises: it is assumed, not checked.
use core::ffi::c_void;

#[flux_rs::extern_spec(libc)]
#[flux_rs::spec(fn(libc::c_int, buf: *mut[@p] c_void, len: usize, libc::c_int) -> isize{v: v <= len}
                requires readable(p, len))]
unsafe fn recv(socket: libc::c_int, buf: *mut c_void, len: libc::size_t, flags: libc::c_int) -> libc::ssize_t;

#[flux_rs::extern_spec(libc)]
#[flux_rs::spec(fn(libc::c_int, buf: *const[@p] c_void, len: usize, libc::c_int) -> isize{v: v <= len}
                requires readable(p, len))]
unsafe fn send(socket: libc::c_int, buf: *const c_void, len: libc::size_t, flags: libc::c_int) -> libc::ssize_t;

#[flux_rs::extern_spec(libc)]
#[flux_rs::spec(fn(libc::c_int, *const[@a] libc::sockaddr, l: u32) -> libc::c_int
                requires readable(a, l))]
unsafe fn bind(socket: libc::c_int, address: *const libc::sockaddr, address_len: libc::socklen_t) -> libc::c_int;

#[flux_rs::extern_spec(libc)]
#[flux_rs::spec(fn(libc::c_int, *const[@a] libc::sockaddr, l: u32) -> libc::c_int
                requires readable(a, l))]
unsafe fn connect(socket: libc::c_int, address: *const libc::sockaddr, len: libc::socklen_t) -> libc::c_int;

#[flux_rs::extern_spec(libc)]
#[flux_rs::spec(fn(libc::c_int, buf: *mut[@p] c_void, len: usize, libc::c_int, *mut[@a] libc::sockaddr, *mut u32) -> isize{v: v <= len}
                requires readable(p, len) && readable(a, storage_size()))]
unsafe fn recvfrom(
    socket: libc::c_int,
    buf: *mut c_void,
    len: libc::size_t,
    flags: libc::c_int,
    addr: *mut libc::sockaddr,
    addrlen: *mut libc::socklen_t,
) -> libc::ssize_t;

#[flux_rs::extern_spec(libc)]
#[flux_rs::spec(fn(libc::c_int, buf: *const[@p] c_void, len: usize, libc::c_int, *const[@a] libc::sockaddr, l: u32) -> isize{v: v <= len}
                requires readable(p, len) && readable(a, l))]
unsafe fn sendto(
    socket: libc::c_int,
    buf: *const c_void,
    len: libc::size_t,
    flags: libc::c_int,
    addr: *const libc::sockaddr,
    addrlen: libc::socklen_t,
) -> libc::ssize_t;

#[flux_rs::extern_spec(libc)]
#[flux_rs::spec(fn(libc::c_int, *mut[@a] libc::sockaddr, *mut u32) -> libc::c_int
                requires readable(a, storage_size()))]
unsafe fn accept(socket: libc::c_int, address: *mut libc::sockaddr, address_len: *mut libc::socklen_t) -> libc::c_int;

#[flux_rs::extern_spec(libc)]
#[flux_rs::spec(fn(libc::c_int, *mut[@a] libc::sockaddr, *mut u32) -> libc::c_int
                requires readable(a, storage_size()))]
unsafe fn getsockname(socket: libc::c_int, address: *mut libc::sockaddr, address_len: *mut libc::socklen_t) -> libc::c_int;

#[flux_rs::extern_spec(libc)]
#[flux_rs::spec(fn(libc::c_int, *mut[@a] libc::sockaddr, *mut u32) -> libc::c_int
                requires readable(a, storage_size()))]
unsafe fn getpeername(socket: libc::c_int, address: *mut libc::sockaddr, address_len: *mut libc::socklen_t) -> libc::c_int;

#[flux_rs::extern_spec(libc)]
#[flux_rs::spec(fn(libc::c_int, libc::c_int, libc::c_int, *const[@p] c_void, l: u32) -> libc::c_int
                requires readable(p, l))]
unsafe fn setsockopt(
    socket: libc::c_int,
    level: libc::c_int,
    name: libc::c_int,
    value: *const c_void,
    option_len: libc::socklen_t,
) -> libc::c_int;

#[flux_rs::extern_spec(libc)]
#[flux_rs::spec(fn(libc::c_int, libc::c_int, libc::c_int, *mut[@p] libc::c_int) -> libc::c_int
                requires readable(p, 2 * <libc::c_int as Sized>::size_of()))]
unsafe fn socketpair(
    domain: libc::c_int,
    type_: libc::c_int,
    protocol: libc::c_int,
    socket_vector: *mut libc::c_int,
) -> libc::c_int;
