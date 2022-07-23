use bollard::models::ContainerSummary;
use core::fmt;
use std::time::{SystemTime, UNIX_EPOCH};
use tui::widgets::ListItem;

mod container_state;

use crate::{app_error::AppError, parse_args::CliArgs, ui::log_sanitizer};
pub use container_state::*;

/// Global app_state, stored in an Arc<Mutex>
#[derive(Debug)]
pub struct AppData {
    args: CliArgs,
    error: Option<AppError>,
    logs_parsed: bool,
    pub containers: StatefulList<ContainerItem>,
    pub init: bool,
    pub show_error: bool,
    sorted_by: Option<(Header, SortedOrder)>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SortedOrder {
    Asc,
    Desc,
}

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
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

/// Convert errors into strings to display
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
        write!(f, "{:>x$}", disp, x = f.width().unwrap_or(1))
    }
}

impl AppData {
    pub fn get_sorted(&self) -> Option<(Header, SortedOrder)> {
        self.sorted_by.clone()
    }

    /// Change the sorted order, also set the selected container state to match new order
    pub fn set_sorted(&mut self, x: Option<(Header, SortedOrder)>) {
        self.sorted_by = x;
        let id = self.get_selected_container_id();
        self.sort_containers();
        self.containers.state.select(
            self.containers
                .items
                .iter()
                .position(|i| Some(i.id.to_owned()) == id),
        );
    }
    /// Generate a default app_state
    pub fn default(args: CliArgs) -> Self {
        Self {
            args,
            containers: StatefulList::new(vec![]),
            error: None,
            init: false,
            logs_parsed: false,
            show_error: false,
            sorted_by: None,
        }
    }

    // Current time as unix timestamp
    fn get_systemtime(&self) -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("In our known reality, this error should never occur")
            .as_secs()
    }

    /// Get the current select docker command
    /// So know which command to execute
    pub fn get_docker_command(&self) -> Option<DockerControls> {
        let mut output = None;
        if let Some(index) = self.containers.state.selected() {
            if let Some(control_index) = self.containers.items[index]
                .docker_controls
                .state
                .selected()
            {
                output =
                    Some(self.containers.items[index].docker_controls.items[control_index].clone())
            }
        }
        output
    }

    /// Change selected choice of docker commands of selected container
    pub fn docker_command_next(&mut self) {
        if let Some(index) = self.containers.state.selected() {
            self.containers.items[index].docker_controls.next()
        }
    }

    /// Change selected choice of docker commands of selected container
    pub fn docker_command_previous(&mut self) {
        if let Some(index) = self.containers.state.selected() {
            self.containers.items[index].docker_controls.previous()
        }
    }

    /// Change selected choice of docker commands of selected container
    pub fn docker_command_start(&mut self) {
        if let Some(index) = self.containers.state.selected() {
            self.containers.items[index].docker_controls.start()
        }
    }

    /// Change selected choice of docker commands of selected container
    pub fn docker_command_end(&mut self) {
        if let Some(index) = self.containers.state.selected() {
            self.containers.items[index].docker_controls.end()
        }
    }

    /// return single app_state error
    pub fn get_error(&self) -> Option<AppError> {
        self.error.clone()
    }

    /// remove single app_state error
    pub fn remove_error(&mut self) {
        self.error = None;
    }

    /// insert single app_state error
    pub fn set_error(&mut self, error: AppError) {
        self.error = Some(error);
    }

    /// Find the id of the currently selected container.
    /// If any containers on system, will always return a string.
    /// Only returns None when no containers found.
    pub fn get_selected_container_id(&self) -> Option<String> {
        let mut output = None;
        if let Some(index) = self.containers.state.selected() {
            let id = self
                .containers
                .items
                .iter()
                .skip(index)
                .take(1)
                .map(|i| i.id.to_owned())
                .collect::<String>();
            output = Some(id)
        }
        output
    }

    /// Sort the containers vec, based on a heading, either ascending or descending
    pub fn sort_containers(&mut self) {
        if let Some((head, so)) = self.sorted_by.as_ref() {
            match head {
                Header::State => match so {
                    SortedOrder::Desc => self
                        .containers
                        .items
                        .sort_by(|a, b| a.state.order().cmp(b.state.order())),
                    SortedOrder::Asc => self
                        .containers
                        .items
                        .sort_by(|a, b| b.state.order().cmp(a.state.order())),
                },
                Header::Status => match so {
                    SortedOrder::Asc => self
                        .containers
                        .items
                        .sort_by(|a, b| a.status.cmp(&b.status)),
                    SortedOrder::Desc => self
                        .containers
                        .items
                        .sort_by(|a, b| b.status.cmp(&a.status)),
                },
                Header::Cpu => match so {
                    SortedOrder::Asc => self
                        .containers
                        .items
                        .sort_by(|a, b| a.cpu_stats.back().cmp(&b.cpu_stats.back())),
                    SortedOrder::Desc => self
                        .containers
                        .items
                        .sort_by(|a, b| b.cpu_stats.back().cmp(&a.cpu_stats.back())),
                },
                Header::Memory => match so {
                    SortedOrder::Asc => self
                        .containers
                        .items
                        .sort_by(|a, b| a.mem_stats.back().cmp(&b.mem_stats.back())),
                    SortedOrder::Desc => self
                        .containers
                        .items
                        .sort_by(|a, b| b.mem_stats.back().cmp(&a.mem_stats.back())),
                },
                Header::Id => match so {
                    SortedOrder::Asc => self.containers.items.sort_by(|a, b| a.id.cmp(&b.id)),
                    SortedOrder::Desc => self.containers.items.sort_by(|a, b| b.id.cmp(&a.id)),
                },
                Header::Image => match so {
                    SortedOrder::Asc => self.containers.items.sort_by(|a, b| a.image.cmp(&b.image)),
                    SortedOrder::Desc => {
                        self.containers.items.sort_by(|a, b| b.image.cmp(&a.image))
                    }
                },
                Header::Name => match so {
                    SortedOrder::Asc => self.containers.items.sort_by(|a, b| a.name.cmp(&b.name)),
                    SortedOrder::Desc => self.containers.items.sort_by(|a, b| b.name.cmp(&a.name)),
                },
                Header::Rx => match so {
                    SortedOrder::Asc => self
                        .containers
                        .items
                        .sort_by(|a, b| a.net_rx.cmp(&b.net_rx)),
                    SortedOrder::Desc => self
                        .containers
                        .items
                        .sort_by(|a, b| b.net_rx.cmp(&a.net_rx)),
                },
                Header::Tx => match so {
                    SortedOrder::Asc => self
                        .containers
                        .items
                        .sort_by(|a, b| a.net_tx.cmp(&b.net_tx)),
                    SortedOrder::Desc => self
                        .containers
                        .items
                        .sort_by(|a, b| b.net_tx.cmp(&a.net_tx)),
                },
            }
        }
    }

    /// Find the index of the currently selected single log line
    pub fn get_selected_log_index(&self) -> Option<usize> {
        let mut output = None;
        if let Some(id) = self.get_selected_container_id() {
            if let Some(index) = self.containers.items.iter().position(|i| i.id == id) {
                output = Some(index);
            }
        }
        output
    }

    /// Get the title for log panel for selected container
    /// will be "logs x/x"
    pub fn get_log_title(&self) -> String {
        if let Some(index) = self.get_selected_log_index() {
            self.containers.items[index].logs.get_state_title()
        } else {
            String::from("")
        }
    }

    /// select next selected log line
    pub fn log_next(&mut self) {
        if let Some(index) = self.get_selected_log_index() {
            self.containers.items[index].logs.next()
        }
    }

    /// select previous selected log line
    pub fn log_previous(&mut self) {
        if let Some(index) = self.get_selected_log_index() {
            self.containers.items[index].logs.previous()
        }
    }

    /// select last selected log line
    pub fn log_end(&mut self) {
        if let Some(index) = self.get_selected_log_index() {
            self.containers.items[index].logs.end()
        }
    }

    /// select first selected log line
    pub fn log_start(&mut self) {
        if let Some(index) = self.get_selected_log_index() {
            self.containers.items[index].logs.start()
        }
    }

    pub fn initialised(&mut self, all_ids: &[(bool, String)]) -> bool {
        let count_is_running = all_ids.iter().filter(|i| i.0).count();
        let number_with_cpu_status = self
            .containers
            .items
            .iter()
            .filter(|i| !i.cpu_stats.is_empty())
            .count();
        self.logs_parsed && count_is_running == number_with_cpu_status
    }

    /// Just get the total number of containers
    pub fn get_container_len(&self) -> usize {
        self.containers.items.len()
    }

    /// Find the widths for the strings in the containers panel.
    /// So can display nicely and evenly
    pub fn get_width(&self) -> Columns {
        let mut output = Columns::new();
        let count = |x: &String| x.chars().count();

        for container in self.containers.items.iter() {
            let cpu_count = count(
                &container
                    .cpu_stats
                    .back()
                    .unwrap_or(&CpuStats::new(0.0))
                    .to_string(),
            );
            let mem_count = count(&format!(
                "{} / {}",
                container.mem_stats.back().unwrap_or(&ByteStats::new(0)),
                container.mem_limit
            ));

            let net_rx_count = count(&container.net_rx.to_string());
            let net_tx_count = count(&container.net_tx.to_string());
            let image_count = count(&container.image);
            let name_count = count(&container.name);
            let state_count = count(&container.state.to_string());
            let status_count = count(&container.status);

            if cpu_count > output.cpu.1 {
                output.cpu.1 = cpu_count;
            };
            if image_count > output.image.1 {
                output.image.1 = image_count;
            };
            if mem_count > output.mem.1 {
                output.mem.1 = mem_count;
            };
            if name_count > output.name.1 {
                output.name.1 = name_count;
            };
            if state_count > output.state.1 {
                output.state.1 = state_count;
            };
            if status_count > output.status.1 {
                output.status.1 = status_count;
            };
            if net_rx_count > output.net_rx.1 {
                output.net_rx.1 = net_rx_count;
            };
            if net_tx_count > output.net_tx.1 {
                output.net_tx.1 = net_tx_count;
            };
        }
        output
    }

    /// Get all containers ids
    pub fn get_all_ids(&self) -> Vec<String> {
        self.containers
            .items
            .iter()
            .map(|i| i.id.to_owned())
            .collect::<Vec<_>>()
    }

    /// find container given id
    fn get_container_by_id(&mut self, id: &str) -> Option<&mut ContainerItem> {
        self.containers.items.iter_mut().find(|i| i.id == id)
    }

    /// Update container mem, cpu, & network stats, in single function so only need to call .lock() once
    pub fn update_stats(
        &mut self,
        id: String,
        cpu_stat: Option<f64>,
        mem_stat: Option<u64>,
        mem_limit: u64,
        rx: u64,
        tx: u64,
    ) {
        if let Some(container) = self.get_container_by_id(&id) {
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

            container.net_rx.update(rx);
            container.net_tx.update(tx);
            container.mem_limit.update(mem_limit);
        }
    }

    /// Update, or insert, containers
    pub fn update_containers(&mut self, containers: &[ContainerSummary]) {
        let all_ids = self.get_all_ids();

        if !containers.is_empty() && self.containers.state.selected().is_none() {
            self.containers.start();
        }

        for (index, id) in all_ids.iter().enumerate() {
            if !containers
                .iter()
                .filter_map(|i| i.id.as_ref())
                .any(|x| x == id)
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

        for i in containers.iter() {
            if let Some(id) = i.id.as_ref() {
                let mut name = i
                    .names
                    .as_ref()
                    .unwrap_or(&vec!["".to_owned()])
                    .get(0)
                    .unwrap_or(&String::from(""))
                    .to_owned();
                if let Some(c) = name.chars().next() {
                    if c == '/' {
                        name.remove(0);
                    }
                }

                let state = State::from(i.state.as_ref().unwrap_or(&"dead".to_owned()).trim());
                let status = i
                    .status
                    .as_ref()
                    .unwrap_or(&"".to_owned())
                    .trim()
                    .to_owned();
                let image = i.image.as_ref().unwrap_or(&"".to_owned()).trim().to_owned();
                if let Some(current_container) = self.get_container_by_id(id) {
                    if current_container.name != name {
                        current_container.name = name
                    };
                    if current_container.status != status {
                        current_container.status = status
                    };
                    if current_container.state != state {
                        current_container.docker_controls.items = DockerControls::gen_vec(&state);

                        // Update the list state, needs to be None if the gen_vec returns an empty vec
                        match state {
                            State::Removing | State::Restarting | State::Unknown => {
                                current_container.docker_controls.state.select(None)
                            }
                            _ => current_container.docker_controls.start(),
                        };
                        current_container.state = state;
                    };
                    if current_container.image != image {
                        current_container.image = image
                    };
                } else {
                    let mut container =
                        ContainerItem::new(id.to_owned(), status, image, state, name);
                    container.logs.end();
                    self.containers.items.push(container);
                }
            }
        }
    }

    /// update logs of a given container, based on index not id
    pub fn update_log_by_index(&mut self, output: Vec<String>, index: usize) {
        let tz = self.get_systemtime();
        if let Some(container) = self.containers.items.get_mut(index) {
            container.last_updated = tz;
            let current_len = container.logs.items.len();
            output.iter().for_each(|i| {
                let lines = if self.args.color {
                    log_sanitizer::colorize_logs(i.to_owned())
                } else if self.args.raw {
                    log_sanitizer::raw(i.to_owned())
                } else {
                    log_sanitizer::remove_ansi(i.to_owned())
                };
                container.logs.items.push(ListItem::new(lines));
            });
            if container.logs.state.selected().is_none()
                || container.logs.state.selected().unwrap_or_default() + 1 == current_len
            {
                container.logs.end();
            }
        }
        self.logs_parsed = true;
    }

    /// Update all containers logs, should only be used on first initialisation
    pub fn update_all_logs(&mut self, all_logs: Vec<Vec<String>>) {
        for (index, output) in all_logs.into_iter().enumerate() {
            self.update_log_by_index(output, index);
        }
    }
}
