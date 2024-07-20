use std::{collections::HashSet, fs, io};

pub mod cbf;
pub mod split_strings;

pub fn get_files() -> io::Result<(HashSet<String>, Vec<cbf::FileEntry>)> {
    let path = if let Ok(path) = fs::read_dir("music") {
        path
    } else {
        fs::create_dir("music")?;
        fs::read_dir("music")?
    };

    let mut entries = Vec::new();
    let mut file_names = HashSet::new();
    for path in path {
        let path = path?.path();
        let file_name = path.file_name().unwrap().to_str().unwrap().to_string();

        file_names.insert(file_name.clone());

        let data = fs::read(path)?;
        entries.push(cbf::FileEntry {
            name: file_name,
            data,
        });
    }

    Ok((file_names, entries))
}

pub fn join_hashset<S>(set: &HashSet<S>, separator: char) -> String
where
    S: AsRef<str>,
{
    let num_elements = set.len();
    if num_elements == 0 {
        return String::new();
    }

    let separator_len = 1;
    let total_length: usize =
        set.iter().map(|s| s.as_ref().len()).sum::<usize>() + (num_elements - 1) * separator_len;

    let mut joined = String::with_capacity(total_length);

    let mut set_iter = set.iter();

    if let Some(first) = set_iter.next() {
        joined.push_str(first.as_ref());
    }

    for item in set_iter {
        joined.push(separator);
        joined.push_str(item.as_ref());
    }

    joined
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_join_hashset() {
        let set = ["file1", "file2", "file3"];
        let set = set
            .iter()
            .map(|s| s.to_string())
            .collect::<HashSet<String>>();

        let result = join_hashset(&set, '|');
        let result_set = result.split('|').collect::<HashSet<&str>>();

        // convert original set to a HashSet<&str> for comparison
        let expected_set = set.iter().map(String::as_str).collect::<HashSet<&str>>();

        assert_eq!(result_set, expected_set);
    }

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

        let mut missing_files = HashSet::new();
        missing_files.insert("file3.txt");
        missing_files.insert("file4.bin");

        let mut buffer = Vec::new();
        cbf::write(&mut buffer, &entries, Some(&missing_files))
            .expect("Failed to write custom format");

        let mut cursor = std::io::Cursor::new(buffer);
        let (read_missing_files, read_entries) =
            cbf::read(&mut cursor).expect("Failed to read custom format");

        assert_eq!(entries.len(), read_entries.len());
        assert_eq!(missing_files.len(), read_missing_files.len());

        for ((entry, read_entry), (missing_file, read_missing_file)) in entries
            .iter()
            .zip(read_entries.iter())
            .zip(missing_files.iter().zip(read_missing_files.iter()))
        {
            assert_eq!(entry.name, read_entry.name);
            assert_eq!(entry.data, read_entry.data);
            assert_eq!(missing_file, read_missing_file);
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
