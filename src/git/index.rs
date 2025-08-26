use anyhow::{Context, Result, bail};
use byteorder::{BigEndian, ReadBytesExt};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::PathBuf;

use crate::git::repo::GitRepository;

#[derive(Debug)]
pub struct GitIndexEntry {
    pub ctime: u32,
    pub mtime: u32,
    pub dev: u32,
    pub ino: u32,
    pub mode: u32,
    pub uid: u32,
    pub gid: u32,
    pub size: u32,
    pub sha: String,
    pub flags: u16,
    pub path: String,
}

pub fn read_index(repo: &GitRepository) -> Result<Vec<GitIndexEntry>> {
    let index_path = repo.gitdir.join("index");
    if !index_path.exists() {
        return Ok(Vec::new());
    }

    let mut f = File::open(&index_path)
        .with_context(|| format!("Could not open index at {:?}", index_path))?;

    let mut signature = [0u8; 4];
    f.read_exact(&mut signature)?;
    if &signature != b"DIRC" {
        bail!("Invalid index signature: {:?}", signature);
    }

    let version = f.read_u32::<BigEndian>()?;
    if version != 2 {
        bail!("Unsupported index version: {version}");
    }

    let num_entries = f.read_u32::<BigEndian>()?;

    let mut entries = Vec::with_capacity(num_entries as usize);

    for _ in 0..num_entries {
        // stat fields
        let ctime = f.read_u32::<BigEndian>()?;
        f.read_u32::<BigEndian>()?; // ctime nanosec, ignore
        let mtime = f.read_u32::<BigEndian>()?;
        f.read_u32::<BigEndian>()?; // mtime nanosec, ignore
        let dev = f.read_u32::<BigEndian>()?;
        let ino = f.read_u32::<BigEndian>()?;
        let mode = f.read_u32::<BigEndian>()?;
        let uid = f.read_u32::<BigEndian>()?;
        let gid = f.read_u32::<BigEndian>()?;
        let size = f.read_u32::<BigEndian>()?;

        let mut sha_buf = [0u8; 20];
        f.read_exact(&mut sha_buf)?;
        let sha = hex::encode(sha_buf);

        let flags = f.read_u16::<BigEndian>()?;

        // path (null-terminated string, padded to 8-byte boundary)
        let mut path_bytes = Vec::new();
        loop {
            let mut byte = [0u8; 1];
            f.read_exact(&mut byte)?;
            if byte[0] == 0 {
                break;
            }
            path_bytes.push(byte[0]);
        }
        let path = String::from_utf8(path_bytes).context("Invalid UTF-8 in index path")?;

        // align to 8 bytes
        let entry_len = 62 + path.len() + 1; // base + path + null
        let padding = (8 - (entry_len % 8)) % 8;
        f.seek(SeekFrom::Current(padding as i64))?;

        entries.push(GitIndexEntry {
            ctime,
            mtime,
            dev,
            ino,
            mode,
            uid,
            gid,
            size,
            sha,
            flags,
            path,
        });
    }

    Ok(entries)
}
