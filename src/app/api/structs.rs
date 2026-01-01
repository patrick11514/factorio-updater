use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt::Display};

#[derive(Serialize, Deserialize, Debug, ValueEnum, Clone)]
pub enum Version {
    #[serde(rename = "vanilla")]
    Vanilla,
    #[serde(rename = "space-age")]
    SpaceAge,
    #[serde(rename = "headless")]
    Headless,
}

pub static ALL_VERSIONS: &[Version] = &[Version::Vanilla, Version::SpaceAge, Version::Headless];

pub fn get_versions_by_platform(platform: &Platform) -> Vec<Version> {
    ALL_VERSIONS
        .iter()
        .map(|version| (version, Arch::from((version, platform))))
        .filter(|(_, arch)| !matches!(arch, Arch::Other))
        .map(|(version, _)| version.clone())
        .collect()
}

impl Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Version::Vanilla => write!(f, "Vanilla"),
            Version::SpaceAge => write!(f, "Space Age"),
            Version::Headless => write!(f, "Headless"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, ValueEnum, Clone)]
pub enum Platform {
    #[serde(rename = "linux32")]
    Linux32,
    #[serde(rename = "linux64")]
    Linux64,
    #[serde(rename = "mac")]
    Mac,
    #[serde(rename = "mac-arm64")]
    MacArm64,
    #[serde(rename = "mac-x64")]
    MacX64,
    #[serde(rename = "win32")]
    Win32,
    #[serde(rename = "win64")]
    Win64,
}

impl Display for Platform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Platform::Linux32 => write!(f, "Linux 32-bit"),
            Platform::Linux64 => write!(f, "Linux 64-bit"),
            Platform::Mac => write!(f, "Mac"),
            Platform::MacArm64 => write!(f, "Mac ARM64"),
            Platform::MacX64 => write!(f, "Mac x64"),
            Platform::Win32 => write!(f, "Windows 32-bit"),
            Platform::Win64 => write!(f, "Windows 64-bit"),
        }
    }
}

pub static ALL_PLATFORMS: &[Platform] = &[
    Platform::Linux32,
    Platform::Linux64,
    //Platform::Mac, //contains the .dmg file
    //Platform::MacArm64,
    //Platform::MacX64,
    Platform::Win32,
    Platform::Win64,
];

#[derive(Serialize, Deserialize, Debug, Clone, ValueEnum, PartialEq, Eq, Hash)]
pub enum Arch {
    #[serde(rename = "core-linux32")]
    CoreLinux32,
    #[serde(rename = "core-linux64")]
    CoreLinux64,
    #[serde(rename = "core-linux_headless64")]
    CoreLinuxHeadless64,
    #[serde(rename = "core-mac")]
    CoreMac,
    #[serde(rename = "core-mac-arm64")]
    CoreMacArm64,
    #[serde(rename = "core-mac-x64")]
    CoreMacX64,
    #[serde(rename = "core-win32")]
    CoreWin32,
    #[serde(rename = "core-win64")]
    CoreWin64,
    #[serde(rename = "core_expansion-linux64")]
    CoreExpansionLinux64,
    #[serde(rename = "core_expansion-mac")]
    CoreExpansionMac,
    #[serde(rename = "core_expansion-win64")]
    CoreExpansionWin64,

    #[serde(other)]
    #[clap(skip)]
    Other,
}

impl Display for Arch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Arch::CoreLinux32 => write!(f, "core-linux32"),
            Arch::CoreLinux64 => write!(f, "core-linux64"),
            Arch::CoreLinuxHeadless64 => write!(f, "core-linux_headless64"),
            Arch::CoreMac => write!(f, "core-mac"),
            Arch::CoreMacArm64 => write!(f, "core-mac-arm64"),
            Arch::CoreMacX64 => write!(f, "core-mac-x64"),
            Arch::CoreWin32 => write!(f, "core-win32"),
            Arch::CoreWin64 => write!(f, "core-win64"),
            Arch::CoreExpansionLinux64 => write!(f, "core_expansion-linux64"),
            Arch::CoreExpansionMac => write!(f, "core_expansion-mac"),
            Arch::CoreExpansionWin64 => write!(f, "core_expansion-win64"),
            Arch::Other => write!(f, "other"),
        }
    }
}

impl From<(Version, Platform)> for Arch {
    fn from((version, platform): (Version, Platform)) -> Self {
        (&version, &platform).into()
    }
}

impl From<(&Version, &Platform)> for Arch {
    fn from((version, platform): (&Version, &Platform)) -> Self {
        match (version, platform) {
            (Version::Vanilla, Platform::Linux32) => Arch::CoreLinux32,
            (Version::Vanilla, Platform::Linux64) => Arch::CoreLinux64,
            (Version::Vanilla, Platform::Mac) => Arch::CoreMac,
            (Version::Vanilla, Platform::MacArm64) => Arch::CoreMacArm64,
            (Version::Vanilla, Platform::MacX64) => Arch::CoreMacX64,
            (Version::Vanilla, Platform::Win32) => Arch::CoreWin32,
            (Version::Vanilla, Platform::Win64) => Arch::CoreWin64,
            (Version::SpaceAge, Platform::Linux64) => Arch::CoreExpansionLinux64,
            (Version::SpaceAge, Platform::Mac) => Arch::CoreExpansionMac,
            (Version::SpaceAge, Platform::Win64) => Arch::CoreExpansionWin64,
            (Version::Headless, Platform::Linux64) => Arch::CoreLinuxHeadless64,
            _ => Arch::Other,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct VersionDiff {
    pub from: String,
    pub to: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct Stable {
    pub stable: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(untagged)]
pub enum Item {
    VersionDiff(VersionDiff),
    Stable(Stable),
}

impl Item {
    pub fn to_string_raw(&self) -> &str {
        match self {
            Item::VersionDiff(v) => &v.from,
            Item::Stable(s) => &s.stable,
        }
    }
}

impl Display for Item {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Item::VersionDiff(v) => write!(f, "{}", v.from),
            Item::Stable(s) => write!(f, "{} (stable)", s.stable),
        }
    }
}

impl PartialOrd for Item {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let self_version = semver::Version::parse(match self {
            Item::VersionDiff(v) => &v.from,
            Item::Stable(s) => &s.stable,
        })
        .unwrap();

        let other_version = semver::Version::parse(match other {
            Item::VersionDiff(v) => &v.from,
            Item::Stable(s) => &s.stable,
        })
        .unwrap();

        Some(self_version.cmp(&other_version))
    }
}

impl Ord for Item {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

pub type Updates = HashMap<Arch, Vec<Item>>;
