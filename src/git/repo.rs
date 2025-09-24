use anyhow::{Context, Result};
use ini::Ini;
use std::env;
use std::fs::{self, create_dir};
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct GitRepository {
    pub worktree: PathBuf,
    pub gitdir: PathBuf,
    pub config: Option<RepositoryConfig>,
}

#[derive(Debug)]
pub struct RepositoryConfig {
    pub repository_format_version: u8,
}

impl GitRepository {
    pub fn new<P: AsRef<Path>>(path: P, force: bool) -> Result<Self> {
        let worktree = path.as_ref().to_path_buf();
        let gitdir = worktree.join(".git");

        if !(force || gitdir.is_dir()) {
            anyhow::bail!("Not a rust-git repository: {}", worktree.display());
        }

        if !gitdir.exists() {
            create_dir(&gitdir)?;
        }

        let config_path = gitdir.join("config");
        let config = if config_path.exists() {
            Some(read_config(&config_path)?)
        } else if !force {
            anyhow::bail!("Configuration file missing: {}", config_path.display());
        } else {
            None
        };

        if !force {
            if let Some(cfg) = &config {
                if cfg.repository_format_version != 0 {
                    anyhow::bail!(
                        "Unsupported repositoryformatversion: {}",
                        cfg.repository_format_version
                    );
                }
            }
        }

        Ok(GitRepository {
            worktree,
            gitdir,
            config,
        })
    }

    pub fn create<P: AsRef<Path>>(path: P) -> Result<Self> {
        let worktree = path.as_ref().to_path_buf();
        let repo = GitRepository::new(&worktree, true)?;

        if worktree.exists() {
            if !worktree.is_dir() {
                anyhow::bail!("{} is not a directory", worktree.display());
            }
            if repo.gitdir.read_dir()?.next().is_some() {
                anyhow::bail!("{} is not empty", repo.gitdir.display());
            }
        } else {
            fs::create_dir_all(&worktree)
                .with_context(|| format!("Failed to create directory {}", worktree.display()))?;
        }

        repo.create_dir("branches")?;
        repo.create_dir("objects")?;
        repo.create_dir("refs/tags")?;
        repo.create_dir("refs/heads")?;

        fs::write(
            repo.repo_file("description"),
            "Unnamed repository; edit this file 'description' to name the repository.\n",
        )?;

        fs::write(repo.repo_file("HEAD"), "ref: refs/heads/master\n")?;

        fs::write(
            repo.repo_file("config"),
            "[core]\n\trepositoryformatversion = 0\n\tfilemode = false\n\tbare = false\n",
        )?;

        Ok(repo)
    }

    fn create_dir(&self, path: &str) -> Result<()> {
        fs::create_dir_all(self.repo_path(path))
            .with_context(|| format!("Failed to create directory {}", path))?;
        Ok(())
    }

    pub fn repo_path(&self, path: &str) -> PathBuf {
        self.gitdir.join(path)
    }

    fn repo_file(&self, path: &str) -> PathBuf {
        self.gitdir.join(path)
    }
}

fn read_config(path: &Path) -> Result<RepositoryConfig> {
    let content = fs::read_to_string(path)?;
    let mut version: Option<u8> = None;

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("repositoryformatversion") {
            if let Some(eq_pos) = trimmed.find('=') {
                let num_str = trimmed[(eq_pos + 1)..].trim();
                version = Some(num_str.parse()?);
            }
        }
    }

    Ok(RepositoryConfig {
        repository_format_version: version.unwrap_or(0),
    })
}

pub fn repo_find<P: AsRef<Path>>(path: P, required: bool) -> Result<Option<GitRepository>> {
    let path = fs::canonicalize(path.as_ref())
        .with_context(|| format!("Invalid path: {}", path.as_ref().display()))?;

    if path.join(".git").is_dir() {
        return Ok(Some(GitRepository::new(&path, false)?));
    }

    let parent = path.parent().map(Path::to_path_buf);

    match parent {
        Some(parent_path) if parent_path != path => repo_find(parent_path, required),
        _ => {
            if required {
                anyhow::bail!("No git directory found starting from {}", path.display());
            } else {
                Ok(None)
            }
        }
    }
}

pub fn gitconfig_read() -> Result<Ini> {
    let xdg_config_home = env::var("XDG_CONFIG_HOME").unwrap_or_else(|_| {
        match env::var("HOME") {
            Ok(home) => format!("{}/.config", home),
            Err(_) => {
                // Fallback to current directory if HOME is not set
                ".config".to_string()
            }
        }
    });

    let configfiles = vec![
        PathBuf::from(format!("{xdg_config_home}/git/config")),
        PathBuf::from(match env::var("HOMEPATH") {
            Ok(home) => format!("{}/.gitconfig", home),
            Err(_) => "~/.gitconfig".to_string(),
        }),
    ];

    let mut merged = Ini::new();

    for path in configfiles {
        if path.exists() {
            if let Ok(cfg) = Ini::load_from_file(&path) {
                for (sec, prop) in &cfg {
                    let section = sec.clone();
                    for (k, v) in prop.iter() {
                        merged
                            .with_section(section.clone())
                            .set(k, v);
                    }
                }
            }
        }
    }

    Ok(merged)
}

pub fn gitconfig_user_get(config: &Ini) -> Option<String> {
    if let Some(section) = config.section(Some("user")) {
        if let (Some(name), Some(email)) = (section.get("name"), section.get("email")) {
            return Some(format!("{name} <{email}>"));
        }
    }
    None
}
