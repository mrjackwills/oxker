use std::process;

use clap::Parser;
use tracing::error;

#[derive(Parser, Debug, Clone)]
#[clap(about, version, author)]

pub struct CliArgs {
    /// Docker update interval in ms, minimum 1, reccomended 500+
    #[clap(short = 'd', value_name = "ms", default_value_t = 1000)]
    pub docker_interval: u32,

    /// Don't draw gui - for debugging - mostly pointless
    #[clap(short = 'g')]
    pub gui: bool,

    /// Remove timestamps from Docker logs
    #[clap(short = 't')]
    pub timestamp: bool,

    /// Show raw logs, default is to remove ansi formatting
    #[clap(short = 'r', conflicts_with = "color")]
    pub raw: bool,

    /// Attempt to colorize the logs
    #[clap(short = 'c', conflicts_with = "raw")]
    pub color: bool,
}

impl CliArgs {
    /// Parse cli arguments
    pub fn new() -> Self {
        let args = CliArgs::parse();

        // Quit the program if the docker update argument is 0
        // Should maybe change it to check if less than 100
        if args.docker_interval == 0 {
            error!("docker args needs to be greater than 0");
            process::exit(1)
        }
        Self {
            color: args.color,
            docker_interval: args.docker_interval,
            gui: !args.gui,
            raw: args.raw,
            timestamp: !args.timestamp,
        }
    }
}
