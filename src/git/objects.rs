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
    blob,
    commit,
    tree,
    tag,
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

/// Resolve a "name" (HEAD, branch, tag, SHA) to a full 40-hex SHA1.
pub fn object_resolve(repo: &GitRepository, name: &str) -> Result<String> {
    if name.chars().all(|c| c.is_ascii_hexdigit()) && (4..=40).contains(&name.len()) {
        return resolve_sha(repo, name);
    }

    match name {
        "HEAD" => {
            let head_path = repo.gitdir.join("HEAD");
            let data = fs::read_to_string(&head_path)
                .with_context(|| format!("Failed to read {:?}", head_path))?;

            if data.starts_with("ref: ") {
                let refname = data[5..].trim();
                return resolve_ref(repo, refname);
            } else {
                return Ok(data.trim().to_string());
            }
        }
        _ => {
            let ref_path = repo.gitdir.join(name);
            if ref_path.exists() {
                let sha = fs::read_to_string(&ref_path)?.trim().to_string();
                return Ok(sha);
            }

            for prefix in &["refs/heads", "refs/tags", "refs/remotes"] {
                let ref_path = repo.gitdir.join(prefix).join(name);
                if ref_path.exists() {
                    let sha = fs::read_to_string(&ref_path)?.trim().to_string();
                    return Ok(sha);
                }
            }
        }
    }

    bail!("Not a valid object name: {name}")
}

pub fn object_find(repo: &GitRepository, name: &str, fmt: Option<GitObjectType>) -> Result<String> {
    let sha = object_resolve(repo, name)?;

    if let Some(expected) = fmt {
        let (got_type, _) = object_read(&repo, &sha)?;

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

pub fn object_hash(repo: &GitRepository, data: Vec<u8>, type_name: &GitObjectType) -> Result<String> {

    let obj: Box<dyn GitObject> = match &type_name {
        GitObjectType::blob => Box::new(GitBlob::deserialize(&data)?),
        GitObjectType::commit => Box::new(GitCommit::deserialize(&data)?),
        GitObjectType::tree => Box::new(GitTree::deserialize(&data)?),
        GitObjectType::tag => Box::new(GitTag::deserialize(&data)?),
    };

    return object_write(&repo, obj.as_ref(), &type_name, true);
} 

pub fn object_write(
    repo: &GitRepository,
    obj: &dyn GitObject,
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
        "blob" => {
            let obj = GitBlob::deserialize(content)?;
            Ok((GitObjectType::blob, Box::new(obj)))
        }
        "commit" => {
            let obj = GitCommit::deserialize(content)?;
            Ok((GitObjectType::commit, Box::new(obj)))
        }
        "tree" => {
            let obj = GitTree::deserialize(content)?;
            Ok((GitObjectType::tree, Box::new(obj)))
        }
        "tag" => Err(anyhow::anyhow!("Tag object not yet implemented")),
        _ => Err(anyhow::anyhow!("Unknown object type: {}", type_name)),
    }
}
