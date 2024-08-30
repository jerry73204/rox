pub const SUPPORTED_ROS_DISTROS: &[&str] =
    &["foxy", "galactic", "humble", "iron", "jazzy", "rolling"];

pub const WATCHED_ENV_VARS: &[&str] = &[
    "AMENT_PREFIX_PATH",
    "CMAKE_PREFIX_PATH",
    "CMAKE_IDL_PACKAGES",
    "IDL_PACKAGE_FILTER",
    "ROS_DISTRO",
];

#[cfg(target_os = "windows")]
pub const PATH_SEPARATOR: char = ';';

#[cfg(not(target_os = "windows"))]
pub const PATH_SEPARATOR: char = ':';
