//! Build script of `tarok_env`: a fingerprint of the sources the module is
//! built from (decision D130).
//!
//! The module is installed into a Python environment and stays there when
//! the engine's sources change. This script writes down what it was built
//! from: one line per source file with its path, size and CRC-32, and a hash
//! of those lines. `tarok_train/env.py` computes the same text from the files
//! on disk and refuses to start the environment when it differs.
//!
//! Counted: `Cargo.toml` and every `.rs` file under `src` of the engine
//! crate; `Cargo.toml`, `Cargo.lock`, `build.rs` and every `.rs` file under
//! `src` of this crate. The same list is in `tarok_train/env.py`.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

/// Crate folder under the repo's `src`, and its counted files outside `src`.
const CRATES: [(&str, &[&str]); 2] = [
    ("engine", &["Cargo.toml"]),
    ("train/tarok_env", &["Cargo.toml", "Cargo.lock", "build.rs"]),
];

/// CRC-32 as zlib computes it (Python's `zlib.crc32`).
fn crc32(bytes: &[u8]) -> u32 {
    let mut crc = !0u32;
    for &byte in bytes {
        crc ^= u32::from(byte);
        for _ in 0..8 {
            crc = if crc & 1 == 1 {
                (crc >> 1) ^ 0xEDB8_8320
            } else {
                crc >> 1
            };
        }
    }
    !crc
}

/// FNV-1a, 64 bits.
fn fnv1a(bytes: &[u8]) -> u64 {
    let mut hash = 0xCBF2_9CE4_8422_2325u64;
    for &byte in bytes {
        hash = (hash ^ u64::from(byte)).wrapping_mul(0x0000_0100_0000_01B3);
    }
    hash
}

/// Every `.rs` file under `folder`, with its name from the repo's `src` on.
fn rust_files(folder: &Path, name: &str, files: &mut Vec<(String, PathBuf)>) {
    let entries = fs::read_dir(folder)
        .unwrap_or_else(|error| panic!("cannot list {}: {error}", folder.display()));
    for entry in entries {
        let entry =
            entry.unwrap_or_else(|error| panic!("cannot list {}: {error}", folder.display()));
        let path = entry.path();
        let entry_name = format!("{name}/{}", entry.file_name().to_string_lossy());
        if path.is_dir() {
            rust_files(&path, &entry_name, files);
        } else if entry_name.ends_with(".rs") {
            files.push((entry_name, path));
        }
    }
}

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("set by cargo"));
    let root = manifest_dir.join("..").join("..");

    let mut files = Vec::new();
    for (folder, names) in CRATES {
        let base = root.join(folder);
        for name in names {
            let path = base.join(name);
            println!("cargo:rerun-if-changed={}", path.display());
            if path.is_file() {
                files.push((format!("{folder}/{name}"), path));
            }
        }
        let sources = base.join("src");
        println!("cargo:rerun-if-changed={}", sources.display());
        rust_files(&sources, &format!("{folder}/src"), &mut files);
    }
    files.sort();

    let mut manifest = String::new();
    for (name, path) in &files {
        let bytes = fs::read(path)
            .unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()));
        let (size, crc) = (bytes.len(), crc32(&bytes));
        manifest.push_str(&format!("{name}\t{size}\t{crc:08x}\n"));
    }

    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("set by cargo"));
    fs::write(out_dir.join("source_manifest.txt"), &manifest)
        .unwrap_or_else(|error| panic!("cannot write the source manifest: {error}"));
    let hash = fnv1a(manifest.as_bytes());
    println!("cargo:rustc-env=TAROK_SOURCE_HASH={hash:016x}");
}
