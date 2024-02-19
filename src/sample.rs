use libc::c_int;
use std::ffi::CString;
use std::io;
use std::io::Write;

const SIZE_OF_BLOCK: usize = 512;
const ALIGN_SIZE_OF_BLOCK: usize = !(SIZE_OF_BLOCK - 1);

pub struct DirectIO<const N: usize> {
    fd: c_int,
    buf: [u8; N],
    n: usize,
}

impl<const N:usize> Drop for DirectIO<N> {
    fn drop(&mut self) {
        unsafe {
            let i = libc::close(self.fd);
            if i != 0 {
                panic!("{}", io::Error::last_os_error())
            }
        }
    }
}

impl<const N:usize> DirectIO<N> {
    pub fn open(path: &str) -> io::Result<Self> {        
        let path = CString::new(path).unwrap();
        let flags = libc::O_DIRECT | libc::O_WRONLY | libc::O_CREAT | libc::O_TRUNC;
        let mode: c_int = 0o644;
        let fd = unsafe { libc::open(path.as_ptr(), flags, mode) };
        if fd == -1 {
            Err(io::Error::last_os_error())
        } else {
            Ok(Self {
                fd,
                buf: [0u8; N],
                n: 0,
            })
        }
    }

    fn write_direct(&self, buf: &[u8]) -> io::Result<usize> {
        if buf.is_empty() {
            return Ok(0);
        }
        let rt = unsafe { libc::write(self.fd, buf.as_ptr().cast(), buf.len()) };
        if rt >= 0 {
            Ok(rt as usize)
        } else {
            Err(io::Error::last_os_error())
        }
    }
}

impl<const N:usize> Write for DirectIO<N> {
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
        while buf.len() >= SIZE_OF_BLOCK {
            let r = buf.len() & ALIGN_SIZE_OF_BLOCK;
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
        let fd = self.fd;
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