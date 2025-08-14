use anyhow::Context;
use anyhow::Result;
use flate2::{Compression, read::ZlibDecoder, write::ZlibEncoder};
use hex;
use sha1::{Digest, Sha1};
use std::fs;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

use crate::git::repo::GitRepository;

pub trait GitObject {
    fn serialize(&self) -> Result<Vec<u8>>;

    fn deserialize(data: &[u8]) -> Result<Self>
    where
        Self: Sized;

    fn init() -> Result<Self>
    where
        Self: Sized,
    {
        Err(anyhow::anyhow!("Init not implemented for this object type"))
    }
}

pub enum GitObjectType {
    Blob(GitBlob),
    Commit,
    Tree,
    Tag,
}

pub struct GitBlob {
    pub data: Vec<u8>,
}

impl GitObject for GitBlob {
    fn serialize(&self) -> Result<Vec<u8>> {
        Ok(self.data.clone())
    }

    fn deserialize(data: &[u8]) -> Result<Self> {
        Ok(GitBlob {
            data: data.to_vec(),
        })
    }

    fn init() -> Result<Self> {
        Ok(GitBlob { data: Vec::new() })
    }
}

pub fn hash_object(
    repo: &GitRepository,
    obj: &impl GitObject,
    type_name: &str,
    write: bool,
) -> Result<String> {
    let data = obj.serialize()?;
    let header = format!("{} {}\0", type_name, data.len());
    let store_data = [header.as_bytes(), &data[..]].concat();

    let mut hasher = Sha1::new();
    hasher.update(&store_data);
    let hash_bytes = hasher.finalize();
    let hash_hex = hex::encode(hash_bytes);

    if write {
        let dir_path = repo.gitdir.join("objects").join(&hash_hex[..2]);
        let file_path = dir_path.join(&hash_hex[2..]);

        fs::create_dir_all(&dir_path)
            .with_context(|| format!("Failed to create directory {:?}", dir_path))?;

        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&store_data)?;
        let compressed = encoder.finish()?;

        let mut file = File::create(&file_path)
            .with_context(|| format!("Failed to create file {:?}", file_path))?;
        file.write_all(&compressed)?;
    }

    Ok(hash_hex)
}

pub fn cat_file(repo: &GitRepository, hash: &str) -> Result<GitObjectType> {
    let path = repo
        .gitdir
        .join("objects")
        .join(&hash[..2])
        .join(&hash[2..]);

    let compressed =
        fs::read(&path).with_context(|| format!("Failed to read object file at {:?}", path))?;

    let mut decoder = ZlibDecoder::new(&compressed[..]);
    let mut decompressed = Vec::new();
    decoder.read_to_end(&mut decompressed)?;

    let null_pos = decompressed
        .iter()
        .position(|&b| b == 0)
        .context("Invalid object format: missing header null byte")?;
    let header = &decompressed[..null_pos];
    let content = &decompressed[null_pos + 1..];

    let header_str = String::from_utf8_lossy(header);
    let mut header_parts = header_str.split_whitespace();
    let type_name = header_parts
        .next()
        .context("Invalid object header: missing type")?;

    match type_name {
        "blob" => Ok(GitObjectType::Blob(GitBlob::deserialize(content)?)),
        "commit" => Ok(GitObjectType::Commit),
        "tree" => Ok(GitObjectType::Tree),
        "tag" => Ok(GitObjectType::Tag),
        _ => Err(anyhow::anyhow!("Unknown object type: {}", type_name)),
    }
}
