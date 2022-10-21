use std::{
    path::PathBuf,
    process::{Command, Stdio},
};

use anyhow::Error;
use clap::{Parser, Subcommand};
use colored::Colorize;

use crate::{
    config::{Config, ConfigUser},
    util::{ensure_config, git_user, set_user},
};

#[derive(Debug, Parser)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Token {
        #[command(subcommand)]
        command: TokenCommands,
    },
    Login {
        user: String,
    },
    Logout,
    List {
        #[arg(long)]
        show: bool,
    },
    Info,
    Sync,
}

#[derive(Debug, Subcommand)]
enum TokenCommands {
    Set { user: String, key: String },
    Del { user: String },
}

pub fn cli(base_path: PathBuf, config_path: &'static str) -> Result<(), Error> {
    let cli = Cli::parse();
    let path = ensure_config(base_path, config_path)?;
    match &cli.command {
        Commands::Token { command } => {
            let mut config = Config::from_file(&path)?;
            match command {
                TokenCommands::Set { user, key } => {
                    config.users.insert(
                        user.clone(),
                        ConfigUser {
                            email: None,
                            token: key.clone(),
                        },
                    );
                    println!("{} updated key for user {}", "token(set):", user);
                }
                TokenCommands::Del { user } => {
                    config.users.remove(user);
                    println!("{} deleted user {}", "token(del):", user);
                }
            }
            Config::to_file(&config, &path)?;
        }
        Commands::Login { user } => {
            set_user(user, &path, false)?;
        }
        Commands::Logout => {
            let output = Command::new("gh")
                .args(["auth", "logout"])
                .stdin(Stdio::inherit())
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .output()?;
            if !output.status.success() {
                return Err(Error::msg("failed to logout with gh"));
            }
            let mut config = Config::from_file(&path)?;
            config.current_user = None;
            Config::to_file(&config, &path)?;
            println!("{} cleared credentials", "logout:".bold());
        }
        Commands::List { show } => {
            let config = Config::from_file(&path)?;
            println!("{}", "list:".bold());
            for (username, user) in config.users.iter() {
                let token = if *show {
                    user.token.clone()
                } else {
                    "*".repeat(user.token.len())
                };
                let active = if Some(username.clone()) == config.current_user {
                    format!(" {}", "(active)".green())
                } else {
                    "".to_string()
                };
                println!(" * {} -> {}{}", "user".italic(), username, active);
                println!("    - {} -> {}", "token".italic(), token);
            }
        }
        Commands::Info => {
            let user = git_user()?;
            let config = Config::from_file(&path)?;
            match config.current_user {
                Some(c_user) => {
                    if user != c_user {
                        println!("{} {}", "info:".bold(), "conflict".red().bold());
                        println!(" * {} -> {}", "git".italic(), user);
                        println!(" * {} -> {}", "gh ".italic(), c_user);
                        return Ok(());
                    }
                    println!("{} {}", "info:".bold(), "sync".green().bold());
                    println!(" * {} -> {}", "git".italic(), user);
                    println!(" * {} -> {}", "gh ".italic(), c_user);
                    return Ok(());
                }
                None => {
                    println!("{} {}", "info:".bold(), "no-auth".yellow().bold());
                    println!(" * {} -> {}", "git".italic(), user);
                    println!(" * {} -> {}", "gh ".italic(), "none".dimmed());
                    return Ok(());
                }
            }
        }
        Commands::Sync => {
            let user = git_user()?;
            set_user(&user, &path, true)?;
        }
    }
    Ok(())
}
