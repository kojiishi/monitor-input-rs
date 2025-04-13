use monitor_input::{Cli,Monitor};
use clap::Parser;

fn main() -> anyhow::Result<()> {
    let mut cli: Cli = Cli::parse();
    cli.init_logger();
    Monitor::set_dry_run(cli.dry_run);
    cli.monitors = Monitor::enumerate();
    cli.apply_filters()?;
    cli.run()
}
