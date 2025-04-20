use std::time::Instant;

use super::*;
use clap::{ArgAction, Parser};
use log::*;
use regex::Regex;

#[derive(Debug, Default, Parser)]
#[command(version, about)]
/// A command line tool to change display monitors' input sources via DDC/CI.
///
/// # Examples
/// ```
/// # use monitor_input::Cli;
/// fn run_cli(args: Vec<String>) -> anyhow::Result<()> {
///     let mut cli = Cli::new();
///     cli.args = args;
///     cli.run()
/// }
/// ```
/// To setup [`Cli`] from the command line arguments:
/// ```no_run
/// use clap::Parser;
/// use monitor_input::{Cli,Monitor};
///
/// fn main() -> anyhow::Result<()> {
///     let mut cli = Cli::parse();
///     cli.init_logger();
///     cli.monitors = Monitor::enumerate();
///     cli.run()
/// }
/// ```
/// See <https://github.com/kojiishi/monitor-input-rs> for more details.
pub struct Cli {
    #[arg(skip)]
    /// The list of [`Monitor`]s to run the command line tool on.
    /// This field is usually initialized to [`Monitor::enumerate()`].
    pub monitors: Vec<Monitor>,

    #[arg(short, long)]
    /// Filter by the backend name.
    pub backend: Option<String>,

    #[arg(id = "capabilities", short, long)]
    /// Get capabilities from the display monitors.
    pub needs_capabilities: bool,

    #[arg(short = 'n', long)]
    /// Dry-run (prevent actual changes).
    pub dry_run: bool,

    #[arg(short, long, action = ArgAction::Count)]
    /// Show verbose information.
    pub verbose: u8,

    #[arg(skip)]
    set_index: Option<usize>,

    /// `name` to search,
    /// `name=input` to change the input source,
    /// or `name=input1,input2` to toggle.
    pub args: Vec<String>,
}

impl Cli {
    /// Construct an instance with display monitors from [`Monitor::enumerate()`].
    pub fn new() -> Self {
        Cli {
            monitors: Monitor::enumerate(),
            ..Default::default()
        }
    }

    /// Initialize the logging.
    /// The configurations depend on [`Cli::verbose`].
    pub fn init_logger(&self) {
        simplelog::CombinedLogger::init(vec![simplelog::TermLogger::new(
            match self.verbose {
                0 => simplelog::LevelFilter::Info,
                1 => simplelog::LevelFilter::Debug,
                _ => simplelog::LevelFilter::Trace
            },
            simplelog::Config::default(),
            simplelog::TerminalMode::Mixed,
            simplelog::ColorChoice::Auto,
        )])
        .unwrap();
    }

    fn apply_filters(&mut self) -> anyhow::Result<()> {
        if let Some(backend_str) = &self.backend {
            self.monitors
                .retain(|monitor| monitor.contains_backend(backend_str));
        }
        Ok(())
    }

    fn for_each<C>(&mut self, name: &str, mut callback: C) -> anyhow::Result<()>
    where
        C: FnMut(usize, &mut Monitor) -> anyhow::Result<()>,
    {
        if let Ok(index) = name.parse::<usize>() {
            let monitor = &mut self.monitors[index];
            if self.needs_capabilities {
                // This may fail in some cases. Print warning but keep looking.
                let _ = monitor.update_capabilities();
            }
            return callback(index, monitor);
        }

        let mut has_match = false;
        for (index, monitor) in (&mut self.monitors).into_iter().enumerate() {
            if self.needs_capabilities {
                // This may fail in some cases. Print warning but keep looking.
                let _ = monitor.update_capabilities();
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

        anyhow::bail!("No display monitors found for \"{name}\".");
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
                let current_input_source = monitor.current_input_source()?;
                set_index = Some(Self::compute_toggle_set_index(
                    current_input_source,
                    &input_sources,
                ));
                debug!(
                    "Set = {index} (because InputSource({monitor}) is {input_source})",
                    index = set_index.unwrap(),
                    input_source = InputSource::str_from_raw(current_input_source)
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
        self.for_each(name, |index, monitor| {
            println!("{index}: {}", monitor.to_long_string());
            trace!("{:?}", monitor);
            Ok(())
        })
    }

    fn sleep_all_if_needed(&mut self) {
        let start_time = Instant::now();
        for monitor in &mut self.monitors {
            monitor.sleep_if_needed();
        }
        debug!("sleep_all() elapsed: {:?}", start_time.elapsed());
    }

    const RE_SET_PATTERN: &str = r"^([^=]+)=(.+)$";

    /// Run the command line tool.
    pub fn run(&mut self) -> anyhow::Result<()> {
        let start_time = Instant::now();
        Monitor::set_dry_run(self.dry_run);
        self.apply_filters()?;

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
        self.sleep_all_if_needed();
        debug!("Elapsed: {:?}", start_time.elapsed());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::vec;

    use super::*;

    #[test]
    fn cli_parse() {
        let mut cli = Cli::parse_from([""]);
        assert_eq!(cli.verbose, 0);
        assert_eq!(cli.args.len(), 0);

        cli = Cli::parse_from(["", "abc", "def"]);
        assert_eq!(cli.verbose, 0);
        assert_eq!(cli.args, ["abc", "def"]);

        cli = Cli::parse_from(["", "-v", "abc", "def"]);
        assert_eq!(cli.verbose, 1);
        assert_eq!(cli.args, ["abc", "def"]);

        cli = Cli::parse_from(["", "-vv", "abc", "def"]);
        assert_eq!(cli.verbose, 2);
        assert_eq!(cli.args, ["abc", "def"]);
    }

    #[test]
    fn cli_parse_option_after_positional() {
        let cli = Cli::parse_from(["", "abc", "def", "-v"]);
        assert_eq!(cli.verbose, 1);
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
