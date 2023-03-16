use crate::codegen::DOMType;

pub trait Inheritance {
    fn as_type() -> DOMType;
}
