use anyhow::Error;
use clap::{Parser, Subcommand};
use colored::Colorize;
use home::home_dir;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs::{create_dir_all, File, OpenOptions},
    io::{Read, Write},
    path::PathBuf,
    process::{Command, Stdio},
};

const CONFIG_PATH: &'static str = ".goat.toml";

#[derive(Deserialize, Serialize, Debug)]
struct Config {
    current_user: Option<String>,
    #[serde(serialize_with = "toml::ser::tables_last")]
    users: HashMap<String, ConfigUser>,
}
impl Config {
    fn from_file(path: &PathBuf) -> Result<Self, Error> {
        let mut config_str = String::new();
        let mut file = File::open(path.clone())?;
        file.read_to_string(&mut config_str)?;
        let config: Config = toml::from_str(&config_str)?;
        Ok(config)
    }
    fn to_file(config: &Config, path: &PathBuf) -> Result<(), Error> {
        let config_str = toml::to_string(&config)?;
        let mut file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(path.clone())?;
        write!(file, "{}", config_str)?;
        Ok(())
    }
}
#[derive(Deserialize, Serialize, Debug)]
struct ConfigUser {
    email: Option<String>,
    token: String,
}

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
    User {
        user: String,
    },
    Sync {
        args: Vec<String>,
    },
}

#[derive(Debug, Subcommand)]
enum TokenCommands {
    Set { user: String, key: String },
    Del { user: String },
}

fn ensure_config() -> Result<PathBuf, Error> {
    let home_path = home_dir().ok_or(Error::msg("failed to find home directory"))?;
    let path = home_path.join(CONFIG_PATH);
    if !path.exists() {
        if let Some(parent) = path.parent() {
            create_dir_all(parent)?;
            let mut file = File::create(path.clone())?;
            write!(
                file,
                "{}",
                toml::to_string(&Config {
                    users: HashMap::new(),
                    current_user: None
                })?
            )?;
        }
    }
    Ok(path)
}

fn __main() -> Result<(), Error> {
    let cli = Cli::parse();
    let path = ensure_config()?;
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
                }
                TokenCommands::Del { user } => {
                    config.users.remove(user);
                }
            }
            Config::to_file(&config, &path)?;
        }
        Commands::User { user } => {
            let config = Config::from_file(&path)?;
            match config.users.get(user) {
                Some(c_user) => {
                    let mut cmd = Command::new("gh")
                        .args(["auth", "login", "--with-token"])
                        .stdin(Stdio::piped())
                        .stdout(Stdio::inherit())
                        .stderr(Stdio::inherit())
                        .spawn()?;
                    let cmd_stdin = cmd
                        .stdin
                        .as_mut()
                        .ok_or(Error::msg("could not get gh stdin"))?;
                    write!(cmd_stdin, "{}", c_user.token)?;
                    Command::new("gh")
                        .args(["auth", "status"])
                        .stderr(Stdio::inherit())
                        .output()?;
                    let mut config = Config::from_file(&path)?;
                    config.current_user = Some(user.clone());
                    Config::to_file(&config, &path)?;
                }
                None => {
                    return Err(Error::msg(format!("no token found for user '{}'", user)));
                }
            }
        }
        Commands::Sync { args } => {
            let mut cmd = Command::new("git")
                .args(["config", "--local", "user.name"])
                .stdout(Stdio::piped())
                .stderr(Stdio::inherit())
                .stdin(Stdio::inherit())
                .spawn()?;
            let cmd_stdout = cmd
                .stdout
                .as_mut()
                .ok_or(Error::msg("could not get git stdin"))?;
            let mut user = String::new();
            cmd_stdout.read_to_string(&mut user)?;
            user = user.trim().to_string();
            if user.is_empty() {
                let mut cmd = Command::new("git")
                    .args(["config", "user.name"])
                    .stdout(Stdio::piped())
                    .stderr(Stdio::inherit())
                    .stdin(Stdio::inherit())
                    .spawn()?;
                let cmd_stdout = cmd
                    .stdout
                    .as_mut()
                    .ok_or(Error::msg("could not get git stdin"))?;
                cmd_stdout.read_to_string(&mut user)?;
            }
            if user.is_empty() {
                return Err(Error::msg("could not find git config username"));
            }
            let config = Config::from_file(&path)?;
            match config.current_user {
                Some(c_user) => {
                    if user != c_user {
                        return Err(Error::msg(format!(
                            "attempted to use git with user '{}', but authenticated with user '{}'",
                            user, c_user
                        )));
                    }
                }
                None => {
                    return Err(Error::msg(
                        "no GitHub user authenticated, use 'user' command",
                    ));
                }
            }
            println!("{} {}", "current user:".bold(), user);
        }
    }
    Ok(())
}

fn main() {
    if let Err(error) = __main() {
        println!("{} {}", "error:".red().bold(), error)
    }
}
