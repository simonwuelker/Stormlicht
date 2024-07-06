use sl_std::ascii;

#[derive(Clone, Copy, Debug)]
pub struct PathSegments<'a> {
    pub is_opaque: bool,
    pub path: &'a ascii::Str,
}

impl<'a> Iterator for PathSegments<'a> {
    type Item = &'a ascii::Str;

    fn next(&mut self) -> Option<Self::Item> {
        if self.path.is_empty() {
            return None;
        }

        let segment = if self.is_opaque {
            std::mem::take(&mut self.path)
        } else {
            if let Some((chunk, remaining)) = self.path.split_once(ascii::Char::Solidus) {
                self.path = remaining;
                chunk
            } else {
                std::mem::take(&mut self.path)
            }
        };

        Some(segment)
    }
}
