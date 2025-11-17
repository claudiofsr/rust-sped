use crate::EFDResult;
use std::{fs, io::Write};
use tempfile::NamedTempFile;

/// Create a named temporary file and write some data into it
pub fn create_a_temp_file(contents: &str, read_file: bool) -> EFDResult<NamedTempFile> {
    // Create a file inside of `env::temp_dir()`.
    let mut file = NamedTempFile::new()?;

    // Write some test data to the file handle.
    file.write_all(contents.as_bytes())?;

    if read_file {
        // Reading an entire file into a String:
        let string = fs::read_to_string(file.path())?; // The '?' operator propagates errors
        println!(
            "Conteúdo do arquivo temporário [{:?}]:\n{}",
            file.path(),
            string
        );
    }

    Ok(file)
}
