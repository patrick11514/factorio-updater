use std::time::Duration;

use anyhow::Context;
use console::style;
use indicatif::{ProgressBar, ProgressStyle};

use crate::{
    functions::get_base_query_params,
    steps::TICK_STRINGS,
    structs::{Args, Updates},
};

pub async fn get_updates(args: &Args) -> anyhow::Result<Updates> {
    let progress_style = ProgressStyle::with_template("{spinner} {prefix} {wide_msg}")
        .unwrap()
        .tick_strings(TICK_STRINGS);

    let bar = ProgressBar::new_spinner()
        .with_style(progress_style.clone())
        .with_prefix(format!(
            "{} Fetching available updates...",
            style("[1/3]").bold().blue(),
        ));
    bar.enable_steady_tick(Duration::from_millis(100));

    let client = reqwest::Client::new();
    let resp = client
        .get("https://updater.factorio.com/get-available-versions")
        .query(&get_base_query_params(args))
        .send()
        .await
        .context("Failed to get list of versions from Factorio.com")?;

    let updates: Updates = resp
        .json()
        .await
        .context("Failed to parse response from Factorio.com")?;

    bar.finish();

    Ok(updates)
}
