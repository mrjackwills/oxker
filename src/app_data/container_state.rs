use std::{
    cmp::Ordering,
    collections::{HashSet, VecDeque},
    fmt,
    net::IpAddr,
};

use bollard::service::Port;
use jiff::{Timestamp, tz::TimeZone};
use ratatui::{
    layout::Size,
    style::Color,
    text::{Line, Text},
    widgets::ListState,
};

use crate::config::AppColors;

use super::Header;

const ONE_KB: f64 = 1000.0;
const ONE_MB: f64 = ONE_KB * 1000.0;
const ONE_GB: f64 = ONE_MB * 1000.0;

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub enum ScrollDirection {
    Next,
    Previous,
}

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub struct ContainerId(String);

impl From<&str> for ContainerId {
    fn from(x: &str) -> Self {
        Self(x.to_owned())
    }
}

impl ContainerId {
    pub const fn get(&self) -> &str {
        self.0.as_str()
    }

    /// Only return first 8 chars of id, is usually more than enough for uniqueness
    /// need to update tests to use real ids, or atleast strings of the correct-ish length
    pub fn get_short(&self) -> String {
        self.0.chars().take(8).collect::<String>()
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

pub trait Contains {
    fn contains(&self, input: &str) -> bool;
}
/// ContainerName and ContainerImage are simple structs, used so can implement custom fmt functions to them
macro_rules! unit_struct {
    ($name:ident) => {
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
        pub struct $name(String);

        impl From<String> for $name {
            fn from(value: String) -> Self {
                Self(value)
            }
        }

        #[cfg(test)]
        impl From<&str> for $name {
            fn from(value: &str) -> Self {
                Self(value.to_owned())
            }
        }

        impl $name {
            pub const fn get(&self) -> &str {
                self.0.as_str()
            }

            pub fn set(&mut self, value: String) {
                self.0 = value;
            }
        }

        impl Contains for $name {
            fn contains(&self, input: &str) -> bool {
                self.0.to_lowercase().contains(input)
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                if self.0.chars().count() >= 30 {
                    write!(f, "{}…", self.0.chars().take(29).collect::<String>())
                } else {
                    write!(f, "{}", self.0)
                }
            }
        }
    };
}

unit_struct!(ContainerName);
unit_struct!(ContainerImage);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ContainerPorts {
    pub ip: Option<IpAddr>,
    pub private: u16,
    pub public: Option<u16>,
}

impl From<Port> for ContainerPorts {
    fn from(value: Port) -> Self {
        Self {
            ip: value.ip.and_then(|i| i.parse::<IpAddr>().ok()),
            private: value.private_port,
            public: value.public_port,
        }
    }
}

impl ContainerPorts {
    pub fn len_ip(&self) -> usize {
        self.ip
            .as_ref()
            .map_or(0, |i| i.to_string().chars().count())
    }
    pub fn len_private(&self) -> usize {
        format!("{}", self.private).chars().count()
    }
    pub fn len_public(&self) -> usize {
        format!("{}", self.public.unwrap_or_default())
            .chars()
            .count()
    }

    /// Return as tuple of Strings, ip address, private port, and public port
    pub fn get_all(&self) -> (String, String, String) {
        (
            self.ip
                .as_ref()
                .map_or(String::new(), std::string::ToString::to_string),
            format!("{}", self.private),
            self.public.map_or(String::new(), |s| s.to_string()),
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
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

    pub fn scroll(&mut self, scroll: &ScrollDirection) {
        match scroll {
            ScrollDirection::Next => self.next(),
            ScrollDirection::Previous => self.previous(),
        }
    }

    fn next(&mut self) {
        if !self.items.is_empty() {
            self.state.select(Some(
                self.state.selected().map_or(
                    0,
                    |i| {
                        if i < self.items.len() - 1 { i + 1 } else { i }
                    },
                ),
            ));
        }
    }

    fn previous(&mut self) {
        if !self.items.is_empty() {
            self.state.select(Some(
                self.state
                    .selected()
                    .map_or(0, |i| if i == 0 { 0 } else { i - 1 }),
            ));
        }
    }

    /// Return the current status of the select list, e.g. 2/5,
    /// MAYBE add up down arrows, check if at start or end etc
    pub fn get_state_title(&self) -> String {
        if self.items.is_empty() {
            String::new()
        } else {
            let len = self.items.len();
            let count = self
                .state
                .selected()
                .map_or(0, |value| if len > 0 { value + 1 } else { value });
            format!(" {count}/{len}")
        }
    }
}

/// Store the containers status in a  struct, so can then check for healthy/unhealthy status
/// It's usually something like "Up 1 hour", "Exited (0) 10 hours ago", "Up 10 minutes (unhealthy)"
#[derive(Debug, Clone, Eq, PartialEq, PartialOrd)]
pub struct ContainerStatus(String);

impl From<String> for ContainerStatus {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl ContainerStatus {
    /// Check if a container is unhealthy
    pub fn unhealthy(&self) -> bool {
        self.contains("(unhealthy)")
    }

    /// Get a reference to the source string
    pub const fn get(&self) -> &String {
        &self.0
    }
}

impl Contains for ContainerStatus {
    /// Check if the state contains a specific string
    fn contains(&self, item: &str) -> bool {
        self.0.to_lowercase().contains(item)
    }
}

/// By default a container's running status will be healthy
#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd)]
pub enum RunningState {
    Healthy,
    Unhealthy,
}
/// States of the container
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum State {
    Dead,
    Exited,
    Paused,
    Removing,
    Restarting,
    Running(RunningState),
    Unknown,
}

impl State {
    /// The container is alive if the start is Running, either healthy or unhealthy
    pub const fn is_alive(self) -> bool {
        matches!(self, Self::Running(_))
    }

    /// Check if state is running & healthy
    pub const fn is_healthy(self) -> bool {
        match self {
            Self::Running(x) => match x {
                RunningState::Healthy => true,
                RunningState::Unhealthy => false,
            },
            _ => false,
        }
    }
    /// Color of the state for the containers section
    pub const fn get_color(self, colors: AppColors) -> Color {
        match self {
            Self::Dead => colors.container_state.dead,
            Self::Exited => colors.container_state.exited,
            Self::Paused => colors.container_state.paused,
            Self::Removing => colors.container_state.removing,
            Self::Restarting => colors.container_state.restarting,
            Self::Running(RunningState::Healthy) => colors.container_state.running_healthy,
            Self::Running(RunningState::Unhealthy) => colors.container_state.running_unhealthy,
            Self::Unknown => colors.container_state.unknown,
        }
    }
    /// Dirty way to create order for the state, rather than impl Ord
    pub const fn order(self) -> u8 {
        match self {
            Self::Running(RunningState::Healthy) => 0,
            Self::Running(RunningState::Unhealthy) => 1,
            Self::Paused => 2,
            Self::Restarting => 3,
            Self::Removing => 4,
            Self::Exited => 5,
            Self::Dead => 6,
            Self::Unknown => 7,
        }
    }
}

/// Need status, to check if container is unhealthy or not
impl From<(&str, &ContainerStatus)> for State {
    fn from((input, status): (&str, &ContainerStatus)) -> Self {
        match input {
            "dead" => Self::Dead,
            "exited" => Self::Exited,
            "paused" => Self::Paused,
            "removing" => Self::Removing,
            "restarting" => Self::Restarting,
            "running" => {
                if status.unhealthy() {
                    Self::Running(RunningState::Unhealthy)
                } else {
                    Self::Running(RunningState::Healthy)
                }
            }
            _ => Self::Unknown,
        }
    }
}

/// Need status, to check if container is unhealthy or not
impl
    From<(
        &bollard::secret::ContainerSummaryStateEnum,
        &ContainerStatus,
    )> for State
{
    fn from(
        (input, status): (
            &bollard::secret::ContainerSummaryStateEnum,
            &ContainerStatus,
        ),
    ) -> Self {
        match input {
            bollard::secret::ContainerSummaryStateEnum::DEAD => Self::Dead,
            bollard::secret::ContainerSummaryStateEnum::EXITED => Self::Exited,
            bollard::secret::ContainerSummaryStateEnum::PAUSED => Self::Paused,
            bollard::secret::ContainerSummaryStateEnum::REMOVING => Self::Removing,
            bollard::secret::ContainerSummaryStateEnum::RESTARTING => Self::Restarting,
            bollard::secret::ContainerSummaryStateEnum::RUNNING => {
                if status.unhealthy() {
                    Self::Running(RunningState::Unhealthy)
                } else {
                    Self::Running(RunningState::Healthy)
                }
            }
            _ => Self::Unknown,
        }
    }
}

/// Again, need status, to check if container is unhealthy or not
impl
    From<(
        Option<&bollard::secret::ContainerSummaryStateEnum>,
        &ContainerStatus,
    )> for State
{
    fn from(
        (input, status): (
            Option<&bollard::secret::ContainerSummaryStateEnum>,
            &ContainerStatus,
        ),
    ) -> Self {
        input.map_or(Self::Unknown, |input| Self::from((input, status)))
    }
}

/// Again, need status, to check if container is unhealthy or not
impl From<(Option<String>, &ContainerStatus)> for State {
    fn from((input, status): (Option<String>, &ContainerStatus)) -> Self {
        input.map_or(Self::Unknown, |input| Self::from((input.as_str(), status)))
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
            Self::Running(RunningState::Healthy) => "✓ running",
            Self::Running(RunningState::Unhealthy) => "! running",
            Self::Unknown => "? unknown",
        };
        write!(f, "{disp}")
    }
}

/// Items for the container control list
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DockerCommand {
    Pause,
    Restart,
    Start,
    Stop,
    Resume,
    Delete,
}

impl DockerCommand {
    pub const fn get_color(self, colors: AppColors) -> Color {
        match self {
            Self::Pause => colors.commands.pause,
            Self::Restart => colors.commands.restart,
            Self::Start => colors.commands.start,
            Self::Stop => colors.commands.stop,
            Self::Delete => colors.commands.delete,
            Self::Resume => colors.commands.resume,
        }
    }

    /// Docker commands available depending on the containers state
    pub fn gen_vec(state: State) -> Vec<Self> {
        match state {
            State::Dead | State::Exited => vec![Self::Start, Self::Restart, Self::Delete],
            State::Paused => vec![Self::Resume, Self::Stop, Self::Delete],
            State::Restarting => vec![Self::Stop, Self::Delete],
            State::Running(_) => vec![Self::Pause, Self::Restart, Self::Stop, Self::Delete],
            _ => vec![Self::Delete],
        }
    }
}

impl fmt::Display for DockerCommand {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let disp = match self {
            Self::Pause => "pause",
            Self::Delete => "delete",
            Self::Restart => "restart",
            Self::Start => "start",
            Self::Stop => "stop",
            Self::Resume => "resume",
        };
        write!(f, "{disp}")
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
        Some(self.cmp(other))
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
        write!(f, "{disp:>x$}", x = f.width().unwrap_or(1))
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
        Some(self.cmp(other))
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
    pub const fn update(&mut self, value: u64) {
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
            _ => format!("{y:.2} kB", y = as_f64 / ONE_KB),
        };
        write!(f, "{p:>x$}", x = f.width().unwrap_or(1))
    }
}

pub type MemTuple = (Vec<(f64, f64)>, ByteStats, State);
pub type CpuTuple = (Vec<(f64, f64)>, CpuStats, State);

/// Used to make sure that each log entry, for each container, is unique,
/// will only push a log entry into the logs vec if timestamp of said log entry isn't in the hashset
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct LogsTz(String);

impl fmt::Display for LogsTz {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl LogsTz {
    /// With a given &str, split into a logtz and content, so that we only need to `use split_once()` once
    /// The docker log, which should always contain a timestamp, is in the format `2023-01-14T19:13:30.783138328Z Lorem ipsum dolor sit amet`
    pub fn splitter(input: &str) -> (Self, String) {
        let (tz, content) = input.split_once(' ').unwrap_or_default();
        (Self(tz.to_owned()), content.to_owned())
    }

    /// Display the timestamp in a given format, and if provided, with a timezone offset
    pub fn display_with_formatter(&self, tz: Option<&TimeZone>, format: &str) -> Option<String> {
        self.0.parse::<Timestamp>().map_or(None, |t| {
            if let Some(tz) = tz.as_ref() {
                let tz = tz.iana_name()?;
                let z = t.in_tz(tz).ok()?;
                Some(z.strftime(format).to_string())
            } else {
                Some(t.strftime(format).to_string())
            }
        })
    }
}

/// Store the logs alongside a HashSet, each log *should* generate a unique timestamp,
/// so if we store the timestamp separately in a HashSet, we can then check if we should insert a log line into the
/// stateful list dependent on whether the timestamp is in the HashSet or not
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Logs {
    lines: StatefulList<Text<'static>>,
    tz: HashSet<LogsTz>,
    search_results: Vec<usize>,
    search_term: Option<String>,
    offset: u16,
    max_log_len: usize,
    adjusted_max_width: usize,
    adjust_max_width_text_len: usize,
}

impl Default for Logs {
    fn default() -> Self {
        let mut lines = StatefulList::new(vec![]);
        lines.end();
        Self {
            lines,
            tz: HashSet::new(),
            offset: 0,
            search_term: None,
            search_results: vec![],
            adjusted_max_width: 0,
            adjust_max_width_text_len: 0,
            max_log_len: 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub enum LogsButton {
    Both,
    Next,
    Previous,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Hash)]
pub struct LogSearch {
    pub term: Option<String>,
    pub result: Option<String>,
    pub buttons: Option<LogsButton>,
}

/// LogSearch is used in FrameData
impl From<&Logs> for LogSearch {
    fn from(l: &Logs) -> Self {
        let buttons = l.lines.state.selected().as_ref().and_then(|x| {
            let show_next = l.search_results.iter().any(|n| n > x);
            let show_previous = l.search_results.iter().any(|n| n < x);
            match (show_next, show_previous) {
                (true, true) => Some(LogsButton::Both),
                (true, false) => Some(LogsButton::Next),
                (false, true) => Some(LogsButton::Previous),
                (false, false) => None,
            }
        });
        Self {
            term: l.search_term.clone(),
            result: l.get_search_result(),
            buttons,
        }
    }
}

impl Logs {
    pub fn gen_log_search(&self) -> LogSearch {
        LogSearch::from(self)
    }

    /// Scroll to the next or previous search result, accounts for when currently selected line isn't in the results vec
    pub fn search_scroll(&mut self, sd: &ScrollDirection) -> Option<()> {
        if let Some(current_selected) = self.lines.state.selected() {
            if let Some(current_position) = self
                .search_results
                .iter()
                .position(|i| i == &current_selected)
            {
                if let Some(new_index) = match sd {
                    ScrollDirection::Next => current_position.checked_add(1),
                    ScrollDirection::Previous => current_position.checked_sub(1),
                } {
                    if let Some(f) = self.search_results.get(new_index) {
                        self.lines.state.select(Some(*f));
                        return Some(());
                    }
                }
            } else {
                let range = match sd {
                    ScrollDirection::Previous => (0..=current_selected).rev().collect::<Vec<_>>(),
                    ScrollDirection::Next => (current_selected
                        ..=self
                            .search_results
                            .last()
                            .map_or_else(|| current_selected, |i| *i))
                        .collect::<Vec<_>>(),
                };
                for i in range {
                    if self.search_results.contains(&i) {
                        self.lines.state.select(Some(i));
                        return Some(());
                    }
                }
            }
        }
        None
    }

    /// Get a string x/y, where y is total matches found, and x is current ordered selected line
    /// WIll be padded by max chars of total matches
    fn get_search_result(&self) -> Option<String> {
        if self.search_results.is_empty() {
            return None;
        }
        Some(self.lines.state.selected().map_or_else(
            || format!("{}", self.search_results.len()),
            |current_index| {
                self.search_results
                    .iter()
                    .position(|i| i == &current_index)
                    .map_or_else(
                        || format!("{}", self.search_results.len()),
                        |index| {
                            let len = format!("{}", self.search_results.len());
                            let len_width = len.chars().count();
                            format!("{:>len_width$}/{len:>len_width$}", index + 1)
                        },
                    )
            },
        ))
    }

    /// Search through the logs for a matching string
    pub fn search(&mut self, case_sensitive: bool, scroll: bool) {
        if let Some(search_term) = self.search_term.as_ref() {
            let term = if case_sensitive {
                search_term.to_owned()
            } else {
                search_term.to_lowercase()
            };
            self.search_results = self
                .lines
                .items
                .iter()
                .enumerate()
                .filter_map(|(index, a)| {
                    a.lines
                        .iter()
                        .any(|b| {
                            b.spans.iter().any(|c| {
                                if case_sensitive {
                                    c.content.contains(&term)
                                } else {
                                    c.content.to_lowercase().contains(&term)
                                }
                            })
                        })
                        .then_some(index)
                })
                .collect();
            if !self.search_results.is_empty() && scroll {
                self.lines.state.select(self.search_results.last().copied());
                self.offset = 0;
            }
        } else {
            self.search_results.clear();
        }
    }

    /// Set a single char into the filter term
    pub fn search_term_push(&mut self, c: char, case_sensitive: bool) {
        if let Some(term) = self.search_term.as_mut() {
            term.push(c);
        } else {
            self.search_term = Some(format!("{c}"));
        }
        self.search(case_sensitive, true);
    }

    /// Delete the final char of the filter term
    pub fn search_term_pop(&mut self, case_sensitive: bool) {
        if let Some(term) = self.search_term.as_mut() {
            term.pop();
            if term.is_empty() {
                self.search_term = None;
            }
        }
        self.search(case_sensitive, true);
    }

    /// Remove the filter completely
    pub fn search_term_clear(&mut self) {
        self.search_term = None;
        self.search_results.clear();
    }

    /// Only allow a new log line to be inserted if the log timestamp isn't in the tz HashSet
    pub fn insert(&mut self, line: Text<'static>, tz: LogsTz, case_sensitive: bool) {
        if self.tz.insert(tz) {
            self.max_log_len = self.max_log_len.max(line.width());
            self.lines.items.push(line);
            // Maybe - Ideally we'd re-render here
            if self.search_term.is_some() {
                self.search(case_sensitive, false);
            }
        }
    }

    /// If scrolling horizontally along the logs, display a counter of the position in the in the scroll, `x/y`
    pub fn get_scroll_title(&mut self, width: u16) -> Option<String> {
        if self.horizontal_scroll_able(width) {
            let text_width = self.adjust_max_width_text_len;
            let arrow_left = if self.offset > 0 { " ←" } else { "  " };
            let arrow_right = if usize::from(self.offset) < self.adjusted_max_width {
                "→ "
            } else {
                "  "
            };
            Some(format!(
                "{left} {offset:>text_width$}/{adjusted_max_width} {right}",
                offset = self.offset,
                adjusted_max_width = self.adjusted_max_width,
                left = arrow_left,
                right = arrow_right,
            ))
        } else {
            None
        }
    }

    /// Format a log lone. Only return screen width amount of chars
    /// If offset set, remove `char_offset` number of chars from a Text
    /// `text` *should* only be a single line, so just use the .first() method rather than trying to iterate
    fn format_log_line(text: &Text<'static>, char_offset: usize, width: u16) -> Text<'static> {
        let mut skipped = 0;
        text.lines.first().map_or_else(Text::default, |line| {
            Text::from(Line::from(
                line.spans
                    .iter()
                    .filter_map(|span| {
                        if skipped >= char_offset {
                            Some(ratatui::text::Span::styled(
                                span.content.chars().take(width.into()).collect::<String>(),
                                span.style,
                            ))
                        } else {
                            let span_len = span.content.chars().count();
                            if skipped + span_len <= char_offset {
                                skipped += span_len;
                                None
                            } else {
                                let start_index = char_offset - skipped;
                                skipped = char_offset;
                                Some(ratatui::text::Span::styled(
                                    span.content
                                        .chars()
                                        .skip(start_index)
                                        .take(width.into())
                                        .collect::<String>(),
                                    span.style,
                                ))
                            }
                        }
                    })
                    .collect::<Vec<_>>(),
            ))
        })
    }

    /// Get the logs vec, but instead of cloning to whole vec, only clone items within x of the currently selected index, as well as only the current screen widths number of chars
    /// Where x is the abs different of the index plus the panel height & a padding
    /// Take into account the char offset, so that can scroll a line
    /// The rest can be just empty list items
    pub fn get_visible_logs(&self, size: Size, padding: usize) -> Vec<Text<'static>> {
        let current_index = self.lines.state.selected().unwrap_or_default();
        let height_padding = usize::from(size.height) + padding;
        let char_offset = if usize::from(self.offset) > self.max_log_len {
            self.max_log_len
        } else {
            self.offset.into()
        };

        self.lines
            .items
            .iter()
            .enumerate()
            .map(|(index, item)| {
                if current_index.abs_diff(index) <= height_padding {
                    Self::format_log_line(item, char_offset, size.width)
                } else {
                    Text::from("")
                }
            })
            .collect()
    }

    /// The rest of the methods are basically forwarding from the underlying StatefulList
    pub fn get_state_title(&self) -> String {
        self.lines.get_state_title()
    }

    /// Return true it currently selected container logs are wide enough to horizontally scroll
    pub fn horizontal_scroll_able(&mut self, width: u16) -> bool {
        if self.lines.items.is_empty() {
            return false;
        }
        self.adjusted_max_width = self.max_log_len.saturating_sub(width.into()) + 4;
        self.adjust_max_width_text_len = self.adjusted_max_width.to_string().chars().count();
        self.max_log_len + 4 > usize::from(width)
    }

    /// Add a padding so one char will always be visilbe?
    pub fn forward(&mut self, width: u16) {
        let offset = usize::from(self.offset);
        if self.horizontal_scroll_able(width) {
            if self.adjusted_max_width > 0 && offset < self.adjusted_max_width {
                self.offset = self.offset.saturating_add(1);
            }
        }
    }

    /// Reduce the char offset
    pub const fn back(&mut self) {
        self.offset = self.offset.saturating_sub(1);
    }

    /// Scroll lines down by one
    pub fn next(&mut self) {
        self.lines.next();
    }

    /// Scroll lines up by one
    pub fn previous(&mut self) {
        self.lines.previous();
    }

    /// Go to the end of the lines
    pub fn end(&mut self) {
        self.lines.end();
    }

    /// Go to the start of the lines
    pub fn start(&mut self) {
        self.lines.start();
    }

    /// Get total number of log lines
    pub const fn len(&self) -> usize {
        self.lines.items.len()
    }

    pub const fn state(&mut self) -> &mut ListState {
        &mut self.lines.state
    }
}

/// Info for each container
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContainerItem {
    pub cpu_stats: VecDeque<CpuStats>,
    pub created: u64,
    pub docker_controls: StatefulList<DockerCommand>,
    pub id: ContainerId,
    pub image: ContainerImage,
    pub is_oxker: bool,
    pub last_updated: u64,
    pub logs: Logs,
    pub mem_limit: ByteStats,
    pub mem_stats: VecDeque<ByteStats>,
    pub name: ContainerName,
    pub ports: Vec<ContainerPorts>,
    pub rx: ByteStats,
    pub state: State,
    pub status: ContainerStatus,
    pub tx: ByteStats,
}

/// Basic display information, for when running in debug mode
impl fmt::Display for ContainerItem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}, {}, {}, {}",
            self.id.get_short(),
            self.name,
            self.cpu_stats.back().unwrap_or(&CpuStats::new(0.0)),
            self.mem_stats.back().unwrap_or(&ByteStats::new(0))
        )
    }
}

impl ContainerItem {
    #[allow(clippy::too_many_arguments)]
    /// Create a new container item
    pub fn new(
        created: u64,
        id: ContainerId,
        image: String,
        is_oxker: bool,
        name: String,
        ports: Vec<ContainerPorts>,
        state: State,
        status: ContainerStatus,
    ) -> Self {
        let mut docker_controls = StatefulList::new(DockerCommand::gen_vec(state));
        docker_controls.start();

        Self {
            cpu_stats: VecDeque::with_capacity(60),
            created,
            docker_controls,
            id,
            image: image.into(),
            is_oxker,
            last_updated: 0,
            logs: Logs::default(),
            mem_limit: ByteStats::default(),
            mem_stats: VecDeque::with_capacity(60),
            name: name.into(),
            ports,
            rx: ByteStats::default(),
            state,
            status,
            tx: ByteStats::default(),
        }
    }

    /// Find the max value in the cpu stats VecDeque
    fn max_cpu_stats(&self) -> CpuStats {
        self.cpu_stats
            .iter()
            .max()
            .map_or_else(CpuStats::default, |value| *value)
    }

    /// Find the max value in the mem stats VecDeque
    fn max_mem_stats(&self) -> ByteStats {
        self.mem_stats
            .iter()
            .max()
            .map_or_else(ByteStats::default, |value| *value)
    }

    /// Convert cpu stats into a vec for the charts function
    #[allow(clippy::cast_precision_loss)]
    fn get_cpu_dataset(&self) -> Vec<(f64, f64)> {
        self.cpu_stats
            .iter()
            .enumerate()
            .map(|i| (i.0 as f64, i.1.0))
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Columns {
    pub name: (Header, u8),
    pub state: (Header, u8),
    pub status: (Header, u8),
    pub cpu: (Header, u8),
    pub mem: (Header, u8, u8),
    pub id: (Header, u8),
    pub image: (Header, u8),
    pub net_rx: (Header, u8),
    pub net_tx: (Header, u8),
}

impl Columns {
    /// (Column titles, minimum header string length)
    pub const fn new() -> Self {
        Self {
            name: (Header::Name, 4),
            state: (Header::State, 5),
            status: (Header::Status, 6),
            cpu: (Header::Cpu, 3),
            mem: (Header::Memory, 7, 7),
            id: (Header::Id, 8),
            image: (Header::Image, 5),
            net_rx: (Header::Rx, 4),
            net_tx: (Header::Tx, 4),
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {

    use jiff::tz::TimeZone;
    use ratatui::{
        layout::Size,
        text::{Line, Text},
    };

    use crate::{
        app_data::{ContainerImage, LogSearch, Logs, LogsTz, RunningState},
        ui::log_sanitizer,
    };

    use super::{ByteStats, ContainerName, ContainerStatus, CpuStats, State};

    #[test]
    /// Display CpuStats as a string
    fn test_container_state_cpustats_to_string() {
        let test = |f: f64, s: &str| {
            assert_eq!(CpuStats::new(f).to_string(), s);
        };

        test(0.0, "00.00%");
        test(1.5, "01.50%");
        test(15.15, "15.15%");
        test(150.15, "150.15%");
    }

    #[test]
    /// Display bytestats as a string, convert into correct data unit (Kb, MB, GB)
    fn test_container_state_bytestats_to_string() {
        let test = |u: u64, s: &str| {
            assert_eq!(ByteStats::new(u).to_string(), s);
        };

        test(0, "0.00 kB");
        test(150, "0.15 kB");
        test(1500, "1.50 kB");
        test(150_000, "150.00 kB");
        test(1_500_000, "1.50 MB");
        test(15_000_000, "15.00 MB");
        test(150_000_000, "150.00 MB");
        test(1_500_000_000, "1.50 GB");
        test(15_000_000_000, "15.00 GB");
        test(150_000_000_000, "150.00 GB");
    }

    #[test]
    /// ContainerName as string truncated correctly
    fn test_container_state_container_name_to_string() {
        let result = ContainerName::from("name_01");
        assert_eq!(result.to_string(), "name_01");

        let result = ContainerName::from("name_01_name_01_name_01_name_01_");
        assert_eq!(result.to_string(), "name_01_name_01_name_01_name_…");

        let result = result.get();
        assert_eq!(result, "name_01_name_01_name_01_name_01_");
    }

    #[test]
    /// ContainerImage as string truncated correctly
    fn test_container_state_container_image() {
        let result = ContainerImage::from("name_01");
        assert_eq!(result.to_string(), "name_01");

        let result = ContainerImage::from("name_01_name_01_name_01_name_01_");
        assert_eq!(result.to_string(), "name_01_name_01_name_01_name_…");

        let result = result.get();
        assert_eq!(result, "name_01_name_01_name_01_name_01_");
    }

    #[test]
    /// LogzTz correctly splits a line by timestamp
    fn test_container_state_logz_splitter() {
        let input = "2023-01-14T12:01:20.012345678Z Lorem ipsum dolor sit amet";
        let log_tz = LogsTz::splitter(input);

        assert_eq!(
            log_tz.0,
            super::LogsTz("2023-01-14T12:01:20.012345678Z".to_owned())
        );
        assert_eq!(log_tz.1, "Lorem ipsum dolor sit amet");
    }

    #[test]
    /// LogsTz display correctly formats with a given timestamp string
    fn test_container_state_logz_display() {
        let input = "2023-01-14T12:01:20.012345678Z Lorem ipsum dolor sit amet";
        let log_tz = LogsTz::splitter(input);

        let result = log_tz
            .0
            .display_with_formatter(None, "%Y-%m-%dT%H:%M:%S.%8f");
        assert!(result.is_some());
        let result = result.unwrap();
        assert_eq!(result, "2023-01-14T12:01:20.01234567");

        let result = log_tz.0.display_with_formatter(None, "%Y-%m-%d %H:%M:%S");
        assert!(result.is_some());
        let result = result.unwrap();
        assert_eq!(result, "2023-01-14 12:01:20");

        let result = log_tz.0.display_with_formatter(None, "%Y-%j");
        assert!(result.is_some());
        let result = result.unwrap();

        assert_eq!(result, "2023-014");
    }

    #[test]
    /// LogsTz display correctly formats with a given timestamp string & timezone
    fn test_container_state_logz_display_with_timezone() {
        let input = "2023-01-14T12:01:20.012345678Z Lorem ipsum dolor sit amet";
        let log_tz = LogsTz::splitter(input);

        let timezone = Some(TimeZone::get("Asia/Tokyo").unwrap());
        let result = log_tz
            .0
            .display_with_formatter(timezone.as_ref(), "%Y-%m-%dT%H:%M:%S.%8f");
        assert!(result.is_some());
        let result = result.unwrap();
        assert_eq!(result, "2023-01-14T21:01:20.01234567");

        let result = log_tz
            .0
            .display_with_formatter(timezone.as_ref(), "%Y-%m-%d %H:%M:%S");
        assert!(result.is_some());
        let result = result.unwrap();
        assert_eq!(result, "2023-01-14 21:01:20");

        let result = log_tz.0.display_with_formatter(timezone.as_ref(), "%Y-%j");
        assert!(result.is_some());
        let result = result.unwrap();
        assert_eq!(result, "2023-014");
    }

    #[test]
    /// Logs can only contain 1 entry per LogzTz
    fn test_container_state_logz() {
        let input = "2023-01-14T19:13:30.783138328Z Lorem ipsum dolor sit amet";
        let (tz, _) = LogsTz::splitter(input);
        let mut logs = Logs::default();
        let line = log_sanitizer::remove_ansi(input);

        logs.insert(Text::from(line.clone()), tz.clone(), true);
        logs.insert(Text::from(line.clone()), tz.clone(), true);
        logs.insert(Text::from(line), tz, true);

        assert_eq!(logs.lines.items.len(), 1);

        let input = "2023-01-15T19:13:30.783138328Z Lorem ipsum dolor sit amet";
        let (tz, _) = LogsTz::splitter(input);
        let line = log_sanitizer::remove_ansi(input);

        logs.insert(Text::from(line.clone()), tz.clone(), true);
        logs.insert(Text::from(line.clone()), tz.clone(), true);
        logs.insert(Text::from(line), tz, true);

        assert_eq!(logs.lines.items.len(), 2);
    }

    #[test]
    /// check ContainerStatus unhealthy state
    fn test_container_state_unhealthy() {
        let input = ContainerStatus::from("Up 1 hour".to_owned());

        assert!(!input.unhealthy());

        let input = ContainerStatus::from("Up 1 hour (unhealthy)".to_owned());

        assert!(input.unhealthy());
    }

    #[test]
    /// Generate container State from a &str and &ContainerStatus
    fn test_container_status_unhealthy() {
        let healthy = ContainerStatus::from("Up 1 hour".to_owned());
        let unhealthy = ContainerStatus::from("Up 1 hour (unhealthy)".to_owned());

        // Running and healthy
        let input = State::from(("running", &healthy));
        assert_eq!(input, State::Running(RunningState::Healthy));

        // Running and unhealthy
        let input = State::from(("running", &unhealthy));
        assert_eq!(input, State::Running(RunningState::Unhealthy));

        // Dead
        let input = State::from(("dead", &healthy));
        assert_eq!(input, State::Dead);

        // Exited
        let input = State::from(("exited", &healthy));
        assert_eq!(input, State::Exited);

        // Paused
        let input = State::from(("paused", &healthy));
        assert_eq!(input, State::Paused);

        // Removing
        let input = State::from(("removing", &healthy));
        assert_eq!(input, State::Removing);

        // Restarting
        let input = State::from(("restarting", &healthy));
        assert_eq!(input, State::Restarting);

        // Unknown
        let input = State::from(("oxker", &healthy));
        assert_eq!(input, State::Unknown);
    }

    #[test]
    /// Test the format_log_line methods, should ideally check colours are being correct kept as well
    fn test_to_vec() {
        let mut logs = Logs::default();

        let input = "2023-01-14T19:13:30.783138328Z Hello world some long line".to_owned();
        let (tz, _) = LogsTz::splitter(&input);
        logs.insert(Text::from(input), tz, true);

        let input = "2023-01-14T19:13:31.783138328Z Hello world some line".to_owned();
        let (tz, _) = LogsTz::splitter(&input);
        logs.insert(Text::from(input), tz, true);

        let input = "2023-01-14T19:13:32.783138328Z Hello world".to_owned();
        let (tz, _) = LogsTz::splitter(&input);
        logs.insert(Text::from(input), tz, true);

        logs.offset = 43;
        let result = logs.get_visible_logs(
            Size {
                width: 14,
                height: 10,
            },
            10,
        );
        assert_eq!(
            vec![
                Text::from(Line::from("some long line")),
                Text::from(Line::from("some line")),
                Text::from(Line::default())
            ],
            result
        );
    }

    #[test]
    /// Test the get_scroll_title methods
    fn test_scroll_title() {
        let mut logs = Logs::default();

        let result = logs.get_scroll_title(10);
        assert!(result.is_none());

        let input = "short".to_owned();
        let (tz, _) = LogsTz::splitter(&input);
        logs.insert(Text::from(input), tz, true);

        let result = logs.get_scroll_title(10);
        assert!(result.is_none());

        let input = "2023-01-14T19:13:30.783138328Z Hello world some long line".to_owned();
        let (tz, _) = LogsTz::splitter(&input);
        logs.insert(Text::from(input), tz, true);

        let result = logs.get_scroll_title(10);
        assert_eq!(result, Some("    0/51 → ".to_owned()));

        logs.forward(10);

        let result = logs.get_scroll_title(10);
        assert_eq!(result, Some(" ←  1/51 → ".to_owned()));

        for _ in 0..=49 {
            logs.forward(10);
        }
        let result = logs.get_scroll_title(10);
        assert_eq!(result, Some(" ← 51/51   ".to_owned()));
    }

    #[test]
    /// Test the log search
    fn test_logsearch() {
        let mut logs = Logs::default();

        for i in 1..=10 {
            let input = if i % 2 == 0 {
                format!("{i}, hello world some long line {i}")
            } else {
                format!("{i}, Hello world some long line {i}")
            };
            let (tz, _) = LogsTz::splitter(&input);
            logs.insert(Text::from(input), tz, true);
        }

        logs.search_term_push('H', true);
        assert_eq!(logs.search_results, [0, 2, 4, 6, 8]);
        logs.search_term_clear();
        logs.search_term_push('H', false);
        assert_eq!(logs.search_results, [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
    }

    #[test]
    /// Test the LogSearch::From() methods
    fn test_logsearch_from() {
        let mut logs = Logs::default();

        for i in 1..=10 {
            let input = format!("{i}, Hello world some long line {i}");
            let (tz, _) = LogsTz::splitter(&input);
            logs.insert(Text::from(input), tz, true);
        }

        let log_search = LogSearch::from(&logs);
        assert_eq!(
            log_search,
            LogSearch {
                term: None,
                result: None,
                buttons: None
            }
        );

        logs.search_term_push('H', true);
        let log_search = LogSearch::from(&logs);
        assert_eq!(
            log_search,
            LogSearch {
                term: Some("H".to_owned()),
                result: Some("10/10".to_owned()),
                buttons: Some(crate::app_data::LogsButton::Previous)
            }
        );

        logs.previous();

        let log_search = LogSearch::from(&logs);
        assert_eq!(
            log_search,
            LogSearch {
                term: Some("H".to_owned()),
                result: Some(" 9/10".to_owned()),
                buttons: Some(crate::app_data::LogsButton::Both)
            }
        );

        logs.start();

        let log_search = LogSearch::from(&logs);
        assert_eq!(
            log_search,
            LogSearch {
                term: Some("H".to_owned()),
                result: Some(" 1/10".to_owned()),
                buttons: Some(crate::app_data::LogsButton::Next)
            }
        );

        logs.search_term_push('H', true);
        let log_search = LogSearch::from(&logs);
        assert_eq!(
            log_search,
            LogSearch {
                term: Some("HH".to_owned()),
                result: None,
                buttons: None
            }
        );

        logs.search_term_clear();
        logs.search_term_push('2', true);
        let log_search = LogSearch::from(&logs);
        assert_eq!(logs.lines.state.selected(), Some(1));
        assert_eq!(
            log_search,
            LogSearch {
                term: Some("2".to_owned()),
                result: Some("1/1".to_owned()),
                buttons: None
            }
        );

        logs.next();

        let log_search = LogSearch::from(&logs);
        assert_eq!(
            log_search,
            LogSearch {
                term: Some("2".to_owned()),
                result: Some("1".to_owned()),
                buttons: Some(crate::app_data::LogsButton::Previous)
            }
        );
    }
}
