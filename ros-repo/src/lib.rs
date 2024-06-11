use anyhow::{bail, Result};
use itertools::izip;
use rayon::prelude::*;
use ros_package_manifest::Package;
use std::{
    collections::{hash_map::Entry, HashMap},
    path::{Path, PathBuf},
};
use strong_xml::XmlRead;

pub fn resolve<P>(dir: P) -> Result<()>
where
    P: AsRef<Path>,
{
    // Find manifest files
    let manifest_paths = find_manifest_files(dir);

    // Load contents from manifest files
    let manifest_texts: Result<Vec<_>, _> = manifest_paths
        .par_iter()
        .map(std::fs::read_to_string)
        .collect();
    let manifest_texts = manifest_texts?;

    // Parse manifest file contents
    let manifests: Result<Vec<_>, _> = manifest_texts
        .par_iter()
        .map(|text| Package::from_str(text))
        .collect();
    let manifests = manifests?;

    // Check duplicated package names
    {
        let mut names: HashMap<&str, &Path> = HashMap::with_capacity(manifests.len());

        for (path, manifest) in izip!(&manifest_paths, &manifests) {
            let name: &str = &manifest.name.0;
            match names.entry(name) {
                Entry::Occupied(entry) => {
                    let offending_path = entry.get();
                    bail!("the manifest files {offending_path:?} and {path:?} have the same package name '{name}'");
                }
                Entry::Vacant(entry) => {
                    entry.insert(path);
                }
            }
        }
    }

    Ok(())
}

fn find_manifest_files(dir: impl AsRef<Path>) -> Vec<PathBuf> {
    let mut stack = vec![dir.as_ref().to_path_buf()];
    let mut manifest_paths = vec![];

    while let Some(dir) = stack.pop() {
        let manifest_path = dir.join("package.xml");

        if manifest_path.exists() {
            manifest_paths.push(manifest_path);
        } else {
            let Ok(read_dir) = std::fs::read_dir(dir) else {
                continue;
            };

            for entry in read_dir {
                let Ok(entry) = entry else { continue };
                let Ok(metadata) = entry.metadata() else {
                    continue;
                };

                let path = if metadata.is_dir() {
                    entry.path()
                } else if metadata.is_file() {
                    continue;
                } else if metadata.is_symlink() {
                    let path = entry.path();

                    let Ok(can_path) = path.canonicalize() else {
                        continue;
                    };

                    if !can_path.is_dir() {
                        continue;
                    }

                    path
                } else {
                    continue;
                };

                stack.push(path);
            }
        }
    }

    manifest_paths
}
