use std::collections::HashMap;

use serde::Deserialize;
use serde_json::Value;

use crate::errors::InstallerError;

use super::{GameSide, manifest::MinecraftVersion};

const META_URL: &str = "https://meta.ornithemc.net";

#[derive(Deserialize, Clone)]
pub struct LoaderVersion {
    pub version: String,
    stable: bool,
    maven: String,
    separator: String,
    build: i32,
    #[serde(rename(deserialize = "versionNoSide"))]
    version_no_side: String,
}

#[derive(PartialEq, Eq, Hash, Clone)]
pub enum LoaderType {
    Fabric,
    Quilt,
}

impl LoaderType {
    pub fn get_name(&self) -> &str {
        match self {
            LoaderType::Fabric => "fabric",
            LoaderType::Quilt => "quilt",
        }
    }

    pub fn get_localized_name(&self) -> &str {
        match self {
            LoaderType::Fabric => "Fabric",
            LoaderType::Quilt => "Quilt",
        }
    }
}

impl GameSide {
    fn launch_json_endpoint(&self) -> &str {
        match self {
            GameSide::Client => "/v3/versions/{}-loader/{}/{}/profile/json",
            GameSide::Server => "/v3/versions/{}-loader/{}/{}/server/json",
        }
    }
}

pub async fn fetch_launch_json(
    side: GameSide,
    version: &MinecraftVersion,
    loader_type: &LoaderType,
    loader_version: &LoaderVersion,
) -> Result<String, InstallerError> {
    let mut text = super::CLIENT
        .get(
            META_URL.to_owned()
                + &side
                    .launch_json_endpoint()
                    .replacen("{}", loader_type.get_name(), 1)
                    .replacen("{}", version.get_id(&side).await?.as_str(), 1)
                    .replacen("{}", &loader_version.version, 1),
        )
        .send()
        .await?
        .json::<Value>()
        .await?;
    if let Some(libraries) = text["libraries"].as_object_mut() {
        for lib in libraries {
            let lib_mut = lib.1.as_object_mut().unwrap();
            if let Some(name) = lib_mut.clone()["name"].as_str() {
                if name.starts_with("net.fabricmc:intermediary") {
                    lib_mut.insert(
                        "name".to_string(),
                        Value::String(name.replace(
                            "net.fabricmc:intermediary",
                            "net.ornithemc:calamus-intermediary",
                        )),
                    );
                    lib_mut.insert(
                        "url".to_string(),
                        Value::String("https://maven.ornithemc.net/releases".to_string()),
                    );
                }
                if name.starts_with("org.quiltmc:hashed") {
                    lib_mut.insert(
                        "name".to_string(),
                        Value::String(
                            name.replace(
                                "org.quiltmc:hashed",
                                "net.ornithemc:calamus-intermediary",
                            ),
                        ),
                    );
                    lib_mut.insert(
                        "url".to_string(),
                        Value::String("https://maven.ornithemc.net/releases".to_string()),
                    );
                }
            }
        }
    }
    Ok(serde_json::to_string_pretty(&text)?)
}

pub async fn fetch_loader_versions()
-> Result<HashMap<LoaderType, Vec<LoaderVersion>>, InstallerError> {
    let mut out = HashMap::new();
    for loader in [LoaderType::Fabric, LoaderType::Quilt] {
        let versions = fetch_loader_versions_type(&loader).await?;
        out.insert(loader, versions);
    }
    Ok(out)
}

async fn fetch_loader_versions_type(
    loader_type: &LoaderType,
) -> Result<Vec<LoaderVersion>, InstallerError> {
    let url = META_URL.to_owned()
        + "/v3/versions/"
        + match loader_type {
            LoaderType::Fabric => "fabric-loader",
            LoaderType::Quilt => "quilt-loader",
        };
    super::CLIENT
        .get(url)
        .send()
        .await?
        .json::<Vec<LoaderVersion>>()
        .await
        .map_err(|e| e.into())
}

#[derive(Deserialize)]
pub struct IntermediaryVersion {
    pub version: String,
    stable: bool,
    maven: String,
    #[serde(rename(deserialize = "versionNoSide"))]
    version_no_side: String,
}

pub async fn fetch_intermediary_versions()
-> Result<HashMap<String, IntermediaryVersion>, InstallerError> {
    let versions = super::CLIENT
        .get(META_URL.to_owned() + "/v3/versions/intermediary")
        .send()
        .await?
        .json::<Vec<IntermediaryVersion>>()
        .await
        .map_err(|e| Into::<InstallerError>::into(e))?;
    let mut out = HashMap::with_capacity(versions.len());
    for ver in versions {
        out.insert(ver.version.clone(), ver);
    }
    Ok(out)
}
