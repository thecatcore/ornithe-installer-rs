use chrono::{DateTime, Utc};
use serde::Deserialize;
use serde_json::{Map, Value};

use crate::errors::InstallerError;

use super::GameSide;

const LAUNCHER_META_URL: &str = "https://skyrising.github.io/mc-versions/version_manifest.json";
const VERSION_META_URL: &str = "https://skyrising.github.io/mc-versions/version/manifest/{}.json";

pub async fn fetch_versions() -> Result<(LatestVersions, Vec<MinecraftVersion>), reqwest::Error> {
    super::CLIENT
        .get(LAUNCHER_META_URL)
        .send()
        .await?
        .json::<(LatestVersions, Vec<MinecraftVersion>)>()
        .await
}

pub async fn fetch_launch_json(version: &MinecraftVersion) -> Result<String, InstallerError> {
    let res = super::CLIENT
        .get(VERSION_META_URL.replace("{}", version.id.as_str()))
        .send()
        .await?;
    if let Some(val) = res.json::<Value>().await?.as_object_mut() {
        let version_details = fetch_version_details(&version).await?;

        for manifest_url in version_details.manifests {
            if let Some(manifest) = super::CLIENT
                .get(manifest_url)
                .send()
                .await?
                .json::<Value>()
                .await?
                .as_object()
            {
                build_version_json_from_manifest(val, manifest);
            }
        }

        val.insert(
            "id".to_string(),
            Value::String(format!("{}-vanilla", version.id.clone())),
        );

        return Ok(serde_json::to_string_pretty(val)?);
    }
    Err(InstallerError("Error".to_string()))
}

fn build_version_json_from_manifest(
    version_json: &mut Map<String, Value>,
    manifest: &Map<String, Value>,
) {
    for entry in manifest {
        if version_json.contains_key(entry.0) {
            let version_json_element = version_json.get_mut(entry.0).unwrap();
            let manifest_element = entry.1;

            if version_json_element != manifest_element
                && version_json_element.is_object()
                && manifest_element.is_object()
            {
                build_version_json_from_manifest(
                    version_json_element.as_object_mut().unwrap(),
                    manifest_element.as_object().unwrap(),
                );
            }
        } else {
            version_json.insert(entry.0.to_string(), entry.1.clone());
        }
    }
}

async fn fetch_version_details(
    version: &MinecraftVersion,
) -> Result<VersionDetails, reqwest::Error> {
    super::CLIENT
        .get(version.details.clone())
        .send()
        .await?
        .json::<VersionDetails>()
        .await
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
    pub id: String,
    _type: String,
    url: String,
    time: DateTime<Utc>,
    release_time: DateTime<Utc>,
    details: String,
}

impl MinecraftVersion {
    pub async fn get_id(&self, side: &GameSide) -> Result<String, reqwest::Error> {
        if fetch_version_details(self).await?.shared_mappings {
            Ok(self.id.clone())
        } else {
            Ok(self.id.clone() + "-" + side.id())
        }
    }
}

#[derive(Deserialize)]
pub struct VersionDetails {
    manifests: Vec<String>,
    shared_mappings: bool,
    normalized_version: String,
}
