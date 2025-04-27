use crate::module::CheckSyntax::syntax::{CheckResult, CheckSyntaxConfig};
use log::{error, info};
use rayon::prelude::*;
use std::fs::{self, File};
use std::io::BufReader;
use std::path::Path;
use walkdir::WalkDir;

pub struct ToolConfig {
    pub path: String,
    pub log_path: String,
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

        // Thu thập danh sách file JSON
        let json_files: Vec<String> = WalkDir::new(&self.path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path().is_file() && e.path().extension().map_or(false, |ext| ext == "json")
            })
            .map(|e| e.path().to_string_lossy().into_owned())
            .collect();

        let file_count = json_files.len();
        info!("Found {} JSON files to check", file_count);

        // Giới hạn số luồng để tránh quá tải
        rayon::ThreadPoolBuilder::new()
            .num_threads(4) // Giới hạn 4 luồng
            .build_global()
            .map_err(|e| format!("Không thể thiết lập thread pool: {}", e))?;

        // Kiểm tra file song song
        let error_count: usize = json_files
            .par_iter()
            .filter_map(|path| {
                let config = CheckSyntaxConfig { path: path.clone() };
                config.check_json_file().err().map(|_| 1)
            })
            .sum();

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
