use crate::app_data::DockerCommand;
use std::fmt;

/// app errors to set in global state
#[derive(Debug, Clone, Copy)]
pub enum AppError {
    DockerCommand(DockerCommand),
    DockerExec,
    DockerLogs,
    DockerConnect,
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
            Self::MouseCapture(x) => {
                let reason = if *x { "en" } else { "dis" };
                write!(f, "Unable to {reason}able mouse capture")
            }
            Self::Terminal => write!(f, "Unable to fully render to terminal"),
        }
    }
}
