#![cfg_attr(feature = "winapp", windows_subsystem = "windows")]

use clap::Parser;
use monitor_input::{Cli, Monitor};

fn main() -> anyhow::Result<()> {
    let mut cli: Cli = Cli::parse();
    cli.monitors = Monitor::enumerate();
    cli.run()
}
