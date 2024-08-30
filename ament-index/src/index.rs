use crate::consts;
use eyre::bail;
use indexmap::{IndexMap, IndexSet};
use itertools::{chain, Itertools};
use once_cell::sync::OnceCell;
use os_str_bytes::RawOsString;
use std::{env, path::Path};

#[derive(Debug, Clone)]
pub struct AmentIndex {
    pub ament_dirs: &'static IndexSet<&'static Path>,
    pub packages: IndexMap<String, PackageIndex>,
}

#[derive(Debug, Clone)]
pub struct PackageIndex {
    pub ament_dir: &'static Path,
}

impl AmentIndex {
    pub fn from_env() -> eyre::Result<Self> {
        let ament_dirs = ament_dirs()?;

        let packages: IndexMap<_, _> = ament_dirs
            .iter()
            .map(|ament_dir| {
                let share_dir = ament_dir.join("share");
                let resource_index_dir = share_dir.join("ament_index").join("resource_index");
                let packages_dir = resource_index_dir.join("packages");

                let packages: Vec<_> = packages_dir
                    .read_dir()?
                    .map(|entry| {
                        let entry = entry?;
                        let package_name = entry.file_name().into_string().unwrap();
                        eyre::Ok(package_name)
                    })
                    .map_ok(|package_name| (package_name, PackageIndex { ament_dir }))
                    .try_collect()?;

                eyre::Ok(packages)
            })
            .flatten_ok()
            .try_collect()?;

        Ok(Self {
            ament_dirs,
            packages,
        })
    }
}

pub fn ament_index() -> eyre::Result<&'static AmentIndex> {
    static AMENT_INDEX: OnceCell<AmentIndex> = OnceCell::new();
    AMENT_INDEX.get_or_try_init(AmentIndex::from_env)
}

pub fn ament_dirs() -> eyre::Result<&'static IndexSet<&'static Path>> {
    static AMENT_PREFIX_PATH: OnceCell<RawOsString> = OnceCell::new();
    static CMAKE_PREFIX_PATH: OnceCell<Option<RawOsString>> = OnceCell::new();
    static AMENT_DIRS: OnceCell<IndexSet<&'static Path>> = OnceCell::new();

    let ament_prefix_path = AMENT_PREFIX_PATH.get_or_try_init(|| {
        let Some(ament_prefix_path) = env::var_os("AMENT_PREFIX_PATH") else {
            bail!("Source your ROS!")
        };
        Ok(RawOsString::new(ament_prefix_path))
    })?;
    let cmake_prefix_path =
        CMAKE_PREFIX_PATH.get_or_init(|| env::var_os("CMAKE_PREFIX_PATH").map(RawOsString::new));

    let ament_dirs = AMENT_DIRS.get_or_init(|| {
        chain!(
            ament_prefix_path.split(consts::PATH_SEPARATOR),
            cmake_prefix_path
                .as_ref()
                .map(|s| s.split(consts::PATH_SEPARATOR))
                .into_iter()
                .flatten()
        )
        .map(Path::new)
        .collect()
    });

    Ok(ament_dirs)
}
