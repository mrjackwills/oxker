use core::fmt;
use tracing::error;

use crate::app_data::DockerControls;

/// app errors to set in global state
#[allow(unused)]
#[derive(Debug, Clone)]
pub enum AppError {
    DockerConnect,
    DockerInterval,
    InputPoll,
    DockerCommand(DockerControls),
    Terminal,
}

impl AppError {
    /// for handling errors from terminal
    pub fn disp(&self) {
        match self {
            Self::DockerConnect => error!("Unable to access docker daemon"),
            Self::DockerInterval => error!("Docker update interval needs to be greater than 0"),
            Self::InputPoll => error!("Unable to poll user input"),
            Self::Terminal => error!("Unable to draw to terminal"),
            Self::DockerCommand(s) => {
                let error = format!("Unable to {} container", s);
                error!(%error);
            }
        }
    }
}

/// Convert errors into strings to display
impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let disp = match self {
            Self::DockerConnect => "Unable to access docker daemon".to_owned(),
            Self::DockerInterval => "Docker update interval needs to be greater than 0".to_owned(),
            Self::InputPoll => "Unable to poll user input".to_owned(),
            Self::Terminal => "Unable to draw to terminal".to_owned(),
            Self::DockerCommand(s) => format!("Unable to {} container", s),
        };
        write!(f, "{}", disp)
    }
}
