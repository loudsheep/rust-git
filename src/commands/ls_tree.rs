use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};

use crate::git::{
    objects::{GitObjectType, object_find, object_read},
    repo::{GitRepository, repo_find},
    tree::GitTree,
};

fn mode_to_type(mode: &str) -> Result<&'static str> {
    let prefix = if mode.len() == 5 {
        &mode[0..1]
    } else {
        &mode[0..2]
    };

    match prefix {
        "04" => Ok("tree"),
        "10" => Ok("blob"),   // regular file
        "12" => Ok("blob"),   // symlink
        "16" => Ok("commit"), // submodule
        _ => bail!("Weird tree leaf mode {mode}"),
    }
}

fn ls_tree(repo: &GitRepository, sha: &str, recursive: bool, prefix: &Path) -> Result<()> {
    let (obj_type, obj) = object_read(&repo, sha)?;

    if obj_type == GitObjectType::Tree {
        let tree = obj
            .as_any()
            .downcast_ref::<GitTree>()
            .context("Failed to downcast to GitTree")?;

        for entry in &tree.entries {
            let otype = mode_to_type(&entry.mode)?;
            let path = prefix.join(&entry.path);

            if !(recursive && otype == "Tree") {
                let padded_mode = format!("{:0>6}", entry.mode);

                println!(
                    "{} {} {}\t{}",
                    padded_mode,
                    otype,
                    hex::encode(entry.sha),
                    path.display()
                );
            } else {
                ls_tree(repo, &hex::encode(entry.sha), recursive, &path)?;
            }
        }
    }

    Ok(())
}

pub fn run(tree: &str, recursive: bool) -> Result<()> {
    let repo = repo_find(".", true)?.unwrap();

    let sha = object_find(&repo, tree, &GitObjectType::Tree);

    ls_tree(&repo, sha, recursive, Path::new(""))
}
