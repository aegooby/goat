use std::env;

use anyhow::Error;
use colored::Colorize;

fn build() -> Result<(), Error> {
    let target_os = env::var("CARGO_CFG_TARGET_OS")?;
    match target_os.as_str() {
        "linux" => {
            env::set_var("TARGET", "x86_64-unknown-linux-gnu-gcc");
        }
        _ => {}
    }
    Ok(())
}

fn main() {
    if let Err(error) = build() {
        println!("{} {}", "build error:".bold().red(), error);
    }
}
