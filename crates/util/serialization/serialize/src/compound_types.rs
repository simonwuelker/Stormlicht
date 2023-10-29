use std::collections::HashMap;

pub trait Sequence: Default {
    type Item;

    fn add_item(&mut self, item: Self::Item);
}

pub trait Map: Default {
    type Value;

    fn add_key_value(&mut self, key: String, value: Self::Value);
}

impl<I> Sequence for Vec<I> {
    type Item = I;

    fn add_item(&mut self, item: Self::Item) {
        self.push(item);
    }
}

impl<V> Map for HashMap<String, V> {
    type Value = V;

    fn add_key_value(&mut self, key: String, value: Self::Value) {
        self.insert(key, value);
    }
}
