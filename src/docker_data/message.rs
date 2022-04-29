#[derive(Debug, Clone)]
pub enum DockerMessage {
    Update,
    Start(String),
    Restart(String),
    Pause(String),
    Unpause(String),
    Stop(String),
}
