pub mod cbf;
pub mod split_strings;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_read() {
        let entries = vec![
            cbf::FileEntry {
                name: "file1.txt".to_string(),
                data: b"Hello, world!".to_vec(),
            },
            cbf::FileEntry {
                name: "file2.bin".to_string(),
                data: vec![0x01, 0x02, 0x03, 0x04],
            },
        ];

        let mut buffer = Vec::new();
        cbf::write(&mut buffer, &entries).expect("Failed to write custom format");

        let mut cursor = std::io::Cursor::new(buffer);
        let read_entries = cbf::read(&mut cursor).expect("Failed to read custom format");

        assert_eq!(entries.len(), read_entries.len());
        for (entry, read_entry) in entries.iter().zip(read_entries.iter()) {
            assert_eq!(entry.name, read_entry.name);
            assert_eq!(entry.data, read_entry.data);
        }
    }

    #[test]
    fn test_split_strings() {
        let input = "file1|file2|file3";
        let mut iter = split_strings::SplitStrings::new(input, '|');

        assert_eq!(iter.next(), Some("file1".to_string()));
        assert_eq!(iter.next(), Some("file2".to_string()));
        assert_eq!(iter.next(), Some("file3".to_string()));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_empty_string() {
        let input = "";
        let mut iter = split_strings::SplitStrings::new(input, '|');

        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_no_delimiter() {
        let input = "singlefile";
        let mut iter = split_strings::SplitStrings::new(input, '|');

        assert_eq!(iter.next(), Some("singlefile".to_string()));
        assert_eq!(iter.next(), None);
    }
}