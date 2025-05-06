use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// Represents the structured data for tool responses.
#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(tag = "type")]
pub enum ToolResponseData {
    /// File read operation response
    #[serde(rename = "file_read")]
    FileRead {
        /// Path of the file that was read
        path: String,
        /// Total number of lines in the file
        total_lines: Option<usize>,
        /// Additional metadata specific to file read
        #[serde(flatten)]
        metadata: HashMap<String, serde_json::Value>,
    },
    
    /// File write operation response
    #[serde(rename = "file_write")]
    FileWrite {
        /// Path of the file that was written
        path: String,
        /// Total bytes written
        bytes_written: Option<usize>,
        /// Whether the operation was an update to an existing file
        was_update: Option<bool>,
        /// Additional metadata specific to file write
        #[serde(flatten)]
        metadata: HashMap<String, serde_json::Value>,
    },
    
    /// Shell command execution response
    #[serde(rename = "shell")]
    Shell {
        /// The command that was executed
        command: String,
        /// Exit code of the command
        exit_code: Option<i32>,
        /// Additional metadata specific to shell commands
        #[serde(flatten)]
        metadata: HashMap<String, serde_json::Value>,
    },
    
    /// Patch operation response (for file modifications)
    #[serde(rename = "patch")]
    Patch {
        /// Path of the file that was patched
        path: String,
        /// Total characters in the patched file
        total_chars: Option<usize>,
        /// Warning message if any syntax issues were detected
        warning: Option<String>,
        /// Additional metadata specific to patch operations
        #[serde(flatten)]
        metadata: HashMap<String, serde_json::Value>,
    },
    
    /// Generic response for tools without specific structured data
    #[serde(rename = "generic")]
    Generic {
        /// Tool-specific metadata
        #[serde(flatten)]
        metadata: HashMap<String, serde_json::Value>,
    },
}

impl ToolResponseData {
    /// Create a FileRead response
    pub fn file_read(path: impl Into<String>) -> Self {
        Self::FileRead {
            path: path.into(),
            total_lines: None,
            metadata: HashMap::new(),
        }
    }
    
    /// Create a FileWrite response
    pub fn file_write(path: impl Into<String>) -> Self {
        Self::FileWrite {
            path: path.into(),
            bytes_written: None,
            was_update: None,
            metadata: HashMap::new(),
        }
    }
    
    /// Create a Shell response
    pub fn shell(command: impl Into<String>) -> Self {
        Self::Shell {
            command: command.into(),
            exit_code: None,
            metadata: HashMap::new(),
        }
    }
    
    /// Create a Patch response
    pub fn patch(path: impl Into<String>) -> Self {
        Self::Patch {
            path: path.into(),
            total_chars: None,
            warning: None,
            metadata: HashMap::new(),
        }
    }
    
    /// Create a Generic response
    pub fn generic() -> Self {
        Self::Generic {
            metadata: HashMap::new(),
        }
    }
    
    /// Add a value to the metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<serde_json::Value>) -> Self {
        match &mut self {
            Self::FileRead { metadata, .. } => {
                metadata.insert(key.into(), value.into());
            }
            Self::FileWrite { metadata, .. } => {
                metadata.insert(key.into(), value.into());
            }
            Self::Shell { metadata, .. } => {
                metadata.insert(key.into(), value.into());
            }
            Self::Patch { metadata, .. } => {
                metadata.insert(key.into(), value.into());
            }
            Self::Generic { metadata } => {
                metadata.insert(key.into(), value.into());
            }
        }
        self
    }
    
    /// Update the total lines for FileRead
    pub fn with_total_lines(mut self, total_lines: usize) -> Self {
        if let Self::FileRead { total_lines: t, .. } = &mut self {
            *t = Some(total_lines);
        }
        self
    }
    
    /// Update the bytes written for FileWrite
    pub fn with_bytes_written(mut self, bytes_written: usize) -> Self {
        if let Self::FileWrite { bytes_written: b, .. } = &mut self {
            *b = Some(bytes_written);
        }
        self
    }
    
    /// Update the was_update flag for FileWrite
    pub fn with_was_update(mut self, was_update: bool) -> Self {
        if let Self::FileWrite { was_update: w, .. } = &mut self {
            *w = Some(was_update);
        }
        self
    }
    
    /// Update the exit code for Shell
    pub fn with_exit_code(mut self, exit_code: i32) -> Self {
        if let Self::Shell { exit_code: e, .. } = &mut self {
            *e = Some(exit_code);
        }
        self
    }
    
    /// Update the total chars for Patch
    pub fn with_total_chars(mut self, total_chars: usize) -> Self {
        if let Self::Patch { total_chars: t, .. } = &mut self {
            *t = Some(total_chars);
        }
        self
    }
    
    /// Update the warning for Patch
    pub fn with_warning(mut self, warning: impl Into<String>) -> Self {
        if let Self::Patch { warning: w, .. } = &mut self {
            *w = Some(warning.into());
        }
        self
    }
    
    /// Convert the ToolResponseData to YAML front matter format
    /// 
    /// The front matter contains all the metadata, while the content is provided separately.
    /// This follows the format:
    /// ```text
    /// ---
    /// key1: value1
    /// key2: value2
    /// ---
    /// Content goes here
    /// ```
    pub fn to_front_matter(&self, content: impl Into<String>) -> String {
        let content_str = content.into();
        
        // Convert the enum to a serializable representation
        let data = serde_json::to_value(self).unwrap_or_default();
        
        // Build the front matter manually
        let mut result = String::from("---\n");
        
        if let serde_json::Value::Object(map) = data {
            // Use serde_yml to convert the JSON object to YAML
            let yaml = serde_yml::to_string(&map).unwrap_or_default();
            result.push_str(&yaml);
        }
        
        result.push_str("---\n");
        result.push_str(&content_str);
        
        result
    }
    
    /// Parse front matter format back into a ToolResponseData and content
    pub fn from_front_matter(text: &str) -> (Option<Self>, String) {
        // Simple parsing using string operations rather than relying on gray_matter
        if !text.starts_with("---\n") {
            return (None, text.to_string());
        }
        
        // Find the end of the front matter
        if let Some(end_index) = text[4..].find("\n---\n") {
            let yaml_end = 4 + end_index;
            let yaml_content = &text[4..yaml_end];
            let content = &text[(yaml_end + 5)..]; // Skip past the ending delimiter
            
            // Parse the YAML
            if let Ok(value) = serde_yml::from_str::<serde_json::Value>(yaml_content) {
                if let Ok(data) = serde_json::from_value::<Self>(value) {
                    return (Some(data), content.to_string());
                }
            }
            
            // Return just the content if we couldn't parse the YAML
            return (None, content.to_string());
        }
        
        // If we couldn't find the ending delimiter, return the original text
        (None, text.to_string())
    }
} 