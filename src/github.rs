use std::env::{consts::OS, current_exe};

use anyhow::Error;
use futures_util::stream::StreamExt;
use octocrab::instance;
use reqwest::{get, Url};
use tokio::{fs::OpenOptions, io::AsyncWriteExt};

const REPO_OWNER: &'static str = "aegooby";
const REPO_NAME: &'static str = "goat";

pub async fn latest_release_download() -> Result<String, Error> {
    let gh_client = instance();
    let latest_release = gh_client
        .repos(REPO_OWNER, REPO_NAME)
        .releases()
        .get_latest()
        .await?;
    let download_url = latest_release
        .assets
        .iter()
        .find_map(|asset| {
            if asset.name.contains(OS) {
                Some(asset.browser_download_url.clone())
            } else {
                None
            }
        })
        .ok_or(Error::msg(format!(
            "no release found for operating system {}",
            OS
        )))?;

    let mut stream = get::<Url>(download_url.as_str().parse()?)
        .await?
        .bytes_stream();
    let mut bin = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(current_exe()?)
        .await?;
    while let Some(chunk) = stream.next().await {
        bin.write_all_buf(&mut chunk?).await?;
    }
    Ok(latest_release.tag_name)
}
