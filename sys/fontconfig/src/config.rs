use std::{fmt, path, ptr};

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

    pub fn system_root(&self) -> Option<path::PathBuf> {
        // SAFETY: self.ptr is guaranteed to be valid
        let system_root = unsafe { bindings::FcConfigGetSysRoot(self.ptr) };

        if system_root.is_null() {
            return None;
        }

        // SAFETY: fontconfig is assumed to return valid strings
        let system_root = unsafe { bindings::to_str(system_root) };

        // FIXME: do we need to free the original string here?
        //        the fontconfig docs aren't really clear about ownership

        let mut path = path::PathBuf::new();
        path.push(system_root);
        Some(path)
    }

    pub fn best_match_for_pattern(
        &self,
        pattern: &Pattern,
    ) -> Result<Pattern, bindings::LookupError> {
        // Prepare the pattern for matching
        let success = unsafe {
            bindings::FcConfigSubstitute(
                self.ptr,
                pattern.as_ptr(),
                bindings::FcMatchKind::FcMatchFont,
            )
        };
        if !bindings::fcbool(success) {
            return Err(bindings::LookupError::OutOfMemory);
        }

        pattern.perform_default_substitutions();

        // Search for the pattern
        let mut result = bindings::FcResult::FcResultMatch;
        let ptr =
            unsafe { bindings::FcFontMatch(self.ptr, pattern.as_ptr(), ptr::addr_of_mut!(result)) };

        result.to_rust_result(|| Pattern::from_ptr(ptr))
    }
}

impl Drop for Config {
    fn drop(&mut self) {
        // Calling FcFini() here causes an assertion failure inside fontconfig
        // I don't quite understand the cause of the bug but other applications (chromium)
        // have chosen to simply live with the memory leak (https://bugs.chromium.org/p/chromium/issues/detail?id=32091)
        // I don't think this is our fault ¯\_(ツ)_/¯
        //
        // It also seems like maybe, we never should call FcFini in the first place and just rely on the refcounting from
        // the config. (https://github.com/OpenTTD/OpenTTD/pull/10916)

        // unsafe { bindings::FcFini(); }

        unsafe {
            bindings::FcConfigDestroy(self.ptr);
        }
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
