use std::{io::Write, path::PathBuf};

use clap::{ArgMatches, Command, arg, command, value_parser};

use crate::{
    errors::InstallerError,
    net::{
        manifest::MinecraftVersion,
        meta::{LoaderType, LoaderVersion},
    },
};

pub async fn run() {
    let matches = command!()
        .name("Ornithe Installer")
        .subcommand(
            add_arguments(Command::new("client")
                .about("Client installation for the official launcher")
                .long_flag("client")
                .arg(
                    arg!(-d --dir <DIR> "Installation directory")
                        .default_value(super::dot_minecraft_location())
                        .value_parser(value_parser!(PathBuf)),
                )
                .arg(
                    arg!(-p --"generate-profile" <VALUE> "Whether to generate a launch profile")
                    .default_value("true")
                        .value_parser(value_parser!(bool)),
                )),
        )
        .subcommand(
            add_arguments(Command::new("mmc")
                .visible_alias("prism")
                .long_flag("mmc")
                .visible_long_flag_alias("prism")
                .about("Generate an instance for MultiMC/PrismLauncher")
                .arg(
                    arg!(-d --dir <DIR> "Output directory")
                        .default_value(super::current_location())
                        .value_parser(value_parser!(PathBuf)),
                )
                .arg(arg!(-c --"copy-profile-path" <VALUE> "Whether to copy the path of the generated profile to the clipboard")
                .default_value("false")
            .value_parser(value_parser!(bool)))),
        )
        .subcommand(
            add_arguments(Command::new("server")
                .about("Server installation")
                .long_flag("server")
                .arg(
                    arg!(-d --dir <DIR> "Installation directory")
                        .default_value(super::server_location())
                        .value_parser(value_parser!(PathBuf)),
                )
                .arg(arg!(--"download-minecraft" "Whether to download the minecraft server jar"))
                .subcommand(Command::new("run").about("Install and run the server")
                .arg(arg!(--args <ARGS> "Whether to also run the installed server, with the provided arguments")
            .default_value(""))
                .arg(arg!(--java <PATH> "The java binary to use to run the server").value_parser(value_parser!(PathBuf)))),
        ))
        .subcommand(
            Command::new("game-versions")
            .alias("minecraft-versions")
            .long_flag("list-game-versions")
            .long_flag_alias("list-minecraft-versions")
                .about("List supported game versions")
                .arg(arg!(-s --"show-snapshots" "Include snapshot versions")),
        )
        .subcommand(
            Command::new("loader-versions")
            .long_flag("list-loader-versions")
                .about("List available loader versions")
                .arg(arg!(-b --"show-betas" "Include beta versions"))
                .arg(arg!(--"loader-type" <TYPE> "Loader type to use")
                .default_value("fabric")
                .ignore_case(true)
                .value_parser(["fabric", "quilt"])),
        )
        .get_matches();

    match parse(matches).await {
        Ok(_) => {}
        Err(e) => {
            std::io::stderr()
                .write_all(("Failed to load Ornithe Installer CLI: ".to_owned() + &e.0).as_bytes())
                .expect("Failed to print error!");
        }
    }
}

async fn parse(matches: ArgMatches) -> Result<(), InstallerError> {
    if let Some(matches) = matches.subcommand_matches("loader-versions") {
        let versions = crate::net::meta::fetch_loader_versions().await?;
        let loader_type = get_loader_type(matches)?;
        let betas = matches.get_flag("show-betas");

        let mut out = String::new();
        for version in versions.get(&loader_type).unwrap() {
            if betas || !version.version.contains("-") {
                out += &(version.version.clone() + " ");
            }
        }
        writeln!(
            std::io::stdout(),
            "Latest {} Loader version: {}",
            loader_type.get_localized_name(),
            versions
                .get(&loader_type)
                .and_then(|list| list.get(0))
                .map(|v| v.version.clone())
                .unwrap_or("<not available>".to_owned())
        )?;
        writeln!(
            std::io::stdout(),
            "Available {} Loader versions:",
            loader_type.get_localized_name()
        )?;
        writeln!(std::io::stdout(), "{}", out)?;

        return Ok(());
    }

    let minecraft_versions = crate::net::manifest::fetch_versions().await?;
    let intermediary_versions = crate::net::meta::fetch_intermediary_versions().await?;

    let mut available_minecraft_versions = Vec::new();

    for version in minecraft_versions.versions {
        if intermediary_versions.contains_key(&version.id)
            || intermediary_versions.contains_key(&(version.id.clone() + "-client"))
            || intermediary_versions.contains_key(&(version.id.clone() + "-server"))
        {
            available_minecraft_versions.push(version);
        }
    }

    if let Some(matches) = matches.subcommand_matches("game-versions") {
        let mut out = String::new();
        let snapshots = matches.get_flag("show-snapshots");
        for version in available_minecraft_versions {
            if snapshots || version._type == "release" {
                out += &(version.id.clone() + " ");
            }
        }
        writeln!(std::io::stdout(), "Available Minecraft versions:\n")?;
        writeln!(std::io::stdout(), "{}", out)?;
        return Ok(());
    }

    let loader_versions = crate::net::meta::fetch_loader_versions().await?;

    if let Some(matches) = matches.subcommand_matches("client") {
        let minecraft_version = get_minecraft_version(matches, available_minecraft_versions)?;
        let loader_type = get_loader_type(matches)?;
        let loader_versions = loader_versions.get(&loader_type).unwrap();
        let loader_version = get_loader_version(matches, loader_versions)?;
        let location = matches.get_one::<PathBuf>("dir").unwrap().clone();
        let create_profile = matches.get_flag("generate-profile");
        return crate::actions::client::install(
            minecraft_version,
            loader_type,
            loader_version,
            location,
            create_profile,
        )
        .await;
    }

    if let Some(matches) = matches.subcommand_matches("server") {
        let minecraft_version = get_minecraft_version(matches, available_minecraft_versions)?;
        let loader_type = get_loader_type(matches)?;
        let loader_versions = loader_versions.get(&loader_type).unwrap();
        let loader_version = get_loader_version(matches, loader_versions)?;
        let location = matches.get_one::<PathBuf>("dir").unwrap().clone();
        if let Some(matches) = matches.subcommand_matches("run") {
            let java = matches.get_one::<PathBuf>("java");
            let run_args = matches.get_one::<String>("args").unwrap();
            return crate::actions::server::install_and_run(
                minecraft_version,
                loader_type,
                loader_version,
                location,
                java,
                run_args.split(" "),
            )
            .await;
        }
        return crate::actions::server::install(
            minecraft_version,
            loader_type,
            loader_version,
            location,
            matches.get_flag("download-minecraft"),
        )
        .await;
    }

    if let Some(matches) = matches.subcommand_matches("mmc") {
        let minecraft_version = get_minecraft_version(matches, available_minecraft_versions)?;
        let loader_type = get_loader_type(matches)?;
        let loader_versions = loader_versions.get(&loader_type).unwrap();
        let loader_version = get_loader_version(matches, loader_versions)?;
        let output_dir = matches.get_one::<PathBuf>("dir").unwrap().clone();
        let copy_profile_path = matches.get_flag("copy-profile-path");
        return crate::actions::mmc_pack::install(
            minecraft_version,
            loader_type,
            loader_version,
            output_dir,
            copy_profile_path,
        )
        .await;
    }

    Ok(())
}

fn get_minecraft_version(
    matches: &ArgMatches,
    versions: Vec<MinecraftVersion>,
) -> Result<MinecraftVersion, InstallerError> {
    let minecraft_version_arg = matches.get_one::<String>("minecraft-version").unwrap();

    for version in versions {
        if version.id == *minecraft_version_arg {
            return Ok(version);
        }
    }
    Err(InstallerError(
        "Could not find Minecraft version ".to_owned()
            + minecraft_version_arg
            + " among supported versions!",
    ))
}

fn get_loader_type(matches: &ArgMatches) -> Result<LoaderType, InstallerError> {
    Ok(
        match matches.get_one::<String>("loader-type").unwrap().as_str() {
            "quilt" => crate::net::meta::LoaderType::Quilt,
            "fabric" => crate::net::meta::LoaderType::Fabric,
            &_ => {
                return Err(InstallerError("Unsupported loader type!".to_owned()));
            }
        },
    )
}

fn get_loader_version(
    matches: &ArgMatches,
    versions: &Vec<LoaderVersion>,
) -> Result<LoaderVersion, InstallerError> {
    let arg = matches.get_one::<String>("loader-version").unwrap();

    if *arg == "latest" {
        return versions.get(0).map(|v| v.clone()).ok_or(InstallerError(
            "Failed to find loader version in list".to_owned(),
        ));
    }

    for version in versions {
        if version.version == *arg {
            return Ok(version.clone());
        }
    }

    Err(InstallerError(
        "Could not find loader version: ".to_owned() + arg,
    ))
}

fn add_arguments(command: Command) -> Command {
    command
        .arg(arg!(-m --"minecraft-version" <VERSION> "Minecraft version to use"))
        .arg(
            arg!(--"loader-type" <TYPE> "Loader type to use")
                .default_value("fabric")
                .ignore_case(true)
                .value_parser(["fabric", "quilt"]),
        )
        .arg(arg!(--"loader-version" <VERSION> "Loader version to use").default_value("latest"))
}
