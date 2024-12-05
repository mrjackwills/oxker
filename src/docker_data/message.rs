use std::sync::Arc;

use crate::app_data::{ContainerId, DockerCommand};
use bollard::Docker;
use tokio::sync::oneshot::Sender;

#[derive(Debug)]
pub enum DockerMessage {
    ConfirmDelete(ContainerId),
    Control((DockerCommand, ContainerId)),
    Exec(Sender<Arc<Docker>>),
    Update,
}
