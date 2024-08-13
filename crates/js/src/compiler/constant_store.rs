use crate::Value;

/// Contains all literals referenced inside the compiled code
///
/// Literals are automatically deduplicated, meaning
/// ```
/// let x = 1;
/// let y = 1;
/// ```
/// still only stores `1` once.
#[derive(Clone, Debug, Default)]
pub struct ConstantStore {
    values: Vec<Value>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ConstantHandle(usize);

impl ConstantStore {
    #[must_use]
    pub fn get_or_insert_constant(&mut self, constant: Value) -> ConstantHandle {
        let index = self
            .values
            .iter()
            .position(|v| v == &constant)
            .unwrap_or_else(|| {
                let index = self.values.len();
                self.values.push(constant);
                index
            });

        ConstantHandle(index)
    }

    #[must_use]
    pub fn get_constant(&self, handle: ConstantHandle) -> &Value {
        &self.values[handle.0]
    }
}
