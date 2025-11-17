use crate::{EFDResult, NEWLINE_BYTE, Tipo, ValidatedLine};
use std::{
    collections::HashMap,
    io::BufRead,
    path::{Path, PathBuf},
};

/// `EFDLineIterator` is a custom iterator adapter designed to read, parse,
/// and validate lines from an EFD (Escrituração Fiscal Digital) file.
///
/// It wraps a `BufRead` source, splits it into lines, and then attempts to
/// convert each raw byte line into a `ValidatedLine`. It handles potential
/// decoding issues and filtering of ignorable lines.
///
/// The iterator stops when the "9999" record is encountered, signaling the end of the file.
pub struct EFDLineIterator<'a, R> {
    /// The inner iterator, pairing raw byte lines with their 1-based line numbers.
    inner: std::iter::Zip<std::io::Split<R>, std::ops::RangeFrom<usize>>,
    /// The path to the EFD file, used for detailed error reporting.
    arquivo: PathBuf,
    /// A reference to the EFD schema definitions, used for record validation.
    registros_efd: &'a HashMap<&'static str, HashMap<u16, (&'static str, Tipo)>>,
}

impl<'a, R: BufRead> EFDLineIterator<'a, R> {
    /// Creates a new `EFDLineIterator`.
    ///
    /// # Arguments
    /// * `reader` - A `BufRead` implementor (e.g., `BufReader<File>`) that provides the file content.
    /// * `arquivo_ref` - A reference to the path of the EFD file.
    /// * `registros_efd` - A reference to the HashMap containing EFD record definitions.
    pub fn new(
        reader: R,
        arquivo_ref: &Path,
        registros_efd: &'a HashMap<&'static str, HashMap<u16, (&'static str, Tipo)>>,
    ) -> Self {
        Self {
            inner: reader.split(NEWLINE_BYTE).zip(1..),
            arquivo: arquivo_ref.to_path_buf(),
            registros_efd,
        }
    }
}

impl<'a, R: BufRead> Iterator for EFDLineIterator<'a, R> {
    /// The type of items yielded by this iterator: a `Result` that can contain
    /// a `ValidatedLine` or an `EFDError`.
    type Item = EFDResult<ValidatedLine>;

    /// Advances the iterator and returns the next `ValidatedLine` or an `EFDError`.
    ///
    /// This method continuously attempts to process lines until a valid, non-ignorable
    /// line is found, an error occurs, or the end of the file/iterator is reached.
    ///
    /// The iterator terminates gracefully when it encounters a record "9999".
    ///
    /// # Returns
    /// * `Some(Ok(processed_line))`: A successfully validated line.
    /// * `Some(Err(e))`: A critical `EFDError` encountered during processing.
    /// * `None`: The end of the file/iterator has been reached (including encountering record "9999").
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            // Get the next raw line bytes and its line number from the inner iterator.
            let (line_bytes_result, line_number) = self.inner.next()?; // `?` propagates `None` if inner is exhausted.

            // Attempt to create a `ValidatedLine` from the raw bytes.
            match ValidatedLine::try_from_raw_bytes(
                line_bytes_result,
                line_number,
                &self.arquivo,
                self.registros_efd,
            ) {
                // Successfully processed a line that is not ignorable.
                Ok(Some(processed_line)) => {
                    // Check for the "9999" record, which signifies the logical end of the EFD file.
                    if processed_line
                        .fields
                        .first()
                        .is_some_and(|reg| reg == "9999")
                    {
                        return None; // Terminate iteration.
                    }
                    return Some(Ok(processed_line)); // Yield the valid line.
                }
                // The line was ignorable (e.g., empty, too short, etc.), continue to the next line.
                Ok(None) => continue,
                // A critical error occurred during `try_from_raw_bytes` (e.g., severe decoding error).
                Err(e) => return Some(Err(e)), // Yield the error.
            }
        }
    }
}

//----------------------------------------------------------------------------//
//                                   Tests                                    //
//----------------------------------------------------------------------------//

/// Run tests with:
/// cargo test -- --show-output line_iterator
#[cfg(test)]
mod line_iterator_tests {
    use super::*;
    use crate::{EFDError, Tipo}; // Assuming Tipo and EFDError are in crate root
    use std::io::{self, Cursor, ErrorKind, Read};

    // Helper function to create a dummy registros_efd for tests
    fn create_dummy_registros_efd() -> HashMap<&'static str, HashMap<u16, (&'static str, Tipo)>> {
        let mut map = HashMap::new();
        let mut reg_0000 = HashMap::new();
        reg_0000.insert(0, ("nivel", Tipo::N)); // Placeholder for level
        reg_0000.insert(1, ("COD_VER", Tipo::N));
        map.insert("0000", reg_0000);

        let mut reg_c100 = HashMap::new();
        reg_c100.insert(0, ("nivel", Tipo::N));
        reg_c100.insert(1, ("IND_OPER", Tipo::C));
        map.insert("C100", reg_c100);

        let mut reg_9999 = HashMap::new();
        reg_9999.insert(0, ("nivel", Tipo::N));
        map.insert("9999", reg_9999);

        map
    }

    #[test]
    fn test_efd_line_iterator_basic_iteration() {
        let registros = create_dummy_registros_efd();
        let path = Path::new("test.efd");
        let data = b"|0000|001|\n|C100|0|\n|9999|\n";
        let reader = Cursor::new(data);

        let mut iter = EFDLineIterator::new(reader, path, &registros);

        // First line: |0000|001|
        let line1 = iter.next().unwrap().unwrap();
        assert_eq!(line1.line_number, 1);
        assert_eq!(line1.fields, vec!["0000", "001"]);

        // Second line: |C100|0|
        let line2 = iter.next().unwrap().unwrap();
        assert_eq!(line2.line_number, 2);
        assert_eq!(line2.fields, vec!["C100", "0"]);

        // Third line: |9999| - Should terminate the iterator
        assert!(iter.next().is_none());
    }

    #[test]
    fn test_efd_line_iterator_empty_lines_and_whitespace() {
        let registros = create_dummy_registros_efd();
        let path = Path::new("test.efd");
        let data = b"|0000|001|\n  \n|C100|0|\n\n|9999|\n"; // Empty lines and whitespace
        let reader = Cursor::new(data);

        let mut iter = EFDLineIterator::new(reader, path, &registros);

        let line1 = iter.next().unwrap().unwrap();
        assert_eq!(line1.line_number, 1);
        assert_eq!(line1.fields, vec!["0000", "001"]);

        let line2 = iter.next().unwrap().unwrap();
        assert_eq!(line2.line_number, 3); // Line number should increment past ignored lines
        assert_eq!(line2.fields, vec!["C100", "0"]);

        assert!(iter.next().is_none()); // After |9999|
    }

    #[test]
    fn test_efd_line_iterator_undefined_record_error() {
        let registros = create_dummy_registros_efd();
        let path = Path::new("test.efd");
        let data = b"|0000|001|\n|UNKNOWN|FIELD|\n|C100|0|\n|9999|\n";
        let reader = Cursor::new(data);

        let mut iter = EFDLineIterator::new(reader, path, &registros);

        let line1 = iter.next().unwrap().unwrap();
        assert_eq!(line1.line_number, 1);

        let err = iter.next().unwrap();
        assert!(err.is_err());
        if let Err(EFDError::UndefinedRecord { record, .. }) = err {
            assert_eq!(record, "UNKNOWN");
        } else {
            panic!("Expected EFDError::UndefinedRecord");
        }

        // After an error, the iterator should theoretically continue if possible,
        // but for `UndefinedRecord`, it's propagated directly.
        // Depending on error handling strategy, subsequent calls might fail or yield more errors.
        // For this test, we expect the first error to stop processing this path.
    }

    #[test]
    /// cargo test -- --show-output error_in_split
    fn test_efd_line_iterator_io_error_in_split() {
        struct FaultyReader {
            data: Cursor<&'static [u8]>,
            fault_on_line: usize,
            current_line: usize,
        }

        impl Read for FaultyReader {
            fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
                self.data.read(buf)
            }
        }

        impl BufRead for FaultyReader {
            fn fill_buf(&mut self) -> io::Result<&[u8]> {
                self.data.fill_buf()
            }
            fn consume(&mut self, amt: usize) {
                let prev_pos = self.data.position();
                self.data.consume(amt);
                let current_pos = self.data.position();
                if current_pos > prev_pos {
                    // Approximate line counting for test
                    for &byte in &self.data.get_ref()[(prev_pos as usize)..(current_pos as usize)] {
                        if byte == NEWLINE_BYTE {
                            self.current_line += 1;
                        }
                    }
                }
            }
            fn read_until(&mut self, byte: u8, buf: &mut Vec<u8>) -> io::Result<usize> {
                // Simulate an I/O error on a specific line
                if self.current_line == self.fault_on_line {
                    self.current_line += 1; // Increment anyway to simulate attempting to read
                    return Err(io::Error::other("Simulated I/O error"));
                }
                self.current_line += 1;
                self.data.read_until(byte, buf)
            }
        }

        let registros = create_dummy_registros_efd();
        println!("registros: {registros:#?}");
        let path = Path::new("test.efd");
        let data = b"|0000|foo|\n|c100|bar|\n|9999|\n";
        let faulty_reader = FaultyReader {
            data: Cursor::new(data),
            fault_on_line: 2, // Make it fail on the second logical line read (C100)
            current_line: 0,
        };

        let mut iter = EFDLineIterator::new(faulty_reader, path, &registros);

        let line1 = iter.next().unwrap().unwrap();
        println!("line1: {line1:#?}");
        assert_eq!(line1.line_number, 1);
        assert_eq!(line1.fields, ["0000", "foo"]);

        let line2 = iter.next().unwrap().unwrap();
        println!("line2: {line2:#?}");
        assert_eq!(line2.line_number, 2);
        assert_eq!(line2.fields, ["C100", "bar"]);

        let err = iter.next().unwrap();
        assert!(err.is_err());

        println!("Error: {err:#?}");

        if let Err(EFDError::InOut { source, path }) = err {
            assert_eq!(source.kind(), ErrorKind::Other);
            assert_eq!(path, Path::new("test.efd"));
        } else {
            panic!("Error: {err:?}\nExpected EFDError::Io");
        }

        assert!(iter.next().is_none()); // Iterator should be exhausted after the error and subsequent reads.
    }

    #[test]
    fn test_efd_line_iterator_no_9999_at_end() {
        let registros = create_dummy_registros_efd();
        let path = Path::new("test.efd");
        let data = b"|0000|001|\n|C100|0|\n"; // No 9999 record
        let reader = Cursor::new(data);

        let mut iter = EFDLineIterator::new(reader, path, &registros);

        assert!(iter.next().is_some());
        assert!(iter.next().is_some());
        assert!(iter.next().is_none()); // Should end when underlying reader is exhausted
    }
}
