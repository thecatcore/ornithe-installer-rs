use std::{collections::HashMap, path::Path};

use egui::{Button, ComboBox, RichText, Sense, Vec2};
use egui_dropdown::DropDownBox;
use log::{error, info};
use rfd::{AsyncMessageDialog, FileDialog, MessageButtons};
use tokio::task::JoinHandle;

use crate::{
    errors::InstallerError,
    net::{
        self,
        manifest::MinecraftVersion,
        meta::{LoaderType, LoaderVersion},
    },
};

use super::Mode;

pub async fn run() {
    info!("Starting GUI installer...");
    let app = App::create().await;
    match app {
        Ok(app) => match create_window(app).await {
            Ok(_) => {}
            Err(e) => {
                error!("{}", e.0);
                display_dialog("Ornithe Installer Error", &e.0)
            }
        },
        Err(_) => {
            error!("Failed to launch gui!");
        }
    }
}

async fn create_window(app: App) -> Result<(), InstallerError> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([630.0, 490.0]),
        renderer: eframe::Renderer::Wgpu,
        ..Default::default()
    };

    eframe::run_native(
        &("Ornithe Installer ".to_owned() + crate::VERSION),
        options,
        Box::new(|_cc| Ok(Box::new(app))),
    )?;
    Ok(())
}

fn display_dialog(title: &str, message: &str) {
    display_dialog_ext(title, message, MessageButtons::Ok, |_| {});
}

fn display_dialog_ext<F>(title: &str, message: &str, buttons: MessageButtons, after: F)
where
    F: FnOnce(bool) -> (),
    F: Send,
    F: 'static,
{
    info!("Displaying dialog: {}: {}", title, message);
    let dialog = AsyncMessageDialog::new()
        .set_title(title)
        .set_level(rfd::MessageLevel::Info)
        .set_description(&message)
        .set_buttons(buttons);
    tokio::spawn(async move {
        let out = dialog.show().await;
        after(out);
    });
}

struct App {
    mode: Mode,
    selected_minecraft_version: String,
    available_minecraft_versions: Vec<MinecraftVersion>,
    available_intermediary_versions: Vec<String>,
    show_snapshots: bool,
    selected_loader_type: LoaderType,
    selected_loader_version: String,
    available_loader_versions: HashMap<LoaderType, Vec<LoaderVersion>>,
    show_betas: bool,
    create_profile: bool,
    client_install_location: String,
    mmc_output_location: String,
    server_install_location: String,
    copy_generated_location: bool,
    download_minecraft_server: bool,
    installation_task: Option<JoinHandle<Result<(), InstallerError>>>,
}

impl App {
    async fn create() -> Result<App, InstallerError> {
        let mut available_minecraft_versions = Vec::new();
        let mut available_intermediary_versions = Vec::new();
        let mut available_loader_versions = HashMap::new();

        info!("Loading versions...");
        if let Ok(versions) = net::manifest::fetch_versions().await {
            for ele in versions.versions {
                available_minecraft_versions.push(ele);
            }
        }
        if let Ok(versions) = net::meta::fetch_intermediary_versions().await {
            for v in versions.keys() {
                available_intermediary_versions.push(v.clone());
            }
        }
        info!(
            "Loaded {} Minecraft versions",
            available_minecraft_versions.len()
        );

        if let Ok(versions) = net::meta::fetch_loader_versions().await {
            available_loader_versions = versions;
        }

        let app = App {
            mode: Mode::Client,
            selected_minecraft_version: String::new(),
            available_minecraft_versions,
            available_intermediary_versions,
            show_snapshots: false,
            selected_loader_type: LoaderType::Fabric,
            selected_loader_version: available_loader_versions
                .get(&LoaderType::Fabric)
                .map(|v| v.get(0).unwrap().version.clone())
                .unwrap_or(String::new()),
            available_loader_versions,
            show_betas: false,
            create_profile: true,
            client_install_location: super::dot_minecraft_location(),
            mmc_output_location: super::current_location(),
            server_install_location: super::server_location(),
            copy_generated_location: false,
            download_minecraft_server: true,
            installation_task: None,
        };
        Ok(app)
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.set_zoom_factor(1.5);
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("Ornithe Installer");
            });
            ui.vertical(|ui| {
                ui.add_space(15.0);

                ui.label("Environment");
                ui.horizontal(|ui| {
                    ui.radio_value(&mut self.mode, Mode::Client, "Client (Official Launcher)");
                    ui.radio_value(&mut self.mode, Mode::MMC, "MultiMC/PrismLauncher");
                    ui.radio_value(&mut self.mode, Mode::Server, "Server");
                });

                ui.add_space(15.0);
                ui.label("Minecraft Version");
                ui.horizontal(|ui| {
                    ui.add(
                        DropDownBox::from_iter(
                            &self
                                .available_minecraft_versions
                                .iter()
                                .filter(|v| {
                                    self.available_intermediary_versions.contains(&v.id)
                                        || self.available_intermediary_versions.contains(
                                            &(v.id.clone()
                                                + "-"
                                                + match self.mode {
                                                    Mode::Server => "server",
                                                    _ => "client",
                                                }),
                                        )
                                })
                                .filter(|v| self.show_snapshots || v._type == "release")
                                .map(|v| v.id.clone())
                                .collect::<Vec<String>>(),
                            "minecraft_version",
                            &mut self.selected_minecraft_version,
                            |ui, text| ui.selectable_label(false, text),
                        )
                        .max_height(130.0)
                        .hint_text("Search available versions..."),
                    );
                    ui.checkbox(&mut self.show_snapshots, "Show Snapshots")
                });
                ui.add_space(15.0);
                ui.label("Loader");
                ui.horizontal(|ui| {
                    ComboBox::from_id_salt("loader_type")
                        .selected_text(format!(
                            "{} Loader",
                            &self.selected_loader_type.get_localized_name()
                        ))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut self.selected_loader_type,
                                LoaderType::Fabric,
                                "Fabric Loader",
                            );
                            ui.selectable_value(
                                &mut self.selected_loader_type,
                                LoaderType::Quilt,
                                "Quilt Loader",
                            );
                        });

                    ui.label("Version: ");
                    ComboBox::from_id_salt("loader_version")
                        .selected_text(format!("{}", &self.selected_loader_version))
                        .show_ui(ui, |ui| {
                            for ele in self
                                .available_loader_versions
                                .get(&self.selected_loader_type)
                                .unwrap()
                            {
                                if self.show_betas || !ele.version.contains("-") {
                                    ui.selectable_value(
                                        &mut self.selected_loader_version,
                                        ele.version.clone(),
                                        ele.version.clone(),
                                    );
                                }
                            }
                        });
                    let checkbox_response = ui.checkbox(&mut self.show_betas, "Show Betas");
                    if !self
                        .available_loader_versions
                        .get(&self.selected_loader_type)
                        .unwrap()
                        .iter()
                        .find(|v| v.version == self.selected_loader_version)
                        .is_some()
                        || checkbox_response.clicked()
                    {
                        self.selected_loader_version = self
                            .available_loader_versions
                            .get(&self.selected_loader_type)
                            .unwrap()
                            .iter()
                            .map(|v| v.version.clone())
                            .filter(|v| self.show_betas || !v.contains("-"))
                            .next()
                            .unwrap()
                            .clone();
                    }
                });

                ui.add_space(15.0);
                ui.label(match self.mode {
                    Mode::MMC => "Output Location",
                    _ => "Install Location",
                });
                ui.horizontal(|ui| match self.mode {
                    Mode::MMC => {
                        ui.text_edit_singleline(&mut self.mmc_output_location);
                        if ui.button("Pick Location").clicked() {
                            let picked = FileDialog::new()
                                .set_directory(Path::new(&self.mmc_output_location))
                                .pick_folder();
                            if let Some(path) = picked {
                                if let Some(path) = path.to_str() {
                                    self.mmc_output_location = path.to_owned();
                                }
                            }
                        }
                    }
                    Mode::Client => {
                        ui.text_edit_singleline(&mut self.client_install_location);
                        if ui.button("Pick Location").clicked() {
                            let picked = FileDialog::new()
                                .set_directory(Path::new(&self.client_install_location))
                                .pick_folder();
                            if let Some(path) = picked {
                                if let Some(path) = path.to_str() {
                                    self.client_install_location = path.to_owned();
                                }
                            }
                        }
                    }
                    Mode::Server => {
                        ui.text_edit_singleline(&mut self.server_install_location);
                        if ui.button("Pick Location").clicked() {
                            let picked = FileDialog::new()
                                .set_directory(Path::new(&self.server_install_location))
                                .pick_folder();
                            if let Some(path) = picked {
                                if let Some(path) = path.to_str() {
                                    self.server_install_location = path.to_owned();
                                }
                            }
                        }
                    }
                });
            });
            ui.vertical_centered(|ui| {
                ui.add_space(15.0);
                match self.mode {
                    Mode::Client => {
                        ui.checkbox(&mut self.create_profile, "Generate Profile");
                    }
                    Mode::Server => {
                        ui.checkbox(
                            &mut self.download_minecraft_server,
                            "Download Minecraft Server",
                        );
                    }
                    Mode::MMC => {
                        ui.checkbox(
                            &mut self.copy_generated_location,
                            "Copy Profile Path to Clipboard",
                        );
                    }
                }
            });
            ui.add_space(15.0);
            ui.vertical_centered(|ui| {
                let mut install_button =
                    Button::new(RichText::new("Install").heading()).min_size(Vec2::new(100.0, 0.0));
                if self.installation_task.is_some() {
                    install_button = install_button.sense(Sense::empty());
                }
                if ui.add(install_button).clicked() {
                    if let Some(version) = self
                        .available_minecraft_versions
                        .iter()
                        .find(|v| v.id == self.selected_minecraft_version)
                    {
                        let selected_version = version.clone();
                        let loader_version = self
                            .available_loader_versions
                            .get(&self.selected_loader_type)
                            .unwrap()
                            .iter()
                            .find(|v| v.version == self.selected_loader_version)
                            .unwrap()
                            .clone();
                        match self.mode {
                            Mode::Client => {
                                let loader_type = self.selected_loader_type.clone();
                                let location =
                                    Path::new(&self.client_install_location).to_path_buf();
                                let create_profile = self.create_profile;
                                let handle = tokio::spawn(async move {
                                    crate::actions::client::install(
                                        selected_version,
                                        loader_type,
                                        loader_version,
                                        location,
                                        create_profile,
                                    )
                                    .await
                                });
                                self.installation_task = Some(handle);
                            }
                            Mode::Server => {
                                let loader_type = self.selected_loader_type.clone();
                                let location =
                                    Path::new(&self.server_install_location).to_path_buf();
                                let download_server = self.download_minecraft_server;
                                self.installation_task = Some(tokio::spawn(async move {
                                    crate::actions::server::install(
                                        selected_version,
                                        loader_type,
                                        loader_version,
                                        location,
                                        download_server,
                                    )
                                    .await
                                }));
                            }
                            Mode::MMC => {
                                let loader_type = self.selected_loader_type.clone();
                                let location = Path::new(&self.mmc_output_location).to_path_buf();
                                let copy_profile_path = self.copy_generated_location;
                                let handle = tokio::spawn(async move {
                                    crate::actions::mmc_pack::install(
                                        selected_version,
                                        loader_type,
                                        loader_version,
                                        location,
                                        copy_profile_path,
                                    )
                                    .await
                                });
                                self.installation_task = Some(handle);
                            }
                        }
                    } else {
                        display_dialog(
                            "Installation Failed",
                            "No supported Minecraft version is selected",
                        );
                    }
                }
            });
        });

        if let Some(task) = &self.installation_task {
            if task.is_finished() {
                let handle = self.installation_task.take().unwrap();
                tokio::spawn(async move {
                    match handle.await.unwrap() {
                        Err(e) => {
                            error!("{}", e.0);
                            display_dialog(
                                "Installation Failed",
                                &("Failed to install: ".to_owned() + &e.0),
                            )
                        }
                        Ok(_) => display_dialog_ext(
                            "Installation Successful",
                            "Ornithe has been successfully installed.\nMost mods require that you also download the Ornithe Standard Libraries mod and place it in your mods folder.\nWould you like to open OSL's modrinth page now?",
                            MessageButtons::YesNo,
                            |res| {
                                if res {
                                    if webbrowser::open(crate::OSL_MODRINTH_URL).is_err() {
                                        display_dialog("Failed to open modrinth", &("Failed to open modrinth page for Ornithe Standard Libraries.\nYou can find it at ".to_owned()+crate::OSL_MODRINTH_URL).as_str());
                                    }
                                }
                            },
                        ),
                    }
                });
            }
        }
    }
}
