use std::{collections::HashMap, path::PathBuf, process::Stdio};

use anyhow::Error;
use colored::Colorize;
use tokio::{
    fs::{create_dir_all, File},
    io::{AsyncReadExt, AsyncWriteExt},
    process::Command,
};

use crate::config::Config;

pub async fn ensure_config(
    base_path: PathBuf,
    config_path: &'static str,
) -> Result<PathBuf, Error> {
    let path = base_path.join(config_path);
    if !path.exists() {
        if let Some(parent) = path.parent() {
            create_dir_all(parent).await?;
            let mut file = File::create(path.clone()).await?;
            file.write(
                toml::to_string(&Config {
                    users: HashMap::new(),
                    current_user: None,
                })?
                .as_bytes(),
            )
            .await?;
        }
    }
    Ok(path)
}

pub async fn git_user() -> Result<String, Error> {
    let mut cmd = Command::new("git")
        .args(["config", "--local", "user.name"])
        .stdout(Stdio::piped())
        .spawn()?;
    let cmd_stdout = cmd
        .stdout
        .as_mut()
        .ok_or(Error::msg("could not get git stdin"))?;
    let mut user = String::new();
    cmd_stdout.read_to_string(&mut user).await?;
    user = user.trim().to_string();
    if user.is_empty() {
        let mut cmd = Command::new("git")
            .args(["config", "user.name"])
            .stdout(Stdio::piped())
            .spawn()?;
        let cmd_stdout = cmd
            .stdout
            .as_mut()
            .ok_or(Error::msg("could not get git stdin"))?;
        cmd_stdout.read_to_string(&mut user).await?;
    }
    if user.is_empty() {
        return Err(Error::msg("could not find git config username"));
    }
    Ok(user)
}

pub async fn set_user(user: &String, path: &PathBuf, sync: bool) -> Result<(), Error> {
    let config = Config::from_file(&path).await?;
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
            cmd_stdin.write(c_user.token.as_bytes()).await?;
            let output = cmd.wait_with_output().await?;
            if !output.status.success() {
                return Err(Error::msg("failed to authenticate with gh"));
            }
            let mut config = Config::from_file(&path).await?;
            config.current_user = Some(user.clone());
            Config::to_file(&config, &path).await?;

            println!("{} logged in as {}", op, user);
            Ok(())
        }
        None => Err(Error::msg(format!("no token found for user '{}'", user))),
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::path::Path;

    #[tokio::test]
    async fn test_ensure_config() -> Result<(), Error> {
        let base_path = Path::new(".test").to_path_buf();
        let config_path = ".goat-test.toml";
        ensure_config(base_path.clone(), config_path).await?;
        assert!(base_path.join(config_path).exists());
        Ok(())
    }
}
