use std::path::PathBuf;

use clap::Parser;
use jiff::tz::TimeZone;
use parse_args::Args;
use parse_config_file::ConfigFile;
mod color_parser;
mod keymap_parser;

use crate::{ENV_KEY, ENV_VALUE};
pub use {color_parser::AppColors, keymap_parser::Keymap};

mod parse_args;
mod parse_config_file;

#[derive(Debug, Clone)]
#[allow(clippy::struct_excessive_bools)]
pub struct Config {
    pub app_colors: AppColors,
    pub color_logs: bool,
    pub docker_interval_ms: u32,
    pub gui: bool,
    pub host: Option<String>,
    pub in_container: bool,
    pub keymap: Keymap,
    pub raw_logs: bool,
    pub save_dir: Option<PathBuf>,
    pub show_self: bool,
    pub show_std_err: bool,
    pub show_timestamp: bool,
    pub timezone: Option<TimeZone>,
    pub timestamp_format: String,
    pub use_cli: bool,
}

impl From<&Args> for Config {
    fn from(args: &Args) -> Self {
        Self {
            app_colors: AppColors::new(),
            color_logs: args.color,
            docker_interval_ms: args.docker_interval,
            gui: !args.gui,
            host: args.host.clone(),
            in_container: Self::check_if_in_container(),
            keymap: Keymap::new(),
            raw_logs: args.raw,
            save_dir: Self::try_get_logs_dir(args.save_dir.as_ref()),
            show_self: !args.show_self,
            show_std_err: !args.no_std_err,
            show_timestamp: !args.timestamp,
            timezone: Self::parse_timezone(args.timezone.clone()),
            timestamp_format: Self::parse_timestamp_format(None),
            use_cli: args.use_cli,
        }
    }
}

impl From<ConfigFile> for Config {
    fn from(config_file: ConfigFile) -> Self {
        Self {
            app_colors: AppColors::from(config_file.colors),
            color_logs: config_file.color_logs.unwrap_or(false),
            docker_interval_ms: config_file.docker_interval.unwrap_or(1000),
            gui: config_file.gui.unwrap_or(true),
            host: config_file.host,
            in_container: Self::check_if_in_container(),
            keymap: Keymap::from(config_file.keymap),
            raw_logs: config_file.raw_logs.unwrap_or(false),
            save_dir: Self::try_get_logs_dir(config_file.save_dir.as_ref()),
            show_self: config_file.show_self.unwrap_or(false),
            show_std_err: config_file.show_std_err.unwrap_or(true),
            show_timestamp: config_file.show_timestamp.unwrap_or(true),
            timezone: Self::parse_timezone(config_file.timezone),
            timestamp_format: Self::parse_timestamp_format(config_file.timestamp_format),
            use_cli: config_file.use_cli.unwrap_or(false),
        }
    }
}

impl Config {
    /// A basic timestampt format parser, will only take 32 chars, and checks if the parsed timestamp isn't identical to the given formatter
    fn parse_timestamp_format(input: Option<String>) -> String {
        let default = || "%Y-%m-%dT%H:%M:%S.%8f".to_owned();
        input.map_or_else(default, |input| {
            if input.chars().count() >= 32
                || jiff::Timestamp::now().strftime(&input).to_string() == input
            {
                default()
            } else {
                input
            }
        })
    }

    /// Attempt to parse a timezone into a jiff::tz::TimeZone
    /// Also return a format to display the timesampt in
    fn parse_timezone(input: Option<String>) -> Option<TimeZone> {
        let timezone_str = input?;
        let Ok(tz) = jiff::tz::TimeZone::get(&timezone_str) else {
            return None;
        };
        let current_ts = jiff::Timestamp::now();
        let offset = tz.to_offset(current_ts);
        if jiff::tz::TimeZone::UTC.to_offset(current_ts) == offset {
            None
        } else {
            Some(tz)
        }
    }
    /// Check if oxker is running inside of a container
    fn check_if_in_container() -> bool {
        std::env::var(ENV_KEY).is_ok_and(|i| i == ENV_VALUE)
    }

    /// If a cli_arg is provided, create a pathbuf from it, else try to get home_dir automatically
    fn try_get_logs_dir(dir: Option<&String>) -> Option<PathBuf> {
        dir.as_ref()
            .map_or_else(Self::try_get_home_dir, |home_dir| {
                Some(std::path::Path::new(&home_dir).to_owned())
            })
    }

    /// Try to get the home dir of the current user
    fn try_get_home_dir() -> Option<PathBuf> {
        directories::BaseDirs::new().map(|base_dirs| base_dirs.home_dir().to_owned())
    }

    /// Combine config from CLI into config file, the cli take priority
    /// and also make sure color_logs and raw_logs can't clash
    fn merge_args(mut self, config_from_cli: Self) -> Self {
        let default_args = Args::default();

        if config_from_cli.color_logs != default_args.color {
            self.color_logs = config_from_cli.color_logs;
            self.raw_logs = !self.color_logs;
        }

        if config_from_cli.raw_logs != default_args.raw {
            self.raw_logs = config_from_cli.raw_logs;
            self.color_logs = !self.raw_logs;
        }

        if config_from_cli.gui != default_args.gui {
            self.gui = config_from_cli.gui;
        }

        if config_from_cli.docker_interval_ms != default_args.docker_interval {
            self.docker_interval_ms = config_from_cli.docker_interval_ms;
        }

        if config_from_cli.docker_interval_ms < 1000 {
            self.docker_interval_ms = default_args.docker_interval;
        }

        if config_from_cli.raw_logs != default_args.raw {
            self.raw_logs = config_from_cli.raw_logs;
        }

        if config_from_cli.show_self != default_args.show_self {
            self.show_self = config_from_cli.show_self;
        }

        if config_from_cli.show_std_err != default_args.no_std_err {
            self.show_std_err = config_from_cli.show_std_err;
        }

        if config_from_cli.show_timestamp != default_args.timestamp {
            self.show_timestamp = config_from_cli.show_timestamp;
        }

        if config_from_cli.use_cli != default_args.use_cli {
            self.use_cli = config_from_cli.use_cli;
        }

        if let Some(host) = config_from_cli.host {
            self.host = Some(host);
        }

        if let Some(x) = config_from_cli.save_dir {
            self.save_dir = Some(x);
        }

        if let Some(tz) = config_from_cli.timezone {
            self.timezone = Some(tz);
        }

        if self.color_logs && self.raw_logs {
            self.raw_logs = false;
        }
        self
    }

    /// Generate a new config file
    /// First check cli args,
    /// then if a config file location is given check then
    /// Else check the default location
    /// else just return the default config + the cli args
    /// cli args will take precedence over config settings
    pub fn new() -> Self {
        let in_container = Self::check_if_in_container();

        let args = Args::parse();
        let config_from_cli = Self::from(&args);

        if let Some(config_file) = &args.config_file {
            if let Some(config_file) =
                parse_config_file::ConfigFile::try_parse_from_file(config_file)
            {
                return Self::from(config_file).merge_args(config_from_cli);
            }
        }

        if let Some(config_file) = parse_config_file::ConfigFile::try_parse(in_container) {
            return Self::from(config_file).merge_args(config_from_cli);
        }
        config_from_cli
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use jiff::tz::TimeZone;

    /// Test the basic timestamp_format parsing/checker function
    #[test]
    fn test_config_parse_timestamp_format() {
        let default = "%Y-%m-%dT%H:%M:%S.%8f";

        let result = super::Config::parse_timestamp_format(None);
        assert_eq!(result, default);

        let result = super::Config::parse_timestamp_format(Some(String::new()));
        assert_eq!(result, default);

        let result = super::Config::parse_timestamp_format(Some(" ".to_owned()));
        assert_eq!(result, default);

        let result = super::Config::parse_timestamp_format(Some(" ".to_owned()));
        assert_eq!(result, default);

        let result =
            super::Config::parse_timestamp_format(Some("not a valid formatter".to_owned()));
        assert_eq!(result, default);

        let result = super::Config::parse_timestamp_format(Some(
            "%A, %B %d, %Y %I:%M %p %A, %B %d, %Y %I:%M %p".to_owned(),
        ));
        assert_eq!(result, default);

        let input = "%Y-%m-%d %H:%M:%S";
        let result = super::Config::parse_timestamp_format(Some(input.to_owned()));
        assert_eq!(result, input);

        let input = "%Y-%j";
        let result = super::Config::parse_timestamp_format(Some(input.to_owned()));
        assert_eq!(result, input);
    }

    #[test]
    /// Test various timezones get parsed correctly
    fn test_config_parse_timezone() {
        // Timezone with no offset just return None
        for i in [None, Some("UTC".to_owned())] {
            assert!(super::Config::parse_timezone(i).is_none());
        }

        let expected = Some(TimeZone::get("Asia/Tokyo").unwrap());
        // string case ignored
        for i in ["ASIA/TOKYO", "asia/tokyo", "aSiA/tOkYo"] {
            let result = super::Config::parse_timezone(Some(i.to_owned()));
            assert!(result.is_some());
            assert_eq!(result, expected);
        }
    }
}
