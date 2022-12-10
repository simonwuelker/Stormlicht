use std::fs;
use std::path::Path;

pub struct Surface {
    width: usize,
    height: usize,
    data: Vec<f32>,
}

impl Surface {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width: width,
            height: height,
            data: vec![0.0; width * height],
        }
    }

    pub fn store_as_bitmap<P: AsRef<Path>>(self, path: P) -> std::io::Result<()> {
        let f = fs::File::create(path)?;
        _ = f;
        Ok(())
    }
}
