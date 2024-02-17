use std::{fs, path::Path};

use image::DynamicTexture;

#[cfg(test)]
mod images {
    pub const VALID: &str = concat!(env!("TEST_DIR"), "/bmp-testsuite/valid");
    pub const CORRUPT: &str = concat!(env!("TEST_DIR"), "/bmp-testsuite/corrupt");
    pub const QUESTIONABLE: &str = concat!(env!("TEST_DIR"), "/bmp-testsuite/questionable");
}

#[test]
fn valid_images() {
    // Assert that all valid images can be parsed
    for test_file in test_files(images::VALID) {
        let result = DynamicTexture::from_bytes(&test_file);
        assert!(result.is_ok());
    }
}

#[test]
fn corrupt_images() {
    // Assert that all corrupt images fail to parse without crashing
    for test_file in test_files(images::CORRUPT) {
        let result = DynamicTexture::from_bytes(&test_file);
        assert!(result.is_err());
    }
}

#[test]
fn questionable_images() {
    // Assert that no questionable images cause crashes
    // (Whether or not parsing succeeds is not specified)
    for test_file in test_files(images::QUESTIONABLE) {
        let _ = DynamicTexture::from_bytes(&test_file);
    }
}

fn test_files<P>(path: P) -> impl Iterator<Item = Vec<u8>>
where
    P: AsRef<Path>,
{
    fs::read_dir(path)
        .expect("Cannot read dir")
        .into_iter()
        .map(|entry| entry.expect("Cannot read entry"))
        .map(|entry| entry.path())
        .filter(|path| path.is_file())
        .filter(|path| path.extension().is_some_and(|ext| ext == "bmp"))
        .inspect(|path| println!("Testing {:?}", path.file_name().unwrap_or_default()))
        .map(|path| fs::read(path).expect("Cannot read image file"))
}
