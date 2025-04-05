use std::path::PathBuf;

use log::info;

use crate::{
    errors::InstallerError,
    net::{
        manifest::MinecraftVersion,
        meta::{LoaderType, LoaderVersion},
    },
};

pub async fn install(
    version: MinecraftVersion,
    loader_type: LoaderType,
    loader_version: LoaderVersion,
    location: PathBuf,
) -> Result<(), InstallerError> {
    let _ = install_path(&version, &loader_type, &loader_version, &location).await?;

    info!(
        "Installed Ornithe Server for Minecraft {} using {} Loader {} to {}",
        &version.id,
        &loader_type.get_localized_name(),
        &loader_version.version,
        &location.to_str().unwrap_or_default()
    );

    Ok(())
}

async fn install_path(
    version: &MinecraftVersion,
    loader_type: &LoaderType,
    loader_version: &LoaderVersion,
    location: &PathBuf,
) -> Result<PathBuf, InstallerError> {
    todo!()
}

pub async fn install_and_run(
    version: MinecraftVersion,
    loader_type: LoaderType,
    loader_version: LoaderVersion,
    location: PathBuf,
) -> Result<(), InstallerError> {
    let installed = install_path(&version, &loader_type, &loader_version, &location).await?;

    todo!()
}
