use std::env;
use std::str::FromStr;

use anyhow::Result;
use ddc_hi::{Ddc, FeatureCode};
use log::*;
use regex::Regex;
use strum_macros::{AsRefStr, EnumString, FromRepr};

#[derive(Debug, PartialEq, AsRefStr, EnumString, FromRepr)]
#[repr(u16)]
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
    fn u16_from_str(input: &str) -> Result<u16, strum::ParseError> {
        if let Ok(value) = input.parse::<u16>() {
            return Ok(value);
        }
        match InputSource::from_str(input) {
            Ok(value) => Ok(value as u16),
            Err(e) => Err(e),
        }
    }

    fn str_from_u16(value: u16) -> String {
        match InputSource::from_repr(value as u16) {
            Some(input_source) => input_source.as_ref().to_string(),
            None => value.to_string(),
        }
    }
}

/// VCP feature code for input select
const INPUT_SELECT: FeatureCode = 0x60;

struct Display {
    ddc_hi_display: ddc_hi::Display,
    is_capabilities_updated: bool,
}

impl std::fmt::Display for Display {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.ddc_hi_display.info.id)
    }
}

impl std::fmt::Debug for Display {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.ddc_hi_display.info)
    }
}

impl Display {
    fn new(ddc_hi_display: ddc_hi::Display) -> Self {
        Display {
            ddc_hi_display: ddc_hi_display,
            is_capabilities_updated: false,
        }
    }

    fn enumerate() -> Vec<Self> {
        ddc_hi::Display::enumerate()
            .into_iter()
            .map(|d| Display::new(d))
            .collect()
    }

    fn update_capabilities(self: &mut Display) -> Result<()> {
        debug!("update_capabilities: {}", self);
        self.is_capabilities_updated = true;
        self.ddc_hi_display.update_capabilities()
    }

    fn ensure_capabilities(self: &mut Display) -> Result<()> {
        if self.is_capabilities_updated {
            return Ok(());
        }
        self.update_capabilities()
    }

    fn ensure_capabilities_as_warn(self: &mut Display) {
        if let Err(e) = self.ensure_capabilities() {
            warn!("{}: Failed to update capabilities: {}", self, e);
        }
    }

    fn contains(self: &Display, name: &str) -> bool {
        self.ddc_hi_display.info.id.contains(name)
    }

    fn feature_code(self: &mut Display, feature_code: FeatureCode) -> FeatureCode {
        // TODO: `mccs_database` is initialized by `display.update_capabilities()`
        // which is quite slow, and it seems to work without this.
        // See also https://github.com/mjkoo/monitor-switch/blob/master/src/main.rs.
        if let Some(feature) = self.ddc_hi_display.info.mccs_database.get(feature_code) {
            return feature.code;
        }
        feature_code
    }

    fn get_current_input_source(self: &mut Display) -> Result<u8> {
        let feature_code: FeatureCode = self.feature_code(INPUT_SELECT);
        Ok(self.ddc_hi_display.handle.get_vcp_feature(feature_code)?.sl)
        // Err(io::Error::new(io::ErrorKind::Unsupported, "INPUT_SELECT not in MCCS").into())
    }

    fn set_current_input_source(self: &mut Display, value: u16) -> Result<()> {
        info!("{}.InputSource = {}", self, value);
        let feature_code: FeatureCode = self.feature_code(INPUT_SELECT);
        self.ddc_hi_display
            .handle
            .set_vcp_feature(feature_code, value)
    }

    fn to_long_string(self: &mut Display) -> String {
        let indent = "    ";
        let input_source = self.get_current_input_source();
        format!(
            "{}\n\
            {indent}Input Source: {:}",
            self,
            match input_source {
                Ok(value) => InputSource::str_from_u16(value as u16),
                Err(e) => e.to_string(),
            }
        )
    }
}

struct Cli {
    displays: Vec<Display>,
    is_debug: bool,
    is_logger_initialized: bool,
    needs_capabilities: bool,
}

impl Default for Cli {
    fn default() -> Self {
        Cli::new(Display::enumerate())
    }
}

impl Cli {
    fn new(displays: Vec<Display>) -> Self {
        Cli {
            displays: displays,
            is_debug: false,
            is_logger_initialized: false,
            needs_capabilities: false,
        }
    }

    fn for_each<C>(self: &mut Cli, name: &str, mut callback: C) -> Result<()>
    where
        C: FnMut(&mut Display) -> Result<()>,
    {
        if let Ok(index) = name.parse::<usize>() {
            return callback(&mut self.displays[index]);
        }

        for display in &mut self.displays {
            if self.needs_capabilities {
                display.ensure_capabilities_as_warn();
            }
            if !display.contains(name) {
                continue;
            }
            callback(display)?;
        }

        Ok(())
    }

    fn set(self: &mut Cli, name: &str, value: &str) -> Result<()> {
        let input_source = InputSource::u16_from_str(value)?;
        self.for_each(name, |display: &mut Display| {
            display.set_current_input_source(input_source)
        })
    }

    fn print_list(self: &mut Cli) -> Result<()> {
        self.ensure_logger();
        for (index, display) in (&mut self.displays).into_iter().enumerate() {
            if self.needs_capabilities {
                display.ensure_capabilities_as_warn();
            }
            println!("{index}: {}", display.to_long_string());
            debug!("{:?}", display);
        }
        Ok(())
    }

    fn ensure_logger(self: &mut Cli) {
        if self.is_logger_initialized {
            return;
        }
        self.is_logger_initialized = true;

        simplelog::CombinedLogger::init(vec![simplelog::TermLogger::new(
            if self.is_debug {
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

    fn parse_options(self: &mut Cli, arg: &String) {
        for ch in arg.chars().skip(1) {
            match ch {
                'c' => self.needs_capabilities = true,
                'D' => self.is_debug = true,
                _ => {
                    error!("Invalid option \"{}\".", ch);
                    std::process::exit(1);
                }
            }
        }
    }

    const RE_SET_PATTERN: &str = r"^([^=]+)=(.+)$";

    fn run(self: &mut Cli) -> Result<()> {
        let re_set = Regex::new(Self::RE_SET_PATTERN).unwrap();
        let mut has_valid_args = false;
        for arg in env::args().skip(1) {
            if arg.starts_with('-') {
                self.parse_options(&arg);
                continue;
            }
            self.ensure_logger();

            if let Some(captures) = re_set.captures(&arg) {
                self.set(&captures[1], &captures[2])?;
                has_valid_args = true;
                continue;
            }

            self.for_each(&arg, |display| {
                has_valid_args = true;
                println!("{display}");
                debug!("{:?}", display);
                Ok(())
            })?;
        }
        if !has_valid_args {
            self.print_list()?;
        }
        Ok(())
    }
}

fn main() -> Result<()> {
    let mut cli: Cli = Cli::default();
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
    fn input_source_u16_from_str() {
        assert_eq!(InputSource::u16_from_str("27"), Ok(27));
        // Upper-compatible with `from_str`.
        assert_eq!(
            InputSource::u16_from_str("Hdmi1"),
            Ok(InputSource::Hdmi1 as u16)
        );
        // Test failures.
        assert!(InputSource::u16_from_str("xyz").is_err());
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
    }
}
