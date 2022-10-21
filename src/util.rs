use anyhow::Error;
use colored::Colorize;
use home::home_dir;
use std::{
    collections::HashMap,
    fs::{create_dir_all, File},
    io::{Read, Write},
    path::PathBuf,
    process::{Command, Stdio},
};

use crate::config::Config;

pub fn ensure_config(base_path: PathBuf, config_path: &'static str) -> Result<PathBuf, Error> {
    let path = base_path.join(config_path);
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

pub fn git_user() -> Result<String, Error> {
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
    Ok(user)
}

pub fn set_user(user: &String, path: &PathBuf, sync: bool) -> Result<(), Error> {
    let config = Config::from_file(&path)?;
    let op = if sync { "sync:" } else { "login:" }.bold();
    if let Some(cc_user) = config.current_user {
        if cc_user == *user {
            println!("{} already logged in as {}", op, user);
            return Ok(());
        }
    }
    match config.users.get(user) {
        Some(c_user) => {
            let mut cmd = Command::new("gh")
                .args(["auth", "login", "--with-token"])
                .stdin(Stdio::piped())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()?;
            let cmd_stdin = cmd
                .stdin
                .as_mut()
                .ok_or(Error::msg("could not get gh stdin"))?;
            write!(cmd_stdin, "{}", c_user.token)?;
            let output = cmd.wait_with_output()?;
            if !output.status.success() {
                return Err(Error::msg("failed to authenticate with gh"));
            }
            let mut config = Config::from_file(&path)?;
            config.current_user = Some(user.clone());
            Config::to_file(&config, &path)?;

            println!("{} logged in as {}", op, user);
            Ok(())
        }
        None => Err(Error::msg(format!("no token found for user '{}'", user))),
    }
}
