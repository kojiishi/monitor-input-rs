#![cfg_attr(all(windows, feature = "winapp"), windows_subsystem = "windows")]

#[cfg(all(windows, feature = "winapp"))]
use std::fmt;
#[cfg(not(all(windows, feature = "winapp")))]
use std::{env, io::Write};

use clap::Parser;
#[cfg(all(windows, feature = "winapp"))]
use toast_logger_win::{Notification, ToastLogger};

use monitor_input::{Cli, Monitor};

fn main() -> anyhow::Result<()> {
    let mut cli: Cli = Cli::parse();
    init_logger(cli.verbose);
    cli.monitors = Monitor::enumerate();
    cli.run()?;
    #[cfg(all(windows, feature = "winapp"))]
    ToastLogger::flush()?;
    Ok(())
}

#[cfg(not(all(windows, feature = "winapp")))]
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

#[cfg(all(windows, feature = "winapp"))]
fn init_logger(verbose: u8) {
    ToastLogger::builder()
        .auto_flush(false)
        .max_level(match verbose {
            0 => log::LevelFilter::Error,
            1 => log::LevelFilter::Info,
            2 => log::LevelFilter::Debug,
            _ => log::LevelFilter::Trace,
        })
        .format(
            |buf: &mut dyn fmt::Write, record: &log::Record| match record.level() {
                log::Level::Info => write!(buf, "{}", record.args()),
                _ => write!(buf, "{}: {}", record.level(), record.args()),
            },
        )
        .create_notification(|records| {
            let mut notification = Notification::new_with_records(records)?;
            let min_level = records.iter().map(|r| r.level).min().unwrap();
            if min_level >= log::Level::Info {
                notification.expires_in(std::time::Duration::from_secs(10))?;
            }
            Ok(notification)
        })
        .init()
        .unwrap();
}
