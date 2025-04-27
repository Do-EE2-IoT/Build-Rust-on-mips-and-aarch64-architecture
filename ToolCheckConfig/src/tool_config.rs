use crate::module::CheckSyntax::syntax::{CheckResult, CheckSyntaxConfig};
use log::info;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

pub struct ToolConfig {
    path: String,
    log_path: String,
}

impl ToolConfig {
    pub fn new(path: String, log_path: String) -> Self {
        Self { path, log_path }
    }

    pub fn setup_logger(&self) -> Result<(), fern::InitError> {
        fern::Dispatch::new()
            .format(|out, message, record| {
                out.finish(format_args!(
                    "{} [{}] {}",
                    chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                    record.level(),
                    message
                ))
            })
            .level(log::LevelFilter::Info)
            .chain(std::io::stdout())
            .chain(fern::log_file(&self.log_path)?)
            .apply()?;
        Ok(())
    }

    pub fn run_check_config(&self) -> CheckResult {
        self.setup_logger()
            .map_err(|e| format!("Không thể thiết lập logger: {}", e))?;

        let mut error_count = 0;
        let mut file_count = 0;

        for entry in WalkDir::new(&self.path).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_file() && path.extension().map_or(false, |ext| ext == "json") {
                file_count += 1;
                let config = CheckSyntaxConfig {
                    path: path.to_string_lossy().into_owned(),
                };
                if let Err(e) = config.check_json_file() {
                    error_count += 1;
                }
            }
        }

        info!(
            "[SUMMARY] Total files checked: {}\nErrors: {}",
            file_count, error_count
        );
        if error_count > 0 {
            Err(format!(
                "Found {} errors in configuration files",
                error_count
            ))
        } else {
            Ok(())
        }
    }
}