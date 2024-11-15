use std::sync::Arc;

use crate::app_data::{ContainerId, DockerCommand};
use bollard::Docker;
use tokio::sync::oneshot::Sender;

#[derive(Debug)]
pub enum DockerMessage {
    ConfirmDelete(ContainerId),
    Control((DockerCommand, ContainerId)),

    // Delete(ContainerId),
    Exec(Sender<Arc<Docker>>),
    // Pause(ContainerId),
    Quit,
    // Restart(ContainerId),
    // Start(ContainerId),
    // Stop(ContainerId),
    // Resume(ContainerId),
    Update,
}
