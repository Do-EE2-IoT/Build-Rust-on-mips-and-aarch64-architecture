use crate::module::validate::path::{CheckResult, PathChecker};
use crate::module::validate::syntax::CheckSyntaxConfig;
use log::info;
use rayon::prelude::*;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

pub struct ToolConfig {
    pub paths: Vec<String>, // Đầu vào là danh sách các thư mục
    pub log_path: String,
}

impl ToolConfig {
    pub fn new(paths: Vec<String>, log_path: String) -> Self {
        Self { paths, log_path }
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

        let files: Vec<(String, &str)> = self
            .paths
            .iter()
            .flat_map(|path| {
                WalkDir::new(path)
                    .into_iter()
                    .filter_map(|e| e.ok())
                    .filter(|e| {
                        e.path().is_file()
                            && e.path().extension().is_some_and(|ext| {
                                let ext: String = ext.to_string_lossy().to_lowercase();
                                ext == "json" || ext == "service"
                            })
                    })
                    .map(|e| {
                        let path = e.path().to_string_lossy().into_owned();
                        let ext = e
                            .path()
                            .extension()
                            .unwrap()
                            .to_string_lossy()
                            .to_lowercase();
                        (path, if ext == "json" { "json" } else { "service" })
                    })
            })
            .collect();

        let file_count = files.len();
        info!(
            "Found {} files to check (JSON and systemd service)",
            file_count
        );

        rayon::ThreadPoolBuilder::new()
            .num_threads(4)
            .build_global()
            .map_err(|e| format!("Không thể thiết lập thread pool: {}", e))?;

        let error_count: usize = files
            .par_iter()
            .filter_map(|(path, _file_type)| {
                let syntax_config = CheckSyntaxConfig { path: path.clone() };
                if let Err(e) = syntax_config.check_file() {
                    return Some(1);
                }

                let path_checker = PathChecker::new(path.clone());
                if let Err(e) = path_checker.check_content() {
                    return Some(1);
                }
                None
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
