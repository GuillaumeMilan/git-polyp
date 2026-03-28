use std::env;
use std::path::{Path, PathBuf};

use super::metadata::METADATA_FILENAME;

#[derive(Debug)]
pub enum DetectError {
    NotFound,
    IoError(std::io::Error),
}

impl From<std::io::Error> for DetectError {
    fn from(err: std::io::Error) -> Self {
        DetectError::IoError(err)
    }
}

/// Find the git-polyp worktree root by searching upward for .git-polyp-worktree.json
pub fn find_worktree_root() -> Result<PathBuf, DetectError> {
    let current_dir = env::current_dir()?;
    find_worktree_root_from(&current_dir)
}

/// Find the git-polyp worktree root starting from a specific path
pub fn find_worktree_root_from(start_path: &Path) -> Result<PathBuf, DetectError> {
    let mut current = start_path.to_path_buf();

    loop {
        let metadata_path = current.join(METADATA_FILENAME);
        if metadata_path.exists() {
            return Ok(current);
        }

        match current.parent() {
            Some(parent) => current = parent.to_path_buf(),
            None => return Err(DetectError::NotFound),
        }
    }
}

/// Check if a path is inside a git-polyp worktree workspace
pub fn is_in_worktree_workspace() -> bool {
    find_worktree_root().is_ok()
}

/// Check if a path contains a git-polyp worktree workspace (has .git-polyp-worktree.json)
pub fn is_worktree_root(path: &Path) -> bool {
    path.join(METADATA_FILENAME).exists()
}

/// Get the path to the bare repository (.bare directory)
pub fn get_bare_repo_path(worktree_root: &Path) -> PathBuf {
    worktree_root.join(".bare")
}

/// Get the path to the template directory
pub fn get_template_path(worktree_root: &Path) -> PathBuf {
    worktree_root.join(".template")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn create_temp_dir(name: &str) -> PathBuf {
        let temp_dir = std::env::temp_dir().join(format!("git_polyp_detect_test_{}", name));
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();
        temp_dir
    }

    fn cleanup_temp_dir(path: &Path) {
        let _ = fs::remove_dir_all(path);
    }

    #[test]
    fn test_is_worktree_root_false() {
        let temp_dir = create_temp_dir("is_root_false");
        assert!(!is_worktree_root(&temp_dir));
        cleanup_temp_dir(&temp_dir);
    }

    #[test]
    fn test_is_worktree_root_true() {
        let temp_dir = create_temp_dir("is_root_true");
        let metadata_path = temp_dir.join(METADATA_FILENAME);
        fs::write(&metadata_path, "{}").unwrap();
        assert!(is_worktree_root(&temp_dir));
        cleanup_temp_dir(&temp_dir);
    }

    #[test]
    fn test_get_bare_repo_path() {
        let root = PathBuf::from("/some/path");
        assert_eq!(get_bare_repo_path(&root), PathBuf::from("/some/path/.bare"));
    }

    #[test]
    fn test_get_bare_repo_path_relative() {
        let root = PathBuf::from("my-repo");
        assert_eq!(get_bare_repo_path(&root), PathBuf::from("my-repo/.bare"));
    }

    #[test]
    fn test_get_template_path() {
        let root = PathBuf::from("/some/path");
        assert_eq!(
            get_template_path(&root),
            PathBuf::from("/some/path/.template")
        );
    }

    #[test]
    fn test_get_template_path_relative() {
        let root = PathBuf::from("my-repo");
        assert_eq!(
            get_template_path(&root),
            PathBuf::from("my-repo/.template")
        );
    }

    #[test]
    fn test_find_worktree_root_from_at_root() {
        let temp_dir = create_temp_dir("find_at_root");
        let metadata_path = temp_dir.join(METADATA_FILENAME);
        fs::write(&metadata_path, "{}").unwrap();

        let result = find_worktree_root_from(&temp_dir);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), temp_dir);

        cleanup_temp_dir(&temp_dir);
    }

    #[test]
    fn test_find_worktree_root_from_nested_directory() {
        let temp_dir = create_temp_dir("find_nested");
        let metadata_path = temp_dir.join(METADATA_FILENAME);
        fs::write(&metadata_path, "{}").unwrap();

        // Create nested directories
        let nested = temp_dir.join("feature-a").join("src").join("deep");
        fs::create_dir_all(&nested).unwrap();

        let result = find_worktree_root_from(&nested);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), temp_dir);

        cleanup_temp_dir(&temp_dir);
    }

    #[test]
    fn test_find_worktree_root_from_worktree_subdirectory() {
        let temp_dir = create_temp_dir("find_worktree_subdir");
        let metadata_path = temp_dir.join(METADATA_FILENAME);
        fs::write(&metadata_path, "{}").unwrap();

        // Create a worktree directory with nested structure
        let worktree_dir = temp_dir.join("feature-branch");
        let nested = worktree_dir.join("src").join("components");
        fs::create_dir_all(&nested).unwrap();

        // Searching from nested should find the root
        let result = find_worktree_root_from(&nested);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), temp_dir);

        cleanup_temp_dir(&temp_dir);
    }

    #[test]
    fn test_find_worktree_root_from_not_found() {
        let temp_dir = create_temp_dir("find_not_found");

        // No metadata file - should fail
        let result = find_worktree_root_from(&temp_dir);
        assert!(matches!(result, Err(DetectError::NotFound)));

        cleanup_temp_dir(&temp_dir);
    }

    #[test]
    fn test_find_worktree_root_from_deeply_nested_not_found() {
        let temp_dir = create_temp_dir("find_deep_not_found");
        let deep_path = temp_dir.join("a").join("b").join("c").join("d");
        fs::create_dir_all(&deep_path).unwrap();

        // No metadata file anywhere - should fail
        let result = find_worktree_root_from(&deep_path);
        assert!(matches!(result, Err(DetectError::NotFound)));

        cleanup_temp_dir(&temp_dir);
    }

    #[test]
    fn test_detect_error_from_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "test error");
        let detect_err: DetectError = io_err.into();
        assert!(matches!(detect_err, DetectError::IoError(_)));
    }

    #[test]
    fn test_directory_structure_paths() {
        // Test that the expected directory structure paths are correct
        let root = PathBuf::from("/workspace/my-project");

        let bare = get_bare_repo_path(&root);
        let template = get_template_path(&root);

        assert_eq!(bare, PathBuf::from("/workspace/my-project/.bare"));
        assert_eq!(template, PathBuf::from("/workspace/my-project/.template"));

        // Verify the metadata filename constant matches the spec
        assert_eq!(METADATA_FILENAME, ".git-polyp-worktree.json");
    }
}
