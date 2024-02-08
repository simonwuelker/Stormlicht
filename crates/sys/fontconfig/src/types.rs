//! Contains public types, usually thin and safe wrappers around fontconfig equivalents

use std::ffi;

use crate::{bindings, Range};

#[derive(Clone, Debug)]
pub enum Value {
    Void,
    Integer(i32),
    Double(f64),
    String(String),
    Bool(bool),
    Range(Range),
}

impl From<bindings::FcValue> for Value {
    fn from(value: bindings::FcValue) -> Self {
        match value.type_tag {
            bindings::FcType::FcTypeUnknown => panic!("Cannot convert FcType \"unknown\""),
            bindings::FcType::FcTypeVoid => Self::Void,
            bindings::FcType::FcTypeInteger => Self::Integer(unsafe { value.content.i }),
            bindings::FcType::FcTypeDouble => Self::Double(unsafe { value.content.d }),
            bindings::FcType::FcTypeString => {
                let c_str = unsafe { ffi::CStr::from_ptr(value.content.s) };
                Self::String(c_str.to_string_lossy().into_owned())
            },
            bindings::FcType::FcTypeBool => Self::Bool(unsafe { value.content.b } != 0),
            bindings::FcType::FcTypeRange => {
                let range = Range::from_ptr(unsafe { value.content.r });
                Self::Range(range)
            },
            _ => todo!(),
        }
    }
}
