use super::{Serialize, Serializer};

impl<'a> Serialize for &'a str {
    fn serialize_to<S>(&self, serializer: &mut S) -> Result<(), S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_string(self)
    }
}

impl<'a> Serialize for String {
    fn serialize_to<S>(&self, serializer: &mut S) -> Result<(), S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_string(self.as_str())
    }
}

impl<'a> Serialize for usize {
    fn serialize_to<S>(&self, serializer: &mut S) -> Result<(), S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_usize(*self)
    }
}
