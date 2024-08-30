use crate::{consts, index::ament_dirs};
use eyre::bail;
use itertools::Itertools;
use os_str_bytes::RawOsString;
use std::{
    env, fs,
    path::{Path, PathBuf},
};

pub fn setup_bindgen_builder() -> eyre::Result<bindgen::Builder> {
    let builder = bindgen::Builder::default()
        .layout_tests(false)
        .derive_copy(false)
        .size_t_is_usize(true)
        .default_enum_style(bindgen::EnumVariation::Rust {
            non_exhaustive: false,
        });

    let builder = if let Some(cmake_includes) = env::var_os("CMAKE_INCLUDE_DIRS") {
        let cmake_includes = RawOsString::from(cmake_includes);

        // note, this is a colon on both windows and linux, it is set
        // in r2r_cargo.cmake
        let mut includes = cmake_includes
            .split(consts::PATH_SEPARATOR)
            .collect::<Vec<_>>();
        includes.sort_unstable();
        includes.dedup();

        includes.iter().fold(builder, |builder, include| {
            let include = Path::new(include);
            let clang_arg = format!("-I{}", include.display());
            builder.clang_arg(clang_arg)
        })
    } else {
        builder
    };

    let builder = ament_dirs()?
        .iter()
        .try_fold(builder, |builder, ament_dir| {
            let include_dir = Path::new(ament_dir).join("include");

            let entries = fs::read_dir(include_dir)?;
            let dirs: Vec<PathBuf> = entries
                .map(|entry| {
                    let entry = entry?;
                    let is_dir = entry.metadata()?.is_dir();
                    let path = is_dir.then(|| entry.path());
                    eyre::Ok(path)
                })
                .flatten_ok()
                .try_collect()?;

            let builder = dirs.iter().fold(builder, |builder, d| {
                // Hack to build rolling after https://github.com/ros2/rcl/pull/959 was merged.
                //
                // The problem is that now we need to use CMAKE to properly find the
                // include paths. But we don't want to do that so we hope that the ros
                // developers use the same convention everytime they move the include
                // files to a subdirectory.
                //
                // The convention is to put include files in include/${PROJECT_NAME}
                //
                // So we check if there is a double directory on the form
                // include/${PROJECT_NAME}/${PROJECT_NAME}, and if so append it only once.
                //
                // Should work mostly, and shouldn't really change often, so manual
                // intervention could be applied. But yes it is hacky.
                if let Some(leaf) = d.file_name() {
                    let double_include_path = Path::new(d).join(leaf);
                    if double_include_path.is_dir() {
                        let temp = d.to_str().unwrap();
                        builder.clang_arg(format!("-I{}", temp))
                    } else {
                        // pre humble case, where we did not have include/package/package
                        let temp = d.parent().unwrap().to_str().unwrap();
                        builder.clang_arg(format!("-I{}", temp))
                    }
                } else {
                    builder
                }
            });

            eyre::Ok(builder)
        })?;

    Ok(builder)
}

pub fn print_cargo_ros_distro() -> eyre::Result<()> {
    let Ok(ros_distro) = env::var("ROS_DISTRO") else {
        bail!("ROS_DISTRO is not set or is not UTF-8: Source your ROS!");
    };

    if consts::SUPPORTED_ROS_DISTROS.contains(&ros_distro.as_str()) {
        println!("cargo:rustc-cfg=r2r__ros__distro__{ros_distro}");
    } else {
        bail!("ROS_DISTRO not supported: {ros_distro}");
    }

    Ok(())
}

pub fn print_cargo_link_search() -> eyre::Result<()> {
    for path in ament_dirs()? {
        if cfg!(target_os = "windows") {
            let lib_path = Path::new(path).join("Lib");
            if !lib_path.exists() {
                continue;
            }
            if let Some(s) = lib_path.to_str() {
                println!("cargo:rustc-link-search={}", s);
            }
        } else {
            let lib_path = Path::new(&path.as_os_str()).join("lib");
            if let Some(s) = lib_path.to_str() {
                println!("cargo:rustc-link-search=native={}", s)
            }
        }
    }

    Ok(())
}

pub fn print_cargo_watches() {
    for var in consts::WATCHED_ENV_VARS {
        println!("cargo:rerun-if-env-changed={}", var);
    }
}
