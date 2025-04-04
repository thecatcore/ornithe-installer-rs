mod actions;
mod errors;
mod net;
mod ui;

static VERSION: &str = env!("CARGO_PKG_VERSION");
static USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    if std::env::args().count() > 0 {
        crate::ui::cli::run();
    }

    crate::ui::gui::run();
    Ok(())
}
