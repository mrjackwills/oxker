use clap::Parser;
use serde::Deserialize;

#[derive(Parser, Debug, Clone, Deserialize)]
#[allow(clippy::struct_excessive_bools)]
#[command(version, about)]
pub struct Args {
    /// Docker update interval in ms, minimum effectively 1000
    #[clap(short = 'd', value_name = "ms", default_value_t = 1000)]
    pub docker_interval: u32,

    /// Remove timestamps from Docker logs
    #[clap(short = 't')]
    pub timestamp: bool,

    /// Attempt to colorize the logs, conflicts with "-r"
    #[clap(short = 'c', conflicts_with = "raw")]
    pub color: bool,

    /// Show raw logs, default is to remove ansi formatting, conflicts with "-c"
    #[clap(short = 'r', conflicts_with = "color")]
    pub raw: bool,

    /// Show self when running as a docker container
    #[clap(short = 's')]
    pub show_self: bool,

    /// Don't draw gui - for debugging - mostly pointless
    #[clap(short = 'g')]
    pub gui: bool,

    /// Docker host, defaults to `/var/run/docker.sock`
    #[clap(long, short = None)]
    pub host: Option<String>,

    /// Do not include stderr output in logs
    #[clap(long = "no-stderr")]
    pub no_std_err: bool,

    /// Display the container logs timestamp with a given timezone, default is UTC
    #[clap(long="timezone", short = None)]
    pub timezone: Option<String>,

    /// Directory for saving exported logs, defaults to `$HOME`
    #[clap(long="save-dir", short = None)]
    pub save_dir: Option<String>,

    /// Path to a config file, readable as TOML, JSONC, or JSON
    #[clap(long="config-file", short = None)]
    pub config_file: Option<String>,

    /// Force use of docker cli when execing into containers
    #[clap(long="use-cli", short = None)]
    pub use_cli: bool,
}

impl Default for Args {
    fn default() -> Self {
        Self {
            docker_interval: 1000,
            timestamp: true,
            color: false,
            raw: false,
            show_self: false,
            gui: true,
            host: None,
            no_std_err: true,
            timezone: None,
            save_dir: None,
            config_file: None,
            use_cli: false,
        }
    }
}
