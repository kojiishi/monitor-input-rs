#![cfg_attr(feature = "winapp", windows_subsystem = "windows")]

use std::fmt;

use clap::Parser;
use toast_logger_win::{Notification, ToastLogger};

use monitor_input::{Cli, Monitor};

fn main() -> anyhow::Result<()> {
    let mut cli: Cli = Cli::parse();
    init_logger(cli.verbose);
    cli.monitors = Monitor::enumerate();
    cli.run()?;
    ToastLogger::flush()?;
    Ok(())
}

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
