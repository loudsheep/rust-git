use anyhow::{Context, Result};
use std::collections::HashSet;

use crate::git::{
    objects::{GitBlob, GitCommit, GitObjectType, object_read, object_write},
    repo::{GitRepository, repo_find},
};

pub fn run(sha: &str) -> Result<()> {
    let repo = repo_find(".", true)?.unwrap();

    println!("digraph wyaglog{{");
    println!("  node[shape=rect]");

    let mut seen = HashSet::<String>::new();
    walk(&repo, sha, &mut seen);

    println!("}}");
    Ok(())
}

fn walk(repo: &GitRepository, sha: &str, seen: &mut HashSet<String>) -> Result<()> {
    if !seen.insert(sha.to_string()) {
        return Ok(());
    }

    let (obj_type, obj) = object_read(repo, sha)?;
    let commit = match obj_type {
        GitObjectType::Commit => {
            let commit = obj
                .as_any()
                .downcast_ref::<GitCommit>()
                .context("Failed to downcast to GitCommit")?;

            commit
            // let msg = String::from_utf8_lossy(&commit.kvlm.message);
            // println!("commit {sha}\n\n  {}", msg.lines().next().unwrap_or(""));
        }
        _ => anyhow::bail!("object {sha} is not a commit"),
    };

    let mut first_line = String::new();
    if let Ok(msg) = String::from_utf8(commit.kvlm.message.clone()) {
        first_line = msg.lines().next().unwrap_or("").to_string();
        first_line = first_line.replace('\\', "\\\\").replace('"', "\\\"");
    }
    println!(
        r#"  c_{s} [label="{short}: {label}"]"#,
        s = sha,
        short = &sha[..7.min(sha.len())],
        label = first_line
    );

    // Parents: all headers with key "parent"
    for (_k, v) in commit
        .kvlm
        .headers
        .iter()
        .filter(|(k, _)| k.as_slice() == b"parent")
    {
        let p = String::from_utf8_lossy(v).to_string();
        println!("  c_{s} -> c_{p};", s = sha, p = p);
        walk(repo, &p, seen)?;
    }

    Ok(())
}
