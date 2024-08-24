use glob::glob;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 使用 glob 模式匹配找到 protos 目錄下的所有 .proto 文件
    let proto_files: Vec<PathBuf> = glob("./protos/**/*.proto")
        .expect("Failed to read glob pattern 無法讀取全域模式")
        .filter_map(Result::ok)
        .collect();

    // 使用 tonic_build 來編譯 .proto 文件
    tonic_build::configure()
        .protoc_arg("--experimental_allow_proto3_optional")
        .build_server(true)
        .build_client(true)
        .out_dir("./src")
        .type_attribute(".", "#[derive(serde::Serialize,serde::Deserialize)]")
        .compile(
            &proto_files, // 使用找到的所有.proto文件
            &["protos"],
        )?;

    //  將產生的檔案寫入 src/lib.rs
    let src_dir = Path::new("src");
    let lib_path = src_dir.join("lib.rs");

    let existing_content = fs::read_to_string(&lib_path)?;
    let mut new_content = String::new();

    for entry in fs::read_dir(src_dir)? {
        let entry = entry?;
        let path = entry.path();
        let file_name = match path.file_name().and_then(|n| n.to_str()) {
            Some(name) => name,
            None => continue,
        };

        // Skip lib.rs itself and non-Rust files
        if file_name == "lib.rs" || !file_name.ends_with(".rs") {
            continue;
        }

        let mod_name = file_name.trim_end_matches(".rs");
        let mod_decl = format!("pub mod {};", mod_name);

        // Check if the module declaration already exists
        if !existing_content.contains(&mod_decl) {
            new_content.push_str(&mod_decl);
            new_content.push('\n');
        }
    }

    if !new_content.is_empty() {
        let mut file = OpenOptions::new().append(true).open(lib_path)?;
        writeln!(file, "\n{}", new_content)?;
    }

    let _status: std::process::ExitStatus = Command::new("cargo")
        .arg("fmt")
        .status()
        .expect("Failed to execute `cargo fmt`");

    Ok(())
}
