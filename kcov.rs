use std::{
    os::raw::c_void,
    sync::atomic::{AtomicU64, Ordering},
};

use nix::{errno::Errno, fcntl::OFlag, request_code_none, request_code_read, sys::stat::Mode};

pub const DEFAULT_COVER_SIZE: usize = 64 << 10;

const KCOV_INIT_TRACE: u64 = request_code_read!('c', 1, core::mem::size_of::<usize>());
const KCOV_ENABLE: u64 = request_code_none!('c', 100);
const KCOV_DISABLE: u64 = request_code_none!('c', 101);

#[derive(Clone, Copy)]
#[repr(usize)]
pub enum KcovMode {
    Pc = 0,
    Cmp = 1,
}

#[repr(C)]
pub struct CmpCoverage {
    typ: u64,
    pub arg1: u64,
    pub arg2: u64,
    pub pc: u64,
}

impl CmpCoverage {
    /// Size of the operands
    pub const fn size(&self) -> usize {
        // bit 1, 2 contain log size of input operands
        1 << ((self.typ & (3 << 1)) >> 1)
    }

    /// True if either operand is a compile time constant
    pub const fn is_const(&self) -> bool {
        (self.typ & 1) != 0
    }
}

impl core::fmt::Debug for CmpCoverage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CmpCoverage")
            .field("typ", &self.typ)
            .field("arg1", &self.arg1)
            .field("arg2", &self.arg2)
            .field("pc", &self.pc)
            .field("[size]", &self.size())
            .field("[is_const]", &self.is_const())
            .finish()
    }
}

#[derive(Debug)]
pub enum Coverage<'kcov> {
    Pc(&'kcov [u64]),
    Cmp(&'kcov [CmpCoverage]),
}

struct CoverageResult {
    collected: usize,
    mode: KcovMode,
}

pub struct Kcov {
    fd: i32,
    last_result: Option<CoverageResult>,

    map_size: usize,
    map: *const AtomicU64,
}

pub struct Collect<'kcov> {
    mode: KcovMode,
    kcov: &'kcov mut Kcov,
}

impl<'kcov> Collect<'kcov> {
    fn enable(mode: KcovMode, kcov: &'kcov mut Kcov) -> Result<Self, nix::errno::Errno> {
        unsafe {
            let res = libc::ioctl(kcov.fd, KCOV_ENABLE, mode as usize);
            if res != 0 {
                return Err(Errno::last());
            }
        }

        // reset counters
        kcov.map()[0].store(0, Ordering::Relaxed);

        Ok(Self { mode, kcov })
    }
}

impl<'kcov> Drop for Collect<'kcov> {
    fn drop(&mut self) {
        // get number of available counters
        let collected = self.kcov.map()[0].load(Ordering::Relaxed);

        unsafe {
            let res = libc::ioctl(self.kcov.fd, KCOV_DISABLE, 0);
            if res != 0 {
                panic!("KCOV_DISABLE failed: {}", Errno::last().desc());
            }
        }

        self.kcov.last_result = Some(CoverageResult {
            collected: collected as usize,
            mode: self.mode,
        });
    }
}

impl Kcov {
    pub fn new(map_size: usize) -> Result<Self, nix::errno::Errno> {
        unsafe {
            let fd = nix::fcntl::open("/sys/kernel/debug/kcov", OFlag::O_RDWR, Mode::empty())?;
            let res = libc::ioctl(fd, KCOV_INIT_TRACE, map_size);
            if res != 0 {
                return Err(Errno::last());
            }

            let map = libc::mmap(
                core::ptr::null_mut(),
                map_size * core::mem::size_of::<AtomicU64>(),
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_SHARED,
                fd,
                0,
            );
            if map == libc::MAP_FAILED {
                return Err(Errno::last());
            }

            Ok(Self {
                fd,
                map_size,
                last_result: None,
                map: map as *const AtomicU64,
            })
        }
    }

    pub fn enable(&mut self, mode: KcovMode) -> Result<Collect, nix::errno::Errno> {
        self.last_result = None;

        Collect::enable(mode, self)
    }

    fn map(&self) -> &[AtomicU64] {
        unsafe { core::slice::from_raw_parts(self.map, self.map_size) }
    }

    pub fn coverage(&self) -> Option<Coverage> {
        if let Some(res) = self.last_result.as_ref() {
            let data = unsafe { self.map.offset(1) as *const u64 };

            Some(match res.mode {
                KcovMode::Pc => unsafe {
                    Coverage::Pc(core::slice::from_raw_parts(data, res.collected))
                },
                KcovMode::Cmp => unsafe {
                    Coverage::Cmp(core::slice::from_raw_parts(
                        data as *const CmpCoverage,
                        res.collected,
                    ))
                },
            })
        } else {
            None
        }
    }

    pub const fn map_size(&self) -> usize {
        self.map_size - 1
    }
}

impl Drop for Kcov {
    fn drop(&mut self) {
        unsafe {
            let res = libc::munmap(
                self.map as *mut c_void,
                self.map_size * core::mem::size_of::<AtomicU64>(),
            );
            if res != 0 {
                panic!("munmap() failed: {}", Errno::last().desc());
            }
        }
    }
}
