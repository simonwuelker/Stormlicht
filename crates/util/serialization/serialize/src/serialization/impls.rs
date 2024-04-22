use super::{Serialize, Serializer};

impl<'a> Serialize for &'a str {
    fn serialize_to<S>(&self, serializer: &mut S) -> Result<(), S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_string(self)
    }
}
