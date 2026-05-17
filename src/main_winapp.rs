#![cfg_attr(
    all(feature = "winapp", target_os = "windows"),
    windows_subsystem = "windows"
)]

#[cfg(all(feature = "winapp", target_os = "windows"))]
use std::fmt;

#[cfg(all(feature = "winapp", target_os = "windows"))]
use clap::Parser;
#[cfg(all(feature = "winapp", target_os = "windows"))]
use toast_logger_win::{Notification, ToastLogger};

#[cfg(all(feature = "winapp", target_os = "windows"))]
use monitor_input::{Cli, Monitor};

#[cfg(all(feature = "winapp", target_os = "windows"))]
fn main() -> anyhow::Result<()> {
    let mut cli: Cli = Cli::parse();
    init_logger(cli.verbose);
    cli.monitors = Monitor::enumerate();
    cli.run()?;
    ToastLogger::flush()?;
    Ok(())
}

#[cfg(all(feature = "winapp", target_os = "windows"))]
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

#[cfg(not(all(feature = "winapp", target_os = "windows")))]
include!("main.rs");
