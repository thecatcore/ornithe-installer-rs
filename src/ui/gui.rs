use std::sync::Arc;

use egui::{ComboBox, mutex::RwLock};
use reqwest::Error;
use tokio::sync::Mutex;

use crate::{
    errors::InstallerError,
    net::{self, manifest::MinecraftVersion},
};

pub async fn run() -> anyhow::Result<()> {
    match create_window().await {
        Ok(_) => {}
        Err(e) => {
            egui_dialogs::Dialogs::new().error("Ornithe Installer Error", e.0);
        }
    }

    Ok(())
}

async fn create_window() -> Result<(), InstallerError> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([800.0, 490.0]),
        renderer: eframe::Renderer::Wgpu,
        ..Default::default()
    };
    let app = App::create().await?;
    eframe::run_native(
        "Ornithe Installer",
        options,
        Box::new(|_cc| Ok(Box::new(app))),
    )?;
    Ok(())
}

struct App {
    mode: Mode,
    selected_minecraft_version: String,
    available_minecraft_versions: Vec<MinecraftVersion>,
}

impl App {
    async fn create() -> Result<App, Error> {
        let app = App {
            mode: Mode::Client,
            selected_minecraft_version: String::new(),
            available_minecraft_versions: Vec::new(),
        };
        tokio::spawn(async move {
            for ele in net::manifest::fetch_versions().await.1 {
                app.available_minecraft_versions.push(ele);
            }
        });
        Ok(app)
    }
}

#[derive(PartialEq)]
enum Mode {
    Client,
    Server,
    MMC,
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        ctx.set_zoom_factor(1.5);
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered_justified(|ui| {
                ui.heading("Ornithe Installer");

                ui.vertical(|ui| {
                    ui.label("Environment");
                    ui.horizontal(|ui| {
                        ui.radio_value(&mut self.mode, Mode::Client, "Client (Official Launcher)");
                        ui.radio_value(&mut self.mode, Mode::MMC, "MultiMC/PrismLauncher");
                        ui.radio_value(&mut self.mode, Mode::Server, "Server");
                    });

                    ui.spacing();
                    ComboBox::from_label("Minecraft version")
                        .selected_text(format!("{:?}", self.selected_minecraft_version))
                        .show_ui(ui, |ui| {});
                });
            });
        });
    }
}
