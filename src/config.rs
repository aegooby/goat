use std::{collections::HashMap, path::PathBuf};

use anyhow::Error;
use serde::{Deserialize, Serialize};
use tokio::{
    fs::{File, OpenOptions},
    io::{AsyncReadExt, AsyncWriteExt},
};

#[derive(Deserialize, Serialize, Debug)]
pub struct Config {
    pub current_user: Option<String>,
    #[serde(serialize_with = "toml::ser::tables_last")]
    pub users: HashMap<String, ConfigUser>,
}
impl Config {
    pub async fn from_file(path: &PathBuf) -> Result<Self, Error> {
        let mut config_str = String::new();
        let mut file = File::open(path.clone()).await?;
        file.read_to_string(&mut config_str).await?;
        let config: Config = toml::from_str(&config_str)?;
        Ok(config)
    }
    pub async fn to_file(config: &Config, path: &PathBuf) -> Result<(), Error> {
        let config_str = toml::to_string(&config)?;
        let mut file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(path.clone())
            .await?;
        file.write(config_str.as_bytes()).await?;
        Ok(())
    }
}
#[derive(Deserialize, Serialize, Debug)]
pub struct ConfigUser {
    pub email: Option<String>,
    pub token: String,
}
