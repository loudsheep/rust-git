use anyhow::Context;
use anyhow::Result;
use anyhow::bail;
use clap::ValueEnum;
use flate2::{Compression, read::ZlibDecoder, write::ZlibEncoder};
use hex;
use sha1::{Digest, Sha1};
use std::any::Any;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::io::Write;

use crate::git::kvlm::Kvlm;
use crate::git::kvlm::kvlm_parse;
use crate::git::kvlm::kvlm_serialize;
use crate::git::refs::resolve_ref;
use crate::git::refs::resolve_sha;
use crate::git::repo::GitRepository;
use crate::git::tree::GitTree;

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

    fn as_any(&self) -> &dyn Any;
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
pub enum GitObjectType {
    Blob,
    Commit,
    Tree,
    Tag,
}

pub struct GitBlob {
    pub data: Vec<u8>,
}

pub struct GitCommit {
    pub kvlm: Kvlm,
}

pub struct GitTag {
    pub kvlm: Kvlm,
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

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl GitObject for GitCommit {
    fn serialize(&self) -> Result<Vec<u8>> {
        Ok(kvlm_serialize(&self.kvlm))
    }

    fn deserialize(data: &[u8]) -> Result<Self> {
        let kvlm = kvlm_parse(data)?;
        Ok(Self { kvlm })
    }

    fn init() -> Result<Self> {
        Ok(Self { kvlm: Kvlm::new() })
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl GitObject for GitTag {
    fn serialize(&self) -> Result<Vec<u8>> {
        Ok(kvlm_serialize(&self.kvlm))
    }

    fn deserialize(data: &[u8]) -> Result<Self> {
        let kvlm = kvlm_parse(data)?;
        Ok(Self { kvlm })
    }

    fn init() -> Result<Self> {
        Ok(Self { kvlm: Kvlm::new() })
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub fn object_find(repo: &GitRepository, name: &str, fmt: Option<GitObjectType>) -> Result<String> {
    if name == "HEAD" {
        let head = fs::read_to_string(repo.gitdir.join("HEAD")).context("Could not read HEAD")?;

        if head.starts_with("ref: ") {
            let ref_path = head[5..].trim();
            return resolve_ref(&repo, &ref_path);
        } else {
            return Ok(head.trim().to_string());
        }
    }

    let sha = resolve_sha(repo, name)?;

    if let Some(expected) = fmt {
        let (got_type, obj) = object_read(&repo, &sha)?;

        if got_type != expected {
            bail!(
                "Object {} is not of expected type {:?}, got {:?}",
                sha,
                expected,
                got_type
            );
        }
    }

    Ok(sha)
}

pub fn object_write(
    repo: &GitRepository,
    obj: &impl GitObject,
    type_name: &GitObjectType,
    write: bool,
) -> Result<String> {
    let data = obj.serialize()?;
    let header = format!("{:?} {}\0", &type_name, data.len());
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

pub fn object_read(repo: &GitRepository, sha: &str) -> Result<(GitObjectType, Box<dyn GitObject>)> {
    let path = repo.gitdir.join("objects").join(&sha[..2]).join(&sha[2..]);

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
        "Blob" => {
            let obj = GitBlob::deserialize(content)?;
            Ok((GitObjectType::Blob, Box::new(obj)))
        }
        "Commit" => {
            let obj = GitCommit::deserialize(content)?;
            Ok((GitObjectType::Commit, Box::new(obj)))
        }
        "Tree" => {
            let obj = GitTree::deserialize(content)?;
            Ok((GitObjectType::Tree, Box::new(obj)))
        }
        "Tag" => Err(anyhow::anyhow!("Tag object not yet implemented")),
        _ => Err(anyhow::anyhow!("Unknown object type: {}", type_name)),
    }
}
