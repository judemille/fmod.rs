// SPDX-FileCopyrightText: 2024 Julia DeMille <me@jdemille.com>
//
// SPDX-License-Identifier: MPL-2.0

use std::fs;

use camino::Utf8PathBuf;
use fmod_build_utils::{emit_lib_link, make_bindings_builders};

fn main() {
    println!("cargo::rerun-if-env-changed=FMOD_SDK_DIR");
    let target = std::env::var("TARGET").expect("Couldn't get TARGET env var");
    assert!([
        "i686-pc-windows-msvc",
        "x86_64-pc-windows-msvc",
        "i686-unknown-linux-gnu",
        "x86_64-unknown-linux-gnu",
        "aarch64-apple-darwin",
        "x86_64-apple-darwin"
    ]
    .contains(&target.as_ref()));

    let is_windows = ["i686-pc-windows-msvc", "x86_64-pc-windows-msvc"].contains(&target.as_ref());

    let fmod_sdk_dir = std::env::var("FMOD_SDK_DIR")
        .expect("FMOD_SDK_DIR should be set to the root of your Fmod SDK");
    let api_dir = Utf8PathBuf::from(fmod_sdk_dir).join("api").join("fsbank");
    let debug_logging = std::env::var("DEBUG")
        .expect("Cargo should set DEBUG, but it didn't?")
        .parse::<bool>()
        .map(|res| {
            // Fucking Windows has to be special.
            if is_windows {
                false
            } else {
                res
            }
        })
        .unwrap();
    emit_lib_link(&api_dir, "fsbank", debug_logging);
    println!(
        "cargo::rustc-link-lib=dylib={}",
        if target == "i686-pc-windows-msvc" {
            "libfsbvorbis"
        } else if target == "x86_64-pc-windows-msvc" {
            "libfsbvorbis64"
        } else {
            "fsbvorbis"
        }
    );
    println!("cargo::rustc-link-lib=dylib=opus");

    let inc_dir = api_dir.join("inc");
    println!("cargo::metadata=include={inc_dir}");
    let (fns_only, no_fns) = make_bindings_builders("src/bindgen.h", &inc_dir, &[]);

    let bindings = &[
        r#"#[cfg(feature = "mockall")]
use mockall::automock;
#[cfg_attr(feature = "mockall", automock)]
/// An artifact of mocking support.
/// All functions live in this module.
pub mod functions {
    use super::*;"#,
        &fns_only,
        "}",
        &no_fns,
    ]
    .join("\n");

    let out_path =
        std::path::PathBuf::from(std::env::var_os("OUT_DIR").expect("Cargo should set OUT_DIR"))
            .join("bindings.rs");

    fs::write(out_path, bindings.as_bytes()).expect("Should be able to write bindings");
}
