// SPDX-FileCopyrightText: 2024 Julia DeMille <me@jdemille.com>
//
// SPDX-License-Identifier: MPL-2.0

use std::fs;

use camino::Utf8PathBuf;
use fmod_build_utils::{emit_lib_link, make_bindings_builders};

fn main() {
    println!("cargo::rerun-if-env-changed=FMOD_SDK_DIR");
    let fmod_sdk_dir = std::env::var("FMOD_SDK_DIR")
        .expect("FMOD_SDK_DIR should be set to the root of your Fmod SDK");
    let api_dir = Utf8PathBuf::from(fmod_sdk_dir).join("api").join("studio");
    let debug_logging = std::env::var("DEBUG")
        .expect("Cargo should set DEBUG, but it didn't?")
        .parse::<bool>()
        .unwrap();
    emit_lib_link(&api_dir, "fmod", debug_logging);

    let inc_dir = api_dir.join("inc");
    println!("cargo::metadata=include={inc_dir}");
    let core_inc_dir = std::env::var("DEP_FMOD_INCLUDE")
        .map(Utf8PathBuf::from)
        .expect("DEP_FMOD_INCLUDE should be set and valid UTF-8");
    let (fns_only, no_fns) = make_bindings_builders("src/bindgen.h", &inc_dir, &[&core_inc_dir]);

    let bindings = &[
        r#"use fmod_sys::*;

        #[cfg(feature = "mockall")]
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
