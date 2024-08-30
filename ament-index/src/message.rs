use crate::{consts, index::ament_dirs};
use eyre::bail;
use itertools::Itertools;
use std::{
    env,
    fs::{self, File},
    io::{prelude::*, BufReader},
    path::Path,
};

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
        get_ros_msgs(ament_dirs()?.iter().copied())?
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

fn get_ros_msgs<'a, I>(paths: I) -> eyre::Result<Vec<RosMsg>>
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

fn get_ros_msgs_files(paths: &[&Path]) -> Vec<RosMsg> {
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
