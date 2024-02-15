use std::fs;

use image::DynamicTexture;

const VALID: &str = concat!(env!("TEST_DIR"), "/bmp-testsuite/valid");
const CORRUPT: &str = concat!(env!("TEST_DIR"), "/bmp-testsuite/corrupt");
const QUESTIONABLE: &str = concat!(env!("TEST_DIR"), "/bmp-testsuite/questionable");

#[test]
fn valid_images() {
    // Assert that all valid images can be parsed
    for entry in fs::read_dir(VALID).expect("Cannot read dir") {
        let entry = entry.expect("Cannot read entry");
        let path = entry.path();
        if path.is_file() {
            println!("Testing {:?}", path.file_name().unwrap_or_default());
            let bytes = fs::read(path).expect("Cannot read image file");
            let result = DynamicTexture::from_bytes(&bytes);
            assert!(result.is_ok());
        }
    }
}

#[test]
fn corrupt_images() {
    // Assert that all corrupt images fail to parse without crashing
    for entry in fs::read_dir(CORRUPT).expect("Cannot read dir") {
        let entry = entry.expect("Cannot read entry");
        let path = entry.path();
        if path.is_file() {
            println!("Testing {:?}", path.file_name().unwrap_or_default());
            let bytes = fs::read(path).expect("Cannot read image file");
            let result = DynamicTexture::from_bytes(&bytes);
            assert!(result.is_err());
        }
    }
}

#[test]
fn questionable_images() {
    // Assert that no questionable images cause crashes
    // (Whether or not parsing succeeds is not specified)
    for entry in fs::read_dir(QUESTIONABLE).expect("Cannot read dir") {
        let entry = entry.expect("Cannot read entry");
        let path = entry.path();
        if path.is_file() {
            println!("Testing {:?}", path.file_name().unwrap_or_default());
            let bytes = fs::read(path).expect("Cannot read image file");
            let _ = DynamicTexture::from_bytes(&bytes);
        }
    }
}
