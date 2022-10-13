use std::{
    collections::{HashMap, HashSet},
    fmt,
};
use tui::layout::{Constraint, Rect};
use uuid::Uuid;

use crate::app_data::Header;

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub enum SelectablePanel {
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

    // Should combine with get_vertical_constraints and just return a tuple of (vc, hc)?
    pub const fn get_horizontal_constraints(
        self,
        blank_vertical: u16,
        text_width: u16,
    ) -> [Constraint; 3] {
        match self {
            Self::TopLeft | Self::MiddleLeft | Self::BottomLeft => [
                Constraint::Max(text_width),
                Constraint::Max(blank_vertical),
                Constraint::Max(blank_vertical),
            ],
            Self::TopCentre | Self::MiddleCentre | Self::BottomCentre => [
                Constraint::Max(blank_vertical),
                Constraint::Max(text_width),
                Constraint::Max(blank_vertical),
            ],
            Self::TopRight | Self::MiddleRight | Self::BottomRight => [
                Constraint::Max(blank_vertical),
                Constraint::Max(blank_vertical),
                Constraint::Max(text_width),
            ],
        }
    }
    pub const fn get_vertical_constraints(
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

/// State for the loading animation
#[derive(Debug, Clone, Copy)]
pub enum Loading {
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
}

impl Loading {
    pub const fn next(self) -> Self {
        match self {
            Self::One => Self::Two,
            Self::Two => Self::Three,
            Self::Three => Self::Four,
            Self::Four => Self::Five,
            Self::Five => Self::Six,
            Self::Six => Self::Seven,
            Self::Seven => Self::Eight,
            Self::Eight => Self::Nine,
            Self::Nine => Self::Ten,
            Self::Ten => Self::One,
        }
    }
}

impl fmt::Display for Loading {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let disp = match self {
            Self::One => '⠋',
            Self::Two => '⠙',
            Self::Three => '⠹',
            Self::Four => '⠸',
            Self::Five => '⠼',
            Self::Six => '⠴',
            Self::Seven => '⠦',
            Self::Eight => '⠧',
            Self::Nine => '⠇',
            Self::Ten => '⠏',
        };
        write!(f, "{}", disp)
    }
}

/// Global gui_state, stored in an Arc<Mutex>
#[derive(Debug, Clone)]
pub struct GuiState {
    panel_map: HashMap<SelectablePanel, Rect>,
    heading_map: HashMap<Header, Rect>,
    loading_icon: Loading,
    is_loading: HashSet<Uuid>,
    pub selected_panel: SelectablePanel,
    pub show_help: bool,
    pub info_box_text: Option<String>,
}
impl GuiState {
    /// Generate a default gui_state
    pub fn default() -> Self {
        Self {
            panel_map: HashMap::new(),
            heading_map: HashMap::new(),
            loading_icon: Loading::One,
            selected_panel: SelectablePanel::Containers,
            show_help: false,
            is_loading: HashSet::new(),
            info_box_text: None,
        }
    }

    /// Clear panels hash map, so on resize can fix the sizes for mouse clicks
    pub fn clear_area_map(&mut self) {
        self.panel_map.clear();
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
    pub fn update_map(&mut self, region: Region, area: Rect) {
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
        };
    }

    /// Change to next selectable panel
    pub fn next_panel(&mut self) {
        self.selected_panel = self.selected_panel.next();
    }

    /// Change to previous selectable panel
    pub fn previous_panel(&mut self) {
        self.selected_panel = self.selected_panel.prev();
    }

    /// Insert a new loading_uuid into hashset, and advance the animation by one frame
    pub fn next_loading(&mut self, uuid: Uuid) {
        self.loading_icon = self.loading_icon.next();
        self.is_loading.insert(uuid);
    }

    /// If is_loading has any entries, return the current loading_icon, else an emtpy string
    // Option<String>?
    pub fn get_loading(&mut self) -> String {
        if self.is_loading.is_empty() {
            String::from(" ")
        } else {
            self.loading_icon.to_string()
        }
    }

    /// Remove a loading_uuid from the is_loading hashset
    pub fn remove_loading(&mut self, uuid: Uuid) {
        self.is_loading.remove(&uuid);
    }

    /// Set info box content
    pub fn set_info_box(&mut self, text: String) {
        self.info_box_text = Some(text);
    }

    /// Remove info box content
    pub fn reset_info_box(&mut self) {
        self.info_box_text = None;
    }
}
