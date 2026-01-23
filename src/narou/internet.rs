#![allow(dead_code)]
use std::convert::From;
use std::ffi::c_void;
use std::ptr::null;
use std::str::Utf8Error;
use windows_sys::Win32::Foundation::{ERROR_INSUFFICIENT_BUFFER, GetLastError, WIN32_ERROR};
use windows_sys::Win32::Networking::WinInet::*;

#[derive(Debug)]
pub enum Error {
    SystemErrorCode(u32),
    InvalidCharCode,
    BadStatus(u32),
}

pub type Result<T> = std::result::Result<T, Error>;

impl From<WIN32_ERROR> for Error {
    fn from(value: WIN32_ERROR) -> Self {
        Error::SystemErrorCode(value)
    }
}

impl From<Utf8Error> for Error {
    fn from(_: Utf8Error) -> Self {
        Error::InvalidCharCode
    }
}

pub struct Internet {
    handle: *mut c_void,
}

impl Internet {
    pub fn new() -> Result<Self> {
        unsafe {
            let handle = InternetOpenA(
                super::AGENT_NAME.as_ptr(),
                INTERNET_OPEN_TYPE_PRECONFIG,
                null(),
                null(),
                0,
            );
            if handle.is_null() {
                Err(GetLastError())?
            } else {
                Ok(Internet { handle })
            }
        }
    }

    pub fn open(&self, url: &str) -> Result<Response> {
        unsafe {
            let handle = InternetOpenUrlA(
                self.handle,
                format!("{}\0", url).as_ptr(),
                null(),
                0,
                INTERNET_FLAG_RELOAD | INTERNET_FLAG_SECURE | INTERNET_FLAG_NO_AUTO_REDIRECT,
                0,
            );
            if handle.is_null() {
                Err(GetLastError())?
            } else {
                Ok(Response { handle })
            }
        }
    }
}

pub struct Response {
    handle: *mut c_void,
}

impl Drop for Internet {
    fn drop(&mut self) {
        unsafe {
            InternetCloseHandle(self.handle);
        }
    }
}

impl Drop for Response {
    fn drop(&mut self) {
        unsafe {
            InternetCloseHandle(self.handle);
        }
    }
}

impl std::io::Read for Response {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        unsafe {
            let mut bytes_read = 0;
            if InternetReadFile(
                self.handle,
                buf.as_mut_ptr() as _,
                buf.len() as u32,
                &mut bytes_read,
            ) == 0
            {
                Err(std::io::Error::last_os_error())
            } else {
                Ok(bytes_read as usize)
            }
        }
    }
}

#[repr(u32)]
#[derive(Clone, Copy)]
pub enum Query {
    Location = HTTP_QUERY_LOCATION,
}

impl Response {
    pub fn status_code(&self) -> Result<u32> {
        unsafe {
            let mut status: u32 = 0;
            let mut buflen: u32 = std::mem::size_of::<u32>() as _;
            if HttpQueryInfoA(
                self.handle,
                HTTP_QUERY_STATUS_CODE | HTTP_QUERY_FLAG_NUMBER,
                &mut status as *mut u32 as _,
                &mut buflen as *mut u32 as _,
                std::ptr::null_mut(),
            ) == 0
            {
                Err(GetLastError())?
            } else {
                Ok(status)
            }
        }
    }

    pub fn header(&self, query: Query) -> Result<String> {
        unsafe {
            let mut buffer = vec![0; 100];
            let mut buflen = buffer.len() as u32;

            'b: loop {
                if HttpQueryInfoA(
                    self.handle,
                    query as _,
                    buffer.as_mut_ptr() as _,
                    &mut buflen as *mut u32 as _,
                    std::ptr::null_mut(),
                ) == 0
                {
                    let error_code = GetLastError();
                    if error_code == ERROR_INSUFFICIENT_BUFFER {
                        buffer.resize(buflen as usize, 0);
                        continue;
                    } else {
                        Err(error_code)?;
                    }
                } else {
                    // HttpQueryInfoA が返す文字列は UTF-8 ではないが
                    // 今回の用途ではアスキーの範囲内のため雑に処理
                    break 'b Ok(std::str::from_utf8(&buffer[..buflen as usize])?.to_string());
                }
            }
        }
    }

    pub fn error_for_status(self) -> Result<Self> {
        let code = self.status_code()?;
        if code == 200 {
            Ok(self)
        } else {
            Err(Error::BadStatus(code))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() -> Result<()> {
        let internet = Internet::new()?;
        let response = internet.open("https://x.gd/3ZG6F")?;
        assert_eq!(response.status_code()?, 301);
        assert_eq!(response.header(Query::Location)?, "https://example.com/");
        Ok(())
    }
}
