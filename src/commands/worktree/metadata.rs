use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::Path;

pub const METADATA_FILENAME: &str = ".git-polyp-worktree.json";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WorktreeEntry {
    pub branch: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WorktreeMetadata {
    pub version: u32,
    pub main_branch: String,
    pub worktrees: HashMap<String, WorktreeEntry>,
}

#[derive(Debug)]
pub enum MetadataError {
    IoError(io::Error),
    ParseError(serde_json::Error),
    NotFound,
}

impl std::fmt::Display for MetadataError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MetadataError::IoError(e) => write!(f, "IO error: {}", e),
            MetadataError::ParseError(e) => write!(f, "parse error: {}", e),
            MetadataError::NotFound => write!(f, "metadata file not found"),
        }
    }
}

impl From<io::Error> for MetadataError {
    fn from(err: io::Error) -> Self {
        MetadataError::IoError(err)
    }
}

impl From<serde_json::Error> for MetadataError {
    fn from(err: serde_json::Error) -> Self {
        MetadataError::ParseError(err)
    }
}

impl WorktreeMetadata {
    pub fn new(main_branch: String) -> Self {
        Self {
            version: 1,
            main_branch,
            worktrees: HashMap::new(),
        }
    }

    pub fn load(path: &Path) -> Result<Self, MetadataError> {
        let metadata_path = path.join(METADATA_FILENAME);
        if !metadata_path.exists() {
            return Err(MetadataError::NotFound);
        }
        let content = fs::read_to_string(&metadata_path)?;
        let metadata: WorktreeMetadata = serde_json::from_str(&content)?;
        Ok(metadata)
    }

    pub fn save(&self, path: &Path) -> Result<(), MetadataError> {
        let metadata_path = path.join(METADATA_FILENAME);
        let content = serde_json::to_string_pretty(self)?;
        fs::write(&metadata_path, content)?;
        Ok(())
    }

    pub fn add_worktree(&mut self, name: String, branch: String) {
        let entry = WorktreeEntry {
            branch,
            created_at: current_timestamp(),
        };
        self.worktrees.insert(name, entry);
    }

    pub fn remove_worktree(&mut self, name: &str) -> bool {
        self.worktrees.remove(name).is_some()
    }

    pub fn has_worktree(&self, name: &str) -> bool {
        self.worktrees.contains_key(name)
    }

    pub fn get_worktree(&self, name: &str) -> Option<&WorktreeEntry> {
        self.worktrees.get(name)
    }
}

fn current_timestamp() -> String {
    chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn create_temp_dir(name: &str) -> std::path::PathBuf {
        let temp_dir = std::env::temp_dir().join(format!("git_polyp_test_{}", name));
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();
        temp_dir
    }

    fn cleanup_temp_dir(path: &std::path::Path) {
        let _ = fs::remove_dir_all(path);
    }

    #[test]
    fn test_new_metadata() {
        let metadata = WorktreeMetadata::new("main".to_string());
        assert_eq!(metadata.version, 1);
        assert_eq!(metadata.main_branch, "main");
        assert!(metadata.worktrees.is_empty());
    }

    #[test]
    fn test_new_metadata_with_master() {
        let metadata = WorktreeMetadata::new("master".to_string());
        assert_eq!(metadata.main_branch, "master");
    }

    #[test]
    fn test_add_worktree() {
        let mut metadata = WorktreeMetadata::new("main".to_string());
        metadata.add_worktree("feature-a".to_string(), "feature-a".to_string());
        assert!(metadata.has_worktree("feature-a"));
        assert!(!metadata.has_worktree("feature-b"));
    }

    #[test]
    fn test_add_multiple_worktrees() {
        let mut metadata = WorktreeMetadata::new("main".to_string());
        metadata.add_worktree("main".to_string(), "main".to_string());
        metadata.add_worktree("feature-a".to_string(), "feature-a".to_string());
        metadata.add_worktree("feature-b".to_string(), "feature-b".to_string());

        assert!(metadata.has_worktree("main"));
        assert!(metadata.has_worktree("feature-a"));
        assert!(metadata.has_worktree("feature-b"));
        assert_eq!(metadata.worktrees.len(), 3);
    }

    #[test]
    fn test_add_worktree_with_different_name_and_branch() {
        let mut metadata = WorktreeMetadata::new("main".to_string());
        metadata.add_worktree(
            "my-feature".to_string(),
            "origin/feature-branch".to_string(),
        );

        let entry = metadata.get_worktree("my-feature").unwrap();
        assert_eq!(entry.branch, "origin/feature-branch");
    }

    #[test]
    fn test_remove_worktree() {
        let mut metadata = WorktreeMetadata::new("main".to_string());
        metadata.add_worktree("feature-a".to_string(), "feature-a".to_string());
        assert!(metadata.remove_worktree("feature-a"));
        assert!(!metadata.has_worktree("feature-a"));
        assert!(!metadata.remove_worktree("feature-a"));
    }

    #[test]
    fn test_remove_nonexistent_worktree() {
        let mut metadata = WorktreeMetadata::new("main".to_string());
        assert!(!metadata.remove_worktree("nonexistent"));
    }

    #[test]
    fn test_get_worktree() {
        let mut metadata = WorktreeMetadata::new("main".to_string());
        metadata.add_worktree("feature-a".to_string(), "feature-a".to_string());

        let entry = metadata.get_worktree("feature-a");
        assert!(entry.is_some());
        assert_eq!(entry.unwrap().branch, "feature-a");

        assert!(metadata.get_worktree("nonexistent").is_none());
    }

    #[test]
    fn test_get_worktree_created_at_format() {
        let mut metadata = WorktreeMetadata::new("main".to_string());
        metadata.add_worktree("feature-a".to_string(), "feature-a".to_string());

        let entry = metadata.get_worktree("feature-a").unwrap();
        // Check ISO 8601 format: YYYY-MM-DDTHH:MM:SSZ
        assert!(entry.created_at.contains("T"));
        assert!(entry.created_at.ends_with("Z"));
        assert_eq!(entry.created_at.len(), 20);
    }

    #[test]
    fn test_serialization() {
        let mut metadata = WorktreeMetadata::new("main".to_string());
        metadata.add_worktree("main".to_string(), "main".to_string());

        let json = serde_json::to_string(&metadata).unwrap();
        let parsed: WorktreeMetadata = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.version, metadata.version);
        assert_eq!(parsed.main_branch, metadata.main_branch);
        assert!(parsed.has_worktree("main"));
    }

    #[test]
    fn test_serialization_matches_spec_format() {
        let mut metadata = WorktreeMetadata::new("main".to_string());
        metadata.worktrees.insert(
            "main".to_string(),
            WorktreeEntry {
                branch: "main".to_string(),
                created_at: "2024-01-15T10:30:00Z".to_string(),
            },
        );
        metadata.worktrees.insert(
            "feature-a".to_string(),
            WorktreeEntry {
                branch: "feature-a".to_string(),
                created_at: "2024-01-15T11:00:00Z".to_string(),
            },
        );

        let json = serde_json::to_string_pretty(&metadata).unwrap();

        // Verify JSON structure contains expected fields
        assert!(json.contains("\"version\": 1"));
        assert!(json.contains("\"main_branch\": \"main\""));
        assert!(json.contains("\"worktrees\""));
        assert!(json.contains("\"branch\""));
        assert!(json.contains("\"created_at\""));
    }

    #[test]
    fn test_save_and_load_roundtrip() {
        let temp_dir = create_temp_dir("save_load");

        let mut metadata = WorktreeMetadata::new("main".to_string());
        metadata.add_worktree("main".to_string(), "main".to_string());
        metadata.add_worktree("feature-x".to_string(), "feature-x".to_string());

        // Save
        metadata.save(&temp_dir).unwrap();

        // Verify file exists
        assert!(temp_dir.join(METADATA_FILENAME).exists());

        // Load
        let loaded = WorktreeMetadata::load(&temp_dir).unwrap();

        assert_eq!(loaded.version, metadata.version);
        assert_eq!(loaded.main_branch, metadata.main_branch);
        assert!(loaded.has_worktree("main"));
        assert!(loaded.has_worktree("feature-x"));
        assert_eq!(loaded.worktrees.len(), 2);

        cleanup_temp_dir(&temp_dir);
    }

    #[test]
    fn test_load_not_found() {
        let temp_dir = create_temp_dir("load_not_found");

        let result = WorktreeMetadata::load(&temp_dir);
        assert!(matches!(result, Err(MetadataError::NotFound)));

        cleanup_temp_dir(&temp_dir);
    }

    #[test]
    fn test_load_invalid_json() {
        let temp_dir = create_temp_dir("load_invalid");
        let metadata_path = temp_dir.join(METADATA_FILENAME);

        fs::write(&metadata_path, "not valid json").unwrap();

        let result = WorktreeMetadata::load(&temp_dir);
        assert!(matches!(result, Err(MetadataError::ParseError(_))));

        cleanup_temp_dir(&temp_dir);
    }

    #[test]
    fn test_current_timestamp_format() {
        let timestamp = current_timestamp();

        // Format: YYYY-MM-DDTHH:MM:SSZ
        assert_eq!(timestamp.len(), 20);
        assert!(timestamp.contains("T"));
        assert!(timestamp.ends_with("Z"));

        // Verify parts are numeric
        let parts: Vec<&str> = timestamp
            .split(|c| c == '-' || c == 'T' || c == ':' || c == 'Z')
            .collect();
        assert!(parts[0].parse::<u32>().is_ok()); // year
        assert!(parts[1].parse::<u32>().is_ok()); // month
        assert!(parts[2].parse::<u32>().is_ok()); // day
        assert!(parts[3].parse::<u32>().is_ok()); // hour
        assert!(parts[4].parse::<u32>().is_ok()); // minute
        assert!(parts[5].parse::<u32>().is_ok()); // second
    }

    #[test]
    fn test_metadata_filename_constant() {
        assert_eq!(METADATA_FILENAME, ".git-polyp-worktree.json");
    }
}
