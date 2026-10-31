//! CENTRIFUGE: flux-only refinement-level definitions and extern specs.
//! Nothing here affects compiled code.

flux_rs::defs! {
    // `p` can be read or written for `n` bytes (flux-core's `valid`, which isn't exported).
    fn readable(p: ptr, n: int) -> bool {
        p.addr != 0 && p.base <= p.addr && n <= p.size
    }

    // A buffer passed to a system call: null is fine for an empty one.
    fn ffi_buf(p: ptr, n: int) -> bool { n == 0 || readable(p, n) }

    // Bytes in a `sockaddr_storage`, which every address has to fit in.
    fn storage_size() -> int { <crate::sys::sockaddr_storage as Sized>::size_of() }

    fn mu8_size() -> int { <core::mem::MaybeUninit<u8> as Sized>::size_of() }

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

#[flux_rs::extern_spec]
#[flux_rs::assoc(fn cloned(old: int, new: int) -> bool { old == new })]
impl Clone for i32 {
    #[flux_rs::spec(fn(&i32[@x]) -> i32[x])]
    fn clone(&self) -> i32;
}

#[flux_rs::extern_spec(core::cmp)]
#[flux_rs::spec(fn(v1: T[@a], v2: T[@b]) -> T{v: <T as Ord>::min_res(a, b, v)})]
fn min<T: Ord>(v1: T, v2: T) -> T;

#[cfg(unix)]
flux_rs::defs! {
    // Where `sun_path` sits in a `sockaddr_un`: after a 2-byte family (Linux) or
    // length and family bytes (the BSDs).
    fn sun_path_offset() -> int { 2 }
    fn sun_path_size() -> int { <libc::sockaddr_un as Sized>::size_of() - sun_path_offset() }

    // `len` is the length of a named `sockaddr_un`: past the family, and the name,
    // with its leading (abstract) or trailing (pathname) null, fits `sun_path`.
    fn named_unix_len(len: int, abstract_name: bool) -> bool {
        sun_path_offset() < len && len - sun_path_offset() <= sun_path_size()
    }
}

#[cfg(unix)]
mod ffi {
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

    #[flux_rs::extern_spec(libc)]
    #[flux_rs::spec(fn(libc::c_int, buf: *mut[@p] c_void, len: usize, libc::c_int) -> isize{v: v <= len}
                    requires crate::flux_specs::readable(p, len))]
    unsafe fn recv(socket: libc::c_int, buf: *mut c_void, len: libc::size_t, flags: libc::c_int) -> libc::ssize_t;

    #[flux_rs::extern_spec(libc)]
    #[flux_rs::spec(fn(libc::c_int, buf: *const[@p] c_void, len: usize, libc::c_int) -> isize{v: v <= len}
                    requires crate::flux_specs::readable(p, len))]
    unsafe fn send(socket: libc::c_int, buf: *const c_void, len: libc::size_t, flags: libc::c_int) -> libc::ssize_t;

    #[flux_rs::extern_spec(libc)]
    #[flux_rs::spec(fn(libc::c_int, *const[@a] libc::sockaddr, l: u32) -> libc::c_int
                    requires crate::flux_specs::readable(a, l))]
    unsafe fn bind(socket: libc::c_int, address: *const libc::sockaddr, address_len: libc::socklen_t) -> libc::c_int;

    #[flux_rs::extern_spec(libc)]
    #[flux_rs::spec(fn(libc::c_int, *const[@a] libc::sockaddr, l: u32) -> libc::c_int
                    requires crate::flux_specs::readable(a, l))]
    unsafe fn connect(socket: libc::c_int, address: *const libc::sockaddr, len: libc::socklen_t) -> libc::c_int;

    #[flux_rs::extern_spec(libc)]
    #[flux_rs::spec(fn(libc::c_int, buf: *mut[@p] c_void, len: usize, libc::c_int, *mut[@a] libc::sockaddr, *mut u32) -> isize{v: v <= len}
                    requires crate::flux_specs::readable(p, len) && crate::flux_specs::readable(a, crate::flux_specs::storage_size()))]
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
                    requires crate::flux_specs::readable(p, len) && crate::flux_specs::readable(a, l))]
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
                    requires crate::flux_specs::readable(a, crate::flux_specs::storage_size()))]
    unsafe fn accept(socket: libc::c_int, address: *mut libc::sockaddr, address_len: *mut libc::socklen_t) -> libc::c_int;

    #[flux_rs::extern_spec(libc)]
    #[flux_rs::spec(fn(libc::c_int, *mut[@a] libc::sockaddr, *mut u32) -> libc::c_int
                    requires crate::flux_specs::readable(a, crate::flux_specs::storage_size()))]
    unsafe fn getsockname(socket: libc::c_int, address: *mut libc::sockaddr, address_len: *mut libc::socklen_t) -> libc::c_int;

    #[flux_rs::extern_spec(libc)]
    #[flux_rs::spec(fn(libc::c_int, *mut[@a] libc::sockaddr, *mut u32) -> libc::c_int
                    requires crate::flux_specs::readable(a, crate::flux_specs::storage_size()))]
    unsafe fn getpeername(socket: libc::c_int, address: *mut libc::sockaddr, address_len: *mut libc::socklen_t) -> libc::c_int;

    #[flux_rs::extern_spec(libc)]
    #[flux_rs::spec(fn(libc::c_int, libc::c_int, libc::c_int, *const[@p] c_void, l: u32) -> libc::c_int
                    requires crate::flux_specs::ffi_buf(p, l))]
    unsafe fn setsockopt(
        socket: libc::c_int,
        level: libc::c_int,
        name: libc::c_int,
        value: *const c_void,
        option_len: libc::socklen_t,
    ) -> libc::c_int;

    #[flux_rs::extern_spec(libc)]
    #[flux_rs::spec(fn(libc::c_int, libc::c_int, libc::c_int, *mut[@p] libc::c_int) -> libc::c_int
                    requires crate::flux_specs::readable(p, 2 * <libc::c_int as Sized>::size_of()))]
    unsafe fn socketpair(
        domain: libc::c_int,
        type_: libc::c_int,
        protocol: libc::c_int,
        socket_vector: *mut libc::c_int,
    ) -> libc::c_int;
}

#[cfg(windows)]
mod ffi {
    use windows_sys::Win32::Networking::WinSock::SOCKADDR;

    // `WSABUF`, indexed by what it describes.
    #[flux_rs::extern_spec(windows_sys::Win32::Networking::WinSock)]
    #[flux_rs::refined_by(len: int, base: ptr)]
    struct WSABUF {
        #[field(u32[len])]
        len: u32,
        #[field(*mut[base] u8)]
        buf: *mut u8,
    }

    // Winsock, as for `libc` on unix. The bound on what's returned is the kernel's.
    #[flux_rs::extern_spec(windows_sys::Win32::Networking::WinSock)]
    #[flux_rs::spec(fn(usize, *mut[@p] u8, len: i32, i32) -> i32{v: v <= len} requires 0 <= len && crate::flux_specs::readable(p, len))]
    unsafe fn recv(s: usize, buf: *mut u8, len: i32, flags: i32) -> i32;

    #[flux_rs::extern_spec(windows_sys::Win32::Networking::WinSock)]
    #[flux_rs::spec(fn(usize, *const[@p] u8, len: i32, i32) -> i32{v: v <= len} requires 0 <= len && crate::flux_specs::readable(p, len))]
    unsafe fn send(s: usize, buf: *const u8, len: i32, flags: i32) -> i32;

    #[flux_rs::extern_spec(windows_sys::Win32::Networking::WinSock)]
    #[flux_rs::spec(fn(usize, *const[@p] u8, len: i32, i32, *const[@a] SOCKADDR, l: i32) -> i32{v: v <= len}
                    requires 0 <= len && crate::flux_specs::readable(p, len) && crate::flux_specs::readable(a, l))]
    unsafe fn sendto(s: usize, buf: *const u8, len: i32, flags: i32, to: *const SOCKADDR, tolen: i32) -> i32;

    #[flux_rs::extern_spec(windows_sys::Win32::Networking::WinSock)]
    #[flux_rs::spec(fn(usize, *const[@a] SOCKADDR, l: i32) -> i32 requires crate::flux_specs::readable(a, l))]
    unsafe fn bind(s: usize, name: *const SOCKADDR, namelen: i32) -> i32;

    #[flux_rs::extern_spec(windows_sys::Win32::Networking::WinSock)]
    #[flux_rs::spec(fn(usize, *const[@a] SOCKADDR, l: i32) -> i32 requires crate::flux_specs::readable(a, l))]
    unsafe fn connect(s: usize, name: *const SOCKADDR, namelen: i32) -> i32;

    #[flux_rs::extern_spec(windows_sys::Win32::Networking::WinSock)]
    #[flux_rs::spec(fn(usize, i32, i32, *const[@p] u8, l: i32) -> i32 requires 0 <= l && crate::flux_specs::ffi_buf(p, l))]
    unsafe fn setsockopt(s: usize, level: i32, optname: i32, optval: *const u8, optlen: i32) -> i32;
}
