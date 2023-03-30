use crate::app_data::ContainerId;

#[derive(Debug, Clone)]
pub enum DockerMessage {
    Delete(ContainerId),
    ConfirmDelete(ContainerId),
    Pause(ContainerId),
    Quit,
    Restart(ContainerId),
    Start(ContainerId),
    Stop(ContainerId),
    Unpause(ContainerId),
    Update,
}
