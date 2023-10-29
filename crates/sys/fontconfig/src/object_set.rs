use std::ffi;

use crate::bindings;

pub struct ObjectSet {
    ptr: *mut bindings::FcObjectSet,
}

#[derive(Clone, Copy, Debug)]
pub struct Object {
    inner: &'static ffi::CStr,
}

impl Object {
    #[inline]
    pub(crate) fn as_ptr(&self) -> *const ffi::c_char {
        self.inner.as_ptr()
    }
}

pub mod objects {
    use std::ffi;

    use super::Object;

    macro_rules! object {
        ( $s:literal ) => {{
            let inner = match ffi::CStr::from_bytes_with_nul(concat!($s, "\0").as_bytes()) {
                Ok(value) => value,
                Err(_) => panic!("null byte in object identifier"),
            };

            Object { inner }
        }};
    }

    pub const FC_FAMILY: Object = object!("family");
    pub const FC_STYLE: Object = object!("style");
    pub const FC_SLANT: Object = object!("slant");
    pub const FC_WEIGHT: Object = object!("weight");
    pub const FC_SIZE: Object = object!("size");
    pub const FC_ASPECT: Object = object!("aspect");
    pub const FC_PIXEL_SIZE: Object = object!("pixelsize");
    pub const FC_SPACING: Object = object!("spacing");
    pub const FC_FOUNDRY: Object = object!("foundry");
    pub const FC_ANTIALIAS: Object = object!("antialias");
    pub const FC_HINTING: Object = object!("hinting");
    pub const FC_HINT_STYLE: Object = object!("hintstyle");
    pub const FC_VERTICAL_LAYOUT: Object = object!("verticallayout");
    pub const FC_AUTOHINT: Object = object!("autohint");
    pub const FC_WIDTH: Object = object!("width");
    pub const FC_FILE: Object = object!("file");
    pub const FC_INDEX: Object = object!("index");
    pub const FC_FT_FACE: Object = object!("ftface");
    pub const FC_RASTERIZER: Object = object!("rasterizer");
    pub const FC_OUTLINE: Object = object!("outline");
    pub const FC_SCALABLE: Object = object!("scalable");
    pub const FC_COLOR: Object = object!("color");
    pub const FC_VARIABLE: Object = object!("variable");
    pub const FC_SCALE: Object = object!("scale");
    pub const FC_SYMBOL: Object = object!("symbol");
    pub const FC_DPI: Object = object!("dpi");
    pub const FC_RGBA: Object = object!("rgba");
    pub const FC_MINSPACE: Object = object!("minspace");
    pub const FC_SOURCE: Object = object!("source");
    pub const FC_CHARSET: Object = object!("charset");
    pub const FC_LANG: Object = object!("lang");
    pub const FC_FONTVERSION: Object = object!("FC_FONTVERSION");
    pub const FC_FULLNAME: Object = object!("fullname");
    pub const FC_FAMILYLANG: Object = object!("familylang");
    pub const FC_STYLELANG: Object = object!("stylelang");
    pub const FC_FULLNAMELANG: Object = object!("fullnamelang");
    pub const FC_CAPABILITY: Object = object!("capability");
    pub const FC_FONTFORMAT: Object = object!("fontformat");
    pub const FC_EMBOLDEN: Object = object!("embolden");
    pub const FC_EMBEDDED_BITMAP: Object = object!("embeddedbitmap");
    pub const FC_DECORATIVE: Object = object!("decorative");
    pub const FC_LCD_FILTER: Object = object!("lcdfilter");
    pub const FC_FONT_FEATURES: Object = object!("fontfeatures");
    pub const FC_FONT_VARIATIONS: Object = object!("fontvariations");
    pub const FC_NAMELANG: Object = object!("namelang");
    pub const FC_PRGNAME: Object = object!("prgname");
    pub const FC_HASH: Object = object!("hash");
    pub const FC_POSTSCRIPT_NAME: Object = object!("postscriptname");
}

impl ObjectSet {
    pub fn add_object(&mut self, object: Object) -> &mut Self {
        let ident = object.as_ptr();

        // SAFETY: ident is guaranteed to be a valid object
        let value = unsafe { bindings::FcObjectSetAdd(self.ptr, ident) };

        if !bindings::fcbool(value) {
            panic!("Out of memory");
        }
        self
    }

    pub(crate) fn as_ptr(&self) -> *mut bindings::FcObjectSet {
        self.ptr
    }
}

impl Default for ObjectSet {
    fn default() -> Self {
        // SAFETY: FcObjectSetCreate is not unsafe
        let ptr = unsafe { bindings::FcObjectSetCreate() };

        Self { ptr }
    }
}

impl Drop for ObjectSet {
    fn drop(&mut self) {
        unsafe { bindings::FcObjectSetDestroy(self.ptr) }
    }
}
