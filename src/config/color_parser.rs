use ratatui::style::Color;

/// The macro accepts a list of struct names with key names
/// Returns a struct where every key name is an Option<String>, with the correct derived attributes
macro_rules! optional_config_struct {
    ($($struct_name:ident, $($key_name:ident),*);*) => {
        $(
            #[derive(Debug, serde::Deserialize, Clone, PartialEq, Eq)]
            struct $struct_name {
                $(
                    $key_name: Option<String>,
                )*
            }
        )*
    };
}

/// The macro accepts a list of struct names with key names
macro_rules! config_struct {
    ($($struct_name:ident, $($key_name:ident),*);*) => {
        $(
            #[derive(Debug, Clone, PartialEq, Eq, Copy)]
            pub struct $struct_name {
                $(
                    pub $key_name: Color,
                )*
            }
        )*
    };
}

impl AppColors {
    fn map_color(color_str: Option<&str>, setter: &mut Color) {
        color_str.map(|i| i.parse::<Color>().map(|i| *setter = i));
    }
}

impl From<Option<ConfigColors>> for AppColors {
    #[allow(clippy::too_many_lines)]
    fn from(value: Option<ConfigColors>) -> Self {
        let mut app_colors = Self::new();

        if let Some(config_colors) = value {
            // Heading bar
            if let Some(hb) = config_colors.headers_bar {
                Self::map_color(
                    hb.background.as_deref(),
                    &mut app_colors.headers_bar.background,
                );
                Self::map_color(
                    hb.loading_spinner.as_deref(),
                    &mut app_colors.headers_bar.loading_spinner,
                );
                Self::map_color(hb.text.as_deref(), &mut app_colors.headers_bar.text);
                Self::map_color(
                    hb.text_selected.as_deref(),
                    &mut app_colors.headers_bar.text_selected,
                );
            }

            // Seletable panel borders
            if let Some(b) = config_colors.borders {
                Self::map_color(b.selected.as_deref(), &mut app_colors.borders.selected);
                Self::map_color(b.unselected.as_deref(), &mut app_colors.borders.unselected);
            }

            // Error Popup
            if let Some(ep) = config_colors.popup_error {
                Self::map_color(
                    ep.background.as_deref(),
                    &mut app_colors.popup_error.background,
                );
                Self::map_color(ep.text.as_deref(), &mut app_colors.popup_error.text);
            }

            // Help Popup
            if let Some(hp) = config_colors.popup_help {
                Self::map_color(
                    hp.background.as_deref(),
                    &mut app_colors.popup_help.background,
                );
                Self::map_color(hp.text.as_deref(), &mut app_colors.popup_help.text);
                Self::map_color(
                    hp.text_highlight.as_deref(),
                    &mut app_colors.popup_help.text_highlight,
                );
            }

            // Info Popup
            if let Some(ip) = config_colors.popup_info {
                Self::map_color(
                    ip.background.as_deref(),
                    &mut app_colors.popup_info.background,
                );
                Self::map_color(ip.text.as_deref(), &mut app_colors.popup_info.text);
            }

            // Delete Popup
            if let Some(dp) = config_colors.popup_delete {
                Self::map_color(
                    dp.background.as_deref(),
                    &mut app_colors.popup_delete.background,
                );
                Self::map_color(dp.text.as_deref(), &mut app_colors.popup_delete.text);
                Self::map_color(
                    dp.text_highlight.as_deref(),
                    &mut app_colors.popup_delete.text_highlight,
                );
            }

            // Chart Cpu
            if let Some(cc) = config_colors.chart_cpu {
                Self::map_color(
                    cc.background.as_deref(),
                    &mut app_colors.chart_cpu.background,
                );
                Self::map_color(cc.border.as_deref(), &mut app_colors.chart_cpu.border);
                Self::map_color(cc.max.as_deref(), &mut app_colors.chart_cpu.max);
                Self::map_color(cc.points.as_deref(), &mut app_colors.chart_cpu.points);
                Self::map_color(cc.title.as_deref(), &mut app_colors.chart_cpu.title);
                Self::map_color(cc.y_axis.as_deref(), &mut app_colors.chart_cpu.y_axis);
            }

            // Chart Memory
            if let Some(cm) = config_colors.chart_memory {
                Self::map_color(
                    cm.background.as_deref(),
                    &mut app_colors.chart_memory.background,
                );
                Self::map_color(cm.border.as_deref(), &mut app_colors.chart_memory.border);
                Self::map_color(cm.max.as_deref(), &mut app_colors.chart_memory.max);
                Self::map_color(cm.points.as_deref(), &mut app_colors.chart_memory.points);
                Self::map_color(cm.title.as_deref(), &mut app_colors.chart_memory.title);
                Self::map_color(cm.y_axis.as_deref(), &mut app_colors.chart_memory.y_axis);
            }

            // Chart ports
            if let Some(cp) = config_colors.chart_ports {
                Self::map_color(
                    cp.background.as_deref(),
                    &mut app_colors.chart_ports.background,
                );
                Self::map_color(cp.border.as_deref(), &mut app_colors.chart_ports.border);
                Self::map_color(cp.headings.as_deref(), &mut app_colors.chart_ports.headings);
                Self::map_color(cp.text.as_deref(), &mut app_colors.chart_ports.text);
                Self::map_color(cp.title.as_deref(), &mut app_colors.chart_ports.title);
            }

            // Containers
            if let Some(c) = config_colors.containers {
                Self::map_color(
                    c.background.as_deref(),
                    &mut app_colors.containers.background,
                );
                Self::map_color(c.icon.as_deref(), &mut app_colors.containers.icon);
                Self::map_color(c.text.as_deref(), &mut app_colors.containers.text);
                Self::map_color(c.text_rx.as_deref(), &mut app_colors.containers.text_rx);
                Self::map_color(c.text_tx.as_deref(), &mut app_colors.containers.text_tx);
            }

            // Commands
            if let Some(cc) = config_colors.commands {
                Self::map_color(
                    cc.background.as_deref(),
                    &mut app_colors.commands.background,
                );
                Self::map_color(cc.pause.as_deref(), &mut app_colors.commands.pause);
                Self::map_color(cc.restart.as_deref(), &mut app_colors.commands.restart);
                Self::map_color(cc.stop.as_deref(), &mut app_colors.commands.stop);
                Self::map_color(cc.delete.as_deref(), &mut app_colors.commands.start);
                Self::map_color(cc.resume.as_deref(), &mut app_colors.commands.resume);
                Self::map_color(cc.start.as_deref(), &mut app_colors.commands.start);
            }

            // Container State
            if let Some(cs) = config_colors.container_state {
                Self::map_color(cs.dead.as_deref(), &mut app_colors.container_state.dead);
                Self::map_color(cs.exited.as_deref(), &mut app_colors.container_state.exited);
                Self::map_color(cs.paused.as_deref(), &mut app_colors.container_state.paused);
                Self::map_color(
                    cs.removing.as_deref(),
                    &mut app_colors.container_state.removing,
                );
                Self::map_color(
                    cs.restarting.as_deref(),
                    &mut app_colors.container_state.restarting,
                );
                Self::map_color(
                    cs.running_healthy.as_deref(),
                    &mut app_colors.container_state.running_healthy,
                );
                Self::map_color(
                    cs.running_unhealthy.as_deref(),
                    &mut app_colors.container_state.running_unhealthy,
                );
                Self::map_color(
                    cs.unknown.as_deref(),
                    &mut app_colors.container_state.unknown,
                );
            }
        }
        app_colors
    }
}

const ORANGE: Color = Color::Rgb(255, 178, 36);

optional_config_struct!(
    ConfigBackgroundText, background, text;
    ConfigBackgroundTextHighlight, background, text, text_highlight;
    ConfigBorders, selected, unselected;
    ConfigChartCpu, background, border, order, title, max, points,y_axis;
    ConfigChartMemory, background, border, title, max, points, y_axis;
    ConfigChartPorts, background, border, title, headings, text;
    ConfigCommands, background, pause, restart, stop, delete, resume, start;
    ConfigContainers, background, icon, text, text_rx, text_tx;
    ConfigContainerState, background, dead, exited, paused, removing, restarting, running_healthy, running_unhealthy, unknown;
    ConfigHeadersBar, background, loading_spinner, text, text_selected
);

config_struct!(
    Borders, selected, unselected;
    ChartCpu, background, border, title, max, points, y_axis;
    ChartMemory, background, border, title, max, points, y_axis;
    ChartPorts, background, border, title, headings, text;
    Commands, background, pause, restart, stop, delete, resume, start;
    Containers, background, icon, text, text_rx, text_tx;
    ContainerState, dead, exited, paused, removing, restarting, running_healthy, running_unhealthy, unknown;
    HeadersBar, background, text_selected, loading_spinner, text;
    PopupDelete, background, text, text_highlight;
    PopupError, background, text;
    PopupHelp, background, text, text_highlight;
    PopupInfo, background, text
);

#[derive(Debug, serde::Deserialize, Clone, PartialEq, Eq)]
pub struct ConfigColors {
    borders: Option<ConfigBorders>,
    chart_cpu: Option<ConfigChartCpu>,
    chart_memory: Option<ConfigChartMemory>,
    chart_ports: Option<ConfigChartPorts>,
    commands: Option<ConfigCommands>,
    container_state: Option<ConfigContainerState>,
    containers: Option<ConfigContainers>,
    headers_bar: Option<ConfigHeadersBar>,
    popup_delete: Option<ConfigBackgroundTextHighlight>,
    popup_error: Option<ConfigBackgroundText>,
    popup_help: Option<ConfigBackgroundTextHighlight>,
    popup_info: Option<ConfigBackgroundText>,
}

/// Default colours for the header bar
impl HeadersBar {
    const fn new() -> Self {
        Self {
            background: Color::Magenta,
            loading_spinner: Color::White,
            text: Color::Black,
            text_selected: Color::Gray,
        }
    }
}

/// Default colours for the borders
impl Borders {
    const fn new() -> Self {
        Self {
            selected: Color::LightCyan,
            unselected: Color::Gray,
        }
    }
}

/// Default colours for the delete popup
impl Commands {
    const fn new() -> Self {
        Self {
            background: Color::Reset,
            pause: Color::Yellow,
            restart: Color::Magenta,
            stop: Color::Red,
            delete: Color::Gray,
            resume: Color::Blue,
            start: Color::Green,
        }
    }
}

/// Default colours for the help popup
impl ChartCpu {
    const fn new() -> Self {
        Self {
            background: Color::Reset,
            border: Color::White,
            title: Color::Green,
            max: ORANGE,
            points: Color::Magenta,
            y_axis: Color::White,
        }
    }
}

/// Default colours for the help popup
impl ChartMemory {
    const fn new() -> Self {
        Self {
            background: Color::Reset,
            border: Color::White,
            title: Color::Green,
            max: ORANGE,
            points: Color::Cyan,
            y_axis: Color::White,
        }
    }
}

/// Default colours for the help popup
impl ChartPorts {
    const fn new() -> Self {
        Self {
            background: Color::Reset,
            border: Color::White,
            title: Color::Green,
            headings: Color::Yellow,
            text: Color::White,
        }
    }
}

/// Default colours for the help popup
impl Containers {
    const fn new() -> Self {
        Self {
            background: Color::Reset,
            icon: Color::White,
            text: Color::Blue,
            text_rx: Color::Rgb(255, 233, 193),
            text_tx: Color::Rgb(205, 140, 140),
        }
    }
}

/// Default colours for the help popup
impl ContainerState {
    const fn new() -> Self {
        Self {
            paused: Color::Yellow,
            removing: Color::LightRed,
            restarting: Color::LightGreen,
            running_healthy: Color::Green,
            running_unhealthy: ORANGE,
            dead: Color::Red,
            exited: Color::Red,
            unknown: Color::Red,
        }
    }
}
/// Default colours for the Error popup
impl PopupError {
    const fn new() -> Self {
        Self {
            background: Color::Red,
            text: Color::White,
        }
    }
}

/// Default colours for the info popup
impl PopupInfo {
    const fn new() -> Self {
        Self {
            background: Color::Blue,
            text: Color::White,
        }
    }
}

/// Default colours for the help popup
impl PopupHelp {
    const fn new() -> Self {
        Self {
            background: Color::Magenta,
            text: Color::Black,
            text_highlight: Color::White,
        }
    }
}

/// Default colours for the delete popup
impl PopupDelete {
    const fn new() -> Self {
        Self {
            background: Color::White,
            text: Color::Black,
            text_highlight: Color::Red,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub struct AppColors {
    pub borders: Borders,
    pub chart_cpu: ChartCpu,
    pub chart_memory: ChartMemory,
    pub chart_ports: ChartPorts,
    pub commands: Commands,
    pub container_state: ContainerState,
    pub containers: Containers,
    pub headers_bar: HeadersBar,
    pub popup_delete: PopupDelete,
    pub popup_error: PopupError,
    pub popup_help: PopupHelp,
    pub popup_info: PopupInfo,
}

impl AppColors {
    pub const fn new() -> Self {
        Self {
            borders: Borders::new(),
            chart_cpu: ChartCpu::new(),
            chart_memory: ChartMemory::new(),
            chart_ports: ChartPorts::new(),
            commands: Commands::new(),
            container_state: ContainerState::new(),
            containers: Containers::new(),
            headers_bar: HeadersBar::new(),
            popup_delete: PopupDelete::new(),
            popup_error: PopupError::new(),
            popup_help: PopupHelp::new(),
            popup_info: PopupInfo::new(),
        }
    }
}
