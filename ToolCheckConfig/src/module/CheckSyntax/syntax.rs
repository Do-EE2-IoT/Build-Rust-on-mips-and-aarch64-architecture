use serde::{Deserialize, Serialize};
use std::fs::{self, File};

use fern;
use log::{error, info};

pub type CheckResult = Result<(), String>;

#[derive(Clone)]
pub struct CheckSyntaxConfig {
    pub path: String,
}

impl CheckSyntaxConfig {
    pub fn check_json_file(&self) -> CheckResult {
        let path = self.path.clone();
        let content = fs::read_to_string(&path)
            .map_err(|e| format!("Không thể đọc file {}: {}", self.path, e))?;

        match serde_json::from_str::<serde_json::Value>(&content) {
            Ok(_) => {
                info!("File {} checked successfully", self.path);
                Ok(())
            }
            Err(e) => {
                let error_msg = e.to_string();
                let line_number = e.line();
                let column = e.column();
                let lines: Vec<&str> = content.lines().collect();
                let error_line = lines.get(line_number - 1).unwrap_or(&"");

                let detailed_error = format!(
                    "Lỗi cú pháp trong {} tại dòng {}, cột {}: {}\nDòng lỗi: '{}'",
                    self.path, line_number, column, error_msg, error_line
                );
                error!("{}", detailed_error);
                Err(detailed_error)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_json() {
        let temp_file = "test_valid.json";
        std::fs::write(
            temp_file,
            r#"{
                "name": "test",
                "value": 42,
                "enabled": true
            }"#,
        )
        .unwrap();
        let config = CheckSyntaxConfig {
            path: temp_file.to_string(),
        };
        let result = config.check_json_file();
        assert!(result.is_ok());
        std::fs::remove_file(temp_file).unwrap();
    }

    #[test]
    fn test_invalid_json_missing_comma() {
        let temp_file = "test_invalid.json";
        std::fs::write(
            temp_file,
            r#"{
                "name": "test"
                "value": 42
            }"#,
        )
        .unwrap();
        let config = CheckSyntaxConfig {
            path: temp_file.to_string(),
        };
        let result = config.check_json_file();
        assert!(result.is_err());
        let error_msg = result.unwrap_err();
        println!("Lỗi: {}", error_msg);
        assert!(error_msg.contains("Lỗi cú pháp"));
        assert!(error_msg.contains("dòng"));
        assert!(error_msg.contains("cột"));
        std::fs::remove_file(temp_file).unwrap();
    }

    #[test]
    fn test_invalid_json_multiline() {
        let temp_file = "test_invalid_multiline.json";
        std::fs::write(
            temp_file,
            r#"{
                "name": "test",
                "settings": {
                    "timeout": 30
                    "retries": 3
                }
            }"#,
        )
        .unwrap();
        let config = CheckSyntaxConfig {
            path: temp_file.to_string(),
        };
        let result = config.check_json_file();
        assert!(result.is_err());
        let error_msg = result.unwrap_err();
        println!("Lỗi: {}", error_msg);
        assert!(error_msg.contains("Lỗi cú pháp"));
        assert!(error_msg.contains("dòng"));
        assert!(error_msg.contains("cột"));
        std::fs::remove_file(temp_file).unwrap();
    }

    #[test]
    fn test_file_not_found() {
        let temp_file = "non_existent.json";
        let config = CheckSyntaxConfig {
            path: temp_file.to_string(),
        };
        let result = config.check_json_file();
        assert!(result.is_err());
        let error_msg = result.unwrap_err();
        println!("Lỗi: {}", error_msg);
        assert!(error_msg.contains("Không thể đọc file"));
    }
}
