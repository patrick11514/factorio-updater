use anyhow::Context;
use semver::Version;
use tokio::fs;

use crate::{
    functions::load_config,
    structs::{Arch, Args, Item, Updates, VersionDiff},
};

pub fn get_arch_folder(arch: &Arch) -> &'static str {
    match arch {
        Arch::CoreLinux64 => "linux64",
        Arch::CoreMac => "mac",
        Arch::CoreWin64 => "win64",
        Arch::CoreExpansionLinux64 => "spaceage-linux64",
        Arch::CoreExpansionMac => "spaceage-mac",
        Arch::CoreExpansionWin64 => "spaceage-win64",
        Arch::CoreLinuxHeadless64 => "linux-server",
        Arch::Other => panic!(),
    }
}

pub enum UpdateType<'a> {
    FullGame(String),
    Patch(Vec<&'a VersionDiff>),
    None,
}

pub async fn resolve_updates<'a>(
    args: &mut Args,
    updates: &'a Updates,
) -> anyhow::Result<UpdateType<'a>> {
    let arch = (args.version.clone(), args.platform.clone()).into();
    let folder = if let Some(folder) = &args.custom_folder {
        folder.as_str()
    } else {
        get_arch_folder(&arch)
    };

    let base_folder = std::path::Path::new(folder);

    if !fs::try_exists(base_folder)
        .await
        .context("Failed to check if folder exists")?
    {
        fs::create_dir(base_folder)
            .await
            .context("Failed to create folder")?;
    }

    let config = load_config(base_folder).await?;
    if let Some(config) = &config {
        args.version = config.version.clone();
        args.platform = config.platform.clone();
    }

    let arch: Arch = (args.version.clone(), args.platform.clone()).into();

    let items = updates
        .get(&arch)
        .context("No updates available for this architecture")?;

    let stable = items
        .iter()
        .find(|item| matches!(item, Item::Stable(_)))
        .and_then(|item| {
            if let Item::Stable(stable) = item {
                Some(stable.stable.clone())
            } else {
                None
            }
        })
        .context("No stable version found for full game download")?;

    let config = if let Some(config) = config {
        config
    } else {
        return Ok(UpdateType::FullGame(stable));
    };

    if Version::parse(&config.current_version).context("Unable to parse config version")?
        == Version::parse(&stable).context("Unable to parse stable version")?
    {
        return Ok(UpdateType::None);
    }

    let mut collected_updates = Vec::new();
    let mut to_walk = vec![&config.current_version];

    loop {
        let version = if let Some(version) = to_walk.pop() {
            version
        } else {
            break;
        };

        match items.iter().find(|item| match item {
            Item::VersionDiff(version_diff) => version_diff.from == *version,
            Item::Stable(stable) => stable.stable == *version,
        }) {
            Some(item) => match item {
                Item::VersionDiff(version_diff) => {
                    //accumulate path for patching
                    collected_updates.push(version_diff);
                    to_walk.push(&version_diff.to);
                }
                //we reached stable version, so stop here
                Item::Stable(_) => break,
            },
            None => {
                //Somehow we can't find a diff
                return Ok(UpdateType::FullGame(stable));
            }
        }
    }

    Ok(UpdateType::Patch(collected_updates))
}
