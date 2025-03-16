use std::env;

use anyhow::Result;
use ddc_hi::{Ddc, FeatureCode};
use regex::Regex;

/// VCP feature code for input select
const INPUT_SELECT: FeatureCode = 0x60;

struct Display {
    ddc_hi_display: ddc_hi::Display,
}

impl std::fmt::Display for Display {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.ddc_hi_display.info.id)
    }
}

impl Display {
    fn update_capabilities(self: &mut Display) -> Result<()> {
        self.ddc_hi_display.update_capabilities()
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

    fn set_current_input_source(self: &mut Display, value: &str) -> Result<()> {
        println!("{} = {}", self, value);
        let feature_code: FeatureCode = self.feature_code(INPUT_SELECT);
        let ivalue = value.parse::<u16>().unwrap();
        self.ddc_hi_display
            .handle
            .set_vcp_feature(feature_code, ivalue)
    }

    fn to_long_string(self: &mut Display) -> String {
        let indent = "    ";
        let input_source = self.get_current_input_source();
        format!(
            "{}\n\
            {indent}Input Source: {:?}",
            self, input_source
        )
    }
}

struct Cli {
    displays: Vec<Display>,
    is_debug: bool,
}

impl Default for Cli {
    fn default() -> Self {
        Cli {
            displays: ddc_hi::Display::enumerate()
                .into_iter()
                .map(|d| Display { ddc_hi_display: d })
                .collect(),
            is_debug: false,
        }
    }
}

impl Cli {
    fn for_each<C>(self: &mut Cli, name: &str, mut callback: C) -> Result<()>
    where
        C: FnMut(&mut Display) -> Result<()>,
    {
        if let Ok(index) = name.parse::<usize>() {
            return callback(&mut self.displays[index]);
        }

        for display in (&mut self.displays)
            .into_iter()
            .filter(|d| d.contains(name))
        {
            callback(display)?;
        }

        Ok(())
    }

    fn set(self: &mut Cli, name: &str, value: &str) -> Result<()> {
        println!("Set: {name} = {value}");
        self.for_each(name, |display: &mut Display| {
            display.set_current_input_source(value)
        })
    }

    fn update_capabilities(self: &mut Cli) -> Result<()> {
        for display in &mut self.displays {
            if let Err(e) = display.update_capabilities() {
                eprintln!(
                    "{}: Failed to update capabilities, ignored.\n{}",
                    display, e
                );
            }
        }
        Ok(())
    }

    fn print_list(self: &mut Cli) -> Result<()> {
        for (index, display) in (&mut self.displays).into_iter().enumerate() {
            println!("{index}: {}", display.to_long_string());
            if self.is_debug {
                println!("{:?}", display.ddc_hi_display.info);
            }
        }
        Ok(())
    }

    fn run(self: &mut Cli) -> Result<()> {
        let re_set = Regex::new(r"^([^=]+)=([^=]+)$").unwrap();
        let mut has_valid_args = false;
        for arg in env::args().skip(1) {
            if arg.starts_with('-') {
                for ch in arg.chars().skip(1) {
                    match ch {
                        'c' => self.update_capabilities()?,
                        'D' => self.is_debug = true,
                        _ => {
                            println!("Invalid option \"{}\".", ch);
                            std::process::exit(1);
                        }
                    }
                }
                continue;
            }

            if let Some(captures) = re_set.captures(&arg) {
                self.set(&captures[1], &captures[2])?;
                has_valid_args = true;
                continue;
            }

            self.for_each(&arg, |display| {
                has_valid_args = true;
                println!("{display}");
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
    let mut cli: Cli = Default::default();
    cli.run()
}
