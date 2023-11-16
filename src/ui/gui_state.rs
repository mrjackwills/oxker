use parking_lot::Mutex;
use ratatui::layout::{Constraint, Rect};
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};
use tokio::task::JoinHandle;
use uuid::Uuid;

use crate::app_data::{ContainerId, Header};

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
    Delete(DeleteButton),
}

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub enum DeleteButton {
    Yes,
    No,
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
                Constraint::Max(text_width),
                Constraint::Max(blank_horizontal),
                Constraint::Max(blank_horizontal),
            ],
            Self::TopCentre | Self::MiddleCentre | Self::BottomCentre => [
                Constraint::Max(blank_horizontal),
                Constraint::Max(text_width),
                Constraint::Max(blank_horizontal),
            ],
            Self::TopRight | Self::MiddleRight | Self::BottomRight => [
                Constraint::Max(blank_horizontal),
                Constraint::Max(blank_horizontal),
                Constraint::Max(text_width),
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
                Constraint::Max(number_lines),
                Constraint::Max(blank_vertical),
                Constraint::Max(blank_vertical),
            ],
            Self::MiddleLeft | Self::MiddleCentre | Self::MiddleRight => [
                Constraint::Max(blank_vertical),
                Constraint::Max(number_lines),
                Constraint::Max(blank_vertical),
            ],
            Self::BottomLeft | Self::BottomCentre | Self::BottomRight => [
                Constraint::Max(blank_vertical),
                Constraint::Max(blank_vertical),
                Constraint::Max(number_lines),
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
    Exec,
    DeleteConfirm,
    DockerConnect,
    Error,
    Help,
    Init,
}

/// Global gui_state, stored in an Arc<Mutex>
#[derive(Debug, Default, Clone)]
pub struct GuiState {
    delete_container: Option<ContainerId>,
    delete_map: HashMap<DeleteButton, Rect>,
    heading_map: HashMap<Header, Rect>,
    is_loading: HashSet<Uuid>,
    loading_index: u8,
    panel_map: HashMap<SelectablePanel, Rect>,
    selected_panel: SelectablePanel,
    status: HashSet<Status>,
    pub info_box_text: Option<String>,
}
impl GuiState {
    /// Clear panels hash map, so on resize can fix the sizes for mouse clicks
    pub fn clear_area_map(&mut self) {
        self.panel_map.clear();
    }

    /// Get the currently selected panel
    pub const fn get_selected_panel(&self) -> SelectablePanel {
        self.selected_panel
    }

    /// Check if a given Rect (a clicked area of 1x1), interacts with any known panels
    pub fn panel_intersect(&mut self, rect: Rect) {
        if let Some(data) = self
            .panel_map
            .iter()
            .filter(|i| i.1.intersects(rect))
            .collect::<Vec<_>>()
            .get(0)
        {
            self.selected_panel = *data.0;
        }
    }

    /// Check if a given Rect (a clicked area of 1x1), interacts with any known delete button
    pub fn button_intersect(&mut self, rect: Rect) -> Option<DeleteButton> {
        self.delete_map
            .iter()
            .filter(|i| i.1.intersects(rect))
            .collect::<Vec<_>>()
            .get(0)
            .map(|data| *data.0)
    }

    /// Check if a given Rect (a clicked area of 1x1), interacts with any known panels
    pub fn header_intersect(&mut self, rect: Rect) -> Option<Header> {
        self.heading_map
            .iter()
            .filter(|i| i.1.intersects(rect))
            .collect::<Vec<_>>()
            .get(0)
            .map(|data| *data.0)
    }

    /// Insert, or updates header area panel into heading_map
    pub fn update_region_map(&mut self, region: Region, area: Rect) {
        match region {
            Region::Header(header) => self
                .heading_map
                .entry(header)
                .and_modify(|w| *w = area)
                .or_insert(area),
            Region::Panel(panel) => self
                .panel_map
                .entry(panel)
                .and_modify(|w| *w = area)
                .or_insert(area),
            Region::Delete(button) => self
                .delete_map
                .entry(button)
                .and_modify(|w| *w = area)
                .or_insert(area),
        };
    }

    /// Check if an ContainerId is set in the delete_container field
    pub fn get_delete_container(&self) -> Option<ContainerId> {
        self.delete_container.clone()
    }

    /// Set either a ContainerId, or None, to the delete_container field
    /// If Some, will also insert the DeleteConfirm status into self.status
    pub fn set_delete_container(&mut self, id: Option<ContainerId>) {
        if id.is_some() {
            self.status.insert(Status::DeleteConfirm);
        } else {
            self.delete_map.clear();
            self.status.remove(&Status::DeleteConfirm);
        }
        self.delete_container = id;
    }

    /// Check if the current gui_status contains any of the given status'
    /// Don't really like this methodology for gui state, needs a re-think
    pub fn status_contains(&self, status: &[Status]) -> bool {
        status.iter().any(|i| self.status.contains(i))
    }

    /// Remove a gui_status into the current gui_status HashSet
    pub fn status_del(&mut self, status: Status) {
        self.status.remove(&status);
        if status == Status::DeleteConfirm {
            self.status.remove(&Status::DeleteConfirm);
        }
    }

    /// Insert a gui_status into the current gui_status HashSet
    pub fn status_push(&mut self, status: Status) {
        self.status.insert(status);
    }

    /// Change to next selectable panel
    pub fn next_panel(&mut self) {
        self.selected_panel = self.selected_panel.next();
    }

    /// Change to previous selectable panel
    pub fn previous_panel(&mut self) {
        self.selected_panel = self.selected_panel.prev();
    }

    /// Insert a new loading_uuid into HashSet, and advance the loading_index by one frame, or reset to 0 if at end of array
    pub fn next_loading(&mut self, uuid: Uuid) {
        if self.loading_index == FRAMES_LEN {
            self.loading_index = 0;
        } else {
            self.loading_index += 1;
        }
        self.is_loading.insert(uuid);
    }

    /// If is_loading has any entries, return the char at FRAMES[index], else an empty char, which needs to take up the same space, hence ' '
    pub fn get_loading(&self) -> char {
        if self.is_loading.is_empty() {
            ' '
        } else {
            FRAMES[usize::from(self.loading_index)]
        }
    }

    /// Remove a loading_uuid from the is_loading HashSet, if empty, reset loading_index to 0
    fn remove_loading(&mut self, uuid: Uuid) {
        self.is_loading.remove(&uuid);
        if self.is_loading.is_empty() {
            self.loading_index = 0;
        }
    }

    /// Animate the loading icon in its own Tokio thread
    pub fn start_loading_animation(
        gui_state: &Arc<Mutex<Self>>,
        loading_uuid: Uuid,
    ) -> JoinHandle<()> {
        gui_state.lock().next_loading(loading_uuid);
        let gui_state = Arc::clone(gui_state);
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                gui_state.lock().next_loading(loading_uuid);
            }
        })
    }

    /// Stop the loading_spin function, and reset gui loading status
    pub fn stop_loading_animation(&mut self, handle: &JoinHandle<()>, loading_uuid: Uuid) {
        handle.abort();
        self.remove_loading(loading_uuid);
    }

    /// Set info box content
    pub fn set_info_box(&mut self, text: &str) {
        self.info_box_text = Some(text.to_owned());
    }

    /// Remove info box content
    pub fn reset_info_box(&mut self) {
        self.info_box_text = None;
    }
}
