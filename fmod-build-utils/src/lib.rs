// SPDX-FileCopyrightText: 2024 Julia DeMille <me@jdemille.com>
//
// SPDX-License-Identifier: MPL-2.0

use bindgen::{Builder as BindingBuilder, CodegenConfig, EnumVariation, MacroTypeVariation};
use camino::Utf8Path;

pub fn emit_lib_link(api_dir: &Utf8Path, lib_name: &str, debug_logging: bool) {
    let lib_dir = api_dir.join("lib");

    let target = std::env::var("TARGET").expect("Couldn't get TARGET env var");
    let tgt_components = target.split('-').collect::<Vec<_>>();
    let l_suffix = if debug_logging { "L" } else { "" };
    let (lib_dir, lib_name, kind, modifiers) = match tgt_components.as_slice() {
        [arch, "unknown", "linux", "gnu" | "gnueabihf"] => {
            // Linux.
            (
                lib_dir.join(match *arch {
                    "aarch64" => "arm64",
                    "armv7" => "arm",
                    "i686" => "x86",
                    "x86_64" => arch,
                    _ => panic!("unsupported Linux architecture {arch}!"),
                }),
                format!("{lib_name}{l_suffix}"),
                "dylib",
                None,
            )
        }
        [arch, "pc", "windows", "msvc" | "gnu"] => {
            // Windows.
            (
                lib_dir.join(match *arch {
                    "i686" => "x86",
                    "x86_64" => "x64",
                    _ => panic!("unsupported Windows architecture {arch}!"),
                }),
                format!("{lib_name}{l_suffix}_vc"),
                "dylib",
                None,
            )
        }
        ["x86_64" | "aarch64", "apple", "darwin"] => {
            (lib_dir, format!("{lib_name}{l_suffix}"), "dylib", None)
        }
        [arch, "apple", os @ ("ios" | "tvos"), tail @ ..] => {
            // iOS/tvOS
            let is_sim = tail == ["sim"];
            (
                lib_dir,
                format!(
                    "{lib_name}{l_suffix}_{}",
                    match (*arch, *os, is_sim) {
                        ("aarch64" | "arm64e", "ios", false) => "iphoneos",
                        ("aarch64" | "x86_64", "ios", true) => "iphonesimulator",
                        ("arm64e", "tvos", false) => "appletvos",
                        ("aarch64" | "x86_64", "tvos", true) => "appletvsimulator",
                        (_, _, _) => panic!("unsupported iOS-like target {target}!"),
                    }
                ),
                "static",
                None,
            )
        }
        ["wasm32", "unknown", "emscripten"] => (
            lib_dir.join("upstream").join("w32"),
            format!("{lib_name}{l_suffix}_wasm.a"),
            "static",
            Some("+verbatim"),
        ),
        [arch, "linux", "android" | "androideabi"] => {
            // Android.
            (
                lib_dir.join(match *arch {
                    "aarch64" => "arm64-v8a",
                    "armv7" => "armeabi-v7a",
                    "i686" => "x86",
                    "x86_64" => arch,
                    _ => panic!("unsupported Android architecture {arch}!"),
                }),
                format!("{lib_name}{l_suffix}"),
                "dylib",
                None,
            )
        }
        [arch, "uwp", "windows", "msvc" | "gnu"] => {
            // UWP.
            (
                lib_dir.join(match *arch {
                    "i686" => "x86",
                    "x86_64" => "x64",
                    "thumbv7a" => "arm",
                    _ => panic!("unsupported UWP architecture {arch}"),
                }),
                format!("{lib_name}{l_suffix}"),
                "dylib",
                None,
            )
        }
        _ => panic!("unsupported target triple {target}"),
    };
    println!("cargo::rustc-link-search={lib_dir}");
    println!(
        "cargo::rustc-link-lib={}{}={}",
        kind,
        modifiers.map_or_else(|| String::new(), |m| format!(":{m}")),
        lib_name
    );
}

pub fn make_bindings_builders(
    header: &str,
    include_dir: &Utf8Path,
    extra_include_dirs: &[&Utf8Path],
) -> (String, String) {
    let mut base_builder = BindingBuilder::default()
        .header(header)
        .use_core()
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .prepend_enum_name(false)
        .default_enum_style(EnumVariation::NewType {
            is_bitfield: false,
            is_global: false,
        })
        .default_macro_constant_type(MacroTypeVariation::Signed)
        .allowlist_recursively(false)
        .allowlist_file(format!(
            r#"{}[\\/].*"#,
            include_dir.to_string().replace('\\', "\\\\")
        ))
        .allowlist_file(format!(
            r#"{}[\\/]src[\\/]bindgen\.h"#,
            std::env::current_dir().unwrap().to_str().unwrap().replace('\\', "\\\\")
        ))
        .clang_args(extra_include_dirs.iter().map(|it| format!("-I{it}")))
        .clang_arg(format!("-I{include_dir}"))
        .merge_extern_blocks(true);

    if std::env::var("TARGET")
        .map(|it| it == "wasm32-unknown-emscripten")
        .unwrap_or(false)
    {
        base_builder = base_builder.clang_args(&["-DDLL_EXPORTS", "-DF_USE_ATTRIBUTE"]);
    }

    let bindings_fns_only = base_builder
        .clone()
        .with_codegen_config(CodegenConfig::FUNCTIONS)
        .generate()
        .unwrap()
        .to_string();

    let bindings_except_fns = base_builder
        .ignore_functions()
        .generate()
        .unwrap()
        .to_string();

    (bindings_fns_only, bindings_except_fns)
}
