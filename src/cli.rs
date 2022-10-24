use std::path::PathBuf;

use anyhow::Error;
use clap::{Parser, Subcommand};
use colored::Colorize;
use tokio::process::Command;

use crate::{
    config::{Config, ConfigUser},
    github::latest_release_download,
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
    Update,
    Init {
        user: String,
        #[arg(long)]
        email: String,
    },
}

#[derive(Debug, Subcommand)]
enum TokenCommands {
    Set { user: String, key: String },
    Del { user: String },
}

pub async fn cli(base_path: PathBuf, config_path: &'static str) -> Result<(), Error> {
    let cli = Cli::parse();
    let path = ensure_config(base_path, config_path).await?;
    match &cli.command {
        Commands::Token { command } => {
            let mut config = Config::from_file(&path).await?;
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
            Config::to_file(&config, &path).await?;
        }
        Commands::Login { user } => {
            set_user(user, &path, false).await?;
        }
        Commands::Logout => {
            let output = Command::new("gh").args(["auth", "logout"]).output().await?;
            if !output.status.success() {
                return Err(Error::msg("failed to logout with gh"));
            }
            let mut config = Config::from_file(&path).await?;
            config.current_user = None;
            Config::to_file(&config, &path).await?;
            println!("{} cleared credentials", "logout:".bold());
        }
        Commands::List { show } => {
            let config = Config::from_file(&path).await?;
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
            let user = git_user().await?;
            let config = Config::from_file(&path).await?;
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
            let user = git_user().await?;
            set_user(&user, &path, true).await?;
        }
        Commands::Update => {
            println!("{} v{}", "update:".bold(), latest_release_download().await?);
        }
        Commands::Init { user, email } => {
            let user_output = Command::new("git")
                .args(["config", "--local", "user.name", user.as_str()])
                .output()
                .await?;
            if !user_output.status.success() {
                return Err(Error::msg("failed to set local git config"));
            }
            let email_output = Command::new("git")
                .args(["config", "--local", "user.email", email.as_str()])
                .output()
                .await?;
            if !email_output.status.success() {
                return Err(Error::msg("failed to set local git config"));
            }
        }
    }
    Ok(())
}
