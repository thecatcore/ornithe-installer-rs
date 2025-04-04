use clap::Parser;

#[derive(Parser)]
pub struct Args {
    version: Option<String>,
}

pub fn run() {
    let args = Args::parse();
}
