mod consts;

use eyre::bail;
use itertools::{chain, Itertools};
use once_cell::sync::OnceCell;
use os_str_bytes::RawOsString;
use std::{
    collections::HashMap,
    env,
    fs::{self, File},
    io::{prelude::*, BufReader},
    path::{Path, PathBuf},
};

pub fn print_cargo_watches() {
    for var in consts::WATCHED_ENV_VARS {
        println!("cargo:rerun-if-env-changed={}", var);
    }
}

pub fn ament_dirs() -> eyre::Result<impl Iterator<Item = &'static Path>> {
    static AMENT_PREFIX_PATH: OnceCell<RawOsString> = OnceCell::new();
    static CMAKE_PREFIX_PATH: OnceCell<Option<RawOsString>> = OnceCell::new();

    let ament_prefix_path = AMENT_PREFIX_PATH.get_or_try_init(|| {
        let Some(ament_prefix_path) = env::var_os("AMENT_PREFIX_PATH") else {
            bail!("Source your ROS!")
        };
        Ok(RawOsString::new(ament_prefix_path))
    })?;
    let cmake_prefix_path =
        CMAKE_PREFIX_PATH.get_or_init(|| env::var_os("CMAKE_PREFIX_PATH").map(RawOsString::new));

    let ament_dirs = chain!(
        ament_prefix_path.split(consts::PATH_SEPARATOR),
        cmake_prefix_path.as_deref()
    )
    .map(|dir| Path::new(dir));

    Ok(ament_dirs)
}

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

    let builder = ament_dirs()?.try_fold(builder, |builder, ament_dir| {
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

pub fn get_wanted_messages() -> eyre::Result<Vec<RosMsg>> {
    let msgs = if let Ok(cmake_idl_packages) = env::var("CMAKE_IDL_PACKAGES") {
        // CMAKE_PACKAGE_DIRS should be a (cmake) list of "cmake" dirs
        // e.g. For each dir install/r2r_minimal_node_msgs/share/r2r_minimal_node_msgs/cmake
        // we can traverse back and then look for .msg files in msg/ srv/ action/
        let dirs: Vec<&Path> = cmake_idl_packages
            .split(consts::PATH_SEPARATOR)
            .filter_map(|i| Path::new(i).parent())
            .collect();

        get_ros_msgs_files(&dirs)
    } else {
        let ament_dirs = ament_dirs()?.map(Path::new);
        get_ros_msgs(ament_dirs)?
    };

    // let msgs = parse_msgs(&msgs);

    // When working on large workspaces without colcon, build times
    // can be a pain. This code adds a the possibility to define an
    // additional filter to make building a little bit quicker.
    //
    // The environment variable IDL_PACKAGE_FILTER should be a semicolon
    // separated list of package names (e.g. std_msgs;my_msgs), so it
    // is required to be correct for packages to be used. This means
    // dependencies need to be manually specified.
    //
    // Suitable to customize with .cargo/config.toml [env] from consumers
    // of the r2r package.
    let needed_msg_pkgs = &[
        "rcl_interfaces",
        "builtin_interfaces",
        "unique_identifier_msgs",
        "action_msgs",
    ];

    let msgs = if let Ok(idl_filter) = env::var("IDL_PACKAGE_FILTER") {
        let mut idl_packages: Vec<&str> = idl_filter.split(consts::PATH_SEPARATOR).collect();
        for needed in needed_msg_pkgs {
            if !idl_packages.contains(needed) {
                idl_packages.push(needed);
            }
        }
        msgs.into_iter()
            .filter(|msg| idl_packages.contains(&msg.module.as_str()))
            .collect()
    } else {
        msgs
    };

    Ok(msgs)
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RosMsg {
    pub module: String, // e.g. std_msgs
    pub prefix: String, // e.g. "msg" or "srv"
    pub name: String,   // e.g. "String"
}

fn get_msgs_from_package(package_dir: &Path) -> eyre::Result<Vec<RosMsg>> {
    let rosidl_interfaces_file = package_dir
        .join("share")
        .join("ament_index")
        .join("resource_index")
        .join("rosidl_interfaces");

    let mut msgs: Vec<_> = fs::read_dir(rosidl_interfaces_file)?
        .map(|entry| {
            let entry = entry?;
            let path = entry.path();
            let module = path.file_name().unwrap().to_str().unwrap().to_string();
            let lines = BufReader::new(File::open(&path)?).lines();

            let msgs = lines
                .map(move |line| -> eyre::Result<Option<_>> {
                    let line = line?;

                    let tokens: Vec<_> = line.split('/').take(3).collect();
                    let &[prefix, msg_file_name] = tokens.as_slice() else {
                        bail!("invalid message format \"{line}\"");
                    };
                    let Some((msg_name, ext)) = msg_file_name.rsplit_once('.') else {
                        return Ok(None);
                    };

                    let included = match prefix {
                        "msg" => ["idl", "msg"].contains(&ext),
                        "srv" => ["idl", "srv"].contains(&ext),
                        "action" => ["idl", "action"].contains(&ext),
                        _ => false,
                    };

                    let msg = included.then(|| RosMsg {
                        module: module.clone(),
                        prefix: prefix.to_string(),
                        name: msg_name.to_string(),
                    });
                    eyre::Ok(msg)
                })
                .flatten_ok();

            eyre::Ok(msgs)
        })
        .flatten_ok()
        .flatten_ok()
        .try_collect()?;

    msgs.sort();
    msgs.dedup();
    Ok(msgs)
}

pub fn get_ros_msgs<'a, I>(paths: I) -> eyre::Result<Vec<RosMsg>>
where
    I: IntoIterator<Item = &'a Path>,
{
    let mut msgs: Vec<RosMsg> = Vec::new();

    for p in paths {
        let package_msgs = get_msgs_from_package(p)?;
        msgs.extend(package_msgs)
    }
    msgs.sort();
    msgs.dedup();
    Ok(msgs)
}

fn get_msgs_in_dir(base: &Path, subdir: &str, package: &str) -> Vec<RosMsg> {
    let path = base.join(subdir);

    let mut msgs = vec![];

    if let Ok(paths) = fs::read_dir(path) {
        for path in paths {
            let path = path.unwrap().path();
            let filename = path.file_name().unwrap().to_str().unwrap();

            // message name.idl or name.msg
            if !filename.ends_with(".idl") {
                continue;
            }

            let substr = &filename[0..filename.len() - 4];

            msgs.push(RosMsg {
                module: package.to_string(),
                prefix: subdir.to_string(),
                name: substr.to_string(),
            });
        }
    }
    msgs
}

pub fn get_ros_msgs_files(paths: &[&Path]) -> Vec<RosMsg> {
    let mut msgs: Vec<RosMsg> = Vec::new();

    for p in paths {
        if let Some(package_name) = p.file_name() {
            let package_name = package_name.to_str().unwrap();
            msgs.extend(get_msgs_in_dir(p, "msg", package_name));
            msgs.extend(get_msgs_in_dir(p, "srv", package_name));
            msgs.extend(get_msgs_in_dir(p, "action", package_name));
        }
    }
    msgs.sort();
    msgs.dedup();
    msgs
}

pub fn parse_msgs(msgs: &[String]) -> Vec<RosMsg> {
    let v: Vec<Vec<&str>> = msgs
        .iter()
        .map(|l| l.split('/').take(3).collect())
        .collect();

    // hack because I don't have time to find out the root cause of this at the moment.
    // for some reason the library files generated to this are called
    // liblibstatistics_collector_test_msgs__..., but I don't know where test_msgs come from.
    // (this seems to be a useless package anyway)
    // also affects message generation below.
    v.iter()
        .filter(|v| v.len() == 3)
        .map(|v| RosMsg {
            module: v[0].into(),
            prefix: v[1].into(),
            name: v[2].into(),
        })
        .filter(|v| v.module != "libstatistics_collector")
        .collect()
}

pub fn as_map(included_msgs: &[RosMsg]) -> HashMap<&str, HashMap<&str, Vec<&str>>> {
    let mut msgs = HashMap::new();
    for msg in included_msgs {
        msgs.entry(msg.module.as_str())
            .or_insert_with(HashMap::new)
            .entry(msg.prefix.as_str())
            .or_insert_with(Vec::new)
            .push(msg.name.as_str());
    }
    msgs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_msgs() {
        let msgs = "
std_msgs/msg/Bool
x/y
std_msgs/msg/String
";
        let msgs = msgs.lines().map(|l| l.to_string()).collect::<Vec<_>>();
        let parsed = parse_msgs(&msgs);
        assert_eq!(parsed[0].module, "std_msgs");
        assert_eq!(parsed[0].prefix, "msg");
        assert_eq!(parsed[0].name, "Bool");
        assert_eq!(parsed[1].module, "std_msgs");
        assert_eq!(parsed[1].prefix, "msg");
        assert_eq!(parsed[1].name, "String");
    }

    #[test]
    fn test_as_map() {
        let msgs = "
std_msgs/msg/Bool
x/y
std_msgs/msg/String
";
        let msgs: Vec<String> = msgs.lines().map(|l| l.to_string()).collect();
        let parsed = parse_msgs(&msgs);
        let map = as_map(&parsed);

        assert_eq!(map.get("std_msgs").unwrap().get("msg").unwrap()[0], "Bool");
        assert_eq!(
            map.get("std_msgs").unwrap().get("msg").unwrap()[1],
            "String"
        );
    }
}
