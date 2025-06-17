use parking_lot::Mutex;
use ratatui::layout::{Constraint, Rect};
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
    time::Instant,
};
use tokio::task::JoinHandle;
use uuid::Uuid;

use crate::{
    app_data::{AppData, ContainerId, Header},
    exec::ExecMode,
};

use super::Rerender;

#[derive(Debug, Default, Clone, Copy, Eq, Hash, PartialEq)]
pub enum SelectablePanel {
    #[default]
    Containers,
    Commands,
    Logs,
}

impl SelectablePanel {
    pub const fn title(self) -> &'static str {
        match self {
            Self::Containers => "Containers",
            Self::Logs => "Logs",
            Self::Commands => "",
        }
    }
    pub const fn next(self) -> Self {
        match self {
            Self::Containers => Self::Commands,
            Self::Commands => Self::Logs,
            Self::Logs => Self::Containers,
        }
    }
    pub const fn prev(self) -> Self {
        match self {
            Self::Containers => Self::Logs,
            Self::Commands => Self::Containers,
            Self::Logs => Self::Commands,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Region {
    Panel(SelectablePanel),
    Header(Header),
    HelpPanel,
    Delete(DeleteButton),
}

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub enum DeleteButton {
    Confirm,
    Cancel,
}

#[allow(unused)]
#[derive(Debug, Clone, Copy)]
pub enum BoxLocation {
    TopLeft,
    TopCentre,
    TopRight,
    MiddleLeft,
    MiddleCentre,
    MiddleRight,
    BottomLeft,
    BottomCentre,
    BottomRight,
}

impl BoxLocation {
    /// Screen is divided into 3x3 sections
    pub const fn get_indexes(self) -> (usize, usize) {
        match self {
            Self::TopLeft => (0, 0),
            Self::TopCentre => (0, 1),
            Self::TopRight => (0, 2),
            Self::MiddleLeft => (1, 0),
            Self::MiddleCentre => (1, 1),
            Self::MiddleRight => (1, 2),
            Self::BottomLeft => (2, 0),
            Self::BottomCentre => (2, 1),
            Self::BottomRight => (2, 2),
        }
    }

    /// Get both the vertical and hoziztonal constrains
    pub const fn get_constraints(
        self,
        blank_horizontal: u16,
        blank_vertical: u16,
        text_lines: u16,
        text_width: u16,
    ) -> ([Constraint; 3], [Constraint; 3]) {
        (
            Self::get_horizontal_constraints(self, blank_horizontal, text_width),
            Self::get_vertical_constraints(self, blank_vertical, text_lines),
        )
    }

    const fn get_horizontal_constraints(
        self,
        blank_horizontal: u16,
        text_width: u16,
    ) -> [Constraint; 3] {
        match self {
            Self::TopLeft | Self::MiddleLeft | Self::BottomLeft => [
                Constraint::Min(text_width),
                Constraint::Max(blank_horizontal),
                Constraint::Max(blank_horizontal),
            ],
            Self::TopCentre | Self::MiddleCentre | Self::BottomCentre => [
                Constraint::Max(blank_horizontal),
                Constraint::Min(text_width),
                Constraint::Max(blank_horizontal),
            ],
            Self::TopRight | Self::MiddleRight | Self::BottomRight => [
                Constraint::Max(blank_horizontal),
                Constraint::Max(blank_horizontal),
                Constraint::Min(text_width),
            ],
        }
    }

    const fn get_vertical_constraints(
        self,
        blank_vertical: u16,
        number_lines: u16,
    ) -> [Constraint; 3] {
        match self {
            Self::TopLeft | Self::TopCentre | Self::TopRight => [
                Constraint::Min(number_lines),
                Constraint::Max(blank_vertical),
                Constraint::Max(blank_vertical),
            ],
            Self::MiddleLeft | Self::MiddleCentre | Self::MiddleRight => [
                Constraint::Max(blank_vertical),
                Constraint::Min(number_lines),
                Constraint::Max(blank_vertical),
            ],
            Self::BottomLeft | Self::BottomCentre | Self::BottomRight => [
                Constraint::Max(blank_vertical),
                Constraint::Max(blank_vertical),
                Constraint::Min(number_lines),
            ],
        }
    }
}

// loading animation frames
const FRAMES: [char; 10] = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
const FRAMES_LEN: u8 = 9;

/// The application gui state can be in multiple of these four states at the same time
/// Various functions (e.g input handler), operate differently depending upon current Status
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum Status {
    DeleteConfirm,
    DockerConnect,
    Error,
    Exec,
    Filter,
    Help,
    Init,
    Logs,
}

/// Global gui_state, stored in an Arc<Mutex>
#[derive(Debug)]
pub struct GuiState {
    delete_container_id: Option<ContainerId>,
    exec_mode: Option<ExecMode>,
    intersect_delete: HashMap<DeleteButton, Rect>,
    intersect_heading: HashMap<Header, Rect>,
    intersect_help: Option<Rect>,
    intersect_panel: HashMap<SelectablePanel, Rect>,
    loading_handle: Option<JoinHandle<()>>,
    loading_index: u8,
    loading_set: HashSet<Uuid>,
    log_height: u16,
    rerender: Arc<Rerender>,
    selected_panel: SelectablePanel,
    show_logs: bool,
    status: HashSet<Status>,
    pub info_box_text: Option<(String, Instant)>,
}
impl GuiState {
    pub fn new(redraw: &Arc<Rerender>, show_logs: bool) -> Self {
        Self {
            delete_container_id: None,
            exec_mode: None,
            info_box_text: None,
            intersect_delete: HashMap::new(),
            intersect_heading: HashMap::new(),
            intersect_help: None,
            intersect_panel: HashMap::new(),
            loading_handle: None,
            loading_index: 0,
            loading_set: HashSet::new(),
            log_height: 75,
            rerender: Arc::clone(redraw),
            selected_panel: SelectablePanel::default(),
            show_logs,
            status: HashSet::new(),
        }
    }
    /// Increase the height of the log panel, then rerender
    pub fn log_height_increase(&mut self) {
        if self.show_logs && self.log_height <= 75 {
            self.log_height = self.log_height.saturating_add(5);
            self.rerender.update();
        }
    }

    /// Reduce the height of the logs panel, then rerender
    /// Unselect logs panel if currently selected
    pub fn log_height_decrease(&mut self) {
        if self.show_logs {
            self.log_height = self.log_height.saturating_sub(5);
            if self.log_height == 0 && self.selected_panel == SelectablePanel::Logs {
                self.show_logs = false;
                self.selected_panel = SelectablePanel::Containers;
            }
            self.rerender.update();
        }
    }

    pub const fn get_show_logs(&self) -> bool {
        self.show_logs
    }

    pub fn toggle_show_logs(&mut self) {
        self.show_logs = !self.show_logs;
        if !self.show_logs && self.selected_panel == SelectablePanel::Logs {
            self.selected_panel = SelectablePanel::Containers;
        }
        self.rerender.update();
    }

    /// Set the log_height to zero, for now only used by tests
    #[cfg(test)]
    pub const fn log_height_zero(&mut self) {
        self.log_height = 0;
    }

    /// Get the log height, *should* be a u8 between 0 and 80, essentially a percentage
    pub const fn get_log_height(&self) -> u16 {
        self.log_height
    }

    /// Clear panels hash map, so on resize can fix the sizes for mouse clicks
    pub fn clear_area_map(&mut self) {
        self.intersect_panel.clear();
    }

    /// Get the currently selected panel
    pub const fn get_selected_panel(&self) -> SelectablePanel {
        self.selected_panel
    }

    /// Check if a given Rect (a clicked area of 1x1), interacts with any known panels
    pub fn check_panel_intersect(&mut self, rect: Rect) {
        if let Some(data) = self
            .intersect_panel
            .iter()
            .filter(|i| i.1.intersects(rect))
            .collect::<Vec<_>>()
            .first()
        {
            self.selected_panel = *data.0;
            self.rerender.update();
        }
    }

    /// Check if a given Rect (a clicked area of 1x1), interacts with any known delete button
    pub fn get_intersect_button(&self, rect: Rect) -> Option<DeleteButton> {
        self.intersect_delete
            .iter()
            .filter(|i| i.1.intersects(rect))
            .collect::<Vec<_>>()
            .first()
            .map(|data| *data.0)
    }

    /// Check if a given Rect (a clicked area of 1x1), interacts with any known panels
    pub fn get_intersect_header(&self, rect: Rect) -> Option<Header> {
        self.intersect_heading
            .iter()
            .filter(|i| i.1.intersects(rect))
            .collect::<Vec<_>>()
            .first()
            .map(|data| *data.0)
    }

    /// Check if a the "show/hide help" section has been clicked
    pub fn get_intersect_help(&self, rect: Rect) -> bool {
        self.intersect_help
            .as_ref()
            .is_some_and(|i| i.intersects(rect))
    }

    /// Insert, or updates header area panel into heading_map
    pub fn update_region_map(&mut self, region: Region, area: Rect) {
        match region {
            Region::Header(header) => {
                self.intersect_heading
                    .entry(header)
                    .and_modify(|w| *w = area)
                    .or_insert(area);
            }
            Region::Panel(panel) => {
                self.intersect_panel
                    .entry(panel)
                    .and_modify(|w| *w = area)
                    .or_insert(area);
            }
            Region::Delete(button) => {
                self.intersect_delete
                    .entry(button)
                    .and_modify(|w| *w = area)
                    .or_insert(area);
            }
            Region::HelpPanel => {
                self.intersect_help = Some(area);
            }
        }
    }

    /// Check if an ContainerId is set in the delete_container field
    pub fn get_delete_container(&self) -> Option<ContainerId> {
        self.delete_container_id.clone()
    }

    /// Set either a ContainerId, or None, to the delete_container field
    /// If Some, will also insert the DeleteConfirm status into self.status
    pub fn set_delete_container(&mut self, id: Option<ContainerId>) {
        if id.is_some() {
            self.status.insert(Status::DeleteConfirm);
        } else {
            self.intersect_delete.clear();
            self.status_del(Status::DeleteConfirm);
        }
        self.delete_container_id = id;
        self.rerender.update();
    }

    /// Return a copy of the Status HashSet
    pub fn get_status(&self) -> HashSet<Status> {
        self.status.clone()
    }

    /// Remove a gui_status into the current gui_status HashSet
    /// Remove exec mode & deleteConfirm is required
    pub fn status_del(&mut self, status: Status) {
        self.status.remove(&status);
        match status {
            Status::DeleteConfirm => {
                self.status.remove(&Status::DeleteConfirm);
            }
            Status::Exec => {
                self.exec_mode = None;
            }
            _ => (),
        }
        self.rerender.update();
    }

    /// Inset the ExecMode into self, and set the Status as exec
    /// Using StatusPush with Status::Exec won't insert into the hash map
    /// To force self.exec_mode to be set
    pub fn set_exec_mode(&mut self, mode: ExecMode) {
        self.exec_mode = Some(mode);
        self.status.insert(Status::Exec);
        self.rerender.update();
    }

    pub fn get_exec_mode(&self) -> Option<ExecMode> {
        self.exec_mode.clone()
    }

    /// Insert a gui_status into the current gui_status HashSet
    /// If the status is Exec, it won't get inserted, set_exec_mode() should be used instead
    pub fn status_push(&mut self, status: Status) {
        if status != Status::Exec {
            self.status.insert(status);
            self.rerender.update();
        }
    }

    /// Change to next selectable panel
    pub fn selectable_panel_next(&mut self, app_data: &Arc<Mutex<AppData>>) {
        self.selected_panel = self.selected_panel.next();
        if (app_data.lock().get_container_len() == 0
            && self.get_selected_panel() == SelectablePanel::Commands)
            || (self.log_height == 0 && self.get_selected_panel() == SelectablePanel::Logs)
        {
            self.selected_panel = self.selected_panel.next();
        }
        self.rerender.update();
    }

    /// Change to previous selectable panel
    pub fn selectable_panel_previous(&mut self, app_data: &Arc<Mutex<AppData>>) {
        self.selected_panel = self.selected_panel.prev();
        if (app_data.lock().get_container_len() == 0
            && self.get_selected_panel() == SelectablePanel::Commands)
            || (self.log_height == 0 && self.get_selected_panel() == SelectablePanel::Logs)
        {
            self.selected_panel = self.selected_panel.prev();
        }
        self.rerender.update();
    }

    /// Insert a new loading_uuid into HashSet, and advance the loading_index by one frame, or reset to 0 if at end of array
    pub fn next_loading(&mut self, uuid: Uuid) {
        if self.loading_index == FRAMES_LEN {
            self.loading_index = 0;
        } else {
            self.loading_index += 1;
        }
        self.loading_set.insert(uuid);
        self.rerender.update();
    }

    pub fn is_loading(&self) -> bool {
        !self.loading_set.is_empty()
    }
    /// If is_loading has any entries, return the char at FRAMES[index], else an empty char, which needs to take up the same space, hence ' '
    pub fn get_loading(&self) -> char {
        if self.is_loading() {
            FRAMES[usize::from(self.loading_index)]
        } else {
            ' '
        }
    }

    /// Animate the loading icon in its own Tokio thread
    /// This should only be able to executed once, rather than multiple spawns
    pub fn start_loading_animation(gui_state: &Arc<Mutex<Self>>, loading_uuid: Uuid) {
        if !gui_state.lock().is_loading() {
            let inner_state = Arc::clone(gui_state);
            gui_state.lock().loading_handle = Some(tokio::spawn(async move {
                loop {
                    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                    inner_state.lock().next_loading(loading_uuid);
                }
            }));
        }
        gui_state.lock().next_loading(loading_uuid);
    }

    /// Stop the loading_spin function, and reset gui loading status
    pub fn stop_loading_animation(&mut self, loading_uuid: Uuid) {
        self.loading_set.remove(&loading_uuid);
        self.rerender.update();
        if self.loading_set.is_empty() {
            self.loading_index = 0;
            if let Some(h) = &self.loading_handle {
                h.abort();
            }
            self.loading_handle = None;
        }
    }

    /// Set info box content
    pub fn set_info_box(&mut self, text: &str) {
        self.info_box_text = Some((text.to_owned(), std::time::Instant::now()));
        self.rerender.update();
    }

    /// Remove info box content
    pub fn reset_info_box(&mut self) {
        self.info_box_text = None;
        self.rerender.update();
    }
}
