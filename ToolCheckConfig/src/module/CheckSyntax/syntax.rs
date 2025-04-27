use log::error;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

pub type CheckResult = Result<(), String>;

#[derive(Clone)]
pub struct CheckSyntaxConfig {
    pub path: String,
}

impl CheckSyntaxConfig {
    pub fn check_json_file(&self) -> CheckResult {
        // Mở file và đọc theo luồng
        let file = File::open(&self.path)
            .map_err(|e| format!("Không thể đọc file {}: {}", self.path, e))?;
        let reader = BufReader::new(file);

        // Parse JSON từ luồng
        match serde_json::from_reader::<_, serde_json::Value>(reader) {
            Ok(_) => {
                log::info!("File {} checked successfully", self.path);
                Ok(())
            }
            Err(e) => {
                let error_msg = e.to_string();
                let line_number = e.line();
                let column = e.column();

                // Đọc lại file để lấy dòng lỗi (chỉ đọc khi cần)
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
}