use serde_json::Value;

use crate::errors::InstallerError;

use super::{GameSide, manifest::MinecraftVersion};

const META_URL: &str = "https://meta.ornithemc.net";

pub struct LoaderVersion {
    pub version: String,
}

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
            side.launch_json_endpoint()
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
