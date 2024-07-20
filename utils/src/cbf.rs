use std::{
    collections::HashSet,
    io::{self, Read, Write},
};

pub struct FileEntry {
    pub name: String,
    pub data: Vec<u8>,
}

impl AsRef<FileEntry> for FileEntry {
    fn as_ref(&self) -> &FileEntry {
        self
    }
}

pub fn write<W, T, S>(
    writer: &mut W,
    entries: &[T],
    missing_files: Option<&HashSet<S>>,
) -> io::Result<()>
where
    W: Write,
    T: AsRef<FileEntry>,
    S: AsRef<str>,
{
    if let Some(missing_files) = missing_files {
        writer.write_all(&(missing_files.len() as u16).to_le_bytes())?;

        for missing_file in missing_files {
            let name_length = missing_file.as_ref().len() as u8;
            writer.write_all(&[name_length])?;
            writer.write_all(missing_file.as_ref().as_bytes())?;
        }
    } else {
        writer.write_all(&0u16.to_le_bytes())?;
    }

    for entry in entries {
        let entry = entry.as_ref();
        let file_size = entry.data.len() as u32;
        writer.write_all(&file_size.to_le_bytes())?;

        let name_length = entry.name.len() as u8;
        writer.write_all(&[name_length])?;

        writer.write_all(entry.name.as_bytes())?;

        writer.write_all(&entry.data)?;
    }
    Ok(())
}

pub fn read<R: Read>(reader: &mut R) -> io::Result<(Vec<String>, Vec<FileEntry>)> {
    let mut missing_files = Vec::new();
    let missing_files_count = u16::from_le_bytes(read_n_bytes(reader, 2)?.try_into().unwrap());

    for _ in 0..missing_files_count {
        let name_length = read_n_bytes(reader, 1)?[0] as usize;
        let name_bytes = read_n_bytes(reader, name_length)?;
        let name = String::from_utf8(name_bytes)
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid UTF-8"))?;
        missing_files.push(name);
    }

    let mut entries = Vec::new();
    while let Ok(file_size_bytes) = read_n_bytes(reader, 4) {
        let file_size = u32::from_le_bytes(file_size_bytes.try_into().unwrap());

        let name_length = read_n_bytes(reader, 1)?[0] as usize;
        let name_bytes = read_n_bytes(reader, name_length)?;
        let name = String::from_utf8(name_bytes)
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid UTF-8"))?;
        let mut data = vec![0u8; file_size as usize];
        reader.read_exact(&mut data)?;

        entries.push(FileEntry { name, data });
    }
    Ok((missing_files, entries))
}

fn read_n_bytes<R: Read>(reader: &mut R, n: usize) -> io::Result<Vec<u8>> {
    let mut buffer = vec![0u8; n];
    reader.read_exact(&mut buffer)?;
    Ok(buffer)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_read() {
        let entries = vec![
            FileEntry {
                name: "file1.txt".to_string(),
                data: b"Hello, world!".to_vec(),
            },
            FileEntry {
                name: "file2.bin".to_string(),
                data: vec![0x01, 0x02, 0x03, 0x04],
            },
        ];

        let mut missing_files = HashSet::new();
        missing_files.insert("file3.txt");
        missing_files.insert("file4.bin");

        let mut buffer = Vec::new();
        write(&mut buffer, &entries, Some(&missing_files)).expect("Failed to write custom format");

        let mut cursor = std::io::Cursor::new(buffer);
        let (read_missing_files, read_entries) =
            read(&mut cursor).expect("Failed to read custom format");

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
    fn test_write_read_no_missing_files() {
        let entries = vec![
            FileEntry {
                name: "file1.txt".to_string(),
                data: b"Hello, world!".to_vec(),
            },
            FileEntry {
                name: "file2.bin".to_string(),
                data: vec![0x01, 0x02, 0x03, 0x04],
            },
        ];

        let mut buffer = Vec::new();
        write(&mut buffer, &entries, None::<&HashSet<String>>)
            .expect("Failed to write custom format");

        let mut cursor = std::io::Cursor::new(buffer);
        let (read_missing_files, read_entries) =
            read(&mut cursor).expect("Failed to read custom format");

        assert_eq!(entries.len(), read_entries.len());
        assert_eq!(0, read_missing_files.len());

        for (entry, read_entry) in entries.iter().zip(read_entries.iter()) {
            assert_eq!(entry.name, read_entry.name);
            assert_eq!(entry.data, read_entry.data);
        }
    }
}
