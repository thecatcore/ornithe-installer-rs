use chrono::{DateTime, Utc};
use reqwest::{Client, Response};
use serde::Deserialize;

const LAUNCHER_META_URL: &str = "https://skyrising.github.io/mc-versions/version_manifest.json";
const VERSION_META_URL: &str = "https://skyrising.github.io/mc-versions/version/manifest/{}.json";

pub async fn fetch_versions() -> (LatestVersions, Vec<MinecraftVersion>) {
    match super::CLIENT.get(LAUNCHER_META_URL).send().await {
        Ok(res) => {
            return res
                .json::<(LatestVersions, Vec<MinecraftVersion>)>()
                .await
                .unwrap();
        }
        Err(_) => todo!(),
    }
}

#[derive(Deserialize)]
pub struct LatestVersions {
    old_alpha: String,
    classic_server: String,
    alpha_server: String,
    old_beta: String,
    snapshot: String,
    release: String,
    pending: String,
}

#[derive(Deserialize, Clone)]
pub struct MinecraftVersion {
    id: String,
    _type: String,
    url: String,
    time: DateTime<Utc>,
    release_time: DateTime<Utc>,
    details: String,
}
