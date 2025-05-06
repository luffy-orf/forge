use derive_setters::Setters;
use serde::{Deserialize, Serialize};

use crate::{ToolCallFull, ToolCallId, ToolName, tool_response_data::ToolResponseData};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, Setters)]
#[setters(strip_option, into)]
pub struct ToolResult {
    pub name: ToolName,
    pub call_id: Option<ToolCallId>,
    #[setters(skip)]
    pub content: String,
    #[setters(skip)]
    pub is_error: bool,
    #[setters(skip)]
    #[serde(skip)]
    pub data: Option<ToolResponseData>,
}

impl ToolResult {
    pub fn new(name: ToolName) -> ToolResult {
        Self {
            name,
            call_id: None,
            content: String::default(),
            is_error: false,
            data: None,
        }
    }

    pub fn success(mut self, content: impl Into<String>) -> Self {
        self.content = content.into();
        self.is_error = false;
        self
    }

    pub fn failure(mut self, err: anyhow::Error) -> Self {
        let mut output = String::new();
        output.push_str("\nERROR:\n");

        for cause in err.chain() {
            output.push_str(&format!("Caused by: {cause}\n"));
        }

        self.content = output;
        self.is_error = true;
        self
    }
    
    pub fn with_data(mut self, data: ToolResponseData) -> Self {
        self.data = Some(data);
        self
    }
    
    /// Helper method to set both the content and structured data in one call
    /// This also returns the content directly, making it convenient for tools to generate their response
    pub fn with_frontmatter_response(mut self, data: ToolResponseData, content: impl Into<String>) -> Self {
        let content_str = content.into();
        self.content = content_str;
        self.data = Some(data);
        self
    }
}

impl From<ToolCallFull> for ToolResult {
    fn from(value: ToolCallFull) -> Self {
        Self {
            name: value.name,
            call_id: value.call_id,
            content: String::default(),
            is_error: false,
            data: None,
        }
    }
}

impl std::fmt::Display for ToolResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // If we have ToolResponseData, use front matter format
        if let Some(data) = &self.data {
            write!(f, "{}", data.to_front_matter(&self.content))
        } else {
            // Legacy XML format for backward compatibility
            write!(f, "<forge_tool_result>")?;
            write!(
                f,
                "<forge_tool_name>{}</forge_tool_name>",
                self.name.as_str()
            )?;
            let content = format!("<![CDATA[{}]]>", self.content);
            if self.is_error {
                write!(f, "<e>{content}</e>")?;
            } else {
                write!(f, "<success>{content}</success>")?;
            }

            write!(f, "</forge_tool_result>")
        }
    }
}

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;
    use serde_json::json;

    use super::*;
    use crate::tool_response_data::ToolResponseData;

    #[test]
    fn test_snapshot_minimal() {
        let result = ToolResult::new(ToolName::new("test_tool"));
        assert_snapshot!(result);
    }

    #[test]
    fn test_snapshot_full() {
        let result = ToolResult::new(ToolName::new("complex_tool"))
            .call_id(ToolCallId::new("123"))
            .failure(anyhow::anyhow!(
                json!({"key": "value", "number": 42}).to_string()
            ));
        assert_snapshot!(result);
    }

    #[test]
    fn test_snapshot_with_special_chars() {
        let result = ToolResult::new(ToolName::new("xml_tool")).success(
            json!({
                "text": "Special chars: < > & ' \"",
                "nested": {
                    "html": "<div>Test</div>"
                }
            })
            .to_string(),
        );
        assert_snapshot!(result);
    }

    #[test]
    fn test_display_minimal() {
        let result = ToolResult::new(ToolName::new("test_tool"));
        assert_snapshot!(result.to_string());
    }

    #[test]
    fn test_display_full() {
        let result = ToolResult::new(ToolName::new("complex_tool"))
            .call_id(ToolCallId::new("123"))
            .success(
                json!({
                    "user": "John Doe",
                    "age": 42,
                    "address": [{"city": "New York"}, {"city": "Los Angeles"}]
                })
                .to_string(),
            );
        assert_snapshot!(result.to_string());
    }

    #[test]
    fn test_display_special_chars() {
        let result = ToolResult::new(ToolName::new("xml_tool")).success(
            json!({
                "text": "Special chars: < > & ' \"",
                "nested": {
                    "html": "<div>Test</div>"
                }
            })
            .to_string(),
        );
        assert_snapshot!(result.to_string());
    }

    #[test]
    fn test_success_and_failure_content() {
        let success = ToolResult::new(ToolName::new("test_tool")).success("success message");
        assert!(!success.is_error);
        assert_eq!(success.content, "success message");

        let failure =
            ToolResult::new(ToolName::new("test_tool")).failure(anyhow::anyhow!("error message"));
        assert!(failure.is_error);
        assert_eq!(failure.content, "\nERROR:\nCaused by: error message\n");
    }
    
    #[test]
    fn test_frontmatter_format() {
        let data = ToolResponseData::file_read("/path/to/file.txt")
            .with_total_lines(100)
            .with_metadata("encoding", "utf-8");
            
        let result = ToolResult::new(ToolName::new("file_read"))
            .success("File content here")
            .with_data(data);
            
        let output = result.to_string();
        assert!(output.starts_with("---"));
        assert!(output.contains("type: file_read"));
        assert!(output.contains("path: /path/to/file.txt"));
        assert!(output.contains("total_lines: 100"));
        assert!(output.contains("encoding: utf-8"));
        assert!(output.contains("File content here"));
    }
    
    #[test]
    fn test_parse_frontmatter() {
        let input = r#"---
type: file_read
path: /path/to/file.txt
total_lines: 100
encoding: utf-8
---
File content here"#;
        
        let (data, content) = ToolResponseData::from_front_matter(input);
        assert!(data.is_some());
        if let Some(ToolResponseData::FileRead { path, total_lines, metadata }) = data {
            assert_eq!(path, "/path/to/file.txt");
            assert_eq!(total_lines, Some(100));
            assert_eq!(metadata.get("encoding").unwrap().as_str().unwrap(), "utf-8");
        } else {
            panic!("Expected FileRead data");
        }
        assert_eq!(content, "File content here");
    }

    #[test]
    fn test_complete_frontmatter_example() {
        // Create a structured data object for a file read operation
        let data = ToolResponseData::file_read("/example/file.txt")
            .with_total_lines(42)
            .with_metadata("encoding", "utf-8")
            .with_metadata("file_size", 1024);
            
        // Create a tool result with the structured data
        let result = ToolResult::new(ToolName::new("forge_tool_fs_read"))
            .call_id(ToolCallId::new("abc123"))
            .with_frontmatter_response(data, "This is the content of the file\nSecond line\nThird line");
            
        // Convert to string to get the front matter format
        let output = result.to_string();
        
        // Verify the output contains expected fields
        assert!(output.contains("---"));
        assert!(output.contains("type: file_read"));
        assert!(output.contains("path: /example/file.txt"));
        assert!(output.contains("total_lines: 42"));
        assert!(output.contains("encoding: utf-8"));
        assert!(output.contains("file_size: 1024"));
        assert!(output.contains("---"));
        assert!(output.contains("This is the content of the file"));
        assert!(output.contains("Second line"));
        assert!(output.contains("Third line"));
        
        // Parse back from string
        let (parsed_data, content) = ToolResponseData::from_front_matter(&output);
        assert!(parsed_data.is_some());
        if let Some(ToolResponseData::FileRead { path, total_lines, metadata }) = parsed_data {
            assert_eq!(path, "/example/file.txt");
            assert_eq!(total_lines, Some(42));
            assert_eq!(metadata.get("encoding").unwrap().as_str().unwrap(), "utf-8");
            assert_eq!(metadata.get("file_size").unwrap().as_u64().unwrap(), 1024);
        } else {
            panic!("Expected FileRead data");
        }
        
        assert_eq!(content, "This is the content of the file\nSecond line\nThird line");
    }
}
