use crate::app_data::DockerControls;
use std::fmt;

/// app errors to set in global state
#[allow(unused)]
#[derive(Debug, Clone, Copy)]
pub enum AppError {
    DockerCommand(DockerControls),
    DockerExec,
    DockerLogs,
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
            Self::DockerCommand(s) => write!(f, "Unable to {s} container"),
            Self::DockerExec => write!(f, "Unable to exec into container"),
            Self::DockerLogs => write!(f, "Unable to save logs"),
            Self::DockerConnect => write!(f, "Unable to access docker daemon"),
            Self::DockerInterval => write!(f, "Docker update interval needs to be greater than 0"),
            Self::InputPoll => write!(f, "Unable to poll user input"),
            Self::MouseCapture(x) => {
                let reason = if *x { "en" } else { "dis" };
                write!(f, "Unable to {reason}able mouse capture")
            }
            Self::Terminal => write!(f, "Unable to fully render to terminal"),
        }
    }
}
