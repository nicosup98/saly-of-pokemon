use std::env;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

fn visit_dirs(base: &Path, dir: &Path, files: &mut Vec<(String, PathBuf)>) {
    if let Ok(entries) = fs::read_dir(dir) {
        let mut entries_vec: Vec<_> = entries.filter_map(|e| e.ok()).collect();
        entries_vec.sort_by_key(|e| e.path());
        for entry in entries_vec {
            let path = entry.path();
            if path.is_dir() {
                visit_dirs(base, &path, files);
            } else if path.is_file() {
                if let Ok(rel) = path.strip_prefix(base) {
                    let rel_str = rel.to_string_lossy().replace('\\', "/");
                    files.push((rel_str, path));
                }
            }
        }
    }
}

fn main() {
    println!("cargo:rerun-if-changed=assets");

    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let assets_dir = Path::new(&manifest_dir).join("assets");

    let mut asset_files = Vec::new();
    if assets_dir.exists() {
        visit_dirs(&assets_dir, &assets_dir, &mut asset_files);
    }

    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("embedded_assets.rs");
    let mut f = File::create(&dest_path).unwrap();

    writeln!(
        f,
        "/// Lista autogenerada de todos los assets embebidos en el binario ejecutable."
    )
    .unwrap();
    writeln!(f, "pub static EMBEDDED_ASSETS: &[(&str, &[u8])] = &[").unwrap();
    for (rel_path, abs_path) in &asset_files {
        let abs_str = abs_path.to_string_lossy().replace('\\', "/");
        writeln!(f, "    ({:?}, include_bytes!({:?})),", rel_path, abs_str).unwrap();
    }
    writeln!(f, "];").unwrap();
}
