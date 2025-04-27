
mod module;
mod tool_config;
use tool_config::ToolConfig;

fn main() {
    let tool = ToolConfig::new(
        "/home/do30032003/rust_project/Build-Rust-on-mips-and-aarch64-architecture/ToolCheckConfig/src/test.json".to_string(),
        "/home/do30032003/rust_project/Build-Rust-on-mips-and-aarch64-architecture/ToolCheckConfig/src/test.log".to_string(),
    );

    match tool.run_check_config() {
        Ok(()) => println!("All configuration files checked successfully!"),
        Err(e) => eprintln!("Error: {}", e),
    }
}