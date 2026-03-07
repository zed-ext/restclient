use crate::response::ExecutionResult;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

const DEFAULT_HISTORY_SIZE: usize = 100;
const HISTORY_DIR: &str = ".http-client/history";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub id: String,
    pub result: ExecutionResult,
}

pub struct ResponseHistory {
    history_dir: PathBuf,
    max_entries: usize,
}

impl ResponseHistory {
    /// Create a new ResponseHistory with default settings
    pub fn new() -> Self {
        Self {
            history_dir: PathBuf::from(HISTORY_DIR),
            max_entries: DEFAULT_HISTORY_SIZE,
        }
    }

    /// Create a ResponseHistory with custom directory
    pub fn with_directory<P: AsRef<Path>>(dir: P) -> Self {
        Self {
            history_dir: dir.as_ref().to_path_buf(),
            max_entries: DEFAULT_HISTORY_SIZE,
        }
    }

    /// Set maximum number of history entries to keep
    pub fn set_max_entries(&mut self, max: usize) {
        self.max_entries = max;
    }

    /// Initialize history directory
    fn ensure_history_dir(&self) -> Result<(), String> {
        fs::create_dir_all(&self.history_dir)
            .map_err(|e| format!("Failed to create history directory: {}", e))
    }

    /// Generate a unique ID for a history entry
    fn generate_id() -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        format!("{}", timestamp)
    }

    /// Save an execution result to history
    pub fn save(&self, result: ExecutionResult) -> Result<String, String> {
        self.ensure_history_dir()?;

        let id = Self::generate_id();
        let entry = HistoryEntry {
            id: id.clone(),
            result,
        };

        let filename = format!("{}.json", id);
        let filepath = self.history_dir.join(&filename);

        let json = serde_json::to_string_pretty(&entry)
            .map_err(|e| format!("Failed to serialize history entry: {}", e))?;

        fs::write(&filepath, json)
            .map_err(|e| format!("Failed to write history file: {}", e))?;

        // Clean up old entries if we exceed max_entries
        self.cleanup_old_entries()?;

        Ok(id)
    }

    /// Get a specific history entry by ID
    pub fn get(&self, id: &str) -> Result<HistoryEntry, String> {
        let filename = format!("{}.json", id);
        let filepath = self.history_dir.join(&filename);

        if !filepath.exists() {
            return Err(format!("History entry {} not found", id));
        }

        let content = fs::read_to_string(&filepath)
            .map_err(|e| format!("Failed to read history file: {}", e))?;

        let entry: HistoryEntry = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse history entry: {}", e))?;

        Ok(entry)
    }

    /// Get all history entries, sorted by timestamp (newest first)
    pub fn get_all(&self) -> Result<Vec<HistoryEntry>, String> {
        if !self.history_dir.exists() {
            return Ok(Vec::new());
        }

        let mut entries = Vec::new();

        let dir_entries = fs::read_dir(&self.history_dir)
            .map_err(|e| format!("Failed to read history directory: {}", e))?;

        for entry in dir_entries {
            let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                match self.load_entry(&path) {
                    Ok(history_entry) => entries.push(history_entry),
                    Err(e) => eprintln!("Warning: Failed to load history entry {:?}: {}", path, e),
                }
            }
        }

        // Sort by ID (which is timestamp-based) in descending order
        entries.sort_by(|a, b| b.id.cmp(&a.id));

        Ok(entries)
    }

    /// Load a history entry from a file
    fn load_entry(&self, path: &Path) -> Result<HistoryEntry, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read file: {}", e))?;

        serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse JSON: {}", e))
    }

    /// Get the N most recent history entries
    pub fn get_recent(&self, count: usize) -> Result<Vec<HistoryEntry>, String> {
        let mut all_entries = self.get_all()?;
        all_entries.truncate(count);
        Ok(all_entries)
    }

    /// Clear all history
    pub fn clear(&self) -> Result<(), String> {
        if !self.history_dir.exists() {
            return Ok(());
        }

        let entries = fs::read_dir(&self.history_dir)
            .map_err(|e| format!("Failed to read history directory: {}", e))?;

        for entry in entries {
            let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
            let path = entry.path();

            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("json") {
                fs::remove_file(&path)
                    .map_err(|e| format!("Failed to remove history file: {}", e))?;
            }
        }

        Ok(())
    }

    /// Remove a specific history entry
    pub fn remove(&self, id: &str) -> Result<(), String> {
        let filename = format!("{}.json", id);
        let filepath = self.history_dir.join(&filename);

        if !filepath.exists() {
            return Err(format!("History entry {} not found", id));
        }

        fs::remove_file(&filepath)
            .map_err(|e| format!("Failed to remove history file: {}", e))
    }

    /// Clean up old entries if we exceed max_entries
    fn cleanup_old_entries(&self) -> Result<(), String> {
        let mut all_entries = self.get_all()?;

        if all_entries.len() <= self.max_entries {
            return Ok(());
        }

        // Sort by ID (timestamp) descending, keep only max_entries
        all_entries.sort_by(|a, b| b.id.cmp(&a.id));

        // Remove entries beyond max_entries
        for entry in all_entries.iter().skip(self.max_entries) {
            let _ = self.remove(&entry.id); // Ignore errors during cleanup
        }

        Ok(())
    }

    /// Get count of history entries
    pub fn count(&self) -> usize {
        self.get_all().map(|e| e.len()).unwrap_or(0)
    }

    /// Search history by URL pattern
    pub fn search_by_url(&self, pattern: &str) -> Result<Vec<HistoryEntry>, String> {
        let all_entries = self.get_all()?;
        Ok(all_entries
            .into_iter()
            .filter(|entry| entry.result.request.url.contains(pattern))
            .collect())
    }

    /// Search history by method
    pub fn search_by_method(&self, method: &str) -> Result<Vec<HistoryEntry>, String> {
        let all_entries = self.get_all()?;
        let method_upper = method.to_uppercase();
        Ok(all_entries
            .into_iter()
            .filter(|entry| entry.result.request.method.as_str() == method_upper)
            .collect())
    }

    /// Get history statistics
    pub fn get_stats(&self) -> Result<HistoryStats, String> {
        let entries = self.get_all()?;

        let total_requests = entries.len();
        let mut success_count = 0;
        let mut error_count = 0;
        let mut total_duration_ms = 0u128;

        for entry in &entries {
            if entry.result.response.is_success() {
                success_count += 1;
            } else {
                error_count += 1;
            }
            total_duration_ms += entry.result.duration.as_millis();
        }

        let avg_duration_ms = if total_requests > 0 {
            (total_duration_ms / total_requests as u128) as u64
        } else {
            0
        };

        Ok(HistoryStats {
            total_requests,
            success_count,
            error_count,
            avg_duration_ms,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryStats {
    pub total_requests: usize,
    pub success_count: usize,
    pub error_count: usize,
    pub avg_duration_ms: u64,
}

impl Default for ResponseHistory {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{HttpMethod, HttpRequest, RequestMetadata};
    use crate::response::HttpResponse;
    use std::collections::HashMap;
    use std::time::Duration;

    fn create_test_result(method: HttpMethod, url: &str, status: u16) -> ExecutionResult {
        let request = HttpRequest {
            method,
            url: url.to_string(),
            http_version: None,
            headers: vec![],
            body: None,
            response_handler: None,
            metadata: RequestMetadata {
                start_line: 0,
                end_line: 1,
                name: None,
            },
        };

        let response = HttpResponse::new(
            status,
            "OK".to_string(),
            HashMap::new(),
            b"test body".to_vec(),
        );

        ExecutionResult::new(request, response, Duration::from_millis(100))
    }

    #[test]
    fn test_save_and_get_history() {
        let temp_dir = "/tmp/test_history_save_get";
        let _ = fs::remove_dir_all(temp_dir);

        let history = ResponseHistory::with_directory(temp_dir);
        let result = create_test_result(HttpMethod::GET, "https://api.example.com/test", 200);

        let id = history.save(result.clone()).unwrap();
        let retrieved = history.get(&id).unwrap();

        assert_eq!(retrieved.id, id);
        assert_eq!(retrieved.result.request.url, result.request.url);

        fs::remove_dir_all(temp_dir).ok();
    }

    #[test]
    fn test_get_all_history() {
        let temp_dir = "/tmp/test_history_get_all";
        let _ = fs::remove_dir_all(temp_dir);

        let history = ResponseHistory::with_directory(temp_dir);

        // Save multiple entries
        for i in 1..=5 {
            let result = create_test_result(
                HttpMethod::GET,
                &format!("https://api.example.com/test{}", i),
                200,
            );
            history.save(result).unwrap();
        }

        let all_entries = history.get_all().unwrap();
        assert_eq!(all_entries.len(), 5);

        fs::remove_dir_all(temp_dir).ok();
    }

    #[test]
    fn test_clear_history() {
        let temp_dir = "/tmp/test_history_clear";
        let _ = fs::remove_dir_all(temp_dir);

        let history = ResponseHistory::with_directory(temp_dir);

        // Save entries
        for _ in 0..3 {
            let result = create_test_result(HttpMethod::GET, "https://api.example.com/test", 200);
            history.save(result).unwrap();
        }

        assert_eq!(history.count(), 3);

        history.clear().unwrap();
        assert_eq!(history.count(), 0);

        fs::remove_dir_all(temp_dir).ok();
    }

    #[test]
    fn test_max_entries_cleanup() {
        let temp_dir = "/tmp/test_history_max_entries";
        let _ = fs::remove_dir_all(temp_dir);

        let mut history = ResponseHistory::with_directory(temp_dir);
        history.set_max_entries(3);

        // Save 5 entries
        for i in 1..=5 {
            let result = create_test_result(
                HttpMethod::GET,
                &format!("https://api.example.com/test{}", i),
                200,
            );
            history.save(result).unwrap();
            // Small delay to ensure different timestamps
            std::thread::sleep(std::time::Duration::from_millis(10));
        }

        // Should only keep the 3 most recent
        let count = history.count();
        assert!(count <= 3, "Expected <= 3 entries, got {}", count);

        fs::remove_dir_all(temp_dir).ok();
    }

    #[test]
    fn test_search_by_url() {
        let temp_dir = "/tmp/test_history_search_url";
        let _ = fs::remove_dir_all(temp_dir);

        let history = ResponseHistory::with_directory(temp_dir);

        history.save(create_test_result(HttpMethod::GET, "https://api.example.com/users", 200)).unwrap();
        history.save(create_test_result(HttpMethod::GET, "https://api.example.com/posts", 200)).unwrap();
        history.save(create_test_result(HttpMethod::GET, "https://other.com/data", 200)).unwrap();

        let results = history.search_by_url("example.com").unwrap();
        assert_eq!(results.len(), 2);

        fs::remove_dir_all(temp_dir).ok();
    }

    #[test]
    fn test_search_by_method() {
        let temp_dir = "/tmp/test_history_search_method";
        let _ = fs::remove_dir_all(temp_dir);

        let history = ResponseHistory::with_directory(temp_dir);

        history.save(create_test_result(HttpMethod::GET, "https://api.example.com/users", 200)).unwrap();
        history.save(create_test_result(HttpMethod::POST, "https://api.example.com/users", 201)).unwrap();
        history.save(create_test_result(HttpMethod::GET, "https://api.example.com/posts", 200)).unwrap();

        let results = history.search_by_method("GET").unwrap();
        assert_eq!(results.len(), 2);

        let results = history.search_by_method("POST").unwrap();
        assert_eq!(results.len(), 1);

        fs::remove_dir_all(temp_dir).ok();
    }

    #[test]
    fn test_history_stats() {
        let temp_dir = "/tmp/test_history_stats";
        let _ = fs::remove_dir_all(temp_dir);

        let history = ResponseHistory::with_directory(temp_dir);

        history.save(create_test_result(HttpMethod::GET, "https://api.example.com/1", 200)).unwrap();
        history.save(create_test_result(HttpMethod::GET, "https://api.example.com/2", 200)).unwrap();
        history.save(create_test_result(HttpMethod::GET, "https://api.example.com/3", 404)).unwrap();

        let stats = history.get_stats().unwrap();

        assert_eq!(stats.total_requests, 3);
        assert_eq!(stats.success_count, 2);
        assert_eq!(stats.error_count, 1);
        assert_eq!(stats.avg_duration_ms, 100);

        fs::remove_dir_all(temp_dir).ok();
    }
}
