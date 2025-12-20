#[derive(Parser)]
#[command(
    about = "Factorio Updater CLI",
    long_about = "A command line interface, for fetching and updating Factorio versions using the Factorio Updater API. It keeps track of each platform's current downloaded version, and uses patches to patch them quickly, instead of downloading full version."
)]
pub struct Args {
    /// Which version of Factorio to update
    #[arg(long, default_value = "vanilla")]
    pub version: Version,
    /// Which platform to update
    #[arg(long, default_value = "win64")]
    pub platform: Platform,
    /// Your factorio.com username (for authentication)
    #[arg(long)]
    pub username: String,
    /// Your factorio.com token (for authentication)
    #[arg(long, env)]
    pub token: String,
    #[arg(long)]
    pub custom_folder: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub version: Version,
    pub platform: Platform,
    pub current_version: String,
}
