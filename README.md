# Ornithe Installer

Installer for [Ornithe](https://ornithemc.net) offering a gui
and cli to install a profile for the official launcher, generate
an instance for MultiMC/PrismLauncher and install/bootstrap a server.

### Usage

If no command-line arguments are specified, the GUI will be opened.
Should it not be possible to open the GUI, the CLI help message will be
printed.

The help message can also be accessed using the `--help` flag or the `help` subcommand.

The CLI supports a few options that are not present
in the GUI:

- Installing & running a server in a single step
  - passing arguments to the server
  - specifying a java binary to use to run the server

  
### Building

Requirements: a recent rust toolchain

To build:

`$ cargo build --release`

Binaries are then output in the `target/release` directory.

### License

This installer is licensed under the Apache-2.0 license.
For more details see the included LICENSE file.
