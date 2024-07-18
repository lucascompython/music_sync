use std::io::{self, Read, Write};

pub struct FileEntry {
    pub name: String,
    pub data: Vec<u8>,
}

pub fn write<W: Write>(writer: &mut W, entries: &[FileEntry]) -> io::Result<()> {
    for entry in entries {
        let file_size = entry.data.len() as u32;
        writer.write_all(&file_size.to_le_bytes())?;

        let name_length = entry.name.len() as u8;
        writer.write_all(&[name_length])?;

        writer.write_all(entry.name.as_bytes())?;

        writer.write_all(&entry.data)?;
    }
    Ok(())
}

/// Reads file entries from a reader in the custom binary format.
pub fn read<R: Read>(reader: &mut R) -> io::Result<Vec<FileEntry>> {
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
    Ok(entries)
}

fn read_n_bytes<R: Read>(reader: &mut R, n: usize) -> io::Result<Vec<u8>> {
    let mut buffer = vec![0u8; n];
    reader.read_exact(&mut buffer)?;
    Ok(buffer)
}
