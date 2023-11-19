use bollard::models::ContainerSummary;
use core::fmt;
use parking_lot::Mutex;
use ratatui::widgets::{ListItem, ListState};
use std::{
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

mod container_state;

use crate::{
    app_error::AppError,
    parse_args::CliArgs,
    ui::{log_sanitizer, GuiState, Status},
    ENTRY_POINT,
};
pub use container_state::*;

#[cfg(not(debug_assertions))]
/// Global app_state, stored in an Arc<Mutex>
#[derive(Debug, Clone)]
pub struct AppData {
    containers: StatefulList<ContainerItem>,
    error: Option<AppError>,
    sorted_by: Option<(Header, SortedOrder)>,
    pub args: CliArgs,
}

#[cfg(debug_assertions)]
/// Global app_state, stored in an Arc<Mutex>
#[derive(Debug, Clone)]
pub struct AppData {
    containers: StatefulList<ContainerItem>,
    error: Option<AppError>,
    sorted_by: Option<(Header, SortedOrder)>,
    debug_string: String,
    pub args: CliArgs,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum SortedOrder {
    Asc,
    Desc,
}

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub enum Header {
    State,
    Status,
    Cpu,
    Memory,
    Id,
    Name,
    Image,
    Rx,
    Tx,
}

/// Convert Header enum into strings to display
impl fmt::Display for Header {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let disp = match self {
            Self::State => "state",
            Self::Status => "status",
            Self::Cpu => "cpu",
            Self::Memory => "memory/limit",
            Self::Id => "id",
            Self::Name => "name",
            Self::Image => "image",
            Self::Rx => "↓ rx",
            Self::Tx => "↑ tx",
        };
        write!(f, "{disp:>x$}", x = f.width().unwrap_or(1))
    }
}

impl AppData {
    #[cfg(debug_assertions)]
    pub fn get_debug_string(&self) -> &str {
        &self.debug_string
    }

    #[cfg(debug_assertions)]
    #[allow(unused)]
    pub fn push_debug_string(&mut self, x: &str) {
        self.debug_string.push_str(x);
    }

    /// Change the sorted order, also set the selected container state to match new order
    fn set_sorted(&mut self, x: Option<(Header, SortedOrder)>) {
        self.sorted_by = x;
        self.sort_containers();
        self.containers
            .state
            .select(self.containers.items.iter().position(|i| {
                self.get_selected_container_id()
                    .map_or(false, |id| i.id == id)
            }));
    }

    /// Current time as unix timestamp
    #[allow(clippy::expect_used)]
    fn get_systemtime() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("In our known reality, this error should never occur")
            .as_secs()
    }

    /// Generate a default app_state
    #[cfg(not(debug_assertions))]
    pub fn default(args: CliArgs) -> Self {
        Self {
            args,
            containers: StatefulList::new(vec![]),
            error: None,
            sorted_by: None,
        }
    }

    /// Generate a default app_state
    #[cfg(debug_assertions)]
    pub fn default(args: CliArgs) -> Self {
        Self {
            args,
            containers: StatefulList::new(vec![]),
            error: None,
            sorted_by: None,
            debug_string: String::new(),
        }
    }

    /// Container sort related methods

    /// Remove the sorted header & order, and sort by default - created datetime
    pub fn reset_sorted(&mut self) {
        self.set_sorted(None);
    }

    /// Sort containers based on a given header, if headings match, and already ascending, remove sorting
    pub fn set_sort_by_header(&mut self, selected_header: Header) {
        let mut output = Some((selected_header, SortedOrder::Asc));
        if let Some((current_header, order)) = self.get_sorted() {
            if current_header == selected_header {
                match order {
                    SortedOrder::Desc => output = None,
                    SortedOrder::Asc => output = Some((selected_header, SortedOrder::Desc)),
                }
            }
        }
        self.set_sorted(output);
    }

    pub const fn get_sorted(&self) -> Option<(Header, SortedOrder)> {
        self.sorted_by
    }

    /// Sort the containers vec, based on a heading (and if clash, then by name), either ascending or descending,
    /// If not sort set, then sort by created time
    pub fn sort_containers(&mut self) {
        if let Some((head, ord)) = self.sorted_by {
            let sort_closure = |a: &ContainerItem, b: &ContainerItem| -> std::cmp::Ordering {
                match head {
                    Header::State => match ord {
                        SortedOrder::Asc => {
                            a.state.order().cmp(&b.state.order()).then_with(|| a.name.cmp(&b.name))
                        }
                        SortedOrder::Desc => {
                            b.state.order().cmp(&a.state.order()).then_with(|| b.name.cmp(&a.name))
                        }
                    },
                    Header::Status => match ord {
                        SortedOrder::Asc => {
                            a.status.cmp(&b.status).then_with(|| a.name.cmp(&b.name))
                        }
                        SortedOrder::Desc => {
                            b.status.cmp(&a.status).then_with(|| b.name.cmp(&a.name))
                        }
                    },
                    Header::Cpu => match ord {
                        SortedOrder::Asc => a
                            .cpu_stats
                            .back()
                            .cmp(&b.cpu_stats.back())
                            .then_with(|| a.name.cmp(&b.name)),
                        SortedOrder::Desc => b
                            .cpu_stats
                            .back()
                            .cmp(&a.cpu_stats.back())
                            .then_with(|| b.name.cmp(&a.name)),
                    },
                    Header::Memory => match ord {
                        SortedOrder::Asc => a
                            .mem_stats
                            .back()
                            .cmp(&b.mem_stats.back())
                            .then_with(|| a.name.cmp(&b.name)),
                        SortedOrder::Desc => b
                            .mem_stats
                            .back()
                            .cmp(&a.mem_stats.back())
                            .then_with(|| b.name.cmp(&a.name)),
                    },
                    Header::Id => match ord {
                        SortedOrder::Asc => a.id.cmp(&b.id).then_with(|| a.name.cmp(&b.name)),
                        SortedOrder::Desc => b.id.cmp(&a.id).then_with(|| b.name.cmp(&a.name)),
                    },
                    Header::Image => match ord {
                        SortedOrder::Asc => a.image.cmp(&b.image).then_with(|| a.name.cmp(&b.name)),
                        SortedOrder::Desc => {
                            b.image.cmp(&a.image).then_with(|| b.name.cmp(&a.name))
                        }
                    },
                    Header::Name => match ord {
                        SortedOrder::Asc => a.name.cmp(&b.name).then_with(|| a.id.cmp(&b.id)),
                        SortedOrder::Desc => b.name.cmp(&a.name).then_with(|| b.id.cmp(&a.id)),
                    },
                    Header::Rx => match ord {
                        SortedOrder::Asc => a.rx.cmp(&b.rx).then_with(|| a.name.cmp(&b.name)),
                        SortedOrder::Desc => b.rx.cmp(&a.rx).then_with(|| b.name.cmp(&a.name)),
                    },
                    Header::Tx => match ord {
                        SortedOrder::Asc => a.tx.cmp(&b.tx).then_with(|| a.name.cmp(&b.name)),
                        SortedOrder::Desc => b.tx.cmp(&a.tx).then_with(|| b.name.cmp(&a.name)),
                    },
                }
            };
            self.containers.items.sort_by(sort_closure);
        } else {
            self.containers
                .items
                .sort_by(|a, b| a.created.cmp(&b.created).then_with(|| a.name.cmp(&b.name)));
        }
    }

    /// Container state methods

    /// Just get the total number of containers
    pub fn get_container_len(&self) -> usize {
        self.containers.items.len()
    }

    /// Get title for containers section
    pub fn container_title(&self) -> String {
        self.containers.get_state_title()
    }

    /// Select the first container
    pub fn containers_start(&mut self) {
        self.containers.start();
    }

    /// select the last container
    pub fn containers_end(&mut self) {
        self.containers.end();
    }

    /// Select the next container
    pub fn containers_next(&mut self) {
        self.containers.next();
    }

    /// select the previous container
    pub fn containers_previous(&mut self) {
        self.containers.previous();
    }

    /// Get Container items
    pub const fn get_container_items(&self) -> &Vec<ContainerItem> {
        &self.containers.items
    }

    /// Get Option of the current selected container
    pub fn get_selected_container(&self) -> Option<&ContainerItem> {
        self.containers
            .state
            .selected()
            .and_then(|i| self.containers.items.get(i))
    }

    /// Get mutable Option of the current selected container
    fn get_mut_selected_container(&mut self) -> Option<&mut ContainerItem> {
        self.containers
            .state
            .selected()
            .and_then(|i| self.containers.items.get_mut(i))
    }

    /// Get ListState of containers
    pub fn get_container_state(&mut self) -> &mut ListState {
        &mut self.containers.state
    }

    /// Selected DockerCommand methods

    /// Get the current selected docker command
    /// So know which command to execute
    pub fn selected_docker_command(&self) -> Option<DockerControls> {
        self.get_selected_container().and_then(|i| {
            i.docker_controls.state.selected().and_then(|x| {
                i.docker_controls
                    .items
                    .get(x)
                    .map(std::borrow::ToOwned::to_owned)
            })
        })
    }
    /// Get mutable Option of the currently selected container DockerControls state
    pub fn get_control_state(&mut self) -> Option<&mut ListState> {
        self.get_mut_selected_container()
            .map(|i| &mut i.docker_controls.state)
    }

    /// Get mutable Option of the currently selected container DockerControls items
    pub fn get_control_items(&mut self) -> Option<&mut Vec<DockerControls>> {
        self.get_mut_selected_container()
            .map(|i| &mut i.docker_controls.items)
    }

    /// Change selected choice of docker commands of selected container
    pub fn docker_command_next(&mut self) {
        if let Some(i) = self.get_mut_selected_container() {
            i.docker_controls.next();
        }
    }

    /// Change selected choice of docker commands of selected container
    pub fn docker_command_previous(&mut self) {
        if let Some(i) = self.get_mut_selected_container() {
            i.docker_controls.previous();
        }
    }

    /// Change selected choice of docker commands of selected container
    pub fn docker_command_start(&mut self) {
        if let Some(i) = self.get_mut_selected_container() {
            i.docker_controls.start();
        }
    }

    /// Change selected choice of docker commands of selected container
    pub fn docker_command_end(&mut self) {
        if let Some(i) = self.get_mut_selected_container() {
            i.docker_controls.end();
        }
    }

    /// Logs related methods

    /// Get the title for log panel for selected container, will be either
    /// 1) "logs x/x - container_name" where container_name is 32 chars max
    /// 2) "logs - container_name" when no logs found, again 32 chars max
    /// 3) "" no container currently selected - aka no containers on system
    pub fn get_log_title(&self) -> String {
        self.get_selected_container().map_or_else(String::new, |y| {
            let logs_len = y.logs.get_state_title();
            let mut name = y.name.clone();
            name.truncate(32);
            if logs_len.is_empty() {
                format!("- {name} ")
            } else {
                format!("{logs_len} - {name}")
            }
        })
    }

    /// select next selected log line
    pub fn log_next(&mut self) {
        if let Some(i) = self.get_mut_selected_container() {
            i.logs.next();
        }
    }

    /// select previous selected log line
    pub fn log_previous(&mut self) {
        if let Some(i) = self.get_mut_selected_container() {
            i.logs.previous();
        }
    }

    /// select last selected log line
    pub fn log_end(&mut self) {
        if let Some(i) = self.get_mut_selected_container() {
            i.logs.end();
        }
    }

    /// select first selected log line
    pub fn log_start(&mut self) {
        if let Some(i) = self.get_mut_selected_container() {
            i.logs.start();
        }
    }

    /// Chart data related methods

    /// Get mutable Option of the currently selected container chart data
    pub fn get_chart_data(&mut self) -> Option<(CpuTuple, MemTuple)> {
        self.containers
            .state
            .selected()
            .and_then(|i| self.containers.items.get_mut(i))
            .map(|i| i.get_chart_data())
    }

    /// Logs related methods

    /// Get mutable Vec of current containers logs
    pub fn get_logs(&mut self) -> Vec<ListItem<'static>> {
        self.containers
            .state
            .selected()
            .and_then(|i| self.containers.items.get_mut(i))
            .map_or(vec![], |i| i.logs.to_vec())
    }

    /// Get mutable Option of the currently selected container Logs state
    pub fn get_log_state(&mut self) -> Option<&mut ListState> {
        self.containers
            .state
            .selected()
            .and_then(|i| self.containers.items.get_mut(i))
            .map(|i| i.logs.state())
    }

    /// Error related methods

    /// return single app_state error
    pub const fn get_error(&self) -> Option<AppError> {
        self.error
    }

    /// remove single app_state error
    pub fn remove_error(&mut self) {
        self.error = None;
    }

    /// insert single app_state error
    pub fn set_error(&mut self, error: AppError, gui_state: &Arc<Mutex<GuiState>>, status: Status) {
        gui_state.lock().status_push(status);
        self.error = Some(error);
    }

    /// Check if the selected container is a dockerised version of oxker
    /// So that can disallow commands to be send
    /// Is a shabby way of implementing this
    pub fn is_oxker(&self) -> bool {
        self.get_selected_container().map_or(false, |i| i.is_oxker)
    }

    /// Find the widths for the strings in the containers panel.
    /// So can display nicely and evenly
    pub fn get_width(&self) -> Columns {
        let mut columns = Columns::new();
        let count = |x: &str| u8::try_from(x.chars().count()).unwrap_or(12);

        // Should probably find a refactor here somewhere
        for container in &self.containers.items {
            let cpu_count = count(
                &container
                    .cpu_stats
                    .back()
                    .unwrap_or(&CpuStats::default())
                    .to_string(),
            );

            let mem_current_count = count(
                &container
                    .mem_stats
                    .back()
                    .unwrap_or(&ByteStats::default())
                    .to_string(),
            );

            columns.cpu.1 = columns.cpu.1.max(cpu_count);
            columns.image.1 = columns.image.1.max(count(&container.image));
            columns.mem.1 = columns.mem.1.max(mem_current_count);
            columns.mem.2 = columns.mem.2.max(count(&container.mem_limit.to_string()));
            columns.name.1 = columns.name.1.max(count(&container.name));
            columns.net_rx.1 = columns.net_rx.1.max(count(&container.rx.to_string()));
            columns.net_tx.1 = columns.net_tx.1.max(count(&container.tx.to_string()));
            columns.state.1 = columns.state.1.max(count(&container.state.to_string()));
            columns.status.1 = columns.status.1.max(count(&container.status));
        }
        columns
    }

    /// Update related methods

    /// return a mutable container by given id
    fn get_container_by_id(&mut self, id: &ContainerId) -> Option<&mut ContainerItem> {
        self.containers.items.iter_mut().find(|i| &i.id == id)
    }

    /// return a mutable container by given id
    pub fn get_container_name_by_id(&mut self, id: &ContainerId) -> Option<String> {
        self.containers
            .items
            .iter_mut()
            .find(|i| &i.id == id)
            .map(|i| i.name.clone())
    }

    /// Find the id of the currently selected container.
    /// If any containers on system, will always return a ContainerId
    /// Only returns None when no containers found.
    pub fn get_selected_container_id(&self) -> Option<ContainerId> {
        self.get_selected_container().map(|i| i.id.clone())
    }

    /// Get the Id and State for the currently selected container - used by the exec check method
    pub fn get_selected_container_id_state_name(&self) -> Option<(ContainerId, State, String)> {
        self.get_selected_container()
            .map(|i| (i.id.clone(), i.state, i.name.clone()))
    }

    /// Update container mem, cpu, & network stats, in single function so only need to call .lock() once
    /// Will also, if a sort is set, sort the containers
    pub fn update_stats(
        &mut self,
        id: &ContainerId,
        cpu_stat: Option<f64>,
        mem_stat: Option<u64>,
        mem_limit: u64,
        rx: u64,
        tx: u64,
    ) {
        if let Some(container) = self.get_container_by_id(id) {
            if container.cpu_stats.len() >= 60 {
                container.cpu_stats.pop_front();
            }
            if container.mem_stats.len() >= 60 {
                container.mem_stats.pop_front();
            }

            if let Some(cpu) = cpu_stat {
                container.cpu_stats.push_back(CpuStats::new(cpu));
            }
            if let Some(mem) = mem_stat {
                container.mem_stats.push_back(ByteStats::new(mem));
            }

            container.rx.update(rx);
            container.tx.update(tx);
            container.mem_limit.update(mem_limit);
        }
        // need to benchmark this?
        self.sort_containers();
    }

    /// Update, or insert, containers
    pub fn update_containers(&mut self, all_containers: &mut [ContainerSummary]) {
        let all_ids = self
            .containers
            .items
            .iter()
            .map(|i| i.id.clone())
            .collect::<Vec<_>>();

        // Only sort it no containers currently set, as afterwards the order is fixed
        if self.containers.items.is_empty() {
            all_containers.sort_by(|a, b| a.created.cmp(&b.created));
        }

        if !all_containers.is_empty() && self.containers.state.selected().is_none() {
            self.containers.start();
        }

        for (index, id) in all_ids.iter().enumerate() {
            if !all_containers
                .iter()
                .filter_map(|i| i.id.as_ref())
                .any(|x| x == id.get())
            {
                // If removed container is currently selected, then change selected to previous
                // This will default to 0 in any edge cases
                if self.containers.state.selected().is_some() {
                    self.containers.previous();
                }
                // Check is some, else can cause out of bounds error, if containers get removed before a docker update
                if self.containers.items.get(index).is_some() {
                    self.containers.items.remove(index);
                }
            }
        }

        for i in all_containers {
            if let Some(id) = i.id.as_ref() {
                let name = i.names.as_mut().map_or(String::new(), |names| {
                    names.first_mut().map_or(String::new(), |f| {
                        if f.starts_with('/') {
                            f.remove(0);
                        }
                        (*f).to_string()
                    })
                });

                let id = ContainerId::from(id.as_str());

                let is_oxker = i
                    .command
                    .as_ref()
                    .map_or(false, |i| i.starts_with(ENTRY_POINT));

                let state = State::from(i.state.as_ref().map_or("dead", |z| z));
                let status = i
                    .status
                    .as_ref()
                    .map_or(String::new(), std::clone::Clone::clone);

                let image = i
                    .image
                    .as_ref()
                    .map_or(String::new(), std::clone::Clone::clone);

                let created = i
                    .created
                    .map_or(0, |i| u64::try_from(i).unwrap_or_default());
                // If container info already in containers Vec, then just update details
                if let Some(item) = self.get_container_by_id(&id) {
                    if item.name != name {
                        item.name = name;
                    };
                    if item.status != status {
                        item.status = status;
                    };
                    if item.state != state {
                        item.docker_controls.items = DockerControls::gen_vec(state);
                        // Update the list state, needs to be None if the gen_vec returns an empty vec
                        match state {
                            State::Removing | State::Restarting | State::Unknown => {
                                item.docker_controls.state.select(None);
                            }
                            _ => item.docker_controls.start(),
                        };
                        item.state = state;
                    };
                    if item.image != image {
                        item.image = image;
                    };
                } else {
                    // container not known, so make new ContainerItem and push into containers Vec
                    let container =
                        ContainerItem::new(created, id, image, is_oxker, name, state, status);
                    self.containers.items.push(container);
                }
            }
        }
    }

    /// update logs of a given container, based on id
    pub fn update_log_by_id(&mut self, logs: Vec<String>, id: &ContainerId) {
        let color = self.args.color;
        let raw = self.args.raw;

        let timestamp = self.args.timestamp;

        if let Some(container) = self.get_container_by_id(id) {
            if !container.is_oxker {
                container.last_updated = Self::get_systemtime();
                let current_len = container.logs.len();

                for mut i in logs {
                    let tz = LogsTz::from(i.as_str());
                    // Strip the timestamp if `-t` flag set
                    if !timestamp {
                        i = i.replace(&tz.to_string(), "");
                    }
                    let lines = if color {
                        log_sanitizer::colorize_logs(&i)
                    } else if raw {
                        log_sanitizer::raw(&i)
                    } else {
                        log_sanitizer::remove_ansi(&i)
                    };
                    container.logs.insert(ListItem::new(lines), tz);
                }

                // Set the logs selected row for each container
                // Either when no long currently selected, or currently selected (before updated) is already at end
                if container.logs.state().selected().is_none()
                    || container.logs.state().selected().map_or(1, |f| f + 1) == current_len
                {
                    container.logs.end();
                }
            }
        }
    }
}
