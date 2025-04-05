use std::str::FromStr;

use anyhow::Context;
use clap::Parser;
use ddc_hi::{Ddc, DdcHost, FeatureCode};
use log::*;
use regex::Regex;
use strum_macros::{AsRefStr, EnumString, FromRepr};

type InputSourceRaw = u8;

#[derive(Debug, PartialEq, AsRefStr, EnumString, FromRepr)]
#[repr(u8)]
#[strum(ascii_case_insensitive)]
enum InputSource {
    #[strum(serialize = "DP1")]
    DisplayPort1 = 0x0F,
    #[strum(serialize = "DP2")]
    DisplayPort2 = 0x10,
    Hdmi1 = 0x11,
    Hdmi2 = 0x12,
    UsbC1 = 0x19,
    UsbC2 = 0x1B,
}

impl InputSource {
    fn raw_from_str(input: &str) -> anyhow::Result<InputSourceRaw> {
        if let Ok(value) = input.parse::<InputSourceRaw>() {
            return Ok(value);
        }
        InputSource::from_str(input)
            .map(|value| value as InputSourceRaw)
            .with_context(|| format!("\"{input}\" is not a valid input source"))
    }

    fn str_from_raw(value: InputSourceRaw) -> String {
        match InputSource::from_repr(value) {
            Some(input_source) => input_source.as_ref().to_string(),
            None => value.to_string(),
        }
    }
}

/// VCP feature code for input select
const INPUT_SELECT: FeatureCode = 0x60;

static mut DRY_RUN: bool = false;

struct Monitor {
    ddc_hi_display: ddc_hi::Display,
    is_capabilities_updated: bool,
    needs_sleep: bool,
}

impl std::fmt::Display for Monitor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.ddc_hi_display.info.id)
    }
}

impl std::fmt::Debug for Monitor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.ddc_hi_display.info)
    }
}

impl Monitor {
    fn new(ddc_hi_display: ddc_hi::Display) -> Self {
        Monitor {
            ddc_hi_display: ddc_hi_display,
            is_capabilities_updated: false,
            needs_sleep: false,
        }
    }

    fn enumerate() -> Vec<Self> {
        ddc_hi::Display::enumerate()
            .into_iter()
            .map(|d| Monitor::new(d))
            .collect()
    }

    fn is_dry_run() -> bool {
        unsafe { return DRY_RUN }
    }

    fn set_dry_run(value: bool) {
        unsafe { DRY_RUN = value }
    }

    fn update_capabilities(&mut self) -> anyhow::Result<()> {
        debug!("update_capabilities: {}", self);
        self.is_capabilities_updated = true;
        self.ddc_hi_display.update_capabilities()
    }

    fn ensure_capabilities(&mut self) -> anyhow::Result<()> {
        if self.is_capabilities_updated {
            return Ok(());
        }
        self.update_capabilities()
    }

    fn ensure_capabilities_as_warn(&mut self) {
        if let Err(e) = self.ensure_capabilities() {
            warn!("{}: Failed to update capabilities: {}", self, e);
        }
    }

    fn contains(&self, name: &str) -> bool {
        self.ddc_hi_display.info.id.contains(name)
    }

    fn feature_code(&self, feature_code: FeatureCode) -> FeatureCode {
        // TODO: `mccs_database` is initialized by `display.update_capabilities()`
        // which is quite slow, and it seems to work without this.
        // See also https://github.com/mjkoo/monitor-switch/blob/master/src/main.rs.
        if let Some(feature) = self.ddc_hi_display.info.mccs_database.get(feature_code) {
            return feature.code;
        }
        feature_code
    }

    fn get_current_input_source(&mut self) -> anyhow::Result<u8> {
        let feature_code: FeatureCode = self.feature_code(INPUT_SELECT);
        Ok(self.ddc_hi_display.handle.get_vcp_feature(feature_code)?.sl)
    }

    fn set_current_input_source(&mut self, value: InputSourceRaw) -> anyhow::Result<()> {
        if Self::is_dry_run() {
            info!(
                "{}.InputSource = {} (dry-run)",
                self,
                InputSource::str_from_raw(value)
            );
            return Ok(());
        }
        info!(
            "{}.InputSource = {}",
            self,
            InputSource::str_from_raw(value)
        );
        let feature_code: FeatureCode = self.feature_code(INPUT_SELECT);
        self.ddc_hi_display
            .handle
            .set_vcp_feature(feature_code, value as u16)
            .inspect(|_| self.needs_sleep = true)
    }

    fn sleep_if_needed(&mut self) {
        if self.needs_sleep {
            debug!("{}.sleep()", self);
            self.needs_sleep = false;
            self.ddc_hi_display.handle.sleep();
            debug!("{}.sleep() done", self);
        }
    }

    fn to_long_string(&mut self) -> String {
        let mut lines = Vec::new();
        lines.push(self.to_string());
        let input_source = self.get_current_input_source();
        lines.push(format!(
            "Input Source: {}",
            match input_source {
                Ok(value) => InputSource::str_from_raw(value as InputSourceRaw),
                Err(e) => e.to_string(),
            }
        ));
        if let Some(model) = &self.ddc_hi_display.info.model_name {
            lines.push(format!("Model: {}", model));
        }
        return lines.join("\n    ");
    }
}

#[derive(Debug, Default, Parser)]
#[command(version, about)]
/// A command line tool to change display monitors' input sources via DDC/CI.
///
/// See https://github.com/kojiishi/monitor-input-rs for more details.
struct Cli {
    #[arg(skip)]
    monitors: Vec<Monitor>,

    #[arg(id = "capabilities", short, long)]
    /// Get capabilities from the display monitors.
    needs_capabilities: bool,

    #[arg(short = 'n', long)]
    /// Dry-run (prevent actual changes).
    dry_run: bool,

    #[arg(short, long)]
    /// Show verbose information.
    verbose: bool,

    #[arg(skip)]
    set_index: Option<usize>,

    /// `name` to search,
    /// `name=input` to change the input source,
    /// or `name=input1,input2` to toggle.
    args: Vec<String>,
}

impl Cli {
    fn init_logger(&self) {
        simplelog::CombinedLogger::init(vec![simplelog::TermLogger::new(
            if self.verbose {
                simplelog::LevelFilter::Debug
            } else {
                simplelog::LevelFilter::Info
            },
            simplelog::Config::default(),
            simplelog::TerminalMode::Mixed,
            simplelog::ColorChoice::Auto,
        )])
        .unwrap();
    }

    fn for_each<C>(&mut self, name: &str, mut callback: C) -> anyhow::Result<()>
    where
        C: FnMut(usize, &mut Monitor) -> anyhow::Result<()>,
    {
        if let Ok(index) = name.parse::<usize>() {
            return callback(index, &mut self.monitors[index]);
        }

        let mut has_match = false;
        for (index, monitor) in (&mut self.monitors).into_iter().enumerate() {
            if self.needs_capabilities {
                monitor.ensure_capabilities_as_warn();
            }
            if name.len() > 0 && !monitor.contains(name) {
                continue;
            }
            has_match = true;
            callback(index, monitor)?;
        }
        if has_match {
            return Ok(());
        }

        anyhow::bail!("No display monitors found for \"{}\".", name);
    }

    fn compute_toggle_set_index(
        current_input_source: InputSourceRaw,
        input_sources: &[InputSourceRaw],
    ) -> usize {
        input_sources
            .iter()
            .position(|v| *v == current_input_source)
            // Toggle to the next index, or 0 if it's not in the list.
            .map_or(0, |i| i + 1)
    }

    fn toggle(&mut self, name: &str, values: &[&str]) -> anyhow::Result<()> {
        let mut input_sources: Vec<InputSourceRaw> = vec![];
        for value in values {
            input_sources.push(InputSource::raw_from_str(value)?);
        }
        let mut set_index = self.set_index;
        let result = self.for_each(name, |_, monitor: &mut Monitor| {
            if set_index.is_none() {
                let current_input_source = monitor.get_current_input_source()?;
                set_index = Some(Self::compute_toggle_set_index(
                    current_input_source,
                    &input_sources,
                ));
                debug!(
                    "Set = {} (because {monitor}.InputSource is {})",
                    set_index.unwrap(),
                    InputSource::str_from_raw(current_input_source)
                );
            }
            let used_index = set_index.unwrap().min(input_sources.len() - 1);
            let input_source = input_sources[used_index];
            monitor.set_current_input_source(input_source)
        });
        self.set_index = set_index;
        result
    }

    fn set(&mut self, name: &str, value: &str) -> anyhow::Result<()> {
        let toggle_values: Vec<&str> = value.split(',').collect();
        if toggle_values.len() > 1 {
            return self.toggle(name, &toggle_values);
        }
        let input_source = InputSource::raw_from_str(value)?;
        self.for_each(name, |_, monitor: &mut Monitor| {
            monitor.set_current_input_source(input_source)
        })
    }

    fn print_list(&mut self, name: &str) -> anyhow::Result<()> {
        let needs_capabilities = self.needs_capabilities;
        self.for_each(name, |index, monitor| {
            if needs_capabilities {
                monitor.ensure_capabilities_as_warn();
            }
            println!("{index}: {}", monitor.to_long_string());
            debug!("{:?}", monitor);
            Ok(())
        })
    }

    fn sleep_if_needed(&mut self) {
        for monitor in &mut self.monitors {
            monitor.sleep_if_needed();
        }
        debug!("All sleep() done");
    }

    const RE_SET_PATTERN: &str = r"^([^=]+)=(.+)$";

    fn run(&mut self) -> anyhow::Result<()> {
        let re_set = Regex::new(Self::RE_SET_PATTERN).unwrap();
        let mut has_valid_args = false;
        let args = self.args.clone();
        for arg in args {
            if let Some(captures) = re_set.captures(&arg) {
                self.set(&captures[1], &captures[2])?;
                has_valid_args = true;
                continue;
            }

            self.print_list(&arg)?;
            has_valid_args = true;
        }
        if !has_valid_args {
            self.print_list("")?;
        }
        self.sleep_if_needed();
        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    let mut cli: Cli = Cli::parse();
    cli.init_logger();
    Monitor::set_dry_run(cli.dry_run);
    cli.monitors = Monitor::enumerate();
    cli.run()
}

#[cfg(test)]
mod tests {
    use std::vec;

    use super::*;

    #[test]
    fn input_source_from_str() {
        assert_eq!(InputSource::from_str("Hdmi1"), Ok(InputSource::Hdmi1));
        // Test `ascii_case_insensitive`.
        assert_eq!(InputSource::from_str("hdmi1"), Ok(InputSource::Hdmi1));
        assert_eq!(InputSource::from_str("HDMI1"), Ok(InputSource::Hdmi1));
        // Test `serialize`.
        assert_eq!(InputSource::from_str("DP1"), Ok(InputSource::DisplayPort1));
        assert_eq!(InputSource::from_str("dp2"), Ok(InputSource::DisplayPort2));
        // Test failures.
        assert!(InputSource::from_str("xyz").is_err());
    }

    #[test]
    fn input_source_raw_from_str() {
        assert_eq!(InputSource::raw_from_str("27").unwrap(), 27);
        // Upper-compatible with `from_str`.
        assert_eq!(
            InputSource::raw_from_str("Hdmi1").unwrap(),
            InputSource::Hdmi1 as InputSourceRaw
        );
        // Test failures.
        assert!(InputSource::raw_from_str("xyz").is_err());
        assert!(
            InputSource::raw_from_str("xyz")
                .unwrap_err()
                .to_string()
                .contains("xyz")
        );
    }

    #[test]
    fn cli_parse() {
        let mut cli = Cli::parse_from([""]);
        assert!(!cli.verbose);
        assert_eq!(cli.args.len(), 0);

        cli = Cli::parse_from(["", "abc", "def"]);
        assert!(!cli.verbose);
        assert_eq!(cli.args, ["abc", "def"]);

        cli = Cli::parse_from(["", "-v", "abc", "def"]);
        assert!(cli.verbose);
        assert_eq!(cli.args, ["abc", "def"]);
    }

    #[test]
    fn cli_parse_option_after_positional() {
        let cli = Cli::parse_from(["", "abc", "def", "-v"]);
        assert!(cli.verbose);
        assert_eq!(cli.args, ["abc", "def"]);
    }

    #[test]
    fn cli_parse_positional_with_hyphen() {
        let cli = Cli::parse_from(["", "--", "-abc", "-def"]);
        assert_eq!(cli.args, ["-abc", "-def"]);
    }

    fn matches<'a>(re: &'a Regex, input: &'a str) -> Vec<&'a str> {
        re.captures(input)
            .unwrap()
            .iter()
            .skip(1)
            .map(|m| m.unwrap().as_str())
            .collect()
    }

    #[test]
    fn re_set() {
        let re_set = Regex::new(Cli::RE_SET_PATTERN).unwrap();
        assert_eq!(re_set.is_match("a"), false);
        assert_eq!(re_set.is_match("a="), false);
        assert_eq!(re_set.is_match("=a"), false);
        assert_eq!(matches(&re_set, "a=b"), vec!["a", "b"]);
        assert_eq!(matches(&re_set, "1=23"), vec!["1", "23"]);
        assert_eq!(matches(&re_set, "12=34"), vec!["12", "34"]);
        assert_eq!(matches(&re_set, "12=3,4"), vec!["12", "3,4"]);
    }

    #[test]
    fn compute_toggle_set_index() {
        assert_eq!(Cli::compute_toggle_set_index(1, &[1, 4, 9]), 1);
        assert_eq!(Cli::compute_toggle_set_index(4, &[1, 4, 9]), 2);
        assert_eq!(Cli::compute_toggle_set_index(9, &[1, 4, 9]), 3);
        // The result should be 0 if the `value` isn't in the list.
        assert_eq!(Cli::compute_toggle_set_index(0, &[1, 4, 9]), 0);
        assert_eq!(Cli::compute_toggle_set_index(2, &[1, 4, 9]), 0);
        assert_eq!(Cli::compute_toggle_set_index(10, &[1, 4, 9]), 0);
    }
}
