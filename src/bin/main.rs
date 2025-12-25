use factorio_updater::app::App;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    //let args = Args::parse();
    //handle_update(args).await?;

    simple_logging::log_to_file("debug.log", log::LevelFilter::Debug)?;

    let mut term = ratatui::init();

    let app = App::new().await;
    app.main_loop(&mut term).await?;

    ratatui::restore();

    Ok(())
}
