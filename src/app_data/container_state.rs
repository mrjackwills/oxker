use std::{cmp::Ordering, collections::VecDeque, fmt};

use tui::{
    style::Color,
    widgets::{ListItem, ListState},
};

use super::Header;

const ONE_KB: f64 = 1000.0;
const ONE_MB: f64 = ONE_KB * 1000.0;
const ONE_GB: f64 = ONE_MB * 1000.0;

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub struct ContainerId(String);

impl From<String> for ContainerId {
    fn from(x: String) -> Self {
        Self(x)
    }
}

impl From<&String> for ContainerId {
    fn from(x: &String) -> Self {
        Self(x.clone())
    }
}

impl From<&str> for ContainerId {
    fn from(x: &str) -> Self {
        Self(x.to_owned())
    }
}

impl ContainerId {
    pub fn get(&self) -> &str {
        self.0.as_str()
    }
}

impl Ord for ContainerId {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}

impl PartialOrd for ContainerId {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Clone)]
pub struct StatefulList<T> {
    pub state: ListState,
    pub items: Vec<T>,
}

impl<T> StatefulList<T> {
    pub fn new(items: Vec<T>) -> Self {
        Self {
            state: ListState::default(),
            items,
        }
    }

    pub fn end(&mut self) {
        let len = self.items.len();
        if len > 0 {
            self.state.select(Some(self.items.len() - 1));
        }
    }

    pub fn start(&mut self) {
        self.state.select(Some(0));
    }

    pub fn next(&mut self) {
        if !self.items.is_empty() {
            let i = match self.state.selected() {
                Some(i) => {
                    if i < self.items.len() - 1 {
                        i + 1
                    } else {
                        i
                    }
                }
                None => 0,
            };
            self.state.select(Some(i));
        }
    }

    pub fn previous(&mut self) {
        if !self.items.is_empty() {
            let i = match self.state.selected() {
                Some(i) => {
                    if i == 0 {
                        0
                    } else {
                        i - 1
                    }
                }
                None => 0,
            };
            self.state.select(Some(i));
        }
    }

	/// Return the current status of the select list, e.g. 2/5,
    pub fn get_state_title(&self) -> String {
        if self.items.is_empty() {
            String::new()
        } else {
            let len = self.items.len();
            let c = self
                .state
                .selected()
                .map_or(0, |value| if len > 0 { value + 1 } else { value });
            format!("{}/{}", c, self.items.len())
        }
    }
}

/// States of the container
#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd)]
pub enum State {
    Dead,
    Exited,
    Paused,
    Removing,
    Restarting,
    Running,
    Unknown,
}

impl State {
    pub const fn get_color(self) -> Color {
        match self {
            Self::Paused => Color::Yellow,
            Self::Removing => Color::LightRed,
            Self::Restarting => Color::LightGreen,
            Self::Running => Color::Green,
            _ => Color::Red,
        }
    }
    // Dirty way to create order for the state, rather than impl Ord
    pub const fn order(self) -> u8 {
        match self {
            Self::Running => 0,
            Self::Paused => 1,
            Self::Restarting => 2,
            Self::Removing => 3,
            Self::Exited => 4,
            Self::Dead => 5,
            Self::Unknown => 6,
        }
    }
}

impl From<String> for State {
    fn from(input: String) -> Self {
        match input.as_ref() {
            "dead" => Self::Dead,
            "exited" => Self::Exited,
            "paused" => Self::Paused,
            "removing" => Self::Removing,
            "restarting" => Self::Restarting,
            "running" => Self::Running,
            _ => Self::Unknown,
        }
    }
}

impl From<&str> for State {
    fn from(input: &str) -> Self {
        match input {
            "dead" => Self::Dead,
            "exited" => Self::Exited,
            "paused" => Self::Paused,
            "removing" => Self::Removing,
            "restarting" => Self::Restarting,
            "running" => Self::Running,
            _ => Self::Unknown,
        }
    }
}

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let disp = match self {
            Self::Dead => "✖ dead",
            Self::Exited => "✖ exited",
            Self::Paused => "॥ paused",
            Self::Removing => "removing",
            Self::Restarting => "↻ restarting",
            Self::Running => "✓ running",
            Self::Unknown => "? unknown",
        };
        write!(f, "{}", disp)
    }
}

/// Items for the container control list
#[derive(Debug, Clone, Copy)]
pub enum DockerControls {
    Pause,
    Restart,
    Start,
    Stop,
    Unpause,
}

impl DockerControls {
    pub const fn get_color(self) -> Color {
        match self {
            Self::Pause => Color::Yellow,
            Self::Restart => Color::Magenta,
            Self::Start => Color::Green,
            Self::Stop => Color::Red,
            Self::Unpause => Color::Blue,
        }
    }

    /// Docker commands available depending on the containers state
    pub fn gen_vec(state: State) -> Vec<Self> {
        match state {
            State::Dead | State::Exited => vec![Self::Start, Self::Restart],
            State::Paused => vec![Self::Unpause, Self::Stop],
            State::Restarting => vec![Self::Stop],
            State::Running => vec![Self::Pause, Self::Restart, Self::Stop],
            _ => vec![],
        }
    }
}

impl fmt::Display for DockerControls {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let disp = match self {
            Self::Pause => "pause",
            Self::Restart => "restart",
            Self::Start => "start",
            Self::Stop => "stop",
            Self::Unpause => "unpause",
        };
        write!(f, "{}", disp)
    }
}

pub trait Stats {
    fn get_value(&self) -> f64;
}

/// Struct for frequently updated CPU stats
/// So can use custom display formatter
/// Use trait Stats for use as generic in draw_chart function
#[derive(Debug, Default, Clone, Copy)]
pub struct CpuStats(f64);

impl CpuStats {
    pub const fn new(value: f64) -> Self {
        Self(value)
    }
}

impl Eq for CpuStats {}

impl PartialEq for CpuStats {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl PartialOrd for CpuStats {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl Ord for CpuStats {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.0 > other.0 {
            Ordering::Greater
        } else if (self.0 - other.0).abs() < 0.01 {
            Ordering::Equal
        } else {
            Ordering::Less
        }
    }
}

impl Stats for CpuStats {
    fn get_value(&self) -> f64 {
        self.0
    }
}

impl fmt::Display for CpuStats {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let disp = format!("{:05.2}%", self.0);
        write!(f, "{:>x$}", disp, x = f.width().unwrap_or(1))
    }
}

/// Struct for frequently updated memory usage stats
/// So can use custom display formatter
/// Use trait Stats for use as generic in draw_chart function
#[derive(Debug, Default, Clone, Copy, Eq)]
pub struct ByteStats(u64);

impl PartialEq for ByteStats {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl PartialOrd for ByteStats {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl Ord for ByteStats {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}

impl ByteStats {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }
    pub fn update(&mut self, value: u64) {
        self.0 = value;
    }
}

#[allow(clippy::cast_precision_loss)]
impl Stats for ByteStats {
    fn get_value(&self) -> f64 {
        self.0 as f64
    }
}

/// convert from bytes to kB, MB, GB etc
impl fmt::Display for ByteStats {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let as_f64 = self.get_value();
        let p = match as_f64 {
            x if x >= ONE_GB => format!("{y:.2} GB", y = as_f64 / ONE_GB),
            x if x >= ONE_MB => format!("{y:.2} MB", y = as_f64 / ONE_MB),
            x if x >= ONE_KB => format!("{y:.2} kB", y = as_f64 / ONE_KB),
            _ => format!("{} B", self.0),
        };
        write!(f, "{:>x$}", p, x = f.width().unwrap_or(1))
    }
}

pub type MemTuple = (Vec<(f64, f64)>, ByteStats, State);
pub type CpuTuple = (Vec<(f64, f64)>, CpuStats, State);

/// Info for each container
#[derive(Debug, Clone)]
pub struct ContainerItem {
    pub cpu_stats: VecDeque<CpuStats>,
    pub docker_controls: StatefulList<DockerControls>,
    pub id: ContainerId,
    pub image: String,
    pub last_updated: u64,
    pub logs: StatefulList<ListItem<'static>>,
    pub mem_limit: ByteStats,
    pub mem_stats: VecDeque<ByteStats>,
    pub name: String,
    pub rx: ByteStats,
    pub tx: ByteStats,
    pub state: State,
    pub status: String,
}

impl ContainerItem {
    /// Create a new container item
    pub fn new(id: ContainerId, status: String, image: String, state: State, name: String) -> Self {
        let mut docker_controls = StatefulList::new(DockerControls::gen_vec(state));
        docker_controls.start();
        let mut logs = StatefulList::new(vec![]);
        logs.end();
        Self {
            cpu_stats: VecDeque::with_capacity(60),
            docker_controls,
            id,
            image,
            last_updated: 0,
            logs,
            mem_limit: ByteStats::default(),
            mem_stats: VecDeque::with_capacity(60),
            name,
            rx: ByteStats::default(),
            tx: ByteStats::default(),
            state,
            status,
        }
    }

    /// Find the max value in the cpu stats VecDeque
    fn max_cpu_stats(&self) -> CpuStats {
        match self.cpu_stats.iter().max() {
            Some(value) => *value,
            None => CpuStats::default(),
        }
    }

    /// Find the max value in the mem stats VecDeque
    fn max_mem_stats(&self) -> ByteStats {
        match self.mem_stats.iter().max() {
            Some(value) => *value,
            None => ByteStats::default(),
        }
    }

    /// Convert cpu stats into a vec for the charts function
    #[allow(clippy::cast_precision_loss)]
    fn get_cpu_dataset(&self) -> Vec<(f64, f64)> {
        self.cpu_stats
            .iter()
            .enumerate()
            .map(|i| (i.0 as f64, i.1.0 as f64))
            .collect::<Vec<_>>()
    }

    /// Convert mem stats into a Vec for the charts function
    #[allow(clippy::cast_precision_loss)]
    fn get_mem_dataset(&self) -> Vec<(f64, f64)> {
        self.mem_stats
            .iter()
            .enumerate()
            .map(|i| (i.0 as f64, i.1.0 as f64))
            .collect::<Vec<_>>()
    }

    /// Get all cpu chart data
    fn get_cpu_chart_data(&self) -> CpuTuple {
        (self.get_cpu_dataset(), self.max_cpu_stats(), self.state)
    }

    /// Get all mem chart data
    fn get_mem_chart_data(&self) -> MemTuple {
        (self.get_mem_dataset(), self.max_mem_stats(), self.state)
    }

    /// Get chart info for cpu & memory in one function
    /// So only need to call .lock() once
    pub fn get_chart_data(&self) -> (CpuTuple, MemTuple) {
        (self.get_cpu_chart_data(), self.get_mem_chart_data())
    }
}

/// Container information panel headings + widths, for nice pretty formatting
#[derive(Debug, Clone, Copy)]
pub struct Columns {
    pub state: (Header, usize),
    pub status: (Header, usize),
    pub cpu: (Header, usize),
    pub mem: (Header, usize),
    pub id: (Header, usize),
    pub name: (Header, usize),
    pub image: (Header, usize),
    pub net_rx: (Header, usize),
    pub net_tx: (Header, usize),
}

impl Columns {
    /// (Column titles, minimum header string length)
    pub const fn new() -> Self {
        Self {
            state: (Header::State, 11),
            status: (Header::Status, 16),
            // 7 to allow for "100.00%"
            cpu: (Header::Cpu, 7),
            mem: (Header::Memory, 12),
            id: (Header::Id, 8),
            name: (Header::Name, 4),
            image: (Header::Image, 5),
            net_rx: (Header::Rx, 5),
            net_tx: (Header::Tx, 5),
        }
    }
}
