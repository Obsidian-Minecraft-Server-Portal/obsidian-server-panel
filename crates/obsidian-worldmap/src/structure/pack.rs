//! Region packs: the section records a region writes, in one file, with an index so one section can be read on its own.
//! Little-endian. Layout: magic `OBSR`, version, reserved, the entry count as u32, the index offset as u64, the records one
//! after another, then the index: for each entry its chunk x, section y and chunk z as i32, then its offset and length.

use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{ErrorKind, Read, Seek, SeekFrom, Write};
use std::path::Path;
use std::thread;
use std::time::Duration;

use crate::error::{Result, WorldmapError};

const MAGIC: [u8; 4] = *b"OBSR";
const VERSION: u16 = 1;
const HEADER_LEN: u64 = 20;
const ENTRY_LEN: usize = 24;

/// The chunk x, section y and chunk z that name a section in a pack.
pub type SectionKey = [i32; 3];

/// Writes a pack at `destination`. `fill` pushes the records, and the file appears only once it is complete.
pub(super) fn write_pack(destination: &Path, fill: impl FnOnce(&mut PackWriter) -> Result<()>) -> Result<usize> {
    let temporary = destination.with_extension("obsr.tmp");
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut writer = PackWriter::create(&temporary)?;
    let result = fill(&mut writer).and_then(|()| writer.finish(&temporary, destination));
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
}

pub(super) struct PackWriter {
    file: File,
    entries: Vec<(SectionKey, u64, u32)>,
    position: u64,
}

impl PackWriter {
    fn create(path: &Path) -> Result<Self> {
        let mut file = File::create(path)?;
        let mut header = [0_u8; HEADER_LEN as usize];
        header[..4].copy_from_slice(&MAGIC);
        header[4..6].copy_from_slice(&VERSION.to_le_bytes());
        file.write_all(&header)?;
        Ok(Self {
            file,
            entries: Vec::new(),
            position: HEADER_LEN,
        })
    }

    pub(super) fn push(&mut self, key: SectionKey, record: &[u8]) -> Result<()> {
        let length = u32::try_from(record.len()).map_err(|_| WorldmapError::Structure("section record is too large".into()))?;
        self.file.write_all(record)?;
        self.entries.push((key, self.position, length));
        self.position += u64::from(length);
        Ok(())
    }

    fn finish(mut self, temporary: &Path, destination: &Path) -> Result<usize> {
        let count = u32::try_from(self.entries.len()).map_err(|_| WorldmapError::Structure("region pack has too many sections".into()))?;
        let index_offset = self.position;
        let mut index = Vec::with_capacity(self.entries.len() * ENTRY_LEN);
        for (key, offset, length) in &self.entries {
            for coordinate in key {
                index.extend_from_slice(&coordinate.to_le_bytes());
            }
            index.extend_from_slice(&offset.to_le_bytes());
            index.extend_from_slice(&length.to_le_bytes());
        }
        self.file.write_all(&index)?;
        self.file.seek(SeekFrom::Start(8))?;
        self.file.write_all(&count.to_le_bytes())?;
        self.file.write_all(&index_offset.to_le_bytes())?;
        self.file.sync_all()?;
        drop(self.file);
        replace_file(temporary, destination)?;
        Ok(self.entries.len())
    }
}

/// Moves a finished pack into place. Windows will not replace a file that a bundle request still has open, so this retries
/// for a moment, since a request holds the file for milliseconds.
fn replace_file(temporary: &Path, destination: &Path) -> Result<()> {
    for _ in 0..20 {
        if fs::rename(temporary, destination).is_ok() {
            return Ok(());
        }
        thread::sleep(Duration::from_millis(25));
    }
    fs::rename(temporary, destination)?;
    Ok(())
}

/// A pack opened for reading. The index is read once, and a section is then read by its offset.
pub(super) struct PackReader {
    file: File,
    index: HashMap<SectionKey, (u64, u32)>,
}

impl PackReader {
    /// `None` when no pack exists at `path`.
    pub(super) fn open(path: &Path) -> Result<Option<Self>> {
        let mut file = match File::open(path) {
            Ok(file) => file,
            Err(error) if error.kind() == ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(error.into()),
        };
        let length = file.metadata()?.len();
        let mut header = [0_u8; HEADER_LEN as usize];
        file.read_exact(&mut header).map_err(|_| malformed())?;
        if header[..4] != MAGIC || u16_at(&header, 4) != VERSION {
            return Err(malformed());
        }
        let count = u64::from(u32_at(&header, 8));
        let index_offset = u64_at(&header, 12);
        if index_offset < HEADER_LEN || index_offset > length || count * ENTRY_LEN as u64 > length - index_offset {
            return Err(malformed());
        }
        file.seek(SeekFrom::Start(index_offset))?;
        let mut bytes = vec![0_u8; count as usize * ENTRY_LEN];
        file.read_exact(&mut bytes).map_err(|_| malformed())?;
        let mut index = HashMap::with_capacity(count as usize);
        for entry in bytes.as_chunks::<ENTRY_LEN>().0 {
            let key = [i32_at(entry, 0), i32_at(entry, 4), i32_at(entry, 8)];
            let (offset, record) = (u64_at(entry, 12), u32_at(entry, 20));
            if offset < HEADER_LEN || offset + u64::from(record) > index_offset {
                return Err(malformed());
            }
            index.insert(key, (offset, record));
        }
        Ok(Some(Self { file, index }))
    }

    /// The record of the section at `key`, or `None` when the pack holds no such section.
    pub(super) fn section(&mut self, key: SectionKey) -> Result<Option<Vec<u8>>> {
        let Some(&(offset, length)) = self.index.get(&key) else { return Ok(None) };
        self.file.seek(SeekFrom::Start(offset))?;
        let mut record = vec![0_u8; length as usize];
        self.file.read_exact(&mut record)?;
        Ok(Some(record))
    }
}

fn malformed() -> WorldmapError {
    WorldmapError::Structure("region pack is malformed".into())
}

fn u16_at(bytes: &[u8], at: usize) -> u16 {
    u16::from_le_bytes([bytes[at], bytes[at + 1]])
}

fn u32_at(bytes: &[u8], at: usize) -> u32 {
    u32::from_le_bytes([bytes[at], bytes[at + 1], bytes[at + 2], bytes[at + 3]])
}

fn i32_at(bytes: &[u8], at: usize) -> i32 {
    u32_at(bytes, at) as i32
}

fn u64_at(bytes: &[u8], at: usize) -> u64 {
    u64::from_le_bytes([
        bytes[at],
        bytes[at + 1],
        bytes[at + 2],
        bytes[at + 3],
        bytes[at + 4],
        bytes[at + 5],
        bytes[at + 6],
        bytes[at + 7],
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write(destination: &Path, sections: &[(SectionKey, &[u8])]) -> Result<usize> {
        write_pack(destination, |pack| {
            for (key, record) in sections {
                pack.push(*key, record)?;
            }
            Ok(())
        })
    }

    #[test]
    fn reads_each_section_back_by_its_key() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("r.0.0.obsr");
        let written = write(&path, &[([0, 0, 0], &b"abc"[..]), ([5, -3, 7], &b"hello"[..])]).unwrap();
        assert_eq!(written, 2);
        let mut pack = PackReader::open(&path).unwrap().unwrap();
        assert_eq!(pack.section([5, -3, 7]).unwrap().as_deref(), Some(&b"hello"[..]));
        assert_eq!(pack.section([0, 0, 0]).unwrap().as_deref(), Some(&b"abc"[..]));
        assert_eq!(pack.section([1, 1, 1]).unwrap(), None);
    }

    #[test]
    fn a_missing_file_is_no_pack() {
        let dir = tempfile::tempdir().unwrap();
        assert!(PackReader::open(&dir.path().join("r.9.9.obsr")).unwrap().is_none());
    }

    #[test]
    fn a_failed_write_leaves_no_pack_behind() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("r.0.0.obsr");
        let result = write_pack(&path, |pack| {
            pack.push([0, 0, 0], b"abc")?;
            Err(WorldmapError::Structure("stopped".into()))
        });
        assert!(result.is_err());
        assert!(!path.exists());
        assert!(!dir.path().join("r.0.0.obsr.tmp").exists());
    }

    #[test]
    fn rejects_a_file_that_is_not_a_pack() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("r.0.0.obsr");
        fs::write(&path, b"OBSV and then some more bytes").unwrap();
        assert!(PackReader::open(&path).is_err());
    }
}
