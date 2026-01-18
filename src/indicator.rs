use std::fmt::Write;
use std::mem::MaybeUninit;
use std::{ffi::os_str::OsStr, os::windows::ffi::OsStrExt};

use windows_sys::{
    Win32::{
        Foundation::{
            CloseHandle, GENERIC_READ, GENERIC_WRITE, GetLastError, INVALID_HANDLE_VALUE,
            WIN32_ERROR,
        },
        Storage::FileSystem::{CreateFileW, FILE_SHARE_WRITE, OPEN_EXISTING},
        System::Console::{
            // 文字色
            BACKGROUND_BLUE,
            FOREGROUND_BLUE,
            FOREGROUND_GREEN,
            FOREGROUND_INTENSITY,
        },
        System::Console::{
            CONSOLE_SCREEN_BUFFER_INFO, COORD, GetConsoleScreenBufferInfo,
            SetConsoleCursorPosition, SetConsoleTextAttribute, WriteConsoleW,
        },
    },
    w,
};

struct Terminal {
    handle: *mut std::ffi::c_void,
}

impl Terminal {
    pub fn new() -> Result<Self, WIN32_ERROR> {
        unsafe {
            let handle = CreateFileW(
                w!("CONOUT$"),
                GENERIC_READ | GENERIC_WRITE,
                FILE_SHARE_WRITE,
                std::ptr::null(),
                OPEN_EXISTING,
                0,
                std::ptr::null_mut(),
            );
            if handle == INVALID_HANDLE_VALUE {
                Err(GetLastError())
            } else {
                Ok(Self { handle })
            }
        }
    }

    pub fn info(&self) -> Result<CONSOLE_SCREEN_BUFFER_INFO, WIN32_ERROR> {
        let mut info: MaybeUninit<CONSOLE_SCREEN_BUFFER_INFO> = MaybeUninit::uninit();
        unsafe {
            if GetConsoleScreenBufferInfo(self.handle, info.as_mut_ptr()) == 0 {
                Err(GetLastError())
            } else {
                Ok(info.assume_init())
            }
        }
    }

    pub fn console_width(&self) -> Result<i16, WIN32_ERROR> {
        let info = self.info()?;
        let width = info.srWindow.Right - info.srWindow.Left + 1;
        Ok(width)
    }

    pub fn set_cursor_position(&self, position: COORD) {
        unsafe {
            SetConsoleCursorPosition(self.handle, position);
        }
    }

    pub fn write(&self, s: &[u16]) -> Result<u32, WIN32_ERROR> {
        let mut count: MaybeUninit<u32> = MaybeUninit::uninit();
        unsafe {
            if WriteConsoleW(
                self.handle,
                s.as_ptr(),
                s.len() as u32,
                count.as_mut_ptr(),
                std::ptr::null(),
            ) == 0
            {
                Err(GetLastError())
            } else {
                Ok(count.assume_init())
            }
        }
    }
}

impl Drop for Terminal {
    fn drop(&mut self) {
        unsafe {
            CloseHandle(self.handle);
        }
    }
}

impl Write for Terminal {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        let wstr: Vec<u16> = OsStr::new(s).encode_wide().collect();
        self.write(wstr.as_slice()).map_err(|_| std::fmt::Error)?;
        Ok(())
    }
}

pub struct Indicator {
    terminal: Terminal,
    position: COORD,
    limit: u32,
    cursor: u32,
    original_attributes: u16,
    buffer: Vec<u16>,
}

impl Indicator {
    pub fn new(limit: u32) -> Result<Self, WIN32_ERROR> {
        let terminal = Terminal::new()?;
        let info = terminal.info()?;
        let position = info.dwCursorPosition;
        let original_attributes = info.wAttributes;
        let mut obj = Self {
            terminal,
            position,
            limit,
            cursor: 0,
            original_attributes,
            buffer: vec![],
        };
        let _ = obj.display();
        Ok(obj)
    }

    pub fn increment(&mut self) {
        self.cursor += 1;
        let _ = self.display();
    }

    pub fn display(&mut self) -> Result<(), WIN32_ERROR> {
        const BLOCKS: [char; 8] = [' ', '▏', '▎', '▍', '▌', '▋', '▊', '▉'];
        self.terminal.set_cursor_position(self.position);
        let console_width = self.terminal.console_width()?;
        let number_field = format!("] {}/{}", self.cursor, self.limit);
        let bar_length = (console_width - number_field.len() as i16 - 2) as usize;
        let current = (self.cursor as f64 / self.limit as f64) * bar_length as f64;
        let integer_part = current as usize;
        let fractional_part = ((current - integer_part as f64) * 8.0) as usize;
        let rest_part = bar_length - integer_part - usize::from(fractional_part != 0);
        self.buffer.clear();
        unsafe {
            SetConsoleTextAttribute(
                self.terminal.handle,
                FOREGROUND_BLUE | FOREGROUND_GREEN | FOREGROUND_INTENSITY | BACKGROUND_BLUE,
            );
        }
        self.buffer.push('[' as u16);
        for _ in 0..integer_part {
            self.buffer.push('█' as u16);
        }
        if fractional_part != 0 {
            self.buffer.push(BLOCKS[fractional_part] as u16);
        }
        for _ in 0..rest_part {
            self.buffer.push(' ' as u16);
        }
        for e in number_field.chars() {
            self.buffer.push(e as u16);
        }
        self.terminal.write(&self.buffer).unwrap();
        Ok(())
    }
}

impl Drop for Indicator {
    fn drop(&mut self) {
        unsafe {
            SetConsoleTextAttribute(self.terminal.handle, self.original_attributes);
            self.terminal.set_cursor_position(COORD {
                X: 0,
                Y: self.position.Y + 1,
            });
        }
    }
}
