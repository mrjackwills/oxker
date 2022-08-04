use std::{collections::HashMap, fmt};
use tui::layout::{Constraint, Rect};

use crate::app_data::Header;

#[derive(Debug, PartialEq, std::hash::Hash, std::cmp::Eq, Clone, Copy)]
pub enum SelectablePanel {
    Containers,
    Commands,
    Logs,
}

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
    pub fn get_indexes(self) -> (usize, usize) {
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

    // Should combine and just return a tuple?
    pub fn get_horizontal_constraints(
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
    pub fn get_vertical_constraints(
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

#[derive(Debug, Clone)]
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
    pub fn next(&self) -> Self {
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
            Self::One => "⠋",
            Self::Two => "⠙",
            Self::Three => "⠹",
            Self::Four => "⠸",
            Self::Five => "⠼",
            Self::Six => "⠴",
            Self::Seven => "⠦",
            Self::Eight => "⠧",
            Self::Nine => "⠇",
            Self::Ten => "⠏",
        };
        write!(f, "{}", disp)
    }
}

impl SelectablePanel {
    pub fn title(self) -> &'static str {
        match self {
            Self::Containers => "Containers",
            Self::Logs => "Logs",
            Self::Commands => "",
        }
    }
    pub fn next(self) -> Self {
        match self {
            Self::Containers => Self::Commands,
            Self::Commands => Self::Logs,
            Self::Logs => Self::Containers,
        }
    }
    pub fn prev(self) -> Self {
        match self {
            Self::Containers => Self::Logs,
            Self::Commands => Self::Containers,
            Self::Logs => Self::Commands,
        }
    }
}

/// Global gui_state, stored in an Arc<Mutex>
#[derive(Debug, Clone)]
pub struct GuiState {
    // Think this should be a BMapTree, so can define order when iterating over potential intersects
    // Is an issue if two panels are in the same space, sush as a smaller panel embedded, yet infront of, a larger panel
    // If a BMapTree think it would mean have to implement ordering for SelectablePanel
    panel_map: HashMap<SelectablePanel, Rect>,
    heading_map: HashMap<Header, Rect>,
    loading_icon: Loading,
    // Should be a vec, each time loading add a new to the vec, and reset remove from vec
    // for for if is_loading just check if vec is empty or not
    is_loading: bool,
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
            is_loading: false,
            info_box_text: None,
        }
    }

    /// clear panels hash map, so on resize can fix the sizes for mouse clicks
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
            .map(|data| data.0.clone())
    }

    /// Insert, or updatem header area panel into heading_map
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

    /// Advance loading animation
    pub fn next_loading(&mut self) {
        self.loading_icon = self.loading_icon.next();
        self.is_loading = true;
    }

    /// if is_loading, return loading animation frame, else single space
    pub fn get_loading(&mut self) -> String {
        if self.is_loading {
            self.loading_icon.to_string()
        } else {
            String::from(" ")
        }
    }

    /// set is_loading to false, but keep animation frame at same state
    pub fn reset_loading(&mut self) {
        self.is_loading = false;
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
