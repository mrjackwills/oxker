use crate::app_data::DockerControls;
use std::fmt;

/// app errors to set in global state
#[allow(unused)]
#[derive(Debug, Clone, Copy)]
pub enum AppError {
    DockerCommand(DockerControls),
    DockerConnect,
    DockerInterval,
    InputPoll,
    MouseCapture(bool),
    Terminal,
}

/// Convert errors into strings to display
impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::DockerCommand(s) => write!(f, "Unable to {} container", s),
            Self::DockerConnect => write!(f, "Unable to access docker daemon"),
            Self::DockerInterval => write!(f, "Docker update interval needs to be greater than 0"),
            Self::InputPoll => write!(f, "Unable to poll user input"),
            Self::MouseCapture(x) => {
                let reason = if *x { "en" } else { "dis" };
                write!(f, "Unbale to {}able mouse capture", reason)
            }
            Self::Terminal => write!(f, "Unable to draw to terminal"),
        }
    }
}
