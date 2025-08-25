use std::{fs, path::Path};

use anyhow::{Context, Result, bail};

use crate::git::repo::GitRepository;

/// Resolve a "name" (HEAD, branch, tag, SHA) to a full 40-hex SHA1.
pub fn ref_parse(repo: &GitRepository, name: &str) -> Result<String> {
    if name.chars().all(|c| c.is_ascii_hexdigit()) && (4..=40).contains(&name.len()) {
        return resolve_sha(repo, name);
    }

    match name {
        "HEAD" => {
            let head_path = repo.gitdir.join("HEAD");
            let data = fs::read_to_string(&head_path)
                .with_context(|| format!("Failed to read {:?}", head_path))?;

            if data.starts_with("red: ") {
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

/// Expand abbreviated SHA by searching objects
fn resolve_sha(repo: &GitRepository, short: &str) -> Result<String> {
    if short.len() == 40 {
        return Ok(short.to_string());
    }

    let dir = repo.gitdir.join("objects").join(&short[..2]);
    if !dir.exists() {
        bail!("No objects found for prefix {short}");
    }

    let mut matches = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let name = entry.file_name().into_string().unwrap();

        let candidate = format!("{}{}", &short[..2], name);
        if candidate.starts_with(short) {
            matches.push(candidate);
        }
    }

    match matches.len() {
        0 => bail!("No objects found for prefix {short}"),
        1 => Ok(matches.remove(0)),
        _ => bail!("Ambiguous prefix {short}, matches {:?}", matches),
    }
}

/// Resolve a symbolic ref like "refs/heads/main"
fn resolve_ref(repo: &GitRepository, refname: &str) -> Result<String> {
    let ref_path = repo.gitdir.join(refname);
    if ref_path.exists() {
        let sha = fs::read_to_string(&ref_path)?.trim().to_string();
        Ok(sha)
    } else {
        bail!("Invalid ref: {refname}")
    }
}

pub fn collect_refs(base: &Path, prefix: &str) -> Result<Vec<(String, String)>> {
    let mut refs = Vec::new();

    for entry in fs::read_dir(base)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            let subprefix = format!("{}/{}", prefix, entry.file_name().to_string_lossy());
            refs.extend(collect_refs(&path, &subprefix)?);
        } else {
            let sha = fs::read_to_string(&path)
                .with_context(|| format!("Failed to read {:?}", path))?
                .trim()
                .to_string();
            let refname = format!("{}/{}", prefix, entry.file_name().to_string_lossy());
            refs.push((sha, refname));
        }
    }

    Ok(refs)
}