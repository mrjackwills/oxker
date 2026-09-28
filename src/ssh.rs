use std::{
    path::PathBuf,
    process::Stdio,
    time::{Duration, Instant},
};

use tokio::{
    io::AsyncReadExt,
    process::{Child, Command},
};

const SCHEME: &str = "ssh://";
const DEFAULT_REMOTE_SOCKET: &str = "/var/run/docker.sock";
/// Generous, so that the user has time to answer password / host key prompts
const CONNECT_TIMEOUT: Duration = Duration::from_secs(60);
const POLL_INTERVAL: Duration = Duration::from_millis(50);

/// A remote Docker daemon reachable over ssh, parsed from `ssh://[user@]host[:port][/remote/socket]`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SshTarget {
    destination: String,
    port: Option<u16>,
    remote_socket: String,
}

impl SshTarget {
    /// Returns None if the host isn't an ssh url, or has no destination
    pub fn parse(host: &str) -> Option<Self> {
        let rest = host.trim().strip_prefix(SCHEME)?;
        let (authority, path) = rest
            .find('/')
            .map_or((rest, ""), |i| (&rest[..i], &rest[i..]));

        let host_start = authority.rfind('@').map_or(0, |i| i + 1);
        let (destination, port) = match authority.rsplit_once(':') {
            Some((dest, port)) if dest.len() >= host_start => port
                .parse::<u16>()
                .map_or((authority, None), |port| (dest, Some(port))),
            _ => (authority, None),
        };
        let destination = destination.replace(['[', ']'], "");
        if destination.is_empty() || destination.ends_with('@') {
            return None;
        }

        let remote_socket = if path.len() > 1 {
            path.to_owned()
        } else {
            DEFAULT_REMOTE_SOCKET.to_owned()
        };

        Some(Self {
            destination,
            port,
            remote_socket,
        })
    }

    fn args(&self, local_socket: &str) -> Vec<String> {
        let mut args = vec![
            "-NT".to_owned(),
            "-o".to_owned(),
            "ExitOnForwardFailure=yes".to_owned(),
            "-o".to_owned(),
            "LogLevel=ERROR".to_owned(),
            "-L".to_owned(),
            format!("{local_socket}:{}", self.remote_socket),
        ];
        if let Some(port) = self.port {
            args.extend(["-p".to_owned(), port.to_string()]);
        }
        args.extend(["--".to_owned(), self.destination.clone()]);
        args
    }
}

/// A system `ssh` process forwarding the remote Docker socket to a private local unix socket.
/// Uses the user's ssh config, keys, agent, and ProxyJump settings. The process is killed on drop.
#[derive(Debug)]
pub struct SshTunnel {
    child: Child,
    socket: PathBuf,
}

impl SshTunnel {
    pub async fn open(target: &SshTarget) -> Result<Self, String> {
        if cfg!(not(unix)) {
            return Err("ssh hosts are only supported on unix platforms".to_owned());
        }
        let id = uuid::Uuid::new_v4().simple().to_string();
        let socket = std::env::temp_dir().join(format!("oxker-{}.sock", &id[..16]));

        let mut child = Command::new("ssh")
            .args(target.args(&socket.to_string_lossy()))
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .kill_on_drop(true)
            .spawn()
            .map_err(|e| format!("unable to run ssh: {e}"))?;

        let start = Instant::now();
        loop {
            if socket.exists() {
                return Ok(Self { child, socket });
            }
            if let Ok(Some(status)) = child.try_wait() {
                let mut stderr = String::new();
                if let Some(mut pipe) = child.stderr.take() {
                    pipe.read_to_string(&mut stderr).await.ok();
                }
                return Err(format!("ssh exited with {status}: {}", stderr.trim()));
            }
            if start.elapsed() > CONNECT_TIMEOUT {
                return Err(format!("timed out connecting to {}", target.destination));
            }
            tokio::time::sleep(POLL_INTERVAL).await;
        }
    }

    pub fn socket(&self) -> String {
        self.socket.to_string_lossy().into_owned()
    }
}

impl Drop for SshTunnel {
    fn drop(&mut self) {
        self.child.start_kill().ok();
        std::fs::remove_file(&self.socket).ok();
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::SshTarget;

    fn target(destination: &str, port: Option<u16>, remote_socket: &str) -> Option<SshTarget> {
        Some(SshTarget {
            destination: destination.to_owned(),
            port,
            remote_socket: remote_socket.to_owned(),
        })
    }

    #[test]
    /// Non-ssh hosts, or ssh urls without a destination, are ignored
    fn test_ssh_target_parse_invalid() {
        for i in [
            "/var/run/docker.sock",
            "unix:///var/run/docker.sock",
            "tcp://127.0.0.1:2375",
            "ssh://",
            "ssh:///run/docker.sock",
            "ssh://user@",
        ] {
            assert!(SshTarget::parse(i).is_none(), "{i}");
        }
    }

    #[test]
    /// Destination, optional user, port, and remote socket are all parsed
    fn test_ssh_target_parse_valid() {
        let default = "/var/run/docker.sock";
        assert_eq!(
            SshTarget::parse("ssh://myhost"),
            target("myhost", None, default)
        );
        assert_eq!(
            SshTarget::parse(" ssh://myhost/ "),
            target("myhost", None, default)
        );
        assert_eq!(
            SshTarget::parse("ssh://user@10.0.0.1:2222"),
            target("user@10.0.0.1", Some(2222), default)
        );
        assert_eq!(
            SshTarget::parse("ssh://user@host/run/user/1000/podman/podman.sock"),
            target("user@host", None, "/run/user/1000/podman/podman.sock")
        );
        assert_eq!(
            SshTarget::parse("ssh://user@[::1]:22"),
            target("user@::1", Some(22), default)
        );
    }

    #[test]
    /// Port is passed with -p, and destination is always last, after --
    fn test_ssh_target_args() {
        let args = SshTarget::parse("ssh://user@host:2222/remote.sock")
            .unwrap()
            .args("/tmp/local.sock");
        assert!(args.contains(&"/tmp/local.sock:/remote.sock".to_owned()));
        assert!(args.windows(2).any(|w| w == ["-p", "2222"]));
        assert_eq!(&args[args.len() - 2..], ["--", "user@host"]);
    }
}
