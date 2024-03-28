//! <https://262.ecma-international.org/14.0/#sec-object-type>

mod vtable;

use crate::bytecode::{Exception, ThrowCompletionOr};

use self::vtable::ObjectMethods;

use super::Value;

use std::{collections::HashMap, fmt, ptr};

/// <https://262.ecma-international.org/14.0/#sec-object-type>
#[derive(Clone)]
pub struct Object {
    extensible: bool,
    properties: HashMap<PropertyKey, PropertyDescriptor>,
    methods: vtable::ObjectMethods,
}

/// <https://262.ecma-international.org/14.0/#sec-property-descriptor-specification-type>
#[derive(Clone, Debug)]
pub struct PropertyDescriptor {
    enumerable: Option<bool>,
    configurable: Option<bool>,
    variant: PropertyDescriptorVariant,
}

#[derive(Clone, Debug)]
enum PropertyDescriptorVariant {
    Data(DataProperty),
    Accessor(AccessorProperty),
}

#[derive(Clone, Debug, Default)]
pub struct DataProperty {
    value: Option<Value>,
    writable: Option<bool>,
}

#[derive(Clone, Debug, Default)]
pub struct AccessorProperty;

impl Default for PropertyDescriptorVariant {
    fn default() -> Self {
        Self::Data(DataProperty::default())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum PropertyKey {
    String(String),
}

impl PropertyDescriptor {
    pub fn set_value(&mut self, value: Option<Value>) {
        match &mut self.variant {
            PropertyDescriptorVariant::Data(data_descriptor) => data_descriptor.value = value,
            PropertyDescriptorVariant::Accessor(_) => todo!(),
        }
    }

    fn has_fields(&self) -> bool {
        if self.enumerable.is_some() || self.configurable.is_some() {
            return true;
        }

        match &self.variant {
            PropertyDescriptorVariant::Data(data_descriptor) => {
                data_descriptor.value.is_some() || data_descriptor.writable.is_some()
            },
            PropertyDescriptorVariant::Accessor(_) => todo!(),
        }
    }

    pub fn value(&self) -> Option<&Value> {
        match &self.variant {
            PropertyDescriptorVariant::Data(data_descriptor) => data_descriptor.value.as_ref(),
            PropertyDescriptorVariant::Accessor(_) => todo!(),
        }
    }
}

impl Object {
    #[inline]
    pub fn vtable(&self) -> &vtable::ObjectMethods {
        &self.methods
    }

    #[inline]
    #[must_use]
    pub fn get_prototype_of(&self) -> Option<Object> {
        (self.vtable().get_prototype_of)(self)
    }

    #[inline]
    #[must_use]
    pub fn set_prototype_of(&mut self, v: Option<Object>) -> bool {
        (self.vtable().set_prototype_of)(self, v)
    }

    #[inline]
    #[must_use]
    pub fn is_extensible(&self) -> bool {
        (self.vtable().is_extensible)(self)
    }

    #[inline]
    #[must_use]
    pub fn prevent_extension(&mut self) -> bool {
        (self.vtable().prevent_extensions)(self)
    }

    #[inline]
    #[must_use]
    pub fn get_own_property(&self, p: &PropertyKey) -> Option<PropertyDescriptor> {
        (self.vtable().get_own_property)(self, p)
    }

    #[inline]
    #[must_use]
    pub fn define_own_property(
        &mut self,
        p: &PropertyKey,
        desc: PropertyDescriptor,
    ) -> ThrowCompletionOr<bool> {
        (self.vtable().define_own_property)(self, p, desc)
    }

    /// <https://262.ecma-international.org/14.0/#sec-createdatapropertyorthrow>
    pub fn create_data_property_or_throw(
        o: &mut Self,
        p: &PropertyKey,
        v: Value,
    ) -> ThrowCompletionOr<()> {
        // 1. Let success be ? CreateDataProperty(O, P, V).
        let success = Object::create_data_property(o, p, v)?;

        // 2. If success is false, throw a TypeError exception.
        if !success {
            return Err(Exception::type_error());
        }

        // 3. Return unused.
        Ok(())
    }

    /// <https://262.ecma-international.org/14.0/#sec-createdataproperty>
    pub fn create_data_property(
        o: &mut Self,
        p: &PropertyKey,
        v: Value,
    ) -> ThrowCompletionOr<bool> {
        // 1. Let newDesc be the PropertyDescriptor { [[Value]]: V, [[Writable]]: true, [[Enumerable]]: true, [[Configurable]]: true }.
        let new_desc = PropertyDescriptor {
            variant: PropertyDescriptorVariant::Data(DataProperty {
                value: Some(v),
                writable: Some(true),
            }),
            enumerable: Some(true),
            configurable: Some(true),
        };

        // 2. Return ? O.[[DefineOwnProperty]](P, newDesc).
        o.define_own_property(p, new_desc)
    }
}

impl PartialEq for Object {
    fn eq(&self, other: &Self) -> bool {
        // Objects are compared by identity
        ptr::eq(self, other)
    }
}

/// <https://262.ecma-international.org/14.0/#sec-validateandapplypropertydescriptor>
fn validate_and_apply_property_descriptor(
    o: Option<&mut Object>,
    p: PropertyKey,
    extensible: bool,
    desc: PropertyDescriptor,
    current: Option<PropertyDescriptor>,
) -> bool {
    // 1. Assert: IsPropertyKey(P) is true.
    //            Note: Pointless, we don't pass this as a language value

    // 2. If current is undefined, then
    let Some(current) = current else {
        // a. If extensible is false, return false.
        if !extensible {
            return false;
        }

        // b. If O is undefined, return true.
        let Some(o) = o else {
            return true;
        };

        // c. If IsAccessorDescriptor(Desc) is true, then
        //    i. Create an own accessor property named P of object O whose [[Get]], [[Set]], [[Enumerable]], and [[Configurable]]
        //       attributes are set to the value of the corresponding field in Desc if Desc has that field, or to the attribute's default value otherwise.
        // d. Else,
        //    i. Create an own data property named P of object O whose [[Value]], [[Writable]], [[Enumerable]], and [[Configurable]]
        //       attributes are set to the value of the corresponding field in Desc if Desc has that field, or to the attribute's default value otherwise.
        o.properties.insert(p, desc);

        // e. Return true.
        return true;
    };

    // 3. Assert: current is a fully populated Property Descriptor.
    // assert!(current.is_fully_populated());

    // 4. If Desc does not have any fields, return true.
    if !desc.has_fields() {
        return true;
    }

    // 5. If current.[[Configurable]] is false, then
    if current.configurable == Some(false) {
        // a. If Desc has a [[Configurable]] field and Desc.[[Configurable]] is true, return false.
        if desc.configurable == Some(true) {
            return false;
        }

        // b. If Desc has an [[Enumerable]] field and SameValue(Desc.[[Enumerable]], current.[[Enumerable]]) is false, return false.
        if desc.enumerable != current.enumerable {
            return false;
        }

        todo!()
    }
    todo!()
}

impl fmt::Debug for Object {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Object")
            .field("is_extensible", &self.extensible)
            .field("properties", &self.properties)
            .finish()
    }
}

impl Default for Object {
    fn default() -> Self {
        Self {
            extensible: true,
            properties: HashMap::default(),
            methods: ObjectMethods::ORDINARY,
        }
    }
}
