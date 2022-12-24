use std::ffi::{CString};
use std::os::raw::c_char;
use std::ptr;

use generic::Backend;

#[link(name = "windows")]
extern {
	fn MessageBox(handle: u32, message: *const c_char, title: *const c_char, flags: u32) -> u32;
}

pub struct Win32Backend {
}

impl Backend for Win32Backend {
	fn init(width: usize, height: usize) -> Result<Self, String> {
		let title = CString::new("Title").unwrap();
		let message = CString::new("Message").unwrap();

		unsafe {
			MessageBox(0, message.as_ptr(), title.as_ptr(), 0);
		}
		
		Ok(Self {})
	}
}
