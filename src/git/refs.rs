use std::{fs, path::Path};

use anyhow::{Context, Result, bail};

use crate::git::repo::GitRepository;

/// Expand abbreviated SHA by searching objects
pub fn resolve_sha(repo: &GitRepository, short: &str) -> Result<String> {
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
pub fn resolve_ref(repo: &GitRepository, refname: &str) -> Result<String> {
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

pub fn ref_create(repo: &GitRepository, ref_name: &str, sha: &str) -> Result<()> {
    let path = repo.gitdir.join(ref_name);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&path, format!("{sha}\n"))
        .with_context(|| format!("Failed to write ref {:?}", path))?;
    Ok(())
}
