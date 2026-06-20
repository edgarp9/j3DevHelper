use super::core::current_ui_language;
use super::core::{DEFAULT_FONT_SIZE, normalize_ui_font_size};
use super::localization::UiLanguage;
use super::state::AppState;

pub const APP_TITLE: &str = "j3DevHelper";
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const APP_REPOSITORY_URL: &str = "https://github.com/edgarp9";
pub const APP_LINUX_APPLICATION_ID: &str = "io.github.edgarp9.j3DevHelper";
pub const APP_LINUX_DESKTOP_ENTRY_NAME: &str = "io.github.edgarp9.j3DevHelper";
pub const APP_ICON_SVG_FILE_NAME: &str = "icon.svg";
pub const APP_ICON_PNG_FILE_NAME: &str = "icon.png";
pub const DEFAULT_DPI: u32 = 96;

const BASE_CONTENT_MARGIN: i32 = 8;
const BASE_CONTENT_TOP_GAP: i32 = 6;
const BASE_PANEL_GAP: i32 = 10;
const BASE_TREE_PANEL_WIDTH: i32 = 160;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WindowSize {
    pub width: i32,
    pub height: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ClientSize {
    pub width: i32,
    pub height: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RectSpec {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl RectSpec {
    fn from_edges(left: i32, top: i32, right: i32, bottom: i32) -> Self {
        Self {
            x: left,
            y: top,
            width: (right - left).max(0),
            height: (bottom - top).max(0),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LayoutSpec {
    pub content_margin: i32,
    pub content_top_gap: i32,
    pub panel_gap: i32,
    pub tree_panel_width: i32,
}

impl LayoutSpec {
    pub fn for_font_size(font_size: u16) -> Self {
        let font_size = normalize_ui_font_size(font_size);
        let extra = i32::from(font_size.saturating_sub(DEFAULT_FONT_SIZE));

        Self {
            content_margin: BASE_CONTENT_MARGIN,
            content_top_gap: BASE_CONTENT_TOP_GAP + extra.min(4),
            panel_gap: BASE_PANEL_GAP + extra.min(6),
            tree_panel_width: BASE_TREE_PANEL_WIDTH + extra * 8,
        }
    }

    pub fn for_font_size_and_dpi(font_size: u16, dpi: u32) -> Self {
        self::scale_layout_for_dpi(Self::for_font_size(font_size), dpi)
    }

    pub fn arrange_main_content(self, client: ClientSize) -> MainContentLayout {
        let left = self.content_margin;
        let top = self.content_top_gap;
        let panel_top = top;
        let bottom = (client.height - self.content_margin).max(panel_top);
        let tree_right = left + self.tree_panel_width;
        let tabs_left = tree_right + self.panel_gap;
        let right = (client.width - self.content_margin).max(tabs_left);

        MainContentLayout {
            tree_panel: RectSpec::from_edges(left, panel_top, tree_right, bottom),
            command_tabs_panel: RectSpec::from_edges(tabs_left, panel_top, right, bottom),
        }
    }
}

fn scale_layout_for_dpi(layout: LayoutSpec, dpi: u32) -> LayoutSpec {
    LayoutSpec {
        content_margin: scale_dimension_for_dpi(layout.content_margin, dpi),
        content_top_gap: scale_dimension_for_dpi(layout.content_top_gap, dpi),
        panel_gap: scale_dimension_for_dpi(layout.panel_gap, dpi),
        tree_panel_width: scale_dimension_for_dpi(layout.tree_panel_width, dpi),
    }
}

pub fn scale_dimension_for_dpi(value: i32, dpi: u32) -> i32 {
    let dpi = dpi.max(1);
    let scaled = i64::from(value) * i64::from(dpi);
    let scaled = (scaled + i64::from(DEFAULT_DPI / 2)) / i64::from(DEFAULT_DPI);
    scaled.clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MainContentLayout {
    pub tree_panel: RectSpec,
    pub command_tabs_panel: RectSpec,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MenuDefinition {
    pub label: &'static str,
    pub items: &'static [MenuItemDefinition],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MenuItemDefinition {
    pub label: &'static str,
    pub enabled: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MainWindowSpec {
    pub title: &'static str,
    pub initial_size: WindowSize,
    pub layout: LayoutSpec,
    pub menus: &'static [MenuDefinition],
    pub state: AppState,
}

impl MainWindowSpec {
    pub fn initial(state: AppState) -> Self {
        let layout = LayoutSpec::for_font_size(state.settings().view.font_size);

        Self {
            title: APP_TITLE,
            initial_size: WindowSize {
                width: 484,
                height: 613,
            },
            layout,
            menus: main_menu_for_language(current_ui_language(&state.settings().view)),
            state,
        }
    }
}

pub fn main_menu_for_language(language: UiLanguage) -> &'static [MenuDefinition] {
    match language {
        UiLanguage::Korean => &MAIN_MENU_KO,
        UiLanguage::English => &MAIN_MENU_EN,
    }
}

const FILE_MENU_ITEMS_KO: [MenuItemDefinition; 6] = [
    MenuItemDefinition {
        label: "글꼴",
        enabled: true,
    },
    MenuItemDefinition {
        label: "테마",
        enabled: true,
    },
    MenuItemDefinition {
        label: "UI 언어",
        enabled: true,
    },
    MenuItemDefinition {
        label: "워크스페이스 언어",
        enabled: true,
    },
    MenuItemDefinition {
        label: "정보",
        enabled: true,
    },
    MenuItemDefinition {
        label: "종료",
        enabled: true,
    },
];

const FILE_MENU_ITEMS_EN: [MenuItemDefinition; 6] = [
    MenuItemDefinition {
        label: "Font",
        enabled: true,
    },
    MenuItemDefinition {
        label: "Theme",
        enabled: true,
    },
    MenuItemDefinition {
        label: "UI Language",
        enabled: true,
    },
    MenuItemDefinition {
        label: "Workspace Languages",
        enabled: true,
    },
    MenuItemDefinition {
        label: "About",
        enabled: true,
    },
    MenuItemDefinition {
        label: "Exit",
        enabled: true,
    },
];

const TREE_MENU_ITEMS_KO: [MenuItemDefinition; 6] = [
    MenuItemDefinition {
        label: "추가",
        enabled: true,
    },
    MenuItemDefinition {
        label: "분류 추가",
        enabled: true,
    },
    MenuItemDefinition {
        label: "편집",
        enabled: false,
    },
    MenuItemDefinition {
        label: "위로",
        enabled: false,
    },
    MenuItemDefinition {
        label: "아래로",
        enabled: false,
    },
    MenuItemDefinition {
        label: "삭제",
        enabled: false,
    },
];

const TREE_MENU_ITEMS_EN: [MenuItemDefinition; 6] = [
    MenuItemDefinition {
        label: "Add",
        enabled: true,
    },
    MenuItemDefinition {
        label: "Add Category",
        enabled: true,
    },
    MenuItemDefinition {
        label: "Edit",
        enabled: false,
    },
    MenuItemDefinition {
        label: "Move Up",
        enabled: false,
    },
    MenuItemDefinition {
        label: "Move Down",
        enabled: false,
    },
    MenuItemDefinition {
        label: "Delete",
        enabled: false,
    },
];

const TABS_MENU_ITEMS_KO: [MenuItemDefinition; 5] = [
    MenuItemDefinition {
        label: "추가",
        enabled: true,
    },
    MenuItemDefinition {
        label: "이름 변경",
        enabled: false,
    },
    MenuItemDefinition {
        label: "위로",
        enabled: false,
    },
    MenuItemDefinition {
        label: "아래로",
        enabled: false,
    },
    MenuItemDefinition {
        label: "삭제",
        enabled: false,
    },
];

const TABS_MENU_ITEMS_EN: [MenuItemDefinition; 5] = [
    MenuItemDefinition {
        label: "Add",
        enabled: true,
    },
    MenuItemDefinition {
        label: "Rename",
        enabled: false,
    },
    MenuItemDefinition {
        label: "Move Up",
        enabled: false,
    },
    MenuItemDefinition {
        label: "Move Down",
        enabled: false,
    },
    MenuItemDefinition {
        label: "Delete",
        enabled: false,
    },
];

const COMMANDS_MENU_ITEMS_KO: [MenuItemDefinition; 6] = [
    MenuItemDefinition {
        label: "실행",
        enabled: false,
    },
    MenuItemDefinition {
        label: "추가",
        enabled: false,
    },
    MenuItemDefinition {
        label: "편집",
        enabled: false,
    },
    MenuItemDefinition {
        label: "앞으로",
        enabled: false,
    },
    MenuItemDefinition {
        label: "뒤로",
        enabled: false,
    },
    MenuItemDefinition {
        label: "삭제",
        enabled: false,
    },
];

const COMMANDS_MENU_ITEMS_EN: [MenuItemDefinition; 6] = [
    MenuItemDefinition {
        label: "Run",
        enabled: false,
    },
    MenuItemDefinition {
        label: "Add",
        enabled: false,
    },
    MenuItemDefinition {
        label: "Edit",
        enabled: false,
    },
    MenuItemDefinition {
        label: "Previous",
        enabled: false,
    },
    MenuItemDefinition {
        label: "Next",
        enabled: false,
    },
    MenuItemDefinition {
        label: "Delete",
        enabled: false,
    },
];

const MAIN_MENU_KO: [MenuDefinition; 4] = [
    MenuDefinition {
        label: "파일",
        items: &FILE_MENU_ITEMS_KO,
    },
    MenuDefinition {
        label: "워크스페이스",
        items: &TREE_MENU_ITEMS_KO,
    },
    MenuDefinition {
        label: "명령 그룹",
        items: &TABS_MENU_ITEMS_KO,
    },
    MenuDefinition {
        label: "명령",
        items: &COMMANDS_MENU_ITEMS_KO,
    },
];

const MAIN_MENU_EN: [MenuDefinition; 4] = [
    MenuDefinition {
        label: "File",
        items: &FILE_MENU_ITEMS_EN,
    },
    MenuDefinition {
        label: "Workspace",
        items: &TREE_MENU_ITEMS_EN,
    },
    MenuDefinition {
        label: "Command Group",
        items: &TABS_MENU_ITEMS_EN,
    },
    MenuDefinition {
        label: "Command",
        items: &COMMANDS_MENU_ITEMS_EN,
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scale_dimension_for_dpi_preserves_normal_dimensions() {
        assert_eq!(scale_dimension_for_dpi(160, DEFAULT_DPI), 160);
        assert_eq!(scale_dimension_for_dpi(160, DEFAULT_DPI * 3 / 2), 240);
        assert_eq!(scale_dimension_for_dpi(10, 120), 13);
    }

    #[test]
    fn scale_dimension_for_dpi_saturates_large_positive_dimensions() {
        let scaled = scale_dimension_for_dpi(i32::MAX, DEFAULT_DPI * 2);

        assert_eq!(scaled, i32::MAX);
        assert!(scaled > 0);
    }
}
