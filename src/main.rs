use clap::Parser;
use monitor_input::{Cli, Monitor};

fn main() -> anyhow::Result<()> {
    let mut cli: Cli = Cli::parse();
    cli.init_logger();
    cli.monitors = Monitor::enumerate();
    cli.run()
}
