use crate::app_data::DockerControls;
use std::fmt;

/// app errors to set in global state
#[allow(unused)]
#[derive(Debug, Clone)]
pub enum AppError {
    DockerConnect,
    DockerInterval,
    InputPoll,
    DockerCommand(DockerControls),
    MouseCapture(bool),
    Terminal,
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
            Self::MouseCapture(x) => {
                let reason = if *x { "en" } else { "dis" };
                format!("Unable to {}able mouse capture", reason)
            }
        };
        write!(f, "{}", disp)
    }
}
