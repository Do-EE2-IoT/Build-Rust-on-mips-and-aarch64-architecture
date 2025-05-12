use log::{error, info};
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

pub type CheckResult = Result<(), String>;

#[derive(Clone)]
pub struct CheckSyntaxConfig {
    pub path: String,
}

impl CheckSyntaxConfig {
    pub fn check_file(&self) -> CheckResult {
        let extension = Path::new(&self.path)
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.to_lowercase());

        match extension {
            Some(ext) => match ext.as_str() {
                "json" => self.check_json_file(),
                "service" => self.check_service_file(),
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

    fn check_json_file(&self) -> CheckResult {
        let file = File::open(&self.path)
            .map_err(|e| format!("Không thể đọc file {}: {}", self.path, e))?;
        let reader = BufReader::new(file);

        match serde_json::from_reader::<_, serde_json::Value>(reader) {
            Ok(_) => {
                info!("File JSON {} checked successfully", self.path);
                Ok(())
            }
            Err(e) => {
                let error_msg = e.to_string();
                let line_number = e.line();
                let column = e.column();

                let content = std::fs::read_to_string(&self.path)
                    .map_err(|e| format!("Không thể đọc file {}: {}", self.path, e))?;
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

    fn check_service_file(&self) -> CheckResult {
        let file = File::open(&self.path)
            .map_err(|e| format!("Không thể đọc file {}: {}", self.path, e))?;
        let mut reader = BufReader::new(file);
        let mut content = String::new();
        reader
            .read_to_string(&mut content)
            .map_err(|e| format!("Không thể đọc nội dung file {}: {}", self.path, e))?;

        let lines: Vec<&str> = content.lines().collect();
        let mut errors = Vec::new();

        // Kiểm tra các section bắt buộc
        let has_unit = content.contains("[Unit]");
        let has_service = content.contains("[Service]");
        let has_install = content.contains("[Install]");

        if !has_unit {
            errors.push("Thiếu section [Unit]".to_string());
        }
        if !has_service {
            errors.push("Thiếu section [Service]".to_string());
        }
        if !has_install {
            errors.push("Thiếu section [Install]".to_string());
        }

        // Kiểm tra các trường bắt buộc trong [Service]
        let mut in_service_section = false;
        let mut has_exec_start = false;
        for (line_num, line) in lines.iter().enumerate() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if line == "[Service]" {
                in_service_section = true;
            } else if in_service_section {
                if line.starts_with('[') {
                    in_service_section = false;
                } else if line.starts_with("ExecStart=") {
                    has_exec_start = true;
                    let exec_start_value = line.trim_start_matches("ExecStart=").trim();
                    if exec_start_value.is_empty() {
                        errors.push(format!(
                            "ExecStart trống tại dòng {}: '{}'",
                            line_num + 1,
                            line
                        ));
                    }
                }
            }
        }

        if in_service_section && !has_exec_start {
            errors.push("Thiếu trường ExecStart trong section [Service]".to_string());
        }

        // Kiểm tra cú pháp cơ bản: đảm bảo các dòng có dạng key=value
        for (line_num, line) in lines.iter().enumerate() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') || line.starts_with('[') {
                continue;
            }
            if !line.contains('=') {
                errors.push(format!(
                    "Dòng không đúng định dạng key=value tại dòng {}: '{}'",
                    line_num + 1,
                    line
                ));
            }
        }

        if errors.is_empty() {
            info!("File systemd service {} checked successfully", self.path);
            Ok(())
        } else {
            let error_msg = format!(
                "Lỗi cú pháp trong file systemd service {}:\n{}",
                self.path,
                errors.join("\n")
            );
            error!("{}", error_msg);
            Err(error_msg)
        }
    }
}