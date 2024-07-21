use std::{collections::HashSet, fs, io};

pub mod cbf;
pub mod encryption;
pub mod split_strings;

pub fn get_files(path: &str) -> io::Result<(HashSet<String>, cbf::FileEntries)> {
    let path = if let Ok(path) = fs::read_dir(path) {
        path
    } else {
        fs::create_dir(path)?;
        fs::read_dir(path)?
    };

    let mut entries = cbf::FileEntries::new();
    let mut file_names = HashSet::new();
    for path in path {
        let path = path?.path();

        if path.is_dir() {
            continue;
        }

        let file_name = path.file_name().unwrap().to_str().unwrap().to_string();

        file_names.insert(file_name.clone());

        let data = fs::read(path)?;
        entries.insert(file_name, data);
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
}
