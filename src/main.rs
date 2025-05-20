use std::env;
use std::io::Write;

use clap::Parser;

use monitor_input::{Cli, Monitor};

fn main() -> anyhow::Result<()> {
    let mut cli: Cli = Cli::parse();
    init_logger(cli.verbose);
    cli.monitors = Monitor::enumerate();
    cli.run()
}

fn init_logger(verbose: u8) {
    // If `RUST_LOG` is set, initialize the `env_logger` in its default config.
    if env::var("RUST_LOG").is_ok() {
        env_logger::init();
        return;
    }

    // Otherwise setup according to the `verbose` level, in a simpler format.
    env_logger::Builder::new()
        .filter_level(match verbose {
            0 => log::LevelFilter::Info,
            1 => log::LevelFilter::Debug,
            _ => log::LevelFilter::Trace,
        })
        .format(|buf, record| match record.level() {
            log::Level::Info => writeln!(buf, "{}", record.args()),
            _ => {
                let style = buf.default_level_style(record.level());
                writeln!(buf, "{style}{}{style:#}: {}", record.level(), record.args())
            }
        })
        .init();
}
