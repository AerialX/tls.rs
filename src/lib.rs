#![no_std]
#![cfg_attr(feature = "unstable", feature(global_asm, const_fn, const_raw_ptr_to_usize_cast))]
use static_assertions::*;

use core::slice::from_raw_parts;
use core::mem::{self, size_of, MaybeUninit};

global_asm!(include_str!(concat!(env!("OUT_DIR"), "/tls.s")));
include!(concat!(env!("OUT_DIR"), "/tls.rs"));

#[repr(transparent)]
struct External(u32);

impl External {
    const fn addr(s: *const Self) -> *const () {
        s as *const _
    }

    #[cfg(feature = "unstable")]
    const fn value(s: *const Self) -> usize {
        unsafe {
            Self::addr(s) as usize
        }
    }

    #[cfg(not(feature = "unstable"))]
    fn value(s: *const Self) -> usize {
        unsafe {
            Self::addr(s) as usize
        }
    }
}

extern "C" {
    static __sitdata: External;
    static __tdata_size: External;
    static __tbss_size: External;
    static __tls_size: External;
}

#[inline]
pub fn tdata_start() -> *const u32 {
    unsafe {
        External::addr(&__sitdata) as *const _
    }
}

#[inline]
pub fn tdata_size() -> usize {
    unsafe {
        External::value(&__tdata_size)
    }
}

#[inline]
pub fn tbss_size() -> usize {
    unsafe {
        External::value(&__tbss_size)
    }
}

#[inline]
pub fn tls_size() -> usize {
    unsafe {
        External::value(&__tls_size)
    }
}

pub fn tls_length() -> usize {
    tls_size() / size_of::<u32>()
}

pub unsafe fn tls_init(tls: &mut [u32]) {
    let sdata = tls.as_mut_ptr();
    let edata = sdata.add(tdata_size() / size_of::<u32>());
    r0::init_data(sdata, edata, tdata_start());

    let ebss = edata.add(tbss_size() / size_of::<u32>());
    r0::zero_bss(edata, ebss);
}

impl ThreadBlock<[u32]> {
    pub fn init(&mut self) {
        // the linker asserts ThreadBlockStorage is big enough for TLS to fit
        unsafe {
            tls_init(&mut self.tls)
        }
    }
}

pub const TB_SIZE: usize = 1 << TB_SHIFT;
pub const TB_LENGTH: usize = TB_SIZE / size_of::<u32>();

const_assert_eq!(size_of::<ThreadBlockStorage>(), TB_SIZE);

#[repr(C)]
#[derive(Copy, Clone)]
pub struct ThreadBlock<T: ?Sized = [u32; 0]> {
    tls: T,
}

#[repr(C)]
pub union ThreadBlockStorage {
    header: ThreadBlock,
    repr: [u32; TB_LENGTH],
}

impl ThreadBlockStorage {
    pub fn header_mut(&mut self) -> &mut ThreadBlock<[u32]> {
        unsafe {
            const SZ: usize = TB_LENGTH - size_of::<ThreadBlock>() / size_of::<u32>();
            type SizedThreadBlock = ThreadBlock<[u32; SZ]>;
            let res: &mut SizedThreadBlock = mem::transmute(&mut self.header);
            res
        }
    }

    pub fn header(&self) -> &ThreadBlock {
        unsafe {
            &self.header
        }
    }
}

#[used]
#[link_section = ".__tb_size"]
pub static THREAD_BLOCK_STORAGE: MaybeUninit<ThreadBlockStorage> = MaybeUninit::uninit();

#[used]
#[link_section = ".__tb_size_header"]
pub static THREAD_BLOCK_HEADER: MaybeUninit<ThreadBlock> = MaybeUninit::uninit();

#[macro_export]
macro_rules! export_tls {
    ($id:ident[$count:expr]) => {
        #[used]
        #[export_name = "__tb_base"]
        #[link_section = ".uninit.tb"]
        pub static mut $id: [core::mem::MaybeUninit<$crate::ThreadBlockStorage>; $count] = [core::mem::MaybeUninit::uninit(); $count];
    };
}
