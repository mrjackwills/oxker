use crate::app_data::ContainerId;

#[derive(Debug, Clone)]
pub enum DockerMessage {
    Update,
    Start(ContainerId),
    Restart(ContainerId),
    Pause(ContainerId),
    Unpause(ContainerId),
    Stop(ContainerId),
    Quit,
}
