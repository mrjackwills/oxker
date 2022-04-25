use std::{collections::HashMap, fmt};
use tui::layout::Rect;

#[derive(Debug, PartialEq, std::hash::Hash, std::cmp::Eq, Clone, Copy)]
pub enum SelectablePanel {
    Containers,
    Commands,
    Logs,
}
#[derive(Debug)]
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
            // Self::Five => Self::One
        }
    }
}
// "⠋",
// 			"⠙",
// 			"⠹",
// 			"⠸",
// 			"⠼",
// 			"⠴",
// 			"⠦",
// 			"⠧",
// 			"⠇",
// 			"⠏"

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
            _ => "",
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
#[derive(Debug)]
pub struct GuiState {
    // Think this should be a BMapTree, so can define order when iterating over potential intersects
    // Is an issue if two panels are in the same space, sush as a smaller panel embedded, yet infront of, a larger panel
    // If a BMapTree think it would mean have to implement ordering for SelectablePanel
    area_map: HashMap<SelectablePanel, Rect>,
    loading: Loading,
    pub selected_panel: SelectablePanel,
    pub show_help: bool,
}

impl GuiState {
    /// Generate a default gui_state
    pub fn default() -> Self {
        Self {
            area_map: HashMap::new(),
            loading: Loading::One,
            selected_panel: SelectablePanel::Containers,
            show_help: false,
        }
    }

    /// clear panels hash map, so on resize can fix the sizes for mouse clicks
    pub fn clear_area_map(&mut self) {
        self.area_map.clear();
    }

    /// Check if a given Rect (a clicked area of 1x1), interacts with any known panels
    pub fn rect_insersects(&mut self, rect: Rect) {
        if let Some(data) = self
            .area_map
            .iter()
            .filter(|i| i.1.intersects(rect))
            .collect::<Vec<_>>()
            .get(0)
        {
            self.selected_panel = *data.0;
        }
    }

    /// Insert selectable gui panel into area map
    pub fn insert_into_area_map(&mut self, panel: SelectablePanel, area: Rect) {
        self.area_map.entry(panel).or_insert(area);
    }

    /// Change to next selectable panel
    pub fn next_panel(&mut self) {
        self.selected_panel = self.selected_panel.next();
    }

    /// Change to previous selectable panel
    pub fn previous_panel(&mut self) {
        self.selected_panel = self.selected_panel.prev();
    }

    pub fn next_loading(&mut self) {
        self.loading = self.loading.next()
    }

    pub fn get_loading(&mut self) -> String {
        self.loading.to_string()
    }

    pub fn reset_loading(&mut self) {
        self.loading = Loading::One;
    }
}
