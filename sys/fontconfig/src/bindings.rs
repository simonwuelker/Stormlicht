// See https://www.freedesktop.org/software/fontconfig/fontconfig-devel/
// for documentation of the library API

use std::{ffi, iter::FusedIterator, marker::PhantomData};

pub type FcChar8 = i8;
pub type FcChar16 = u16;
pub type FcChar32 = u32;
pub type FcBool = u8;

#[repr(C)]
pub struct FcMatrix {
    xx: ffi::c_double,
    xy: ffi::c_double,
    yx: ffi::c_double,
    yy: ffi::c_double,
}

#[repr(C)]
pub struct FcValue {
    type_tag: FcType,
    content: FcValueContents,
}

#[repr(C)]
pub union FcValueContents {
    s: *const FcChar8,
    i: ffi::c_int,
    b: FcBool,
    d: ffi::c_double,
    m: *const FcMatrix,
    c: *const FcCharSet,
    f: *const ffi::c_void,
    l: *const FcLangSet,
    r: *const FcRange,
}

#[repr(C)]
pub struct Set<T> {
    pub(crate) num_values: ffi::c_int,
    pub(crate) s_value: ffi::c_int,
    pub(crate) value: *mut T,
}

pub type FcFontSet = Set<*mut FcPattern>;
pub type FcObjectSet = Set<*const ffi::c_char>;

#[repr(C)]
pub struct FcObjectType {
    object: *const ffi::c_char,
    object_type: FcType,
}

#[repr(C)]
pub enum FcType {
    FcTypeUnknown = -1,
    FcTypeVoid,
    FcTypeInteger,
    FcTypeDouble,
    FcTypeString,
    FcTypeBool,
    FcTypeMatrix,
    FcTypeCharSet,
    FcTypeFTFace,
    FcTypeLangSet,
    FcTypeRange,
}

#[repr(C)]
pub enum FcResult {
    FcResultMatch,
    FcResultNoMatch,
    FcResultTypeMismatch,
    FcResultNoId,
    FcResultOutOfMemory,
}

#[repr(C)]
pub enum FcMatchKind {
    FcMatchPattern, // FcMatchKindBegin
    FcMatchFont,
    FcMatchScan,
    FcMatchKindEnd,
}

#[repr(C)]
pub enum FcSetName {
    FcSetSystem,
    FcSetApplication,
}

#[allow(non_camel_case_types)]
pub type FT_Face = *mut FT_FaceRec_;

#[link(name = "fontconfig")]
#[allow(dead_code)]
extern "C" {
    pub type FcCharSet;
    pub type FcLangSet;
    pub type FcLangResult;
    pub type FcConfig;
    pub type FcPattern;
    pub type FcPatternIter;
    pub type FcRange;
    pub type FcValueBinding;
    pub type FT_FaceRec_;
    pub type FcStrSet;
    pub type FcStrList;
    pub type FcConfigFileInfoIter;

    pub static FC_FAMILY: *const ffi::c_char;
    pub static FC_STYLE: *const ffi::c_char;
    pub static FC_SLANT: *const ffi::c_char;
    pub static FC_WEIGHT: *const ffi::c_char;
    pub static FC_SIZE: *const ffi::c_char;
    pub static FC_ASPECT: *const ffi::c_char;
    pub static FC_PIXEL_SIZE: *const ffi::c_char;
    pub static FC_SPACING: *const ffi::c_char;
    pub static FC_FOUNDRY: *const ffi::c_char;
    pub static FC_ANTIALIAS: *const ffi::c_char;
    pub static FC_HINTING: *const ffi::c_char;
    pub static FC_HINT_STYLE: *const ffi::c_char;
    pub static FC_VERTICAL_LAYOUT: *const ffi::c_char;
    pub static FC_AUTOHINT: *const ffi::c_char;
    pub static FC_WIDTH: *const ffi::c_char;
    pub static FC_FILE: *const ffi::c_char;
    pub static FC_INDEX: *const ffi::c_char;
    pub static FC_FT_FACE: *const ffi::c_char;
    pub static FC_RASTERIZER: *const ffi::c_char;
    pub static FC_OUTLINE: *const ffi::c_char;
    pub static FC_SCALABLE: *const ffi::c_char;
    pub static FC_COLOR: *const ffi::c_char;
    pub static FC_VARIABLE: *const ffi::c_char;
    pub static FC_SCALE: *const ffi::c_char;
    pub static FC_SYMBOL: *const ffi::c_char;
    pub static FC_DPI: *const ffi::c_char;
    pub static FC_RGBA: *const ffi::c_char;
    pub static FC_MINSPACE: *const ffi::c_char;
    pub static FC_SOURCE: *const ffi::c_char;
    pub static FC_CHARSET: *const ffi::c_char;
    pub static FC_LANG: *const ffi::c_char;
    pub static FC_FONTVERSION: *const ffi::c_char;
    pub static FC_FULLNAME: *const ffi::c_char;
    pub static FC_FAMILYLANG: *const ffi::c_char;
    pub static FC_STYLELANG: *const ffi::c_char;
    pub static FC_FULLNAMELANG: *const ffi::c_char;
    pub static FC_CAPABILITY: *const ffi::c_char;
    pub static FC_FONTFORMAT: *const ffi::c_char;
    pub static FC_EMBOLDEN: *const ffi::c_char;
    pub static FC_EMBEDDED_BITMAP: *const ffi::c_char;
    pub static FC_DECORATIVE: *const ffi::c_char;
    pub static FC_LCD_FILTER: *const ffi::c_char;
    pub static FC_FONT_FEATURES: *const ffi::c_char;
    pub static FC_FONT_VARIATIONS: *const ffi::c_char;
    pub static FC_NAMELANG: *const ffi::c_char;
    pub static FC_PRGNAME: *const ffi::c_char;
    pub static FC_HASH: *const ffi::c_char;
    pub static FC_POSTSCRIPT_NAME: *const ffi::c_char;

    pub static FC_CACHE_SUFFIX: *const ffi::c_char;
    pub static FC_DIR_CACHE_FILE: *const ffi::c_char;
    pub static FC_USER_CACHE_FILE: *const ffi::c_char;

    /// <https://www.freedesktop.org/software/fontconfig/fontconfig-devel/fcinitloadconfig.html>
    pub fn FcInitLoadConfig() -> *mut FcConfig;

    /// <https://www.freedesktop.org/software/fontconfig/fontconfig-devel/fcinitloadconfigandfonts.html>
    pub fn FcInitLoadConfigAndFonts() -> *mut FcConfig;

    /// <https://www.freedesktop.org/software/fontconfig/fontconfig-devel/fcinit.html>
    pub fn FcInit() -> FcBool;

    /// <https://www.freedesktop.org/software/fontconfig/fontconfig-devel/fcfini.html>
    pub fn FcFini();

    /// <https://www.freedesktop.org/software/fontconfig/fontconfig-devel/fcgetversion.html>
    pub fn FcGetVersion() -> ffi::c_int;

    /// <https://www.freedesktop.org/software/fontconfig/fontconfig-devel/fcinitreinitialize.html>
    pub fn FcInitReinitialize() -> FcBool;

    /// <https://www.freedesktop.org/software/fontconfig/fontconfig-devel/fcinitbringuptodate.html>
    pub fn FcInitBringUptoDate() -> FcBool;

    /// <https://www.freedesktop.org/software/fontconfig/fontconfig-devel/fcpatterncreate.html>
    pub fn FcPatternCreate() -> *mut FcPattern;

    /// <https://www.freedesktop.org/software/fontconfig/fontconfig-devel/fcpatternduplicate.html>
    pub fn FcPatternDuplicate(pattern: *const FcPattern) -> *mut FcPattern;

    /// <https://www.freedesktop.org/software/fontconfig/fontconfig-devel/fcpatternreference.html>
    pub fn FcPatternReference(pattern: *const FcPattern);

    /// <https://www.freedesktop.org/software/fontconfig/fontconfig-devel/fcpatterndestroy.html>
    pub fn FcPatternDestroy(pattern: *const FcPattern);

    /// <https://www.freedesktop.org/software/fontconfig/fontconfig-devel/fcpatternobjectcount.html>
    pub fn FcPatternObjectCount(pattern: *const FcPattern) -> ffi::c_int;

    /// <https://www.freedesktop.org/software/fontconfig/fontconfig-devel/fcpatternequal.html>
    pub fn FcPatternEqual(pa: *const FcPattern, pb: *const FcPattern) -> FcBool;

    /// <https://www.freedesktop.org/software/fontconfig/fontconfig-devel/fcpatternequalsubset.html>
    pub fn FcPatternEqualSubset(
        pa: *const FcPattern,
        pb: *const FcPattern,
        os: *const FcObjectSet,
    ) -> FcBool;

    /// <https://www.freedesktop.org/software/fontconfig/fontconfig-devel/fcpatternfilter.html>
    pub fn FcPatternFilter(p: *mut FcPattern, os: *const FcObjectSet) -> *mut FcPattern;

    /// <https://www.freedesktop.org/software/fontconfig/fontconfig-devel/fcpatternhash.html>
    pub fn FcPatternHash(p: *const FcPattern) -> FcChar32;

    /// <https://www.freedesktop.org/software/fontconfig/fontconfig-devel/fcpatternadd.html>
    pub fn FcPatternAdd(
        p: *mut FcPattern,
        object: *const ffi::c_char,
        value: FcValue,
        append: FcBool,
    ) -> FcBool;

    /// <https://www.freedesktop.org/software/fontconfig/fontconfig-devel/fcpatternaddweak.html>
    pub fn FcPatternAddWeak(
        p: *mut FcPattern,
        object: *const ffi::c_char,
        value: FcValue,
        append: FcBool,
    ) -> FcBool;

    /// <https://www.freedesktop.org/software/fontconfig/fontconfig-devel/fcpatternadd-type.html>
    pub fn FcPatternAddInteger(
        p: *mut FcPattern,
        object: *const ffi::c_char,
        i: ffi::c_int,
    ) -> FcBool;

    /// <https://www.freedesktop.org/software/fontconfig/fontconfig-devel/fcpatternadd-type.html>
    pub fn FcPatternAddDouble(
        p: *mut FcPattern,
        object: *const ffi::c_char,
        d: ffi::c_double,
    ) -> FcBool;

    /// <https://www.freedesktop.org/software/fontconfig/fontconfig-devel/fcpatternadd-type.html>
    pub fn FcPatternAddString(
        p: *mut FcPattern,
        object: *const ffi::c_char,
        s: *const FcChar8,
    ) -> FcBool;

    /// <https://www.freedesktop.org/software/fontconfig/fontconfig-devel/fcpatternadd-type.html>
    pub fn FcPatternAddMatrix(
        p: *mut FcPattern,
        object: *const ffi::c_char,
        m: *const FcMatrix,
    ) -> FcBool;

    /// <https://www.freedesktop.org/software/fontconfig/fontconfig-devel/fcpatternadd-type.html>
    pub fn FcPatternAddCharSet(
        p: *mut FcPattern,
        object: *const ffi::c_char,
        c: *const FcCharSet,
    ) -> FcBool;

    /// <https://www.freedesktop.org/software/fontconfig/fontconfig-devel/fcpatternadd-type.html>
    pub fn FcPatternAddBool(p: *mut FcPattern, object: *const ffi::c_char, b: FcBool) -> FcBool;

    /// <https://www.freedesktop.org/software/fontconfig/fontconfig-devel/fcpatternadd-type.html>
    pub fn FcPatternAddFTFace(p: *mut FcPattern, object: *const ffi::c_char, f: FT_Face) -> FcBool;

    /// <https://www.freedesktop.org/software/fontconfig/fontconfig-devel/fcpatternadd-type.html>
    pub fn FcPatternAddLangSet(
        p: *mut FcPattern,
        object: *const ffi::c_char,
        l: *const FcLangSet,
    ) -> FcBool;

    /// <https://www.freedesktop.org/software/fontconfig/fontconfig-devel/fcpatternadd-type.html>
    pub fn FcPatternAddRange(
        p: *mut FcPattern,
        object: *const ffi::c_char,
        r: *const FcRange,
    ) -> FcBool;

    /// <https://www.freedesktop.org/software/fontconfig/fontconfig-devel/fcpatterngetwithbinding.html>
    pub fn FcPatternGetWithBinding(
        p: *mut FcPattern,
        object: *const ffi::c_char,
        id: ffi::c_int,
        v: *mut FcValue,
        b: *mut FcValueBinding,
    ) -> FcResult;

    /// <https://www.freedesktop.org/software/fontconfig/fontconfig-devel/fcpatternget.html>
    pub fn FcPatternGet(
        p: *mut FcPattern,
        object: *const ffi::c_char,
        id: ffi::c_int,
        v: *mut FcValue,
    ) -> FcResult;

    /// <https://www.freedesktop.org/software/fontconfig/fontconfig-devel/fcpatternget-type.html>
    pub fn FcPatternGetInteger(
        p: *mut FcPattern,
        object: *const ffi::c_char,
        n: ffi::c_int,
        i: *mut ffi::c_int,
    ) -> FcResult;

    /// <https://www.freedesktop.org/software/fontconfig/fontconfig-devel/fcpatternget-type.html>
    pub fn FcPatternGetDouble(
        p: *mut FcPattern,
        object: *const ffi::c_char,
        n: ffi::c_int,
        d: *mut ffi::c_double,
    ) -> FcResult;

    /// <https://www.freedesktop.org/software/fontconfig/fontconfig-devel/fcpatternget-type.html>
    pub fn FcPatternGetString(
        p: *mut FcPattern,
        object: *const ffi::c_char,
        n: ffi::c_int,
        s: *mut *const FcChar8,
    ) -> FcResult;

    /// <https://www.freedesktop.org/software/fontconfig/fontconfig-devel/fcpatternget-type.html>
    pub fn FcPatternGetMatrix(
        p: *mut FcPattern,
        object: *const ffi::c_char,
        n: ffi::c_int,
        m: *mut *mut FcMatrix,
    ) -> FcResult;

    /// <https://www.freedesktop.org/software/fontconfig/fontconfig-devel/fcpatternget-type.html>
    pub fn FcPatternGetCharSet(
        p: *mut FcPattern,
        object: *const ffi::c_char,
        n: ffi::c_int,
        c: *mut *mut FcCharSet,
    ) -> FcResult;

    /// <https://www.freedesktop.org/software/fontconfig/fontconfig-devel/fcpatternget-type.html>
    pub fn FcPatternGetBool(
        p: *mut FcPattern,
        object: *const ffi::c_char,
        n: ffi::c_int,
        b: *mut FcBool,
    ) -> FcResult;

    /// <https://www.freedesktop.org/software/fontconfig/fontconfig-devel/fcpatternget-type.html>
    pub fn FcPatternGetFtFace(
        p: *mut FcPattern,
        object: *const ffi::c_char,
        n: ffi::c_int,
        f: *mut FT_Face,
    ) -> FcResult;

    /// <https://www.freedesktop.org/software/fontconfig/fontconfig-devel/fcpatternget-type.html>
    pub fn FcPatternGetLangSet(
        p: *mut FcPattern,
        object: *const ffi::c_char,
        n: ffi::c_int,
        l: *mut *mut FcLangSet,
    ) -> FcResult;

    /// <https://www.freedesktop.org/software/fontconfig/fontconfig-devel/fcpatternget-type.html>
    pub fn FcPatternGetRange(
        p: *mut FcPattern,
        object: *const ffi::c_char,
        n: ffi::c_int,
        r: *mut *mut FcRange,
    ) -> FcResult;

    /// <https://www.freedesktop.org/software/fontconfig/fontconfig-devel/fcpatternbuild.html>
    pub fn FcPatternBuild(pattern: *mut FcPattern, ...) -> *mut FcPattern;

    /// <https://www.freedesktop.org/software/fontconfig/fontconfig-devel/fcpatterndel.html>
    pub fn FcPatternDel(p: *mut FcPattern, object: *const ffi::c_char) -> FcBool;

    /// <https://www.freedesktop.org/software/fontconfig/fontconfig-devel/fcpatternremove.html>
    pub fn FcPatternRemove(p: *mut FcPattern, object: *const ffi::c_char, id: ffi::c_int)
        -> FcBool;

    /// <https://www.freedesktop.org/software/fontconfig/fontconfig-devel/fcpatterniterstart.html>
    pub fn FcPatternIterStart(p: *const FcPattern, iter: *mut FcPatternIter);

    /// <https://www.freedesktop.org/software/fontconfig/fontconfig-devel/fcpatterniternext.html>
    pub fn FcPatternIterNext(p: *const FcPattern, iter: *mut FcPatternIter) -> FcBool;

    /// <https://www.freedesktop.org/software/fontconfig/fontconfig-devel/fcpatterniterequal.html>
    pub fn FcPatternIterEqual(
        p1: *const FcPattern,
        i1: *mut FcPatternIter,
        p2: *const FcPattern,
        i2: *mut FcPatternIter,
    ) -> FcBool;

    /// <https://www.freedesktop.org/software/fontconfig/fontconfig-devel/fcpatternfinditer.html>
    pub fn FcPatternFindIter(
        p: *const FcPattern,
        iter: *mut FcPatternIter,
        object: *const ffi::c_char,
    ) -> FcBool;

    /// <https://www.freedesktop.org/software/fontconfig/fontconfig-devel/fcpatterniterisvalid.html>
    pub fn FcPatternIterIsValid(p: *const FcPattern, iter: *mut FcPatternIter) -> FcBool;

    /// <https://www.freedesktop.org/software/fontconfig/fontconfig-devel/fcpatternitergetobject.html>
    pub fn FcPatternIterGetObject(
        p: *const FcPattern,
        iter: *mut FcPatternIter,
    ) -> *const ffi::c_char;

    /// <https://www.freedesktop.org/software/fontconfig/fontconfig-devel/fcpatternitervaluecount.html>
    pub fn FcPatternIterValueCount(p: *const FcPattern, iter: *mut FcPatternIter) -> ffi::c_int;

    /// <https://www.freedesktop.org/software/fontconfig/fontconfig-devel/fcpatternitergetvalue.html>
    pub fn FcPatternIterGetValue(
        p: *const FcPattern,
        iter: *mut FcPatternIter,
        id: ffi::c_int,
        v: *mut FcValue,
        b: *mut FcValueBinding,
    ) -> FcResult;

    /// <https://www.freedesktop.org/software/fontconfig/fontconfig-devel/fcpatternprint.html>
    pub fn FcPatternPrint(p: *const FcPattern);

    /// <https://www.freedesktop.org/software/fontconfig/fontconfig-devel/fcdefaultsubstitute.html>
    pub fn FcDefaultSubstitute(p: *mut FcPattern);

    /// <https://www.freedesktop.org/software/fontconfig/fontconfig-devel/fcnameparse.html>
    pub fn FcNameParse(name: *const FcChar8) -> *mut FcPattern;

    /// <https://www.freedesktop.org/software/fontconfig/fontconfig-devel/fcnameunparse.html>
    pub fn FcNameUnparse(pat: *mut FcPattern) -> *mut FcChar8;

    /// <https://www.freedesktop.org/software/fontconfig/fontconfig-devel/fcpatternformat.html>
    pub fn FcPatternFormat(pat: *mut FcPattern, format: *const FcChar8) -> *mut FcChar8;

    /// <https://www.freedesktop.org/software/fontconfig/fontconfig-devel/fcfontsetcreate.html>
    pub fn FcFontSetCreate() -> *mut FcFontSet;

    /// <https://www.freedesktop.org/software/fontconfig/fontconfig-devel/fcfontsetdestroy.html>
    pub fn FcFontSetDestroy(s: *mut FcFontSet);

    /// <https://www.freedesktop.org/software/fontconfig/fontconfig-devel/fcfontsetadd.html>
    pub fn FcFontSetAdd(s: *mut FcFontSet, font: *mut FcPattern) -> FcBool;

    /// <https://www.freedesktop.org/software/fontconfig/fontconfig-devel/fcfontsetlist.html>
    pub fn FcFontSetList(
        config: *mut FcConfig,
        sets: *mut *mut FcFontSet,
        nsets: ffi::c_int,
        pattern: *mut FcPattern,
        object_set: *mut FcObjectSet,
    ) -> *mut FcFontSet;

    /// <https://www.freedesktop.org/software/fontconfig/fontconfig-devel/fcfontsetmatch.html>
    pub fn FcFontSetMatch(
        config: *mut FcConfig,
        sets: *mut *mut FcFontSet,
        nsets: ffi::c_int,
        pattern: *mut FcPattern,
        result: *mut FcResult,
    ) -> *mut FcPattern;

    /// <https://www.freedesktop.org/software/fontconfig/fontconfig-devel/fcfontsetprint.html>
    pub fn FcFontSetPrint(set: *mut FcFontSet);

    /// <https://www.freedesktop.org/software/fontconfig/fontconfig-devel/fcfontsetsort.html>
    pub fn FcFontSetSort(
        config: *mut FcConfig,
        sets: *mut *mut FcFontSet,
        nsets: ffi::c_int,
        pattern: *mut FcPattern,
        trim: FcBool,
        csp: *mut *mut FcCharSet,
        result: *mut FcResult,
    ) -> *mut FcFontSet;

    /// <https://www.freedesktop.org/software/fontconfig/fontconfig-devel/fcobjectsetcreate.html>
    pub fn FcObjectSetCreate() -> *mut FcObjectSet;

    /// <https://www.freedesktop.org/software/fontconfig/fontconfig-devel/fcobjectsetadd.html>
    pub fn FcObjectSetAdd(os: *mut FcObjectSet, object: *const ffi::c_char) -> FcBool;

    /// <https://www.freedesktop.org/software/fontconfig/fontconfig-devel/fcobjectsetdestroy.html>
    pub fn FcObjectSetDestroy(os: *mut FcObjectSet);

    /// <https://www.freedesktop.org/software/fontconfig/fontconfig-devel/fcobjectsetbuild.html>
    pub fn FcObjectSetBuild(first: *const ffi::c_char, ...) -> *mut FcObjectSet;

    // MISSING METHODS HERE

    /// <https://www.freedesktop.org/software/fontconfig/fontconfig-devel/fcconfigcreate.html>
    pub fn FcConfigCreate() -> *mut FcConfig;

    /// <https://www.freedesktop.org/software/fontconfig/fontconfig-devel/fcconfigreference.html>
    pub fn FcConfigReference(config: *mut FcConfig) -> *mut FcConfig;

    /// <https://www.freedesktop.org/software/fontconfig/fontconfig-devel/fcconfigdestroy.html>
    pub fn FcConfigDestroy(config: *mut FcConfig);

    /// <https://www.freedesktop.org/software/fontconfig/fontconfig-devel/fcconfigsetcurrent.html>
    pub fn FcConfigSetCurrent(config: *mut FcConfig) -> FcBool;

    /// <https://www.freedesktop.org/software/fontconfig/fontconfig-devel/fcconfiggetcurrent.html>
    pub fn FcConfigGetCurrent() -> *mut FcConfig;

    /// <https://www.freedesktop.org/software/fontconfig/fontconfig-devel/fcconfiguptodate.html>
    pub fn FcConfigUptoDate(config: *mut FcConfig) -> FcBool;

    /// <https://www.freedesktop.org/software/fontconfig/fontconfig-devel/fcconfighome.html>
    pub fn FcConfigHome() -> *mut FcChar8;

    /// <https://www.freedesktop.org/software/fontconfig/fontconfig-devel/fcconfigenablehome.html>
    pub fn FcConfigEnableHome(enable: FcBool) -> FcBool;

    /// <https://www.freedesktop.org/software/fontconfig/fontconfig-devel/fcconfigbuildfonts.html>
    pub fn FcConfigBuildFonts(config: *mut FcConfig) -> FcBool;

    /// <https://www.freedesktop.org/software/fontconfig/fontconfig-devel/fcconfiggetconfigdirs.html>
    pub fn FcConfigGetConfigDirs(config: *mut FcConfig) -> *mut FcStrList;

    /// <https://www.freedesktop.org/software/fontconfig/fontconfig-devel/fcconfiggetfontdirs.html>
    pub fn FcConfigGetFontDirs(config: *mut FcConfig) -> *mut FcStrList;

    /// <https://www.freedesktop.org/software/fontconfig/fontconfig-devel/fcconfiggetconfigfiles.html>
    pub fn FcConfigGetConfigFiles(config: *mut FcConfig) -> *mut FcStrList;

    /// <https://www.freedesktop.org/software/fontconfig/fontconfig-devel/fcconfiggetcache.html>
    ///
    /// DEPRECATED
    pub fn FcConfigGetCache(config: *mut FcConfig) -> *mut FcChar8;

    /// <https://www.freedesktop.org/software/fontconfig/fontconfig-devel/fcconfiggetcachedirs.html>
    pub fn FcConfigGetCacheDirs(config: *mut FcConfig) -> *mut FcStrList;

    /// <https://www.freedesktop.org/software/fontconfig/fontconfig-devel/fcconfiggetfonts.html>
    pub fn FcConfigGetFonts(config: *mut FcConfig, set: FcSetName) -> *mut FcFontSet;

    /// <https://www.freedesktop.org/software/fontconfig/fontconfig-devel/fcconfiggetrescaninterval.html>
    pub fn FcConfigGetRescanInterval(config: *mut FcConfig) -> ffi::c_int;

    /// <https://www.freedesktop.org/software/fontconfig/fontconfig-devel/fcconfigsetrescaninterval.html>
    pub fn FcConfigSetRescanInterval(config: *mut FcConfig, rescan_interval: ffi::c_int) -> FcBool;

    /// <https://www.freedesktop.org/software/fontconfig/fontconfig-devel/fcconfigappfontaddfile.html>
    pub fn FcConfigAppFontAddFile(config: *mut FcConfig, file: *const FcChar8) -> FcBool;

    /// <https://www.freedesktop.org/software/fontconfig/fontconfig-devel/fcconfigappfontadddir.html>
    pub fn FcConfigAppFontAddDir(config: *mut FcConfig, dir: *const FcChar8) -> FcBool;

    /// <https://www.freedesktop.org/software/fontconfig/fontconfig-devel/fcconfigappfontclear.html>
    pub fn FcConfigAppFontClear(config: *mut FcConfig);

    /// <https://www.freedesktop.org/software/fontconfig/fontconfig-devel/fcconfigsubstitutewithpat.html>
    pub fn FcConfigSubstituteWithPat(
        config: *mut FcConfig,
        p: *mut FcPattern,
        p_pat: *mut FcPattern,
        kind: FcMatchKind,
    ) -> FcBool;

    /// <https://www.freedesktop.org/software/fontconfig/fontconfig-devel/fcconfigsubstitute.html>
    pub fn FcConfigSubstitute(
        config: *mut FcConfig,
        pattern: *mut FcPattern,
        kind: FcMatchKind,
    ) -> FcBool;

    /// <https://www.freedesktop.org/software/fontconfig/fontconfig-devel/fcfontmatch.html>
    pub fn FcFontMatch(
        config: *mut FcConfig,
        pattern: *mut FcPattern,
        result: *mut FcResult,
    ) -> *mut FcPattern;

    /// <https://www.freedesktop.org/software/fontconfig/fontconfig-devel/fcfontsort.html>
    pub fn FcFontSort(
        config: *mut FcConfig,
        pattern: *mut FcPattern,
        trim: FcBool,
        csp: *mut *mut FcCharSet,
        result: *mut FcResult,
    ) -> *mut FcFontSet;

    /// <https://www.freedesktop.org/software/fontconfig/fontconfig-devel/fcfontrenderprepare.html>
    pub fn FcFontRenderPrepare(
        config: *mut FcConfig,
        pattern: *mut FcPattern,
        font: *mut FcPattern,
    ) -> *mut FcPattern;

    /// <https://www.freedesktop.org/software/fontconfig/fontconfig-devel/fcfontlist.html>
    pub fn FcFontList(
        config: *mut FcConfig,
        pattern: *mut FcPattern,
        os: *mut FcObjectSet,
    ) -> *mut FcFontSet;

    /// <https://www.freedesktop.org/software/fontconfig/fontconfig-devel/fcconfiggetfilename.html>
    pub fn FcConfigGetFilename(config: *mut FcConfig, name: *const FcChar8) -> *mut FcChar8;

    /// <https://www.freedesktop.org/software/fontconfig/fontconfig-devel/fcconfigparseandload.html>
    pub fn FcConfigParseAndLoad(
        config: *mut FcConfig,
        file: *const FcChar8,
        complain: FcBool,
    ) -> FcBool;

    /// <https://www.freedesktop.org/software/fontconfig/fontconfig-devel/fcconfigparseandloadfrommemory.html>
    pub fn FcConfigParseAndLoadFromMemory(
        config: *mut FcConfig,
        buffer: *const FcChar8,
        complain: FcBool,
    ) -> FcBool;

    /// <https://www.freedesktop.org/software/fontconfig/fontconfig-devel/fcconfiggetsysroot.html>
    pub fn FcConfigGetSysRoot(config: *mut FcConfig) -> *mut FcChar8;

    /// <https://www.freedesktop.org/software/fontconfig/fontconfig-devel/fcconfigsetsysroot.html>
    pub fn FcConfigSetSysRoot(config: *mut FcConfig, sysroot: *const FcChar8);

    /// <https://www.freedesktop.org/software/fontconfig/fontconfig-devel/fcconfigfileinfoiterinit.html>
    pub fn FcConfigFileInfoIterInit(config: *mut FcConfig, iter: *mut FcConfigFileInfoIter);

    /// <https://www.freedesktop.org/software/fontconfig/fontconfig-devel/fcconfigfileinfoiternext.html>
    pub fn FcConfigFileInfoIterNext(
        config: *mut FcConfig,
        iter: *mut FcConfigFileInfoIter,
    ) -> FcBool;

    /// <https://www.freedesktop.org/software/fontconfig/fontconfig-devel/fcconfigfileinfoiterget.html>
    pub fn FcConfigFileInfoIterGet(
        config: *mut FcConfig,
        iter: *mut FcConfigFileInfoIter,
        name: *mut *mut FcChar8,
        description: *mut *mut FcChar8,
        enabled: *mut FcBool,
    ) -> FcBool;

    // MISSING FUNCTIONS

    /// <https://www.freedesktop.org/software/fontconfig/fontconfig-devel/fcstrsetcreate.html>
    pub fn FcStrSetCreate() -> *mut FcStrSet;

    /// <https://www.freedesktop.org/software/fontconfig/fontconfig-devel/fcstrsetmember.html>
    pub fn FcStrSetMember(set: *mut FcStrSet, s: *const FcChar8) -> FcBool;

    /// <https://www.freedesktop.org/software/fontconfig/fontconfig-devel/fcstrsetequal.html>
    pub fn FcStrSetEqual(set_a: *mut FcStrSet, set_b: *mut FcStrSet) -> FcBool;

    /// <https://www.freedesktop.org/software/fontconfig/fontconfig-devel/fcstrsetadd.html>
    pub fn FcStrSetAdd(set: *mut FcStrSet, s: *const FcChar8) -> FcBool;

    /// <https://www.freedesktop.org/software/fontconfig/fontconfig-devel/fcstrsetaddfilename.html>
    pub fn FcStrSetAddFilename(set: *mut FcStrSet, s: *const FcChar8) -> FcBool;

    /// <https://www.freedesktop.org/software/fontconfig/fontconfig-devel/fcstrsetdel.html>
    pub fn FcStrSetDel(set: *mut FcStrSet, s: *const FcChar8) -> FcBool;

    /// <https://www.freedesktop.org/software/fontconfig/fontconfig-devel/fcstrsetdestroy.html>
    pub fn FcStrSetDestroy(set: *mut FcStrSet);

    /// <https://www.freedesktop.org/software/fontconfig/fontconfig-devel/fcstrlistcreate.html>
    pub fn FcStrListCreate(set: *mut FcStrSet) -> *mut FcStrList;

    /// <https://www.freedesktop.org/software/fontconfig/fontconfig-devel/fcstrlistfirst.html>
    pub fn FcStrListFirst(list: *mut FcStrList);

    /// <https://www.freedesktop.org/software/fontconfig/fontconfig-devel/fcstrlistnext.html>
    pub fn FcStrListNext(list: *mut FcStrList) -> *const FcChar8;

    /// <https://www.freedesktop.org/software/fontconfig/fontconfig-devel/fcstrlistdone.html>
    pub fn FcStrListDone(list: *mut FcStrList);

    // MISSING FUNCTIONS

    /// <https://www.freedesktop.org/software/fontconfig/fontconfig-devel/fcstrfree.html>
    pub fn FcStrFree(s: *mut FcChar8);
}

#[inline]
pub(crate) fn fcbool(fcbool: FcBool) -> bool {
    fcbool != 0
}

pub struct SetIterator<'a, T>
where
    T: ?Sized,
{
    pub(crate) current: *mut *mut T,
    pub(crate) remaining: usize,
    pub(crate) phantom_data: PhantomData<&'a T>,
}

impl<'a, T> Iterator for SetIterator<'a, T>
where
    T: ?Sized,
{
    type Item = &'a *mut T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            return None;
        }

        let item = self.current;

        self.current = self.current.wrapping_add(1);
        self.remaining -= 1;

        assert!(!item.is_null());
        Some(unsafe { &*item })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining, Some(self.remaining))
    }
}

impl<'a, T> FusedIterator for SetIterator<'a, T> {}
