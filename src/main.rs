use env_logger::Env;

mod actions;
mod errors;
mod net;
mod ui;

static VERSION: &str = env!("CARGO_PKG_VERSION");
static USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);
static ORNITHE_ICON_BYTES: &[u8] = include_bytes!("../res/icon.png");

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init_from_env(Env::default().default_filter_or("ornithe_installer_rs=info"));
    if std::env::args().count() > 0 {
        crate::ui::cli::run();
    }

    crate::ui::gui::run().await?;
    Ok(())
}
