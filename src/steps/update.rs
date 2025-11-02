use std::{
    env::temp_dir,
    os::unix::process,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

use anyhow::Context;
use console::style;
use futures_util::{StreamExt, future::join_all};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use serde_json::json;
use tempdir::TempDir;
use tokio::{fs, io::AsyncWriteExt};
use uuid::Uuid;

use crate::{
    functions::get_base_query_params,
    steps::items::UpdateType,
    structs::{Arch, Args, Item, Version, VersionDiff},
};

fn get_download_links(arch: &Arch, version: &str) -> String {
    let base_url = "https://factorio.com/get-download";
    match arch {
        Arch::CoreLinux64 => format!("{base_url}/{version}/alpha/linux64"),
        Arch::CoreLinuxHeadless64 => format!("{base_url}/{version}/headless/linux64"),
        Arch::CoreExpansionLinux64 => format!("{base_url}/{version}/expansion/linux64"),
        Arch::CoreMac => format!("{base_url}/{version}/alpha/osx"),
        Arch::CoreExpansionMac => format!("{base_url}/{version}/expansion/osx"),
        Arch::CoreWin64 => format!("{base_url}/{version}/alpha/win64-manual"),
        Arch::CoreExpansionWin64 => format!("{base_url}/{version}/expansion/win64-manual"),
        Arch::Other => panic!(),
    }
}

async fn get_patch_download_link(args: &Args, item: &VersionDiff) -> anyhow::Result<String> {
    let arch: Arch = (args.version.clone(), args.platform.clone()).into();

    let client = reqwest::Client::new();

    let resp = client
        .get("https://updater.factorio.com/get-download-link")
        .query(&get_base_query_params(args))
        .query(&json!({
            "package": arch,
            "from": item.from,
            "to": item.to,
        }))
        .send()
        .await
        .context("Unable to send get download link request")?;

    let mut data = resp
        .json::<Vec<String>>()
        .await
        .context("Unable to parse download link response")?;

    if data.len() != 1 {
        return Err(anyhow::anyhow!("Unexpected download link response"));
    }

    Ok(data.pop().unwrap())
}

enum UpdateInfo<'a, 'b> {
    FullGame {
        url: &'b str,
        version: &'a Version,
        number: &'a str,
    },
    Patch {
        url: &'b str,
        version: &'a Version,
        from: &'a str,
        to: &'a str,
    },
}

async fn download_zip<'a, 'b>(
    update: UpdateInfo<'a, 'b>,
    args: &Args,
    mp: Option<&MultiProgress>,
    file_path: Option<&TempDir>,
) -> anyhow::Result<PathBuf> {
    let url = match update {
        UpdateInfo::FullGame { url, .. } => url,
        UpdateInfo::Patch { url, .. } => url,
    };

    let client = reqwest::Client::new();
    let resp = client
        .get(url)
        .query(&get_base_query_params(args))
        .send()
        .await
        .context("Unable to send download request")?;

    let total_length = resp
        .headers()
        .get(reqwest::header::CONTENT_LENGTH)
        .and_then(|opt| {
            Some(
                opt.to_str()
                    .unwrap_or("0")
                    .parse::<usize>()
                    .unwrap_or(0usize),
            )
        })
        .unwrap_or(0usize);

    let progress_style = ProgressStyle::with_template("{spinner} {prefix} {bar} {wide_msg}")
        .unwrap()
        .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
        .progress_chars("=O ");

    let (download_string, file_name) = match update {
        UpdateInfo::FullGame {
            version, number, ..
        } => (
            format!("Downloading Factorio {} v{}...", version, number),
            format!("factorio_{}_v{}.zip", version, number),
        ),
        UpdateInfo::Patch {
            version, from, to, ..
        } => (
            format!(
                "Downloading Factorio {} patch v{} to v{}...",
                version, from, to
            ),
            format!("factorio_{}_patch_v{}_to_v{}.zip", version, from, to),
        ),
    };

    let mut pb = ProgressBar::new(total_length as u64)
        .with_style(progress_style.clone())
        .with_prefix(format!(
            "{} {}",
            match mp {
                Some(_) => style("[2/~]").bold().blue(),
                None => style("[2/3]").bold().blue(),
            },
            download_string
        ));

    if let Some(mp) = mp {
        pb = mp.add(pb);
    }

    let mut stream = resp.bytes_stream();
    let file_path = if let Some(fp) = file_path {
        fp
    } else {
        &TempDir::new(&Uuid::new_v4().to_string()).context("Unable to create temp dir")?
    };
    let file_path = file_path.path().join(file_name);

    let mut file = fs::File::create_new(&file_path)
        .await
        .context("Unable to create zip file")?;

    let start = Instant::now();
    let mut acc = 0;
    let mut eta = Duration::ZERO;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.context("Unable to download data")?;

        pb.inc(chunk.len() as u64);

        acc += chunk.len();
        pb.set_message(format!(
            "{}/{} ({}%) - ETA: {}s",
            humansize::format_size(acc, humansize::DECIMAL),
            humansize::format_size(total_length, humansize::DECIMAL),
            (acc as f64 / total_length as f64 * 100.0) as usize,
            eta.as_secs()
        ));

        //recalculate eta
        let elapsed = start.elapsed();
        if acc > 0 && elapsed.as_secs() > 0 {
            let speed = acc as f64 / elapsed.as_secs_f64();
            let remaining = total_length as f64 - acc as f64;
            eta = Duration::from_secs_f64(remaining / speed);
        }

        file.write(&chunk).await.context("Error writing zip file")?;
    }

    file.flush().await.context("Error flushing zip file")?;

    if let Some(_) = mp {
        pb.finish_with_message("Download completed.");
    } else {
        pb.finish_and_clear();
    }

    Ok(file_path)
}

pub async fn do_update<'a>(args: &'a Args, update_type: UpdateType<'a>) -> anyhow::Result<()> {
    match update_type {
        UpdateType::FullGame(version) => process_full_version(args, &version).await?,
        UpdateType::Patch(items) => process_diff(args, items).await?,
        UpdateType::None => {
            println!("{}", style("No updates available.").green().bold());
        }
    }
    Ok(())
}

async fn process_full_version(args: &Args, version: &str) -> anyhow::Result<()> {
    let arch: Arch = (args.version.clone(), args.platform.clone()).into();
    let link = get_download_links(&arch, &version);
    let file = download_zip(
        UpdateInfo::FullGame {
            url: &link,
            version: &args.version,
            number: &version,
        },
        args,
        None,
        None,
    )
    .await?;

    Ok(())
}

async fn process_diff(args: &Args, items: Vec<&VersionDiff>) -> anyhow::Result<()> {
    let progress_style = ProgressStyle::with_template("{spinner} {prefix} {wide_msg}")
        .unwrap()
        .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]);

    let spinner = ProgressBar::new_spinner()
        .with_style(progress_style.clone())
        .with_prefix(format!(
            "{} Fetching patch download links...",
            style("[2/3]").bold().blue(),
        ));

    spinner.enable_steady_tick(Duration::from_millis(100));

    let paths = join_all(
        items
            .iter()
            .map(async |item| get_patch_download_link(&args, *item).await),
    )
    .await
    .into_iter()
    .map(|res| res.context("Unable to get download url for patch").unwrap())
    .collect::<Vec<_>>();

    spinner.finish();

    let mp = MultiProgress::new();
    let file_path =
        TempDir::new(Uuid::new_v4().to_string().as_str()).context("Unable to create temp dir")?;

    let files = join_all(
        paths
            .into_iter()
            .zip(items.iter())
            .map(async |(link, patch)| {
                let link = link;

                download_zip(
                    UpdateInfo::Patch {
                        url: &link,
                        version: &args.version,
                        from: &patch.from,
                        to: &patch.to,
                    },
                    &args,
                    Some(&mp),
                    Some(&file_path),
                )
                .await
                .context("Unable to download patch")
                .unwrap()
            }),
    )
    .await;

    mp.clear().unwrap();

    println!("{:?}", files);

    Ok(())
}
