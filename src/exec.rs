use std::{
    io::{Read, Stdout, Write},
    sync::{atomic::AtomicBool, mpsc::Sender, Arc},
};

use bollard::{
    exec::{CreateExecOptions, ResizeExecOptions, StartExecOptions, StartExecResults},
    Docker,
};
use crossterm::terminal::enable_raw_mode;
use futures_util::StreamExt;
use parking_lot::Mutex;
use ratatui::{backend::CrosstermBackend, Terminal};
use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncWriteExt},
};
use tokio_util::sync::CancellationToken;

use crate::{
    app_data::{AppData, ContainerId, RunningState, State},
    app_error::AppError,
};

/// TTY location
const TTY: &str = "/dev/tty";

/// This will be the start of a docker exec message if one is unable to actually exec into the container
const OCI_ERROR: &str = "OCI runtime exec failed";

/// Set the cursor position on the screen to (0,0)
const CURSOR_POS: &str = "\x1B[J\x1B[H";

/// This needs to be written to stdout when exiting the exec mode, else the input handler thread gets confused,
/// see https://sw.kovidgoyal.net/kitty/keyboard-protocol/#progressive-enhancement
const KEYBOARD_PROTO: &str = "\x1B[?u\x1B[c";

mod command {
    pub const PWD: &str = "pwd";
    pub const DOCKER: &str = "docker";
    pub const EXEC: &str = "exec";
    pub const SH: &str = "sh";
    pub const IT: &str = "-it";
}

/// Currently known byte output after writing KEYBOARD_PROTO to stdout
/// valid arm: [91, 63, 54, 49, 59, 54, 59, 55, 59, 50, 50, 59, 50, 51, 59, 50, 52, 59, 50, 56, 59, 51, 50, 59,52, 50] => [?61;6;7;22;23;24;28;32;2
/// valid x86: [91, 63, 49, 59, 50, 99] => [?1;2c
/// invalid x86: [91, 63, 49, 59, 48, 99] => [?1;0c
enum ByteOutput {
    Arm,
    X86,
}

impl ByteOutput {
    const fn len(&self) -> usize {
        match self {
            Self::Arm => 26,
            Self::X86 => 6,
        }
    }
    const fn last(&self) -> &[u8] {
        match self {
            Self::Arm => &[50],
            Self::X86 => &[99],
        }
    }
}

/// Check the output from tty to see if it matches known sequence.
/// At the moment we only need to check the length and end digit, as x86 valid and invalid match in these two regards
fn byte_sequence_valid(bytes: &[u8]) -> bool {
    [ByteOutput::Arm, ByteOutput::X86]
        .iter()
        .any(|i| i.len() == bytes.len() && bytes.ends_with(i.last()))
}

/// Check if tty is able to be written to, aka not windows
pub fn tty_readable() -> bool {
    std::fs::OpenOptions::new()
        .read(true)
        .write(false)
        .open(TTY)
        .is_ok()
}

struct AsyncTTY {
    rx: std::sync::mpsc::Receiver<u8>,
}

impl AsyncTTY {
    /// Use an async timeout to read data from the file, and send to the "main" thread
    async fn read_loop(mut f: File, tx: Sender<u8>) {
        loop {
            let mut buf = [0];
            if tokio::time::timeout(std::time::Duration::from_millis(10), f.read_exact(&mut buf))
                .await
                .is_ok()
                && tx.send(buf[0]).is_err()
            {
                break;
            }
        }
    }

    /// Async tty reading, spawned into its own tokio thread
    fn get(cancel_token: &CancellationToken) -> Option<Self> {
        if tty_readable() {
            let (tx, rx) = std::sync::mpsc::channel();
            let cancel_token = cancel_token.to_owned();
            tokio::spawn(async move {
                if let Ok(f) = tokio::fs::File::open(TTY).await {
                    tokio::select! {
                    () = cancel_token.cancelled() => (),
                    () = Self::read_loop(f, tx) => cancel_token.cancel(),
                    }
                }
            });
            Some(Self { rx })
        } else {
            None
        }
    }
}

/// This is used to set the terminal size when exec via the Internal method
#[derive(Debug, Clone)]
pub struct TerminalSize {
    width: u16,
    height: u16,
}

impl TerminalSize {
    pub fn new(terminal: &Terminal<CrosstermBackend<Stdout>>) -> Option<Self> {
        terminal.size().map_or(None, |i| {
            Some(Self {
                width: i.width,
                height: i.height,
            })
        })
    }
}

#[derive(Debug, Clone)]
pub enum ExecMode {
    // use Bollard Rust library
    Internal((Arc<ContainerId>, Arc<Docker>)),
    // use the external `docker-cli`
    External(Arc<ContainerId>),
}

impl ExecMode {
    /// Test if we can exec into the selected container, first via the Internal methods, then by the External
    /// If the container is oxker, it will always return None
    pub async fn new(app_data: &Arc<Mutex<AppData>>, docker: &Arc<Docker>) -> Option<Self> {
        let is_oxker = app_data.lock().is_oxker();
        if is_oxker {
            return None;
        }

        let use_cli = app_data.lock().config.use_cli;
        let container = app_data.lock().get_selected_container_id_state_name();

        if let Some((id, state, _)) = container {
            if [
                State::Running(RunningState::Healthy),
                State::Running(RunningState::Unhealthy),
            ]
            .contains(&state)
            {
                if tty_readable() && !use_cli {
                    if let Ok(exec) = docker
                        .create_exec(
                            id.get(),
                            CreateExecOptions {
                                attach_stdout: Some(true),
                                attach_stderr: Some(true),
                                cmd: Some(vec![command::PWD]),
                                ..Default::default()
                            },
                        )
                        .await
                    {
                        if let Ok(StartExecResults::Attached { mut output, .. }) =
                            docker.start_exec(&exec.id, None).await
                        {
                            if let Some(Ok(msg)) = output.next().await {
                                if !msg.to_string().starts_with(OCI_ERROR) {
                                    return Some(Self::Internal((
                                        Arc::new(id),
                                        Arc::clone(docker),
                                    )));
                                }
                            }
                        }
                    }
                }

                if let Ok(output) = std::process::Command::new(command::DOCKER)
                    .args([command::EXEC, id.get(), command::PWD])
                    .output()
                {
                    if let Ok(output) = String::from_utf8(output.stdout) {
                        if !output.starts_with(OCI_ERROR) {
                            return Some(Self::External(Arc::new(id)));
                        }
                    }
                }
            }
        }
        None
    }

    /// exec into the container using the external docker cli, the result it just piped into oxker
    fn exec_external(id: &ContainerId) {
        let mut stdout = std::io::stdout();
        stdout.write_all(CURSOR_POS.as_bytes()).ok();
        if let Ok(mut child) = std::process::Command::new(command::DOCKER)
            .args([command::EXEC, command::IT, id.get(), command::SH])
            .stdin(std::process::Stdio::inherit())
            .stdout(std::process::Stdio::inherit())
            .stderr(std::process::Stdio::inherit())
            .spawn()
        {
            child.wait().ok();
            if child.kill().is_err() {
                std::process::exit(1)
            }
        }
    }

    /// Exec into the container via the Bollard library, stdout & stdin on different threads
    /// Have to deal with strange output once dropped, hence the use of internal_cleanup() method
    async fn exec_internal(
        &self,
        id: &ContainerId,
        docker: &Arc<Docker>,
        terminal_size: Option<TerminalSize>,
    ) -> Result<(), AppError> {
        let cancel_token = CancellationToken::new();

        if let Ok(exec_result) = docker
            .create_exec(
                id.get(),
                CreateExecOptions {
                    attach_stdout: Some(true),
                    attach_stderr: Some(true),
                    attach_stdin: Some(true),
                    tty: Some(true),
                    cmd: Some(vec![command::SH]),
                    ..Default::default()
                },
            )
            .await
        {
            if let Ok(StartExecResults::Attached {
                mut output,
                mut input,
            }) = docker
                .start_exec(
                    &exec_result.id,
                    Some(StartExecOptions {
                        detach: false,
                        ..Default::default()
                    }),
                )
                .await
            {
                if let Some(tty) = AsyncTTY::get(&cancel_token) {
                    tokio::spawn(async move {
                        enable_raw_mode().ok();
                        let mut stdout = std::io::stdout();
                        stdout.write_all(CURSOR_POS.as_bytes()).ok();
                        stdout.flush().ok();
                        while let Some(Ok(x)) = output.next().await {
                            stdout.write_all(&x.into_bytes()).ok();
                            stdout.flush().ok();
                        }
                        cancel_token.cancel();
                    });

                    if let Some(terminal_size) = terminal_size {
                        docker
                            .resize_exec(
                                &exec_result.id,
                                ResizeExecOptions {
                                    height: terminal_size.height,
                                    width: terminal_size.width,
                                },
                            )
                            .await
                            .ok();
                    }

                    while let Ok(x) = tty.rx.recv() {
                        input.write_all(&[x]).await.ok();
                    }

                    self.internal_cleanup()?;
                }
            } else {
                return Err(AppError::Terminal);
            }
        }
        Ok(())
    }

    /// This is the fix for key pressed not being handled correctly on quit
    /// It writes a special message to the stdout, and then listens out for a valid response
    /// afterwhich it's assumes that we're completely done with TTY
    fn internal_cleanup(&self) -> Result<(), AppError> {
        match self {
            Self::External(_) => Ok(()),
            Self::Internal(_) => {
                let waiting = Arc::new(AtomicBool::new(true));
                let waiting_thread = Arc::clone(&waiting);

                std::thread::spawn(move || {
                    let mut bytes = Vec::with_capacity(26);
                    while waiting_thread.load(std::sync::atomic::Ordering::SeqCst) {
                        let mut buf = [0];
                        if let Ok(mut f) = std::fs::File::open(TTY) {
                            if f.read_exact(&mut buf).is_err() {
                                waiting_thread.store(false, std::sync::atomic::Ordering::SeqCst);
                            }
                            bytes.push(buf[0]);
                            if byte_sequence_valid(&bytes) {
                                waiting_thread.store(false, std::sync::atomic::Ordering::SeqCst);
                            }
                        };
                    }
                });

                let mut stdout = std::io::stdout();
                stdout.write_all(KEYBOARD_PROTO.as_bytes()).ok();
                stdout.flush().ok();

                let start = std::time::Instant::now();
                while waiting.load(std::sync::atomic::Ordering::SeqCst) {
                    if start.elapsed().as_millis() > 1500 {
                        waiting.store(false, std::sync::atomic::Ordering::SeqCst);
                        return Err(AppError::Terminal);
                    }
                    std::thread::sleep(std::time::Duration::from_millis(10));
                }
                Ok(())
            }
        }
    }

    pub async fn run(&self, tty_size: Option<TerminalSize>) -> Result<(), AppError> {
        match self {
            Self::External(id) => {
                Self::exec_external(id);
                Ok(())
            }

            Self::Internal((id, docker)) => self.exec_internal(id, docker, tty_size).await,
        }
    }
}
