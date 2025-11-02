use std::time::{Duration, Instant};

use console::style;
use futures_util::StreamExt;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};

use crate::{
    functions::get_base_query_params,
    steps::items::UpdateType,
    structs::{Arch, Args, Version},
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

enum UpdateInfo<'a> {
    FullGame {
        url: &'a str,
        version: &'a Version,
        number: &'a str,
    },
    Patch {
        url: &'a str,
        version: &'a Version,
        from: &'a str,
        to: &'a str,
    },
}

async fn process_zip<'a>(
    update: UpdateInfo<'a>,
    args: &Args,
    mp: Option<&MultiProgress>,
) -> anyhow::Result<()> {
    let url = match update {
        UpdateInfo::FullGame { url, .. } => url,
        UpdateInfo::Patch { url, .. } => url,
    };

    let client = reqwest::Client::new();
    let resp = client
        .get(url)
        .query(&get_base_query_params(args))
        .send()
        .await?;

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
        .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]);

    let mut pb = ProgressBar::new(total_length as u64)
        .with_style(progress_style.clone())
        .with_prefix(format!(
            "{} {}",
            style("[2/2]").bold().blue(),
            match update {
                UpdateInfo::FullGame {
                    version, number, ..
                } => format!("Downloading Factorio {} v{}...", version, number),
                UpdateInfo::Patch {
                    version, from, to, ..
                } => format!(
                    "Downloading Factorio {} patch v{} to v{}...",
                    version, from, to
                ),
            }
        ));

    if let Some(mp) = mp {
        pb = mp.add(pb);
    }

    let mut stream = resp.bytes_stream();

    let start = Instant::now();
    let mut acc = 0;
    let mut eta = Duration::ZERO;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
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
    }
    Ok(())
}

pub async fn do_update<'a>(args: &'a Args, update_type: UpdateType<'a>) -> anyhow::Result<()> {
    let arch: Arch = (args.version.clone(), args.platform.clone()).into();

    match update_type {
        UpdateType::FullGame(version) => {
            let link = get_download_links(&arch, &version);
            process_zip(
                UpdateInfo::FullGame {
                    url: &link,
                    version: &args.version,
                    number: &version,
                },
                args,
                None,
            )
            .await?;
        }
        UpdateType::Patch(items) => {
            println!("{:?}", items);
        }
        UpdateType::None => {
            println!("{}", style("No updates available.").green().bold());
        }
    }
    Ok(())
}
