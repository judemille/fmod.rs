// SPDX-FileCopyrightText: 2024 Julia DeMille <me@jdemille.com>
//
// SPDX-License-Identifier: MPL-2.0

use std::{
    env, fs,
    path::{Path, PathBuf},
    str::FromStr,
};

use bindgen::{Builder as BindingBuilder, CodegenConfig};
use snafu::{prelude::*, FromString, ResultExt, Whatever};
use target_lexicon::{
    Aarch64Architecture, Architecture, ArmArchitecture, BinaryFormat, Environment, OperatingSystem,
    Triple,
};

fn get_clang_args(fmod_dir: &Path) -> Result<Vec<String>, Whatever> {
    let include_dirs = vec![
        "api/core/inc",
        #[cfg(feature = "fsbank")]
        "api/fsbank/inc",
        #[cfg(feature = "studio")]
        "api/studio/inc",
    ];
    let include_dirs = include_dirs.into_iter().map(|dir| {
        fmod_dir
            .join(dir)
            .to_str()
            .ok_or_else(|| {
                Whatever::without_source("A path could not be converted to UTF-8!".into())
            })
            .map(ToOwned::to_owned)
    });
    let defines: Vec<&str> = vec![
        #[cfg(feature = "fsbank")]
        "_BINDGEN_FSBANK_",
        #[cfg(feature = "studio")]
        "_BINDGEN_STUDIO_",
    ];
    let args: Vec<String> = include_dirs
        .map(|dir| dir.map(|dir| format!("-I{dir}")))
        .chain(defines.into_iter().map(|def| Ok(format!("-D{def}"))))
        .collect::<Result<_, _>>()?;
    Ok(args)
}

#[allow(clippy::too_many_lines)]
fn handle_platform(fmod_dir: &Path) -> Result<(), Whatever> {
    let triple = Triple::from_str(
        &env::var("TARGET")
            .whatever_context("The environment variable TARGET was not set. This shouldn't be possible in a build script.")?,
    ).whatever_context("Could not parse target triple!")?;
    let core_libs = fmod_dir.join("api/core/lib");
    let fsbank_libs = fmod_dir.join("api/fsbank/lib");
    let studio_libs = fmod_dir.join("api/studio/lib");
    let dbg = if cfg!(debug_assertions) { "L" } else { "" };
    if matches!(triple.binary_format, BinaryFormat::Wasm) {
        if cfg!(feature = "fsbank") {
            whatever!("fsbank feature not supported on wasm!");
        }
        if cfg!(feature = "studio") {
            let studio_libs = studio_libs.join("upstream/w32");
            println!("cargo:rustc-link-search={}", studio_libs.display());
            println!(
                "cargo:rustc-link-lib=static:+verbatim=fmodstudio{dbg}:fmodstudio{dbg}_wasm.a"
            );
        } else {
            let core_libs = core_libs.join("upstream/w32");
            println!("cargo:rustc-link-search={}", core_libs.display());
            println!("cargo:rustc-link-lib=static=fmod{dbg}:fmod{dbg}_wasm.a");
        }
        return Ok(());
    }
    match triple.operating_system {
        OperatingSystem::Windows => {
            let lib_suffix = Path::new(match triple.architecture {
                Architecture::Arm(_) => "arm",
                Architecture::X86_32(_) => "x86",
                Architecture::X86_64 => "x64",
                _ => whatever!("Unsupported architecture: {}", triple.architecture),
            });
            let lib_infix = if matches!(triple.vendor, target_lexicon::Vendor::Uwp) {
                ""
            } else {
                "_vc"
            };
            println!(
                "cargo:rustc-link-search={}",
                core_libs.join(lib_suffix).display()
            );
            println!("cargo:rustc-link-lib=fmod{dbg}:fmod{dbg}{lib_infix}.lib");
            if cfg!(feature = "fsbank") {
                println!(
                    "cargo:rustc-link-search={}",
                    fsbank_libs.join(lib_suffix).display()
                );
                println!("cargo:rustc-link-lib=fsbank:fsbank_vc.lib"); // The _vc infix applies to UWP, too. Weird.
            }
            if cfg!(feature = "studio") {
                println!(
                    "cargo:rustc-link-search={}",
                    studio_libs.join(lib_suffix).display()
                );
                println!("cargo:rustc-link-lib=fmodstudio{dbg}:fmodstudio{dbg}{lib_infix}.lib");
            }
        }
        OperatingSystem::MacOSX { .. } => {
            println!("cargo:rustc-link-search={}", core_libs.display());
            println!("cargo:rustc-link-lib=fmod{dbg}");
            if cfg!(feature = "fsbank") {
                println!("cargo:rustc-link-search={}", fsbank_libs.display());
                println!("cargo:rustc-link-lib=fsbank{dbg}");
            }
            if cfg!(feature = "studio") {
                println!("cargo:rustc-link-search={}", studio_libs.display());
                println!("cargo:rustc-link-lib=fmodstudio{dbg}");
            }
        }
        OperatingSystem::Linux => {
            if matches!(
                triple.environment,
                Environment::Android | Environment::Androideabi
            ) {
                let lib_suffix = Path::new(match triple.architecture {
                    Architecture::Arm(ArmArchitecture::Armv7a) => "armeabi-v7a",
                    Architecture::Aarch64(Aarch64Architecture::Aarch64) => "arm64-v8a",
                    Architecture::X86_32(_) => "x86",
                    Architecture::X86_64 => "x86_64",
                    _ => whatever!("Unsupported architecture: {}", triple.architecture),
                });
                println!(
                    "cargo:rustc-link-search={}",
                    core_libs.join(lib_suffix).display()
                );
                println!("cargo:rustc-link-lib=fmod{dbg}");
                if cfg!(feature = "fsbank") {
                    whatever!("fsbank feature is unsupported on Android!");
                }
                if cfg!(feature = "studio") {
                    println!(
                        "cargo:rustc-link-search={}",
                        studio_libs.join(lib_suffix).display()
                    );
                    println!("cargo:rustc-link-lib=fmodstudio{dbg}");
                }
            } else {
                let lib_suffix = Path::new(match triple.architecture {
                    Architecture::Arm(_) => "arm",
                    Architecture::Aarch64(_) => "arm64",
                    Architecture::X86_32(_) => "x86",
                    Architecture::X86_64 => "x86_64",
                    _ => whatever!("Unsupported architecture: {}", triple.architecture),
                });
                println!(
                    "cargo:rustc-link-search={}",
                    core_libs.join(lib_suffix).display()
                );
                println!("cargo:rustc-link-lib=fmod{dbg}");
                if cfg!(feature = "fsbank") {
                    println!(
                        "cargo:rustc-link-search={}",
                        fsbank_libs.join(lib_suffix).display()
                    );
                    println!("cargo:rustc-link-lib=fsbank{dbg}");
                }
                if cfg!(feature = "studio") {
                    println!(
                        "cargo:rustc-link-search={}",
                        studio_libs.join(lib_suffix).display()
                    );
                    println!("cargo:rustc-link-lib=fmodstudio{dbg}");
                }
            }
        }
        OperatingSystem::Ios => {
            ios_like(
                matches!(triple.environment, Environment::Sim),
                "iphone",
                &core_libs,
                &studio_libs,
                dbg,
            )?;
        }
        OperatingSystem::Tvos => {
            ios_like(
                matches!(triple.environment, Environment::Sim),
                "appletv",
                &core_libs,
                &studio_libs,
                dbg,
            )?;
        }
        _ => whatever!("Unexpected operating system: {}", triple.operating_system),
    }
    Ok(())
}

fn ios_like(
    is_sim: bool,
    infix: &str,
    core_libs: &Path,
    studio_libs: &Path,
    dbg: &str,
) -> Result<(), Whatever> {
    let sim = if is_sim { "simulator" } else { "" };
    println!("cargo:rustc-link-search={}", core_libs.display());
    println!("cargo:rustc-link-lib=static=fmod{dbg}:libfmod{dbg}_{infix}{sim}.a");
    if cfg!(feature = "studio") {
        println!("cargo:rustc-link-search={}", studio_libs.display());
        println!("cargo:rustc-link-lib=static=fmodstudio{dbg}:libfmodstudio{dbg}_{infix}{sim}.a");
    }
    if cfg!(feature = "fsbank") {
        whatever!("fsbank feature not supported on iOS and derivatives!");
    }
    Ok(())
}

#[snafu::report]
fn main() -> Result<(), Whatever> {
    let fmod_dir = env::var("FMOD_DIR").whatever_context(
        "Please set the environment variable FMOD_DIR to the root of the Fmod Engine SDK!",
    )?;
    let fmod_dir = Path::new(&fmod_dir);
    println!("cargo:root={}", fmod_dir.display());
    ensure_whatever!(fmod_dir.exists(), "The Fmod directory does not exist!");
    ensure_whatever!(fmod_dir.is_dir(), "FMOD_DIR is not a directory!");
    if !cfg!(feature = "mockall") {
        handle_platform(fmod_dir)?;
    }
    let bindings_bld = BindingBuilder::default()
        .header("src/bindgen.h")
        .use_core()
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .prepend_enum_name(false)
        .default_enum_style(bindgen::EnumVariation::NewType {
            is_bitfield: false,
            is_global: false,
        })
        .default_macro_constant_type(bindgen::MacroTypeVariation::Signed)
        .blocklist_function("__va_start") // This symbol breaks builds on Windows, and is unneeded.
        .blocklist_function("__report_gsfailure") // Likewise.
        .clang_args(get_clang_args(fmod_dir)?);

    let bindings_fns_only = bindings_bld
        .clone()
        .with_codegen_config(CodegenConfig::FUNCTIONS)
        .generate()
        .whatever_context("Could not generate function-only bindings!")?
        .to_string();

    let bindings_except_fns = bindings_bld
        .ignore_functions()
        .generate()
        .whatever_context("Could not generate bindings without functions!")?
        .to_string();
    let bindings = &[
        r#"#[cfg(feature = "mockall")]
use mockall::automock;
#[cfg_attr(feature = "mockall", automock)]
/// An artifact of mocking support.
/// All functions live in this module.
pub mod functions {
    use super::*;
"#,
        &bindings_fns_only,
        "}\n",
        &bindings_except_fns,
    ]
    .join("");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap()).join("bindings.rs");
    fs::write(out_path, bindings.as_bytes()).expect("Could not write bindings!");
    Ok(())
}
