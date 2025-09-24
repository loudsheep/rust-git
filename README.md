# rust-git

A simple Git version control system implementation in Rust, providing core Git functionality with a command-line interface.

## Features

rust-git implements the following Git commands:

### Repository Management
- `init` - Initialize a new repository
- `status` - Show the working tree status 

### Object Operations
- `hash-object` - Compute object ID and optionally creates a blob from a file
- `cat-file` - Provide content of repository objects
- `ls-tree` - List the contents of a tree object

### Index Operations  
- `add` - Add file contents to the index
- `rm` - Remove files from the working tree and from the index
- `ls-files` - Show information about files in the index

### Commit Operations
- `commit` - Record changes to the repository
- `log` - Show commit history (outputs GraphViz format)

### Branch and Reference Operations
- `checkout` - Switch branches or restore working tree files
- `show-ref` - List references in the repository
- `rev-parse` - Parse revision (or other objects) identifier
- `tag` - Create, list, or verify tags

### Advanced Operations
- `check-ignore` - Check if paths are ignored by .gitignore rules

## Installation

### Prerequisites
- Rust (2024 edition)
- Cargo

### Build from Source
```bash
git clone https://github.com/loudsheep/rust-git.git
cd rust-git
cargo build --release
```

The binary will be available at `target/release/rust-git`.

## Usage

### Basic Workflow

1. **Initialize a repository:**
```bash
rust-git init [path]
```

2. **Configure user (using system git):**
```bash
git config --global user.name "Your Name"
git config --global user.email "your.email@example.com"
```

3. **Add files to the index:**
```bash
rust-git add file1.txt file2.txt
```

4. **Check staged files:**
```bash
rust-git ls-files
```

5. **Commit changes:**
```bash
rust-git commit -m "Your commit message"
```

6. **View commit history:**
```bash
rust-git log HEAD
```

7. **Remove files:**
```bash
rust-git rm file1.txt
```

### Example Session

```bash
# Initialize a new repository
$ rust-git init my-project
Initialized empty rust-git repository

$ cd my-project

# Create and add some files
$ echo "Hello World" > hello.txt
$ echo "# My Project" > README.md
$ rust-git add hello.txt README.md

# Check what's staged
$ rust-git ls-files
README.md
hello.txt

# Make initial commit
$ rust-git commit -m "Initial commit"
[a1b2c3d] Initial commit

# View the commit log
$ rust-git log HEAD
digraph wyaglog{
  node[shape=rect]
  c_a1b2c3d [label="a1b2c3d: Initial commit"]
}

# Remove a file
$ rust-git rm hello.txt
$ rust-git commit -m "Remove hello.txt"
[e4f5g6h] Remove hello.txt
```

### Command Reference

#### Repository Operations
```bash
# Initialize repository
rust-git init [path]

# Show repository status  
rust-git status
```

#### File Operations
```bash
# Add files to index
rust-git add <file1> [file2] [...]

# Remove files from index and working tree
rust-git rm <file1> [file2] [...]

# List files in index
rust-git ls-files
```

#### Commit Operations
```bash
# Create commit
rust-git commit -m "message"

# View commit history (GraphViz format)
rust-git log <commit-sha>
```

#### Object Inspection
```bash
# Create/inspect objects
rust-git hash-object [-w] [-t <type>] <file>
rust-git cat-file <type> <object-sha>

# List tree contents
rust-git ls-tree [--recursive] <tree-sha>
```

#### References and Tags
```bash
# Parse references
rust-git rev-parse <ref>

# Show all references
rust-git show-ref

# Create/list tags
rust-git tag                    # list tags
rust-git tag <name> [object]    # create tag
rust-git tag -a <name> [object] # create annotated tag
```

#### Advanced Operations
```bash
# Checkout commit/branch
rust-git checkout <commit-sha>

# Check ignore patterns
rust-git check-ignore <path1> [path2] [...]
```

## Configuration

rust-git uses the standard Git configuration system. You need to configure your user name and email:

```bash
git config --global user.name "Your Name"
git config --global user.email "your.email@example.com"
```

## Repository Structure

rust-git creates repositories with the standard Git structure:
```
.git/
├── HEAD              # Points to current branch
├── config            # Repository configuration
├── description       # Repository description
├── index             # Staging area
├── objects/          # Object database
│   ├── <xx>/         # First 2 chars of SHA-1
│   │   └── <xxxxx>   # Remaining 38 chars
├── refs/             # References
│   ├── heads/        # Branch references
│   └── tags/         # Tag references
```

## Implementation Notes

- Supports Git's object model (blobs, trees, commits, tags)
- Uses Git's index format for staging area
- Compatible with Git's object storage format
- Implements KVLM (Key-Value List with Message) parsing for commits/tags
- Supports basic .gitignore functionality

## Limitations

- Simplified timezone handling (UTC only for commits)
- Limited merge functionality
- No remote repository support
- No interactive rebase or advanced Git features
- Simplified file mode handling

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests if applicable
5. Submit a pull request

## License

This project is open source. See the repository for license details.