use http::compress::brotli;
use std::{fs, io::Read};

const TESTS_DIR: &'static str = "../../downloads/brotli/testdata/tests/testdata";

#[test]
fn test_brotli_decode() -> Result<(), std::io::Error> {
    for testfile_or_error in
        fs::read_dir(TESTS_DIR).expect("Test files not found, did you run download.sh?")
    {
        let testfile = testfile_or_error?;

        if let Some(extension) = testfile.path().extension() {
            if extension == "compressed" {
                println!("Testing decompression of {}", testfile.path().display());

                let mut compressed_buffer = vec![];
                fs::File::open(testfile.path())?.read_to_end(&mut compressed_buffer)?;

                let uncompressed_file = testfile
                    .path()
                    .with_file_name(testfile.path().file_stem().unwrap());
                let mut uncompressed_buffer = vec![];
                fs::File::open(&uncompressed_file)?.read_to_end(&mut uncompressed_buffer)?;

                let decompressed =
                    brotli::decode(&compressed_buffer).expect("Brotli decompression failed");

                let mut i = 0;
                for (a, b) in decompressed.iter().zip(&uncompressed_buffer) {
                    if a != b {
                        println!("NOT EQUAL {a} {b} at {i}");
                    }
                    i += 1;
                }

                assert!(
                    decompressed
                        .iter()
                        .zip(&uncompressed_buffer)
                        .all(|(a, b)| a == b)
                        && decompressed.len() == uncompressed_buffer.len()
                );
            }
        }
    }

    Ok(())
}
