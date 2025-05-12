use chrono::Local;
use colored::Colorize;
use serde_json::{Map, Value};
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::thread;
use std::time::Duration;

fn read_json_file(path: &str) -> Result<Value, Box<dyn std::error::Error>> {
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let json: Value = serde_json::from_str(&contents)?;
    Ok(json)
}

fn write_json_file(path: &str, value: &Value) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::create(path)?;
    let json_str = serde_json::to_string_pretty(value)?;
    file.write_all(json_str.as_bytes())?;
    Ok(())
}

fn copy_file(src: &str, dest: &str) -> Result<(), Box<dyn std::error::Error>> {
    fs::copy(src, dest)?;
    Ok(())
}

// Hàm so sánh JSON với path chi tiết
fn compare_json(copy: &Value, modified: &Value, path: &str) -> Vec<String> {
    let mut changes = Vec::new();

    match (copy, modified) {
        (Value::Object(copy_map), Value::Object(mod_map)) => {
            // Check for modified or deleted fields
            for (key, copy_value) in copy_map {
                let new_path = if path.is_empty() {
                    key.clone()
                } else {
                    format!("{}.{}", path, key)
                };
                match mod_map.get(key) {
                    Some(mod_value) => {
                        if copy_value != mod_value {
                            // Nếu là object, đệ quy vào sâu hơn
                            if copy_value.is_object() && mod_value.is_object() {
                                changes.extend(compare_json(copy_value, mod_value, &new_path));
                            } else {
                                changes.push(format!(
                                    "Field '{}' changed from '{}' to '{}'",
                                    new_path, copy_value, mod_value
                                ));
                            }
                        }
                    }
                    None => {
                        changes.push(format!("Field '{}' was deleted", new_path));
                    }
                }
            }

            // Check for new fields
            for (key, mod_value) in mod_map {
                let new_path = if path.is_empty() {
                    key.clone()
                } else {
                    format!("{}.{}", path, key)
                };
                if !copy_map.contains_key(key) {
                    if mod_value.is_object() {
                        // Nếu là object mới, liệt kê tất cả các trường con
                        changes.extend(compare_json(&Value::Null, mod_value, &new_path));
                    } else {
                        changes.push(format!(
                            "New field '{}' added with value '{}'",
                            new_path, mod_value
                        ));
                    }
                }
            }
        }
        // Xử lý các giá trị không phải object (đối với trường hợp đệ quy)
        _ => {
            if copy != modified {
                changes.push(format!(
                    "Field '{}' changed from '{}' to '{}'",
                    path, copy, modified
                ));
            }
        }
    }

    changes
}

fn write_to_log(changes: &[String], log_file: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_file)?;

    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
    for change in changes {
        writeln!(file, "[{}] {}", timestamp, change)?;
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let original_file = "/home/do30032003/rust_project/Build-Rust-on-mips-and-aarch64-architecture/ToolCheckConfig/src/original.json";
    let cop = "/home/do30032003/rust_project/Build-Rust-on-mips-and-aarch64-architecture/ToolCheckConfig/src/copy.json";
    let modified_file = "/home/do30032003/rust_project/Build-Rust-on-mips-and-aarch64-architecture/ToolCheckConfig/src/modified.json";
    let log_file = "/home/do30032003/rust_project/Build-Rust-on-mips-and-aarch64-architecture/ToolCheckConfig/src/changes.log";

    // Initial copy from original to copy file
    copy_file(original_file, cop)?;

    loop {
        // Read copy and modified files
        match (read_json_file(cop), read_json_file(modified_file)) {
            (Ok(copy), Ok(modified)) => {
                // Compare files with detailed path
                let changes = compare_json(&copy, &modified, "");

                // Write changes to log if any
                if !changes.is_empty() {
                    write_to_log(&changes, log_file)?;
                    // In console với màu sắc
                    println!("{}", "Changes detected and logged:".yellow());
                    for change in &changes {
                        if change.contains("changed from") {
                            println!("{}", change.yellow()); // Vàng cho thay đổi
                        } else if change.contains("added") {
                            println!("{}", change.green()); // Xanh lá cho thêm
                        } else if change.contains("deleted") {
                            println!("{}", change.red()); // Đỏ cho xóa
                        }
                    }

                    // Update copy file to match modified file
                    write_json_file(cop, &modified)?;
                    println!("{}", "Updated copy file to match modified file".blue());
                } else {
                    println!("No changes detected at {}", Local::now());
                }
            }
            (Err(e), _) | (_, Err(e)) => {
                let error_msg = format!("Error reading files: {}", e);
                write_to_log(&[error_msg.clone()], log_file)?;
                println!("{}", error_msg.red()); // Lỗi màu đỏ
            }
        }

        // Wait for 10 seconds (để ý bạn đã đổi thành 5s trong code trước, tôi giữ 10s như yêu cầu ban đầu)
        thread::sleep(Duration::from_secs(5));
    }
}
