use log::{error, info};
use serde_json::Value;
use std::fs::File;
use std::io::{BufReader, Read};
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::Command;

pub type CheckResult = Result<(), String>;

#[derive(Debug)]
pub struct PathChecker {
    path: String,
}

impl PathChecker {
    pub fn new(path: String) -> Self {
        Self { path }
    }

    pub fn check_content(&self) -> CheckResult {
        let extension = Path::new(&self.path)
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.to_lowercase());

        match extension {
            Some(ext) => match ext.as_str() {
                "json" => self.check_json_content(),
                "service" => self.check_service_content(),
                _ => {
                    let error_msg = format!("Định dạng file không được hỗ trợ: {}", self.path);
                    error!("{}", error_msg);
                    Err(error_msg)
                }
            },
            None => {
                let error_msg = format!("File không có đuôi mở rộng: {}", self.path);
                error!("{}", error_msg);
                Err(error_msg)
            }
        }
    }

    fn check_json_content(&self) -> CheckResult {
        let file = File::open(&self.path)
            .map_err(|e| format!("Không thể mở file {}: {}", self.path, e))?;
        let reader = BufReader::new(file);
        let json_value: Value = serde_json::from_reader(reader)
            .map_err(|e| format!("Lỗi parse JSON {}: {}", self.path, e))?;

        let mut errors = Vec::new();
        self.traverse_json(&json_value, &mut errors);

        if errors.is_empty() {
            info!("Content check passed for JSON file: {}", self.path);
            Ok(())
        } else {
            let error_msg = errors.join("\n");
            error!(
                "Content check failed for JSON file {}:\n{}",
                self.path, error_msg
            );
            Err(error_msg)
        }
    }

    fn check_service_content(&self) -> CheckResult {
        let file = File::open(&self.path)
            .map_err(|e| format!("Không thể mở file {}: {}", self.path, e))?;
        let mut reader = BufReader::new(file);
        let mut content = String::new();
        reader
            .read_to_string(&mut content)
            .map_err(|e| format!("Không thể đọc nội dung file {}: {}", self.path, e))?;

        let lines: Vec<&str> = content.lines().collect();
        let mut errors = Vec::new();
        let mut in_service_section = false;

        for (line_num, line) in lines.iter().enumerate() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if line == "[Service]" {
                in_service_section = true;
            } else if line.starts_with('[') && in_service_section {
                in_service_section = false;
            } else if in_service_section {
                if line.starts_with("ExecStart=") || line.starts_with("ExecStartPre=") {
                    let command = line.splitn(2, '=').nth(1).unwrap_or("").trim();
                    if !command.is_empty() {
                        self.check_command(command, line_num + 1, &mut errors);
                    }
                } else if line.starts_with("EnvironmentFile=") {
                    let env_file = line.splitn(2, '=').nth(1).unwrap_or("").trim();
                    if !env_file.is_empty() {
                        self.check_path(env_file, &mut errors);
                    }
                }
            }
        }

        if errors.is_empty() {
            info!(
                "Content check passed for systemd service file: {}",
                self.path
            );
            Ok(())
        } else {
            let error_msg = errors.join("\n");
            error!(
                "Content check failed for systemd service file {}:\n{}",
                self.path, error_msg
            );
            Err(error_msg)
        }
    }

    fn traverse_json(&self, value: &Value, errors: &mut Vec<String>) {
        match value {
            Value::Object(map) => {
                for (key, val) in map {
                    self.check_value(key, val, errors);
                    self.traverse_json(val, errors);
                }
            }
            Value::Array(arr) => {
                for item in arr {
                    self.traverse_json(item, errors);
                }
            }
            _ => {}
        }
    }

    fn check_value(&self, key: &str, value: &Value, errors: &mut Vec<String>) {
        if let Value::String(s) = value {
            let key_lower = key.to_lowercase();
            if self.is_path(&key_lower, s) {
                self.check_path(s, errors);
            }
        }
    }

    fn check_command(&self, command: &str, line_num: usize, errors: &mut Vec<String>) {
        // Tách lệnh thành các phần (command và arguments)
        let mut parts = command.split_whitespace();
        let cmd_path = parts.next().unwrap_or("");

        if cmd_path.is_empty() {
            errors.push(format!(
                "Lệnh trống tại dòng {} trong file {}",
                line_num, self.path
            ));
            return;
        }

        if !Path::new(cmd_path).exists() {
            errors.push(format!(
                "Đường dẫn lệnh không tồn tại tại dòng {}: '{}'",
                line_num, cmd_path
            ));
            return;
        }

        if let Ok(metadata) = std::fs::metadata(cmd_path) {
            if metadata.permissions().mode() & 0o111 == 0 {
                errors.push(format!(
                    "Lệnh không có quyền thực thi tại dòng {}: '{}'",
                    line_num, cmd_path
                ));
            }
        }

        // Kiểm tra lệnh có sẵn trong PATH (nếu không phải đường dẫn tuyệt đối)
        if !cmd_path.starts_with('/') {
            let output = Command::new("which").arg(cmd_path).output().map_err(|e| {
                format!(
                    "Không thể chạy lệnh 'which' để kiểm tra {}: {}",
                    cmd_path, e
                )
            });

            match output {
                Ok(output) => {
                    if !output.status.success() {
                        errors.push(format!(
                            "Lệnh không được tìm thấy trong PATH tại dòng {}: '{}'",
                            line_num, cmd_path
                        ));
                    }
                }
                Err(e) => {
                    errors.push(format!(
                        "Lỗi khi kiểm tra lệnh tại dòng {}: '{}'\n{}",
                        line_num, cmd_path, e
                    ));
                }
            }
        }
    }

    fn is_path(&self, key: &str, value: &str) -> bool {
        value.starts_with('/')
            || key.contains("path")
            || key.contains("file")
            || value.ends_with(".log")
            || value.ends_with(".pem")
            || value.ends_with(".db")
            || value.ends_with(".txt")
    }

    fn check_path(&self, path: &str, errors: &mut Vec<String>) {
        if !Path::new(path).exists() {
            errors.push(format!("Đường dẫn không tồn tại: {}", path));
        }
    }
}
