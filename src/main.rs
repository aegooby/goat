use std::process::exit;

use anyhow::Error;
use cli::cli;
use colored::Colorize;
use home::home_dir;

mod cli;
mod config;
mod github;
mod util;

const CONFIG_PATH: &'static str = ".goat.toml";

async fn __main() -> Result<(), Error> {
    let home_path = home_dir().ok_or(Error::msg("failed to find home directory"))?;
    cli(home_path, CONFIG_PATH).await?;
    Ok(())
}

#[tokio::main(flavor = "multi_thread")]
pub async fn main() {
    if let Err(error) = __main().await {
        println!("{} {}", "error:".red().bold(), error);
        exit(1);
    }
}
