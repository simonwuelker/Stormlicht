use std::fmt;

use crate::{bindings, FontSet, ObjectSet, Pattern};

pub struct Config {
    ptr: *mut bindings::FcConfig,
}

impl Config {
    pub fn init() -> Self {
        log::info!("Using fontconfig version {}", Version::get());

        let config = unsafe { bindings::FcInitLoadConfigAndFonts() };

        Self { ptr: config }
    }

    pub fn matching_fonts(&self, pattern: Pattern, objects: ObjectSet) -> FontSet {
        let ptr = unsafe { bindings::FcFontList(self.ptr, pattern.as_ptr(), objects.as_ptr()) };

        // SAFETY: fontconfig is assumed to return valid font set pointers
        unsafe { FontSet::from_ptr(ptr) }
    }
}

impl Drop for Config {
    fn drop(&mut self) {
        // Note: Calling FcFini() here causes an assertion failure inside fontconfig
        // I don't quite understand the cause of the bug but other applications (chromium)
        // have chosen to simply live with the memory leak (https://bugs.chromium.org/p/chromium/issues/detail?id=32091)
        // I don't think this is our fault ¯\_(ツ)_/¯

        // unsafe { FcFini() }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Version {
    major: u8,
    minor: u8,
    revision: u8,
}

impl Version {
    pub fn get() -> Self {
        // We can see how the version value is constructed in
        // https://github.com/behdad/fontconfig/blob/5b41ded2b0ddb98a07ac86264b94403cb7a0fd82/fontconfig/fontconfig.h#L58

        // SAFETY: FcGetVersion is not unsafe
        let value = unsafe { bindings::FcGetVersion() };

        let major = (value / 10_000) as u8;
        let remaining = value % 10_000;

        let minor = (remaining / 100) as u8;
        let revision = (remaining % 100) as u8;

        Self {
            major,
            minor,
            revision,
        }
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.revision)
    }
}
