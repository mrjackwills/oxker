use std::{cmp::Ordering, collections::VecDeque, fmt};

use tui::{
    style::Color,
    widgets::{ListItem, ListState},
};

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

    pub fn get_state_title(&self) -> String {
        if self.items.is_empty() {
            String::from("")
        } else {
            let len = self.items.len();
            let c = if let Some(value) = self.state.selected() {
                if len > 0 {
                    value + 1
                } else {
                    value
                }
            } else {
                0
            };
            format!("{}/{}", c, self.items.len())
        }
    }
}

/// States of the container
// / impl ord
#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub enum State {
    Dead,
    Exited,
    Paused,
    Removing,
    Restarting,
    Running,
    Unknown,
}

// impl Ord for State {
//     fn cmp(&self, other: &Self) -> Ordering {
//         match (self, other) {
// 			(Self::Dead)
//             // (_, Foo::B) => Ordering::Less,
//             // (Foo::A { val: l }, Foo::A { val: r }) => l.cmp(&r),
//             // (Foo::B, _) => Ordering::Greater,
//         }
//     }
// }

impl State {
    pub fn get_color(&self) -> Color {
        match self {
            Self::Running => Color::Green,
            Self::Removing => Color::LightRed,
            Self::Restarting => Color::LightGreen,
            Self::Paused => Color::Yellow,
            _ => Color::Red,
        }
    }
	pub fn as_text(&self) -> &'static str {
			match self {
				Self::Dead => "dead",
				Self::Exited => "exited",
				Self::Paused => "paused",
				Self::Removing => "removing",
				Self::Restarting => "restarting",
				Self::Running => "running",
				Self::Unknown => "unknown",
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
#[derive(Debug, Clone)]
pub enum DockerControls {
    Pause,
    Unpause,
    Restart,
    Stop,
    Start,
}

impl DockerControls {
    pub fn get_color(&self) -> Color {
        match self {
            Self::Start => Color::Green,
            Self::Stop => Color::Red,
            Self::Restart => Color::Magenta,
            Self::Pause => Color::Yellow,
            Self::Unpause => Color::Blue,
        }
    }

    pub fn gen_vec(state: &State) -> Vec<Self> {
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
            Self::Unpause => "unpause",
            Self::Restart => "restart",
            Self::Stop => "stop",
            Self::Start => "start",
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
#[derive(Clone, Debug)]
pub struct CpuStats {
    value: f64,
}

impl CpuStats {
    pub fn new(value: f64) -> Self {
        Self { value }
    }
}

impl Eq for CpuStats {}

impl PartialEq for CpuStats {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl PartialOrd for CpuStats {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.value.partial_cmp(&other.value)
    }
}

impl Ord for CpuStats {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.value > other.value {
            Ordering::Greater
        } else if self.value == other.value {
            Ordering::Equal
        } else {
            Ordering::Less
        }
    }
}

impl Stats for CpuStats {
    fn get_value(&self) -> f64 {
        self.value
    }
}

impl fmt::Display for CpuStats {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let disp = format!("{:05.2}%", self.value);
        write!(f, "{:>x$}", disp, x = f.width().unwrap_or(1))
    }
}

/// Struct for frequently updated memory usage stats
/// So can use custom display formatter
/// Use trait Stats for use as generic in draw_chart function
#[derive(Clone, Debug, Eq)]
pub struct ByteStats {
    value: u64,
}

impl PartialEq for ByteStats {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl PartialOrd for ByteStats {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.value.partial_cmp(&other.value)
    }
}

impl Ord for ByteStats {
    fn cmp(&self, other: &Self) -> Ordering {
        self.value.cmp(&other.value)
    }
}

impl ByteStats {
    pub fn new(value: u64) -> Self {
        Self { value }
    }
    pub fn update(&mut self, value: u64) {
        self.value = value;
    }
}
impl Stats for ByteStats {
    fn get_value(&self) -> f64 {
        self.value as f64
    }
}

// convert from bytes to kB, MB, GB etc
impl fmt::Display for ByteStats {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let one_kb = 1000.0;
        let one_mb = one_kb * one_kb;
        let one_gb = one_mb * 1000.0;
        let as_f64 = self.value as f64;
        let p = match as_f64 {
            x if x >= one_gb => format!("{y:.2} GB", y = as_f64 / one_gb),
            x if x >= one_kb => format!("{y:.2} MB", y = as_f64 / one_mb),
            x if x >= one_mb => format!("{y:.2} kB", y = as_f64 / one_kb),
            _ => format!("{} B", self.value),
        };
        write!(f, "{:>x$}", p, x = f.width().unwrap_or(1))
    }
}

/// Info for each container
#[derive(Debug, Clone)]
pub struct ContainerItem {
    pub cpu_stats: VecDeque<CpuStats>,
    pub docker_controls: StatefulList<DockerControls>,
    pub id: String,
    pub image: String,
    pub last_updated: u64,
    pub logs: StatefulList<ListItem<'static>>,
    pub mem_limit: ByteStats,
    pub mem_stats: VecDeque<ByteStats>,
    pub name: String,
    pub net_rx: ByteStats,
    pub net_tx: ByteStats,
    pub state: State,
    pub status: String,
}

pub type MemTuple = (Vec<(f64, f64)>, ByteStats, State);
pub type CpuTuple = (Vec<(f64, f64)>, CpuStats, State);

impl ContainerItem {
    /// Create a new container item
    pub fn new(id: String, status: String, image: String, state: State, name: String) -> Self {
        let mut docker_controls = StatefulList::new(DockerControls::gen_vec(&state));
        docker_controls.start();
        Self {
            cpu_stats: VecDeque::with_capacity(60),
            docker_controls,
            id,
            image,
            last_updated: 0,
            logs: StatefulList::new(vec![]),
            mem_limit: ByteStats::new(0),
            mem_stats: VecDeque::with_capacity(60),
            name,
            net_rx: ByteStats::new(0),
            net_tx: ByteStats::new(0),
            state,
            status,
        }
    }

    /// Find the max value in the last 30 items in the cpu stats vec
    fn max_cpu_stats(&self) -> CpuStats {
        match self.cpu_stats.iter().max() {
            Some(value) => value.to_owned(),
            None => CpuStats::new(0.0),
        }
    }

    /// Find the max value in the last 30 items in the mem stats vec
    fn max_mem_stats(&self) -> ByteStats {
        match self.mem_stats.iter().max() {
            Some(value) => value.to_owned(),
            None => ByteStats::new(0),
        }
    }

    /// Convert cpu stats into a vec for the charts function
    fn get_cpu_dataset(&self) -> Vec<(f64, f64)> {
        self.cpu_stats
            .iter()
            .enumerate()
            .map(|i| (i.0 as f64, i.1.value))
            .collect::<Vec<_>>()
    }

    /// Convert mem stats into a vec for the charts function
    fn get_mem_dataset(&self) -> Vec<(f64, f64)> {
        self.mem_stats
            .iter()
            .enumerate()
            .map(|i| (i.0 as f64, i.1.value as f64))
            .collect::<Vec<_>>()
    }

    /// Get all cpu chart data
    fn get_cpu_chart_data(&self) -> CpuTuple {
        (
            self.get_cpu_dataset(),
            self.max_cpu_stats(),
            self.state.clone(),
        )
    }

    /// Get all mem chart data
    fn get_mem_chart_data(&self) -> MemTuple {
        (
            self.get_mem_dataset(),
            self.max_mem_stats(),
            self.state.clone(),
        )
    }

    /// Get chart info for cpu & memory in one function
    /// So only need to call .lock() once
    pub fn get_chart_data(&self) -> (CpuTuple, MemTuple) {
        (self.get_cpu_chart_data(), self.get_mem_chart_data())
    }
}

/// Container information panel headings + widths, for nice pretty formatting
#[derive(Debug)]
pub struct Columns {
    pub state: (String, usize),
    pub status: (String, usize),
    pub cpu: (String, usize),
    pub mem: (String, usize),
    pub id: (String, usize),
    pub name: (String, usize),
    pub image: (String, usize),
    pub net_rx: (String, usize),
    pub net_tx: (String, usize),
}

impl Columns {
    //. (Column titles, minimum header string length)
    pub fn new() -> Self {
        Self {
            state: (String::from("state"), 11),
            status: (String::from("status"), 16),
            // 7 to allow for "100.00%"
            cpu: (String::from("cpu"), 7),
            mem: (String::from("memory/limit"), 12),
            id: (String::from("id"), 8),
            name: (String::from("name"), 4),
            image: (String::from("image"), 5),
            net_rx: (String::from("↓ rx"), 5),
            net_tx: (String::from("↑ tx"), 5),
        }
    }
}
