use bollard::models::ContainerSummary;
use core::fmt;
use parking_lot::Mutex;
use ratatui::widgets::{ListItem, ListState};
use std::{
    hash::Hash,
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

#[derive(Debug, Clone, Default, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum FilterBy {
    #[default]
    Name,
    Image,
    Status,
    All,
}

/// Convert errors into strings to display
impl fmt::Display for FilterBy {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Name => "Name",
                Self::Image => "Image",
                Self::Status => "Status",
                Self::All => "All",
            }
        )
    }
}

impl FilterBy {
    const fn next(self) -> Option<Self> {
        match self {
            Self::Name => Some(Self::Image),
            Self::Image => Some(Self::Status),
            Self::Status => Some(Self::All),
            Self::All => None,
        }
    }

    const fn prev(self) -> Option<Self> {
        match self {
            Self::Name => None,
            Self::Image => Some(Self::Name),
            Self::Status => Some(Self::Image),
            Self::All => Some(Self::Status),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Filter {
    pub term: Option<String>,
    pub by: FilterBy,
}
impl Filter {
    pub fn new() -> Self {
        Self {
            term: None,
            by: FilterBy::default(),
        }
    }
}

/// Global app_state, stored in an Arc<Mutex>
#[derive(Debug, Clone)]
#[cfg(not(test))]
pub struct AppData {
    containers: StatefulList<ContainerItem>,
    error: Option<AppError>,
    filter: Filter,
    hidden_containers: Vec<ContainerItem>,
    sorted_by: Option<(Header, SortedOrder)>,
    pub args: CliArgs,
}

#[derive(Debug, Clone)]
#[cfg(test)]
pub struct AppData {
    pub args: CliArgs,
    pub containers: StatefulList<ContainerItem>,
    pub error: Option<AppError>,
    pub filter: Filter,
    pub hidden_containers: Vec<ContainerItem>,
    pub sorted_by: Option<(Header, SortedOrder)>,
}

impl AppData {
    /// Generate a default app_state
    pub fn default(args: CliArgs) -> Self {
        Self {
            args,
            containers: StatefulList::new(vec![]),
            hidden_containers: vec![],
            error: None,
            sorted_by: None,
            filter: Filter::new(),
        }
    }

    /// Current time as unix timestamp
    #[allow(clippy::expect_used)]
    fn get_systemtime() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("In our known reality, this error should never occur")
            .as_secs()
    }

    /// Filter related methods

    /// Get the current filter term
    pub const fn get_filter_term(&self) -> Option<&String> {
        self.filter.term.as_ref()
    }

    /// Get the current filter by choice
    pub const fn get_filter_by(&self) -> FilterBy {
        self.filter.by
    }

    /// Check if a given container can be inserted into the "visible" list, based on current filter term and filter_by
    fn can_insert(&self, container: &ContainerItem) -> bool {
        self.filter.term.as_ref().map_or(true, |term| {
            let term = term.to_lowercase();
            match self.filter.by {
                FilterBy::All => {
                    container.name.contains(&term)
                        || container.image.contains(&term)
                        || container.status.to_lowercase().contains(&term)
                }
                FilterBy::Image => container.image.contains(&term),
                FilterBy::Name => container.name.contains(&term),
                FilterBy::Status => container.status.to_lowercase().contains(&term),
            }
        })
    }

    /// Remove items from the containers list based on the filter term, and insert into a "hidden" vec
    /// sets the state to start if any filtering has occurred
    /// Also search in the "hidden" vec for items and insert back into the main containers vec
    fn filter_containers(&mut self) {
        let pre_len = self.get_container_len();

        if !self.hidden_containers.is_empty() {
            let (mut new_items, tmp_items): (Vec<_>, Vec<_>) = self
                .hidden_containers
                .iter()
                .cloned()
                .partition(|item| self.can_insert(item));

            while let Some(x) = new_items.pop() {
                self.containers.items.push(x);
            }
            self.hidden_containers = tmp_items;
        }

        let (new_items, tmp_items) = self
            .containers
            .items
            .iter()
            .cloned()
            .partition(|item| self.can_insert(item));

        self.containers.items = new_items;
        self.hidden_containers.extend(tmp_items);

        self.sort_containers();
        if self.get_container_len() != pre_len {
            self.containers.start();
        }
    }

    /// Re-filter the containers, used after the filter.by has been changed
    fn re_filter(&mut self) {
        self.containers.items.append(&mut self.hidden_containers);
        self.hidden_containers = vec![];
        self.filter_containers();
    }

    /// Set a single char into the filter term
    pub fn filter_term_push(&mut self, c: char) {
        if let Some(term) = self.filter.term.as_mut() {
            term.push(c);
        } else {
            self.filter.term = Some(format!("{c}"));
        };
        self.filter_containers();
    }

    /// Delete the final char of the filter term
    pub fn filter_term_pop(&mut self) {
        if let Some(term) = self.filter.term.as_mut() {
            // should now search for items in the tmp vec, and insert into containers if found
            term.pop();
            if term.is_empty() {
                self.filter.term = None;
            }
        }
        self.filter_containers();
    }

    // change the filter_by option
    pub fn filter_by_next(&mut self) {
        if let Some(by) = self.filter.by.next() {
            self.filter.by = by;
            self.re_filter();
        }
    }

    // change the filter_by option
    pub fn filter_by_prev(&mut self) {
        if let Some(by) = self.filter.by.prev() {
            self.filter.by = by;
            self.re_filter();
        }
    }

    /// Remove the filter completely
    pub fn filter_term_clear(&mut self) {
        self.filter.term = None;
        while let Some(i) = self.hidden_containers.pop() {
            if self.get_container_by_id(&i.id).is_none() {
                self.containers.items.push(i);
            };
        }
        self.sort_containers();
    }

    /// Container sort related methods

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
                let item_ord = match ord {
                    SortedOrder::Asc => (a, b),
                    SortedOrder::Desc => (b, a),
                };
                match head {
                    Header::State => item_ord
                        .0
                        .state
                        .order()
                        .cmp(&item_ord.1.state.order())
                        .then_with(|| item_ord.0.name.get().cmp(item_ord.1.name.get())),
                    Header::Status => item_ord
                        .0
                        .status
                        .cmp(&item_ord.1.status)
                        .then_with(|| item_ord.0.name.get().cmp(item_ord.1.name.get())),
                    Header::Cpu => item_ord
                        .0
                        .cpu_stats
                        .back()
                        .cmp(&item_ord.1.cpu_stats.back())
                        .then_with(|| item_ord.0.name.get().cmp(item_ord.1.name.get())),
                    Header::Memory => item_ord
                        .0
                        .mem_stats
                        .back()
                        .cmp(&item_ord.1.mem_stats.back())
                        .then_with(|| item_ord.0.name.get().cmp(item_ord.1.name.get())),

                    Header::Id => item_ord
                        .0
                        .id
                        .cmp(&item_ord.1.id)
                        .then_with(|| item_ord.0.name.get().cmp(item_ord.1.name.get())),
                    Header::Image => item_ord
                        .0
                        .image
                        .get()
                        .cmp(item_ord.1.image.get())
                        .then_with(|| item_ord.0.name.get().cmp(item_ord.1.name.get())),
                    Header::Rx => item_ord
                        .0
                        .rx
                        .cmp(&item_ord.1.rx)
                        .then_with(|| item_ord.0.name.get().cmp(item_ord.1.name.get())),
                    Header::Tx => item_ord
                        .0
                        .tx
                        .cmp(&item_ord.1.tx)
                        .then_with(|| item_ord.0.name.get().cmp(item_ord.1.name.get())),

                    Header::Name => item_ord
                        .0
                        .name
                        .get()
                        .cmp(item_ord.1.name.get())
                        .then_with(|| item_ord.0.id.cmp(&item_ord.1.id)),
                }
            };
            self.containers.items.sort_by(sort_closure);
        } else {
            self.containers.items.sort_by(|a, b| {
                a.created
                    .cmp(&b.created)
                    .then_with(|| a.name.get().cmp(b.name.get()))
            });
        }
    }

    /// Container state methods

    /// Get the total number of none "hidden" containers
    pub fn get_container_len(&self) -> usize {
        self.containers.items.len()
    }

    /// Get all the ContainerItems
    pub fn get_container_items(&self) -> &[ContainerItem] {
        &self.containers.items
    }

    /// Get title for containers section, add a suffix indicating if the containers are currently under filter
    pub fn container_title(&self) -> String {
        let suffix = if !self.hidden_containers.is_empty() && !self.containers.items.is_empty() {
            " - filtered"
        } else {
            ""
        };
        format!("{}{}", self.containers.get_state_title(), suffix)
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

    /// Get ListState of containers
    pub fn get_container_state(&mut self) -> &mut ListState {
        &mut self.containers.state
    }

    /// Get Option of the current selected container
    pub fn get_selected_container(&self) -> Option<&ContainerItem> {
        self.containers
            .state
            .selected()
            .and_then(|i| self.containers.items.get(i))
    }

    /// Find the longest port when it's transformed into a string, defaults are header lens (ip, private, public)
    pub fn get_longest_port(&self) -> (usize, usize, usize) {
        let mut longest_ip = 5;
        let mut longest_private = 10;
        let mut longest_public = 9;

        for item in [&self.containers.items, &self.hidden_containers] {
            for item in item {
                longest_ip = longest_ip.max(
                    item.ports
                        .iter()
                        .map(ContainerPorts::len_ip)
                        .max()
                        .unwrap_or(3),
                );
                longest_private = longest_private.max(
                    item.ports
                        .iter()
                        .map(ContainerPorts::len_private)
                        .max()
                        .unwrap_or(8),
                );
                longest_public = longest_public.max(
                    item.ports
                        .iter()
                        .map(ContainerPorts::len_public)
                        .max()
                        .unwrap_or(6),
                );
            }
        }

        (longest_ip, longest_private, longest_public)
    }

    /// Get Option of the current selected container's ports, sorted by private port
    pub fn get_selected_ports(&mut self) -> Option<(Vec<ContainerPorts>, State)> {
        if let Some(item) = self.get_mut_selected_container() {
            let mut ports = item.ports.clone();
            ports.sort_by(|a, b| a.private.cmp(&b.private));
            return Some((ports, item.state));
        }
        None
    }

    /// Get mutable Option of the current selected container
    fn get_mut_selected_container(&mut self) -> Option<&mut ContainerItem> {
        self.containers
            .state
            .selected()
            .and_then(|i| self.containers.items.get_mut(i))
    }

    /// Get a mutable container by given id
    fn get_container_by_id(&mut self, id: &ContainerId) -> Option<&mut ContainerItem> {
        self.containers.items.iter_mut().find(|i| &i.id == id)
    }

    /// Get a mutable container by given id in the tmp_container vec
    fn get_hidden_container_by_id(&mut self, id: &ContainerId) -> Option<&mut ContainerItem> {
        self.hidden_containers.iter_mut().find(|i| &i.id == id)
    }

    /// Get the ContainerName of by ID
    pub fn get_container_name_by_id(&mut self, id: &ContainerId) -> Option<ContainerName> {
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
            .map(|i| (i.id.clone(), i.state, i.name.get().to_owned()))
    }

    /// Selected DockerCommand methods

    /// Get the current selected docker command
    /// So know which command to execute
    pub fn selected_docker_controls(&self) -> Option<DockerControls> {
        self.get_selected_container().and_then(|i| {
            i.docker_controls.state.selected().and_then(|x| {
                i.docker_controls
                    .items
                    .get(x)
                    .map(std::borrow::ToOwned::to_owned)
            })
        })
    }

    /// Change selected choice of docker commands of selected container
    pub fn docker_controls_next(&mut self) {
        if let Some(i) = self.get_mut_selected_container() {
            i.docker_controls.next();
        }
    }

    /// Change selected choice of docker commands of selected container
    pub fn docker_controls_previous(&mut self) {
        if let Some(i) = self.get_mut_selected_container() {
            i.docker_controls.previous();
        }
    }

    /// Change selected choice of docker commands of selected container
    pub fn docker_controls_start(&mut self) {
        if let Some(i) = self.get_mut_selected_container() {
            i.docker_controls.start();
        }
    }

    /// Change selected choice of docker commands of selected container
    pub fn docker_controls_end(&mut self) {
        if let Some(i) = self.get_mut_selected_container() {
            i.docker_controls.end();
        }
    }

    /// Get mutable Option of the currently selected container DockerControls state
    pub fn get_control_state(&mut self) -> Option<&mut ListState> {
        self.get_mut_selected_container()
            .map(|i| &mut i.docker_controls.state)
    }

    /// Get mutable Option of the currently selected container DockerControls items
    /// TODO command or control, need a uniform name across the application
    pub fn get_control_items(&mut self) -> Option<&mut Vec<DockerControls>> {
        self.get_mut_selected_container()
            .map(|i| &mut i.docker_controls.items)
    }

    /// Logs related methods

    /// Get the title for log panel for selected container, will be either
    /// 1) "logs x/x - container_name" where container_name is 32 chars max
    /// 2) "logs - container_name" when no logs found, again 32 chars max
    /// 3) "" no container currently selected - aka no containers on system
    pub fn get_log_title(&self) -> String {
        self.get_selected_container()
            .map_or_else(String::new, |ci| {
                let logs_len = ci.logs.get_state_title();
                let prefix = if logs_len.is_empty() {
                    String::from(" ")
                } else {
                    format!("{logs_len} ")
                };
                format!("{}- {}", prefix, ci.name.get())
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

    /// Chart data related methods

    /// Get mutable Option of the currently selected container chart data
    pub fn get_chart_data(&mut self) -> Option<(CpuTuple, MemTuple)> {
        self.containers
            .state
            .selected()
            .and_then(|i| self.containers.items.get_mut(i))
            .map(|i| i.get_chart_data())
    }

    /// Error related methods

    /// Get single app_state error
    pub const fn get_error(&self) -> Option<AppError> {
        self.error
    }

    /// Remove single app_state error
    pub fn remove_error(&mut self) {
        self.error = None;
    }

    /// Insert single app_state error
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

    /// Check if selected container is oxker and also that oxker is being run in a container
    pub fn is_oxker_in_container(&self) -> bool {
        self.get_selected_container()
            .map_or(false, |i| i.is_oxker && self.args.in_container)
    }

    /// Find the widths for the strings in the containers panel.
    /// So can display nicely and evenly
    /// Searches in both contains & hidden_containers
    pub fn get_width(&self) -> Columns {
        let mut columns = Columns::new();
        let count = |x: &str| u8::try_from(x.chars().count()).unwrap_or(12);

        // Should probably find a refactor here somewhere
        for container in [&self.containers.items, &self.hidden_containers] {
            for container in container {
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
                columns.image.1 = columns.image.1.max(count(&container.image.to_string()));
                columns.mem.1 = columns.mem.1.max(mem_current_count);
                columns.mem.2 = columns.mem.2.max(count(&container.mem_limit.to_string()));
                columns.name.1 = columns.name.1.max(count(&container.name.to_string()));
                columns.net_rx.1 = columns.net_rx.1.max(count(&container.rx.to_string()));
                columns.net_tx.1 = columns.net_tx.1.max(count(&container.tx.to_string()));
                columns.state.1 = columns.state.1.max(count(&container.state.to_string()));
                columns.status.1 = columns.status.1.max(count(&container.status));
            }
        }
        columns
    }

    /// Update related methods

    /// Get mutable reference to a container in the containers vec & the hidden_containers vec
    fn get_any_container_by_id(&mut self, id: &ContainerId) -> Option<&mut ContainerItem> {
        if self.get_hidden_container_by_id(id).is_some() {
            self.get_hidden_container_by_id(id)
        } else {
            self.get_container_by_id(id)
        }
    }

    /// Update container mem, cpu, & network stats, in single function so only need to call .lock() once
    /// Will also, if a sort is set, sort the containers
    pub fn update_stats_by_id(
        &mut self,
        id: &ContainerId,
        cpu_stat: Option<f64>,
        mem_stat: Option<u64>,
        mem_limit: u64,
        rx: u64,
        tx: u64,
    ) {
        if let Some(container) = self.get_any_container_by_id(id) {
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

                let ports = i.ports.as_ref().map_or(vec![], |i| {
                    i.iter().map(ContainerPorts::from).collect::<Vec<_>>()
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

                if let Some(item) = self.get_any_container_by_id(&id) {
                    if item.name.get() != name {
                        item.name.set(name);
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

                    item.ports = ports;

                    if item.image.get() != image {
                        item.image.set(image);
                    };
                } else {
                    // container not known, so make new ContainerItem and push into containers Ve
                    let container = ContainerItem::new(
                        created, id, image, is_oxker, name, ports, state, status,
                    );
                    let can_insert = self.can_insert(&container);
                    if can_insert {
                        self.containers.items.push(container);
                    } else {
                        self.hidden_containers.push(container);
                    }
                }
            }
        }
    }

    /// Update logs of a given container, based on id
    pub fn update_log_by_id(&mut self, logs: Vec<String>, id: &ContainerId) {
        let color = self.args.color;
        let raw = self.args.raw;

        let timestamp = self.args.timestamp;

        if let Some(container) = self.get_any_container_by_id(id) {
            if !container.is_oxker {
                container.last_updated = Self::get_systemtime();
                let current_len = container.logs.len();

                for mut i in logs {
                    let tz = LogsTz::from(i.as_str());
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

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::many_single_char_names)]
mod tests {

    use super::*;
    use crate::tests::{gen_appdata, gen_container_summary, gen_containers};
    use std::collections::VecDeque;

    // ******* //
    // Sort by //
    // ******* //

    #[test]
    /// Sort by header: name
    fn test_app_data_set_sort_by_header_name() {
        let (_ids, containers) = gen_containers();

        let mut app_data = gen_appdata(&containers);

        let result = app_data.get_container_items();
        assert_eq!(result, &containers);

        // descending
        app_data.set_sorted(Some((Header::Name, SortedOrder::Desc)));
        let result = app_data.get_container_items();
        let (a, b, c) = (&result[0], &result[1], &result[2]);
        assert_eq!(a.id, ContainerId::from("3"));
        assert_eq!(b.id, ContainerId::from("2"));
        assert_eq!(c.id, ContainerId::from("1"));

        // ascending
        app_data.set_sorted(Some((Header::Name, SortedOrder::Asc)));
        let result = app_data.get_container_items();
        let (a, b, c) = (&result[0], &result[1], &result[2]);
        assert_eq!(a.id, ContainerId::from("1"));
        assert_eq!(b.id, ContainerId::from("2"));
        assert_eq!(c.id, ContainerId::from("3"));
    }

    #[test]
    /// Sort by header: state
    fn test_app_data_set_sort_by_header_state() {
        let (_ids, containers) = gen_containers();

        let mut app_data = gen_appdata(&containers);

        let result = app_data.get_container_items();
        assert_eq!(result, &containers);

        if let Some(i) = app_data.get_container_by_id(&ContainerId::from("1")) {
            i.state = State::Exited;
        }
        if let Some(i) = app_data.get_container_by_id(&ContainerId::from("2")) {
            i.state = State::Running;
        }
        if let Some(i) = app_data.get_container_by_id(&ContainerId::from("3")) {
            i.state = State::Paused;
        }

        // descending
        app_data.set_sorted(Some((Header::State, SortedOrder::Desc)));
        let result = app_data.get_container_items();
        let (a, b, c) = (&result[0], &result[1], &result[2]);
        assert_eq!(a.id, ContainerId::from("1"));
        assert_eq!(b.id, ContainerId::from("3"));
        assert_eq!(c.id, ContainerId::from("2"));

        // ascending
        app_data.set_sorted(Some((Header::State, SortedOrder::Asc)));
        let result = app_data.get_container_items();
        let (a, b, c) = (&result[0], &result[1], &result[2]);
        assert_eq!(a.id, ContainerId::from("2"));
        assert_eq!(b.id, ContainerId::from("3"));
        assert_eq!(c.id, ContainerId::from("1"));
    }

    #[test]
    /// Sort by header: status
    fn test_app_data_set_sort_by_header_status() {
        let (_ids, containers) = gen_containers();

        let mut app_data = gen_appdata(&containers);

        let result = app_data.get_container_items();
        assert_eq!(result, &containers);

        if let Some(i) = app_data.get_container_by_id(&ContainerId::from("2")) {
            "Exited (0) 10 minutes ago".clone_into(&mut i.status);
        }

        if let Some(i) = app_data.get_container_by_id(&ContainerId::from("3")) {
            "Up 2 hours (Paused)".clone_into(&mut i.status);
        }

        // Sort by status
        // descending
        app_data.set_sorted(Some((Header::Status, SortedOrder::Desc)));
        let result = app_data.get_container_items();
        let (a, b, c) = (&result[0], &result[1], &result[2]);
        assert_eq!(a.id, ContainerId::from("3"));
        assert_eq!(b.id, ContainerId::from("1"));
        assert_eq!(c.id, ContainerId::from("2"));

        // ascending
        app_data.set_sorted(Some((Header::Status, SortedOrder::Asc)));
        let result = app_data.get_container_items();
        let (a, b, c) = (&result[0], &result[1], &result[2]);
        assert_eq!(a.id, ContainerId::from("2"));
        assert_eq!(b.id, ContainerId::from("1"));
        assert_eq!(c.id, ContainerId::from("3"));
    }

    #[test]
    /// Sort by header: cpu
    fn test_app_data_set_sort_by_header_cpu() {
        let (_ids, containers) = gen_containers();

        let mut app_data = gen_appdata(&containers);

        let result = app_data.get_container_items();
        assert_eq!(result, &containers);

        if let Some(i) = app_data.get_container_by_id(&ContainerId::from("1")) {
            i.cpu_stats = VecDeque::from([CpuStats::new(10.1)]);
        }
        if let Some(i) = app_data.get_container_by_id(&ContainerId::from("2")) {
            i.cpu_stats = VecDeque::from([CpuStats::new(8.1)]);
        }
        if let Some(i) = app_data.get_container_by_id(&ContainerId::from("3")) {
            i.cpu_stats = VecDeque::from([CpuStats::new(20.3)]);
        }

        // descending
        app_data.set_sorted(Some((Header::Cpu, SortedOrder::Desc)));
        let result = app_data.get_container_items();
        let (a, b, c) = (&result[0], &result[1], &result[2]);
        assert_eq!(a.id, ContainerId::from("3"));
        assert_eq!(b.id, ContainerId::from("1"));
        assert_eq!(c.id, ContainerId::from("2"));

        // ascending
        app_data.set_sorted(Some((Header::Cpu, SortedOrder::Asc)));
        let result = app_data.get_container_items();
        let (a, b, c) = (&result[0], &result[1], &result[2]);
        assert_eq!(a.id, ContainerId::from("2"));
        assert_eq!(b.id, ContainerId::from("1"));
        assert_eq!(c.id, ContainerId::from("3"));
    }

    #[test]
    /// Sort by header: memory
    fn test_app_data_set_sort_by_header_mem() {
        let (_ids, containers) = gen_containers();

        let mut app_data = gen_appdata(&containers);

        let result = app_data.get_container_items();
        assert_eq!(result, &containers);

        if let Some(i) = app_data.get_container_by_id(&ContainerId::from("1")) {
            i.mem_stats = VecDeque::from([ByteStats::new(40)]);
        }
        if let Some(i) = app_data.get_container_by_id(&ContainerId::from("2")) {
            i.mem_stats = VecDeque::from([ByteStats::new(80)]);
        }
        if let Some(i) = app_data.get_container_by_id(&ContainerId::from("3")) {
            i.mem_stats = VecDeque::from([ByteStats::new(2)]);
        }

        // descending
        app_data.set_sorted(Some((Header::Memory, SortedOrder::Desc)));
        let result = app_data.get_container_items();
        let (a, b, c) = (&result[0], &result[1], &result[2]);
        assert_eq!(a.id, ContainerId::from("2"));
        assert_eq!(b.id, ContainerId::from("1"));
        assert_eq!(c.id, ContainerId::from("3"));

        // ascending
        app_data.set_sorted(Some((Header::Memory, SortedOrder::Asc)));
        let result = app_data.get_container_items();
        let (a, b, c) = (&result[0], &result[1], &result[2]);
        assert_eq!(a.id, ContainerId::from("3"));
        assert_eq!(b.id, ContainerId::from("1"));
        assert_eq!(c.id, ContainerId::from("2"));
    }

    #[test]
    /// Sort by header: id
    fn test_app_data_set_sort_by_header_id() {
        let (_ids, containers) = gen_containers();

        let mut app_data = gen_appdata(&containers);

        let result = app_data.get_container_items();
        assert_eq!(result, &containers);

        // descending
        app_data.set_sorted(Some((Header::Id, SortedOrder::Desc)));
        let result = app_data.get_container_items();
        let (a, b, c) = (&result[0], &result[1], &result[2]);
        assert_eq!(a.id, ContainerId::from("3"));
        assert_eq!(b.id, ContainerId::from("2"));
        assert_eq!(c.id, ContainerId::from("1"));

        // ascending
        app_data.set_sorted(Some((Header::Id, SortedOrder::Asc)));
        let result = app_data.get_container_items();
        let (a, b, c) = (&result[0], &result[1], &result[2]);
        assert_eq!(a.id, ContainerId::from("1"));
        assert_eq!(b.id, ContainerId::from("2"));
        assert_eq!(c.id, ContainerId::from("3"));
    }

    #[test]
    /// Sort by header: image
    fn test_app_data_set_sort_by_header_image() {
        let (_ids, containers) = gen_containers();

        let mut app_data = gen_appdata(&containers);

        let result = app_data.get_container_items();
        assert_eq!(result, &containers);

        // descending
        app_data.set_sorted(Some((Header::Image, SortedOrder::Desc)));
        let result = app_data.get_container_items();
        let (a, b, c) = (&result[0], &result[1], &result[2]);
        assert_eq!(a.id, ContainerId::from("3"));
        assert_eq!(b.id, ContainerId::from("2"));
        assert_eq!(c.id, ContainerId::from("1"));

        // ascending
        app_data.set_sorted(Some((Header::Image, SortedOrder::Asc)));
        let result = app_data.get_container_items();
        let (a, b, c) = (&result[0], &result[1], &result[2]);
        assert_eq!(a.id, ContainerId::from("1"));
        assert_eq!(b.id, ContainerId::from("2"));
        assert_eq!(c.id, ContainerId::from("3"));
    }

    #[test]
    /// Sort by header: rx
    fn test_app_data_set_sort_by_header_rx() {
        let (_ids, containers) = gen_containers();

        let mut app_data = gen_appdata(&containers);

        let result = app_data.get_container_items();
        assert_eq!(result, &containers);

        if let Some(i) = app_data.get_container_by_id(&ContainerId::from("1")) {
            i.rx = ByteStats::new(40);
        }
        if let Some(i) = app_data.get_container_by_id(&ContainerId::from("2")) {
            i.rx = ByteStats::new(80);
        }
        if let Some(i) = app_data.get_container_by_id(&ContainerId::from("3")) {
            i.rx = ByteStats::new(2);
        }

        // descending
        app_data.set_sorted(Some((Header::Rx, SortedOrder::Desc)));
        let result = app_data.get_container_items();
        let (a, b, c) = (&result[0], &result[1], &result[2]);
        assert_eq!(a.id, ContainerId::from("2"));
        assert_eq!(b.id, ContainerId::from("1"));
        assert_eq!(c.id, ContainerId::from("3"));

        // ascending
        app_data.set_sorted(Some((Header::Rx, SortedOrder::Asc)));
        let result = app_data.get_container_items();
        let (a, b, c) = (&result[0], &result[1], &result[2]);
        assert_eq!(a.id, ContainerId::from("3"));
        assert_eq!(b.id, ContainerId::from("1"));
        assert_eq!(c.id, ContainerId::from("2"));
    }

    #[test]
    /// Sort by header: tx
    fn test_app_data_set_sort_by_header_tx() {
        let (_ids, containers) = gen_containers();

        let mut app_data = gen_appdata(&containers);

        let result = app_data.get_container_items();
        assert_eq!(result, &containers);

        if let Some(i) = app_data.get_container_by_id(&ContainerId::from("1")) {
            i.rx = ByteStats::new(400);
        }
        if let Some(i) = app_data.get_container_by_id(&ContainerId::from("2")) {
            i.rx = ByteStats::new(80);
        }
        if let Some(i) = app_data.get_container_by_id(&ContainerId::from("3")) {
            i.rx = ByteStats::new(83);
        }

        // descending
        app_data.set_sorted(Some((Header::Rx, SortedOrder::Desc)));
        let result = app_data.get_container_items();
        let (a, b, c) = (&result[0], &result[1], &result[2]);
        assert_eq!(a.id, ContainerId::from("1"));
        assert_eq!(b.id, ContainerId::from("3"));
        assert_eq!(c.id, ContainerId::from("2"));

        // ascending
        app_data.set_sorted(Some((Header::Rx, SortedOrder::Asc)));
        let result = app_data.get_container_items();
        let (a, b, c) = (&result[0], &result[1], &result[2]);
        assert_eq!(a.id, ContainerId::from("2"));
        assert_eq!(b.id, ContainerId::from("3"));
        assert_eq!(c.id, ContainerId::from("1"));
    }

    #[test]
    /// Sort by header when selected headers match
    fn test_app_data_set_sort_by_header_match() {
        let (_ids, containers) = gen_containers();

        let mut app_data = gen_appdata(&containers);

        let result = app_data.get_container_items();
        assert_eq!(result, &containers);

        // descending
        app_data.set_sorted(Some((Header::Rx, SortedOrder::Desc)));
        let result = app_data.get_container_items();
        let (a, b, c) = (&result[0], &result[1], &result[2]);
        assert_eq!(a.id, ContainerId::from("3"));
        assert_eq!(b.id, ContainerId::from("2"));
        assert_eq!(c.id, ContainerId::from("1"));

        // ascending
        app_data.set_sorted(Some((Header::Rx, SortedOrder::Asc)));
        let result = app_data.get_container_items();
        let (a, b, c) = (&result[0], &result[1], &result[2]);
        assert_eq!(a.id, ContainerId::from("1"));
        assert_eq!(b.id, ContainerId::from("2"));
        assert_eq!(c.id, ContainerId::from("3"));
    }

    #[test]
    /// reset sorted
    fn test_app_data_reset_sorted() {
        let (_ids, containers) = gen_containers();

        let mut app_data = gen_appdata(&containers);

        let result = app_data.get_container_items();
        assert_eq!(result, &containers);

        if let Some(i) = app_data.get_container_by_id(&ContainerId::from("1")) {
            i.rx = ByteStats::new(400);
        }
        if let Some(i) = app_data.get_container_by_id(&ContainerId::from("2")) {
            i.rx = ByteStats::new(80);
        }
        if let Some(i) = app_data.get_container_by_id(&ContainerId::from("3")) {
            i.rx = ByteStats::new(83);
        }

        app_data.set_sorted(Some((Header::Rx, SortedOrder::Asc)));
        let result = app_data.get_container_items();
        let (a, b, c) = (&result[0], &result[1], &result[2]);
        assert_eq!(a.id, ContainerId::from("2"));
        assert_eq!(b.id, ContainerId::from("3"));
        assert_eq!(c.id, ContainerId::from("1"));

        app_data.set_sorted(None);
        let result = app_data.get_container_items();
        let (a, b, c) = (&result[0], &result[1], &result[2]);
        assert_eq!(a.id, ContainerId::from("1"));
        assert_eq!(b.id, ContainerId::from("2"));
        assert_eq!(c.id, ContainerId::from("3"));
    }

    // **************** //
    // Container state  //
    // **************** //

    #[test]
    /// Get len of current containers vec
    fn test_app_data_get_container_len() {
        let (_ids, containers) = gen_containers();
        let app_data = gen_appdata(&containers);
        assert_eq!(app_data.get_container_len(), 3);
    }

    #[test]
    /// Select the first container
    fn test_app_data_containers_start() {
        let (_ids, containers) = gen_containers();
        let mut app_data = gen_appdata(&containers);

        // No container selected
        let result = app_data.get_container_state();
        assert_eq!(result.selected(), None);
        assert_eq!(result.offset(), 0);

        // First container selected
        app_data.containers_start();
        let result = app_data.get_container_state();
        assert_eq!(result.selected(), Some(0));
        assert_eq!(result.offset(), 0);

        let result = app_data.get_selected_container_id();
        assert_eq!(result, Some(ContainerId::from("1")));
        let result = app_data.get_selected_container_id_state_name();
        assert_eq!(
            result,
            Some((
                ContainerId::from("1"),
                State::Running,
                "container_1".to_owned()
            ))
        );

        // Calling previous when at start has no effect
        app_data.containers_previous();
        let result = app_data.get_selected_container_id();
        assert_eq!(result, Some(ContainerId::from("1")));
        let result = app_data.get_selected_container_id_state_name();
        assert_eq!(
            result,
            Some((
                ContainerId::from("1"),
                State::Running,
                "container_1".to_owned()
            ))
        );
    }

    #[test]
    /// advance container list state by one
    /// get get_selected_container_id() & get_selected_container_id_state_name() return valid Some data
    fn test_app_data_containers_next() {
        let (_ids, containers) = gen_containers();
        let mut app_data = gen_appdata(&containers);

        // Advance list state by 1
        app_data.containers_start();
        app_data.containers_next();

        let result = app_data.get_container_state();
        assert_eq!(result.selected(), Some(1));
        assert_eq!(result.offset(), 0);

        let result = app_data.get_selected_container_id();
        assert_eq!(result, Some(ContainerId::from("2")));
        let result = app_data.get_selected_container_id_state_name();
        assert_eq!(
            result,
            Some((
                ContainerId::from("2"),
                State::Running,
                "container_2".to_owned()
            ))
        );
    }

    #[test]
    /// advance container list state to the end
    /// get get_selected_container_id() & get_selected_container_id_state_name() return valid Some data
    fn test_app_data_containers_end() {
        let (_ids, containers) = gen_containers();
        let mut app_data = gen_appdata(&containers);

        app_data.containers_end();
        let result = app_data.get_container_state();
        assert_eq!(result.selected(), Some(2));
        assert_eq!(result.offset(), 0);

        let result = app_data.get_selected_container_id();
        assert_eq!(result, Some(ContainerId::from("3")));
        let result = app_data.get_selected_container_id_state_name();
        assert_eq!(
            result,
            Some((
                ContainerId::from("3"),
                State::Running,
                "container_3".to_owned()
            ))
        );

        // Calling previous when at end has no effect
        app_data.containers_next();
        let result = app_data.get_selected_container_id();
        assert_eq!(result, Some(ContainerId::from("3")));
        let result = app_data.get_selected_container_id_state_name();
        assert_eq!(
            result,
            Some((
                ContainerId::from("3"),
                State::Running,
                "container_3".to_owned()
            ))
        );
    }

    #[test]
    /// go to previous container
    fn test_app_data_containers_prev() {
        let (_ids, containers) = gen_containers();
        let mut app_data = gen_appdata(&containers);

        app_data.containers_end();
        app_data.containers_previous();
        let result = app_data.get_container_state();
        assert_eq!(result.selected(), Some(1));
        assert_eq!(result.offset(), 0);
    }

    #[test]
    /// Get the currently selected container
    fn test_app_data_get_selected_container() {
        let (_ids, mut containers) = gen_containers();
        let mut app_data = gen_appdata(&containers);

        let result = app_data.get_selected_container();
        assert_eq!(result, None);

        app_data.containers.start();
        app_data.containers.next();

        let result = app_data.get_selected_container();
        assert_eq!(result, Some(&containers[1]));

        // As above, but now as mut
        let result = app_data.get_mut_selected_container();
        assert_eq!(result, Some(&mut containers[1]));
    }

    #[test]
    /// Get mut container by id
    fn test_app_data_get_container_by_id() {
        let (_ids, mut containers) = gen_containers();
        let mut app_data = gen_appdata(&containers);

        let result = app_data.get_container_by_id(&ContainerId::from("2"));
        assert_eq!(result, Some(&mut containers[1]));
    }

    #[test]
    /// Get just the containers name by id
    fn test_app_data_get_container_name_by_id() {
        let (_ids, containers) = gen_containers();
        let mut app_data = gen_appdata(&containers);

        let result = app_data.get_container_name_by_id(&ContainerId::from("2"));
        assert_eq!(result, Some(ContainerName::from("container_2")));
    }

    #[test]
    /// Get the id of the currently selected container
    fn test_app_data_get_selected_container_id() {
        let (_ids, containers) = gen_containers();
        let mut app_data = gen_appdata(&containers);
        app_data.containers_end();

        let result = app_data.get_selected_container_id();
        assert_eq!(result, Some(ContainerId::from("3")));
    }

    #[test]
    fn test_app_data_get_selected_container_id_state_name() {
        let (_ids, containers) = gen_containers();
        let mut app_data = gen_appdata(&containers);
        app_data.containers_end();

        let result = app_data.get_selected_container_id_state_name();
        assert_eq!(
            result,
            Some((
                ContainerId::from("3"),
                State::Running,
                "container_3".to_owned()
            ))
        );
    }

    // ************** //
    // DockerControls //
    // ************** //

    #[test]
    /// Docker commands returned correctly
    fn test_app_data_selected_docker_command() {
        let (_ids, containers) = gen_containers();
        let mut app_data = gen_appdata(&containers);

        // No commands when no container selected
        let result = app_data.selected_docker_controls();
        assert!(result.is_none());

        // Correct commands returned
        app_data.containers_start();
        app_data.docker_controls_start();

        let result = app_data.selected_docker_controls();
        assert_eq!(result, Some(DockerControls::Pause));
    }

    #[test]
    /// Docker command next works
    fn test_app_data_selected_docker_command_next() {
        let (_ids, containers) = gen_containers();
        let mut app_data = gen_appdata(&containers);
        app_data.containers_start();
        app_data.docker_controls_start();
        app_data.docker_controls_next();

        let result = app_data.selected_docker_controls();
        assert_eq!(result, Some(DockerControls::Restart));
    }

    #[test]
    /// Dockercommand end works, and next has no effect when at end
    fn test_app_data_selected_docker_command_end() {
        let (_ids, containers) = gen_containers();
        let mut app_data = gen_appdata(&containers);
        app_data.containers_start();
        app_data.docker_controls_end();

        let result = app_data.selected_docker_controls();
        assert_eq!(result, Some(DockerControls::Delete));

        // Next has no effect when at end
        app_data.docker_controls_next();
        let result = app_data.selected_docker_controls();
        assert_eq!(result, Some(DockerControls::Delete));
    }

    #[test]
    /// Docker commands previous works, and has no effect when at start
    fn test_app_data_selected_docker_command_previous() {
        let (_ids, containers) = gen_containers();
        let mut app_data = gen_appdata(&containers);
        app_data.containers_start();
        app_data.docker_controls_end();
        app_data.docker_controls_previous();

        let result = app_data.selected_docker_controls();
        assert_eq!(result, Some(DockerControls::Stop));

        // previous has no effect when at start
        app_data.docker_controls_start();
        app_data.docker_controls_previous();
        let result = app_data.selected_docker_controls();
        assert_eq!(result, Some(DockerControls::Pause));
    }

    #[test]
    /// DockerCommands get correct controls dependant on container state
    fn test_app_data_get_control_items() {
        let test_state = |state: State, expected: &mut Vec<DockerControls>| {
            let gen_item_state = |state: State| {
                ContainerItem::new(
                    1,
                    ContainerId::from("1"),
                    "image_1".to_owned(),
                    false,
                    "container_1".to_owned(),
                    vec![],
                    state,
                    "Up 1 hour".to_owned(),
                )
            };
            let mut app_data = gen_appdata(&[gen_item_state(state)]);
            app_data.containers_start();
            app_data.docker_controls_start();

            let result = app_data.get_control_items();
            assert_eq!(result, Some(expected));
        };

        test_state(
            State::Dead,
            &mut vec![
                DockerControls::Start,
                DockerControls::Restart,
                DockerControls::Delete,
            ],
        );
        test_state(
            State::Exited,
            &mut vec![
                DockerControls::Start,
                DockerControls::Restart,
                DockerControls::Delete,
            ],
        );
        test_state(
            State::Paused,
            &mut vec![
                DockerControls::Resume,
                DockerControls::Stop,
                DockerControls::Delete,
            ],
        );
        test_state(State::Removing, &mut vec![DockerControls::Delete]);
        test_state(
            State::Restarting,
            &mut vec![DockerControls::Stop, DockerControls::Delete],
        );
        test_state(
            State::Running,
            &mut vec![
                DockerControls::Pause,
                DockerControls::Restart,
                DockerControls::Stop,
                DockerControls::Delete,
            ],
        );
        test_state(State::Unknown, &mut vec![DockerControls::Delete]);
    }

    // ****** //
    // Filter //
    // ****** //

    #[test]
    /// Data is filtered correctly by name
    fn test_app_data_filter_by_name() {
        let (_, containers) = gen_containers();

        let mut app_data = gen_appdata(&containers);

        assert!(app_data.get_filter_term().is_none());

        let pre_len = app_data.containers.items.len();
        app_data.filter_term_push('_');
        app_data.filter_term_push('2');

        assert_eq!(app_data.get_filter_term(), Some(&"_2".to_string()));

        app_data.filter_containers();
        let post_len = app_data.containers.items.len();
        assert!(pre_len != post_len);
        assert_eq!(post_len, 1);

        // Can insert checks against the current filter term
        // todo!("fix me");
        assert!(app_data.can_insert(&containers[1]));
        assert!(!app_data.can_insert(&containers[0]));
        assert!(!app_data.can_insert(&containers[2]));
    }

    #[test]
    /// Data is filtered correctly by image
    fn test_app_data_filter_by_image() {
        let (_, containers) = gen_containers();

        let mut app_data = gen_appdata(&containers);

        assert!(app_data.get_filter_term().is_none());

        let pre_len = app_data.containers.items.len();
        for c in ['i', 'm', 'a', 'g', 'e', '_', '2'] {
            app_data.filter_term_push(c);
        }
        // app_data.filter_term_push('2');
        app_data.filter_by_next();

        assert_eq!(app_data.get_filter_by(), FilterBy::Image);
        assert_eq!(app_data.get_filter_term(), Some(&"image_2".to_string()));

        app_data.filter_containers();
        let post_len = app_data.containers.items.len();
        assert!(pre_len != post_len);
        assert_eq!(post_len, 1);

        assert!(!app_data.can_insert(&containers[0]));
        assert!(app_data.can_insert(&containers[1]));
        assert!(!app_data.can_insert(&containers[2]));
    }

    #[test]
    /// Data is filtered correctly by status
    fn test_app_data_filter_by_status() {
        let (_, mut containers) = gen_containers();
        "Exited".clone_into(&mut containers[0].status);
        let mut app_data = gen_appdata(&containers);

        assert!(app_data.get_filter_term().is_none());

        let pre_len = app_data.containers.items.len();
        app_data.filter_term_push('x');

        app_data.filter_by_next();
        app_data.filter_by_next();

        assert_eq!(app_data.get_filter_by(), FilterBy::Status);
        assert_eq!(app_data.get_filter_term(), Some(&"x".to_string()));

        app_data.filter_containers();
        let post_len = app_data.containers.items.len();
        assert!(pre_len != post_len);
        assert_eq!(post_len, 1);

        assert!(app_data.can_insert(&containers[0]));
        assert!(!app_data.can_insert(&containers[1]));
        assert!(!app_data.can_insert(&containers[2]));
    }

    #[test]
    /// Data is filtered correctly by all
    fn test_app_data_filter_by_all() {
        let (_, mut containers) = gen_containers();
        "Exited".clone_into(&mut containers[0].status);
        let mut app_data = gen_appdata(&containers);

        assert!(app_data.get_filter_term().is_none());

        let pre_len = app_data.containers.items.len();
        app_data.filter_term_push('x');

        app_data.filter_by_next();
        app_data.filter_by_next();
        app_data.filter_by_next();

        assert_eq!(app_data.get_filter_by(), FilterBy::All);
        assert_eq!(app_data.get_filter_term(), Some(&"x".to_string()));

        app_data.filter_containers();
        let post_len = app_data.containers.items.len();
        assert!(pre_len != post_len);
        assert_eq!(post_len, 1);

        assert!(app_data.can_insert(&containers[0]));
        assert!(!app_data.can_insert(&containers[1]));
        assert!(!app_data.can_insert(&containers[2]));
    }

    #[test]
    /// Data is filtered correctly after various next() and previous() commands
    fn test_app_data_filter_prev() {
        let (_, mut containers) = gen_containers();
        "Exited".clone_into(&mut containers[0].status);
        let mut app_data = gen_appdata(&containers);

        assert!(app_data.get_filter_term().is_none());

        let pre_len = app_data.containers.items.len();
        app_data.filter_term_push('x');

        app_data.filter_by_next();
        app_data.filter_by_next();

        assert_eq!(app_data.get_filter_by(), FilterBy::Status);
        assert_eq!(app_data.get_filter_term(), Some(&"x".to_string()));

        app_data.filter_containers();
        let post_len = app_data.containers.items.len();
        assert!(pre_len != post_len);
        assert_eq!(post_len, 1);

        assert!(app_data.can_insert(&containers[0]));
        assert!(!app_data.can_insert(&containers[1]));
        assert!(!app_data.can_insert(&containers[2]));

        app_data.filter_by_prev();
        assert_eq!(app_data.get_filter_by(), FilterBy::Image);
        assert_eq!(app_data.get_filter_term(), Some(&"x".to_string()));

        app_data.filter_containers();
        let post_len = app_data.containers.items.len();
        assert!(pre_len != post_len);
        assert_eq!(post_len, 0);

        assert!(!app_data.can_insert(&containers[0]));
        assert!(!app_data.can_insert(&containers[1]));
        assert!(!app_data.can_insert(&containers[2]));
    }

    // **** //
    // Logs //
    // **** //

    #[test]
    /// log title string generated correctly
    fn test_app_data_get_log_title() {
        let (ids, containers) = gen_containers();

        let mut app_data = gen_appdata(&containers);

        // No container selected select
        let result = app_data.get_log_title();
        assert_eq!(result, "");

        // No logs
        app_data.containers.start();
        let result = app_data.get_log_title();
        assert_eq!(result, " - container_1");

        // On last line of logs
        let logs = (1..=3).map(|i| format!("{i}")).collect::<Vec<_>>();
        app_data.update_log_by_id(logs, &ids[0]);
        let result = app_data.get_log_title();
        assert_eq!(result, " 3/3 - container_1");

        // Change log state to no longer be at the end
        app_data.log_previous();
        let result = app_data.get_log_title();
        assert_eq!(result, " 2/3 - container_1");
    }

    #[test]
    /// log title string generated correctly after container change
    fn test_app_data_get_log_title_after_container_change() {
        let (ids, containers) = gen_containers();
        let mut app_data = gen_appdata(&containers);

        // No container selected select
        let result = app_data.get_log_title();
        assert_eq!(result, "");

        app_data.containers_start();

        let result = app_data.get_log_title();
        assert_eq!(result, " - container_1");

        // change container
        app_data.containers_next();
        let result = app_data.get_log_title();
        assert_eq!(result, " - container_2");

        // On last line of logs
        let logs = (1..=3).map(|i| format!("{i}")).collect::<Vec<_>>();
        app_data.update_log_by_id(logs, &ids[1]);
        let result = app_data.get_log_title();
        assert_eq!(result, " 3/3 - container_2");

        // Change log state to no longer be at the end
        app_data.log_previous();
        let result = app_data.get_log_title();
        assert_eq!(result, " 2/3 - container_2");
    }

    #[test]
    /// update logs by id works
    fn test_app_data_update_log_by_id() {
        let (ids, containers) = gen_containers();
        let mut app_data = gen_appdata(&containers);

        // No container selected select
        let result = app_data.get_log_title();
        assert_eq!(result, "");

        app_data.containers_start();
        let logs = (1..=3).map(|i| format!("{i} {i}")).collect::<Vec<_>>();

        app_data.update_log_by_id(logs, &ids[0]);
        // app_data.log_start();

        let result = app_data.get_log_state();
        assert!(result.is_some());
        assert_eq!(result.as_ref().unwrap().selected(), Some(2));
        assert_eq!(result.unwrap().offset(), 0);

        let result = app_data.get_logs();
        assert_eq!(result.len(), 3);

        let result = app_data.get_log_title();
        assert_eq!(result, " 3/3 - container_1");
    }

    #[test]
    /// logs state reset to start
    fn test_app_data_logs_start() {
        let (ids, containers) = gen_containers();
        let mut app_data = gen_appdata(&containers);
        let logs = (1..=3).map(|i| format!("{i} {i}")).collect::<Vec<_>>();
        app_data.containers_start();
        app_data.update_log_by_id(logs, &ids[0]);

        app_data.log_start();

        let result = app_data.get_log_state();
        assert!(result.is_some());
        assert_eq!(result.as_ref().unwrap().selected(), Some(0));
        assert_eq!(result.unwrap().offset(), 0);

        let result = app_data.get_log_title();
        assert_eq!(result, " 1/3 - container_1");
    }

    #[test]
    /// logs state end goes to the end of the logs list
    fn test_app_data_logs_end() {
        let (ids, containers) = gen_containers();
        let mut app_data = gen_appdata(&containers);
        let logs = (1..=3).map(|i| format!("{i} {i}")).collect::<Vec<_>>();
        app_data.containers_start();
        app_data.update_log_by_id(logs, &ids[0]);

        app_data.log_start();

        let result = app_data.get_log_state();
        assert!(result.is_some());
        assert_eq!(result.as_ref().unwrap().selected(), Some(0));
        assert_eq!(result.unwrap().offset(), 0);

        let result = app_data.get_log_title();
        assert_eq!(result, " 1/3 - container_1");

        app_data.log_end();
        let result = app_data.get_log_state();
        assert!(result.is_some());
        assert_eq!(result.as_ref().unwrap().selected(), Some(2));
        assert_eq!(result.unwrap().offset(), 0);

        let result = app_data.get_log_title();
        assert_eq!(result, " 3/3 - container_1");
    }

    #[test]
    /// logs state next works
    /// At end has no effect
    fn test_app_data_logs_next() {
        let (ids, containers) = gen_containers();
        let mut app_data = gen_appdata(&containers);
        let logs = (1..=3).map(|i| format!("{i} {i}")).collect::<Vec<_>>();
        app_data.containers_start();
        app_data.update_log_by_id(logs, &ids[0]);

        app_data.log_start();

        let result = app_data.get_log_state();
        assert!(result.is_some());
        assert_eq!(result.as_ref().unwrap().selected(), Some(0));
        assert_eq!(result.unwrap().offset(), 0);

        let result = app_data.get_log_title();
        assert_eq!(result, " 1/3 - container_1");

        app_data.log_next();

        let result = app_data.get_log_state();
        assert!(result.is_some());
        assert_eq!(result.as_ref().unwrap().selected(), Some(1));
        assert_eq!(result.unwrap().offset(), 0);

        let result = app_data.get_log_title();
        assert_eq!(result, " 2/3 - container_1");

        app_data.log_next();
        let result = app_data.get_log_state();
        assert!(result.is_some());
        assert_eq!(result.as_ref().unwrap().selected(), Some(2));
        assert_eq!(result.unwrap().offset(), 0);

        let result = app_data.get_log_title();
        assert_eq!(result, " 3/3 - container_1");
        app_data.log_next();

        let result = app_data.get_log_state();
        assert!(result.is_some());
        assert_eq!(result.as_ref().unwrap().selected(), Some(2));
        assert_eq!(result.unwrap().offset(), 0);

        let result = app_data.get_log_title();
        assert_eq!(result, " 3/3 - container_1");
    }

    #[test]
    /// logs state previous works
    /// previous at start has no effect
    fn test_app_data_logs_previous() {
        let (ids, containers) = gen_containers();
        let mut app_data = gen_appdata(&containers);
        let logs = (1..=3).map(|i| format!("{i} {i}")).collect::<Vec<_>>();
        app_data.containers_start();
        app_data.update_log_by_id(logs, &ids[0]);

        app_data.log_end();

        let result = app_data.get_log_state();
        assert!(result.is_some());
        assert_eq!(result.as_ref().unwrap().selected(), Some(2));
        assert_eq!(result.unwrap().offset(), 0);

        let result = app_data.get_log_title();
        assert_eq!(result, " 3/3 - container_1");

        app_data.log_previous();

        let result = app_data.get_log_state();
        assert!(result.is_some());
        assert_eq!(result.as_ref().unwrap().selected(), Some(1));
        assert_eq!(result.unwrap().offset(), 0);
        let result = app_data.get_log_title();
        assert_eq!(result, " 2/3 - container_1");

        app_data.log_previous();
        let result = app_data.get_log_state();
        assert!(result.is_some());
        assert_eq!(result.as_ref().unwrap().selected(), Some(0));
        assert_eq!(result.unwrap().offset(), 0);
        let result = app_data.get_log_title();
        assert_eq!(result, " 1/3 - container_1");

        app_data.log_previous();
        let result = app_data.get_log_state();
        assert!(result.is_some());
        assert_eq!(result.as_ref().unwrap().selected(), Some(0));
        assert_eq!(result.unwrap().offset(), 0);
        let result = app_data.get_log_title();
        assert_eq!(result, " 1/3 - container_1");
    }

    // ********** //
    // Chart data //
    // ********** //

    #[test]
    /// Chart data returned correctly
    fn test_app_data_get_chart_data() {
        let (_ids, containers) = gen_containers();
        let mut app_data = gen_appdata(&containers);

        let result = app_data.get_chart_data();
        assert!(result.is_none());

        app_data.containers_start();

        if let Some(item) = app_data.get_container_by_id(&ContainerId::from("1")) {
            item.cpu_stats = VecDeque::from([CpuStats::new(1.1), CpuStats::new(1.2)]);
            item.mem_stats = VecDeque::from([ByteStats::new(1), ByteStats::new(2)]);
        }

        let result = app_data.get_chart_data();
        assert_eq!(
            result,
            Some((
                (
                    vec![(0.0, 1.1), (1.0, 1.2)],
                    CpuStats::new(1.2),
                    State::Running
                ),
                (
                    vec![(0.0, 1.0), (1.0, 2.0)],
                    ByteStats::new(2),
                    State::Running
                )
            ))
        );
    }

    // ************* //
    // Header Widths //
    // ************* //

    #[test]
    /// Header widths return correctly
    fn test_app_data_get_width() {
        let (_ids, containers) = gen_containers();
        let app_data = gen_appdata(&containers);

        let result = app_data.get_width();
        let expected = Columns {
            name: (Header::Name, 11),
            state: (Header::State, 11),
            status: (Header::Status, 16),
            cpu: (Header::Cpu, 7),
            mem: (Header::Memory, 7, 7),
            id: (Header::Id, 8),
            image: (Header::Image, 7),
            net_rx: (Header::Rx, 7),
            net_tx: (Header::Tx, 7),
        };
        assert_eq!(result, expected);
    }

    #[test]
    /// Header widths return correctly when some containers hidden
    fn test_app_data_get_width_filtered() {
        let (_ids, mut containers) = gen_containers();
        containers[0].name = ContainerName::from("some_longer_name_with_filter");
        let mut app_data = gen_appdata(&containers);

        let result = app_data.get_width();
        let expected = Columns {
            name: (Header::Name, 28),
            state: (Header::State, 11),
            status: (Header::Status, 16),
            cpu: (Header::Cpu, 7),
            mem: (Header::Memory, 7, 7),
            id: (Header::Id, 8),
            image: (Header::Image, 7),
            net_rx: (Header::Rx, 7),
            net_tx: (Header::Tx, 7),
        };

        assert_eq!(result, expected);
        app_data.filter_term_push('c');
        app_data.filter_containers();
        assert_eq!(result, expected);
    }

    // ***** //
    // Ports //
    // ***** //

    #[test]
    /// Returns selected containers ports ordered by private ip
    fn test_app_data_get_selected_ports() {
        let (_ids, containers) = gen_containers();
        let mut app_data = gen_appdata(&containers);

        app_data.containers.items[0].ports.push(ContainerPorts {
            ip: None,
            private: 10,
            public: Some(1),
        });
        app_data.containers.items[0].ports.push(ContainerPorts {
            ip: None,
            private: 11,
            public: Some(3),
        });
        app_data.containers.items[0].ports.push(ContainerPorts {
            ip: None,
            private: 4,
            public: Some(2),
        });

        // No containers selected
        let result = app_data.get_selected_ports();
        assert!(result.is_none());

        // Selected container & ports
        app_data.containers_start();
        let result = app_data.get_selected_ports();

        assert_eq!(
            result,
            Some((
                vec![
                    ContainerPorts {
                        ip: None,
                        private: 4,
                        public: Some(2)
                    },
                    ContainerPorts {
                        ip: None,
                        private: 10,
                        public: Some(1)
                    },
                    ContainerPorts {
                        ip: None,
                        private: 11,
                        public: Some(3)
                    },
                    ContainerPorts {
                        ip: None,
                        private: 8001,
                        public: None
                    }
                ],
                State::Running
            ))
        );

        // Selected container & no ports
        app_data.containers_start();
        app_data.containers.items[0].ports = vec![];
        let result = app_data.get_selected_ports();

        assert_eq!(result, Some((vec![], State::Running)));
    }

    // ************** //
    // Update mtehods //
    // ************** //

    #[test]
    /// Update stats functioning
    fn test_app_data_update_stats() {
        let (ids, containers) = gen_containers();

        let mut app_data = gen_appdata(&containers);

        let result = app_data.get_container_items();
        assert_eq!(result[0], containers[0]);

        app_data.update_stats_by_id(&ids[0], Some(10.0), Some(10), 10, 10, 10);

        let result = app_data.get_container_items();
        assert_ne!(result[0], containers[0]);
        assert_eq!(result[0].cpu_stats, VecDeque::from([CpuStats::new(10.0)]));
        assert_eq!(result[0].mem_stats, VecDeque::from([ByteStats::new(10)]));
        assert_eq!(result[0].mem_limit, ByteStats::new(10));
        assert_eq!(result[0].rx, ByteStats::new(10));
        assert_eq!(result[0].tx, ByteStats::new(10));
    }

    #[test]
    /// Update stats functioning
    fn test_app_data_update_containers() {
        let (_ids, containers) = gen_containers();
        let mut app_data = gen_appdata(&containers);
        let result_pre = app_data.get_container_items().to_owned();
        let mut input = [
            gen_container_summary(1, "paused"),
            gen_container_summary(2, "dead"),
        ];

        app_data.update_containers(&mut input);
        let result_post = app_data.get_container_items().to_owned();
        assert_ne!(result_pre, result_post);
        assert_eq!(result_post[0].state, State::Paused);
        assert_eq!(result_post[1].state, State::Dead);
    }

    #[test]
    /// Update logs don't work if container is_oxker: true
    fn test_app_data_update_log_by_id_is_oxker() {
        let (ids, mut containers) = gen_containers();
        containers[0].is_oxker = true;
        let mut app_data = gen_appdata(&containers);
        let logs = (1..=3).map(|i| format!("{i} {i}")).collect::<Vec<_>>();

        app_data.update_log_by_id(logs, &ids[0]);
        app_data.log_start();

        let result = app_data.get_log_state();
        assert!(result.is_none());
    }
}
