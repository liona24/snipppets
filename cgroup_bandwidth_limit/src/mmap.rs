use std::marker::PhantomData;
use std::os::fd::{AsRawFd, BorrowedFd};
use std::ptr::NonNull;

pub struct Mmap<'fd, T> {
    ptr: NonNull<T>,
    _marker: PhantomData<&'fd ()>,
}

macro_rules! align_up {
    ($x:expr, $align:expr) => {
        ($x + ($align - 1)) & !($align - 1)
    };
}

impl<'fd, T> Mmap<'fd, T> {
    const fn len() -> usize {
        align_up!(core::mem::size_of::<T>(), 0x1000)
    }

    pub unsafe fn new(fd: BorrowedFd<'fd>) -> Result<Self, ()> {
        // note that size_of::<*const T>() == size_of::<usize>() because T: Sized

        let len = Self::len();
        let ptr = libc::mmap(
            core::ptr::null_mut(),
            len,
            libc::PROT_READ | libc::PROT_WRITE,
            libc::MAP_SHARED,
            fd.as_raw_fd(),
            0,
        );
        match ptr {
            libc::MAP_FAILED => Err(()),
            ptr => Ok(Self {
                ptr: NonNull::new_unchecked(ptr as *mut T),
                _marker: PhantomData,
            }),
        }
    }
}

impl<'fd, T> Drop for Mmap<'fd, T> {
    fn drop(&mut self) {
        unsafe {
            let _error = libc::munmap(self.ptr.as_ptr() as *mut _, Self::len());
            // ignore the error
        }
    }
}

impl<'fd, T> core::ops::Deref for Mmap<'fd, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { self.ptr.as_ref() }
    }
}
impl<'fd, T> core::ops::DerefMut for Mmap<'fd, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.ptr.as_mut() }
    }
}
