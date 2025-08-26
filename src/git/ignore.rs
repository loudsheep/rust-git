use anyhow::{Result, bail};
use std::fs;
use std::path::Path;
use std::{collections::HashMap, env, path::PathBuf};
use wildmatch::WildMatch;

use crate::git::index::read_index;
use crate::git::objects::object_read;
use crate::git::repo::GitRepository;

pub struct GitIgnore {
    pub absolute: Vec<Vec<(String, bool)>>, // Vec of rulesets
    pub scoped: HashMap<String, Vec<(String, bool)>>, // dir -> ruleset
}

impl GitIgnore {
    pub fn new(
        absolute: Vec<Vec<(String, bool)>>,
        scoped: HashMap<String, Vec<(String, bool)>>,
    ) -> Self {
        GitIgnore { absolute, scoped }
    }
}

fn gitignore_parse1(raw: &str) -> Option<(String, bool)> {
    let raw = raw.trim();

    if raw.is_empty() || raw.starts_with("#") {
        None
    } else if raw.starts_with('!') {
        Some((raw[1..].to_string(), false))
    } else if raw.starts_with('\\') {
        Some((raw[1..].to_string(), true))
    } else {
        Some((raw.to_string(), true))
    }
}

fn gitignore_parse(lines: &[&str]) -> Vec<(String, bool)> {
    let mut res = Vec::new();
    for line in lines {
        if let Some(rule) = gitignore_parse1(line) {
            res.push(rule);
        }
    }

    res
}

pub fn gitignore_read(repo: &GitRepository) -> Result<GitIgnore> {
    let mut gi = GitIgnore::new(Vec::new(), HashMap::new());

    let repo_file = repo.gitdir.join("info").join("exclude");
    if repo_file.exists() {
        let contents = fs::read_to_string(&repo_file)?;
        let lines: Vec<&str> = contents.lines().collect();
        gi.absolute.push(gitignore_parse(&lines));
    }

    let config_home = env::var("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| dirs::home_dir().unwrap().join(".config"));
    let global_file = config_home.join("git").join("ignore");
    if global_file.exists() {
        let contents = fs::read_to_string(&global_file)?;
        let lines: Vec<&str> = contents.lines().collect();
        gi.absolute.push(gitignore_parse(&lines));
    }

    let index = read_index(repo)?;
    for entry in &index {
        if entry.path == ".gitignore" || entry.path.ends_with("/.gitignore") {
            let dir_name = Path::new(&entry.path)
                .parent()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default();

            let (obj_type, obj) = object_read(repo, &entry.sha)?;
            let contents = String::from_utf8(obj.serialize()?)?;
            let lines: Vec<&str> = contents.lines().collect();
            gi.scoped.insert(dir_name, gitignore_parse(&lines));
        }
    }

    Ok(gi)
}

fn check_ignore1(rules: &[(String, bool)], path: &str) -> Option<bool> {
    let mut result = None;
    for (pattern, value) in rules {
        let matcher = WildMatch::new(pattern);
        if matcher.matches(path) {
            result = Some(*value);
        }
    }
    result
}

fn check_ignore_scoped(rules: &HashMap<String, Vec<(String, bool)>>, path: &str) -> Option<bool> {
    let mut parent = Path::new(path)
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_default();

    loop {
        if let Some(rule_set) = rules.get(&parent.to_string_lossy().to_string()) {
            if let Some(result) = check_ignore1(rule_set, path) {
                return Some(result);
            }
        }
        if parent.as_os_str().is_empty() {
            break;
        }
        if !parent.pop() {
            break;
        }
    }

    None
}

fn check_ignore_absolute(rules: &[Vec<(String, bool)>], path: &str) -> bool {
    for ruleset in rules {
        if let Some(result) = check_ignore1(ruleset, path) {
            return result;
        }
    }
    false
}

pub fn check_ignore(rules: &GitIgnore, path: &str) -> Result<bool> {
    if Path::new(path).is_absolute() {
        bail!("check_ignore requires path to be relative to the repo root");
    }

    if let Some(result) = check_ignore_scoped(&rules.scoped, path) {
        return Ok(result);
    }

    Ok(check_ignore_absolute(&rules.absolute, path))
}
