use clap::{Parser, ValueEnum, command};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt::Display};

#[derive(Serialize, Deserialize, Debug, ValueEnum, Clone)]
pub enum Version {
    #[serde(rename = "vanilla")]
    Vanilla,
    #[serde(rename = "space-age")]
    SpaceAge,
}

impl Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Version::Vanilla => write!(f, "vanilla"),
            Version::SpaceAge => write!(f, "space-age"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, ValueEnum, Clone)]
pub enum Platform {
    //#[serde(rename = "linux32")]
    //Linux32,
    #[serde(rename = "linux64")]
    Linux64,
    #[serde(rename = "mac")]
    Mac,
    //#[serde(rename = "mac-arm64")]
    //MacArm64,
    //#[serde(rename = "mac-x64")]
    //MacX64,
    //#[serde(rename = "win32")]
    //Win32,
    #[serde(rename = "win64")]
    Win64,
}

#[derive(Serialize, Deserialize, Debug, Clone, ValueEnum, PartialEq, Eq, Hash)]
pub enum Arch {
    //    #[serde(rename = "core-linux32")]
    //    CoreLinux32,
    #[serde(rename = "core-linux64")]
    CoreLinux64,
    #[serde(rename = "core-linux_headless64")]
    CoreLinuxHeadless64,
    #[serde(rename = "core-mac")]
    CoreMac,
    //    #[serde(rename = "core-mac-arm64")]
    //    CoreMacArm64,
    //    #[serde(rename = "core-mac-x64")]
    //    CoreMacX64,
    //    #[serde(rename = "core-win32")]
    //    CoreWin32,
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

impl From<(Version, Platform)> for Arch {
    fn from((version, platform): (Version, Platform)) -> Self {
        match (version, platform) {
            //(Version::Vanilla, Platform::Linux32) => Arch::CoreLinux32,
            (Version::Vanilla, Platform::Linux64) => Arch::CoreLinux64,
            (Version::Vanilla, Platform::Mac) => Arch::CoreMac,
            //(Version::Vanilla, Platform::MacArm64) => Arch::CoreMacArm64,
            //(Version::Vanilla, Platform::MacX64) => Arch::CoreMacX64,
            //(Version::Vanilla, Platform::Win32) => Arch::CoreWin32,
            (Version::Vanilla, Platform::Win64) => Arch::CoreWin64,
            (Version::SpaceAge, Platform::Linux64) => Arch::CoreExpansionLinux64,
            (Version::SpaceAge, Platform::Mac) => Arch::CoreExpansionMac,
            (Version::SpaceAge, Platform::Win64) => Arch::CoreExpansionWin64,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct VersionDiff {
    pub from: String,
    pub to: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Stable {
    pub stable: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum Item {
    VersionDiff(VersionDiff),
    Stable(Stable),
}

pub type Updates = HashMap<Arch, Vec<Item>>;
