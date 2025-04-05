use clap::{arg, command, value_parser};

use super::Mode;

pub fn run() {
    let matches = command!()
        .arg(
            arg!(-m --mode <MODE> "Installer mode (client/server/mmc)")
                .default_value("server")
                .value_parser(value_parser!(String)),
        )
        .arg(
            arg!(--install_dir <DIR> "Installation directory")
                .default_value(super::current_location()),
        )
        .get_matches();
}
