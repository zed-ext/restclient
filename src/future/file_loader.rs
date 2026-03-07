use std::fs;
use std::path::{Path, PathBuf};

pub struct FileLoader {
    /// Base directory for resolving relative paths (usually the .http file's directory)
    base_dir: PathBuf,
}

impl FileLoader {
    /// Create a new FileLoader with a base directory
    pub fn new<P: AsRef<Path>>(base_dir: P) -> Self {
        Self {
            base_dir: base_dir.as_ref().to_path_buf(),
        }
    }

    /// Create a FileLoader from a file path (uses the file's parent directory)
    pub fn from_file<P: AsRef<Path>>(file_path: P) -> Result<Self, String> {
        let path = file_path.as_ref();
        let base_dir = path
            .parent()
            .ok_or_else(|| format!("Cannot determine parent directory of {:?}", path))?
            .to_path_buf();

        Ok(Self { base_dir })
    }

    /// Resolve a relative path against the base directory
    pub fn resolve_path<P: AsRef<Path>>(&self, relative_path: P) -> PathBuf {
        let path = relative_path.as_ref();

        if path.is_absolute() {
            path.to_path_buf()
        } else {
            self.base_dir.join(path)
        }
    }

    /// Load a file as a string
    pub fn load_text<P: AsRef<Path>>(&self, relative_path: P) -> Result<String, String> {
        let full_path = self.resolve_path(relative_path);

        fs::read_to_string(&full_path)
            .map_err(|e| format!("Failed to read file {:?}: {}", full_path, e))
    }

    /// Load a file as bytes
    pub fn load_bytes<P: AsRef<Path>>(&self, relative_path: P) -> Result<Vec<u8>, String> {
        let full_path = self.resolve_path(relative_path);

        fs::read(&full_path)
            .map_err(|e| format!("Failed to read file {:?}: {}", full_path, e))
    }

    /// Load a JSON file and parse it
    pub fn load_json<P: AsRef<Path>, T: serde::de::DeserializeOwned>(
        &self,
        relative_path: P,
    ) -> Result<T, String> {
        let content = self.load_text(relative_path)?;

        serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse JSON: {}", e))
    }

    /// Check if a file exists
    pub fn file_exists<P: AsRef<Path>>(&self, relative_path: P) -> bool {
        let full_path = self.resolve_path(relative_path);
        full_path.exists() && full_path.is_file()
    }

    /// Get the base directory
    pub fn base_dir(&self) -> &Path {
        &self.base_dir
    }

    /// Load request body from a file reference
    /// Detects content type based on file extension
    pub fn load_request_body<P: AsRef<Path>>(
        &self,
        relative_path: P,
    ) -> Result<(Vec<u8>, Option<String>), String> {
        let path = relative_path.as_ref();
        let bytes = self.load_bytes(path)?;

        // Detect content type from extension
        let content_type = Self::detect_content_type(path);

        Ok((bytes, content_type))
    }

    /// Detect MIME content type from file extension
    fn detect_content_type(path: &Path) -> Option<String> {
        path.extension()
            .and_then(|ext| ext.to_str())
            .and_then(|ext| match ext.to_lowercase().as_str() {
                "json" => Some("application/json".to_string()),
                "xml" => Some("application/xml".to_string()),
                "html" | "htm" => Some("text/html".to_string()),
                "txt" => Some("text/plain".to_string()),
                "yaml" | "yml" => Some("application/yaml".to_string()),
                "csv" => Some("text/csv".to_string()),
                "pdf" => Some("application/pdf".to_string()),
                "png" => Some("image/png".to_string()),
                "jpg" | "jpeg" => Some("image/jpeg".to_string()),
                "gif" => Some("image/gif".to_string()),
                "svg" => Some("image/svg+xml".to_string()),
                _ => None,
            })
    }

    /// Load a JavaScript response handler file
    pub fn load_handler<P: AsRef<Path>>(&self, relative_path: P) -> Result<String, String> {
        self.load_text(relative_path)
    }

    /// Load environments.json file
    pub fn load_environments<P: AsRef<Path>>(
        &self,
        relative_path: P,
    ) -> Result<crate::variables::Environments, String> {
        self.load_json(relative_path)
    }
}

impl Default for FileLoader {
    fn default() -> Self {
        Self::new(".")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_resolve_relative_path() {
        let loader = FileLoader::new("/base/dir");

        let resolved = loader.resolve_path("./file.txt");
        assert_eq!(resolved, PathBuf::from("/base/dir/./file.txt"));

        let resolved = loader.resolve_path("../other/file.txt");
        assert_eq!(resolved, PathBuf::from("/base/dir/../other/file.txt"));
    }

    #[test]
    fn test_resolve_absolute_path() {
        let loader = FileLoader::new("/base/dir");

        let resolved = loader.resolve_path("/absolute/path/file.txt");
        assert_eq!(resolved, PathBuf::from("/absolute/path/file.txt"));
    }

    #[test]
    fn test_detect_content_type() {
        assert_eq!(
            FileLoader::detect_content_type(Path::new("file.json")),
            Some("application/json".to_string())
        );

        assert_eq!(
            FileLoader::detect_content_type(Path::new("file.xml")),
            Some("application/xml".to_string())
        );

        assert_eq!(
            FileLoader::detect_content_type(Path::new("file.txt")),
            Some("text/plain".to_string())
        );

        assert_eq!(
            FileLoader::detect_content_type(Path::new("file.unknown")),
            None
        );
    }

    #[test]
    fn test_load_text_file() {
        // Create a temporary test file
        let temp_dir = "/tmp/test_file_loader";
        fs::create_dir_all(temp_dir).unwrap();

        let test_file = format!("{}/test.txt", temp_dir);
        fs::write(&test_file, "Hello, World!").unwrap();

        let loader = FileLoader::new(temp_dir);
        let content = loader.load_text("test.txt").unwrap();

        assert_eq!(content, "Hello, World!");

        fs::remove_dir_all(temp_dir).ok();
    }

    #[test]
    fn test_load_json_file() {
        use serde::{Deserialize, Serialize};

        #[derive(Debug, Serialize, Deserialize, PartialEq)]
        struct TestData {
            name: String,
            value: i32,
        }

        // Create a temporary test file
        let temp_dir = "/tmp/test_file_loader_json";
        fs::create_dir_all(temp_dir).unwrap();

        let test_file = format!("{}/test.json", temp_dir);
        let test_data = TestData {
            name: "test".to_string(),
            value: 42,
        };
        fs::write(&test_file, serde_json::to_string(&test_data).unwrap()).unwrap();

        let loader = FileLoader::new(temp_dir);
        let loaded: TestData = loader.load_json("test.json").unwrap();

        assert_eq!(loaded, test_data);

        fs::remove_dir_all(temp_dir).ok();
    }

    #[test]
    fn test_file_exists() {
        let temp_dir = "/tmp/test_file_exists";
        fs::create_dir_all(temp_dir).unwrap();

        let test_file = format!("{}/exists.txt", temp_dir);
        fs::write(&test_file, "content").unwrap();

        let loader = FileLoader::new(temp_dir);

        assert!(loader.file_exists("exists.txt"));
        assert!(!loader.file_exists("not_exists.txt"));

        fs::remove_dir_all(temp_dir).ok();
    }

    #[test]
    fn test_load_request_body() {
        let temp_dir = "/tmp/test_request_body";
        fs::create_dir_all(temp_dir).unwrap();

        let json_file = format!("{}/data.json", temp_dir);
        fs::write(&json_file, r#"{"key": "value"}"#).unwrap();

        let loader = FileLoader::new(temp_dir);
        let (bytes, content_type) = loader.load_request_body("data.json").unwrap();

        assert_eq!(bytes, br#"{"key": "value"}"#);
        assert_eq!(content_type, Some("application/json".to_string()));

        fs::remove_dir_all(temp_dir).ok();
    }

    #[test]
    fn test_from_file() {
        let temp_dir = "/tmp/test_from_file";
        fs::create_dir_all(temp_dir).unwrap();

        let test_file = format!("{}/request.http", temp_dir);
        fs::write(&test_file, "GET /api").unwrap();

        let loader = FileLoader::from_file(&test_file).unwrap();

        assert_eq!(loader.base_dir(), Path::new(temp_dir));

        fs::remove_dir_all(temp_dir).ok();
    }
}
