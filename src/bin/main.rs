use clap::Parser;
use factorio_updater::{steps::handle_update, structs::Args};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    handle_update(args).await?;

    Ok(())
}
