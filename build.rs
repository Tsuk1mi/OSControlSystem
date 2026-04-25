//! Встраивание манифеста и сведений о версии в PE (Свойства файла в Проводнике).
//! Устранение предупреждений SmartScreen у незнакомых программ — только Authenticode-подпись
//! (см. `scripts/sign-release.ps1`).

use std::env;
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=windows/app.manifest");

    let target = env::var("TARGET").unwrap_or_default();
    if !target.contains("windows") {
        return;
    }

    let manifest_path = Path::new(env::var("CARGO_MANIFEST_DIR").unwrap().as_str())
        .join("windows")
        .join("app.manifest");
    if !manifest_path.exists() {
        println!(
            "cargo:warning=manifest not found: {}",
            manifest_path.display()
        );
        return;
    }
    let manifest_file = match manifest_path.to_str() {
        Some(s) => s,
        None => {
            println!("cargo:warning=invalid manifest path (non-utf8)");
            return;
        }
    };

    let pkg_ver = env::var("CARGO_PKG_VERSION").unwrap_or_else(|_| "0.1.0".to_string());
    let file_ver = four_part_version(&pkg_ver);

    let mut res = winres::WindowsResource::new();
    res.set_manifest_file(manifest_file);
    res.set("FileDescription", "OS Control Assistant");
    res.set("ProductName", "OS Control Assistant");
    res.set("OriginalFilename", "OSControlAssistant.exe");
    res.set("InternalName", "OSControlAssistant");
    res.set("CompanyName", "OS Control Assistant");
    res.set("LegalCopyright", "Copyright (C) OS Control Assistant");
    res.set("FileVersion", &file_ver);
    res.set("ProductVersion", &pkg_ver);

    if let Err(e) = res.compile() {
        println!("cargo:warning=winres (metadata not embedded): {e}");
    }
}

/// Версия для PE: четыре числа, например 0.1.0 -> 0.1.0.0
fn four_part_version(pkg: &str) -> String {
    let mut parts: Vec<u32> = pkg
        .split(|c| c == '.' || c == '-')
        .filter_map(|s| s.parse().ok())
        .collect();
    while parts.len() < 4 {
        parts.push(0);
    }
    parts.truncate(4);
    format!("{}.{}.{}.{}", parts[0], parts[1], parts[2], parts[3])
}
