use std::{ffi::CString, io::{self, Write}, marker::PhantomData};

use tokio::io::BufWriter;
use libc::c_int; 

pub struct DirectIO<const N: usize, const A: usize> { 
    file_descriptor: c_int, 
    buf: [u8; N],
    n: usize,
    _p: PhantomData<*mut ()>
}

unsafe impl<const T:usize, const A: usize> std::marker::Send for DirectIO<T,A> { }


impl<const N: usize, const A: usize> Drop for DirectIO<N, A> { 
    fn drop(&mut self) {
        unsafe { 
            let i = libc::close(self.file_descriptor); 
            if i != 0{ 
                panic!("Close error: {}", std::io::Error::last_os_error())
            }
        }
    }
}

impl <const N: usize, const A: usize> DirectIO<N, A> { 
    pub fn open(path: impl AsRef<str>) -> io::Result<Self> { 
        let path = CString::new(path.as_ref()).unwrap();
        let flags = libc::O_DIRECT | libc::O_WRONLY | libc::O_CREAT | libc::O_TRUNC;
        let mode: c_int = 0o644;
        let fd = unsafe { libc::open(path.as_ptr(), flags, mode) };
        if fd == -1 {
            Err(std::io::Error::last_os_error())
        } else {
            Ok(Self {
                file_descriptor: fd,
                buf: [0u8; N],
                n: 0,
                _p: PhantomData
            })
        }
    }

    fn write_direct(&self, buf: &[u8]) -> io::Result<usize> { 
        if buf.is_empty() { 
            return Ok(0); 
        }
        let rt = unsafe { 
            libc::write(
                self.file_descriptor, 
                buf.as_ptr().cast(), 
                (buf.len() as u32).try_into().unwrap()
            )
        };
        if rt >= 0 { 
            Ok(rt as usize)
        }
        else { 
            Err(std::io::Error::last_os_error())
        }
    }
}


impl<const N: usize, const A: usize> Write for DirectIO<N, A> { 
    fn write(&mut self, mut buf: &[u8]) -> io::Result<usize> {
        let data_len = buf.len();
        if self.n != 0 {
            let end = self.n + buf.len();
            if end < self.buf.len() {
                self.buf[self.n..end].copy_from_slice(buf);
                self.n = end;
                return Ok(buf.len());
            } else {
                let r = self.buf.len() - self.n;
                self.buf[self.n..].copy_from_slice(&buf[..r]);
                let n = self.write_direct(&self.buf)?;
                assert_eq!(n, self.buf.len());
                self.n = 0;
                buf = &buf[r..];
            }
        }
        while buf.len() >= N {
            let r = buf.len() & A;
            let n = self.write_direct(&buf[..r])?;
            buf = &buf[n..];
        }
        if !buf.is_empty() {
            self.buf[0..buf.len()].copy_from_slice(buf);
            self.n = buf.len();
        }
        Ok(data_len)
    }

    fn flush(&mut self) -> io::Result<()> {
        let fd = self.file_descriptor;
        let flags: c_int = unsafe { libc::fcntl(fd, libc::F_GETFL) };
        if flags == -1 {
            return Err(std::io::Error::last_os_error());
        }
        let new_flags: c_int = flags & !libc::O_DIRECT;
        if unsafe { libc::fcntl(fd, libc::F_SETFL, new_flags) } == -1 {
            return Err(std::io::Error::last_os_error());
        }
        self.write_direct(&self.buf[..self.n]).unwrap();
        self.n = 0;
        if unsafe { libc::fcntl(fd, libc::F_SETFL, flags) } == -1 {
            return Err(std::io::Error::last_os_error());
        }
        Ok(())
    }
}