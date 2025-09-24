use std::fs;

use anyhow::{Context, Result};
use chrono::Utc;

use crate::git::{
    index::{GitIndex, read_index},
    kvlm::Kvlm,
    objects::{GitCommit, GitObjectType, object_write},
    refs::{ref_create, resolve_ref},
    repo::{GitRepository, gitconfig_read, gitconfig_user_get, repo_find},
    tree::{GitTree, GitTreeLeaf},
};

pub fn run(message: &str) -> Result<()> {
    // 1. Find repo
    let repo = repo_find(".", true)?.context("Not a git repository")?;

    // 2. Read index
    let index = read_index(&repo)?;

    // 3. Write tree
    let tree_sha = write_tree(&repo, &index)?;

    // 4. Find parent commit (if HEAD exists)
    let head_ref = repo.gitdir.join("HEAD");
    let parent = if head_ref.exists() {
        let target = fs::read_to_string(&head_ref)?.trim().to_string();
        if target.starts_with("ref:") {
            let refname = target.strip_prefix("ref: ").unwrap();
            // Try to resolve the ref, but it's OK if it doesn't exist for first commit
            match resolve_ref(&repo, refname) {
                Ok(sha) => Some(sha),
                Err(_) => None, // First commit, no parent
            }
        } else if target.len() == 40 {
            Some(target)
        } else {
            None
        }
    } else {
        None
    };

    // 5. Author/committer
    let config = gitconfig_read()?;
    let author = gitconfig_user_get(&config).context("Missing user name/email in git config")?;
    let timestamp = Utc::now().timestamp();
    let tz = "+0000"; // simplify: UTC only

    // 6. Build commit object
    let mut kvlm = Kvlm::new();
    kvlm.headers
        .push((b"tree".to_vec(), tree_sha.as_bytes().to_vec()));
    if let Some(parent_sha) = &parent {
        kvlm.headers
            .push((b"parent".to_vec(), parent_sha.as_bytes().to_vec()));
    }
    kvlm.headers.push((
        b"author".to_vec(),
        format!("{author} {timestamp} {tz}").into_bytes(),
    ));
    kvlm.headers.push((
        b"committer".to_vec(),
        format!("{author} {timestamp} {tz}").into_bytes(),
    ));
    kvlm.message = message.as_bytes().to_vec();

    let commit = GitCommit { kvlm };
    let commit_sha = object_write(&repo, &commit, &GitObjectType::commit, true)?;

    // 7. Update ref
    if head_ref.exists() {
        let target = fs::read_to_string(&head_ref)?.trim().to_string();
        if target.starts_with("ref:") {
            let refname = target.strip_prefix("ref: ").unwrap();
            ref_create(&repo, refname, &commit_sha)?;
        } else {
            fs::write(&head_ref, format!("{commit_sha}\n"))?;
        }
    } else {
        // Create default HEAD pointing to refs/heads/master
        fs::write(&head_ref, "ref: refs/heads/master\n")?;
        ref_create(&repo, "heads/master", &commit_sha)?;
    }

    println!("[{}] {}", &commit_sha[..7], message.trim());

    Ok(())
}

fn write_tree(repo: &GitRepository, index: &GitIndex) -> Result<String> {
    let mut tree = GitTree {
        entries: Vec::new(),
    };

    for entry in &index.entries {
        let mut sha = [0u8; 20];
        hex::decode_to_slice(&entry.sha, &mut sha)?;
        tree.entries.push(GitTreeLeaf {
            mode: "100644".to_string(),
            path: entry.path.clone(),
            sha,
        });
    }

    object_write(repo, &tree, &GitObjectType::tree, true)
}
