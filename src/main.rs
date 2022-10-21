use anyhow::Error;
use cli::cli;
use colored::Colorize;
use home::home_dir;

use std::process::exit;

mod cli;
mod config;
mod util;

const CONFIG_PATH: &'static str = ".goat.toml";

fn __main() -> Result<(), Error> {
    let home_path = home_dir().ok_or(Error::msg("failed to find home directory"))?;
    cli(home_path, CONFIG_PATH)?;
    Ok(())
}

fn main() {
    if let Err(error) = __main() {
        println!("{} {}", "error:".red().bold(), error);
        exit(1);
    }
}
