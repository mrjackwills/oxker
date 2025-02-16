use std::path::PathBuf;

use clap::Parser;
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
    pub docker_interval: u32,
    pub gui: bool,
    pub host: Option<String>,
    pub in_container: bool,
    pub keymap: Keymap,
    pub raw_logs: bool,
    pub save_dir: Option<PathBuf>,
    pub show_self: bool,
    pub show_std_err: bool,
    pub show_timestamp: bool,
    pub use_cli: bool,
}

impl From<Args> for Config {
    fn from(args: Args) -> Self {
        Self {
            app_colors: AppColors::new(),
            color_logs: args.color,
            docker_interval: args.docker_interval,
            gui: !args.gui,
            host: args.host,
            in_container: Self::check_if_in_container(),
            keymap: Keymap::new(),
            raw_logs: args.raw,
            save_dir: Self::try_get_logs_dir(args.save_dir.as_ref()),
            show_self: !args.show_self,
            show_std_err: !args.no_std_err,
            show_timestamp: !args.timestamp,
            use_cli: args.use_cli,
        }
    }
}

impl From<ConfigFile> for Config {
    fn from(config_file: ConfigFile) -> Self {
        Self {
            app_colors: AppColors::from(config_file.colors),
            color_logs: config_file.color_logs.unwrap_or(false),
            docker_interval: config_file.docker_interval.unwrap_or(1000),
            gui: config_file.gui.unwrap_or(true),
            host: config_file.host,
            in_container: Self::check_if_in_container(),
            keymap: Keymap::from(config_file.keymap),
            raw_logs: config_file.raw_logs.unwrap_or(false),
            save_dir: Self::try_get_logs_dir(config_file.save_dir.as_ref()),
            show_self: config_file.show_self.unwrap_or(false),
            show_std_err: config_file.show_std_err.unwrap_or(true),
            show_timestamp: config_file.show_timestamp.unwrap_or(true),
            use_cli: config_file.use_cli.unwrap_or(false),
        }
    }
}

impl Config {
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

    /// Generate a new config file
    /// First check cli args,
    /// then if a config file location is given check then
    /// Else check the default location
    /// else just return the default config + the cli args
    pub fn new() -> Self {
        let in_container = Self::check_if_in_container();

        let args = Args::parse();

        if let Some(config_file) = &args.config_file {
            if let Some(config_file) =
                parse_config_file::ConfigFile::try_parse_from_file(config_file)
            {
                return Self::from(config_file);
            }
        }

        if let Some(config_file) = parse_config_file::ConfigFile::try_parse(in_container) {
            return Self::from(config_file);
        }

        Self::from(args)
    }
}
