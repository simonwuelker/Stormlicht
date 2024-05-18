use super::{Object, PropertyDescriptor, PropertyDescriptorVariant, PropertyKey};
use crate::value::{object::validate_and_apply_property_descriptor, ThrowCompletionOr, Value};

#[derive(Clone, Copy)]
pub struct ObjectMethods {
    /// <https://262.ecma-international.org/14.0/#sec-ordinary-object-internal-methods-and-internal-slots-getprototypeof>
    pub get_prototype_of: fn(o: &Object) -> Option<Object>,

    /// <https://262.ecma-international.org/14.0/#sec-ordinary-object-internal-methods-and-internal-slots-setprototypeof-v>
    pub set_prototype_of: fn(o: &mut Object, v: Option<Object>) -> bool,

    /// <https://262.ecma-international.org/14.0/#sec-ordinary-object-internal-methods-and-internal-slots-isextensible>
    pub is_extensible: fn(o: &Object) -> bool,

    /// <https://262.ecma-international.org/14.0/#sec-ordinary-object-internal-methods-and-internal-slots-preventextensions>
    pub prevent_extensions: fn(o: &mut Object) -> bool,

    /// <https://262.ecma-international.org/14.0/#sec-ordinary-object-internal-methods-and-internal-slots-getownproperty-p>
    pub get_own_property: fn(o: &Object, p: &PropertyKey) -> Option<PropertyDescriptor>,

    /// <https://262.ecma-international.org/14.0/#sec-ordinary-object-internal-methods-and-internal-slots-defineownproperty-p-desc>
    pub define_own_property:
        fn(o: &mut Object, p: &PropertyKey, desc: PropertyDescriptor) -> ThrowCompletionOr<bool>,

    /// <https://262.ecma-international.org/14.0/#sec-ordinary-object-internal-methods-and-internal-slots-get-p-receiver>
    pub get: fn(o: &Object, p: &PropertyKey) -> ThrowCompletionOr<Value>,
}

impl ObjectMethods {
    pub const ORDINARY: Self = Self {
        get_prototype_of: Self::ordinary_get_prototype_of,
        set_prototype_of: Self::ordinary_set_prototype_of,
        is_extensible: Self::ordinary_is_extensible,
        prevent_extensions: Self::ordinary_prevent_extensions,
        get_own_property: Self::ordinary_get_own_property,
        define_own_property: Self::ordinary_define_own_property,
        get: Self::ordinary_get,
    };

    /// <https://262.ecma-international.org/14.0/#sec-ordinarygetprototypeof>
    fn ordinary_get_prototype_of(o: &Object) -> Option<Object> {
        // 1. Return O.[[Prototype]].
        o.properties
            .get(&PropertyKey::String("prototype".to_string()))
            .expect("Object does not have a [[property]] slot")
            .value()
            .map(Value::as_object)
            .cloned()
    }

    /// <https://262.ecma-international.org/14.0/#sec-ordinarysetprototypeof>
    fn ordinary_set_prototype_of(o: &mut Object, v: Option<Object>) -> bool {
        // 1. Let current be O.[[Prototype]].
        let current = o.get_prototype_of();

        // 2. If SameValue(V, current) is true, return true.
        if v == current {
            return true;
        }

        // 3. Let extensible be O.[[Extensible]].
        let extensible = o.extensible;

        // 4. If extensible is false, return false.
        if !extensible {
            return false;
        }

        // 5. Let p be V.
        let mut p = v.as_ref();

        // 6. Let done be false.
        let mut done = false;

        // 7. Repeat, while done is false,
        while !done {
            // a. If p is null, set done to true.
            let Some(nonnull_p) = &mut p else {
                break;
            };

            // b. Else if SameValue(p, O) is true, return false.
            if *nonnull_p == o {
                return false;
            }
            // c. Else,
            else {
                // FIXME: i. If p.[[GetPrototypeOf]] is not the ordinary object internal method defined in 10.1.1, set done to true.
                if false {
                    done = true;
                }

                // ii. Else, set p to p.[[Prototype]].
                p = nonnull_p
                    .properties
                    .get(&PropertyKey::String("prototype".to_string()))
                    .and_then(|desc| desc.value())
                    .map(Value::as_object)
                    .clone();
            }
        }

        // 8. Set O.[[Prototype]] to V.
        o.properties
            .get_mut(&PropertyKey::String("prototype".to_string()))
            .unwrap()
            .set_value(v.map(Value::from));

        // 9. Return true.
        true
    }

    /// <https://262.ecma-international.org/14.0/#sec-ordinaryisextensible>
    fn ordinary_is_extensible(o: &Object) -> bool {
        // 1. Return O.[[Extensible]].
        o.extensible
    }

    /// <https://262.ecma-international.org/14.0/#sec-ordinarydefineownproperty>
    fn ordinary_define_own_property(
        o: &mut Object,
        p: &PropertyKey,
        desc: PropertyDescriptor,
    ) -> ThrowCompletionOr<bool> {
        // 1. Let current be ? O.[[GetOwnProperty]](P).
        let current = o.get_own_property(p);

        // 2. Let extensible be ? IsExtensible(O).
        let extensible = o.is_extensible();

        // 3. Return ValidateAndApplyPropertyDescriptor(O, P, extensible, Desc, current).
        let result =
            validate_and_apply_property_descriptor(Some(o), p.clone(), extensible, desc, current);
        Ok(result)
    }

    /// <https://262.ecma-international.org/14.0/#sec-ordinary-object-internal-methods-and-internal-slots-preventextensions>
    pub fn ordinary_prevent_extensions(o: &mut Object) -> bool {
        // 1. Set O.[[Extensible]] to false.
        _ = o;
        todo!();

        // 2. Return true.
    }

    /// <https://262.ecma-international.org/14.0/#sec-ordinarygetownproperty>
    pub fn ordinary_get_own_property(o: &Object, p: &PropertyKey) -> Option<PropertyDescriptor> {
        // 1. If O does not have an own property with key P, return undefined.
        let x = o.properties.get(p)?;

        // 2. Let D be a newly created Property Descriptor with no fields.
        let d;

        // 3. Let X be O's own property whose key is P.
        //    NOTE: We do this together with 1.

        // 4. If X is a data property, then
        //    a. Set D.[[Value]] to the value of X's [[Value]] attribute.
        //    b. Set D.[[Writable]] to the value of X's [[Writable]] attribute.
        // 5. Else,
        //    a. Assert: X is an accessor property.
        //    b. Set D.[[Get]] to the value of X's [[Get]] attribute.
        //    c. Set D.[[Set]] to the value of X's [[Set]] attribute.
        // 6. Set D.[[Enumerable]] to the value of X's [[Enumerable]] attribute.
        // 7. Set D.[[Configurable]] to the value of X's [[Configurable]] attribute.
        d = x.clone();

        // 8. Return D.
        Some(d)
    }

    /// <https://262.ecma-international.org/14.0/#sec-ordinaryget>
    pub fn ordinary_get(o: &Object, p: &PropertyKey) -> ThrowCompletionOr<Value> {
        // 1. Let desc be ? O.[[GetOwnProperty]](P).
        let desc = o.get_own_property(p);

        // 2. If desc is undefined, then
        let Some(desc) = desc else {
            // a. Let parent be ? O.[[GetPrototypeOf]]().
            let parent = o.get_prototype_of();

            // b. If parent is null, return undefined.
            let Some(parent) = parent else {
                return Ok(Value::Undefined);
            };

            // c. Return ? parent.[[Get]](P, Receiver).
            return parent.get(p);
        };

        // 3. If IsDataDescriptor(desc) is true, return desc.[[Value]].
        match desc.variant {
            PropertyDescriptorVariant::Data(data) => {
                return Ok(data.value.unwrap_or(Value::Undefined));
            },
            _ => {
                // 4. Assert: IsAccessorDescriptor(desc) is true.
                todo!();
            },
        }
    }
}
