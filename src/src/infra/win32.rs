use std::ffi::c_void;
use std::iter::once;
use std::mem::size_of;
use std::os::windows::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::ptr::{null, null_mut};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};

use windows_sys::Win32::Foundation::{
    E_NOINTERFACE, E_POINTER, ERROR_CLASS_ALREADY_EXISTS, GetLastError, HANDLE, HGLOBAL, HINSTANCE,
    HWND, LPARAM, LRESULT, MAX_PATH, POINT, POINTL, RECT, S_OK, WPARAM,
};
use windows_sys::Win32::Graphics::Dwm::{DWMWA_USE_IMMERSIVE_DARK_MODE, DwmSetWindowAttribute};
use windows_sys::Win32::Graphics::Gdi::{
    CLIP_DEFAULT_PRECIS, COLOR_WINDOW, ClientToScreen, CreateFontIndirectW, CreateSolidBrush,
    DEFAULT_CHARSET, DEFAULT_GUI_FONT, DEFAULT_PITCH, DEFAULT_QUALITY, DeleteObject,
    EnumFontFamiliesExW, FF_DONTCARE, FillRect, GetDC, GetDeviceCaps, GetObjectW, GetStockObject,
    GetSysColorBrush, HBRUSH, HDC, HFONT, HGDIOBJ, InvalidateRect, LOGFONTW, LOGPIXELSY,
    OUT_DEFAULT_PRECIS, ReleaseDC, ScreenToClient, SetBkColor, SetBkMode, SetTextColor,
    TEXTMETRICW, UpdateWindow,
};
use windows_sys::Win32::System::Com::{
    CoTaskMemFree, DVASPECT_CONTENT, FORMATETC, STGMEDIUM, TYMED_HGLOBAL,
};
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::System::Ole::{
    CF_HDROP, DROPEFFECT_COPY, DROPEFFECT_NONE, OleInitialize, OleUninitialize, RegisterDragDrop,
    ReleaseStgMedium, RevokeDragDrop,
};
use windows_sys::Win32::UI::Controls::Dialogs::{
    GetOpenFileNameW, OFN_EXPLORER, OFN_FILEMUSTEXIST, OFN_PATHMUSTEXIST, OPENFILENAMEW,
};
use windows_sys::Win32::UI::Controls::{
    BST_CHECKED, CDDS_ITEMPREPAINT, CDDS_PREPAINT, CDRF_DODEFAULT, CDRF_NOTIFYITEMDRAW,
    CLR_DEFAULT, CheckRadioButton, EM_REPLACESEL, HTREEITEM, ICC_LINK_CLASS, ICC_TREEVIEW_CLASSES,
    ICC_WIN95_CLASSES, INITCOMMONCONTROLSEX, InitCommonControlsEx, LWS_TRANSPARENT, NM_CLICK,
    NM_CUSTOMDRAW, NM_DBLCLK, NM_RCLICK, NM_RETURN, NMHDR, NMTREEVIEWW, NMTVCUSTOMDRAW,
    NMTVGETINFOTIPW, SetScrollInfo, SetWindowTheme, ShowScrollBar, TOOLTIPS_CLASSW, TTF_IDISHWND,
    TTF_SUBCLASS, TTM_ADDTOOLW, TTM_DELTOOLW, TTM_UPDATETIPTEXTW, TTS_ALWAYSTIP, TTS_NOPREFIX,
    TTTOOLINFOW, TVE_EXPAND, TVGN_CARET, TVHITTESTINFO, TVHT_ONITEM, TVI_LAST, TVI_ROOT,
    TVIF_HANDLE, TVIF_PARAM, TVIF_TEXT, TVINSERTSTRUCTW, TVINSERTSTRUCTW_0, TVITEMW,
    TVM_DELETEITEM, TVM_EXPAND, TVM_GETITEMRECT, TVM_GETITEMW, TVM_GETNEXTITEM, TVM_HITTEST,
    TVM_INSERTITEMW, TVM_SELECTITEM, TVM_SETBKCOLOR, TVM_SETINSERTMARK, TVM_SETITEMW,
    TVM_SETLINECOLOR, TVM_SETTEXTCOLOR, TVN_BEGINDRAGW, TVN_GETINFOTIPW, TVN_SELCHANGEDW,
    TVS_HASBUTTONS, TVS_HASLINES, TVS_INFOTIP, TVS_LINESATROOT, TVS_SHOWSELALWAYS, WC_LINK,
    WC_TREEVIEWW,
};
use windows_sys::Win32::UI::HiDpi::{
    DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2, GetDpiForWindow, SetProcessDpiAwarenessContext,
};
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    EnableWindow, GetKeyState, IsWindowEnabled, ReleaseCapture, SetActiveWindow, SetCapture,
    SetFocus,
};
use windows_sys::Win32::UI::Shell::{
    BIF_NEWDIALOGSTYLE, BIF_RETURNONLYFSDIRS, BROWSEINFOW, DragQueryFileW, HDROP,
    SHBrowseForFolderW, SHGetPathFromIDListW, ShellExecuteW,
};
use windows_sys::Win32::UI::Shell::{DefSubclassProc, RemoveWindowSubclass, SetWindowSubclass};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    AppendMenuW, BM_GETCHECK, BM_SETSTATE, BS_AUTORADIOBUTTON, BS_DEFPUSHBUTTON, BS_LEFT,
    BS_PUSHBUTTON, CB_ADDSTRING, CB_ERR, CB_FINDSTRINGEXACT, CB_GETCURSEL, CB_GETLBTEXT,
    CB_GETLBTEXTLEN, CB_RESETCONTENT, CB_SETCURSEL, CBN_SELCHANGE, CBS_DROPDOWNLIST,
    CBS_HASSTRINGS, CREATESTRUCTW, CS_HREDRAW, CS_VREDRAW, CW_USEDEFAULT, CreateMenu,
    CreatePopupMenu, CreateWindowExW, DefWindowProcW, DestroyIcon, DestroyMenu, DestroyWindow,
    DispatchMessageW, DrawMenuBar, ES_AUTOHSCROLL, ES_AUTOVSCROLL, ES_MULTILINE, ES_READONLY,
    ES_WANTRETURN, EnableMenuItem, GWLP_USERDATA, GetClientRect, GetMenu, GetMessagePos,
    GetMessageW, GetParent, GetPropW, GetScrollInfo, GetWindowLongPtrW, GetWindowPlacement,
    GetWindowRect, GetWindowTextLengthW, GetWindowTextW, HICON, HMENU, ICON_BIG, ICON_SMALL,
    IDC_ARROW, IDC_SIZEWE, IDCANCEL, IDOK, IDYES, IMAGE_ICON, IsDialogMessageW, IsWindow,
    LR_LOADFROMFILE, LoadCursorW, LoadImageW, MB_DEFBUTTON2, MB_ICONERROR, MB_ICONWARNING, MB_OK,
    MB_YESNO, MF_BYCOMMAND, MF_CHECKED, MF_ENABLED, MF_GRAYED, MF_POPUP, MF_SEPARATOR,
    MF_UNCHECKED, MINMAXINFO, MSG, MessageBoxW, MoveWindow, PostMessageW, PostQuitMessage,
    RegisterClassExW, RemovePropW, SB_BOTTOM, SB_LINEDOWN, SB_LINEUP, SB_PAGEDOWN, SB_PAGEUP,
    SB_THUMBPOSITION, SB_THUMBTRACK, SB_TOP, SB_VERT, SCROLLINFO, SIF_PAGE, SIF_POS, SIF_RANGE,
    SIF_TRACKPOS, SW_SHOW, SendMessageW, SetCursor, SetForegroundWindow, SetMenu, SetPropW,
    SetWindowLongPtrW, SetWindowTextW, ShowWindow, TPM_LEFTALIGN, TPM_RETURNCMD, TPM_RIGHTBUTTON,
    TPM_TOPALIGN, TrackPopupMenu, TranslateMessage, WINDOWPLACEMENT, WM_APP, WM_CANCELMODE,
    WM_CAPTURECHANGED, WM_CLOSE, WM_COMMAND, WM_CONTEXTMENU, WM_CTLCOLORBTN, WM_CTLCOLORSTATIC,
    WM_DESTROY, WM_DPICHANGED, WM_ERASEBKGND, WM_GETMINMAXINFO, WM_KEYDOWN, WM_LBUTTONDOWN,
    WM_LBUTTONUP, WM_MOUSEMOVE, WM_MOUSEWHEEL, WM_NCCREATE, WM_NCDESTROY, WM_NOTIFY, WM_SETFONT,
    WM_SETICON, WM_SIZE, WM_VSCROLL, WNDCLASSEXW, WS_BORDER, WS_CAPTION, WS_CHILD, WS_CLIPCHILDREN,
    WS_CLIPSIBLINGS, WS_EX_CLIENTEDGE, WS_EX_CONTROLPARENT, WS_EX_DLGMODALFRAME, WS_GROUP,
    WS_OVERLAPPEDWINDOW, WS_POPUP, WS_SYSMENU, WS_TABSTOP, WS_VISIBLE, WS_VSCROLL,
};
use windows_sys::core::{GUID, HRESULT, IID_IUnknown, IUnknown_Vtbl};

use crate::domain::{
    APP_REPOSITORY_URL, APP_VERSION, ARGUMENT_TOKENS, Category, ClientSize, CommandButton,
    CommandButtonMutationError, CommandTab, DEFAULT_FONT_FAMILY, DEFAULT_FONT_SIZE,
    DomainValidationError, ExecutionType, LayoutSpec, MainContentLayout, MainWindowSpec,
    MenuDefinition, MenuItemDefinition, RectSpec, TreeRootItemRef, UI_FONT_SIZE_OPTIONS,
    UiLanguage, ViewSettings, ViewTheme, WindowSize, Workspace, about_license_heading,
    current_ui_language, default_workspace_language_for_options,
    default_workspace_language_options, default_workspace_name_for_path,
    infer_workspace_language_from_entry_names, main_menu_for_language, normalize_ui_font_size,
    normalize_workspace_language, normalize_workspace_language_options, scale_dimension_for_dpi,
    unknown_argument_tokens, workspace_belongs_to_category, workspace_paths_equal,
};
use crate::error::{AppError, AppResult};
#[cfg(test)]
use crate::infra::settings;

mod command_execution;
mod dialogs;
mod persistence;

use self::layout_rules::{
    CommandButtonMoveDirection, CommandTabMoveDirection, TreeKeyboardMoveDirection,
    WorkspaceTreeDropAction, WorkspaceTreeDropTarget,
};
use command_execution::{CommandExecutionUi, execute_selected_command_button};
use dialogs::{
    CommandButtonDialogMode, TextInputDialogSpec, WorkspaceDialogMode, show_about_dialog,
    show_argument_text_input_dialog, show_command_button_dialog, show_font_dialog,
    show_language_config_dialog, show_text_input_dialog, show_workspace_dialog,
};
use persistence::{SettingsRestorePoint, persist_settings_or_restore};

const WINDOW_CLASS_NAME: &str = "j3DevHelper.MainWindow";
const COMMAND_TAB_PAGE_CLASS_NAME: &str = "j3DevHelper.CommandTabPage";
const MENU_ID_BASE: usize = 1000;
const MENU_FILE_FONT_ID: u32 = MENU_ID_BASE as u32;
const MENU_FILE_THEME_ID: u32 = (MENU_ID_BASE + 1) as u32;
const MENU_FILE_UI_LANGUAGE_ID: u32 = (MENU_ID_BASE + 2) as u32;
const MENU_FILE_LANGUAGE_CONFIG_ID: u32 = (MENU_ID_BASE + 3) as u32;
const MENU_FILE_ABOUT_ID: u32 = (MENU_ID_BASE + 4) as u32;
const MENU_FILE_CLOSE_ID: u32 = (MENU_ID_BASE + 5) as u32;
const MENU_TREE_ADD_ID: u32 = (MENU_ID_BASE + 100) as u32;
const MENU_TREE_CATEGORY_ADD_ID: u32 = (MENU_ID_BASE + 101) as u32;
const MENU_TREE_EDIT_ID: u32 = (MENU_ID_BASE + 102) as u32;
const MENU_TREE_MOVE_UP_ID: u32 = (MENU_ID_BASE + 103) as u32;
const MENU_TREE_MOVE_DOWN_ID: u32 = (MENU_ID_BASE + 104) as u32;
const MENU_TREE_DELETE_ID: u32 = (MENU_ID_BASE + 105) as u32;
const MENU_TABS_ADD_ID: u32 = (MENU_ID_BASE + 200) as u32;
const MENU_TABS_RENAME_ID: u32 = (MENU_ID_BASE + 201) as u32;
const MENU_TABS_MOVE_LEFT_ID: u32 = (MENU_ID_BASE + 202) as u32;
const MENU_TABS_MOVE_RIGHT_ID: u32 = (MENU_ID_BASE + 203) as u32;
const MENU_TABS_DELETE_ID: u32 = (MENU_ID_BASE + 204) as u32;
const MENU_COMMANDS_EXECUTE_ID: u32 = (MENU_ID_BASE + 300) as u32;
const MENU_COMMANDS_ADD_ID: u32 = (MENU_ID_BASE + 301) as u32;
const MENU_COMMANDS_EDIT_ID: u32 = (MENU_ID_BASE + 302) as u32;
const MENU_COMMANDS_MOVE_PREVIOUS_ID: u32 = (MENU_ID_BASE + 303) as u32;
const MENU_COMMANDS_MOVE_NEXT_ID: u32 = (MENU_ID_BASE + 304) as u32;
const MENU_COMMANDS_DELETE_ID: u32 = (MENU_ID_BASE + 305) as u32;
const MENU_THEME_SYSTEM_ID: u32 = (MENU_ID_BASE + 410) as u32;
const MENU_THEME_LIGHT_ID: u32 = (MENU_ID_BASE + 411) as u32;
const MENU_THEME_CLASSIC_DARK_ID: u32 = (MENU_ID_BASE + 412) as u32;
const MENU_THEME_SEPIA_TEAL_ID: u32 = (MENU_ID_BASE + 413) as u32;
const MENU_THEME_GRAPHITE_ID: u32 = (MENU_ID_BASE + 414) as u32;
const MENU_THEME_FOREST_ID: u32 = (MENU_ID_BASE + 415) as u32;
const MENU_THEME_STEEL_BLUE_ID: u32 = (MENU_ID_BASE + 416) as u32;
const MENU_UI_LANGUAGE_KOREAN_ID: u32 = (MENU_ID_BASE + 430) as u32;
const MENU_UI_LANGUAGE_ENGLISH_ID: u32 = (MENU_ID_BASE + 431) as u32;
const COMMAND_TAB_SELECTOR_CONTROL_ID: i32 = 5500;
const COMMAND_BUTTON_CONTROL_ID_BASE: i32 = 6000;
const COMMAND_BUTTON_SUBCLASS_ID: usize = 1;
const TREE_VIEW_SUBCLASS_ID: usize = 2;
const APP_ICON_RESOURCE_ID: u16 = 1;
const WHEEL_DELTA: i32 = 120;
const VK_CONTROL_KEY: i32 = 0x11;
const VK_LEFT_KEY: WPARAM = 0x25;
const VK_UP_KEY: WPARAM = 0x26;
const VK_DOWN_KEY: WPARAM = 0x28;
const WORKSPACE_LANGUAGE_INFERENCE_ENTRY_LIMIT: usize = 256;
const MIN_TREE_PANEL_WIDTH: i32 = 72;
const MIN_COMMAND_BUTTON_WIDTH: i32 = 96;
const COMMAND_BUTTON_PREFERRED_WIDTH: i32 = 132;
const COMMAND_BUTTON_HORIZONTAL_PADDING: i32 = 10;
const MIN_COMMAND_TABS_PANEL_WIDTH: i32 =
    MIN_COMMAND_BUTTON_WIDTH + COMMAND_BUTTON_HORIZONTAL_PADDING * 2;
const MIN_MAIN_PANEL_HEIGHT: i32 = 160;
const DIALOG_FONT_SIZE_PROPERTY_NAME: &str = "j3DevHelper.DialogFontSize";
const DIALOG_FONT_SCALE_DENOMINATOR: i32 = (DEFAULT_FONT_SIZE as i32) + 4;
const COMMAND_TAB_SELECTOR_HEIGHT: i32 = 24;
const COMMAND_TAB_SELECTOR_PAGE_GAP: i32 = 6;
const COMMAND_TAB_SELECTOR_DROPDOWN_EXTRA_HEIGHT: i32 = 144;
const COMMAND_BUTTON_DRAG_THRESHOLD: i32 = 4;
const DARK_MODE_EXPLORER_THEME: [u16; 18] = [
    'D' as u16, 'a' as u16, 'r' as u16, 'k' as u16, 'M' as u16, 'o' as u16, 'd' as u16, 'e' as u16,
    '_' as u16, 'E' as u16, 'x' as u16, 'p' as u16, 'l' as u16, 'o' as u16, 'r' as u16, 'e' as u16,
    'r' as u16, 0,
];
const GDI_OPAQUE_BACKGROUND_MODE: i32 = 2;
const TREE_VIEW_USE_SYSTEM_COLOR: LPARAM = -1;
const WM_WORKSPACE_DROP_CHECKED: u32 = WM_APP + 1;
static NEXT_WORKSPACE_DROP_CHECK_ID: AtomicU32 = AtomicU32::new(1);

fn tr(language: UiLanguage, korean: &'static str, english: &'static str) -> &'static str {
    language.text(korean, english)
}

fn context_language(context: &WindowContext) -> UiLanguage {
    current_ui_language(&context.spec.state.settings().view)
}

fn context_tr(
    context: &WindowContext,
    korean: &'static str,
    english: &'static str,
) -> &'static str {
    tr(context_language(context), korean, english)
}

fn localized_domain_error(language: UiLanguage, error: &DomainValidationError) -> String {
    error.user_message_for_language(language)
}

fn localized_command_button_mutation_error(
    language: UiLanguage,
    error: &CommandButtonMutationError,
) -> String {
    error.user_message_for_language(language)
}

pub fn run_main_window(mut spec: MainWindowSpec) -> AppResult<()> {
    enable_dpi_awareness();
    let instance = current_module_handle()?;
    initialize_common_controls()?;
    let _ole = initialize_ole_for_ui()?;
    let startup_dpi = system_dpi();
    let icons = load_application_icons(instance);
    let startup_font_warnings = validate_startup_font_settings(&mut spec.state);
    let startup_view = spec.state.settings().view.clone();
    spec.layout = startup_layout(&startup_view, startup_dpi);
    spec.initial_size = startup_window_size(&spec);
    let ui_font = UiFont::from_view(&spec.state.settings().view, startup_dpi);
    let theme_resources = ThemeResources::new(current_view_theme(&spec.state.settings().view))?;

    let mut context = Box::new(WindowContext {
        spec,
        instance,
        dpi: startup_dpi,
        content: MainContentUi::empty(),
        category_tree_items: Vec::new(),
        tree_items: Vec::new(),
        tree_view_snapshot: TreeViewSnapshot::default(),
        splitter_drag: None,
        workspace_tree_drag: None,
        command_button_controls: Vec::new(),
        command_button_tooltip: null_mut(),
        command_button_tooltip_texts: Vec::new(),
        command_button_drag: None,
        command_button_scroll_offset: 0,
        drop_target: None,
        workspace_drop_check: None,
        ui_font,
        theme_resources,
        startup_font_warnings,
    });
    let class_name = wide_null(WINDOW_CLASS_NAME);
    let command_tab_page_class_name = wide_null(COMMAND_TAB_PAGE_CLASS_NAME);
    let title = wide_null(context.spec.title);

    register_window_class(instance, &class_name, &icons)?;
    register_command_tab_page_class(instance, &command_tab_page_class_name)?;
    let menu = create_menu_bar(
        context.spec.menus,
        current_view_theme(&context.spec.state.settings().view),
        current_ui_language(&context.spec.state.settings().view),
    )?;
    let hwnd = create_main_window(instance, &class_name, &title, menu, context.as_mut())?;
    if let Err(error) = create_main_content(hwnd, instance, context.as_mut()) {
        // SAFETY: hwnd is a valid top-level window returned by CreateWindowExW. Destroying it
        // releases any child controls that may have been created before the error.
        unsafe {
            DestroyWindow(hwnd);
        }
        return Err(error);
    }

    set_window_icons(hwnd, &icons);
    show_window(hwnd);
    show_startup_font_warnings(
        hwnd,
        &context.startup_font_warnings,
        context_language(&context),
    );
    message_loop()
}

struct WindowContext {
    spec: MainWindowSpec,
    instance: HINSTANCE,
    dpi: u32,
    content: MainContentUi,
    category_tree_items: Vec<HTREEITEM>,
    tree_items: Vec<HTREEITEM>,
    tree_view_snapshot: TreeViewSnapshot,
    splitter_drag: Option<SplitterDrag>,
    workspace_tree_drag: Option<WorkspaceTreeDrag>,
    command_button_controls: Vec<HWND>,
    command_button_tooltip: HWND,
    command_button_tooltip_texts: Vec<Vec<u16>>,
    command_button_drag: Option<CommandButtonDrag>,
    command_button_scroll_offset: i32,
    drop_target: Option<DropTargetRegistration>,
    workspace_drop_check: Option<PendingWorkspaceDropCheck>,
    ui_font: UiFont,
    theme_resources: ThemeResources,
    startup_font_warnings: Vec<String>,
}

#[derive(Default)]
struct TreeViewSnapshot {
    root_items: Vec<TreeRootItemRef>,
    category_names: Vec<String>,
    workspace_categories: Vec<Option<String>>,
}

struct WorkspaceTreeGroups {
    category_workspace_indexes: Vec<Vec<usize>>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct SplitterDrag;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct WorkspaceTreeDrag {
    source_index: usize,
    target: Option<WorkspaceTreeDropTarget>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct CommandButtonDrag {
    source_index: usize,
    start_x: i32,
    start_y: i32,
    target_index: Option<usize>,
    moved: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum TreeNodeSelection {
    Workspace(usize),
    Category(usize),
}

mod layout_rules {
    pub(super) use crate::domain::{
        CommandButtonMoveDirection, CommandTabMoveDirection, TreeKeyboardMoveDirection,
        WorkspaceTreeDropAction, WorkspaceTreeDropTarget, command_button_drop_destination,
        command_button_move_destination, command_tab_move_destination,
        tree_root_keyboard_move_destination, workspace_belongs_to_category,
        workspace_category_index, workspace_keyboard_move_destination, workspace_tree_drop_action,
    };
    #[cfg(test)]
    pub(super) use crate::domain::{
        tree_root_drop_destination, workspace_tree_drop_destination,
        workspace_tree_visible_group_drop_destination,
    };
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct TreeMenuState {
    can_delete_tree_item: bool,
    can_edit_tree_item: bool,
    can_move_up: bool,
    can_move_down: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct CommandButtonContextMenuState {
    can_execute: bool,
    can_edit: bool,
    can_delete: bool,
    can_move_previous: bool,
    can_move_next: bool,
    can_add_command: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct CommandMenuState {
    can_execute: bool,
    can_add: bool,
    can_edit: bool,
    can_delete: bool,
    can_move_previous: bool,
    can_move_next: bool,
}

struct UiFont {
    handle: HFONT,
    owned: bool,
}

impl UiFont {
    fn from_view(view: &ViewSettings, dpi: u32) -> Self {
        match create_font_handle(&view.font_family, view.font_size, dpi) {
            Some(handle) => Self {
                handle,
                owned: true,
            },
            None => Self::default_stock(),
        }
    }

    fn default_stock() -> Self {
        // SAFETY: DEFAULT_GUI_FONT is a stock object owned by the system.
        let handle = unsafe { GetStockObject(DEFAULT_GUI_FONT) as HFONT };
        Self {
            handle,
            owned: false,
        }
    }

    fn handle(&self) -> HFONT {
        self.handle
    }
}

impl Drop for UiFont {
    fn drop(&mut self) {
        if self.owned && !self.handle.is_null() {
            // SAFETY: owned font handles are created by CreateFontIndirectW in this module.
            unsafe {
                DeleteObject(self.handle as HGDIOBJ);
            }
        }
    }
}

#[derive(Clone, Copy)]
struct ThemePalette {
    window_background: u32,
    control_background: u32,
    control_text: u32,
    tree_line: u32,
    custom_controls: bool,
}

impl ThemePalette {
    fn for_theme(theme: ViewTheme) -> Self {
        match theme {
            ViewTheme::System | ViewTheme::Light => Self {
                window_background: colorref(240, 240, 240),
                control_background: colorref(255, 255, 255),
                control_text: colorref(0, 0, 0),
                tree_line: colorref(160, 160, 160),
                custom_controls: false,
            },
            ViewTheme::ClassicDark => Self {
                window_background: colorref(31, 33, 36),
                control_background: colorref(24, 26, 29),
                control_text: colorref(230, 232, 235),
                tree_line: colorref(92, 97, 105),
                custom_controls: true,
            },
            ViewTheme::SepiaTeal => Self {
                window_background: colorref(24, 25, 24),
                control_background: colorref(31, 52, 56),
                control_text: colorref(236, 232, 219),
                tree_line: colorref(178, 154, 124),
                custom_controls: true,
            },
            ViewTheme::Graphite => Self {
                window_background: colorref(24, 25, 26),
                control_background: colorref(50, 55, 63),
                control_text: colorref(239, 236, 229),
                tree_line: colorref(126, 119, 105),
                custom_controls: true,
            },
            ViewTheme::Forest => Self {
                window_background: colorref(22, 25, 23),
                control_background: colorref(39, 59, 63),
                control_text: colorref(236, 239, 229),
                tree_line: colorref(104, 150, 117),
                custom_controls: true,
            },
            ViewTheme::SteelBlue => Self {
                window_background: colorref(24, 25, 27),
                control_background: colorref(54, 64, 80),
                control_text: colorref(239, 240, 242),
                tree_line: colorref(104, 139, 171),
                custom_controls: true,
            },
        }
    }

    fn uses_custom_controls(self) -> bool {
        self.custom_controls
    }
}

struct ThemeResources {
    window_brush: GdiBrush,
    control_brush: GdiBrush,
    command_button_drop_target_brush: GdiBrush,
}

impl ThemeResources {
    fn new(theme: ViewTheme) -> AppResult<Self> {
        let palette = ThemePalette::for_theme(theme);
        Ok(Self {
            window_brush: GdiBrush::new(palette.window_background)?,
            control_brush: GdiBrush::new(palette.control_background)?,
            command_button_drop_target_brush: GdiBrush::new(
                command_button_drop_target_background(palette),
            )?,
        })
    }

    fn window_brush(&self) -> HBRUSH {
        self.window_brush.handle()
    }

    fn control_brush(&self) -> HBRUSH {
        self.control_brush.handle()
    }

    fn command_button_drop_target_brush(&self) -> HBRUSH {
        self.command_button_drop_target_brush.handle()
    }
}

struct GdiBrush {
    handle: HBRUSH,
}

impl GdiBrush {
    fn new(color: u32) -> AppResult<Self> {
        // SAFETY: CreateSolidBrush accepts any COLORREF value.
        let handle = unsafe { CreateSolidBrush(color) };
        if handle.is_null() {
            return Err(last_error("CreateSolidBrush"));
        }

        Ok(Self { handle })
    }

    fn handle(&self) -> HBRUSH {
        self.handle
    }
}

impl Drop for GdiBrush {
    fn drop(&mut self) {
        if !self.handle.is_null() {
            // SAFETY: handle is an owned GDI brush created by CreateSolidBrush in this module.
            unsafe {
                DeleteObject(self.handle as HGDIOBJ);
            }
        }
    }
}

fn current_view_theme(view: &ViewSettings) -> ViewTheme {
    ViewTheme::from_config_value(&view.theme).unwrap_or_default()
}

struct MainContentUi {
    tree_view: HWND,
    command_tabs: CommandTabsUi,
}

impl MainContentUi {
    fn empty() -> Self {
        Self {
            tree_view: null_mut(),
            command_tabs: CommandTabsUi::empty(),
        }
    }

    fn is_created(&self) -> bool {
        !self.tree_view.is_null() && self.command_tabs.is_created()
    }

    fn handles(&self) -> [HWND; 3] {
        let command_tabs = self.command_tabs.handles();
        [self.tree_view, command_tabs[0], command_tabs[1]]
    }
}

struct CommandTabsUi {
    selector: HWND,
    page: HWND,
}

impl CommandTabsUi {
    fn empty() -> Self {
        Self {
            selector: null_mut(),
            page: null_mut(),
        }
    }

    fn new(selector: HWND, page: HWND) -> Self {
        Self { selector, page }
    }

    fn is_created(&self) -> bool {
        !self.selector.is_null() && !self.page.is_null()
    }

    fn handles(&self) -> [HWND; 2] {
        [self.selector, self.page]
    }

    fn page_rect(&self, panel: RectSpec, font_size: u16, dpi: u32) -> RectSpec {
        command_tab_page_rect(panel, font_size, dpi)
    }

    fn layout(&self, layout: MainContentLayout, font_size: u16, dpi: u32) -> RectSpec {
        move_child(
            self.selector,
            command_tab_selector_rect(layout.command_tabs_panel, font_size, dpi),
        );
        let page = self.page_rect(layout.command_tabs_panel, font_size, dpi);
        move_child(self.page, page);
        page
    }
}

struct IconPair {
    large: HICON,
    small: HICON,
}

impl IconPair {
    fn has_any(&self) -> bool {
        !self.large.is_null() || !self.small.is_null()
    }

    fn is_complete(&self) -> bool {
        !self.large.is_null() && !self.small.is_null()
    }
}

impl Drop for IconPair {
    fn drop(&mut self) {
        // SAFETY: Handles are either null or icon handles returned by LoadImageW with
        // LR_LOADFROMFILE. DestroyIcon is the matching release operation for those handles.
        unsafe {
            if !self.large.is_null() {
                DestroyIcon(self.large);
            }

            if !self.small.is_null() && self.small != self.large {
                DestroyIcon(self.small);
            }
        }
    }
}

fn enable_dpi_awareness() {
    // SAFETY: Process DPI awareness must be set before creating windows. If the embedded manifest
    // already established awareness, Windows returns failure and keeps the manifest policy.
    unsafe {
        SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
    }
}

fn system_dpi() -> u32 {
    u32::try_from(screen_logical_pixels_y())
        .unwrap_or(96)
        .max(1)
}

fn window_dpi(hwnd: HWND) -> u32 {
    if hwnd.is_null() {
        return system_dpi();
    }

    // SAFETY: hwnd is a window handle owned by this UI thread. A zero return is guarded below.
    let dpi = unsafe { GetDpiForWindow(hwnd) };
    if dpi > 0 { dpi } else { system_dpi() }
}

fn startup_window_size(spec: &MainWindowSpec) -> WindowSize {
    let view = &spec.state.settings().view;
    match (view.window_width, view.window_height) {
        (Some(width), Some(height)) => WindowSize { width, height },
        _ => spec.initial_size,
    }
}

fn startup_layout(view: &ViewSettings, dpi: u32) -> LayoutSpec {
    let mut layout = LayoutSpec::for_font_size_and_dpi(view.font_size, dpi);
    if let Some(tree_panel_width) = view.tree_panel_width {
        layout.tree_panel_width = tree_panel_width;
    }
    layout
}

fn scale_dimension_between_dpi(value: i32, from_dpi: u32, to_dpi: u32) -> i32 {
    let from_dpi = i64::from(from_dpi.max(1));
    let scaled = i64::from(value) * i64::from(to_dpi.max(1));
    let scaled = (scaled + from_dpi / 2) / from_dpi;
    scaled.clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32
}

fn scale_rect_for_dpi(rect: RectSpec, dpi: u32) -> RectSpec {
    RectSpec {
        x: scale_dimension_for_dpi(rect.x, dpi),
        y: scale_dimension_for_dpi(rect.y, dpi),
        width: scale_dimension_for_dpi(rect.width, dpi),
        height: scale_dimension_for_dpi(rect.height, dpi),
    }
}

fn scaled_dialog_size_for_dpi(width: i32, height: i32, font_size: u16, dpi: u32) -> (i32, i32) {
    let width = scale_dimension_for_dialog_font(width, font_size);
    let height = scale_dimension_for_dialog_font(height, font_size);
    (
        scale_dimension_for_dpi(width, dpi),
        scale_dimension_for_dpi(height, dpi),
    )
}

fn scale_rect_for_dialog(rect: RectSpec, font_size: u16, dpi: u32) -> RectSpec {
    scale_rect_for_dpi(
        RectSpec {
            x: scale_dimension_for_dialog_font(rect.x, font_size),
            y: scale_dimension_for_dialog_font(rect.y, font_size),
            width: scale_dimension_for_dialog_font(rect.width, font_size),
            height: scale_dimension_for_dialog_font(rect.height, font_size),
        },
        dpi,
    )
}

fn scale_dimension_for_dialog_font(value: i32, font_size: u16) -> i32 {
    let font_size = normalize_ui_font_size(font_size);
    let extra = i32::from(font_size.saturating_sub(DEFAULT_FONT_SIZE));
    if extra == 0 {
        return value;
    }

    let denominator = i64::from(DIALOG_FONT_SCALE_DENOMINATOR);
    let numerator = i64::from(DIALOG_FONT_SCALE_DENOMINATOR + extra);
    let scaled = i64::from(value) * numerator;
    ((scaled + denominator / 2) / denominator).clamp(i64::from(i32::MIN), i64::from(i32::MAX))
        as i32
}

fn dialog_layout_font_size(font: HFONT, configured_font_size: u16, dpi: u32) -> u16 {
    dialog_layout_font_size_from_actual(configured_font_size, font_size_from_font_handle(font, dpi))
}

fn dialog_layout_font_size_from_actual(
    configured_font_size: u16,
    actual_font_size: Option<u16>,
) -> u16 {
    let configured_font_size = normalize_ui_font_size(configured_font_size);
    actual_font_size
        .map(normalize_ui_font_size)
        .unwrap_or(configured_font_size)
        .max(configured_font_size)
}

fn font_size_from_font_handle(font: HFONT, dpi: u32) -> Option<u16> {
    if font.is_null() {
        return None;
    }

    let mut logfont = LOGFONTW::default();
    // SAFETY: font is an HFONT selected by this process or a stock font. logfont points to
    // initialized writable storage large enough for LOGFONTW.
    let copied = unsafe {
        GetObjectW(
            font as HGDIOBJ,
            size_of::<LOGFONTW>() as i32,
            &mut logfont as *mut LOGFONTW as *mut c_void,
        )
    };
    if copied <= 0 {
        return None;
    }

    font_size_from_logfont_height(logfont.lfHeight, dpi)
}

fn font_size_from_logfont_height(height: i32, dpi: u32) -> Option<u16> {
    if height == 0 {
        return None;
    }

    let dpi = i64::from(dpi.max(1));
    let pixels = i64::from(height).abs();
    let points = (pixels * 72 + dpi / 2) / dpi;
    u16::try_from(points).ok().map(normalize_ui_font_size)
}

fn set_dialog_font_size(hwnd: HWND, font_size: u16) -> AppResult<()> {
    let property_name = wide_null(DIALOG_FONT_SIZE_PROPERTY_NAME);
    let font_size = usize::from(normalize_ui_font_size(font_size));
    // SAFETY: hwnd is a live dialog window. The property value is an integer-sized token; it is
    // read back by this process only and is not dereferenced as a pointer.
    let ok = unsafe { SetPropW(hwnd, property_name.as_ptr(), font_size as HANDLE) };
    if ok == 0 {
        Err(last_error("SetPropW dialog font size"))
    } else {
        Ok(())
    }
}

fn dialog_font_size(hwnd: HWND) -> u16 {
    let property_name = wide_null(DIALOG_FONT_SIZE_PROPERTY_NAME);
    // SAFETY: hwnd is a live dialog window. A null result means the property was not set.
    let value = unsafe { GetPropW(hwnd, property_name.as_ptr()) as usize };
    if value == 0 {
        return DEFAULT_FONT_SIZE;
    }

    u16::try_from(value)
        .map(normalize_ui_font_size)
        .unwrap_or(DEFAULT_FONT_SIZE)
}

unsafe fn clear_dialog_font_size(hwnd: HWND) {
    let property_name = wide_null(DIALOG_FONT_SIZE_PROPERTY_NAME);
    // SAFETY: hwnd is the dialog window whose process-local property may have been set above.
    unsafe {
        RemovePropW(hwnd, property_name.as_ptr());
    }
}

fn current_module_handle() -> AppResult<HINSTANCE> {
    // SAFETY: A null module name asks Windows for the current process module handle.
    let instance = unsafe { GetModuleHandleW(null()) };
    if instance.is_null() {
        Err(last_error("GetModuleHandleW"))
    } else {
        Ok(instance)
    }
}

fn initialize_common_controls() -> AppResult<()> {
    let controls = INITCOMMONCONTROLSEX {
        dwSize: size_of::<INITCOMMONCONTROLSEX>() as u32,
        dwICC: ICC_TREEVIEW_CLASSES | ICC_WIN95_CLASSES | ICC_LINK_CLASS,
    };

    // SAFETY: controls points to an initialized INITCOMMONCONTROLSEX value for this call.
    let ok = unsafe { InitCommonControlsEx(&controls) };
    if ok == 0 {
        Err(last_error("InitCommonControlsEx"))
    } else {
        Ok(())
    }
}

struct OleApartment;

impl Drop for OleApartment {
    fn drop(&mut self) {
        // SAFETY: This guard is only constructed after a successful OleInitialize call on the
        // current UI thread, so OleUninitialize is the matching cleanup operation.
        unsafe {
            OleUninitialize();
        }
    }
}

fn initialize_ole_for_ui() -> AppResult<OleApartment> {
    // SAFETY: Initializing OLE on the UI thread enables shell folder picking and OLE drag/drop.
    // A null reserved pointer is the documented value.
    let result = unsafe { OleInitialize(null()) };

    if result >= 0 {
        Ok(OleApartment)
    } else {
        Err(AppError::windows_hresult("OleInitialize", result))
    }
}

fn register_window_class(
    instance: HINSTANCE,
    class_name: &[u16],
    icons: &IconPair,
) -> AppResult<()> {
    // SAFETY: Loading the predefined IDC_ARROW cursor with a null instance is the documented
    // Win32 pattern. A null cursor is still accepted by RegisterClassExW, so failure is nonfatal.
    let cursor = unsafe { LoadCursorW(null_mut(), IDC_ARROW) };
    let window_class = WNDCLASSEXW {
        cbSize: size_of::<WNDCLASSEXW>() as u32,
        style: CS_HREDRAW | CS_VREDRAW,
        lpfnWndProc: Some(window_proc),
        cbClsExtra: 0,
        cbWndExtra: 0,
        hInstance: instance,
        hIcon: icons.large,
        hCursor: cursor,
        hbrBackground: system_color_brush(COLOR_WINDOW),
        lpszMenuName: null(),
        lpszClassName: class_name.as_ptr(),
        hIconSm: icons.small,
    };

    // SAFETY: window_class points to a fully initialized WNDCLASSEXW. class_name and icon
    // handles outlive the call, and the icon handles are kept alive for the message loop.
    let atom = unsafe { RegisterClassExW(&window_class) };
    if atom == 0 {
        let code = unsafe { GetLastError() };
        if code != ERROR_CLASS_ALREADY_EXISTS {
            return Err(AppError::windows_api("RegisterClassExW", code));
        }
    }

    Ok(())
}

fn register_command_tab_page_class(instance: HINSTANCE, class_name: &[u16]) -> AppResult<()> {
    // SAFETY: Loading the predefined IDC_ARROW cursor with a null instance is the documented
    // Win32 pattern. A null cursor is still accepted by RegisterClassExW, so failure is nonfatal.
    let cursor = unsafe { LoadCursorW(null_mut(), IDC_ARROW) };
    let window_class = WNDCLASSEXW {
        cbSize: size_of::<WNDCLASSEXW>() as u32,
        style: CS_HREDRAW | CS_VREDRAW,
        lpfnWndProc: Some(command_tab_page_proc),
        cbClsExtra: 0,
        cbWndExtra: 0,
        hInstance: instance,
        hIcon: null_mut(),
        hCursor: cursor,
        hbrBackground: null_mut(),
        lpszMenuName: null(),
        lpszClassName: class_name.as_ptr(),
        hIconSm: null_mut(),
    };

    // SAFETY: window_class points to a fully initialized WNDCLASSEXW. class_name outlives the
    // call, and the registered class is process-wide.
    let atom = unsafe { RegisterClassExW(&window_class) };
    if atom == 0 {
        let code = unsafe { GetLastError() };
        if code != ERROR_CLASS_ALREADY_EXISTS {
            return Err(AppError::windows_api(
                "RegisterClassExW command tab page",
                code,
            ));
        }
    }

    Ok(())
}

fn create_main_window(
    instance: HINSTANCE,
    class_name: &[u16],
    title: &[u16],
    menu: HMENU,
    context: &mut WindowContext,
) -> AppResult<HWND> {
    // SAFETY: The class has been registered in this process. The menu is either transferred to
    // the created window or explicitly destroyed on failure. context is a boxed value that remains
    // alive until the message loop exits.
    let hwnd = unsafe {
        CreateWindowExW(
            0,
            class_name.as_ptr(),
            title.as_ptr(),
            WS_OVERLAPPEDWINDOW,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            context.spec.initial_size.width,
            context.spec.initial_size.height,
            null_mut(),
            menu,
            instance,
            context as *mut WindowContext as *const c_void,
        )
    };

    if hwnd.is_null() {
        // SAFETY: CreateWindowExW did not accept ownership when it returned null.
        unsafe {
            DestroyMenu(menu);
        }
        Err(last_error("CreateWindowExW"))
    } else {
        Ok(hwnd)
    }
}

fn create_main_content(
    hwnd: HWND,
    instance: HINSTANCE,
    context: &mut WindowContext,
) -> AppResult<()> {
    let layout = current_main_content_layout(hwnd, context.spec.layout)?;
    let tree_style = WS_CHILD
        | WS_VISIBLE
        | WS_BORDER
        | WS_TABSTOP
        | TVS_HASBUTTONS
        | TVS_HASLINES
        | TVS_LINESATROOT
        | TVS_SHOWSELALWAYS;
    let tree_style = tree_style | TVS_INFOTIP;

    let tree_view = create_child_window(
        hwnd,
        instance,
        WC_TREEVIEWW,
        "",
        tree_style,
        layout.tree_panel,
        "CreateWindowExW tree view",
    )?;
    install_tree_view_subclass(tree_view, context as *mut WindowContext)?;

    let command_tabs = create_command_tabs_ui(
        hwnd,
        instance,
        layout,
        context.spec.state.settings().view.font_size,
        context.dpi,
        context as *mut WindowContext,
    )?;
    context.command_button_tooltip = create_command_button_tooltip(hwnd, instance)?;

    context.content = MainContentUi {
        tree_view,
        command_tabs,
    };
    apply_ui_font_to_main(hwnd, context);
    refresh_tree_view(context);
    refresh_command_tab_selector(context);
    context.content.command_tabs.layout(
        layout,
        context.spec.state.settings().view.font_size,
        context.dpi,
    );
    refresh_command_buttons(hwnd, context);
    apply_window_theme(hwnd, context);
    update_tree_menu_state(hwnd, context);
    update_tabs_menu_state(hwnd, context);
    update_commands_menu_state(hwnd, context);
    register_workspace_drop_target(hwnd, context)?;

    Ok(())
}

fn create_command_tabs_ui(
    parent: HWND,
    instance: HINSTANCE,
    layout: MainContentLayout,
    font_size: u16,
    dpi: u32,
    context: *mut WindowContext,
) -> AppResult<CommandTabsUi> {
    let selector_style = WS_CHILD
        | WS_VISIBLE
        | WS_TABSTOP
        | WS_CLIPSIBLINGS
        | WS_VSCROLL
        | CBS_DROPDOWNLIST as u32
        | CBS_HASSTRINGS as u32;
    let page_style = command_tab_page_window_style();
    let selector_class = wide_null("COMBOBOX");
    let page_class = wide_null(COMMAND_TAB_PAGE_CLASS_NAME);

    let selector = create_child_window_with_id(
        parent,
        instance,
        selector_class.as_ptr(),
        "",
        selector_style,
        command_tab_selector_rect(layout.command_tabs_panel, font_size, dpi),
        COMMAND_TAB_SELECTOR_CONTROL_ID,
        "CreateWindowExW command tab selector",
    )?;
    let page = create_child_window_with_id_and_param(
        parent,
        instance,
        page_class.as_ptr(),
        "",
        page_style,
        command_tab_page_rect(layout.command_tabs_panel, font_size, dpi),
        0,
        context as *const c_void,
        "CreateWindowExW command tab page",
    )?;

    Ok(CommandTabsUi::new(selector, page))
}

fn create_command_button_tooltip(parent: HWND, instance: HINSTANCE) -> AppResult<HWND> {
    // SAFETY: TOOLTIPS_CLASSW is provided by common controls initialization, parent is the owning
    // top-level window, and tooltip controls do not use lpParam here.
    let tooltip = unsafe {
        CreateWindowExW(
            0,
            TOOLTIPS_CLASSW,
            null(),
            WS_POPUP | TTS_ALWAYSTIP | TTS_NOPREFIX,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            parent,
            0 as HMENU,
            instance,
            null(),
        )
    };

    if tooltip.is_null() {
        Err(last_error("CreateWindowExW command button tooltip"))
    } else {
        Ok(tooltip)
    }
}

fn create_child_window(
    parent: HWND,
    instance: HINSTANCE,
    class_name: *const u16,
    text: &str,
    style: u32,
    rect: RectSpec,
    operation: &'static str,
) -> AppResult<HWND> {
    create_child_window_with_id(
        parent, instance, class_name, text, style, rect, 0, operation,
    )
}

// This thin helper mirrors CreateWindowExW's shape so call sites keep the Win32 parameters visible.
#[allow(clippy::too_many_arguments)]
fn create_child_window_with_id(
    parent: HWND,
    instance: HINSTANCE,
    class_name: *const u16,
    text: &str,
    style: u32,
    rect: RectSpec,
    control_id: i32,
    operation: &'static str,
) -> AppResult<HWND> {
    create_child_window_with_id_and_param(
        parent,
        instance,
        class_name,
        text,
        style,
        rect,
        control_id,
        null(),
        operation,
    )
}

// This thin helper mirrors CreateWindowExW's shape so call sites that need lpParam keep the Win32
// parameters visible.
#[allow(clippy::too_many_arguments)]
fn create_child_window_with_id_and_param(
    parent: HWND,
    instance: HINSTANCE,
    class_name: *const u16,
    text: &str,
    style: u32,
    rect: RectSpec,
    control_id: i32,
    create_param: *const c_void,
    operation: &'static str,
) -> AppResult<HWND> {
    let text = wide_null(text);
    // SAFETY: parent is a valid top-level window, class_name names a registered Win32 class,
    // text is null-terminated, rect contains child-window coordinates in the client area, and
    // create_param either is null or points to caller-owned data consumed during WM_NCCREATE.
    let hwnd = unsafe {
        CreateWindowExW(
            0,
            class_name,
            text.as_ptr(),
            style,
            rect.x,
            rect.y,
            rect.width,
            rect.height,
            parent,
            control_id as usize as HMENU,
            instance,
            create_param,
        )
    };

    if hwnd.is_null() {
        Err(last_error(operation))
    } else {
        Ok(hwnd)
    }
}

fn apply_ui_font_to_main(hwnd: HWND, context: &WindowContext) {
    let font = context.ui_font.handle();
    if font.is_null() {
        return;
    }

    // SAFETY: hwnd is a top-level window owned by this module. WM_SETFONT is harmless for window
    // classes that ignore it; child controls below use the same handle explicitly.
    unsafe {
        SendMessageW(hwnd, WM_SETFONT, font as WPARAM, 1);
    }

    apply_font_to_handles(&context.content.handles(), font);
    apply_font_to_handles(&context.command_button_controls, font);
    apply_font_to_handles(&[context.command_button_tooltip], font);

    // SAFETY: The menu bar belongs to hwnd. Standard menus keep system drawing policy, but the
    // redraw keeps metrics in sync when Windows applies the changed window font to controls.
    unsafe {
        DrawMenuBar(hwnd);
    }
}

fn apply_font_to_handles(handles: &[HWND], font: HFONT) {
    if font.is_null() {
        return;
    }

    for handle in handles {
        if handle.is_null() {
            continue;
        }

        // SAFETY: handle is a child window created by this module. WM_SETFONT borrows the font
        // handle, which remains alive for at least the lifetime of the owning window or dialog.
        unsafe {
            SendMessageW(*handle, WM_SETFONT, font as WPARAM, 1);
            InvalidateRect(*handle, null(), 1);
        }
    }
}

fn apply_window_theme(hwnd: HWND, context: &WindowContext) {
    let theme = current_view_theme(&context.spec.state.settings().view);
    let palette = ThemePalette::for_theme(theme);
    let dark_mode = theme.uses_dark_mode();

    // SAFETY: hwnd and child handles are owned by this UI thread. Theme calls are best-effort UI
    // updates, and failures leave the controls on their previous system theme.
    unsafe {
        apply_title_bar_theme(hwnd, dark_mode);
        apply_control_window_theme(context.content.tree_view, dark_mode);
        apply_control_window_theme(context.content.command_tabs.selector, dark_mode);
        apply_control_window_theme(context.content.command_tabs.page, dark_mode);
        apply_control_window_theme(context.command_button_tooltip, dark_mode);
        for button in &context.command_button_controls {
            apply_control_window_theme(*button, dark_mode);
        }
        apply_tree_view_theme(context.content.tree_view, palette);
        invalidate_window(hwnd);
        for handle in context.content.handles() {
            invalidate_window(handle);
        }
        for button in &context.command_button_controls {
            invalidate_window(*button);
        }
    }
}

unsafe fn apply_title_bar_theme(hwnd: HWND, dark_theme: bool) {
    if hwnd.is_null() {
        return;
    }

    let enabled: i32 = if dark_theme { 1 } else { 0 };
    let _ = unsafe {
        DwmSetWindowAttribute(
            hwnd,
            DWMWA_USE_IMMERSIVE_DARK_MODE as u32,
            &enabled as *const i32 as *const c_void,
            size_of::<i32>() as u32,
        )
    };
}

unsafe fn apply_control_window_theme(hwnd: HWND, dark_theme: bool) {
    if hwnd.is_null() {
        return;
    }

    let theme_name = if dark_theme {
        DARK_MODE_EXPLORER_THEME.as_ptr()
    } else {
        null()
    };
    let _ = unsafe { SetWindowTheme(hwnd, theme_name, null()) };
}

unsafe fn apply_tree_view_theme(tree_view: HWND, palette: ThemePalette) {
    if tree_view.is_null() {
        return;
    }

    let colors = TreeViewThemeColors::for_palette(palette);
    unsafe {
        SendMessageW(tree_view, TVM_SETBKCOLOR, 0, colors.background);
        SendMessageW(tree_view, TVM_SETTEXTCOLOR, 0, colors.text);
        SendMessageW(tree_view, TVM_SETLINECOLOR, 0, colors.line);
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct TreeViewThemeColors {
    background: LPARAM,
    text: LPARAM,
    line: LPARAM,
}

impl TreeViewThemeColors {
    fn for_palette(palette: ThemePalette) -> Self {
        if palette.uses_custom_controls() {
            Self {
                background: palette.control_background as LPARAM,
                text: palette.control_text as LPARAM,
                line: palette.tree_line as LPARAM,
            }
        } else {
            Self {
                background: TREE_VIEW_USE_SYSTEM_COLOR,
                text: TREE_VIEW_USE_SYSTEM_COLOR,
                line: CLR_DEFAULT as LPARAM,
            }
        }
    }

    fn for_drop_feedback(feedback: DropFeedback, palette: ThemePalette) -> Self {
        let themed_colors = Self::for_palette(palette);
        let background = match feedback {
            DropFeedback::Normal => themed_colors.background,
            DropFeedback::Allowed => {
                drop_feedback_background(palette, colorref(226, 241, 255), colorref(44, 128, 190))
                    as LPARAM
            }
            DropFeedback::Denied => {
                drop_feedback_background(palette, colorref(255, 235, 235), colorref(150, 60, 60))
                    as LPARAM
            }
        };

        Self {
            background,
            ..themed_colors
        }
    }
}

fn drop_feedback_background(palette: ThemePalette, light_color: u32, accent_color: u32) -> u32 {
    if palette.uses_custom_controls() {
        blend_colorref(palette.control_background, accent_color, 36)
    } else {
        light_color
    }
}

fn command_button_drop_target_background(palette: ThemePalette) -> u32 {
    drop_feedback_background(palette, colorref(226, 241, 255), colorref(44, 128, 190))
}

fn workspace_tree_drop_target_background(palette: ThemePalette) -> u32 {
    drop_feedback_background(palette, colorref(226, 241, 255), colorref(44, 128, 190))
}

fn erase_window_background(hwnd: HWND, context: &WindowContext, wparam: WPARAM) -> Option<LRESULT> {
    let palette = ThemePalette::for_theme(current_view_theme(&context.spec.state.settings().view));
    if !palette.uses_custom_controls() {
        return None;
    }

    let hdc = wparam as HDC;
    if hdc.is_null() {
        return None;
    }

    let mut rect = RECT::default();
    // SAFETY: hwnd is the current top-level window and rect points to writable storage.
    let ok = unsafe { GetClientRect(hwnd, &mut rect) };
    if ok == 0 {
        return None;
    }

    // SAFETY: hdc comes from WM_ERASEBKGND for hwnd, rect is initialized, and the brush is owned
    // by context.theme_resources for the duration of painting.
    unsafe {
        FillRect(hdc, &rect, context.theme_resources.window_brush());
    }
    Some(1)
}

fn erase_command_tab_page_background(
    hwnd: HWND,
    context: &WindowContext,
    wparam: WPARAM,
) -> Option<LRESULT> {
    let hdc = wparam as HDC;
    if hwnd.is_null() || hdc.is_null() {
        return None;
    }

    let mut rect = RECT::default();
    // SAFETY: hwnd is the command-tab page window and rect points to writable storage.
    let ok = unsafe { GetClientRect(hwnd, &mut rect) };
    if ok == 0 {
        return None;
    }

    let brush = command_tab_page_background_brush(context);
    if brush.is_null() {
        return None;
    }

    // SAFETY: hdc comes from WM_ERASEBKGND for hwnd, rect is initialized, and brush is either a
    // system brush or a brush owned by context.theme_resources for the duration of painting.
    unsafe {
        FillRect(hdc, &rect, brush);
    }
    Some(1)
}

fn command_tab_page_background_brush(context: &WindowContext) -> HBRUSH {
    let palette = ThemePalette::for_theme(current_view_theme(&context.spec.state.settings().view));
    if palette.uses_custom_controls() {
        context.theme_resources.control_brush()
    } else {
        // SAFETY: GetSysColorBrush returns a system-owned stock brush that must not be deleted.
        unsafe { GetSysColorBrush(COLOR_WINDOW) }
    }
}

fn themed_static_color_brush(context: &WindowContext, wparam: WPARAM) -> Option<LRESULT> {
    let palette = ThemePalette::for_theme(current_view_theme(&context.spec.state.settings().view));
    if !palette.uses_custom_controls() {
        return None;
    }

    apply_themed_control_colors(wparam, palette.control_text, palette.window_background)?;
    Some(context.theme_resources.window_brush() as LRESULT)
}

fn themed_button_color_brush(
    context: &WindowContext,
    wparam: WPARAM,
    lparam: LPARAM,
) -> Option<LRESULT> {
    let palette = ThemePalette::for_theme(current_view_theme(&context.spec.state.settings().view));

    if command_button_drag_target_matches(context, lparam as HWND) {
        let background = command_button_drop_target_background(palette);
        apply_themed_control_colors(wparam, palette.control_text, background)?;
        return Some(context.theme_resources.command_button_drop_target_brush() as LRESULT);
    }

    if !palette.uses_custom_controls() {
        return None;
    }

    apply_themed_control_colors(wparam, palette.control_text, palette.control_background)?;
    Some(context.theme_resources.control_brush() as LRESULT)
}

fn apply_themed_control_colors(wparam: WPARAM, text: u32, background: u32) -> Option<()> {
    let hdc = wparam as HDC;
    if hdc.is_null() {
        return None;
    }

    // SAFETY: hdc is supplied by a WM_CTLCOLOR* message for the current paint operation.
    unsafe {
        SetTextColor(hdc, text);
        SetBkColor(hdc, background);
        SetBkMode(hdc, GDI_OPAQUE_BACKGROUND_MODE);
    }
    Some(())
}

unsafe fn invalidate_window(hwnd: HWND) {
    if !hwnd.is_null() {
        unsafe {
            InvalidateRect(hwnd, null(), 1);
        }
    }
}

fn create_font_handle(font_family: &str, font_size: u16, dpi: u32) -> Option<HFONT> {
    let mut log_font = LOGFONTW {
        lfHeight: -font_pixel_height(font_size, dpi),
        lfCharSet: DEFAULT_CHARSET,
        lfOutPrecision: OUT_DEFAULT_PRECIS,
        lfClipPrecision: CLIP_DEFAULT_PRECIS,
        lfQuality: DEFAULT_QUALITY,
        lfPitchAndFamily: DEFAULT_PITCH | FF_DONTCARE,
        ..LOGFONTW::default()
    };
    copy_face_name(&mut log_font, font_family);

    // SAFETY: log_font is fully initialized, and lfFaceName is null-terminated by zeroed storage.
    let handle = unsafe { CreateFontIndirectW(&log_font) };
    (!handle.is_null()).then_some(handle)
}

fn font_pixel_height(font_size: u16, dpi: u32) -> i32 {
    let font_size = i32::from(normalize_ui_font_size(font_size));
    let dpi = i32::try_from(dpi).unwrap_or(96).max(1);
    ((font_size * dpi) + 36) / 72
}

fn screen_logical_pixels_y() -> i32 {
    // SAFETY: A null HWND asks for the screen DC. ReleaseDC below is the matching release call.
    let hdc = unsafe { GetDC(null_mut()) };
    if hdc.is_null() {
        return 96;
    }

    // SAFETY: hdc is a live screen device context.
    let dpi = unsafe { GetDeviceCaps(hdc, LOGPIXELSY as i32) };
    // SAFETY: hdc came from GetDC(null).
    unsafe {
        ReleaseDC(null_mut(), hdc);
    }

    if dpi > 0 { dpi } else { 96 }
}

fn copy_face_name(log_font: &mut LOGFONTW, font_family: &str) {
    let family = if font_family.trim().is_empty() {
        DEFAULT_FONT_FAMILY
    } else {
        font_family.trim()
    };

    for (index, code_unit) in family
        .encode_utf16()
        .take(log_font.lfFaceName.len().saturating_sub(1))
        .enumerate()
    {
        log_font.lfFaceName[index] = code_unit;
    }
}

fn installed_font_families() -> Vec<String> {
    // SAFETY: A null HWND asks for the screen DC. ReleaseDC below is the matching release call.
    let hdc = unsafe { GetDC(null_mut()) };
    if hdc.is_null() {
        return vec![DEFAULT_FONT_FAMILY.to_owned()];
    }

    let log_font = LOGFONTW {
        lfCharSet: DEFAULT_CHARSET,
        ..LOGFONTW::default()
    };
    let mut fonts = Vec::<String>::new();

    // SAFETY: hdc is live, log_font requests all default-charset families, and lparam points to
    // fonts for the duration of the synchronous EnumFontFamiliesExW call.
    unsafe {
        EnumFontFamiliesExW(
            hdc,
            &log_font,
            Some(enum_font_family_proc),
            &mut fonts as *mut Vec<String> as LPARAM,
            0,
        );
        ReleaseDC(null_mut(), hdc);
    }

    normalize_font_family_list(fonts)
}

unsafe extern "system" fn enum_font_family_proc(
    log_font: *const LOGFONTW,
    _text_metric: *const TEXTMETRICW,
    _font_type: u32,
    lparam: LPARAM,
) -> i32 {
    if log_font.is_null() || lparam == 0 {
        return 1;
    }

    // SAFETY: lparam was supplied by installed_font_families as a Vec<String> pointer, and the
    // callback is only invoked during that synchronous call.
    let fonts = unsafe { &mut *(lparam as *mut Vec<String>) };
    // SAFETY: log_font is supplied by GDI for this callback invocation.
    let font = unsafe { &*log_font };
    let family = wide_null_buffer_to_string(&font.lfFaceName);
    let family = family.trim();

    if !family.is_empty() && !family.starts_with('@') {
        fonts.push(family.to_owned());
    }

    1
}

fn normalize_font_family_list(mut fonts: Vec<String>) -> Vec<String> {
    fonts.sort_by(|left, right| compare_font_family_case_insensitive(left, right));
    fonts.dedup_by(|left, right| left.eq_ignore_ascii_case(right));

    if fonts.is_empty() {
        fonts.push(DEFAULT_FONT_FAMILY.to_owned());
    }

    fonts
}

fn compare_font_family_case_insensitive(left: &str, right: &str) -> std::cmp::Ordering {
    let mut left_chars = left.chars().flat_map(char::to_lowercase);
    let mut right_chars = right.chars().flat_map(char::to_lowercase);

    loop {
        match (left_chars.next(), right_chars.next()) {
            (Some(left_char), Some(right_char)) => {
                let ordering = left_char.cmp(&right_char);
                if ordering != std::cmp::Ordering::Equal {
                    return ordering;
                }
            }
            (Some(_), None) => return std::cmp::Ordering::Greater,
            (None, Some(_)) => return std::cmp::Ordering::Less,
            (None, None) => return std::cmp::Ordering::Equal,
        }
    }
}

fn font_family_available(fonts: &[String], font_family: &str) -> bool {
    fonts
        .iter()
        .any(|candidate| candidate.eq_ignore_ascii_case(font_family))
}

fn validate_startup_font_settings(state: &mut crate::domain::AppState) -> Vec<String> {
    validate_startup_font_settings_with(state, installed_font_family_available)
}

fn validate_startup_font_settings_with(
    state: &mut crate::domain::AppState,
    mut font_family_available: impl FnMut(&str) -> bool,
) -> Vec<String> {
    let current = state.settings().view.clone();
    let mut normalized = ViewSettings::new(
        current.font_family.clone(),
        current.font_size,
        current.theme.clone(),
    )
    .with_ui_language(&current.ui_language)
    .with_window_layout_from(&current);
    let mut warnings = Vec::new();

    if !font_family_available(&normalized.font_family) {
        let warning = match current_ui_language(&current) {
            UiLanguage::Korean => format!(
                "글꼴을 찾을 수 없어 기본값을 사용합니다: {}",
                normalized.font_family
            ),
            UiLanguage::English => {
                format!(
                    "Font not found; using the default: {}",
                    normalized.font_family
                )
            }
        };
        normalized.font_family = DEFAULT_FONT_FAMILY.to_owned();
        warnings.push(warning.clone());
        state.add_restore_warning(warning);
    }

    if normalized != state.settings().view {
        state.set_view_settings(normalized);
    }

    warnings
}

struct FontFamilySearch<'a> {
    requested: &'a str,
    found: bool,
}

fn installed_font_family_available(font_family: &str) -> bool {
    let font_family = font_family.trim();
    let requested = if font_family.is_empty() {
        DEFAULT_FONT_FAMILY
    } else {
        font_family
    };
    if requested.starts_with('@') {
        return false;
    }

    // SAFETY: A null HWND asks for the screen DC. ReleaseDC below is the matching release call.
    let hdc = unsafe { GetDC(null_mut()) };
    if hdc.is_null() {
        return requested.eq_ignore_ascii_case(DEFAULT_FONT_FAMILY);
    }

    let mut log_font = LOGFONTW {
        lfCharSet: DEFAULT_CHARSET,
        ..LOGFONTW::default()
    };
    copy_face_name(&mut log_font, requested);
    let mut search = FontFamilySearch {
        requested,
        found: false,
    };

    // SAFETY: hdc is live, log_font requests a single face name, and lparam points to search for
    // the duration of the synchronous EnumFontFamiliesExW call.
    unsafe {
        EnumFontFamiliesExW(
            hdc,
            &log_font,
            Some(enum_font_family_match_proc),
            &mut search as *mut FontFamilySearch<'_> as LPARAM,
            0,
        );
        ReleaseDC(null_mut(), hdc);
    }

    search.found
}

unsafe extern "system" fn enum_font_family_match_proc(
    log_font: *const LOGFONTW,
    _text_metric: *const TEXTMETRICW,
    _font_type: u32,
    lparam: LPARAM,
) -> i32 {
    if log_font.is_null() || lparam == 0 {
        return 1;
    }

    // SAFETY: lparam was supplied by installed_font_family_available as a FontFamilySearch
    // pointer, and the callback is only invoked during that synchronous call.
    let search = unsafe { &mut *(lparam as *mut FontFamilySearch<'_>) };
    // SAFETY: log_font is supplied by GDI for this callback invocation.
    let font = unsafe { &*log_font };
    let family = wide_null_buffer_to_string(&font.lfFaceName);

    if family.trim().eq_ignore_ascii_case(search.requested) {
        search.found = true;
        return 0;
    }

    1
}

fn show_startup_font_warnings(hwnd: HWND, warnings: &[String], language: UiLanguage) {
    if warnings.is_empty() {
        return;
    }

    show_warning_message(hwnd, tr(language, "글꼴", "Font"), &warnings.join("\n"));
}

fn main_client_size(hwnd: HWND) -> AppResult<ClientSize> {
    let mut client = RECT::default();
    // SAFETY: client points to writable RECT storage and hwnd is a valid window handle.
    let ok = unsafe { GetClientRect(hwnd, &mut client) };
    if ok == 0 {
        return Err(last_error("GetClientRect"));
    }

    Ok(ClientSize {
        width: client.right - client.left,
        height: client.bottom - client.top,
    })
}

fn current_main_content_layout(hwnd: HWND, layout: LayoutSpec) -> AppResult<MainContentLayout> {
    Ok(layout.arrange_main_content(main_client_size(hwnd)?))
}

fn minimum_main_client_size(layout: LayoutSpec, dpi: u32) -> ClientSize {
    ClientSize {
        width: layout.content_margin * 2
            + scale_dimension_for_dpi(MIN_TREE_PANEL_WIDTH, dpi)
            + layout.panel_gap
            + scale_dimension_for_dpi(MIN_COMMAND_TABS_PANEL_WIDTH, dpi),
        height: layout.content_top_gap
            + scale_dimension_for_dpi(MIN_MAIN_PANEL_HEIGHT, dpi)
            + layout.content_margin,
    }
}

fn non_client_size(hwnd: HWND) -> (i32, i32) {
    let mut window = RECT::default();
    let mut client = RECT::default();
    // SAFETY: both RECT values point to writable storage for this window.
    let has_window = unsafe { GetWindowRect(hwnd, &mut window) } != 0;
    // SAFETY: client points to writable storage for this window.
    let has_client = unsafe { GetClientRect(hwnd, &mut client) } != 0;
    if !has_window || !has_client {
        return (0, 0);
    }

    let window_width = window.right - window.left;
    let window_height = window.bottom - window.top;
    let client_width = client.right - client.left;
    let client_height = client.bottom - client.top;

    (
        (window_width - client_width).max(0),
        (window_height - client_height).max(0),
    )
}

fn window_size(hwnd: HWND) -> Option<WindowSize> {
    let mut placement = WINDOWPLACEMENT {
        length: size_of::<WINDOWPLACEMENT>() as u32,
        ..Default::default()
    };
    // SAFETY: placement has its length field initialized as required by GetWindowPlacement.
    if unsafe { GetWindowPlacement(hwnd, &mut placement) } != 0
        && let Some(size) = size_from_rect(placement.rcNormalPosition)
    {
        return Some(size);
    }

    let mut rect = RECT::default();
    // SAFETY: rect points to writable storage and hwnd is expected to be a live top-level window.
    if unsafe { GetWindowRect(hwnd, &mut rect) } == 0 {
        return None;
    }

    size_from_rect(rect)
}

fn size_from_rect(rect: RECT) -> Option<WindowSize> {
    let width = rect.right - rect.left;
    let height = rect.bottom - rect.top;
    if width <= 0 || height <= 0 {
        None
    } else {
        Some(WindowSize { width, height })
    }
}

fn current_view_settings_with_window_layout(
    hwnd: HWND,
    context: &WindowContext,
) -> Option<ViewSettings> {
    let window_size = window_size(hwnd)?;
    Some(view_settings_with_window_layout(
        &context.spec.state.settings().view,
        window_size,
        context.spec.layout.tree_panel_width,
    ))
}

fn view_settings_with_window_layout(
    current: &ViewSettings,
    window_size: WindowSize,
    tree_panel_width: i32,
) -> ViewSettings {
    let mut view = current.clone();
    view.set_window_layout(window_size.width, window_size.height, tree_panel_width);
    view
}

fn persist_window_layout_before_close(hwnd: HWND, context: &mut WindowContext) -> bool {
    let Some(view) = current_view_settings_with_window_layout(hwnd, context) else {
        return true;
    };
    if context.spec.state.settings().view == view {
        return true;
    }

    let previous_state = SettingsRestorePoint::capture(&context.spec.state);
    context.spec.state.set_view_settings(view);
    persist_settings_or_restore(hwnd, &mut context.spec.state, previous_state)
}

fn minimum_window_track_size(hwnd: HWND, context: &WindowContext) -> WindowSize {
    let client = minimum_main_client_size(context.spec.layout, context.dpi);
    let (non_client_width, non_client_height) = non_client_size(hwnd);

    WindowSize {
        width: client.width + non_client_width,
        height: client.height + non_client_height,
    }
}

fn apply_minimum_window_track_size(hwnd: HWND, context: &WindowContext, lparam: LPARAM) {
    if lparam == 0 {
        return;
    }

    let minimum = minimum_window_track_size(hwnd, context);
    // SAFETY: WM_GETMINMAXINFO supplies lparam as a writable MINMAXINFO pointer.
    let info = unsafe { &mut *(lparam as *mut MINMAXINFO) };
    info.ptMinTrackSize.x = info.ptMinTrackSize.x.max(minimum.width);
    info.ptMinTrackSize.y = info.ptMinTrackSize.y.max(minimum.height);
}

fn resize_main_content(hwnd: HWND, context: &mut WindowContext) {
    if !context.content.is_created() {
        return;
    }

    let Ok(client) = main_client_size(hwnd) else {
        return;
    };
    context.spec.layout.tree_panel_width = clamp_tree_panel_width(
        context.spec.layout,
        client,
        context.dpi,
        context.spec.layout.tree_panel_width,
    );
    let layout = context.spec.layout.arrange_main_content(client);

    move_child(context.content.tree_view, layout.tree_panel);
    let command_tab_page = context.content.command_tabs.layout(
        layout,
        context.spec.state.settings().view.font_size,
        context.dpi,
    );
    let page = context.content.command_tabs.page;
    let button_count = context.command_button_controls.len();
    sync_command_button_scroll_area(
        page,
        context,
        layout.command_tabs_panel,
        command_tab_page,
        button_count,
    );
    let scroll_offset = context.command_button_scroll_offset;
    layout_command_button_controls(
        layout.command_tabs_panel,
        command_tab_page,
        &context.command_button_controls,
        context.spec.state.settings().view.font_size,
        context.dpi,
        scroll_offset,
    );
}

fn begin_main_splitter_drag(hwnd: HWND, context: &mut WindowContext, point: POINT) -> bool {
    if !point_hits_main_splitter(hwnd, context, point) {
        return false;
    }

    context.splitter_drag = Some(SplitterDrag);
    set_splitter_cursor();
    // SAFETY: hwnd is the top-level window receiving the mouse down in the splitter gap.
    unsafe {
        SetCapture(hwnd);
    }
    true
}

fn update_main_splitter_drag(hwnd: HWND, context: &mut WindowContext, point: POINT) -> bool {
    if context.splitter_drag.is_none() {
        return false;
    }

    let Ok(client) = main_client_size(hwnd) else {
        return true;
    };
    let tree_panel_width =
        splitter_drag_tree_panel_width(context.spec.layout, client, context.dpi, point);
    if context.spec.layout.tree_panel_width != tree_panel_width {
        context.spec.layout.tree_panel_width = tree_panel_width;
        resize_main_content(hwnd, context);
        // SAFETY: hwnd is the main window owned by this UI thread.
        unsafe {
            invalidate_window(hwnd);
        }
    }
    set_splitter_cursor();
    true
}

fn finish_main_splitter_drag(hwnd: HWND, context: &mut WindowContext, point: POINT) -> bool {
    if context.splitter_drag.is_none() {
        return false;
    }

    update_main_splitter_drag(hwnd, context, point);
    context.splitter_drag = None;
    // SAFETY: The drag path set capture on the main window. Releasing is harmless if capture has
    // already changed.
    unsafe {
        ReleaseCapture();
    }
    true
}

fn cancel_main_splitter_drag(context: &mut WindowContext) {
    context.splitter_drag = None;
}

fn point_hits_main_splitter(hwnd: HWND, context: &WindowContext, point: POINT) -> bool {
    let Ok(layout) = current_main_content_layout(hwnd, context.spec.layout) else {
        return false;
    };

    point_in_rect_spec(point, main_splitter_hit_rect(layout))
}

fn main_splitter_hit_rect(layout: MainContentLayout) -> RectSpec {
    let left = layout.tree_panel.x + layout.tree_panel.width;
    let right = layout.command_tabs_panel.x;
    let top = layout.tree_panel.y.min(layout.command_tabs_panel.y);
    let bottom = (layout.tree_panel.y + layout.tree_panel.height)
        .max(layout.command_tabs_panel.y + layout.command_tabs_panel.height);

    RectSpec {
        x: left,
        y: top,
        width: (right - left).max(0),
        height: (bottom - top).max(0),
    }
}

fn splitter_drag_tree_panel_width(
    layout: LayoutSpec,
    client: ClientSize,
    dpi: u32,
    point: POINT,
) -> i32 {
    clamp_tree_panel_width(
        layout,
        client,
        dpi,
        point.x.saturating_sub(layout.content_margin),
    )
}

fn clamp_tree_panel_width(layout: LayoutSpec, client: ClientSize, dpi: u32, width: i32) -> i32 {
    let minimum_tree = scale_dimension_for_dpi(MIN_TREE_PANEL_WIDTH, dpi);
    let minimum_tabs = scale_dimension_for_dpi(MIN_COMMAND_TABS_PANEL_WIDTH, dpi);
    let maximum_tree = (client.width - layout.content_margin * 2 - layout.panel_gap - minimum_tabs)
        .max(minimum_tree);

    width.clamp(minimum_tree, maximum_tree)
}

fn point_in_rect_spec(point: POINT, rect: RectSpec) -> bool {
    point.x >= rect.x
        && point.x < rect.x + rect.width
        && point.y >= rect.y
        && point.y < rect.y + rect.height
}

fn set_splitter_cursor() {
    // SAFETY: Loading a predefined cursor with a null instance is the documented Win32 pattern.
    let cursor = unsafe { LoadCursorW(null_mut(), IDC_SIZEWE) };
    if !cursor.is_null() {
        // SAFETY: cursor is a system-owned predefined cursor.
        unsafe {
            SetCursor(cursor);
        }
    }
}

fn move_child(hwnd: HWND, rect: RectSpec) {
    // SAFETY: hwnd is a child control created by this module. MoveWindow keeps it in parent
    // client coordinates and repaints immediately.
    unsafe {
        MoveWindow(hwnd, rect.x, rect.y, rect.width, rect.height, 1);
    }
}

fn refresh_tree_view(context: &mut WindowContext) {
    if context.content.tree_view.is_null() {
        return;
    }

    let desired_selection = context.spec.state.selected_workspace_index();
    let tree_view = context.content.tree_view;
    let root_items = context.spec.state.settings().root_tree_items();
    let can_update_existing_items = {
        let settings = context.spec.state.settings();
        tree_view_snapshot_has_same_shape(
            &context.tree_view_snapshot,
            root_items.as_slice(),
            settings.categories.as_slice(),
            settings.workspaces.as_slice(),
        ) && context.category_tree_items.len() == settings.categories.len()
            && context.tree_items.len() == settings.workspaces.len()
            && context.category_tree_items.iter().all(|item| *item != 0)
            && context.tree_items.iter().all(|item| *item != 0)
    };

    if can_update_existing_items {
        let updated_snapshot = {
            let settings = context.spec.state.settings();
            if update_tree_view_item_texts(
                tree_view,
                &context.category_tree_items,
                &context.tree_items,
                settings.categories.as_slice(),
                settings.workspaces.as_slice(),
            ) {
                Some(capture_tree_view_snapshot(
                    root_items.as_slice(),
                    settings.categories.as_slice(),
                    settings.workspaces.as_slice(),
                ))
            } else {
                None
            }
        };
        if let Some(snapshot) = updated_snapshot {
            context.tree_view_snapshot = snapshot;
            restore_tree_view_selection(context, desired_selection);
            return;
        }
    }

    let groups = {
        let settings = context.spec.state.settings();
        workspace_tree_groups(
            settings.workspaces.as_slice(),
            settings.categories.as_slice(),
        )
    };

    // SAFETY: tree_view is a valid TreeView control. Passing TVI_ROOT deletes all items.
    unsafe {
        SendMessageW(tree_view, TVM_DELETEITEM, 0, TVI_ROOT as LPARAM);
    }
    context.tree_items.clear();
    context.category_tree_items.clear();

    let categories = context.spec.state.settings().categories.as_slice();
    let workspaces = context.spec.state.settings().workspaces.as_slice();
    context.category_tree_items = vec![0; categories.len()];
    context.tree_items = vec![0; workspaces.len()];

    for root_item in &root_items {
        match *root_item {
            TreeRootItemRef::Category(category_index) => {
                let Some(category) = categories.get(category_index) else {
                    continue;
                };
                let parent = insert_category_tree_item(tree_view, category_index, category);
                if let Some(slot) = context.category_tree_items.get_mut(category_index) {
                    *slot = parent;
                }
                if parent == 0 {
                    continue;
                }

                let mut inserted_child = false;
                if let Some(workspace_indexes) =
                    groups.category_workspace_indexes.get(category_index)
                {
                    for workspace_index in workspace_indexes {
                        if let Some(workspace) = workspaces.get(*workspace_index) {
                            let item = insert_workspace_tree_item(
                                tree_view,
                                parent,
                                *workspace_index,
                                workspace,
                            );
                            if let Some(slot) = context.tree_items.get_mut(*workspace_index) {
                                *slot = item;
                            }
                            inserted_child |= item != 0;
                        }
                    }
                }

                if inserted_child {
                    expand_tree_item(tree_view, parent);
                }
            }
            TreeRootItemRef::Workspace(index) => {
                let Some(workspace) = workspaces.get(index) else {
                    continue;
                };
                let item = insert_workspace_tree_item(tree_view, TVI_ROOT, index, workspace);
                if let Some(slot) = context.tree_items.get_mut(index) {
                    *slot = item;
                }
            }
        }
    }

    context.tree_view_snapshot =
        capture_tree_view_snapshot(root_items.as_slice(), categories, workspaces);
    restore_tree_view_selection(context, desired_selection);
}

fn restore_tree_view_selection(context: &mut WindowContext, desired_selection: Option<usize>) {
    if let Some(index) = desired_selection {
        if let Some(item) = context
            .tree_items
            .get(index)
            .copied()
            .filter(|item| *item != 0)
        {
            context.spec.state.select_workspace(Some(index));
            // SAFETY: item is an HTREEITEM returned by this TreeView control.
            unsafe {
                SendMessageW(
                    context.content.tree_view,
                    TVM_SELECTITEM,
                    TVGN_CARET as WPARAM,
                    item as LPARAM,
                );
            }
        } else {
            context.spec.state.select_workspace(None);
        }
    }
}

fn update_tree_view_item_texts(
    tree_view: HWND,
    category_items: &[HTREEITEM],
    tree_items: &[HTREEITEM],
    categories: &[Category],
    workspaces: &[Workspace],
) -> bool {
    for (item, category) in category_items.iter().copied().zip(categories) {
        if !set_tree_item_text(tree_view, item, &category.name) {
            return false;
        }
        expand_tree_item(tree_view, item);
    }

    for (item, workspace) in tree_items.iter().copied().zip(workspaces) {
        if !set_tree_item_text(tree_view, item, &workspace.name) {
            return false;
        }
    }

    true
}

fn set_tree_item_text(tree_view: HWND, item: HTREEITEM, text: &str) -> bool {
    if tree_view.is_null() || item == 0 {
        return false;
    }

    let text = wide_null(text);
    let mut tree_item = TVITEMW {
        mask: TVIF_HANDLE | TVIF_TEXT,
        hItem: item,
        pszText: text.as_ptr() as *mut u16,
        ..TVITEMW::default()
    };

    // SAFETY: item belongs to this TreeView, and text remains alive for the duration of the call.
    unsafe {
        SendMessageW(
            tree_view,
            TVM_SETITEMW,
            0,
            &mut tree_item as *mut TVITEMW as LPARAM,
        ) != 0
    }
}

fn tree_view_snapshot_has_same_shape(
    snapshot: &TreeViewSnapshot,
    root_items: &[TreeRootItemRef],
    categories: &[Category],
    workspaces: &[Workspace],
) -> bool {
    snapshot.root_items == root_items
        && snapshot.category_names.len() == categories.len()
        && snapshot.workspace_categories.len() == workspaces.len()
        && snapshot
            .category_names
            .iter()
            .zip(categories)
            .all(|(name, category)| name == &category.name)
        && snapshot
            .workspace_categories
            .iter()
            .zip(workspaces)
            .all(|(category, workspace)| category.as_deref() == workspace.category.as_deref())
}

fn capture_tree_view_snapshot(
    root_items: &[TreeRootItemRef],
    categories: &[Category],
    workspaces: &[Workspace],
) -> TreeViewSnapshot {
    TreeViewSnapshot {
        root_items: root_items.to_vec(),
        category_names: categories
            .iter()
            .map(|category| category.name.clone())
            .collect(),
        workspace_categories: workspaces
            .iter()
            .map(|workspace| workspace.category.clone())
            .collect(),
    }
}

fn workspace_tree_groups(workspaces: &[Workspace], categories: &[Category]) -> WorkspaceTreeGroups {
    let mut category_workspace_indexes = vec![Vec::new(); categories.len()];

    for (workspace_index, workspace) in workspaces.iter().enumerate() {
        if let Some(category_index) = layout_rules::workspace_category_index(workspace, categories)
            && let Some(indexes) = category_workspace_indexes.get_mut(category_index)
        {
            indexes.push(workspace_index);
        }
    }

    WorkspaceTreeGroups {
        category_workspace_indexes,
    }
}

fn insert_category_tree_item(tree_view: HWND, index: usize, category: &Category) -> HTREEITEM {
    let Some(lparam) = category_tree_lparam(index) else {
        return 0;
    };

    let text = wide_null(&category.name);
    let item = TVITEMW {
        mask: TVIF_TEXT | TVIF_PARAM,
        pszText: text.as_ptr() as *mut u16,
        lParam: lparam,
        ..TVITEMW::default()
    };
    let mut insert = TVINSERTSTRUCTW {
        hParent: TVI_ROOT,
        hInsertAfter: TVI_LAST,
        Anonymous: TVINSERTSTRUCTW_0 { item },
    };

    // SAFETY: tree_view is a TreeView control, and insert points to an initialized
    // TVINSERTSTRUCTW whose text buffer remains alive for the duration of the call.
    unsafe {
        SendMessageW(
            tree_view,
            TVM_INSERTITEMW,
            0,
            &mut insert as *mut TVINSERTSTRUCTW as LPARAM,
        ) as HTREEITEM
    }
}

fn insert_workspace_tree_item(
    tree_view: HWND,
    parent: HTREEITEM,
    index: usize,
    workspace: &Workspace,
) -> HTREEITEM {
    let Some(lparam) = workspace_tree_lparam(index) else {
        return 0;
    };

    let text = wide_null(&workspace.name);
    let item = TVITEMW {
        mask: TVIF_TEXT | TVIF_PARAM,
        pszText: text.as_ptr() as *mut u16,
        lParam: lparam,
        ..TVITEMW::default()
    };
    let mut insert = TVINSERTSTRUCTW {
        hParent: parent,
        hInsertAfter: TVI_LAST,
        Anonymous: TVINSERTSTRUCTW_0 { item },
    };

    // SAFETY: tree_view is a TreeView control, and insert points to an initialized
    // TVINSERTSTRUCTW whose text buffer remains alive for the duration of the call.
    unsafe {
        SendMessageW(
            tree_view,
            TVM_INSERTITEMW,
            0,
            &mut insert as *mut TVINSERTSTRUCTW as LPARAM,
        ) as HTREEITEM
    }
}

fn expand_tree_item(tree_view: HWND, item: HTREEITEM) {
    if tree_view.is_null() || item == 0 {
        return;
    }

    // SAFETY: item was returned by this TreeView control. TVM_EXPAND only changes visual state.
    unsafe {
        SendMessageW(tree_view, TVM_EXPAND, TVE_EXPAND as WPARAM, item as LPARAM);
    }
}

fn selected_tree_item(tree_view: HWND) -> HTREEITEM {
    if tree_view.is_null() {
        return 0;
    }

    // SAFETY: tree_view is a TreeView control. TVGN_CARET asks for the current selected item.
    unsafe { SendMessageW(tree_view, TVM_GETNEXTITEM, TVGN_CARET as WPARAM, 0) as HTREEITEM }
}

fn selected_tree_node(context: &WindowContext) -> Option<TreeNodeSelection> {
    let tree_view = context.content.tree_view;
    let selected_item = selected_tree_item(tree_view);
    if let Some(selection) = tree_node_from_tree_item(tree_view, selected_item) {
        return Some(selection);
    }

    context
        .spec
        .state
        .selected_workspace_index()
        .map(TreeNodeSelection::Workspace)
}

fn tree_node_at_current_message_position(context: &WindowContext) -> Option<TreeNodeSelection> {
    if context.content.tree_view.is_null() {
        return None;
    }

    // SAFETY: GetMessagePos returns the screen coordinates for the current message.
    let mut point = point_from_lparam(unsafe { GetMessagePos() as LPARAM });
    // SAFETY: point contains screen coordinates and tree_view is the target client window.
    let converted = unsafe { ScreenToClient(context.content.tree_view, &mut point) };
    if converted == 0 {
        return None;
    }

    tree_node_at_tree_point(context, point)
}

fn tree_node_at_tree_point(context: &WindowContext, point: POINT) -> Option<TreeNodeSelection> {
    let item = tree_item_at_tree_point(context.content.tree_view, point)?;
    tree_node_from_tree_item(context.content.tree_view, item)
}

fn tree_node_from_tree_item(tree_view: HWND, item: HTREEITEM) -> Option<TreeNodeSelection> {
    category_index_from_tree_item(tree_view, item)
        .map(TreeNodeSelection::Category)
        .or_else(|| {
            workspace_index_from_tree_item(tree_view, item).map(TreeNodeSelection::Workspace)
        })
}

fn command_button_click_ignored_for_tree_selection(selection: Option<TreeNodeSelection>) -> bool {
    matches!(selection, Some(TreeNodeSelection::Category(_)))
}

fn select_tree_item(tree_view: HWND, item: HTREEITEM) {
    if tree_view.is_null() || item == 0 {
        return;
    }

    // SAFETY: item is expected to be an HTREEITEM returned by this TreeView control.
    unsafe {
        SendMessageW(
            tree_view,
            TVM_SELECTITEM,
            TVGN_CARET as WPARAM,
            item as LPARAM,
        );
    }
}

fn select_category_tree_item(context: &WindowContext, index: usize) {
    let Some(item) = context
        .category_tree_items
        .get(index)
        .copied()
        .filter(|item| *item != 0)
    else {
        return;
    };

    select_tree_item(context.content.tree_view, item);
}

fn workspace_tree_lparam(index: usize) -> Option<LPARAM> {
    isize::try_from(index).ok()
}

fn category_tree_lparam(index: usize) -> Option<LPARAM> {
    let index = isize::try_from(index).ok()?;
    index.checked_add(1)?.checked_neg()
}

fn workspace_index_from_tree_lparam(lparam: LPARAM) -> Option<usize> {
    if lparam < 0 {
        None
    } else {
        usize::try_from(lparam).ok()
    }
}

fn category_index_from_tree_lparam(lparam: LPARAM) -> Option<usize> {
    if lparam >= 0 {
        None
    } else {
        lparam
            .checked_neg()
            .and_then(|value| value.checked_sub(1))
            .and_then(|value| usize::try_from(value).ok())
    }
}

fn refresh_command_tab_selector(context: &mut WindowContext) {
    if context.content.command_tabs.selector.is_null() {
        return;
    }

    let desired_selection = context.spec.state.selected_command_tab_index();
    let selector = context.content.command_tabs.selector;

    // SAFETY: selector is a ComboBox created by this module. Resetting and repopulating keeps the
    // visible dropdown aligned with the domain state after add/delete/rename/reorder operations.
    unsafe {
        SendMessageW(selector, CB_RESETCONTENT, 0, 0);
    }

    for tab in &context.spec.state.settings().command_tabs {
        add_command_tab_selector_item(selector, tab);
    }

    if let Some(index) =
        desired_selection.filter(|index| *index < context.spec.state.settings().command_tabs.len())
    {
        context.spec.state.select_command_tab(Some(index));
        // SAFETY: index is within the current ComboBox item range.
        unsafe {
            SendMessageW(selector, CB_SETCURSEL, index as WPARAM, 0);
        }
    } else {
        context.spec.state.select_command_tab(None);
        // SAFETY: selector is a ComboBox; -1 clears selection.
        unsafe {
            SendMessageW(selector, CB_SETCURSEL, usize::MAX, 0);
        }
    }

    // SAFETY: selector is a live child control. Disabling the dropdown when empty avoids showing a
    // selectable-but-blank control while retaining the layout.
    unsafe {
        EnableWindow(
            selector,
            if context.spec.state.settings().command_tabs.is_empty() {
                0
            } else {
                1
            },
        );
    }
}

fn add_command_tab_selector_item(selector: HWND, tab: &CommandTab) {
    let text = wide_null(&tab.name);

    // SAFETY: selector is a ComboBox and text remains alive for the duration of the send.
    unsafe {
        SendMessageW(selector, CB_ADDSTRING, 0, text.as_ptr() as LPARAM);
    }
}

fn refresh_command_buttons(hwnd: HWND, context: &mut WindowContext) {
    cancel_command_button_drag(context);
    let context_ptr = context as *mut WindowContext;

    let Some(button_count) = context
        .spec
        .state
        .selected_command_tab()
        .map(|tab| tab.buttons.len())
    else {
        destroy_command_button_controls(context);
        context.spec.state.select_command_button(None);
        reset_command_button_scroll_area(context);
        update_commands_menu_state(hwnd, context);
        return;
    };

    let desired_selection = context.spec.state.selected_command_button_index();
    let Ok(layout) = current_main_content_layout(hwnd, context.spec.layout) else {
        return;
    };
    let font_size = context.spec.state.settings().view.font_size;
    let button_panel =
        context
            .content
            .command_tabs
            .page_rect(layout.command_tabs_panel, font_size, context.dpi);
    let page = context.content.command_tabs.page;
    sync_command_button_scroll_area(
        page,
        context,
        layout.command_tabs_panel,
        button_panel,
        button_count,
    );
    let scroll_offset = context.command_button_scroll_offset;
    if command_button_controls_can_be_updated(&context.command_button_controls, button_count)
        && command_button_tooltips_can_be_updated(context, button_count)
    {
        let updated_existing_controls =
            context
                .spec
                .state
                .selected_command_tab()
                .is_some_and(|tab| {
                    update_existing_command_button_controls(
                        &context.command_button_controls,
                        CommandButtonTooltipUpdate {
                            handle: context.command_button_tooltip,
                            texts: context.command_button_tooltip_texts.as_mut_slice(),
                        },
                        &tab.buttons,
                        CommandButtonControlLayout {
                            panel: layout.command_tabs_panel,
                            parent: button_panel,
                            font_size,
                            dpi: context.dpi,
                            scroll_offset,
                        },
                    )
                });
        if updated_existing_controls {
            finish_command_button_refresh(hwnd, context, desired_selection);
            return;
        }
    }

    destroy_command_button_controls(context);

    let Some(tab) = context.spec.state.selected_command_tab() else {
        context.spec.state.select_command_button(None);
        reset_command_button_scroll_area(context);
        update_commands_menu_state(hwnd, context);
        return;
    };
    let rects = command_button_rects_in_parent(
        layout.command_tabs_panel,
        button_panel,
        tab.buttons.len(),
        font_size,
        context.dpi,
        scroll_offset,
    );
    let button_class = wide_null("BUTTON");
    let style = command_button_window_style();

    for ((index, button), rect) in tab.buttons.iter().enumerate().zip(rects) {
        let control_id = match command_button_control_id(index) {
            Some(control_id) => control_id,
            None => break,
        };

        match create_child_window_with_id(
            context.content.command_tabs.page,
            context.instance,
            button_class.as_ptr(),
            &button.button_name,
            style,
            rect,
            control_id,
            "CreateWindowExW command button",
        ) {
            Ok(handle) => {
                if let Err(error) = install_command_button_subclass(handle, context_ptr) {
                    // SAFETY: handle is a child button created just above and is not stored in
                    // the context if subclass installation fails.
                    unsafe {
                        DestroyWindow(handle);
                    }
                    show_error_message(
                        hwnd,
                        context_tr(context, "명령", "Command"),
                        &error.to_string(),
                    );
                    break;
                }
                if let Err(error) = add_command_button_tooltip(
                    context.command_button_tooltip,
                    &mut context.command_button_tooltip_texts,
                    handle,
                    &button.button_name,
                ) {
                    // SAFETY: handle is a child button created just above and is not stored in
                    // the context if tooltip registration fails.
                    unsafe {
                        RemoveWindowSubclass(
                            handle,
                            Some(command_button_subclass_proc),
                            COMMAND_BUTTON_SUBCLASS_ID,
                        );
                        DestroyWindow(handle);
                    }
                    show_error_message(
                        hwnd,
                        context_tr(context, "명령", "Command"),
                        &error.to_string(),
                    );
                    break;
                }
                context.command_button_controls.push(handle);
            }
            Err(error) => {
                show_error_message(
                    hwnd,
                    context_tr(context, "명령", "Command"),
                    &error.to_string(),
                );
                break;
            }
        }
    }

    finish_command_button_refresh(hwnd, context, desired_selection);
}

fn command_button_controls_can_be_updated(buttons: &[HWND], expected_len: usize) -> bool {
    buttons.len() == expected_len && buttons.iter().all(|button| !button.is_null())
}

fn command_button_tooltips_can_be_updated(context: &WindowContext, expected_len: usize) -> bool {
    !context.command_button_tooltip.is_null()
        && context.command_button_tooltip_texts.len() == expected_len
}

struct CommandButtonControlLayout {
    panel: RectSpec,
    parent: RectSpec,
    font_size: u16,
    dpi: u32,
    scroll_offset: i32,
}

fn update_existing_command_button_controls(
    controls: &[HWND],
    tooltip: CommandButtonTooltipUpdate<'_>,
    buttons: &[CommandButton],
    layout: CommandButtonControlLayout,
) -> bool {
    if controls.len() != buttons.len() || tooltip.texts.len() != buttons.len() {
        return false;
    }

    let rects = command_button_rects_in_parent(
        layout.panel,
        layout.parent,
        buttons.len(),
        layout.font_size,
        layout.dpi,
        layout.scroll_offset,
    );
    for (((control, tooltip_text), button), rect) in controls
        .iter()
        .zip(tooltip.texts.iter_mut())
        .zip(buttons)
        .zip(rects)
    {
        if !set_child_window_text(*control, &button.button_name) {
            return false;
        }
        update_command_button_tooltip_text(
            tooltip.handle,
            *control,
            tooltip_text,
            &button.button_name,
        );
        move_child(*control, rect);
    }

    true
}

struct CommandButtonTooltipUpdate<'a> {
    handle: HWND,
    texts: &'a mut [Vec<u16>],
}

fn set_child_window_text(hwnd: HWND, text: &str) -> bool {
    if hwnd.is_null() {
        return false;
    }

    let text = wide_null(text);
    // SAFETY: hwnd is a live child window owned by this module, and text is null-terminated.
    unsafe { SetWindowTextW(hwnd, text.as_ptr()) != 0 }
}

fn add_command_button_tooltip(
    tooltip: HWND,
    tooltip_texts: &mut Vec<Vec<u16>>,
    button: HWND,
    text: &str,
) -> AppResult<()> {
    if tooltip.is_null() || button.is_null() {
        return Err(last_error("TTM_ADDTOOLW command button tooltip"));
    }

    tooltip_texts.push(wide_null(text));
    let Some(tooltip_text) = tooltip_texts.last_mut() else {
        return Err(last_error("TTM_ADDTOOLW command button tooltip"));
    };
    let mut tool = command_button_tooltip_info(button, tooltip_text.as_mut_ptr());

    // SAFETY: tooltip is a tooltip control, tool identifies a live command button HWND as the
    // tool, and tooltip_texts owns the pointed-to text until the tool is updated or deleted.
    let added = unsafe {
        SendMessageW(
            tooltip,
            TTM_ADDTOOLW,
            0,
            &mut tool as *mut TTTOOLINFOW as LPARAM,
        )
    };

    if added == 0 {
        tooltip_texts.pop();
        Err(last_error("TTM_ADDTOOLW command button tooltip"))
    } else {
        Ok(())
    }
}

fn update_command_button_tooltip_text(
    tooltip: HWND,
    button: HWND,
    tooltip_text: &mut Vec<u16>,
    text: &str,
) {
    if tooltip.is_null() || button.is_null() {
        return;
    }

    *tooltip_text = wide_null(text);
    let mut tool = command_button_tooltip_info(button, tooltip_text.as_mut_ptr());

    // SAFETY: tooltip is a tooltip control, tool identifies an existing command-button tool, and
    // tooltip_text owns the pointed-to string after this synchronous update message returns.
    unsafe {
        SendMessageW(
            tooltip,
            TTM_UPDATETIPTEXTW,
            0,
            &mut tool as *mut TTTOOLINFOW as LPARAM,
        );
    }
}

unsafe fn remove_command_button_tooltip(tooltip: HWND, button: HWND) {
    if tooltip.is_null() || button.is_null() {
        return;
    }

    let mut tool = command_button_tooltip_info(button, null_mut());

    // SAFETY: tooltip is a tooltip control and tool uses the same hwnd/uId pair that was used
    // when the command button was registered as a tooltip tool.
    unsafe {
        SendMessageW(
            tooltip,
            TTM_DELTOOLW,
            0,
            &mut tool as *mut TTTOOLINFOW as LPARAM,
        );
    }
}

fn command_button_tooltip_info(button: HWND, text: *mut u16) -> TTTOOLINFOW {
    TTTOOLINFOW {
        cbSize: size_of::<TTTOOLINFOW>() as u32,
        uFlags: TTF_IDISHWND | TTF_SUBCLASS,
        hwnd: command_button_parent(button),
        uId: button as usize,
        lpszText: text,
        ..TTTOOLINFOW::default()
    }
}

fn finish_command_button_refresh(
    hwnd: HWND,
    context: &mut WindowContext,
    desired_selection: Option<usize>,
) {
    apply_font_to_handles(&context.command_button_controls, context.ui_font.handle());
    context.spec.state.select_command_button(desired_selection);
    apply_window_theme(hwnd, context);
    update_commands_menu_state(hwnd, context);
}

fn destroy_command_button_controls(context: &mut WindowContext) {
    cancel_command_button_drag(context);

    for button in context.command_button_controls.drain(..) {
        if button.is_null() {
            continue;
        }

        // SAFETY: Each handle was created by refresh_command_buttons as a child of the main
        // window and is owned by this context. The subclass was installed by this module.
        unsafe {
            remove_command_button_tooltip(context.command_button_tooltip, button);
            RemoveWindowSubclass(
                button,
                Some(command_button_subclass_proc),
                COMMAND_BUTTON_SUBCLASS_ID,
            );
            DestroyWindow(button);
        }
    }
    context.command_button_tooltip_texts.clear();
}

fn install_command_button_subclass(button: HWND, context: *mut WindowContext) -> AppResult<()> {
    if button.is_null() || context.is_null() {
        return Err(last_error("SetWindowSubclass command button"));
    }

    // SAFETY: button is a live BUTTON child control. context is the WindowContext that owns the
    // button controls and remains alive until the main window is destroyed.
    let ok = unsafe {
        SetWindowSubclass(
            button,
            Some(command_button_subclass_proc),
            COMMAND_BUTTON_SUBCLASS_ID,
            context as usize,
        )
    };

    if ok == 0 {
        Err(last_error("SetWindowSubclass command button"))
    } else {
        Ok(())
    }
}

fn command_tab_page_window_style() -> u32 {
    WS_CHILD | WS_VISIBLE | WS_CLIPCHILDREN | WS_VSCROLL
}

fn command_tab_selector_rect(panel: RectSpec, font_size: u16, dpi: u32) -> RectSpec {
    let control_height = command_tab_selector_control_height(font_size, dpi);
    let dropdown_height = (control_height
        + scale_dimension_for_dpi(COMMAND_TAB_SELECTOR_DROPDOWN_EXTRA_HEIGHT, dpi))
    .min(panel.height.max(control_height))
    .max(control_height);

    rect_spec_from_edges(
        panel.x,
        panel.y,
        panel.x + panel.width,
        panel.y + dropdown_height,
    )
}

fn command_tab_page_rect(panel: RectSpec, font_size: u16, dpi: u32) -> RectSpec {
    let top = panel.y
        + command_tab_selector_control_height(font_size, dpi)
        + command_tab_selector_gap(dpi);

    rect_spec_from_edges(panel.x, top, panel.x + panel.width, panel.y + panel.height)
}

fn command_tab_selector_control_height(font_size: u16, dpi: u32) -> i32 {
    let font_size = normalize_ui_font_size(font_size);
    let extra = i32::from(font_size.saturating_sub(DEFAULT_FONT_SIZE));
    scale_dimension_for_dpi(COMMAND_TAB_SELECTOR_HEIGHT + extra * 2, dpi).max(1)
}

fn command_tab_selector_gap(dpi: u32) -> i32 {
    scale_dimension_for_dpi(COMMAND_TAB_SELECTOR_PAGE_GAP, dpi).max(0)
}

fn rect_spec_from_edges(left: i32, top: i32, right: i32, bottom: i32) -> RectSpec {
    RectSpec {
        x: left,
        y: top,
        width: (right - left).max(0),
        height: (bottom - top).max(0),
    }
}

fn sync_command_button_scroll_area(
    page: HWND,
    context: &mut WindowContext,
    panel: RectSpec,
    parent: RectSpec,
    button_count: usize,
) {
    let font_size = context.spec.state.settings().view.font_size;
    let line_step = command_button_scroll_line_step(font_size, context.dpi);
    let raw_max_offset =
        command_button_max_scroll_offset(panel, parent, button_count, font_size, context.dpi);
    let max_offset = command_button_effective_max_scroll_offset(raw_max_offset, line_step);
    context.command_button_scroll_offset = command_button_aligned_scroll_offset(
        context.command_button_scroll_offset,
        max_offset,
        line_step,
    );
    set_command_tab_page_scroll_info(
        page,
        command_button_scrollbar_content_height(parent.height, max_offset),
        parent.height,
        context.command_button_scroll_offset,
    );
}

fn reset_command_button_scroll_area(context: &mut WindowContext) {
    context.command_button_scroll_offset = 0;
    set_command_tab_page_scroll_info(context.content.command_tabs.page, 0, 0, 0);
}

fn set_command_tab_page_scroll_info(
    page: HWND,
    content_height: i32,
    viewport_height: i32,
    scroll_offset: i32,
) {
    if page.is_null() {
        return;
    }

    let viewport_height = viewport_height.max(0);
    let content_height = content_height.max(viewport_height).max(1);
    let scroll_offset = scroll_offset.clamp(0, max_scroll_offset(content_height, viewport_height));
    let info = SCROLLINFO {
        cbSize: size_of::<SCROLLINFO>() as u32,
        fMask: SIF_RANGE | SIF_PAGE | SIF_POS,
        nMin: 0,
        nMax: content_height.saturating_sub(1),
        nPage: viewport_height as u32,
        nPos: scroll_offset,
        nTrackPos: 0,
    };

    // SAFETY: page is the command-tab page window, and info is initialized according to
    // SetScrollInfo's contract.
    unsafe {
        SetScrollInfo(page, SB_VERT, &info, 1);
        ShowScrollBar(
            page,
            SB_VERT,
            if max_scroll_offset(content_height, viewport_height) > 0 {
                1
            } else {
                0
            },
        );
    }
}

fn layout_command_button_controls(
    panel: RectSpec,
    parent: RectSpec,
    buttons: &[HWND],
    font_size: u16,
    dpi: u32,
    scroll_offset: i32,
) {
    let rects =
        command_button_rects_in_parent(panel, parent, buttons.len(), font_size, dpi, scroll_offset);
    for (button, rect) in buttons.iter().zip(rects) {
        move_child(*button, rect);
    }
}

fn command_button_window_style() -> u32 {
    WS_CHILD | WS_VISIBLE | WS_TABSTOP | WS_CLIPSIBLINGS | BS_PUSHBUTTON as u32 | BS_LEFT as u32
}

fn command_button_rects_in_parent(
    panel: RectSpec,
    parent: RectSpec,
    count: usize,
    font_size: u16,
    dpi: u32,
    scroll_offset: i32,
) -> CommandButtonRects {
    let mut rects = command_button_rects(panel, count, font_size, dpi);
    rects.left -= parent.x;
    rects.top -= parent.y + scroll_offset.max(0);
    rects
}

fn command_button_scroll_content_height(
    panel: RectSpec,
    parent: RectSpec,
    count: usize,
    font_size: u16,
    dpi: u32,
) -> i32 {
    command_button_rects_in_parent(panel, parent, count, font_size, dpi, 0)
        .last()
        .map(|rect| rect.y + rect.height)
        .unwrap_or(0)
}

fn command_button_max_scroll_offset(
    panel: RectSpec,
    parent: RectSpec,
    count: usize,
    font_size: u16,
    dpi: u32,
) -> i32 {
    max_scroll_offset(
        command_button_scroll_content_height(panel, parent, count, font_size, dpi),
        parent.height,
    )
}

fn command_button_effective_max_scroll_offset(required_max_offset: i32, line_step: i32) -> i32 {
    let required_max_offset = required_max_offset.max(0);
    if required_max_offset == 0 {
        return 0;
    }

    let line_step = line_step.max(1);
    let remainder = required_max_offset % line_step;
    if remainder == 0 {
        required_max_offset
    } else {
        required_max_offset.saturating_add(line_step - remainder)
    }
}

fn command_button_aligned_scroll_offset(target: i32, max_offset: i32, line_step: i32) -> i32 {
    let max_offset = max_offset.max(0);
    if max_offset == 0 {
        return 0;
    }

    let target = target.clamp(0, max_offset);
    let line_step = line_step.max(1);
    if max_offset < line_step {
        return target;
    }

    let lower = target / line_step * line_step;
    let upper = lower.saturating_add(line_step).min(max_offset);
    if target - lower <= upper - target {
        lower
    } else {
        upper
    }
}

fn command_button_scrollbar_content_height(viewport_height: i32, max_offset: i32) -> i32 {
    viewport_height
        .max(0)
        .saturating_add(max_offset.max(0))
        .max(1)
}

fn max_scroll_offset(content_height: i32, viewport_height: i32) -> i32 {
    content_height.saturating_sub(viewport_height.max(0)).max(0)
}

fn command_button_scroll_line_step(font_size: u16, dpi: u32) -> i32 {
    let font_size = normalize_ui_font_size(font_size);
    let extra = i32::from(font_size.saturating_sub(DEFAULT_FONT_SIZE));
    let button_height = scale_dimension_for_dpi(30 + extra * 2, dpi);
    let gap = scale_dimension_for_dpi(8 + extra.min(4), dpi);
    (button_height + gap).max(1)
}

fn handle_command_tab_page_vscroll(
    page: HWND,
    context: &mut WindowContext,
    wparam: WPARAM,
) -> bool {
    let Some(metrics) = command_button_scroll_metrics(page, context) else {
        return false;
    };
    if metrics.max_offset <= 0 {
        return false;
    }

    let command = low_word(wparam) as i32;
    let current = context.command_button_scroll_offset;
    let target = match command {
        SB_LINEUP => current - metrics.line_step,
        SB_LINEDOWN => current + metrics.line_step,
        SB_PAGEUP => current - metrics.page_step,
        SB_PAGEDOWN => current + metrics.page_step,
        SB_THUMBPOSITION | SB_THUMBTRACK => {
            command_tab_page_scroll_track_position(page).unwrap_or(current)
        }
        SB_TOP => 0,
        SB_BOTTOM => metrics.max_offset,
        _ => return false,
    };

    scroll_command_tab_page_to(page, context, metrics, target)
}

fn handle_command_tab_page_mouse_wheel(
    page: HWND,
    context: &mut WindowContext,
    wparam: WPARAM,
) -> bool {
    let wheel_delta = signed_high_word_from_wparam(wparam);
    if wheel_delta == 0 {
        return false;
    }

    let Some(metrics) = command_button_scroll_metrics(page, context) else {
        return false;
    };
    if metrics.max_offset <= 0 {
        return false;
    }

    let notches = (wheel_delta.abs() + WHEEL_DELTA - 1) / WHEEL_DELTA;
    let direction = if wheel_delta > 0 { -1 } else { 1 };
    let target =
        context.command_button_scroll_offset + direction * metrics.line_step * notches.max(1) * 3;
    scroll_command_tab_page_to(page, context, metrics, target)
}

fn scroll_command_tab_page_to(
    page: HWND,
    context: &mut WindowContext,
    metrics: CommandButtonScrollMetrics,
    target: i32,
) -> bool {
    let new_offset =
        command_button_aligned_scroll_offset(target, metrics.max_offset, metrics.line_step);
    if new_offset == context.command_button_scroll_offset {
        set_command_tab_page_scroll_info(
            page,
            metrics.content_height,
            metrics.parent.height,
            new_offset,
        );
        return true;
    }

    context.command_button_scroll_offset = new_offset;
    set_command_tab_page_scroll_info(
        page,
        metrics.content_height,
        metrics.parent.height,
        new_offset,
    );
    layout_command_button_controls(
        metrics.panel,
        metrics.parent,
        &context.command_button_controls,
        context.spec.state.settings().view.font_size,
        context.dpi,
        new_offset,
    );

    // SAFETY: page is the command-tab page window; invalidating it repaints the exposed area after
    // children are moved.
    unsafe {
        invalidate_window(page);
    }
    true
}

#[derive(Clone, Copy)]
struct CommandButtonScrollMetrics {
    panel: RectSpec,
    parent: RectSpec,
    content_height: i32,
    max_offset: i32,
    line_step: i32,
    page_step: i32,
}

fn command_button_scroll_metrics(
    page: HWND,
    context: &WindowContext,
) -> Option<CommandButtonScrollMetrics> {
    if page.is_null() || page != context.content.command_tabs.page {
        return None;
    }

    // SAFETY: the command-tab page is a direct child of the main window.
    let owner = unsafe { GetParent(page) };
    if owner.is_null() {
        return None;
    }

    let layout = current_main_content_layout(owner, context.spec.layout).ok()?;
    let font_size = context.spec.state.settings().view.font_size;
    let parent =
        context
            .content
            .command_tabs
            .page_rect(layout.command_tabs_panel, font_size, context.dpi);
    let button_count = context
        .spec
        .state
        .selected_command_tab()
        .map(|tab| tab.buttons.len())
        .unwrap_or(0);
    let line_step = command_button_scroll_line_step(font_size, context.dpi);
    let raw_max_offset = command_button_max_scroll_offset(
        layout.command_tabs_panel,
        parent,
        button_count,
        font_size,
        context.dpi,
    );
    let max_offset = command_button_effective_max_scroll_offset(raw_max_offset, line_step);
    let page_step = (parent.height - line_step).max(line_step);

    Some(CommandButtonScrollMetrics {
        panel: layout.command_tabs_panel,
        parent,
        content_height: command_button_scrollbar_content_height(parent.height, max_offset),
        max_offset,
        line_step,
        page_step,
    })
}

fn command_tab_page_scroll_track_position(page: HWND) -> Option<i32> {
    if page.is_null() {
        return None;
    }

    let mut info = SCROLLINFO {
        cbSize: size_of::<SCROLLINFO>() as u32,
        fMask: SIF_RANGE | SIF_PAGE | SIF_POS | SIF_TRACKPOS,
        ..SCROLLINFO::default()
    };
    // SAFETY: page is a live window with a vertical scrollbar and info is writable storage.
    let ok = unsafe { GetScrollInfo(page, SB_VERT, &mut info) };
    (ok != 0).then_some(info.nTrackPos)
}

fn command_button_rects(
    panel: RectSpec,
    count: usize,
    font_size: u16,
    dpi: u32,
) -> CommandButtonRects {
    let font_size = normalize_ui_font_size(font_size);
    let extra = i32::from(font_size.saturating_sub(DEFAULT_FONT_SIZE));
    let preferred_button_width =
        scale_dimension_for_dpi(COMMAND_BUTTON_PREFERRED_WIDTH + extra * 12, dpi);
    let minimum_button_width = scale_dimension_for_dpi(MIN_COMMAND_BUTTON_WIDTH, dpi).max(1);
    let button_height = scale_dimension_for_dpi(30 + extra * 2, dpi);
    let gap = scale_dimension_for_dpi(8 + extra.min(4), dpi);
    let horizontal_padding = scale_dimension_for_dpi(COMMAND_BUTTON_HORIZONTAL_PADDING, dpi);
    let left = panel.x + horizontal_padding;
    let top = panel.y + scale_dimension_for_dpi(34 + extra * 2, dpi);
    let usable_width = (panel.width - horizontal_padding * 2)
        .max(minimum_button_width)
        .max(1);
    let button_width = preferred_button_width.min(usable_width);
    let columns = ((usable_width + gap) / (button_width + gap)).max(1) as usize;

    CommandButtonRects {
        count,
        next_index: 0,
        columns,
        left,
        top,
        button_width,
        button_height,
        gap,
        usable_width,
    }
}

struct CommandButtonRects {
    count: usize,
    next_index: usize,
    columns: usize,
    left: i32,
    top: i32,
    button_width: i32,
    button_height: i32,
    gap: i32,
    usable_width: i32,
}

impl CommandButtonRects {
    fn index_at_point(&self, point: POINT) -> Option<usize> {
        let width = i64::from(self.button_width.min(self.usable_width));
        let height = i64::from(self.button_height);
        let column_stride = i64::from(self.button_width) + i64::from(self.gap);
        let row_stride = i64::from(self.button_height) + i64::from(self.gap);
        let columns = i64::try_from(self.columns).ok()?;
        if width <= 0 || height <= 0 || column_stride <= 0 || row_stride <= 0 || columns <= 0 {
            return None;
        }

        let dx = i64::from(point.x) - i64::from(self.left);
        let dy = i64::from(point.y) - i64::from(self.top);
        if dx < 0 || dy < 0 {
            return None;
        }

        let column = dx / column_stride;
        let row = dy / row_stride;
        if column >= columns || dx % column_stride >= width || dy % row_stride >= height {
            return None;
        }

        let index = row.checked_mul(columns)?.checked_add(column)?;
        usize::try_from(index)
            .ok()
            .filter(|index| *index < self.count)
    }
}

impl Iterator for CommandButtonRects {
    type Item = RectSpec;

    fn next(&mut self) -> Option<Self::Item> {
        if self.next_index >= self.count {
            return None;
        }

        let index = self.next_index;
        self.next_index += 1;

        let column = (index % self.columns) as i32;
        let row = (index / self.columns) as i32;
        Some(RectSpec {
            x: self.left + column * (self.button_width + self.gap),
            y: self.top + row * (self.button_height + self.gap),
            width: self.button_width.min(self.usable_width),
            height: self.button_height,
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.count.saturating_sub(self.next_index);
        (remaining, Some(remaining))
    }
}

impl ExactSizeIterator for CommandButtonRects {}

fn command_button_control_id(index: usize) -> Option<i32> {
    let index = i32::try_from(index).ok()?;
    COMMAND_BUTTON_CONTROL_ID_BASE.checked_add(index)
}

fn command_button_index_from_control_id(control_id: u32) -> Option<usize> {
    let control_id = i32::try_from(control_id).ok()?;
    if control_id < COMMAND_BUTTON_CONTROL_ID_BASE {
        return None;
    }

    usize::try_from(control_id - COMMAND_BUTTON_CONTROL_ID_BASE).ok()
}

fn install_tree_view_subclass(tree_view: HWND, context: *mut WindowContext) -> AppResult<()> {
    if tree_view.is_null() || context.is_null() {
        return Err(last_error("SetWindowSubclass tree view"));
    }

    // SAFETY: tree_view is a live TreeView child control. context owns the TreeView and remains
    // alive until the main window is destroyed.
    let ok = unsafe {
        SetWindowSubclass(
            tree_view,
            Some(tree_view_subclass_proc),
            TREE_VIEW_SUBCLASS_ID,
            context as usize,
        )
    };

    if ok == 0 {
        Err(last_error("SetWindowSubclass tree view"))
    } else {
        Ok(())
    }
}

fn remove_tree_view_subclass(context: &WindowContext) {
    if context.content.tree_view.is_null() {
        return;
    }

    // SAFETY: The subclass was installed by this module on this TreeView handle.
    unsafe {
        RemoveWindowSubclass(
            context.content.tree_view,
            Some(tree_view_subclass_proc),
            TREE_VIEW_SUBCLASS_ID,
        );
    }
}

fn command_for_view_theme(theme: ViewTheme) -> u32 {
    match theme {
        ViewTheme::System => MENU_THEME_SYSTEM_ID,
        ViewTheme::Light => MENU_THEME_LIGHT_ID,
        ViewTheme::ClassicDark => MENU_THEME_CLASSIC_DARK_ID,
        ViewTheme::SepiaTeal => MENU_THEME_SEPIA_TEAL_ID,
        ViewTheme::Graphite => MENU_THEME_GRAPHITE_ID,
        ViewTheme::Forest => MENU_THEME_FOREST_ID,
        ViewTheme::SteelBlue => MENU_THEME_STEEL_BLUE_ID,
    }
}

fn view_theme_for_command(command_id: u32) -> Option<ViewTheme> {
    match command_id {
        MENU_THEME_SYSTEM_ID => Some(ViewTheme::System),
        MENU_THEME_LIGHT_ID => Some(ViewTheme::Light),
        MENU_THEME_CLASSIC_DARK_ID => Some(ViewTheme::ClassicDark),
        MENU_THEME_SEPIA_TEAL_ID => Some(ViewTheme::SepiaTeal),
        MENU_THEME_GRAPHITE_ID => Some(ViewTheme::Graphite),
        MENU_THEME_FOREST_ID => Some(ViewTheme::Forest),
        MENU_THEME_STEEL_BLUE_ID => Some(ViewTheme::SteelBlue),
        _ => None,
    }
}

fn command_for_ui_language(language: UiLanguage) -> u32 {
    match language {
        UiLanguage::Korean => MENU_UI_LANGUAGE_KOREAN_ID,
        UiLanguage::English => MENU_UI_LANGUAGE_ENGLISH_ID,
    }
}

fn ui_language_for_command(command_id: u32) -> Option<UiLanguage> {
    match command_id {
        MENU_UI_LANGUAGE_KOREAN_ID => Some(UiLanguage::Korean),
        MENU_UI_LANGUAGE_ENGLISH_ID => Some(UiLanguage::English),
        _ => None,
    }
}

fn update_tree_menu_state(hwnd: HWND, context: &WindowContext) {
    // SAFETY: hwnd is the main window; GetMenu returns the attached menu bar or null.
    let menu = unsafe { GetMenu(hwnd) };
    if menu.is_null() {
        return;
    }

    let state = current_tree_menu_state(context);
    let delete_state = if state.can_delete_tree_item {
        MF_BYCOMMAND | MF_ENABLED
    } else {
        MF_BYCOMMAND | MF_GRAYED
    };
    let edit_state = if state.can_edit_tree_item {
        MF_BYCOMMAND | MF_ENABLED
    } else {
        MF_BYCOMMAND | MF_GRAYED
    };
    let move_up_state = if state.can_move_up {
        MF_BYCOMMAND | MF_ENABLED
    } else {
        MF_BYCOMMAND | MF_GRAYED
    };
    let move_down_state = if state.can_move_down {
        MF_BYCOMMAND | MF_ENABLED
    } else {
        MF_BYCOMMAND | MF_GRAYED
    };

    // SAFETY: menu is the main menu and these command IDs were created by create_menu_bar.
    unsafe {
        EnableMenuItem(menu, MENU_TREE_DELETE_ID, delete_state);
        EnableMenuItem(menu, MENU_TREE_MOVE_UP_ID, move_up_state);
        EnableMenuItem(menu, MENU_TREE_MOVE_DOWN_ID, move_down_state);
        EnableMenuItem(menu, MENU_TREE_EDIT_ID, edit_state);
        DrawMenuBar(hwnd);
    }
}

fn current_tree_menu_state(context: &WindowContext) -> TreeMenuState {
    let (can_move_up, can_move_down) = tree_move_menu_state(context);
    let selection = selected_tree_node(context);
    TreeMenuState {
        can_delete_tree_item: selection.is_some(),
        can_edit_tree_item: selection.is_some(),
        can_move_up,
        can_move_down,
    }
}

fn tree_move_menu_state(context: &WindowContext) -> (bool, bool) {
    let Some(selection) = selected_tree_node(context) else {
        return (false, false);
    };
    let settings = context.spec.state.settings();

    match selection {
        TreeNodeSelection::Workspace(index) => {
            let Some(workspace) = settings.workspaces.get(index) else {
                return (false, false);
            };
            if layout_rules::workspace_category_index(workspace, settings.categories.as_slice())
                .is_some()
            {
                (
                    layout_rules::workspace_keyboard_move_destination(
                        settings.workspaces.as_slice(),
                        settings.categories.as_slice(),
                        index,
                        TreeKeyboardMoveDirection::Up,
                    )
                    .is_some(),
                    layout_rules::workspace_keyboard_move_destination(
                        settings.workspaces.as_slice(),
                        settings.categories.as_slice(),
                        index,
                        TreeKeyboardMoveDirection::Down,
                    )
                    .is_some(),
                )
            } else {
                let root_items = settings.root_tree_items();
                (
                    layout_rules::tree_root_keyboard_move_destination(
                        root_items.as_slice(),
                        TreeRootItemRef::Workspace(index),
                        TreeKeyboardMoveDirection::Up,
                    )
                    .is_some(),
                    layout_rules::tree_root_keyboard_move_destination(
                        root_items.as_slice(),
                        TreeRootItemRef::Workspace(index),
                        TreeKeyboardMoveDirection::Down,
                    )
                    .is_some(),
                )
            }
        }
        TreeNodeSelection::Category(index) => {
            let root_items = settings.root_tree_items();
            (
                layout_rules::tree_root_keyboard_move_destination(
                    root_items.as_slice(),
                    TreeRootItemRef::Category(index),
                    TreeKeyboardMoveDirection::Up,
                )
                .is_some(),
                layout_rules::tree_root_keyboard_move_destination(
                    root_items.as_slice(),
                    TreeRootItemRef::Category(index),
                    TreeKeyboardMoveDirection::Down,
                )
                .is_some(),
            )
        }
    }
}

fn update_tabs_menu_state(hwnd: HWND, context: &WindowContext) {
    // SAFETY: hwnd is the main window; GetMenu returns the attached menu bar or null.
    let menu = unsafe { GetMenu(hwnd) };
    if menu.is_null() {
        return;
    }

    let selected_index = context.spec.state.selected_command_tab_index();
    let tab_count = context.spec.state.settings().command_tabs.len();
    let has_selected_command_tab = selected_index.is_some();
    let state = if has_selected_command_tab {
        MF_BYCOMMAND | MF_ENABLED
    } else {
        MF_BYCOMMAND | MF_GRAYED
    };
    let move_left_state = if selected_index
        .and_then(|index| {
            layout_rules::command_tab_move_destination(
                index,
                tab_count,
                CommandTabMoveDirection::Left,
            )
        })
        .is_some()
    {
        MF_BYCOMMAND | MF_ENABLED
    } else {
        MF_BYCOMMAND | MF_GRAYED
    };
    let move_right_state = if selected_index
        .and_then(|index| {
            layout_rules::command_tab_move_destination(
                index,
                tab_count,
                CommandTabMoveDirection::Right,
            )
        })
        .is_some()
    {
        MF_BYCOMMAND | MF_ENABLED
    } else {
        MF_BYCOMMAND | MF_GRAYED
    };

    // SAFETY: menu is the main menu and these command IDs were created by create_menu_bar.
    unsafe {
        EnableMenuItem(menu, MENU_TABS_DELETE_ID, state);
        EnableMenuItem(menu, MENU_TABS_MOVE_LEFT_ID, move_left_state);
        EnableMenuItem(menu, MENU_TABS_MOVE_RIGHT_ID, move_right_state);
        EnableMenuItem(menu, MENU_TABS_RENAME_ID, state);
        DrawMenuBar(hwnd);
    }
}

fn update_commands_menu_state(hwnd: HWND, context: &WindowContext) {
    // SAFETY: hwnd is the main window; GetMenu returns the attached menu bar or null.
    let menu = unsafe { GetMenu(hwnd) };
    if menu.is_null() {
        return;
    }

    let state = current_command_menu_state(context);

    // SAFETY: menu is the main menu and these command IDs were created by create_menu_bar.
    unsafe {
        EnableMenuItem(
            menu,
            MENU_COMMANDS_EXECUTE_ID,
            menu_item_state(state.can_execute),
        );
        EnableMenuItem(menu, MENU_COMMANDS_ADD_ID, menu_item_state(state.can_add));
        EnableMenuItem(menu, MENU_COMMANDS_EDIT_ID, menu_item_state(state.can_edit));
        EnableMenuItem(
            menu,
            MENU_COMMANDS_MOVE_PREVIOUS_ID,
            menu_item_state(state.can_move_previous),
        );
        EnableMenuItem(
            menu,
            MENU_COMMANDS_MOVE_NEXT_ID,
            menu_item_state(state.can_move_next),
        );
        EnableMenuItem(
            menu,
            MENU_COMMANDS_DELETE_ID,
            menu_item_state(state.can_delete),
        );
        DrawMenuBar(hwnd);
    }
}

fn current_command_menu_state(context: &WindowContext) -> CommandMenuState {
    let has_selected_tab = context.spec.state.selected_command_tab().is_some();
    let selected_button_index = context.spec.state.selected_command_button_index();
    let selected_button_count = context
        .spec
        .state
        .selected_command_tab()
        .map(|tab| tab.buttons.len())
        .unwrap_or_default();
    let has_selected_button = context.spec.state.selected_command_button().is_some();
    let can_target_button = has_selected_tab && has_selected_button;

    CommandMenuState {
        can_execute: can_target_button
            && !command_button_click_ignored_for_tree_selection(selected_tree_node(context)),
        can_add: has_selected_tab,
        can_edit: can_target_button,
        can_delete: can_target_button,
        can_move_previous: can_target_button
            && selected_button_index
                .and_then(|index| {
                    layout_rules::command_button_move_destination(
                        index,
                        selected_button_count,
                        CommandButtonMoveDirection::Previous,
                    )
                })
                .is_some(),
        can_move_next: can_target_button
            && selected_button_index
                .and_then(|index| {
                    layout_rules::command_button_move_destination(
                        index,
                        selected_button_count,
                        CommandButtonMoveDirection::Next,
                    )
                })
                .is_some(),
    }
}

fn menu_item_state(enabled: bool) -> u32 {
    if enabled {
        MF_BYCOMMAND | MF_ENABLED
    } else {
        MF_BYCOMMAND | MF_GRAYED
    }
}

#[repr(C)]
struct DropTargetVtbl {
    base: IUnknown_Vtbl,
    drag_enter: unsafe extern "system" fn(
        this: *mut c_void,
        data_object: *mut c_void,
        key_state: u32,
        point: POINTL,
        effect: *mut u32,
    ) -> HRESULT,
    drag_over: unsafe extern "system" fn(
        this: *mut c_void,
        key_state: u32,
        point: POINTL,
        effect: *mut u32,
    ) -> HRESULT,
    drag_leave: unsafe extern "system" fn(this: *mut c_void) -> HRESULT,
    drop: unsafe extern "system" fn(
        this: *mut c_void,
        data_object: *mut c_void,
        key_state: u32,
        point: POINTL,
        effect: *mut u32,
    ) -> HRESULT,
}

#[repr(C)]
struct DataObjectVtbl {
    base: IUnknown_Vtbl,
    get_data: unsafe extern "system" fn(
        this: *mut c_void,
        format_etc: *mut FORMATETC,
        medium: *mut STGMEDIUM,
    ) -> HRESULT,
}

#[repr(C)]
struct WorkspaceDropTarget {
    vtbl: *const DropTargetVtbl,
    ref_count: AtomicU32,
    main_hwnd: HWND,
    tree_view: HWND,
    context: *mut WindowContext,
    feedback: DropFeedback,
}

impl WorkspaceDropTarget {
    fn new(main_hwnd: HWND, tree_view: HWND, context: *mut WindowContext) -> Self {
        Self {
            vtbl: &WORKSPACE_DROP_TARGET_VTBL,
            ref_count: AtomicU32::new(1),
            main_hwnd,
            tree_view,
            context,
            feedback: DropFeedback::Normal,
        }
    }

    fn set_feedback(&mut self, feedback: DropFeedback) {
        if self.feedback == feedback {
            return;
        }

        self.feedback = feedback;
        let theme = current_drop_target_theme(self.context);
        set_tree_drop_feedback(self.tree_view, feedback, theme);
    }
}

fn current_drop_target_theme(context: *mut WindowContext) -> ViewTheme {
    if context.is_null() {
        return ViewTheme::default();
    }

    // SAFETY: The drop target stores the non-owning WindowContext pointer supplied during
    // registration, which remains valid until the target is revoked before child destruction.
    let Some(context) = (unsafe { context.as_ref() }) else {
        return ViewTheme::default();
    };

    current_view_theme(&context.spec.state.settings().view)
}

struct DropTargetRegistration {
    hwnd: HWND,
    target: *mut WorkspaceDropTarget,
}

struct PendingWorkspaceDropCheck {
    request_id: u32,
    result: Arc<Mutex<Option<WorkspaceDropCheckResult>>>,
}

impl Drop for DropTargetRegistration {
    fn drop(&mut self) {
        // SAFETY: hwnd was registered with RegisterDragDrop, and target is the owned COM object
        // reference created by register_workspace_drop_target.
        unsafe {
            let _ = RevokeDragDrop(self.hwnd);
            drop_target_release(self.target as *mut c_void);
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum DropFeedback {
    Normal,
    Allowed,
    Denied,
}

static WORKSPACE_DROP_TARGET_VTBL: DropTargetVtbl = DropTargetVtbl {
    base: IUnknown_Vtbl {
        QueryInterface: drop_target_query_interface,
        AddRef: drop_target_add_ref,
        Release: drop_target_release,
    },
    drag_enter: drop_target_drag_enter,
    drag_over: drop_target_drag_over,
    drag_leave: drop_target_drag_leave,
    drop: drop_target_drop,
};

const IID_IDROPTARGET: GUID = GUID::from_u128(0x00000122_0000_0000_c000_000000000046);
// Workspace drops only accept one folder, but allow a small accidental multi-drop through the
// existing validation path while bounding allocations from external HDROP data.
const MAX_HDROP_PATHS_TO_READ: u32 = 16;
const MAX_HDROP_PATH_CHARS: u32 = 32_767;

fn register_workspace_drop_target(hwnd: HWND, context: &mut WindowContext) -> AppResult<()> {
    context.drop_target = None;

    let target = Box::new(WorkspaceDropTarget::new(
        hwnd,
        context.content.tree_view,
        context as *mut WindowContext,
    ));
    let target = Box::into_raw(target);

    // SAFETY: tree_view is a live child window on the UI thread. target is a valid IDropTarget
    // COM object and remains alive through DropTargetRegistration.
    let result = unsafe { RegisterDragDrop(context.content.tree_view, target as *mut c_void) };
    if result < 0 {
        // SAFETY: RegisterDragDrop failed, so release this module's initial reference.
        unsafe {
            drop_target_release(target as *mut c_void);
        }
        return Err(AppError::windows_hresult("RegisterDragDrop", result));
    }

    context.drop_target = Some(DropTargetRegistration {
        hwnd: context.content.tree_view,
        target,
    });
    Ok(())
}

unsafe extern "system" fn drop_target_query_interface(
    this: *mut c_void,
    iid: *const GUID,
    interface: *mut *mut c_void,
) -> HRESULT {
    if iid.is_null() || interface.is_null() {
        return E_POINTER;
    }

    // SAFETY: interface is validated above and belongs to the caller.
    unsafe {
        *interface = null_mut();
    }

    // SAFETY: iid is validated above and points to a GUID supplied by COM.
    let iid = unsafe { &*iid };
    if guid_eq(iid, &IID_IUnknown) || guid_eq(iid, &IID_IDROPTARGET) {
        // SAFETY: this is the COM object pointer supplied by COM for this call.
        unsafe {
            drop_target_add_ref(this);
            *interface = this;
        }
        S_OK
    } else {
        E_NOINTERFACE
    }
}

unsafe extern "system" fn drop_target_add_ref(this: *mut c_void) -> u32 {
    // SAFETY: COM supplies this as the WorkspaceDropTarget pointer for this vtable.
    let Some(target) = (unsafe { (this as *mut WorkspaceDropTarget).as_ref() }) else {
        return 0;
    };

    target.ref_count.fetch_add(1, Ordering::Relaxed) + 1
}

unsafe extern "system" fn drop_target_release(this: *mut c_void) -> u32 {
    let target = this as *mut WorkspaceDropTarget;
    // SAFETY: COM supplies this as the WorkspaceDropTarget pointer for this vtable.
    let Some(target_ref) = (unsafe { (target as *const WorkspaceDropTarget).as_ref() }) else {
        return 0;
    };

    let remaining = target_ref.ref_count.fetch_sub(1, Ordering::Release) - 1;
    if remaining == 0 {
        std::sync::atomic::fence(Ordering::Acquire);
        // SAFETY: The reference count reached zero, so this module owns the final reference and
        // may reconstruct the Box allocated by register_workspace_drop_target.
        unsafe {
            drop(Box::from_raw(target));
        }
    }

    remaining
}

unsafe extern "system" fn drop_target_drag_enter(
    this: *mut c_void,
    data_object: *mut c_void,
    _key_state: u32,
    _point: POINTL,
    effect: *mut u32,
) -> HRESULT {
    // SAFETY: COM supplies this as the WorkspaceDropTarget pointer for this vtable.
    let Some(target) = (unsafe { (this as *mut WorkspaceDropTarget).as_mut() }) else {
        set_drop_effect(effect, DROPEFFECT_NONE);
        return S_OK;
    };

    let feedback = match drag_data_paths(data_object) {
        Ok(paths) if validate_workspace_drop_paths_for_feedback(target.context, &paths).is_ok() => {
            DropFeedback::Allowed
        }
        Ok(_) | Err(_) => DropFeedback::Denied,
    };

    target.set_feedback(feedback);
    set_drop_effect(effect, drop_effect_for_feedback(feedback));
    S_OK
}

unsafe extern "system" fn drop_target_drag_over(
    this: *mut c_void,
    _key_state: u32,
    _point: POINTL,
    effect: *mut u32,
) -> HRESULT {
    // SAFETY: COM supplies this as the WorkspaceDropTarget pointer for this vtable.
    let feedback = unsafe { (this as *mut WorkspaceDropTarget).as_ref() }
        .map(|target| target.feedback)
        .unwrap_or(DropFeedback::Denied);
    set_drop_effect(effect, drop_effect_for_feedback(feedback));
    S_OK
}

unsafe extern "system" fn drop_target_drag_leave(this: *mut c_void) -> HRESULT {
    // SAFETY: COM supplies this as the WorkspaceDropTarget pointer for this vtable.
    if let Some(target) = unsafe { (this as *mut WorkspaceDropTarget).as_mut() } {
        target.set_feedback(DropFeedback::Normal);
    }
    S_OK
}

unsafe extern "system" fn drop_target_drop(
    this: *mut c_void,
    data_object: *mut c_void,
    _key_state: u32,
    _point: POINTL,
    effect: *mut u32,
) -> HRESULT {
    // SAFETY: COM supplies this as the WorkspaceDropTarget pointer for this vtable.
    let Some(target) = (unsafe { (this as *mut WorkspaceDropTarget).as_mut() }) else {
        set_drop_effect(effect, DROPEFFECT_NONE);
        return S_OK;
    };

    target.set_feedback(DropFeedback::Normal);
    let accepted = match drag_data_paths(data_object) {
        Ok(paths) => {
            // SAFETY: context points to the boxed WindowContext owned by run_main_window.
            match unsafe { target.context.as_mut() } {
                Some(context) => handle_workspace_drop_paths(target.main_hwnd, context, paths),
                None => false,
            }
        }
        Err(_) => {
            let language = unsafe { target.context.as_ref() }
                .map(context_language)
                .unwrap_or_default();
            show_warning_message(
                target.main_hwnd,
                tr(language, "워크스페이스", "Workspace"),
                tr(
                    language,
                    "폴더 하나를 다시 드롭하세요.",
                    "Drop one folder again.",
                ),
            );
            false
        }
    };

    set_drop_effect(
        effect,
        if accepted {
            DROPEFFECT_COPY
        } else {
            DROPEFFECT_NONE
        },
    );
    S_OK
}

fn guid_eq(left: &GUID, right: &GUID) -> bool {
    left.data1 == right.data1
        && left.data2 == right.data2
        && left.data3 == right.data3
        && left.data4 == right.data4
}

fn set_drop_effect(effect: *mut u32, value: u32) {
    if effect.is_null() {
        return;
    }

    // SAFETY: effect is supplied by COM and points to writable DROPEFFECT storage.
    unsafe {
        *effect = value;
    }
}

fn drop_effect_for_feedback(feedback: DropFeedback) -> u32 {
    match feedback {
        // Keep rejected file-system drops deliverable so Drop can show the specific rejection
        // message instead of failing silently at DragEnter/DragOver.
        DropFeedback::Allowed | DropFeedback::Denied => DROPEFFECT_COPY,
        DropFeedback::Normal => DROPEFFECT_NONE,
    }
}

fn set_tree_drop_feedback(tree_view: HWND, feedback: DropFeedback, theme: ViewTheme) {
    if tree_view.is_null() {
        return;
    }

    let colors = TreeViewThemeColors::for_drop_feedback(feedback, ThemePalette::for_theme(theme));

    // SAFETY: tree_view is a TreeView control owned by this UI thread.
    unsafe {
        SendMessageW(tree_view, TVM_SETBKCOLOR, 0, colors.background);
        SendMessageW(tree_view, TVM_SETTEXTCOLOR, 0, colors.text);
        SendMessageW(tree_view, TVM_SETLINECOLOR, 0, colors.line);
        InvalidateRect(tree_view, null(), 1);
    }
}

fn colorref(red: u8, green: u8, blue: u8) -> u32 {
    red as u32 | ((green as u32) << 8) | ((blue as u32) << 16)
}

fn blend_colorref(base: u32, accent: u32, accent_percent: u8) -> u32 {
    let accent_percent = u16::from(accent_percent.min(100));
    let base_percent = 100 - accent_percent;

    colorref(
        blend_color_channel(
            colorref_red(base),
            colorref_red(accent),
            base_percent,
            accent_percent,
        ),
        blend_color_channel(
            colorref_green(base),
            colorref_green(accent),
            base_percent,
            accent_percent,
        ),
        blend_color_channel(
            colorref_blue(base),
            colorref_blue(accent),
            base_percent,
            accent_percent,
        ),
    )
}

fn blend_color_channel(base: u8, accent: u8, base_percent: u16, accent_percent: u16) -> u8 {
    (((u16::from(base) * base_percent) + (u16::from(accent) * accent_percent) + 50) / 100) as u8
}

fn colorref_red(value: u32) -> u8 {
    (value & 0xff) as u8
}

fn colorref_green(value: u32) -> u8 {
    ((value >> 8) & 0xff) as u8
}

fn colorref_blue(value: u32) -> u8 {
    ((value >> 16) & 0xff) as u8
}

#[derive(Debug, Eq, PartialEq)]
enum DropDataError {
    MissingDataObject,
    GetDataFailed,
    MissingHdrop,
    TooManyHdropItems,
    InvalidHdropPathLength,
}

fn drag_data_paths(data_object: *mut c_void) -> Result<Vec<PathBuf>, DropDataError> {
    if data_object.is_null() {
        return Err(DropDataError::MissingDataObject);
    }

    let mut format_etc = FORMATETC {
        cfFormat: CF_HDROP,
        ptd: null_mut(),
        dwAspect: DVASPECT_CONTENT,
        lindex: -1,
        tymed: TYMED_HGLOBAL as u32,
    };
    let mut medium = STGMEDIUM::default();

    // SAFETY: data_object is an IDataObject pointer supplied by OLE. The vtable prefix is used
    // only to call IDataObject::GetData with a CF_HDROP FORMATETC.
    let result = unsafe {
        let vtbl = *(data_object as *mut *const DataObjectVtbl);
        ((*vtbl).get_data)(data_object, &mut format_etc, &mut medium)
    };

    if result < 0 {
        return Err(DropDataError::GetDataFailed);
    }

    // SAFETY: For CF_HDROP with TYMED_HGLOBAL, STGMEDIUM stores an HDROP-compatible HGLOBAL.
    let hdrop = unsafe { medium.u.hGlobal };
    let paths = if hdrop.is_null() {
        Err(DropDataError::MissingHdrop)
    } else {
        hdrop_paths(hdrop)
    };

    // SAFETY: medium was initialized by IDataObject::GetData and must be released once.
    unsafe {
        ReleaseStgMedium(&mut medium);
    }

    paths
}

fn hdrop_paths(hdrop: HGLOBAL) -> Result<Vec<PathBuf>, DropDataError> {
    let hdrop = hdrop as HDROP;
    // SAFETY: hdrop is owned by the STGMEDIUM returned for CF_HDROP.
    let count = unsafe { DragQueryFileW(hdrop, u32::MAX, null_mut(), 0) };
    let mut paths = Vec::with_capacity(hdrop_path_count_capacity(count)?);

    for index in 0..count {
        // SAFETY: hdrop is a valid HDROP. A null buffer asks for the required character count.
        let len = unsafe { DragQueryFileW(hdrop, index, null_mut(), 0) };
        if len == 0 {
            continue;
        }

        let buffer_len = hdrop_path_buffer_len(len)?;
        let buffer_len_u32 =
            u32::try_from(buffer_len).map_err(|_| DropDataError::InvalidHdropPathLength)?;
        let mut buffer = vec![0u16; buffer_len];
        // SAFETY: buffer has len + 1 writable u16 slots.
        let copied = unsafe { DragQueryFileW(hdrop, index, buffer.as_mut_ptr(), buffer_len_u32) };
        if copied == 0 {
            continue;
        }

        let copied_len = hdrop_copied_path_len(copied, len)?;
        paths.push(PathBuf::from(String::from_utf16_lossy(
            &buffer[..copied_len],
        )));
    }

    Ok(paths)
}

fn hdrop_path_count_capacity(count: u32) -> Result<usize, DropDataError> {
    if count > MAX_HDROP_PATHS_TO_READ {
        return Err(DropDataError::TooManyHdropItems);
    }

    usize::try_from(count).map_err(|_| DropDataError::TooManyHdropItems)
}

fn hdrop_path_buffer_len(path_len: u32) -> Result<usize, DropDataError> {
    if path_len > MAX_HDROP_PATH_CHARS {
        return Err(DropDataError::InvalidHdropPathLength);
    }

    let path_len = usize::try_from(path_len).map_err(|_| DropDataError::InvalidHdropPathLength)?;
    path_len
        .checked_add(1)
        .ok_or(DropDataError::InvalidHdropPathLength)
}

fn hdrop_copied_path_len(copied_len: u32, queried_len: u32) -> Result<usize, DropDataError> {
    if copied_len > queried_len {
        return Err(DropDataError::InvalidHdropPathLength);
    }

    usize::try_from(copied_len).map_err(|_| DropDataError::InvalidHdropPathLength)
}

#[derive(Debug, Eq, PartialEq)]
enum WorkspaceDropRejectReason {
    Empty,
    MultipleItems(usize),
    NotFolder(PathBuf),
    UnreadableFolder(PathBuf),
    DuplicatePath { path: PathBuf, name: String },
}

struct WorkspaceDropFolder {
    path: PathBuf,
    entry_names: Vec<Option<String>>,
}

enum WorkspaceDropCheckResult {
    Accepted(WorkspaceDropFolder),
    Rejected(WorkspaceDropRejectReason),
}

fn validate_workspace_drop_paths_for_feedback(
    context: *mut WindowContext,
    paths: &[PathBuf],
) -> Result<(), WorkspaceDropRejectReason> {
    // SAFETY: context is the pointer stored in the OLE drop target while the main window lives.
    let Some(context) = (unsafe { (context as *const WindowContext).as_ref() }) else {
        return Err(WorkspaceDropRejectReason::Empty);
    };

    validate_workspace_drop_paths(context, paths).map(|_| ())
}

fn validate_workspace_drop_paths<'a>(
    context: &WindowContext,
    paths: &'a [PathBuf],
) -> Result<&'a Path, WorkspaceDropRejectReason> {
    validate_workspace_drop_paths_against_workspaces(
        paths,
        context.spec.state.settings().workspaces.as_slice(),
    )
}

fn validate_workspace_drop_paths_against_workspaces<'a>(
    paths: &'a [PathBuf],
    workspaces: &[Workspace],
) -> Result<&'a Path, WorkspaceDropRejectReason> {
    let path = validate_single_workspace_drop_path(paths)?;
    reject_duplicate_workspace_drop_path(path, workspaces)?;
    Ok(path)
}

fn validate_single_workspace_drop_path(
    paths: &[PathBuf],
) -> Result<&Path, WorkspaceDropRejectReason> {
    if paths.is_empty() {
        return Err(WorkspaceDropRejectReason::Empty);
    }

    if paths.len() > 1 {
        return Err(WorkspaceDropRejectReason::MultipleItems(paths.len()));
    }

    Ok(paths[0].as_path())
}

fn reject_duplicate_workspace_drop_path(
    path: &Path,
    workspaces: &[Workspace],
) -> Result<(), WorkspaceDropRejectReason> {
    let path_text = path.display().to_string();
    if let Some(existing) = workspaces
        .iter()
        .find(|workspace| workspace_paths_equal(&workspace.path, &path_text))
    {
        return Err(WorkspaceDropRejectReason::DuplicatePath {
            path: path.to_path_buf(),
            name: existing.name.clone(),
        });
    }

    Ok(())
}

fn inspect_workspace_drop_path_against_workspaces(
    path: &Path,
    workspaces: &[Workspace],
) -> Result<WorkspaceDropFolder, WorkspaceDropRejectReason> {
    let metadata = std::fs::metadata(path)
        .map_err(|_| WorkspaceDropRejectReason::UnreadableFolder(path.to_path_buf()))?;
    if !metadata.is_dir() {
        return Err(WorkspaceDropRejectReason::NotFolder(path.to_path_buf()));
    }

    let entry_names = workspace_folder_entry_names(path)?;
    reject_duplicate_workspace_drop_path(path, workspaces)?;

    Ok(WorkspaceDropFolder {
        path: path.to_path_buf(),
        entry_names,
    })
}

fn workspace_folder_entry_names(
    path: &Path,
) -> Result<Vec<Option<String>>, WorkspaceDropRejectReason> {
    let entries = std::fs::read_dir(path)
        .map_err(|_| WorkspaceDropRejectReason::UnreadableFolder(path.to_path_buf()))?;
    Ok(read_dir_entry_names(entries)
        .take(WORKSPACE_LANGUAGE_INFERENCE_ENTRY_LIMIT)
        .collect())
}

fn read_dir_entry_names(entries: std::fs::ReadDir) -> impl Iterator<Item = Option<String>> {
    entries.map(|entry| {
        let entry = entry.ok()?;
        entry.file_name().to_str().map(ToOwned::to_owned)
    })
}

fn handle_workspace_drop_paths(
    hwnd: HWND,
    context: &mut WindowContext,
    paths: Vec<PathBuf>,
) -> bool {
    let path = match validate_workspace_drop_paths(context, &paths) {
        Ok(path) => path.to_path_buf(),
        Err(reason) => {
            show_warning_message(
                hwnd,
                context_tr(context, "워크스페이스", "Workspace"),
                &workspace_drop_reject_message(context_language(context), reason),
            );
            return false;
        }
    };

    begin_workspace_drop_check(hwnd, context, path)
}

fn begin_workspace_drop_check(hwnd: HWND, context: &mut WindowContext, path: PathBuf) -> bool {
    let request_id = next_workspace_drop_check_id();
    let result = Arc::new(Mutex::new(None));
    let result_for_worker = Arc::clone(&result);
    let hwnd_value = hwnd as isize;
    let workspaces = context.spec.state.settings().workspaces.clone();

    context.workspace_drop_check = Some(PendingWorkspaceDropCheck { request_id, result });

    let spawn_result = std::thread::Builder::new()
        .name("workspace-drop-check".to_owned())
        .spawn(move || {
            let check_result =
                match inspect_workspace_drop_path_against_workspaces(&path, &workspaces) {
                    Ok(folder) => WorkspaceDropCheckResult::Accepted(folder),
                    Err(reason) => WorkspaceDropCheckResult::Rejected(reason),
                };

            let Ok(mut result) = result_for_worker.lock() else {
                return;
            };
            *result = Some(check_result);

            // SAFETY: hwnd_value is a window handle captured as an integer. The worker only posts a
            // message and never dereferences UI state.
            unsafe {
                PostMessageW(
                    hwnd_value as HWND,
                    WM_WORKSPACE_DROP_CHECKED,
                    request_id as WPARAM,
                    0,
                );
            }
        });

    if spawn_result.is_ok() {
        true
    } else {
        context.workspace_drop_check = None;
        show_warning_message(
            hwnd,
            context_tr(context, "워크스페이스", "Workspace"),
            context_tr(
                context,
                "폴더를 확인할 수 없습니다.",
                "Could not validate the folder.",
            ),
        );
        false
    }
}

fn next_workspace_drop_check_id() -> u32 {
    NEXT_WORKSPACE_DROP_CHECK_ID.fetch_add(1, Ordering::Relaxed)
}

fn handle_workspace_drop_checked(hwnd: HWND, context: &mut WindowContext, request_id: u32) {
    let Some(pending) = context.workspace_drop_check.take() else {
        return;
    };

    if pending.request_id != request_id {
        context.workspace_drop_check = Some(pending);
        return;
    }

    let result = match pending.result.lock() {
        Ok(mut result) => result.take(),
        Err(_) => None,
    };

    match result {
        Some(WorkspaceDropCheckResult::Accepted(folder)) => {
            add_workspace_from_dropped_folder(hwnd, context, folder);
        }
        Some(WorkspaceDropCheckResult::Rejected(reason)) => {
            show_warning_message(
                hwnd,
                context_tr(context, "워크스페이스", "Workspace"),
                &workspace_drop_reject_message(context_language(context), reason),
            );
        }
        None => {}
    }
}

fn add_workspace_from_dropped_folder(
    hwnd: HWND,
    context: &mut WindowContext,
    folder: WorkspaceDropFolder,
) -> bool {
    if let Err(reason) = reject_duplicate_workspace_drop_path(
        &folder.path,
        context.spec.state.settings().workspaces.as_slice(),
    ) {
        show_warning_message(
            hwnd,
            context_tr(context, "워크스페이스", "Workspace"),
            &workspace_drop_reject_message(context_language(context), reason),
        );
        return false;
    }

    let Some(workspace) = workspace_from_dropped_folder(hwnd, context, folder) else {
        return false;
    };

    let previous_state = SettingsRestorePoint::capture(&context.spec.state);
    match context.spec.state.add_workspace(workspace) {
        Ok(_) => {
            if !persist_settings_or_restore(hwnd, &mut context.spec.state, previous_state) {
                return false;
            }
            refresh_tree_view(context);
            update_tree_menu_state(hwnd, context);
            true
        }
        Err(error) => {
            show_error_message(
                hwnd,
                context_tr(context, "워크스페이스", "Workspace"),
                &error.user_message_for_language(context_language(context)),
            );
            false
        }
    }
}

fn workspace_from_dropped_folder(
    hwnd: HWND,
    context: &mut WindowContext,
    folder: WorkspaceDropFolder,
) -> Option<Workspace> {
    let path_text = folder.path.display().to_string();
    let name = default_workspace_name_for_path(&folder.path);
    let language_options = context.spec.state.settings().languages.as_slice();

    if let Some(language) = infer_workspace_language_from_folder_entries(folder.entry_names)
        .and_then(|language| normalize_workspace_language(language, language_options))
    {
        return match Workspace::new_with_language_options(
            path_text,
            name,
            language,
            language_options,
        ) {
            Ok(workspace) => Some(workspace),
            Err(error) => {
                show_warning_message(
                    hwnd,
                    context_tr(context, "워크스페이스", "Workspace"),
                    &error.user_message_for_language(context_language(context)),
                );
                None
            }
        };
    }

    let initial_language = default_workspace_language_for_options(language_options);
    let initial_workspace = match Workspace::new_with_language_options(
        path_text,
        name,
        initial_language,
        language_options,
    ) {
        Ok(workspace) => workspace,
        Err(error) => {
            show_warning_message(
                hwnd,
                context_tr(context, "워크스페이스", "Workspace"),
                &error.user_message_for_language(context_language(context)),
            );
            return None;
        }
    };

    let reserved_paths = workspace_paths_except(context, None);
    match show_workspace_dialog(
        hwnd,
        context.instance,
        WorkspaceDialogMode::Add,
        Some(initial_workspace),
        reserved_paths,
        context.spec.state.settings().languages.clone(),
        context.ui_font.handle(),
        context.spec.state.settings().view.font_size,
        context_language(context),
    ) {
        Ok(workspace) => workspace,
        Err(error) => {
            show_error_message(
                hwnd,
                context_tr(context, "워크스페이스", "Workspace"),
                &error.to_string(),
            );
            None
        }
    }
}

fn workspace_drop_reject_message(
    language: UiLanguage,
    reason: WorkspaceDropRejectReason,
) -> String {
    match (language, reason) {
        (UiLanguage::Korean, WorkspaceDropRejectReason::Empty) => {
            "폴더 하나를 드롭하세요.".to_owned()
        }
        (UiLanguage::English, WorkspaceDropRejectReason::Empty) => "Drop one folder.".to_owned(),
        (UiLanguage::Korean, WorkspaceDropRejectReason::MultipleItems(count)) => {
            format!("폴더 하나만 드롭하세요. ({count}개 선택됨)")
        }
        (UiLanguage::English, WorkspaceDropRejectReason::MultipleItems(count)) => {
            format!("Drop only one folder. ({count} items selected)")
        }
        (UiLanguage::Korean, WorkspaceDropRejectReason::NotFolder(path)) => format!(
            "파일은 등록할 수 없습니다. 폴더를 드롭하세요.\n\n항목: {}",
            path.display()
        ),
        (UiLanguage::English, WorkspaceDropRejectReason::NotFolder(path)) => format!(
            "Files cannot be registered. Drop a folder.\n\nItem: {}",
            path.display()
        ),
        (UiLanguage::Korean, WorkspaceDropRejectReason::UnreadableFolder(path)) => {
            format!("폴더를 열 수 없습니다.\n\n폴더: {}", path.display())
        }
        (UiLanguage::English, WorkspaceDropRejectReason::UnreadableFolder(path)) => {
            format!("Could not open the folder.\n\nFolder: {}", path.display())
        }
        (UiLanguage::Korean, WorkspaceDropRejectReason::DuplicatePath { path, name }) => format!(
            "이미 등록된 폴더입니다.\n\n이름: {name}\n폴더: {}",
            path.display()
        ),
        (UiLanguage::English, WorkspaceDropRejectReason::DuplicatePath { path, name }) => format!(
            "This folder is already registered.\n\nName: {name}\nFolder: {}",
            path.display()
        ),
    }
}

fn infer_workspace_language_from_folder(path: &Path) -> Option<&'static str> {
    let entries = std::fs::read_dir(path).ok()?;
    infer_workspace_language_from_folder_entries(read_dir_entry_names(entries))
}

fn infer_workspace_language_from_folder_entries<I>(entry_names: I) -> Option<&'static str>
where
    I: IntoIterator<Item = Option<String>>,
{
    infer_workspace_language_from_limited_entry_names(entry_names)
}

fn infer_workspace_language_from_limited_entry_names<I>(entry_names: I) -> Option<&'static str>
where
    I: IntoIterator<Item = Option<String>>,
{
    let entry_names = entry_names
        .into_iter()
        .take(WORKSPACE_LANGUAGE_INFERENCE_ENTRY_LIMIT)
        .flatten()
        .collect::<Vec<_>>();

    infer_workspace_language_from_entry_names(entry_names.iter().map(String::as_str))
}

fn create_menu_bar(
    menus: &[MenuDefinition],
    selected_theme: ViewTheme,
    selected_language: UiLanguage,
) -> AppResult<HMENU> {
    // SAFETY: CreateMenu has no preconditions.
    let menu_bar = unsafe { CreateMenu() };
    if menu_bar.is_null() {
        return Err(last_error("CreateMenu"));
    }

    for (menu_index, menu) in menus.iter().enumerate() {
        if let Err(error) = append_top_level_menu(
            menu_bar,
            menu,
            menu_index,
            selected_theme,
            selected_language,
        ) {
            // SAFETY: menu_bar is not attached to a window yet, so this releases all owned popups.
            unsafe {
                DestroyMenu(menu_bar);
            }
            return Err(error);
        }
    }

    Ok(menu_bar)
}

fn append_top_level_menu(
    menu_bar: HMENU,
    menu: &MenuDefinition,
    menu_index: usize,
    selected_theme: ViewTheme,
    selected_language: UiLanguage,
) -> AppResult<()> {
    // SAFETY: CreatePopupMenu has no preconditions.
    let popup = unsafe { CreatePopupMenu() };
    if popup.is_null() {
        return Err(last_error("CreatePopupMenu"));
    }

    for (item_index, item) in menu.items.iter().enumerate() {
        if let Err(error) = append_menu_item(
            popup,
            item,
            menu_index,
            item_index,
            selected_theme,
            selected_language,
        ) {
            // SAFETY: popup has not been attached to the top-level menu after an item append
            // failure inside this branch.
            unsafe {
                DestroyMenu(popup);
            }
            return Err(error);
        }
    }

    let label = wide_null(menu.label);
    // SAFETY: menu_bar and popup are valid HMENU handles, and label is null-terminated for the
    // duration of the call. After success, menu_bar owns popup.
    let appended = unsafe { AppendMenuW(menu_bar, MF_POPUP, popup as usize, label.as_ptr()) };
    if appended == 0 {
        // SAFETY: AppendMenuW failed, so ownership of popup was not transferred.
        unsafe {
            DestroyMenu(popup);
        }
        Err(last_error("AppendMenuW top-level"))
    } else {
        Ok(())
    }
}

fn append_menu_item(
    popup: HMENU,
    item: &MenuItemDefinition,
    menu_index: usize,
    item_index: usize,
    selected_theme: ViewTheme,
    selected_language: UiLanguage,
) -> AppResult<()> {
    let item_id = MENU_ID_BASE + menu_index * 100 + item_index;
    if item_id == MENU_FILE_THEME_ID as usize {
        return append_theme_submenu(popup, item, selected_theme, selected_language);
    }
    if item_id == MENU_FILE_UI_LANGUAGE_ID as usize {
        return append_ui_language_submenu(popup, item, selected_language);
    }

    let flags = if item.enabled { MF_ENABLED } else { MF_GRAYED };
    let label = wide_null(item.label);

    // SAFETY: popup is a valid HMENU handle and label is null-terminated for the duration of the
    // call. The menu stores its own copy of the string.
    let appended = unsafe { AppendMenuW(popup, flags, item_id, label.as_ptr()) };
    if appended == 0 {
        Err(last_error("AppendMenuW item"))
    } else {
        Ok(())
    }
}

fn append_theme_submenu(
    parent_popup: HMENU,
    item: &MenuItemDefinition,
    selected_theme: ViewTheme,
    selected_language: UiLanguage,
) -> AppResult<()> {
    // SAFETY: CreatePopupMenu has no preconditions.
    let theme_menu = unsafe { CreatePopupMenu() };
    if theme_menu.is_null() {
        return Err(last_error("CreatePopupMenu theme"));
    }

    for theme in ViewTheme::options().iter().copied() {
        if let Err(error) =
            append_theme_menu_item(theme_menu, theme, selected_theme, selected_language)
        {
            // SAFETY: theme_menu has not been attached to a parent menu after a child append
            // failure inside this branch.
            unsafe {
                DestroyMenu(theme_menu);
            }
            return Err(error);
        }
    }

    let flags = (if item.enabled { MF_ENABLED } else { MF_GRAYED }) | MF_POPUP;
    let label = wide_null(item.label);
    // SAFETY: parent_popup and theme_menu are valid HMENU handles, and label is null-terminated
    // for the duration of the call. After success, parent_popup owns theme_menu.
    let appended = unsafe { AppendMenuW(parent_popup, flags, theme_menu as usize, label.as_ptr()) };
    if appended == 0 {
        // SAFETY: AppendMenuW failed, so ownership of theme_menu was not transferred.
        unsafe {
            DestroyMenu(theme_menu);
        }
        Err(last_error("AppendMenuW theme submenu"))
    } else {
        Ok(())
    }
}

fn append_theme_menu_item(
    theme_menu: HMENU,
    theme: ViewTheme,
    selected_theme: ViewTheme,
    selected_language: UiLanguage,
) -> AppResult<()> {
    let command_id = command_for_view_theme(theme);
    let flags = MF_ENABLED
        | if theme == selected_theme {
            MF_CHECKED
        } else {
            MF_UNCHECKED
        };
    let label = wide_null(theme.display_name_for(selected_language));

    // SAFETY: theme_menu is a valid HMENU handle and label is null-terminated for this call.
    let appended = unsafe { AppendMenuW(theme_menu, flags, command_id as usize, label.as_ptr()) };
    if appended == 0 {
        Err(last_error("AppendMenuW theme item"))
    } else {
        Ok(())
    }
}

fn append_ui_language_submenu(
    parent_popup: HMENU,
    item: &MenuItemDefinition,
    selected_language: UiLanguage,
) -> AppResult<()> {
    // SAFETY: CreatePopupMenu has no preconditions.
    let language_menu = unsafe { CreatePopupMenu() };
    if language_menu.is_null() {
        return Err(last_error("CreatePopupMenu UI language"));
    }

    for language in UiLanguage::options().iter().copied() {
        if let Err(error) = append_ui_language_menu_item(language_menu, language, selected_language)
        {
            // SAFETY: language_menu has not been attached to a parent menu after a child append
            // failure inside this branch.
            unsafe {
                DestroyMenu(language_menu);
            }
            return Err(error);
        }
    }

    let flags = (if item.enabled { MF_ENABLED } else { MF_GRAYED }) | MF_POPUP;
    let label = wide_null(item.label);
    // SAFETY: parent_popup and language_menu are valid HMENU handles, and label is
    // null-terminated for the duration of the call. After success, parent_popup owns language_menu.
    let appended =
        unsafe { AppendMenuW(parent_popup, flags, language_menu as usize, label.as_ptr()) };
    if appended == 0 {
        // SAFETY: AppendMenuW failed, so ownership of language_menu was not transferred.
        unsafe {
            DestroyMenu(language_menu);
        }
        Err(last_error("AppendMenuW UI language submenu"))
    } else {
        Ok(())
    }
}

fn append_ui_language_menu_item(
    language_menu: HMENU,
    language: UiLanguage,
    selected_language: UiLanguage,
) -> AppResult<()> {
    let command_id = command_for_ui_language(language);
    let flags = MF_ENABLED
        | if language == selected_language {
            MF_CHECKED
        } else {
            MF_UNCHECKED
        };
    let label = wide_null(language.display_name_for(selected_language));

    // SAFETY: language_menu is a valid HMENU handle and label is null-terminated for this call.
    let appended =
        unsafe { AppendMenuW(language_menu, flags, command_id as usize, label.as_ptr()) };
    if appended == 0 {
        Err(last_error("AppendMenuW UI language item"))
    } else {
        Ok(())
    }
}

fn load_application_icons(instance: HINSTANCE) -> IconPair {
    let resource_icons = IconPair {
        large: load_icon_from_resource(instance, 32, 32),
        small: load_icon_from_resource(instance, 16, 16),
    };
    if resource_icons.is_complete() {
        return resource_icons;
    }

    if let Some(icon_path) = find_icon_path() {
        let file_icons = IconPair {
            large: load_icon_from_file(&icon_path, 32, 32),
            small: load_icon_from_file(&icon_path, 16, 16),
        };
        if file_icons.has_any() {
            return file_icons;
        }
    }

    resource_icons
}

fn load_icon_from_resource(instance: HINSTANCE, width: i32, height: i32) -> HICON {
    // SAFETY: APP_ICON_RESOURCE_ID matches the icon resource written by build.rs through
    // winresource::WindowsResource::set_icon. MAKEINTRESOURCEW is represented as a low integer
    // value in the resource-name pointer parameter.
    unsafe {
        LoadImageW(
            instance,
            make_int_resource_w(APP_ICON_RESOURCE_ID),
            IMAGE_ICON,
            width,
            height,
            0,
        ) as HICON
    }
}

fn make_int_resource_w(resource_id: u16) -> *const u16 {
    resource_id as usize as *const u16
}

fn find_icon_path() -> Option<PathBuf> {
    if let Ok(exe_path) = std::env::current_exe()
        && let Some(parent) = exe_path.parent()
    {
        let path = parent.join("icon.ico");
        if path.is_file() {
            return Some(path);
        }
    }

    if let Ok(current_dir) = std::env::current_dir() {
        let path = current_dir.join("icon.ico");
        if path.is_file() {
            return Some(path);
        }
    }

    None
}

fn load_icon_from_file(path: &Path, width: i32, height: i32) -> HICON {
    let path = path_wide_null(path);
    // SAFETY: The path buffer is null-terminated and valid for the call. LR_LOADFROMFILE tells
    // Windows to treat the name pointer as a filesystem path.
    unsafe {
        LoadImageW(
            null_mut(),
            path.as_ptr(),
            IMAGE_ICON,
            width,
            height,
            LR_LOADFROMFILE,
        ) as HICON
    }
}

fn set_window_icons(hwnd: HWND, icons: &IconPair) {
    // SAFETY: hwnd is a valid top-level window handle. WM_SETICON takes icon handles in LPARAM.
    unsafe {
        if !icons.large.is_null() {
            SendMessageW(hwnd, WM_SETICON, ICON_BIG as WPARAM, icons.large as LPARAM);
        }

        if !icons.small.is_null() {
            SendMessageW(
                hwnd,
                WM_SETICON,
                ICON_SMALL as WPARAM,
                icons.small as LPARAM,
            );
        }
    }
}

fn show_window(hwnd: HWND) {
    // SAFETY: hwnd is a valid top-level window handle returned by CreateWindowExW.
    unsafe {
        ShowWindow(hwnd, SW_SHOW);
        UpdateWindow(hwnd);
    }
}

fn message_loop() -> AppResult<()> {
    let mut message = MSG::default();

    loop {
        // SAFETY: message points to writable storage and a null HWND receives all thread messages.
        let result = unsafe { GetMessageW(&mut message, null_mut(), 0, 0) };
        if result == -1 {
            return Err(last_error("GetMessageW"));
        }

        if result == 0 {
            break;
        }

        // SAFETY: message contains a message just returned by GetMessageW.
        unsafe {
            TranslateMessage(&message);
            DispatchMessageW(&message);
        }
    }

    Ok(())
}

fn handle_main_command(hwnd: HWND, context: &mut WindowContext, wparam: WPARAM, lparam: LPARAM) {
    let command_id = low_word(wparam);
    let notification = high_word(wparam);

    if command_id == COMMAND_TAB_SELECTOR_CONTROL_ID as u32
        && notification == CBN_SELCHANGE
        && lparam as HWND == context.content.command_tabs.selector
    {
        update_selected_command_tab_from_selector(hwnd, context);
        return;
    }

    if let Some(index) = command_button_index_from_control_id(command_id) {
        handle_execute_command_button(hwnd, context, index);
        return;
    }

    if let Some(theme) = view_theme_for_command(command_id) {
        handle_theme_settings(hwnd, context, theme);
        return;
    }

    if let Some(language) = ui_language_for_command(command_id) {
        handle_ui_language_settings(hwnd, context, language);
        return;
    }

    match command_id {
        MENU_FILE_CLOSE_ID => handle_close_main_window(hwnd),
        MENU_TREE_ADD_ID => handle_add_workspace(hwnd, context),
        MENU_TREE_CATEGORY_ADD_ID => handle_add_category(hwnd, context),
        MENU_TREE_EDIT_ID => handle_edit_tree_item(hwnd, context),
        MENU_TREE_DELETE_ID => handle_delete_tree_item(hwnd, context),
        MENU_TREE_MOVE_UP_ID => {
            handle_move_tree_item(hwnd, context, TreeKeyboardMoveDirection::Up);
        }
        MENU_TREE_MOVE_DOWN_ID => {
            handle_move_tree_item(hwnd, context, TreeKeyboardMoveDirection::Down);
        }
        MENU_TABS_ADD_ID => handle_add_command_tab(hwnd, context),
        MENU_TABS_DELETE_ID => handle_delete_command_tab(hwnd, context),
        MENU_TABS_MOVE_LEFT_ID => {
            handle_move_command_tab(hwnd, context, CommandTabMoveDirection::Left);
        }
        MENU_TABS_MOVE_RIGHT_ID => {
            handle_move_command_tab(hwnd, context, CommandTabMoveDirection::Right);
        }
        MENU_TABS_RENAME_ID => handle_rename_command_tab(hwnd, context),
        MENU_COMMANDS_EXECUTE_ID => handle_execute_selected_command_button(hwnd, context),
        MENU_COMMANDS_ADD_ID => handle_add_command_button(hwnd, context),
        MENU_COMMANDS_DELETE_ID => handle_delete_command_button(hwnd, context),
        MENU_COMMANDS_MOVE_PREVIOUS_ID => {
            handle_move_command_button(hwnd, context, CommandButtonMoveDirection::Previous);
        }
        MENU_COMMANDS_MOVE_NEXT_ID => {
            handle_move_command_button(hwnd, context, CommandButtonMoveDirection::Next);
        }
        MENU_COMMANDS_EDIT_ID => handle_edit_command_button(hwnd, context),
        MENU_FILE_FONT_ID => handle_font_settings(hwnd, context),
        MENU_FILE_UI_LANGUAGE_ID => {}
        MENU_FILE_LANGUAGE_CONFIG_ID => handle_language_config(hwnd, context),
        MENU_FILE_ABOUT_ID => handle_about(hwnd, context),
        _ => {}
    }
}

fn handle_close_main_window(hwnd: HWND) {
    // SAFETY: hwnd is the main window handle owned by this UI thread. Sending WM_CLOSE follows the
    // same shutdown path as the title-bar close button.
    unsafe {
        SendMessageW(hwnd, WM_CLOSE, 0, 0);
    }
}

fn handle_about(hwnd: HWND, context: &WindowContext) {
    if let Err(error) = show_about_dialog(
        hwnd,
        context.instance,
        context.spec.title,
        APP_VERSION,
        context.ui_font.handle(),
        context.spec.state.settings().view.font_size,
        context_language(context),
    ) {
        show_error_message(
            hwnd,
            context_tr(context, "정보", "About"),
            &error.to_string(),
        );
    }
}

fn handle_add_category(hwnd: HWND, context: &mut WindowContext) {
    let result = show_text_input_dialog(
        hwnd,
        context.instance,
        TextInputDialogSpec::new(
            context_tr(context, "분류 추가", "Add Category"),
            context_tr(context, "새 분류 이름", "New category name"),
            "",
        ),
        context.ui_font.handle(),
        context.spec.state.settings().view.font_size,
        context_language(context),
    );

    match result {
        Ok(Some(name)) => match Category::new(name) {
            Ok(category) => {
                let previous_state = SettingsRestorePoint::capture(&context.spec.state);
                match context.spec.state.add_category(category) {
                    Ok(_) => {
                        if !persist_settings_or_restore(
                            hwnd,
                            &mut context.spec.state,
                            previous_state,
                        ) {
                            return;
                        }
                        refresh_tree_view(context);
                        update_tree_menu_state(hwnd, context);
                    }
                    Err(error) => show_error_message(
                        hwnd,
                        context_tr(context, "분류", "Category"),
                        &error.user_message_for_language(context_language(context)),
                    ),
                }
            }
            Err(error) => show_warning_message(
                hwnd,
                context_tr(context, "분류", "Category"),
                &error.user_message_for_language(context_language(context)),
            ),
        },
        Ok(None) => {}
        Err(error) => show_error_message(
            hwnd,
            context_tr(context, "분류", "Category"),
            &error.to_string(),
        ),
    }
}

fn handle_edit_tree_item(hwnd: HWND, context: &mut WindowContext) {
    let Some(selection) = selected_tree_node(context) else {
        show_warning_message(
            hwnd,
            context_tr(context, "워크스페이스", "Workspace"),
            context_tr(
                context,
                "편집할 항목을 선택하세요.",
                "Select an item to edit.",
            ),
        );
        update_tree_menu_state(hwnd, context);
        return;
    };

    handle_edit_tree_selection(hwnd, context, selection);
}

fn handle_edit_tree_selection(
    hwnd: HWND,
    context: &mut WindowContext,
    selection: TreeNodeSelection,
) {
    match selection {
        TreeNodeSelection::Workspace(index) => {
            context.spec.state.select_workspace(Some(index));
            handle_edit_workspace(hwnd, context);
        }
        TreeNodeSelection::Category(index) => {
            context.spec.state.select_workspace(None);
            handle_edit_category(hwnd, context, index);
        }
    }
}

fn handle_tree_item_double_click(hwnd: HWND, context: &mut WindowContext) -> bool {
    let Some(selection) = tree_node_at_current_message_position(context) else {
        return false;
    };

    handle_edit_tree_selection(hwnd, context, selection);
    true
}

fn handle_tree_context_menu(tree_view: HWND, context: &mut WindowContext, lparam: LPARAM) -> bool {
    if tree_view.is_null() {
        return false;
    }

    // SAFETY: tree_view is the TreeView child receiving this context-menu message.
    let parent = unsafe { GetParent(tree_view) };
    if parent.is_null() {
        return false;
    }

    let Some((item, selection, point)) = tree_context_menu_target(tree_view, lparam) else {
        return true;
    };

    select_tree_context_menu_target(tree_view, context, item, selection);
    update_tree_menu_state(parent, context);
    show_tree_context_menu(parent, context, point);
    true
}

fn tree_context_menu_target(
    tree_view: HWND,
    lparam: LPARAM,
) -> Option<(HTREEITEM, TreeNodeSelection, POINT)> {
    if lparam == -1 {
        let item = selected_tree_item(tree_view);
        let selection = tree_node_from_tree_item(tree_view, item)?;
        let point = tree_item_context_menu_point(tree_view, item)?;
        return Some((item, selection, point));
    }

    let screen_point = point_from_lparam(lparam);
    let mut tree_point = screen_point;
    // SAFETY: screen_point contains screen coordinates and tree_view is the target client window.
    let converted = unsafe { ScreenToClient(tree_view, &mut tree_point) };
    if converted == 0 {
        return None;
    }

    let item = tree_item_at_tree_point(tree_view, tree_point)?;
    let selection = tree_node_from_tree_item(tree_view, item)?;
    Some((item, selection, screen_point))
}

fn tree_item_context_menu_point(tree_view: HWND, item: HTREEITEM) -> Option<POINT> {
    let rect = tree_item_rect(tree_view, item)?;
    let mut point = POINT {
        x: rect.left + (rect.right - rect.left).max(0) / 2,
        y: rect.top + (rect.bottom - rect.top).max(0) / 2,
    };

    // SAFETY: point is in tree_view client coordinates and is writable for conversion.
    let converted = unsafe { ClientToScreen(tree_view, &mut point) };
    (converted != 0).then_some(point)
}

fn select_tree_context_menu_target(
    tree_view: HWND,
    context: &mut WindowContext,
    item: HTREEITEM,
    selection: TreeNodeSelection,
) {
    select_tree_item(tree_view, item);
    match selection {
        TreeNodeSelection::Workspace(index) => context.spec.state.select_workspace(Some(index)),
        TreeNodeSelection::Category(_) => context.spec.state.select_workspace(None),
    }

    // SAFETY: tree_view is the control the user invoked; focusing it keeps keyboard follow-up
    // actions on the same selected node.
    unsafe {
        SetFocus(tree_view);
    }
}

fn show_tree_context_menu(hwnd: HWND, context: &mut WindowContext, point: POINT) {
    let popup = match create_tree_context_menu(context) {
        Ok(popup) => popup,
        Err(error) => {
            show_error_message(
                hwnd,
                context_tr(context, "워크스페이스", "Workspace"),
                &error.to_string(),
            );
            return;
        }
    };

    // SAFETY: popup is a valid menu handle. TPM_RETURNCMD returns the selected command ID instead
    // of sending WM_COMMAND, letting this function destroy the temporary menu before dispatch.
    let command = unsafe {
        TrackPopupMenu(
            popup,
            TPM_LEFTALIGN | TPM_TOPALIGN | TPM_RIGHTBUTTON | TPM_RETURNCMD,
            point.x,
            point.y,
            0,
            hwnd,
            null(),
        )
    };

    // SAFETY: popup was created for this one context-menu invocation and is not attached to a
    // window menu bar.
    unsafe {
        DestroyMenu(popup);
    }

    if command != 0 {
        handle_main_command(hwnd, context, command as WPARAM, 0);
    }
}

fn create_tree_context_menu(context: &WindowContext) -> AppResult<HMENU> {
    // SAFETY: CreatePopupMenu has no preconditions.
    let popup = unsafe { CreatePopupMenu() };
    if popup.is_null() {
        return Err(last_error("CreatePopupMenu tree context"));
    }

    let state = current_tree_menu_state(context);
    let result = (|| -> AppResult<()> {
        append_context_menu_command(
            popup,
            context_tr(context, "편집", "Edit"),
            MENU_TREE_EDIT_ID,
            state.can_edit_tree_item,
        )?;
        append_context_menu_separator(popup)?;
        append_context_menu_command(
            popup,
            context_tr(context, "위로", "Move Up"),
            MENU_TREE_MOVE_UP_ID,
            state.can_move_up,
        )?;
        append_context_menu_command(
            popup,
            context_tr(context, "아래로", "Move Down"),
            MENU_TREE_MOVE_DOWN_ID,
            state.can_move_down,
        )?;
        append_context_menu_separator(popup)?;
        append_context_menu_command(
            popup,
            context_tr(context, "워크스페이스 추가", "Add Workspace"),
            MENU_TREE_ADD_ID,
            true,
        )?;
        append_context_menu_command(
            popup,
            context_tr(context, "분류 추가", "Add Category"),
            MENU_TREE_CATEGORY_ADD_ID,
            true,
        )?;
        append_context_menu_separator(popup)?;
        append_context_menu_command(
            popup,
            context_tr(context, "삭제", "Delete"),
            MENU_TREE_DELETE_ID,
            state.can_delete_tree_item,
        )?;
        Ok(())
    })();

    if let Err(error) = result {
        // SAFETY: popup is not attached to another menu when item creation fails.
        unsafe {
            DestroyMenu(popup);
        }
        Err(error)
    } else {
        Ok(popup)
    }
}

fn handle_command_button_context_menu(
    button: HWND,
    context: &mut WindowContext,
    lparam: LPARAM,
) -> bool {
    let Some(index) = command_button_index_from_handle(context, button) else {
        return false;
    };
    let owner = command_button_owner(button, context);
    if owner.is_null() {
        return false;
    }
    let Some(point) = command_button_context_menu_point(button, lparam) else {
        return false;
    };

    cancel_command_button_drag(context);
    context.spec.state.select_command_button(Some(index));
    update_commands_menu_state(owner, context);

    // SAFETY: button is the command button that received the context-menu message.
    unsafe {
        SetFocus(button);
    }

    show_command_button_context_menu(owner, context, point);
    true
}

fn command_button_context_menu_point(button: HWND, lparam: LPARAM) -> Option<POINT> {
    if lparam != -1 {
        return Some(point_from_lparam(lparam));
    }

    let mut rect = RECT::default();
    // SAFETY: button is the command button receiving the keyboard context-menu request.
    let resolved = unsafe { GetWindowRect(button, &mut rect) };
    (resolved != 0).then_some(POINT {
        x: rect.left + (rect.right - rect.left).max(0) / 2,
        y: rect.top + (rect.bottom - rect.top).max(0) / 2,
    })
}

fn show_command_button_context_menu(hwnd: HWND, context: &mut WindowContext, point: POINT) {
    let popup = match create_command_button_context_menu(context) {
        Ok(popup) => popup,
        Err(error) => {
            show_error_message(
                hwnd,
                context_tr(context, "명령", "Command"),
                &error.to_string(),
            );
            return;
        }
    };

    // SAFETY: popup is a valid menu handle. TPM_RETURNCMD returns the selected command ID instead
    // of sending WM_COMMAND, letting this function destroy the temporary menu before dispatch.
    let command = unsafe {
        TrackPopupMenu(
            popup,
            TPM_LEFTALIGN | TPM_TOPALIGN | TPM_RIGHTBUTTON | TPM_RETURNCMD,
            point.x,
            point.y,
            0,
            hwnd,
            null(),
        )
    };

    // SAFETY: popup was created for this one context-menu invocation and is not attached to a
    // window menu bar.
    unsafe {
        DestroyMenu(popup);
    }

    if command != 0 {
        handle_main_command(hwnd, context, command as WPARAM, 0);
    }
}

fn create_command_button_context_menu(context: &WindowContext) -> AppResult<HMENU> {
    // SAFETY: CreatePopupMenu has no preconditions.
    let popup = unsafe { CreatePopupMenu() };
    if popup.is_null() {
        return Err(last_error("CreatePopupMenu command button context"));
    }

    let state = current_command_button_context_menu_state(context);
    let result = (|| -> AppResult<()> {
        append_context_menu_command(
            popup,
            context_tr(context, "실행", "Run"),
            MENU_COMMANDS_EXECUTE_ID,
            state.can_execute,
        )?;
        append_context_menu_separator(popup)?;
        append_context_menu_command(
            popup,
            context_tr(context, "편집", "Edit"),
            MENU_COMMANDS_EDIT_ID,
            state.can_edit,
        )?;
        append_context_menu_command(
            popup,
            context_tr(context, "앞으로", "Previous"),
            MENU_COMMANDS_MOVE_PREVIOUS_ID,
            state.can_move_previous,
        )?;
        append_context_menu_command(
            popup,
            context_tr(context, "뒤로", "Next"),
            MENU_COMMANDS_MOVE_NEXT_ID,
            state.can_move_next,
        )?;
        append_context_menu_separator(popup)?;
        append_context_menu_command(
            popup,
            context_tr(context, "명령 추가", "Add Command"),
            MENU_COMMANDS_ADD_ID,
            state.can_add_command,
        )?;
        append_context_menu_separator(popup)?;
        append_context_menu_command(
            popup,
            context_tr(context, "삭제", "Delete"),
            MENU_COMMANDS_DELETE_ID,
            state.can_delete,
        )?;
        Ok(())
    })();

    if let Err(error) = result {
        // SAFETY: popup is not attached to another menu when item creation fails.
        unsafe {
            DestroyMenu(popup);
        }
        Err(error)
    } else {
        Ok(popup)
    }
}

fn current_command_button_context_menu_state(
    context: &WindowContext,
) -> CommandButtonContextMenuState {
    let has_selected_tab = context.spec.state.selected_command_tab().is_some();
    let selected_button_count = context
        .spec
        .state
        .selected_command_tab()
        .map(|tab| tab.buttons.len())
        .unwrap_or_default();
    command_button_context_menu_state(
        has_selected_tab,
        context.spec.state.selected_command_button().is_some(),
        context.spec.state.selected_command_button_index(),
        selected_button_count,
        selected_tree_node(context),
    )
}

fn command_button_context_menu_state(
    has_selected_tab: bool,
    has_selected_button: bool,
    selected_button_index: Option<usize>,
    selected_button_count: usize,
    tree_selection: Option<TreeNodeSelection>,
) -> CommandButtonContextMenuState {
    let can_target_button = has_selected_tab && has_selected_button;
    CommandButtonContextMenuState {
        can_execute: can_target_button
            && !command_button_click_ignored_for_tree_selection(tree_selection),
        can_edit: can_target_button,
        can_delete: can_target_button,
        can_move_previous: can_target_button
            && selected_button_index
                .and_then(|index| {
                    layout_rules::command_button_move_destination(
                        index,
                        selected_button_count,
                        CommandButtonMoveDirection::Previous,
                    )
                })
                .is_some(),
        can_move_next: can_target_button
            && selected_button_index
                .and_then(|index| {
                    layout_rules::command_button_move_destination(
                        index,
                        selected_button_count,
                        CommandButtonMoveDirection::Next,
                    )
                })
                .is_some(),
        can_add_command: has_selected_tab,
    }
}

fn append_context_menu_command(
    popup: HMENU,
    label: &str,
    command_id: u32,
    enabled: bool,
) -> AppResult<()> {
    let flags = if enabled { MF_ENABLED } else { MF_GRAYED };
    let label = wide_null(label);
    // SAFETY: popup is a valid menu handle and label is null-terminated for this call.
    let appended = unsafe { AppendMenuW(popup, flags, command_id as usize, label.as_ptr()) };
    if appended == 0 {
        Err(last_error("AppendMenuW context menu item"))
    } else {
        Ok(())
    }
}

fn append_context_menu_separator(popup: HMENU) -> AppResult<()> {
    // SAFETY: popup is a valid menu handle. Separators do not use an item ID or label pointer.
    let appended = unsafe { AppendMenuW(popup, MF_SEPARATOR, 0, null()) };
    if appended == 0 {
        Err(last_error("AppendMenuW context menu separator"))
    } else {
        Ok(())
    }
}

fn handle_edit_category(hwnd: HWND, context: &mut WindowContext, index: usize) {
    let Some(category) = context.spec.state.settings().categories.get(index).cloned() else {
        update_tree_menu_state(hwnd, context);
        show_warning_message(
            hwnd,
            context_tr(context, "분류", "Category"),
            context_tr(
                context,
                "선택한 분류를 찾을 수 없습니다.",
                "The selected category could not be found.",
            ),
        );
        return;
    };

    let result = show_text_input_dialog(
        hwnd,
        context.instance,
        TextInputDialogSpec::new(
            context_tr(context, "분류 편집", "Edit Category"),
            context_tr(context, "분류 이름", "Category name"),
            &category.name,
        ),
        context.ui_font.handle(),
        context.spec.state.settings().view.font_size,
        context_language(context),
    );

    match result {
        Ok(Some(name)) => match Category::new(name) {
            Ok(category) => {
                let previous_state = SettingsRestorePoint::capture(&context.spec.state);
                match context.spec.state.rename_category(index, category) {
                    Ok(()) => {
                        if !persist_settings_or_restore(
                            hwnd,
                            &mut context.spec.state,
                            previous_state,
                        ) {
                            return;
                        }
                        refresh_tree_view(context);
                        select_category_tree_item(context, index);
                        update_tree_menu_state(hwnd, context);
                    }
                    Err(error) => show_error_message(
                        hwnd,
                        context_tr(context, "분류", "Category"),
                        &error.user_message_for_language(context_language(context)),
                    ),
                }
            }
            Err(error) => show_warning_message(
                hwnd,
                context_tr(context, "분류", "Category"),
                &error.user_message_for_language(context_language(context)),
            ),
        },
        Ok(None) => {}
        Err(error) => show_error_message(
            hwnd,
            context_tr(context, "분류", "Category"),
            &error.to_string(),
        ),
    }
}

fn handle_add_workspace(hwnd: HWND, context: &mut WindowContext) {
    let reserved_paths = workspace_paths_except(context, None);
    let result = show_workspace_dialog(
        hwnd,
        context.instance,
        WorkspaceDialogMode::Add,
        None,
        reserved_paths,
        context.spec.state.settings().languages.clone(),
        context.ui_font.handle(),
        context.spec.state.settings().view.font_size,
        context_language(context),
    );

    match result {
        Ok(Some(workspace)) => {
            let previous_state = SettingsRestorePoint::capture(&context.spec.state);
            match context.spec.state.add_workspace(workspace) {
                Ok(_) => {
                    if !persist_settings_or_restore(hwnd, &mut context.spec.state, previous_state) {
                        return;
                    }
                    refresh_tree_view(context);
                    update_tree_menu_state(hwnd, context);
                }
                Err(error) => show_error_message(
                    hwnd,
                    context_tr(context, "워크스페이스", "Workspace"),
                    &error.user_message_for_language(context_language(context)),
                ),
            }
        }
        Ok(None) => {}
        Err(error) => show_error_message(
            hwnd,
            context_tr(context, "워크스페이스", "Workspace"),
            &error.to_string(),
        ),
    }
}

fn handle_edit_workspace(hwnd: HWND, context: &mut WindowContext) {
    let Some(index) = context.spec.state.selected_workspace_index() else {
        show_warning_message(
            hwnd,
            context_tr(context, "워크스페이스", "Workspace"),
            context_tr(
                context,
                "편집할 워크스페이스를 선택하세요.",
                "Select a workspace to edit.",
            ),
        );
        update_tree_menu_state(hwnd, context);
        return;
    };

    let Some(workspace) = context.spec.state.selected_workspace().cloned() else {
        context.spec.state.select_workspace(None);
        update_tree_menu_state(hwnd, context);
        show_warning_message(
            hwnd,
            context_tr(context, "워크스페이스", "Workspace"),
            context_tr(
                context,
                "선택한 워크스페이스를 찾을 수 없습니다.",
                "The selected workspace could not be found.",
            ),
        );
        return;
    };

    let reserved_paths = workspace_paths_except(context, Some(index));
    let result = show_workspace_dialog(
        hwnd,
        context.instance,
        WorkspaceDialogMode::Edit,
        Some(workspace),
        reserved_paths,
        context.spec.state.settings().languages.clone(),
        context.ui_font.handle(),
        context.spec.state.settings().view.font_size,
        context_language(context),
    );

    match result {
        Ok(Some(workspace)) => {
            let previous_state = SettingsRestorePoint::capture(&context.spec.state);
            match context.spec.state.update_workspace(index, workspace) {
                Ok(()) => {
                    if !persist_settings_or_restore(hwnd, &mut context.spec.state, previous_state) {
                        return;
                    }
                    refresh_tree_view(context);
                    update_tree_menu_state(hwnd, context);
                }
                Err(error) => show_error_message(
                    hwnd,
                    context_tr(context, "워크스페이스", "Workspace"),
                    &error.user_message_for_language(context_language(context)),
                ),
            }
        }
        Ok(None) => {}
        Err(error) => show_error_message(
            hwnd,
            context_tr(context, "워크스페이스", "Workspace"),
            &error.to_string(),
        ),
    }
}

fn handle_delete_tree_item(hwnd: HWND, context: &mut WindowContext) {
    let Some(selection) = selected_tree_node(context) else {
        show_warning_message(
            hwnd,
            context_tr(context, "워크스페이스", "Workspace"),
            context_tr(
                context,
                "삭제할 항목을 선택하세요.",
                "Select an item to delete.",
            ),
        );
        update_tree_menu_state(hwnd, context);
        return;
    };

    match selection {
        TreeNodeSelection::Workspace(index) => {
            context.spec.state.select_workspace(Some(index));
            handle_delete_workspace(hwnd, context);
        }
        TreeNodeSelection::Category(index) => {
            context.spec.state.select_workspace(None);
            handle_delete_category(hwnd, context, index);
        }
    }
}

fn handle_delete_workspace(hwnd: HWND, context: &mut WindowContext) {
    let Some(index) = context.spec.state.selected_workspace_index() else {
        show_warning_message(
            hwnd,
            context_tr(context, "워크스페이스", "Workspace"),
            context_tr(
                context,
                "삭제할 워크스페이스를 선택하세요.",
                "Select a workspace to delete.",
            ),
        );
        update_tree_menu_state(hwnd, context);
        return;
    };

    let Some(workspace) = context.spec.state.selected_workspace().cloned() else {
        context.spec.state.select_workspace(None);
        update_tree_menu_state(hwnd, context);
        show_warning_message(
            hwnd,
            context_tr(context, "워크스페이스", "Workspace"),
            context_tr(
                context,
                "선택한 워크스페이스를 찾을 수 없습니다.",
                "The selected workspace could not be found.",
            ),
        );
        return;
    };

    let message = match context_language(context) {
        UiLanguage::Korean => format!(
            "워크스페이스를 삭제할까요?\n\n이름: {}\n폴더: {}",
            workspace.name, workspace.path
        ),
        UiLanguage::English => format!(
            "Delete this workspace?\n\nName: {}\nFolder: {}",
            workspace.name, workspace.path
        ),
    };

    if !confirm_destructive_action(
        hwnd,
        context_tr(context, "워크스페이스 삭제", "Delete Workspace"),
        &message,
    ) {
        return;
    }

    let previous_state = SettingsRestorePoint::capture(&context.spec.state);
    if context.spec.state.delete_workspace(index).is_some() {
        if !persist_settings_or_restore(hwnd, &mut context.spec.state, previous_state) {
            return;
        }
        refresh_tree_view(context);
        update_tree_menu_state(hwnd, context);
    }
}

fn handle_delete_category(hwnd: HWND, context: &mut WindowContext, index: usize) {
    let Some(category) = context.spec.state.settings().categories.get(index).cloned() else {
        update_tree_menu_state(hwnd, context);
        show_warning_message(
            hwnd,
            context_tr(context, "분류", "Category"),
            context_tr(
                context,
                "선택한 분류를 찾을 수 없습니다.",
                "The selected category could not be found.",
            ),
        );
        return;
    };

    let workspace_count = context
        .spec
        .state
        .settings()
        .workspaces
        .iter()
        .filter(|workspace| workspace_belongs_to_category(workspace, &category))
        .count();
    let message = match context_language(context) {
        UiLanguage::Korean => format!(
            "분류를 삭제할까요?\n\n이름: {}\n소속 워크스페이스: {}개\n\n워크스페이스는 삭제하지 않고 최상위로 이동합니다.",
            category.name, workspace_count
        ),
        UiLanguage::English => format!(
            "Delete this category?\n\nName: {}\nWorkspaces: {}\n\nWorkspaces will not be deleted and will move to the top level.",
            category.name, workspace_count
        ),
    };

    if !confirm_destructive_action(
        hwnd,
        context_tr(context, "분류 삭제", "Delete Category"),
        &message,
    ) {
        return;
    }

    let previous_state = SettingsRestorePoint::capture(&context.spec.state);
    if context.spec.state.delete_category(index).is_some() {
        if !persist_settings_or_restore(hwnd, &mut context.spec.state, previous_state) {
            return;
        }
        refresh_tree_view(context);
        update_tree_menu_state(hwnd, context);
    }
}

fn handle_move_tree_item(
    hwnd: HWND,
    context: &mut WindowContext,
    direction: TreeKeyboardMoveDirection,
) {
    let Some(selection) = selected_tree_node(context) else {
        show_warning_message(
            hwnd,
            context_tr(context, "워크스페이스", "Workspace"),
            context_tr(
                context,
                "이동할 항목을 선택하세요.",
                "Select an item to move.",
            ),
        );
        update_tree_menu_state(hwnd, context);
        return;
    };

    match selection {
        TreeNodeSelection::Workspace(index) => {
            context.spec.state.select_workspace(Some(index));
            if !handle_keyboard_move_workspace(hwnd, context, direction) {
                show_warning_message(
                    hwnd,
                    context_tr(context, "워크스페이스", "Workspace"),
                    context_tr(
                        context,
                        "이동할 워크스페이스를 선택하세요.",
                        "Select a workspace to move.",
                    ),
                );
            }
        }
        TreeNodeSelection::Category(index) => {
            context.spec.state.select_workspace(None);
            handle_keyboard_move_category(hwnd, context, index, direction);
        }
    }

    update_tree_menu_state(hwnd, context);
}

fn handle_keyboard_move_workspace(
    hwnd: HWND,
    context: &mut WindowContext,
    direction: TreeKeyboardMoveDirection,
) -> bool {
    let Some(index) = context.spec.state.selected_workspace_index() else {
        return false;
    };

    let settings = context.spec.state.settings();
    let Some(workspace) = settings.workspaces.get(index) else {
        return false;
    };

    if layout_rules::workspace_category_index(workspace, settings.categories.as_slice()).is_none() {
        let root_items = settings.root_tree_items();
        let Some(destination_index) = layout_rules::tree_root_keyboard_move_destination(
            root_items.as_slice(),
            TreeRootItemRef::Workspace(index),
            direction,
        ) else {
            return true;
        };

        let previous_state = SettingsRestorePoint::capture(&context.spec.state);
        match context
            .spec
            .state
            .move_root_tree_item(TreeRootItemRef::Workspace(index), destination_index)
        {
            Ok(()) => {
                if !persist_settings_or_restore(hwnd, &mut context.spec.state, previous_state) {
                    return true;
                }
                refresh_tree_view(context);
                update_tree_menu_state(hwnd, context);
            }
            Err(error) => show_error_message(
                hwnd,
                context_tr(context, "워크스페이스", "Workspace"),
                &error.user_message_for_language(context_language(context)),
            ),
        }

        return true;
    }

    let Some(destination_index) = layout_rules::workspace_keyboard_move_destination(
        settings.workspaces.as_slice(),
        settings.categories.as_slice(),
        index,
        direction,
    ) else {
        return true;
    };

    let previous_state = SettingsRestorePoint::capture(&context.spec.state);
    match context.spec.state.move_workspace(index, destination_index) {
        Ok(()) => {
            if !persist_settings_or_restore(hwnd, &mut context.spec.state, previous_state) {
                return true;
            }
            refresh_tree_view(context);
            update_tree_menu_state(hwnd, context);
        }
        Err(error) => show_error_message(
            hwnd,
            context_tr(context, "워크스페이스", "Workspace"),
            &error.user_message_for_language(context_language(context)),
        ),
    }

    true
}

fn handle_keyboard_move_workspace_to_root(hwnd: HWND, context: &mut WindowContext) -> bool {
    let Some(index) = context.spec.state.selected_workspace_index() else {
        return false;
    };

    let workspace_is_in_category = {
        let settings = context.spec.state.settings();
        let Some(workspace) = settings.workspaces.get(index) else {
            return false;
        };
        layout_rules::workspace_category_index(workspace, settings.categories.as_slice()).is_some()
    };

    if !workspace_is_in_category {
        return true;
    }

    let previous_state = SettingsRestorePoint::capture(&context.spec.state);
    match context.spec.state.move_workspace_to_root(index) {
        Ok(()) => {
            if !persist_settings_or_restore(hwnd, &mut context.spec.state, previous_state) {
                return true;
            }
            refresh_tree_view(context);
            update_tree_menu_state(hwnd, context);
        }
        Err(error) => show_error_message(
            hwnd,
            context_tr(context, "워크스페이스", "Workspace"),
            &error.user_message_for_language(context_language(context)),
        ),
    }

    true
}

fn handle_keyboard_move_category(
    hwnd: HWND,
    context: &mut WindowContext,
    index: usize,
    direction: TreeKeyboardMoveDirection,
) -> bool {
    let settings = context.spec.state.settings();
    let root_items = settings.root_tree_items();
    let Some(destination_index) = layout_rules::tree_root_keyboard_move_destination(
        root_items.as_slice(),
        TreeRootItemRef::Category(index),
        direction,
    ) else {
        return true;
    };

    let previous_state = SettingsRestorePoint::capture(&context.spec.state);
    match context
        .spec
        .state
        .move_root_tree_item(TreeRootItemRef::Category(index), destination_index)
    {
        Ok(()) => {
            if !persist_settings_or_restore(hwnd, &mut context.spec.state, previous_state) {
                return true;
            }
            refresh_tree_view(context);
            select_category_tree_item(context, index);
            update_tree_menu_state(hwnd, context);
        }
        Err(error) => show_error_message(
            hwnd,
            context_tr(context, "분류", "Category"),
            &error.user_message_for_language(context_language(context)),
        ),
    }

    true
}

fn handle_font_settings(hwnd: HWND, context: &mut WindowContext) {
    let current = context.spec.state.settings().view.clone();
    let installed_fonts = installed_font_families();
    let result = show_font_dialog(
        hwnd,
        context.instance,
        &current,
        installed_fonts,
        context.ui_font.handle(),
        context.dpi,
        context_language(context),
    );

    match result {
        Ok(Some(view)) => apply_view_settings(hwnd, context, view),
        Ok(None) => {}
        Err(error) => show_error_message(
            hwnd,
            context_tr(context, "글꼴", "Font"),
            &error.to_string(),
        ),
    }
}

fn handle_language_config(hwnd: HWND, context: &mut WindowContext) {
    let current = context.spec.state.settings().languages.clone();
    let result = show_language_config_dialog(
        hwnd,
        context.instance,
        current,
        context.ui_font.handle(),
        context.spec.state.settings().view.font_size,
        context_language(context),
    );

    match result {
        Ok(Some(languages)) => apply_language_config(hwnd, context, languages),
        Ok(None) => {}
        Err(error) => show_error_message(
            hwnd,
            context_tr(context, "언어", "Workspace Languages"),
            &error.to_string(),
        ),
    }
}

fn apply_language_config(hwnd: HWND, context: &mut WindowContext, languages: Vec<String>) {
    let previous_state = SettingsRestorePoint::capture(&context.spec.state);
    match context.spec.state.set_workspace_languages(languages) {
        Ok(()) => {
            if !persist_settings_or_restore(hwnd, &mut context.spec.state, previous_state) {
                return;
            }
            refresh_tree_view(context);
            update_tree_menu_state(hwnd, context);
        }
        Err(error) => show_warning_message(
            hwnd,
            context_tr(context, "언어", "Workspace Languages"),
            &error.user_message_for_language(context_language(context)),
        ),
    }
}

fn handle_theme_settings(hwnd: HWND, context: &mut WindowContext, theme: ViewTheme) {
    let current = context.spec.state.settings().view.clone();
    if current_view_theme(&current) == theme {
        return;
    }

    let view = ViewSettings::new(
        current.font_family.clone(),
        current.font_size,
        theme.as_config_value(),
    )
    .with_ui_language(&current.ui_language)
    .with_window_layout_from(&current);
    apply_view_settings(hwnd, context, view);
}

fn handle_ui_language_settings(hwnd: HWND, context: &mut WindowContext, language: UiLanguage) {
    let current = context.spec.state.settings().view.clone();
    if current_ui_language(&current) == language {
        return;
    }

    let view = ViewSettings::new(
        current.font_family.clone(),
        current.font_size,
        current.theme.clone(),
    )
    .with_ui_language(language.as_config_value())
    .with_window_layout_from(&current);
    apply_view_settings(hwnd, context, view);
}

fn apply_view_settings(hwnd: HWND, context: &mut WindowContext, view: ViewSettings) {
    let current = context.spec.state.settings().view.clone();
    let view = ViewSettings::new(view.font_family, view.font_size, view.theme)
        .with_ui_language(view.ui_language)
        .with_window_layout_from(&current);
    if context.spec.state.settings().view == view {
        return;
    }

    let theme = current_view_theme(&view);
    let new_theme_resources = match ThemeResources::new(theme) {
        Ok(resources) => resources,
        Err(error) => {
            show_error_message(
                hwnd,
                context_tr(context, "테마", "Theme"),
                &error.to_string(),
            );
            return;
        }
    };
    let previous_state = SettingsRestorePoint::capture(&context.spec.state);
    let previous_menus = context.spec.menus;
    context.spec.state.set_view_settings(view.clone());
    context.spec.menus = main_menu_for_language(current_ui_language(&view));
    if !persist_settings_or_restore(hwnd, &mut context.spec.state, previous_state) {
        context.spec.menus = previous_menus;
        return;
    }

    let new_font = UiFont::from_view(&view, context.dpi);
    let old_font = std::mem::replace(&mut context.ui_font, new_font);
    let old_theme_resources = std::mem::replace(&mut context.theme_resources, new_theme_resources);
    let tree_panel_width = context.spec.layout.tree_panel_width;
    context.spec.layout = LayoutSpec::for_font_size_and_dpi(view.font_size, context.dpi);
    context.spec.layout.tree_panel_width = tree_panel_width;
    rebuild_menu_bar(hwnd, context);
    apply_ui_font_to_main(hwnd, context);
    resize_main_content(hwnd, context);
    refresh_command_tab_selector(context);
    refresh_command_buttons(hwnd, context);
    apply_window_theme(hwnd, context);

    drop(old_font);
    drop(old_theme_resources);
}

fn rebuild_menu_bar(hwnd: HWND, context: &WindowContext) {
    let Ok(menu) = create_menu_bar(
        context.spec.menus,
        current_view_theme(&context.spec.state.settings().view),
        current_ui_language(&context.spec.state.settings().view),
    ) else {
        show_error_message(
            hwnd,
            context_tr(context, "메뉴", "Menu"),
            context_tr(
                context,
                "메뉴 글꼴을 다시 계산하지 못했습니다.",
                "Could not recalculate the menu font.",
            ),
        );
        return;
    };

    // SAFETY: hwnd is the main window. SetMenu attaches the new menu; the old menu is no longer
    // attached and can be destroyed after SetMenu succeeds.
    unsafe {
        let old_menu = GetMenu(hwnd);
        if SetMenu(hwnd, menu) == 0 {
            DestroyMenu(menu);
            show_error_message(
                hwnd,
                context_tr(context, "메뉴", "Menu"),
                context_tr(
                    context,
                    "메뉴를 다시 구성하지 못했습니다.",
                    "Could not rebuild the menu.",
                ),
            );
            return;
        }

        if !old_menu.is_null() {
            DestroyMenu(old_menu);
        }
    }

    update_tree_menu_state(hwnd, context);
    update_tabs_menu_state(hwnd, context);
    update_commands_menu_state(hwnd, context);
}

fn handle_add_command_tab(hwnd: HWND, context: &mut WindowContext) {
    let result = show_text_input_dialog(
        hwnd,
        context.instance,
        TextInputDialogSpec::new(
            context_tr(context, "그룹 추가", "Add Group"),
            context_tr(context, "그룹 이름", "Group name"),
            "",
        ),
        context.ui_font.handle(),
        context.spec.state.settings().view.font_size,
        context_language(context),
    );

    match result {
        Ok(Some(name)) => match CommandTab::new(name, Vec::new()) {
            Ok(tab) => {
                let previous_state = SettingsRestorePoint::capture(&context.spec.state);
                match context.spec.state.add_command_tab(tab) {
                    Ok(_) => {
                        if !persist_settings_or_restore(
                            hwnd,
                            &mut context.spec.state,
                            previous_state,
                        ) {
                            return;
                        }
                        refresh_command_tab_selector(context);
                        refresh_command_buttons(hwnd, context);
                        update_tabs_menu_state(hwnd, context);
                        update_commands_menu_state(hwnd, context);
                    }
                    Err(error) => show_error_message(
                        hwnd,
                        context_tr(context, "명령 그룹", "Command Group"),
                        &error.user_message_for_language(context_language(context)),
                    ),
                }
            }
            Err(error) => show_warning_message(
                hwnd,
                context_tr(context, "명령 그룹", "Command Group"),
                &error.user_message_for_language(context_language(context)),
            ),
        },
        Ok(None) => {}
        Err(error) => show_error_message(
            hwnd,
            context_tr(context, "명령 그룹", "Command Group"),
            &error.to_string(),
        ),
    }
}

fn handle_rename_command_tab(hwnd: HWND, context: &mut WindowContext) {
    let Some(index) = context.spec.state.selected_command_tab_index() else {
        show_warning_message(
            hwnd,
            context_tr(context, "명령 그룹", "Command Group"),
            context_tr(
                context,
                "이름을 바꿀 그룹을 선택하세요.",
                "Select a group to rename.",
            ),
        );
        update_tabs_menu_state(hwnd, context);
        return;
    };

    let Some(tab) = context.spec.state.selected_command_tab().cloned() else {
        context.spec.state.select_command_tab(None);
        update_tabs_menu_state(hwnd, context);
        show_warning_message(
            hwnd,
            context_tr(context, "명령 그룹", "Command Group"),
            context_tr(
                context,
                "선택한 명령 그룹을 찾을 수 없습니다.",
                "The selected command group could not be found.",
            ),
        );
        return;
    };

    let result = show_text_input_dialog(
        hwnd,
        context.instance,
        TextInputDialogSpec::new(
            context_tr(context, "그룹 이름 변경", "Rename Group"),
            context_tr(context, "그룹 이름", "Group name"),
            &tab.name,
        ),
        context.ui_font.handle(),
        context.spec.state.settings().view.font_size,
        context_language(context),
    );

    match result {
        Ok(Some(name)) => {
            let previous_state = SettingsRestorePoint::capture(&context.spec.state);
            match context.spec.state.rename_command_tab(index, name) {
                Ok(()) => {
                    if !persist_settings_or_restore(hwnd, &mut context.spec.state, previous_state) {
                        return;
                    }
                    refresh_command_tab_selector(context);
                    refresh_command_buttons(hwnd, context);
                    update_tabs_menu_state(hwnd, context);
                    update_commands_menu_state(hwnd, context);
                }
                Err(error) => show_error_message(
                    hwnd,
                    context_tr(context, "명령 그룹", "Command Group"),
                    &error.user_message_for_language(context_language(context)),
                ),
            }
        }
        Ok(None) => {}
        Err(error) => show_error_message(
            hwnd,
            context_tr(context, "명령 그룹", "Command Group"),
            &error.to_string(),
        ),
    }
}

fn handle_delete_command_tab(hwnd: HWND, context: &mut WindowContext) {
    let Some(index) = context.spec.state.selected_command_tab_index() else {
        show_warning_message(
            hwnd,
            context_tr(context, "명령 그룹", "Command Group"),
            context_tr(
                context,
                "삭제할 명령 그룹을 선택하세요.",
                "Select a command group to delete.",
            ),
        );
        update_tabs_menu_state(hwnd, context);
        return;
    };

    let Some(tab) = context.spec.state.selected_command_tab().cloned() else {
        context.spec.state.select_command_tab(None);
        update_tabs_menu_state(hwnd, context);
        show_warning_message(
            hwnd,
            context_tr(context, "명령 그룹", "Command Group"),
            context_tr(
                context,
                "선택한 명령 그룹을 찾을 수 없습니다.",
                "The selected command group could not be found.",
            ),
        );
        return;
    };

    let message = match context_language(context) {
        UiLanguage::Korean => format!(
            "명령 그룹을 삭제할까요?\n\n이름: {}\n명령: {}개\n포함된 명령도 삭제됩니다.",
            tab.name,
            tab.buttons.len()
        ),
        UiLanguage::English => format!(
            "Delete this command group?\n\nName: {}\nCommands: {}\nIncluded commands will also be deleted.",
            tab.name,
            tab.buttons.len()
        ),
    };

    if !confirm_destructive_action(
        hwnd,
        context_tr(context, "명령 그룹 삭제", "Delete Command Group"),
        &message,
    ) {
        return;
    }

    let previous_state = SettingsRestorePoint::capture(&context.spec.state);
    if context.spec.state.delete_command_tab(index).is_some() {
        if !persist_settings_or_restore(hwnd, &mut context.spec.state, previous_state) {
            return;
        }
        refresh_command_tab_selector(context);
        refresh_command_buttons(hwnd, context);
        update_tabs_menu_state(hwnd, context);
        update_commands_menu_state(hwnd, context);
    }
}

fn handle_move_command_tab(
    hwnd: HWND,
    context: &mut WindowContext,
    direction: CommandTabMoveDirection,
) {
    let Some(index) = context.spec.state.selected_command_tab_index() else {
        show_warning_message(
            hwnd,
            context_tr(context, "명령 그룹", "Command Group"),
            context_tr(
                context,
                "이동할 명령 그룹을 선택하세요.",
                "Select a command group to move.",
            ),
        );
        update_tabs_menu_state(hwnd, context);
        return;
    };

    let tab_count = context.spec.state.settings().command_tabs.len();
    let Some(destination_index) =
        layout_rules::command_tab_move_destination(index, tab_count, direction)
    else {
        show_warning_message(
            hwnd,
            context_tr(context, "명령 그룹", "Command Group"),
            context_tr(
                context,
                "더 이동할 수 없습니다.",
                "Cannot move any further.",
            ),
        );
        update_tabs_menu_state(hwnd, context);
        return;
    };

    let previous_state = SettingsRestorePoint::capture(&context.spec.state);
    match context
        .spec
        .state
        .move_command_tab(index, destination_index)
    {
        Ok(()) => {
            if !persist_settings_or_restore(hwnd, &mut context.spec.state, previous_state) {
                return;
            }
            refresh_command_tab_selector(context);
            refresh_command_buttons(hwnd, context);
            update_tabs_menu_state(hwnd, context);
            update_commands_menu_state(hwnd, context);
        }
        Err(error) => show_error_message(
            hwnd,
            context_tr(context, "명령 그룹", "Command Group"),
            &error.user_message_for_language(context_language(context)),
        ),
    }
}

fn handle_select_command_button(hwnd: HWND, context: &mut WindowContext, index: usize) {
    context.spec.state.select_command_button(Some(index));
    update_commands_menu_state(hwnd, context);

    if let Some(button) = context.command_button_controls.get(index).copied() {
        // SAFETY: button is a child button handle owned by this context. Focusing it gives the
        // user a visible keyboard focus cue for the selected command button.
        unsafe {
            SetFocus(button);
        }
    }
}

fn handle_execute_selected_command_button(hwnd: HWND, context: &mut WindowContext) {
    let Some(index) = context.spec.state.selected_command_button_index() else {
        show_warning_message(
            hwnd,
            context_tr(context, "명령", "Command"),
            context_tr(
                context,
                "실행할 명령을 선택하세요.",
                "Select a command to run.",
            ),
        );
        update_commands_menu_state(hwnd, context);
        return;
    };

    handle_execute_command_button(hwnd, context, index);
}

fn handle_execute_command_button(hwnd: HWND, context: &mut WindowContext, index: usize) {
    if command_button_click_ignored_for_tree_selection(selected_tree_node(context)) {
        return;
    }

    handle_select_command_button(hwnd, context, index);

    let language = context_language(context);
    let font_size = context.spec.state.settings().view.font_size;
    if !execute_selected_command_button(
        hwnd,
        &mut context.spec.state,
        CommandExecutionUi::new(
            context.instance,
            context.ui_font.handle(),
            font_size,
            language,
        ),
    ) {
        update_commands_menu_state(hwnd, context);
    }
}

fn handle_add_command_button(hwnd: HWND, context: &mut WindowContext) {
    let Some(tab_index) = context.spec.state.selected_command_tab_index() else {
        show_warning_message(
            hwnd,
            context_tr(context, "명령", "Command"),
            context_tr(
                context,
                "명령 그룹을 먼저 선택하세요.",
                "Select a command group first.",
            ),
        );
        update_commands_menu_state(hwnd, context);
        return;
    };

    if context.spec.state.selected_command_tab().is_none() {
        context.spec.state.select_command_tab(None);
        show_warning_message(
            hwnd,
            context_tr(context, "명령", "Command"),
            context_tr(
                context,
                "선택한 명령 그룹을 찾을 수 없습니다.",
                "The selected command group could not be found.",
            ),
        );
        update_commands_menu_state(hwnd, context);
        return;
    }

    if let Err(error) = show_command_button_dialog(
        hwnd,
        context.instance,
        CommandButtonDialogMode::Add,
        None,
        tab_index,
        None,
        context as *mut WindowContext,
        context.ui_font.handle(),
        context.spec.state.settings().view.font_size,
        context_language(context),
    ) {
        show_error_message(
            hwnd,
            context_tr(context, "명령", "Command"),
            &error.to_string(),
        );
    }
}

fn handle_edit_command_button(hwnd: HWND, context: &mut WindowContext) {
    let Some(tab_index) = context.spec.state.selected_command_tab_index() else {
        show_warning_message(
            hwnd,
            context_tr(context, "명령", "Command"),
            context_tr(
                context,
                "명령 그룹을 선택하세요.",
                "Select a command group.",
            ),
        );
        update_commands_menu_state(hwnd, context);
        return;
    };
    let Some(button_index) = context.spec.state.selected_command_button_index() else {
        show_warning_message(
            hwnd,
            context_tr(context, "명령", "Command"),
            context_tr(
                context,
                "편집할 명령을 선택하세요.",
                "Select a command to edit.",
            ),
        );
        update_commands_menu_state(hwnd, context);
        return;
    };
    let Some(button) = context.spec.state.selected_command_button().cloned() else {
        context.spec.state.select_command_button(None);
        show_warning_message(
            hwnd,
            context_tr(context, "명령", "Command"),
            context_tr(
                context,
                "선택한 명령을 찾을 수 없습니다.",
                "The selected command could not be found.",
            ),
        );
        update_commands_menu_state(hwnd, context);
        return;
    };

    if let Err(error) = show_command_button_dialog(
        hwnd,
        context.instance,
        CommandButtonDialogMode::Edit,
        Some(button),
        tab_index,
        Some(button_index),
        context as *mut WindowContext,
        context.ui_font.handle(),
        context.spec.state.settings().view.font_size,
        context_language(context),
    ) {
        show_error_message(
            hwnd,
            context_tr(context, "명령", "Command"),
            &error.to_string(),
        );
    }
}

fn handle_delete_command_button(hwnd: HWND, context: &mut WindowContext) {
    let Some(tab_index) = context.spec.state.selected_command_tab_index() else {
        show_warning_message(
            hwnd,
            context_tr(context, "명령", "Command"),
            context_tr(
                context,
                "명령 그룹을 선택하세요.",
                "Select a command group.",
            ),
        );
        update_commands_menu_state(hwnd, context);
        return;
    };
    let Some(button_index) = context.spec.state.selected_command_button_index() else {
        show_warning_message(
            hwnd,
            context_tr(context, "명령", "Command"),
            context_tr(
                context,
                "삭제할 명령을 선택하세요.",
                "Select a command to delete.",
            ),
        );
        update_commands_menu_state(hwnd, context);
        return;
    };
    let Some(button) = context.spec.state.selected_command_button().cloned() else {
        context.spec.state.select_command_button(None);
        show_warning_message(
            hwnd,
            context_tr(context, "명령", "Command"),
            context_tr(
                context,
                "선택한 명령을 찾을 수 없습니다.",
                "The selected command could not be found.",
            ),
        );
        update_commands_menu_state(hwnd, context);
        return;
    };

    let message = match context_language(context) {
        UiLanguage::Korean => format!(
            "명령을 삭제할까요?\n\n이름: {}\n실행 대상: {}",
            button.button_name, button.executable_path
        ),
        UiLanguage::English => format!(
            "Delete this command?\n\nName: {}\nExecutable: {}",
            button.button_name, button.executable_path
        ),
    };
    if !confirm_destructive_action(
        hwnd,
        context_tr(context, "명령 삭제", "Delete Command"),
        &message,
    ) {
        return;
    }

    let previous_state = SettingsRestorePoint::capture(&context.spec.state);
    if context
        .spec
        .state
        .delete_command_button(tab_index, button_index)
        .is_some()
    {
        if !persist_settings_or_restore(hwnd, &mut context.spec.state, previous_state) {
            return;
        }
        refresh_command_buttons(hwnd, context);
        update_commands_menu_state(hwnd, context);
    }
}

fn handle_move_command_button(
    hwnd: HWND,
    context: &mut WindowContext,
    direction: CommandButtonMoveDirection,
) {
    let Some(tab_index) = context.spec.state.selected_command_tab_index() else {
        show_warning_message(
            hwnd,
            context_tr(context, "명령", "Command"),
            context_tr(
                context,
                "명령 그룹을 선택하세요.",
                "Select a command group.",
            ),
        );
        update_commands_menu_state(hwnd, context);
        return;
    };
    let Some(button_index) = context.spec.state.selected_command_button_index() else {
        show_warning_message(
            hwnd,
            context_tr(context, "명령", "Command"),
            context_tr(
                context,
                "이동할 명령을 선택하세요.",
                "Select a command to move.",
            ),
        );
        update_commands_menu_state(hwnd, context);
        return;
    };
    let Some(tab) = context.spec.state.selected_command_tab() else {
        context.spec.state.select_command_tab(None);
        show_warning_message(
            hwnd,
            context_tr(context, "명령", "Command"),
            context_tr(
                context,
                "선택한 명령 그룹을 찾을 수 없습니다.",
                "The selected command group could not be found.",
            ),
        );
        update_commands_menu_state(hwnd, context);
        return;
    };
    let button_count = tab.buttons.len();
    let Some(destination_index) =
        layout_rules::command_button_move_destination(button_index, button_count, direction)
    else {
        show_warning_message(
            hwnd,
            context_tr(context, "명령", "Command"),
            context_tr(
                context,
                "더 이동할 수 없습니다.",
                "Cannot move any further.",
            ),
        );
        update_commands_menu_state(hwnd, context);
        return;
    };

    let previous_state = SettingsRestorePoint::capture(&context.spec.state);
    match context
        .spec
        .state
        .move_command_button(tab_index, button_index, destination_index)
    {
        Ok(()) => {
            if !persist_settings_or_restore(hwnd, &mut context.spec.state, previous_state) {
                return;
            }
            refresh_command_buttons(hwnd, context);
            update_commands_menu_state(hwnd, context);
        }
        Err(error) => show_error_message(
            hwnd,
            context_tr(context, "명령", "Command"),
            &error.user_message_for_language(context_language(context)),
        ),
    }
}

fn workspace_paths_except(context: &WindowContext, except_index: Option<usize>) -> Vec<String> {
    context
        .spec
        .state
        .settings()
        .workspaces
        .iter()
        .enumerate()
        .filter(|(index, _)| Some(*index) != except_index)
        .map(|(_, workspace)| workspace.path.clone())
        .collect()
}

fn handle_main_notify(hwnd: HWND, context: &mut WindowContext, lparam: LPARAM) -> LRESULT {
    if lparam == 0 {
        return 0;
    }

    // SAFETY: WM_NOTIFY lparam points to an NMHDR-compatible notification structure.
    let header = unsafe { &*(lparam as *const NMHDR) };
    match header.code {
        NM_DBLCLK
            if header.hwndFrom == context.content.tree_view
                && handle_tree_item_double_click(hwnd, context) =>
        {
            1
        }
        NM_RCLICK if header.hwndFrom == context.content.tree_view => {
            // SAFETY: GetMessagePos returns the screen coordinates for the current notification.
            let message_pos = unsafe { GetMessagePos() as LPARAM };
            if handle_tree_context_menu(context.content.tree_view, context, message_pos) {
                1
            } else {
                0
            }
        }
        TVN_SELCHANGEDW => {
            update_selected_workspace_from_tree(hwnd, context, lparam);
            0
        }
        TVN_BEGINDRAGW => {
            begin_workspace_tree_drag(hwnd, context, lparam);
            0
        }
        TVN_GETINFOTIPW => {
            fill_workspace_tree_tooltip(context, lparam);
            0
        }
        NM_CUSTOMDRAW => handle_workspace_tree_custom_draw(context, lparam),
        _ => 0,
    }
}

fn handle_workspace_tree_custom_draw(context: &WindowContext, lparam: LPARAM) -> LRESULT {
    if lparam == 0 {
        return CDRF_DODEFAULT as LRESULT;
    }

    // SAFETY: custom draw notifications start with an NMHDR-compatible header.
    let header = unsafe { &*(lparam as *const NMHDR) };
    if header.hwndFrom != context.content.tree_view {
        return CDRF_DODEFAULT as LRESULT;
    }

    // SAFETY: hwndFrom matched our TreeView, so NM_CUSTOMDRAW lparam is an NMTVCUSTOMDRAW.
    let custom_draw = unsafe { &mut *(lparam as *mut NMTVCUSTOMDRAW) };
    match custom_draw.nmcd.dwDrawStage {
        CDDS_PREPAINT => CDRF_NOTIFYITEMDRAW as LRESULT,
        CDDS_ITEMPREPAINT => {
            let item = custom_draw.nmcd.dwItemSpec as HTREEITEM;
            if workspace_tree_drag_target_item_matches(context, item) {
                let palette = ThemePalette::for_theme(current_view_theme(
                    &context.spec.state.settings().view,
                ));
                custom_draw.clrText = palette.control_text;
                custom_draw.clrTextBk = workspace_tree_drop_target_background(palette);
            }
            CDRF_DODEFAULT as LRESULT
        }
        _ => CDRF_DODEFAULT as LRESULT,
    }
}

fn update_selected_workspace_from_tree(hwnd: HWND, context: &mut WindowContext, lparam: LPARAM) {
    // SAFETY: The notification code was TVN_SELCHANGEDW, so lparam is an NMTREEVIEWW pointer.
    let notification = unsafe { &*(lparam as *const NMTREEVIEWW) };
    let selected_index = if notification.itemNew.hItem == 0 {
        None
    } else {
        workspace_index_from_tree_item(context.content.tree_view, notification.itemNew.hItem)
            .or_else(|| workspace_index_from_tree_lparam(notification.itemNew.lParam))
    };

    context.spec.state.select_workspace(selected_index);
    update_tree_menu_state(hwnd, context);
}

fn begin_workspace_tree_drag(hwnd: HWND, context: &mut WindowContext, lparam: LPARAM) {
    if context.content.tree_view.is_null() {
        return;
    }

    // SAFETY: The notification code was TVN_BEGINDRAGW, so lparam is an NMTREEVIEWW pointer.
    let notification = unsafe { &*(lparam as *const NMTREEVIEWW) };
    let Some(source_index) =
        workspace_index_from_tree_item(context.content.tree_view, notification.itemNew.hItem)
            .or_else(|| workspace_index_from_tree_lparam(notification.itemNew.lParam))
    else {
        return;
    };
    let settings = context.spec.state.settings();
    if settings.workspaces.len() <= 1 && settings.categories.is_empty() {
        return;
    }

    context.spec.state.select_workspace(Some(source_index));
    context.workspace_tree_drag = Some(WorkspaceTreeDrag {
        source_index,
        target: None,
    });
    update_tree_menu_state(hwnd, context);

    if notification.itemNew.hItem != 0 {
        // SAFETY: hItem was supplied by the TreeView that raised the drag notification.
        unsafe {
            SendMessageW(
                context.content.tree_view,
                TVM_SELECTITEM,
                TVGN_CARET as WPARAM,
                notification.itemNew.hItem as LPARAM,
            );
        }
    }

    // SAFETY: hwnd is the top-level window for this UI thread. Capturing mouse messages lets the
    // drag finish even if the cursor leaves the TreeView before button release.
    unsafe {
        SetCapture(hwnd);
    }
    update_workspace_tree_drag_from_tree_point(context, notification.ptDrag);
}

fn update_workspace_tree_drag_from_main_point(
    hwnd: HWND,
    context: &mut WindowContext,
    point: POINT,
) {
    let Some(tree_point) = client_point_between_windows(hwnd, context.content.tree_view, point)
    else {
        clear_workspace_tree_drag_target(context);
        return;
    };

    update_workspace_tree_drag_from_tree_point(context, tree_point);
}

fn update_workspace_tree_drag_from_tree_point(context: &mut WindowContext, point: POINT) {
    let Some(drag) = context.workspace_tree_drag else {
        return;
    };

    let target = workspace_tree_drop_target_at_tree_point(context, point)
        .filter(|target| workspace_tree_drop_target_is_valid(context, drag.source_index, *target));

    let previous_target = context.workspace_tree_drag.and_then(|drag| drag.target);
    if let Some(current) = context.workspace_tree_drag.as_mut() {
        current.target = target;
    }
    if previous_target != target {
        invalidate_workspace_tree_view(context);
    }
    set_workspace_tree_insert_mark(context, target);
}

fn workspace_tree_drop_target_is_valid(
    context: &WindowContext,
    source_index: usize,
    target: WorkspaceTreeDropTarget,
) -> bool {
    match target {
        WorkspaceTreeDropTarget::Workspace { .. } => {
            let settings = context.spec.state.settings();
            let root_items = settings.root_tree_items();
            layout_rules::workspace_tree_drop_action(
                &settings.workspaces,
                &settings.categories,
                root_items.as_slice(),
                source_index,
                target,
            )
            .is_some()
        }
        WorkspaceTreeDropTarget::Category { index } => {
            let settings = context.spec.state.settings();
            let Some(workspace) = settings.workspaces.get(source_index) else {
                return false;
            };
            let Some(category) = settings.categories.get(index) else {
                return false;
            };

            !layout_rules::workspace_belongs_to_category(workspace, category)
        }
    }
}

fn finish_workspace_tree_drag(hwnd: HWND, context: &mut WindowContext, lparam: LPARAM) -> bool {
    let Some(drag) = context.workspace_tree_drag else {
        return false;
    };

    update_workspace_tree_drag_from_main_point(hwnd, context, point_from_lparam(lparam));
    let target = context.workspace_tree_drag.and_then(|drag| drag.target);
    clear_workspace_tree_drag(context);

    // SAFETY: The drag was started by this window with SetCapture. Releasing when another window
    // already owns capture is harmless and restores normal mouse dispatch.
    unsafe {
        ReleaseCapture();
    }

    let Some(target) = target else {
        context.spec.state.select_workspace(Some(drag.source_index));
        update_tree_menu_state(hwnd, context);
        return true;
    };

    let previous_state = SettingsRestorePoint::capture(&context.spec.state);
    let language = context_language(context);
    let result = match target {
        WorkspaceTreeDropTarget::Workspace { .. } => {
            let Some(action) = ({
                let settings = context.spec.state.settings();
                let root_items = settings.root_tree_items();
                layout_rules::workspace_tree_drop_action(
                    &settings.workspaces,
                    &settings.categories,
                    root_items.as_slice(),
                    drag.source_index,
                    target,
                )
            }) else {
                context.spec.state.select_workspace(Some(drag.source_index));
                update_tree_menu_state(hwnd, context);
                return true;
            };
            match action {
                WorkspaceTreeDropAction::MoveWorkspace { destination_index } => context
                    .spec
                    .state
                    .move_workspace(drag.source_index, destination_index)
                    .map_err(|error| error.user_message_for_language(language)),
                WorkspaceTreeDropAction::MoveRootItem { destination_index } => context
                    .spec
                    .state
                    .move_root_tree_item(
                        TreeRootItemRef::Workspace(drag.source_index),
                        destination_index,
                    )
                    .map_err(|error| error.user_message_for_language(language)),
            }
        }
        WorkspaceTreeDropTarget::Category { index } => context
            .spec
            .state
            .move_workspace_to_category(drag.source_index, index)
            .map_err(|error| error.user_message_for_language(language)),
    };

    match result {
        Ok(()) => {
            if !persist_settings_or_restore(hwnd, &mut context.spec.state, previous_state) {
                return true;
            }
            refresh_tree_view(context);
            update_tree_menu_state(hwnd, context);
        }
        Err(error) => show_error_message(
            hwnd,
            context_tr(context, "워크스페이스", "Workspace"),
            &error,
        ),
    }

    true
}

fn cancel_workspace_tree_drag(context: &mut WindowContext) {
    if context.workspace_tree_drag.is_some() {
        clear_workspace_tree_drag(context);
    }
}

fn clear_workspace_tree_drag(context: &mut WindowContext) {
    let previous_target = context.workspace_tree_drag.and_then(|drag| drag.target);
    context.workspace_tree_drag = None;
    set_workspace_tree_insert_mark(context, None);
    if previous_target.is_some() {
        invalidate_workspace_tree_view(context);
    }
}

fn clear_workspace_tree_drag_target(context: &mut WindowContext) {
    let previous_target = context.workspace_tree_drag.and_then(|drag| drag.target);
    if let Some(drag) = context.workspace_tree_drag.as_mut() {
        drag.target = None;
    }
    set_workspace_tree_insert_mark(context, None);
    if previous_target.is_some() {
        invalidate_workspace_tree_view(context);
    }
}

fn invalidate_workspace_tree_view(context: &WindowContext) {
    // SAFETY: tree_view is a TreeView handle owned by this UI thread; null is accepted by the
    // helper and ignored.
    unsafe {
        invalidate_window(context.content.tree_view);
    }
}

fn workspace_tree_drag_target_item_matches(context: &WindowContext, item: HTREEITEM) -> bool {
    if item == 0 {
        return false;
    }

    let Some(target) = context.workspace_tree_drag.and_then(|drag| drag.target) else {
        return false;
    };

    workspace_tree_drop_target_item(context, target) == Some(item)
}

fn workspace_tree_drop_target_item(
    context: &WindowContext,
    target: WorkspaceTreeDropTarget,
) -> Option<HTREEITEM> {
    match target {
        WorkspaceTreeDropTarget::Workspace { index, .. } => context.tree_items.get(index).copied(),
        WorkspaceTreeDropTarget::Category { index } => {
            context.category_tree_items.get(index).copied()
        }
    }
    .filter(|item| *item != 0)
}

fn begin_command_button_drag(button: HWND, context: &mut WindowContext, lparam: LPARAM) -> bool {
    let Some(source_index) = command_button_index_from_handle(context, button) else {
        return false;
    };
    let button_count = context
        .spec
        .state
        .selected_command_tab()
        .map(|tab| tab.buttons.len())
        .unwrap_or_default();
    if source_index >= button_count {
        return false;
    }

    let point = point_from_lparam(lparam);
    context.spec.state.select_command_button(Some(source_index));
    context.command_button_drag = Some(CommandButtonDrag {
        source_index,
        start_x: point.x,
        start_y: point.y,
        target_index: None,
        moved: false,
    });

    set_command_button_pressed(button, true);
    // SAFETY: button is the child button that received the mouse message. Capturing the button
    // keeps subsequent drag messages in the same subclass procedure until mouse release.
    unsafe {
        SetFocus(button);
        SetCapture(button);
    }

    let owner = command_button_owner(button, context);
    if !owner.is_null() {
        update_commands_menu_state(owner, context);
    }

    true
}

fn update_command_button_drag_from_button_point(
    button: HWND,
    context: &mut WindowContext,
    point: POINT,
) {
    let Some(drag) = context.command_button_drag else {
        return;
    };

    let moved = drag.moved
        || command_button_drag_exceeds_threshold(drag.start_x, drag.start_y, point, context.dpi);
    let parent = command_button_parent(button);
    let button_count = context
        .spec
        .state
        .selected_command_tab()
        .map(|tab| tab.buttons.len())
        .unwrap_or_default();
    let target_index = if moved && !parent.is_null() {
        client_point_between_windows(button, parent, point)
            .and_then(|point| command_button_drop_target_at_main_point(parent, context, point))
            .filter(|target_index| {
                layout_rules::command_button_drop_destination(
                    drag.source_index,
                    *target_index,
                    button_count,
                )
                .is_some()
            })
    } else {
        None
    };

    let previous_target_index = context
        .command_button_drag
        .and_then(|drag| drag.target_index);
    if let Some(current) = context.command_button_drag.as_mut() {
        current.moved = moved;
        current.target_index = target_index;
    }
    if previous_target_index != target_index {
        invalidate_command_button_at_index(context, previous_target_index);
        invalidate_command_button_at_index(context, target_index);
    }
}

fn finish_command_button_drag(button: HWND, context: &mut WindowContext, lparam: LPARAM) -> bool {
    let Some(_) = context.command_button_drag else {
        return false;
    };

    let drag_button = context_button_for_drag(context, button);
    update_command_button_drag_from_button_point(drag_button, context, point_from_lparam(lparam));
    let Some(drag) = context.command_button_drag else {
        return false;
    };
    clear_command_button_drag(context);
    // SAFETY: The drag path sets capture on the command button; releasing is harmless if capture
    // has already changed.
    unsafe {
        ReleaseCapture();
    }

    let owner = command_button_owner(button, context);
    if !drag.moved {
        if !owner.is_null() {
            handle_execute_command_button(owner, context, drag.source_index);
        }
        return true;
    }

    let Some(tab_index) = context.spec.state.selected_command_tab_index() else {
        if !owner.is_null() {
            update_commands_menu_state(owner, context);
        }
        return true;
    };
    let button_count = context
        .spec
        .state
        .selected_command_tab()
        .map(|tab| tab.buttons.len())
        .unwrap_or_default();
    let Some(destination_index) = drag.target_index.and_then(|target_index| {
        layout_rules::command_button_drop_destination(drag.source_index, target_index, button_count)
    }) else {
        context
            .spec
            .state
            .select_command_button(Some(drag.source_index));
        if !owner.is_null() {
            update_commands_menu_state(owner, context);
        }
        return true;
    };

    let previous_state =
        (!owner.is_null()).then(|| SettingsRestorePoint::capture(&context.spec.state));
    match context
        .spec
        .state
        .move_command_button(tab_index, drag.source_index, destination_index)
    {
        Ok(()) => {
            if let Some(previous_state) = previous_state
                && !persist_settings_or_restore(owner, &mut context.spec.state, previous_state)
            {
                return true;
            }
            if !owner.is_null() {
                refresh_command_buttons(owner, context);
                update_commands_menu_state(owner, context);
            }
        }
        Err(error) => {
            if !owner.is_null() {
                show_error_message(
                    owner,
                    context_tr(context, "명령", "Command"),
                    &error.user_message_for_language(context_language(context)),
                );
            }
        }
    }

    true
}

fn cancel_command_button_drag(context: &mut WindowContext) {
    if context.command_button_drag.is_some() {
        clear_command_button_drag(context);
        // SAFETY: Releasing capture is harmless if this window no longer owns it.
        unsafe {
            ReleaseCapture();
        }
    }
}

fn clear_command_button_drag(context: &mut WindowContext) {
    let drag = context.command_button_drag;
    context.command_button_drag = None;
    if let Some(source_index) = drag.map(|drag| drag.source_index)
        && let Some(button) = context.command_button_controls.get(source_index).copied()
    {
        set_command_button_pressed(button, false);
        // SAFETY: button is a command button handle owned by this UI thread.
        unsafe {
            invalidate_window(button);
        }
    }
    if let Some(target_index) = drag.and_then(|drag| drag.target_index) {
        invalidate_command_button_at_index(context, Some(target_index));
    }
}

fn invalidate_command_button_at_index(context: &WindowContext, index: Option<usize>) {
    if let Some(button) = index.and_then(|index| context.command_button_controls.get(index))
        && !button.is_null()
    {
        // SAFETY: button is a command button handle owned by this UI thread.
        unsafe {
            invalidate_window(*button);
        }
    }
}

fn command_button_drag_target_matches(context: &WindowContext, button: HWND) -> bool {
    if button.is_null() {
        return false;
    }

    context
        .command_button_drag
        .and_then(|drag| drag.target_index)
        .and_then(|index| context.command_button_controls.get(index).copied())
        == Some(button)
}

fn command_button_index_from_handle(context: &WindowContext, button: HWND) -> Option<usize> {
    if button.is_null() {
        return None;
    }

    context
        .command_button_controls
        .iter()
        .position(|handle| *handle == button)
}

fn context_button_for_drag(context: &WindowContext, fallback: HWND) -> HWND {
    context
        .command_button_drag
        .and_then(|drag| {
            context
                .command_button_controls
                .get(drag.source_index)
                .copied()
        })
        .filter(|button| !button.is_null())
        .unwrap_or(fallback)
}

fn command_button_parent(button: HWND) -> HWND {
    if button.is_null() {
        return null_mut();
    }

    // SAFETY: button is a child window handle or null. GetParent returns null on failure.
    unsafe { GetParent(button) }
}

fn command_button_owner(button: HWND, context: &WindowContext) -> HWND {
    let parent = command_button_parent(button);
    if parent.is_null() {
        return null_mut();
    }

    if parent == context.content.command_tabs.page {
        // SAFETY: the command-tab page is a direct child of the main window.
        unsafe { GetParent(parent) }
    } else {
        parent
    }
}

fn command_button_drag_exceeds_threshold(
    start_x: i32,
    start_y: i32,
    point: POINT,
    dpi: u32,
) -> bool {
    let threshold = i64::from(scale_dimension_for_dpi(COMMAND_BUTTON_DRAG_THRESHOLD, dpi).max(1));
    let dx = i64::from(point.x) - i64::from(start_x);
    let dy = i64::from(point.y) - i64::from(start_y);

    dx.abs() >= threshold || dy.abs() >= threshold
}

fn command_button_drop_target_at_main_point(
    parent: HWND,
    context: &WindowContext,
    point: POINT,
) -> Option<usize> {
    if parent.is_null() || parent != context.content.command_tabs.page {
        return None;
    }

    // SAFETY: the command-tab page is a direct child of the main window.
    let owner = unsafe { GetParent(parent) };
    if owner.is_null() {
        return None;
    }

    let layout = current_main_content_layout(owner, context.spec.layout).ok()?;
    let font_size = context.spec.state.settings().view.font_size;
    let parent_rect =
        context
            .content
            .command_tabs
            .page_rect(layout.command_tabs_panel, font_size, context.dpi);
    let button_count = context
        .spec
        .state
        .selected_command_tab()
        .map(|tab| tab.buttons.len())
        .unwrap_or_default()
        .min(context.command_button_controls.len());

    command_button_rects_in_parent(
        layout.command_tabs_panel,
        parent_rect,
        button_count,
        font_size,
        context.dpi,
        context.command_button_scroll_offset,
    )
    .index_at_point(point)
}

fn set_command_button_pressed(button: HWND, pressed: bool) {
    if button.is_null() {
        return;
    }

    // SAFETY: button is a BUTTON control. BM_SETSTATE updates only the visual pressed state.
    unsafe {
        SendMessageW(button, BM_SETSTATE, if pressed { 1 } else { 0 }, 0);
    }
}

fn workspace_tree_drop_target_at_tree_point(
    context: &WindowContext,
    point: POINT,
) -> Option<WorkspaceTreeDropTarget> {
    let hit_item = tree_item_at_tree_point(context.content.tree_view, point)?;

    if let Some(index) = category_index_from_tree_item(context.content.tree_view, hit_item) {
        return Some(WorkspaceTreeDropTarget::Category { index });
    }

    let index = workspace_index_from_tree_item(context.content.tree_view, hit_item)?;
    let rect = tree_item_rect(context.content.tree_view, hit_item)?;
    let midpoint = rect.top + (rect.bottom - rect.top) / 2;

    Some(WorkspaceTreeDropTarget::Workspace {
        index,
        insert_after: point.y >= midpoint,
    })
}

fn tree_item_at_tree_point(tree_view: HWND, point: POINT) -> Option<HTREEITEM> {
    if tree_view.is_null() {
        return None;
    }

    let mut hit_test = TVHITTESTINFO {
        pt: point,
        ..TVHITTESTINFO::default()
    };

    // SAFETY: tree_view is a TreeView control and hit_test points to initialized writable storage.
    let hit_item = unsafe {
        SendMessageW(
            tree_view,
            TVM_HITTEST,
            0,
            &mut hit_test as *mut TVHITTESTINFO as LPARAM,
        ) as HTREEITEM
    };
    if hit_item == 0 || hit_test.hItem == 0 || hit_test.flags & TVHT_ONITEM == 0 {
        return None;
    }

    Some(hit_test.hItem)
}

fn workspace_index_from_tree_item(tree_view: HWND, item: HTREEITEM) -> Option<usize> {
    workspace_index_from_tree_lparam(tree_lparam_from_tree_item(tree_view, item)?)
}

fn category_index_from_tree_item(tree_view: HWND, item: HTREEITEM) -> Option<usize> {
    category_index_from_tree_lparam(tree_lparam_from_tree_item(tree_view, item)?)
}

fn tree_lparam_from_tree_item(tree_view: HWND, item: HTREEITEM) -> Option<LPARAM> {
    if tree_view.is_null() || item == 0 {
        return None;
    }

    let mut tree_item = TVITEMW {
        mask: TVIF_HANDLE | TVIF_PARAM,
        hItem: item,
        ..TVITEMW::default()
    };

    // SAFETY: tree_view is a TreeView control and tree_item requests only the item lParam.
    let ok = unsafe {
        SendMessageW(
            tree_view,
            TVM_GETITEMW,
            0,
            &mut tree_item as *mut TVITEMW as LPARAM,
        )
    };
    if ok == 0 {
        return None;
    }

    Some(tree_item.lParam)
}

fn tree_item_rect(tree_view: HWND, item: HTREEITEM) -> Option<RECT> {
    if tree_view.is_null() || item == 0 {
        return None;
    }

    let mut rect = RECT::default();
    // SAFETY: TVM_GETITEMRECT expects the HTREEITEM to be stored at the start of the RECT buffer.
    // RECT is large enough to hold the handle before the control overwrites it with coordinates.
    unsafe {
        *(&mut rect as *mut RECT as *mut HTREEITEM) = item;
    }

    // SAFETY: tree_view is a TreeView control and rect points to writable RECT storage prepared
    // according to the TVM_GETITEMRECT contract.
    let ok = unsafe {
        SendMessageW(
            tree_view,
            TVM_GETITEMRECT,
            0,
            &mut rect as *mut RECT as LPARAM,
        )
    };
    (ok != 0).then_some(rect)
}

fn set_workspace_tree_insert_mark(
    context: &WindowContext,
    target: Option<WorkspaceTreeDropTarget>,
) {
    if context.content.tree_view.is_null() {
        return;
    }

    let Some(target) = target else {
        // SAFETY: tree_view is a TreeView control. Null hItem clears the insertion mark.
        unsafe {
            SendMessageW(context.content.tree_view, TVM_SETINSERTMARK, 0, 0);
        }
        return;
    };
    let WorkspaceTreeDropTarget::Workspace {
        index,
        insert_after,
    } = target
    else {
        // SAFETY: tree_view is a TreeView control. Null hItem clears the insertion mark.
        unsafe {
            SendMessageW(context.content.tree_view, TVM_SETINSERTMARK, 0, 0);
        }
        return;
    };

    let Some(item) = context.tree_items.get(index).copied() else {
        return;
    };
    if item == 0 {
        return;
    }

    // SAFETY: item was returned by this TreeView control. wParam selects before/after placement.
    unsafe {
        SendMessageW(
            context.content.tree_view,
            TVM_SETINSERTMARK,
            usize::from(insert_after),
            item as LPARAM,
        );
    }
}

fn client_point_between_windows(from: HWND, to: HWND, point: POINT) -> Option<POINT> {
    if from.is_null() || to.is_null() {
        return None;
    }

    let mut converted = point;
    // SAFETY: from and to are live windows in this UI thread, and converted is writable storage.
    let to_screen = unsafe { ClientToScreen(from, &mut converted) };
    if to_screen == 0 {
        return None;
    }

    // SAFETY: converted currently holds a screen coordinate and to is the target child window.
    let to_client = unsafe { ScreenToClient(to, &mut converted) };
    (to_client != 0).then_some(converted)
}

fn update_selected_command_tab_from_selector(hwnd: HWND, context: &mut WindowContext) {
    if context.content.command_tabs.selector.is_null() {
        context.spec.state.select_command_tab(None);
        update_tabs_menu_state(hwnd, context);
        return;
    }

    // SAFETY: selector is a valid ComboBox created by this module.
    let selected =
        unsafe { SendMessageW(context.content.command_tabs.selector, CB_GETCURSEL, 0, 0) };
    let selected_index = if selected == CB_ERR as isize || selected < 0 {
        None
    } else {
        usize::try_from(selected).ok()
    };

    context.spec.state.select_command_tab(selected_index);
    context.command_button_scroll_offset = 0;
    update_tabs_menu_state(hwnd, context);
    refresh_command_buttons(hwnd, context);
    update_commands_menu_state(hwnd, context);
}

fn fill_workspace_tree_tooltip(context: &WindowContext, lparam: LPARAM) {
    // SAFETY: The notification code was TVN_GETINFOTIPW, so lparam is an NMTVGETINFOTIPW pointer.
    let info = unsafe { &mut *(lparam as *mut NMTVGETINFOTIPW) };
    let Some(index) = workspace_index_from_tree_lparam(info.lParam) else {
        return;
    };
    let Some(workspace) = context.spec.state.settings().workspaces.get(index) else {
        return;
    };

    let tooltip = match context_language(context) {
        UiLanguage::Korean => format!("폴더: {}\n언어: {}", workspace.path, workspace.language),
        UiLanguage::English => format!(
            "Folder: {}\nLanguage: {}",
            workspace.path, workspace.language
        ),
    };
    write_wide_text_to_buffer(&tooltip, info.pszText, info.cchTextMax);
}

fn write_wide_text_to_buffer(text: &str, buffer: *mut u16, buffer_len: i32) {
    if buffer.is_null() || buffer_len <= 0 {
        return;
    }

    let wide: Vec<u16> = text.encode_utf16().collect();
    let copy_len = wide.len().min((buffer_len - 1) as usize);

    // SAFETY: buffer points to cchTextMax writable u16 slots supplied by the control. We copy at
    // most cchTextMax - 1 code units and always write a trailing NUL.
    unsafe {
        std::ptr::copy_nonoverlapping(wide.as_ptr(), buffer, copy_len);
        *buffer.add(copy_len) = 0;
    }
}

fn browse_for_selected_file(owner: HWND, language: UiLanguage) -> Option<PathBuf> {
    let mut file_buffer = vec![0u16; 32768];
    let filter = wide_filter(&[tr(language, "모든 파일", "All Files"), "*.*"]);
    let title = wide_null(tr(language, "파일 선택", "Select File"));
    let mut open_file = OPENFILENAMEW {
        lStructSize: size_of::<OPENFILENAMEW>() as u32,
        hwndOwner: owner,
        lpstrFilter: filter.as_ptr(),
        lpstrFile: file_buffer.as_mut_ptr(),
        nMaxFile: file_buffer.len() as u32,
        lpstrTitle: title.as_ptr(),
        Flags: OFN_EXPLORER | OFN_FILEMUSTEXIST | OFN_PATHMUSTEXIST,
        ..OPENFILENAMEW::default()
    };

    // SAFETY: open_file points to initialized OPENFILENAMEW storage and file_buffer is writable
    // for nMaxFile UTF-16 code units for the duration of the call.
    let ok = unsafe { GetOpenFileNameW(&mut open_file) };
    if ok == 0 {
        None
    } else {
        Some(PathBuf::from(wide_null_buffer_to_string(&file_buffer)))
    }
}

fn wide_filter(parts: &[&str]) -> Vec<u16> {
    let mut buffer = Vec::new();
    for part in parts {
        buffer.extend(part.encode_utf16());
        buffer.push(0);
    }
    buffer.push(0);
    buffer
}

fn browse_for_folder(owner: HWND, title: &str) -> Option<PathBuf> {
    let mut display_name = vec![0u16; MAX_PATH as usize + 1];
    let title = wide_null(title);
    let browse_info = BROWSEINFOW {
        hwndOwner: owner,
        pidlRoot: null_mut(),
        pszDisplayName: display_name.as_mut_ptr(),
        lpszTitle: title.as_ptr(),
        ulFlags: BIF_RETURNONLYFSDIRS | BIF_NEWDIALOGSTYLE,
        lpfn: None,
        lParam: 0,
        iImage: 0,
    };

    // SAFETY: browse_info points to initialized storage that remains valid for the call.
    let item_id_list = unsafe { SHBrowseForFolderW(&browse_info) };
    if item_id_list.is_null() {
        return None;
    }

    let mut path = vec![0u16; MAX_PATH as usize + 1];
    // SAFETY: item_id_list was returned by SHBrowseForFolderW and path points to writable storage.
    let ok = unsafe { SHGetPathFromIDListW(item_id_list, path.as_mut_ptr()) };
    // SAFETY: SHBrowseForFolderW allocates the returned PIDL with the COM task allocator.
    unsafe {
        CoTaskMemFree(item_id_list as *const c_void);
    }

    if ok == 0 {
        None
    } else {
        Some(PathBuf::from(wide_null_buffer_to_string(&path)))
    }
}

fn is_accessible_folder(path: &str) -> bool {
    let path = Path::new(path.trim());
    std::fs::metadata(path)
        .map(|metadata| metadata.is_dir())
        .unwrap_or(false)
        && is_readable_folder(path)
}

fn is_readable_folder(path: &Path) -> bool {
    std::fs::read_dir(path).is_ok()
}

fn confirm_destructive_action(hwnd: HWND, caption: &str, message: &str) -> bool {
    let caption = wide_null(caption);
    let message = wide_null(message);
    // SAFETY: hwnd is an owner window and both strings are valid null-terminated UTF-16.
    let result = unsafe {
        MessageBoxW(
            hwnd,
            message.as_ptr(),
            caption.as_ptr(),
            MB_YESNO | MB_ICONWARNING | MB_DEFBUTTON2,
        )
    };
    result == IDYES
}

fn show_error_message(hwnd: HWND, caption: &str, message: &str) {
    show_message(hwnd, caption, message, MB_OK | MB_ICONERROR);
}

fn show_warning_message(hwnd: HWND, caption: &str, message: &str) {
    show_message(hwnd, caption, message, MB_OK | MB_ICONWARNING);
}

fn show_message(hwnd: HWND, caption: &str, message: &str, style: u32) {
    let caption = wide_null(caption);
    let message = wide_null(message);
    // SAFETY: hwnd is an owner window and both strings are valid null-terminated UTF-16.
    unsafe {
        MessageBoxW(hwnd, message.as_ptr(), caption.as_ptr(), style);
    }
}

fn wide_null_buffer_to_string(buffer: &[u16]) -> String {
    let len = buffer
        .iter()
        .position(|code_unit| *code_unit == 0)
        .unwrap_or(buffer.len());
    String::from_utf16_lossy(&buffer[..len])
}

fn handle_dpi_changed(hwnd: HWND, context: &mut WindowContext, wparam: WPARAM, lparam: LPARAM) {
    let previous_dpi = context.dpi;
    let previous_tree_panel_width = context.spec.layout.tree_panel_width;
    let dpi = high_word(wparam).max(low_word(wparam)).max(1);
    let view = context.spec.state.settings().view.clone();
    let new_font = UiFont::from_view(&view, dpi);
    let old_font = std::mem::replace(&mut context.ui_font, new_font);

    context.dpi = dpi;
    context.spec.layout = LayoutSpec::for_font_size_and_dpi(view.font_size, dpi);
    context.spec.layout.tree_panel_width =
        scale_dimension_between_dpi(previous_tree_panel_width, previous_dpi, dpi);

    if lparam != 0 {
        // SAFETY: WM_DPICHANGED supplies lparam as a RECT pointer with the suggested top-level
        // window bounds for the new DPI.
        let suggested = unsafe { &*(lparam as *const RECT) };
        // SAFETY: hwnd is the top-level window being notified. MoveWindow applies the suggested
        // bounds and triggers normal WM_SIZE layout handling.
        unsafe {
            MoveWindow(
                hwnd,
                suggested.left,
                suggested.top,
                suggested.right - suggested.left,
                suggested.bottom - suggested.top,
                1,
            );
        }
    }

    apply_ui_font_to_main(hwnd, context);
    resize_main_content(hwnd, context);
    refresh_command_tab_selector(context);
    refresh_command_buttons(hwnd, context);
    drop(old_font);
}

unsafe extern "system" fn command_button_subclass_proc(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
    _subclass_id: usize,
    ref_data: usize,
) -> LRESULT {
    // SAFETY: ref_data was set to the owning WindowContext pointer by SetWindowSubclass.
    let context = unsafe { (ref_data as *mut WindowContext).as_mut() };
    if let Some(context) = context {
        match message {
            WM_LBUTTONDOWN if begin_command_button_drag(hwnd, context, lparam) => {
                return 0;
            }
            WM_MOUSEMOVE if context.command_button_drag.is_some() => {
                let drag_button = context_button_for_drag(context, hwnd);
                update_command_button_drag_from_button_point(
                    drag_button,
                    context,
                    point_from_lparam(lparam),
                );
                return 0;
            }
            WM_LBUTTONUP if context.command_button_drag.is_some() => {
                finish_command_button_drag(hwnd, context, lparam);
                return 0;
            }
            WM_CANCELMODE | WM_CAPTURECHANGED => {
                cancel_command_button_drag(context);
            }
            WM_MOUSEWHEEL => {
                let parent = command_button_parent(hwnd);
                if parent == context.content.command_tabs.page
                    && handle_command_tab_page_mouse_wheel(parent, context, wparam)
                {
                    return 0;
                }
            }
            WM_CONTEXTMENU if handle_command_button_context_menu(hwnd, context, lparam) => {
                return 0;
            }
            _ => {}
        }
    }

    // SAFETY: Messages not fully handled by this subclass continue through the common-control
    // subclass chain.
    unsafe { DefSubclassProc(hwnd, message, wparam, lparam) }
}

unsafe extern "system" fn command_tab_page_proc(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match message {
        WM_NCCREATE => {
            // SAFETY: For WM_NCCREATE, lparam is a CREATESTRUCTW pointer supplied by Windows.
            let create = unsafe { &*(lparam as *const CREATESTRUCTW) };
            let context = create.lpCreateParams as *mut WindowContext;
            // SAFETY: The context pointer is supplied when creating the command-tab page and
            // remains valid while the main message loop is running.
            unsafe {
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, context as isize);
                DefWindowProcW(hwnd, message, wparam, lparam)
            }
        }
        WM_COMMAND => {
            // SAFETY: the command-tab page is a direct child of the main window.
            let owner = unsafe { GetParent(hwnd) };
            if !owner.is_null() {
                // SAFETY: owner is the main window; forwarding preserves keyboard/button
                // activation now that command buttons are children of the page panel.
                unsafe { SendMessageW(owner, WM_COMMAND, wparam, lparam) }
            } else {
                0
            }
        }
        WM_VSCROLL => {
            // SAFETY: The userdata pointer is set during WM_NCCREATE and cleared at WM_NCDESTROY.
            if let Some(context) = unsafe { command_tab_page_context_mut(hwnd) }
                && handle_command_tab_page_vscroll(hwnd, context, wparam)
            {
                0
            } else {
                // SAFETY: Unhandled scroll messages are delegated to Windows.
                unsafe { DefWindowProcW(hwnd, message, wparam, lparam) }
            }
        }
        WM_MOUSEWHEEL => {
            // SAFETY: The userdata pointer is set during WM_NCCREATE and cleared at WM_NCDESTROY.
            if let Some(context) = unsafe { command_tab_page_context_mut(hwnd) }
                && handle_command_tab_page_mouse_wheel(hwnd, context, wparam)
            {
                0
            } else {
                // SAFETY: Unhandled wheel messages are delegated to Windows.
                unsafe { DefWindowProcW(hwnd, message, wparam, lparam) }
            }
        }
        WM_CTLCOLORBTN => {
            // SAFETY: The userdata pointer is set during WM_NCCREATE and cleared at WM_NCDESTROY.
            if let Some(context) = unsafe { command_tab_page_context_mut(hwnd) }
                && let Some(result) = themed_button_color_brush(context, wparam, lparam)
            {
                result
            } else {
                // SAFETY: Unhandled control color requests are delegated to Windows.
                unsafe { DefWindowProcW(hwnd, message, wparam, lparam) }
            }
        }
        WM_ERASEBKGND => {
            // SAFETY: The userdata pointer is set during WM_NCCREATE and cleared at WM_NCDESTROY.
            if let Some(context) = unsafe { command_tab_page_context_mut(hwnd) }
                && let Some(result) = erase_command_tab_page_background(hwnd, context, wparam)
            {
                result
            } else {
                // SAFETY: Unhandled erase requests are delegated to Windows.
                unsafe { DefWindowProcW(hwnd, message, wparam, lparam) }
            }
        }
        WM_NCDESTROY => {
            // SAFETY: Clear the non-owning context pointer before the window handle dies.
            unsafe {
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
                DefWindowProcW(hwnd, message, wparam, lparam)
            }
        }
        _ => {
            // SAFETY: All messages not handled by this window procedure are delegated to Windows.
            unsafe { DefWindowProcW(hwnd, message, wparam, lparam) }
        }
    }
}

unsafe extern "system" fn tree_view_subclass_proc(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
    _subclass_id: usize,
    ref_data: usize,
) -> LRESULT {
    // SAFETY: ref_data was set to the owning WindowContext pointer by SetWindowSubclass.
    let context = unsafe { (ref_data as *mut WindowContext).as_mut() };
    if let Some(context) = context {
        match message {
            WM_KEYDOWN if handle_tree_view_key_down(hwnd, context, wparam) => {
                return 0;
            }
            WM_CONTEXTMENU if handle_tree_context_menu(hwnd, context, lparam) => {
                return 0;
            }
            _ => {}
        }
    }

    // SAFETY: Messages not handled by this subclass continue through the common-control
    // subclass chain.
    unsafe { DefSubclassProc(hwnd, message, wparam, lparam) }
}

fn handle_tree_view_key_down(tree_view: HWND, context: &mut WindowContext, key: WPARAM) -> bool {
    if !control_key_is_down() {
        return false;
    }

    if key == VK_LEFT_KEY {
        // SAFETY: tree_view is the TreeView child receiving this keyboard message.
        let parent = unsafe { GetParent(tree_view) };
        if parent.is_null() {
            return true;
        }

        let selected_item = selected_tree_item(tree_view);
        if let Some(index) = workspace_index_from_tree_item(tree_view, selected_item) {
            context.spec.state.select_workspace(Some(index));
            return handle_keyboard_move_workspace_to_root(parent, context);
        }

        return false;
    }

    let direction = match key {
        VK_UP_KEY => TreeKeyboardMoveDirection::Up,
        VK_DOWN_KEY => TreeKeyboardMoveDirection::Down,
        _ => return false,
    };

    // SAFETY: tree_view is the TreeView child receiving this keyboard message.
    let parent = unsafe { GetParent(tree_view) };
    if parent.is_null() {
        return true;
    }

    let selected_item = selected_tree_item(tree_view);
    if let Some(index) = category_index_from_tree_item(tree_view, selected_item) {
        context.spec.state.select_workspace(None);
        return handle_keyboard_move_category(parent, context, index, direction);
    }

    if let Some(index) = workspace_index_from_tree_item(tree_view, selected_item) {
        context.spec.state.select_workspace(Some(index));
    }

    handle_keyboard_move_workspace(parent, context, direction)
}

fn control_key_is_down() -> bool {
    // SAFETY: GetKeyState is safe to query for a virtual-key code on the UI thread.
    unsafe { GetKeyState(VK_CONTROL_KEY) < 0 }
}

unsafe extern "system" fn window_proc(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match message {
        WM_NCCREATE => {
            // SAFETY: For WM_NCCREATE, lparam is a CREATESTRUCTW pointer supplied by Windows.
            let create = unsafe { &*(lparam as *const CREATESTRUCTW) };
            let context = create.lpCreateParams as *mut WindowContext;
            // SAFETY: The context pointer was supplied by create_main_window and remains valid
            // while the message loop is running.
            unsafe {
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, context as isize);
                DefWindowProcW(hwnd, message, wparam, lparam)
            }
        }
        WM_SIZE => {
            // SAFETY: The userdata pointer is set during WM_NCCREATE and cleared at WM_NCDESTROY.
            if let Some(context) = unsafe { window_context_mut(hwnd) } {
                resize_main_content(hwnd, context);
            }
            0
        }
        WM_GETMINMAXINFO => {
            // SAFETY: The userdata pointer is set during WM_NCCREATE and cleared at WM_NCDESTROY.
            if let Some(context) = unsafe { window_context_mut(hwnd) } {
                apply_minimum_window_track_size(hwnd, context, lparam);
            }
            0
        }
        WM_DPICHANGED => {
            // SAFETY: The userdata pointer is set during WM_NCCREATE and cleared at WM_NCDESTROY.
            if let Some(context) = unsafe { window_context_mut(hwnd) } {
                handle_dpi_changed(hwnd, context, wparam, lparam);
            }
            0
        }
        WM_COMMAND => {
            // SAFETY: The userdata pointer is set during WM_NCCREATE and cleared at WM_NCDESTROY.
            if let Some(context) = unsafe { window_context_mut(hwnd) } {
                handle_main_command(hwnd, context, wparam, lparam);
            }
            0
        }
        WM_CLOSE => {
            // SAFETY: The userdata pointer is set during WM_NCCREATE and cleared at WM_NCDESTROY.
            if let Some(context) = unsafe { window_context_mut(hwnd) } {
                if !persist_window_layout_before_close(hwnd, context) {
                    return 0;
                }
                // SAFETY: hwnd is the main window being closed by this UI thread.
                unsafe {
                    DestroyWindow(hwnd);
                }
                0
            } else {
                // SAFETY: Unhandled close requests are delegated to Windows.
                unsafe { DefWindowProcW(hwnd, message, wparam, lparam) }
            }
        }
        WM_LBUTTONDOWN => {
            // SAFETY: The userdata pointer is set during WM_NCCREATE and cleared at WM_NCDESTROY.
            if let Some(context) = unsafe { window_context_mut(hwnd) }
                && begin_main_splitter_drag(hwnd, context, point_from_lparam(lparam))
            {
                0
            } else {
                // SAFETY: Unhandled mouse messages are delegated to Windows.
                unsafe { DefWindowProcW(hwnd, message, wparam, lparam) }
            }
        }
        WM_WORKSPACE_DROP_CHECKED => {
            // SAFETY: The userdata pointer is set during WM_NCCREATE and cleared at WM_NCDESTROY.
            if let Some(context) = unsafe { window_context_mut(hwnd) }
                && let Ok(request_id) = u32::try_from(wparam)
            {
                handle_workspace_drop_checked(hwnd, context, request_id);
            }
            0
        }
        WM_MOUSEMOVE => {
            // SAFETY: The userdata pointer is set during WM_NCCREATE and cleared at WM_NCDESTROY.
            if let Some(context) = unsafe { window_context_mut(hwnd) }
                && update_main_splitter_drag(hwnd, context, point_from_lparam(lparam))
            {
                0
            } else if let Some(context) = unsafe { window_context_mut(hwnd) }
                && context.workspace_tree_drag.is_some()
            {
                update_workspace_tree_drag_from_main_point(
                    hwnd,
                    context,
                    point_from_lparam(lparam),
                );
                0
            } else if let Some(context) = unsafe { window_context_mut(hwnd) }
                && point_hits_main_splitter(hwnd, context, point_from_lparam(lparam))
            {
                set_splitter_cursor();
                0
            } else {
                // SAFETY: Unhandled mouse messages are delegated to Windows.
                unsafe { DefWindowProcW(hwnd, message, wparam, lparam) }
            }
        }
        WM_LBUTTONUP => {
            // SAFETY: The userdata pointer is set during WM_NCCREATE and cleared at WM_NCDESTROY.
            if let Some(context) = unsafe { window_context_mut(hwnd) }
                && finish_main_splitter_drag(hwnd, context, point_from_lparam(lparam))
            {
                0
            } else if let Some(context) = unsafe { window_context_mut(hwnd) }
                && finish_workspace_tree_drag(hwnd, context, lparam)
            {
                0
            } else {
                // SAFETY: Unhandled mouse messages are delegated to Windows.
                unsafe { DefWindowProcW(hwnd, message, wparam, lparam) }
            }
        }
        WM_CANCELMODE | WM_CAPTURECHANGED => {
            // SAFETY: The userdata pointer is set during WM_NCCREATE and cleared at WM_NCDESTROY.
            if let Some(context) = unsafe { window_context_mut(hwnd) } {
                cancel_main_splitter_drag(context);
                cancel_workspace_tree_drag(context);
                cancel_command_button_drag(context);
            }
            // SAFETY: The default window procedure still gets capture/cancel bookkeeping.
            unsafe { DefWindowProcW(hwnd, message, wparam, lparam) }
        }
        WM_NOTIFY => {
            // SAFETY: The userdata pointer is set during WM_NCCREATE and cleared at WM_NCDESTROY.
            if let Some(context) = unsafe { window_context_mut(hwnd) } {
                handle_main_notify(hwnd, context, lparam)
            } else {
                0
            }
        }
        WM_ERASEBKGND => {
            // SAFETY: The userdata pointer is set during WM_NCCREATE and cleared at WM_NCDESTROY.
            if let Some(context) = unsafe { window_context_mut(hwnd) }
                && let Some(result) = erase_window_background(hwnd, context, wparam)
            {
                result
            } else {
                // SAFETY: Unhandled erase requests are delegated to the default window procedure.
                unsafe { DefWindowProcW(hwnd, message, wparam, lparam) }
            }
        }
        WM_CTLCOLORSTATIC => {
            // SAFETY: The userdata pointer is set during WM_NCCREATE and cleared at WM_NCDESTROY.
            if let Some(context) = unsafe { window_context_mut(hwnd) }
                && let Some(result) = themed_static_color_brush(context, wparam)
            {
                result
            } else {
                // SAFETY: Unhandled control color requests are delegated to Windows.
                unsafe { DefWindowProcW(hwnd, message, wparam, lparam) }
            }
        }
        WM_CTLCOLORBTN => {
            // SAFETY: The userdata pointer is set during WM_NCCREATE and cleared at WM_NCDESTROY.
            if let Some(context) = unsafe { window_context_mut(hwnd) }
                && let Some(result) = themed_button_color_brush(context, wparam, lparam)
            {
                result
            } else {
                // SAFETY: Unhandled control color requests are delegated to Windows.
                unsafe { DefWindowProcW(hwnd, message, wparam, lparam) }
            }
        }
        WM_DESTROY => {
            // SAFETY: The userdata pointer is valid until WM_NCDESTROY. Revoke the OLE drop
            // target before the child TreeView window is destroyed.
            if let Some(context) = unsafe { window_context_mut(hwnd) } {
                remove_tree_view_subclass(context);
                context.drop_target = None;
                context.workspace_drop_check = None;
            }
            // SAFETY: Posting quit from the UI thread is the normal shutdown path.
            unsafe {
                PostQuitMessage(0);
            }
            0
        }
        WM_NCDESTROY => {
            // SAFETY: Clear the non-owning context pointer before the window handle dies.
            unsafe {
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
                DefWindowProcW(hwnd, message, wparam, lparam)
            }
        }
        _ => {
            // SAFETY: All messages not handled by this window procedure are delegated to Windows.
            unsafe { DefWindowProcW(hwnd, message, wparam, lparam) }
        }
    }
}

unsafe fn window_context_mut(hwnd: HWND) -> Option<&'static mut WindowContext> {
    // SAFETY: The value was stored by this module as a WindowContext pointer.
    let pointer = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) } as *mut WindowContext;
    if pointer.is_null() {
        None
    } else {
        // SAFETY: The pointer is non-owning and valid for the current message dispatch.
        unsafe { pointer.as_mut() }
    }
}

unsafe fn command_tab_page_context_mut(hwnd: HWND) -> Option<&'static mut WindowContext> {
    // SAFETY: The value was stored by this module as a WindowContext pointer.
    let pointer = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) } as *mut WindowContext;
    if pointer.is_null() {
        None
    } else {
        // SAFETY: The pointer is non-owning and valid for the current message dispatch.
        unsafe { pointer.as_mut() }
    }
}

fn low_word(value: WPARAM) -> u32 {
    (value & 0xffff) as u32
}

fn high_word(value: WPARAM) -> u32 {
    ((value >> 16) & 0xffff) as u32
}

fn signed_high_word_from_wparam(value: WPARAM) -> i32 {
    ((value >> 16) & 0xffff) as u16 as i16 as i32
}

fn point_from_lparam(value: LPARAM) -> POINT {
    POINT {
        x: signed_low_word(value),
        y: signed_high_word(value),
    }
}

fn signed_low_word(value: LPARAM) -> i32 {
    (value as u32 & 0xffff) as i16 as i32
}

fn signed_high_word(value: LPARAM) -> i32 {
    ((value as u32 >> 16) & 0xffff) as i16 as i32
}

fn system_color_brush(color: i32) -> HBRUSH {
    ((color + 1) as usize) as HBRUSH
}

fn last_error(operation: &'static str) -> AppError {
    // SAFETY: GetLastError reads the calling thread's last-error code.
    let code = unsafe { GetLastError() };
    AppError::windows_api(operation, code)
}

fn wide_null(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(once(0)).collect()
}

fn path_wide_null(path: &Path) -> Vec<u16> {
    path.as_os_str().encode_wide().chain(once(0)).collect()
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::io;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::command_execution::{
        CommandExecutionError, argument_token_execution_value,
        command_line_from_executable_and_arguments, command_workspace_error,
        execute_external_terminal_command, external_terminal_parameters_for_command,
        prepare_command_arguments, quote_windows_command_argument, shell_execute_open,
    };
    use super::persistence::{SettingsRestorePoint, restore_state_after_failed_persist};
    use super::*;
    use crate::domain::{
        AppSettings, AppState, ArgumentResolutionError, ArgumentToken, DEFAULT_DPI,
    };

    #[test]
    fn denied_workspace_drop_still_reaches_drop_handler() {
        assert_eq!(
            drop_effect_for_feedback(DropFeedback::Denied),
            DROPEFFECT_COPY
        );
    }

    #[test]
    fn normal_drag_state_does_not_advertise_a_drop_effect() {
        assert_eq!(
            drop_effect_for_feedback(DropFeedback::Normal),
            DROPEFFECT_NONE
        );
    }

    #[test]
    fn workspace_tree_drop_destination_accounts_for_removed_source() {
        assert_eq!(
            super::layout_rules::workspace_tree_drop_destination(
                1,
                WorkspaceTreeDropTarget::Workspace {
                    index: 3,
                    insert_after: false,
                },
                4,
            ),
            Some(2)
        );
        assert_eq!(
            super::layout_rules::workspace_tree_drop_destination(
                3,
                WorkspaceTreeDropTarget::Workspace {
                    index: 1,
                    insert_after: true,
                },
                4,
            ),
            Some(2)
        );
    }

    #[test]
    fn workspace_tree_drop_destination_ignores_noop_targets() {
        assert_eq!(
            super::layout_rules::workspace_tree_drop_destination(
                1,
                WorkspaceTreeDropTarget::Workspace {
                    index: 2,
                    insert_after: false,
                },
                4,
            ),
            None
        );
        assert_eq!(
            super::layout_rules::workspace_tree_drop_destination(
                1,
                WorkspaceTreeDropTarget::Workspace {
                    index: 1,
                    insert_after: true,
                },
                4,
            ),
            None
        );
    }

    #[test]
    fn workspace_tree_drop_destination_ignores_category_targets() {
        assert_eq!(
            super::layout_rules::workspace_tree_drop_destination(
                0,
                WorkspaceTreeDropTarget::Category { index: 0 },
                2,
            ),
            None
        );
    }

    #[test]
    fn workspace_tree_visible_group_drop_destination_allows_same_group_targets() {
        let categories = vec![Category::new("Backend").expect("category should be valid")];
        let workspaces = vec![
            Workspace::new("C:\\projects\\api", "api", "Rust")
                .expect("workspace should be valid")
                .with_category(Some("Backend".to_owned())),
            Workspace::new("C:\\projects\\root-a", "root-a", "Rust")
                .expect("workspace should be valid"),
            Workspace::new("C:\\projects\\worker", "worker", "Rust")
                .expect("workspace should be valid")
                .with_category(Some("Backend".to_owned())),
            Workspace::new("C:\\projects\\root-b", "root-b", "Rust")
                .expect("workspace should be valid"),
        ];

        assert_eq!(
            super::layout_rules::workspace_tree_visible_group_drop_destination(
                &workspaces,
                &categories,
                0,
                WorkspaceTreeDropTarget::Workspace {
                    index: 2,
                    insert_after: true,
                },
            ),
            Some(2)
        );
        assert_eq!(
            super::layout_rules::workspace_tree_visible_group_drop_destination(
                &workspaces,
                &categories,
                1,
                WorkspaceTreeDropTarget::Workspace {
                    index: 3,
                    insert_after: true,
                },
            ),
            Some(3)
        );
    }

    #[test]
    fn workspace_tree_visible_group_drop_destination_ignores_cross_group_targets() {
        let categories = vec![
            Category::new("Backend").expect("category should be valid"),
            Category::new("Frontend").expect("category should be valid"),
        ];
        let workspaces = vec![
            Workspace::new("C:\\projects\\api", "api", "Rust")
                .expect("workspace should be valid")
                .with_category(Some("Backend".to_owned())),
            Workspace::new("C:\\projects\\root", "root", "Rust")
                .expect("workspace should be valid"),
            Workspace::new("C:\\projects\\web", "web", "Rust")
                .expect("workspace should be valid")
                .with_category(Some("Frontend".to_owned())),
        ];

        assert_eq!(
            super::layout_rules::workspace_tree_visible_group_drop_destination(
                &workspaces,
                &categories,
                0,
                WorkspaceTreeDropTarget::Workspace {
                    index: 1,
                    insert_after: false,
                },
            ),
            None
        );
        assert_eq!(
            super::layout_rules::workspace_tree_visible_group_drop_destination(
                &workspaces,
                &categories,
                1,
                WorkspaceTreeDropTarget::Workspace {
                    index: 0,
                    insert_after: true,
                },
            ),
            None
        );
        assert_eq!(
            super::layout_rules::workspace_tree_visible_group_drop_destination(
                &workspaces,
                &categories,
                0,
                WorkspaceTreeDropTarget::Workspace {
                    index: 2,
                    insert_after: true,
                },
            ),
            None
        );
    }

    #[test]
    fn keyboard_workspace_move_uses_visible_group_order() {
        let categories = vec![Category::new("Backend").expect("category should be valid")];
        let workspaces = vec![
            Workspace::new("C:\\projects\\api", "api", "Rust")
                .expect("workspace should be valid")
                .with_category(Some("Backend".to_owned())),
            Workspace::new("C:\\projects\\root-a", "root-a", "Rust")
                .expect("workspace should be valid"),
            Workspace::new("C:\\projects\\worker", "worker", "Rust")
                .expect("workspace should be valid")
                .with_category(Some("Backend".to_owned())),
            Workspace::new("C:\\projects\\root-b", "root-b", "Rust")
                .expect("workspace should be valid"),
        ];

        assert_eq!(
            super::layout_rules::workspace_keyboard_move_destination(
                &workspaces,
                &categories,
                2,
                TreeKeyboardMoveDirection::Up,
            ),
            Some(0)
        );
        assert_eq!(
            super::layout_rules::workspace_keyboard_move_destination(
                &workspaces,
                &categories,
                1,
                TreeKeyboardMoveDirection::Down,
            ),
            Some(3)
        );
    }

    #[test]
    fn keyboard_workspace_move_ignores_group_boundaries() {
        let categories = vec![Category::new("Backend").expect("category should be valid")];
        let workspaces = vec![
            Workspace::new("C:\\projects\\api", "api", "Rust")
                .expect("workspace should be valid")
                .with_category(Some("Backend".to_owned())),
            Workspace::new("C:\\projects\\root", "root", "Rust")
                .expect("workspace should be valid"),
        ];

        assert_eq!(
            super::layout_rules::workspace_keyboard_move_destination(
                &workspaces,
                &categories,
                0,
                TreeKeyboardMoveDirection::Up,
            ),
            None
        );
        assert_eq!(
            super::layout_rules::workspace_keyboard_move_destination(
                &workspaces,
                &categories,
                1,
                TreeKeyboardMoveDirection::Down,
            ),
            None
        );
    }

    #[test]
    fn keyboard_root_move_uses_mixed_tree_order() {
        let root_items = vec![
            TreeRootItemRef::Category(0),
            TreeRootItemRef::Workspace(0),
            TreeRootItemRef::Workspace(1),
            TreeRootItemRef::Category(1),
        ];

        assert_eq!(
            super::layout_rules::tree_root_keyboard_move_destination(
                &root_items,
                TreeRootItemRef::Workspace(0),
                TreeKeyboardMoveDirection::Up,
            ),
            Some(0),
        );
        assert_eq!(
            super::layout_rules::tree_root_keyboard_move_destination(
                &root_items,
                TreeRootItemRef::Workspace(1),
                TreeKeyboardMoveDirection::Down,
            ),
            Some(3),
        );
    }

    #[test]
    fn keyboard_root_move_ignores_boundaries() {
        let root_items = vec![TreeRootItemRef::Category(0), TreeRootItemRef::Workspace(0)];

        assert_eq!(
            super::layout_rules::tree_root_keyboard_move_destination(
                &root_items,
                TreeRootItemRef::Category(0),
                TreeKeyboardMoveDirection::Up,
            ),
            None,
        );
        assert_eq!(
            super::layout_rules::tree_root_keyboard_move_destination(
                &root_items,
                TreeRootItemRef::Workspace(0),
                TreeKeyboardMoveDirection::Down,
            ),
            None,
        );
        assert_eq!(
            super::layout_rules::tree_root_keyboard_move_destination(
                &root_items,
                TreeRootItemRef::Category(2),
                TreeKeyboardMoveDirection::Up,
            ),
            None,
        );
    }

    #[test]
    fn root_drop_destination_accounts_for_removed_source() {
        let root_items = vec![
            TreeRootItemRef::Category(0),
            TreeRootItemRef::Workspace(0),
            TreeRootItemRef::Workspace(1),
            TreeRootItemRef::Category(1),
        ];

        assert_eq!(
            super::layout_rules::tree_root_drop_destination(
                &root_items,
                TreeRootItemRef::Workspace(0),
                TreeRootItemRef::Category(1),
                false,
            ),
            Some(2),
        );
        assert_eq!(
            super::layout_rules::tree_root_drop_destination(
                &root_items,
                TreeRootItemRef::Category(1),
                TreeRootItemRef::Workspace(0),
                true,
            ),
            Some(2),
        );
    }

    #[test]
    fn command_button_drop_destination_uses_target_position() {
        assert_eq!(
            super::layout_rules::command_button_drop_destination(0, 2, 4),
            Some(2)
        );
        assert_eq!(
            super::layout_rules::command_button_drop_destination(3, 1, 4),
            Some(1)
        );
    }

    #[test]
    fn command_button_drop_destination_ignores_invalid_or_same_targets() {
        assert_eq!(
            super::layout_rules::command_button_drop_destination(1, 1, 4),
            None
        );
        assert_eq!(
            super::layout_rules::command_button_drop_destination(4, 1, 4),
            None
        );
        assert_eq!(
            super::layout_rules::command_button_drop_destination(1, 4, 4),
            None
        );
        assert_eq!(
            super::layout_rules::command_button_drop_destination(0, 1, 1),
            None
        );
    }

    #[test]
    fn command_button_click_ignores_category_tree_selection() {
        assert!(command_button_click_ignored_for_tree_selection(Some(
            TreeNodeSelection::Category(0)
        )));
        assert!(!command_button_click_ignored_for_tree_selection(Some(
            TreeNodeSelection::Workspace(0)
        )));
        assert!(!command_button_click_ignored_for_tree_selection(None));
    }

    #[test]
    fn command_button_context_menu_state_enables_selected_button_actions() {
        assert_eq!(
            command_button_context_menu_state(
                true,
                true,
                Some(1),
                3,
                Some(TreeNodeSelection::Workspace(0))
            ),
            CommandButtonContextMenuState {
                can_execute: true,
                can_edit: true,
                can_delete: true,
                can_move_previous: true,
                can_move_next: true,
                can_add_command: true,
            }
        );
    }

    #[test]
    fn command_button_context_menu_state_disables_execute_for_category_selection() {
        let state = command_button_context_menu_state(
            true,
            true,
            Some(1),
            3,
            Some(TreeNodeSelection::Category(0)),
        );

        assert!(!state.can_execute);
        assert!(state.can_edit);
        assert!(state.can_delete);
        assert!(state.can_move_previous);
        assert!(state.can_move_next);
        assert!(state.can_add_command);
    }

    #[test]
    fn command_button_context_menu_state_disables_button_actions_without_button() {
        assert_eq!(
            command_button_context_menu_state(true, false, None, 0, None),
            CommandButtonContextMenuState {
                can_execute: false,
                can_edit: false,
                can_delete: false,
                can_move_previous: false,
                can_move_next: false,
                can_add_command: true,
            }
        );
    }

    #[test]
    fn command_button_move_destination_moves_one_step_vertically() {
        assert_eq!(
            super::layout_rules::command_button_move_destination(
                1,
                3,
                CommandButtonMoveDirection::Previous,
            ),
            Some(0)
        );
        assert_eq!(
            super::layout_rules::command_button_move_destination(
                1,
                3,
                CommandButtonMoveDirection::Next,
            ),
            Some(2)
        );
    }

    #[test]
    fn command_button_move_destination_ignores_edges_and_invalid_indices() {
        assert_eq!(
            super::layout_rules::command_button_move_destination(
                0,
                3,
                CommandButtonMoveDirection::Previous,
            ),
            None
        );
        assert_eq!(
            super::layout_rules::command_button_move_destination(
                2,
                3,
                CommandButtonMoveDirection::Next,
            ),
            None
        );
        assert_eq!(
            super::layout_rules::command_button_move_destination(
                3,
                3,
                CommandButtonMoveDirection::Previous,
            ),
            None
        );
        assert_eq!(
            super::layout_rules::command_button_move_destination(
                0,
                1,
                CommandButtonMoveDirection::Next,
            ),
            None
        );
    }

    #[test]
    fn command_tab_move_destination_moves_one_step_horizontally() {
        assert_eq!(
            super::layout_rules::command_tab_move_destination(1, 3, CommandTabMoveDirection::Left,),
            Some(0)
        );
        assert_eq!(
            super::layout_rules::command_tab_move_destination(1, 3, CommandTabMoveDirection::Right,),
            Some(2)
        );
    }

    #[test]
    fn command_tab_move_destination_ignores_edges_and_invalid_indices() {
        assert_eq!(
            super::layout_rules::command_tab_move_destination(0, 3, CommandTabMoveDirection::Left,),
            None
        );
        assert_eq!(
            super::layout_rules::command_tab_move_destination(2, 3, CommandTabMoveDirection::Right,),
            None
        );
        assert_eq!(
            super::layout_rules::command_tab_move_destination(3, 3, CommandTabMoveDirection::Left,),
            None
        );
        assert_eq!(
            super::layout_rules::command_tab_move_destination(0, 1, CommandTabMoveDirection::Right,),
            None
        );
    }

    #[test]
    fn theme_menu_commands_round_trip_all_view_themes() {
        for theme in ViewTheme::options().iter().copied() {
            let command = command_for_view_theme(theme);
            assert_eq!(view_theme_for_command(command), Some(theme));
        }
    }

    #[test]
    fn system_tree_theme_uses_system_colors() {
        let palette = ThemePalette::for_theme(ViewTheme::System);
        let colors = TreeViewThemeColors::for_palette(palette);

        assert_eq!(colors.background, TREE_VIEW_USE_SYSTEM_COLOR);
        assert_eq!(colors.text, TREE_VIEW_USE_SYSTEM_COLOR);
        assert_eq!(colors.line, CLR_DEFAULT as LPARAM);
    }

    #[test]
    fn custom_tree_themes_use_explicit_palette_colors() {
        for theme in ViewTheme::options()
            .iter()
            .copied()
            .filter(|theme| theme.uses_dark_mode())
        {
            let palette = ThemePalette::for_theme(theme);
            let colors = TreeViewThemeColors::for_palette(palette);

            assert_eq!(colors.background, palette.control_background as LPARAM);
            assert_eq!(colors.text, palette.control_text as LPARAM);
            assert_eq!(colors.line, palette.tree_line as LPARAM);
        }
    }

    #[test]
    fn normal_drop_feedback_restores_tree_theme_colors() {
        for theme in ViewTheme::options().iter().copied() {
            let palette = ThemePalette::for_theme(theme);

            assert_eq!(
                TreeViewThemeColors::for_drop_feedback(DropFeedback::Normal, palette),
                TreeViewThemeColors::for_palette(palette)
            );
        }
    }

    #[test]
    fn light_drop_feedback_keeps_light_highlight_colors() {
        let palette = ThemePalette::for_theme(ViewTheme::Light);
        let allowed = TreeViewThemeColors::for_drop_feedback(DropFeedback::Allowed, palette);
        let denied = TreeViewThemeColors::for_drop_feedback(DropFeedback::Denied, palette);

        assert_eq!(allowed.background, colorref(226, 241, 255) as LPARAM);
        assert_eq!(denied.background, colorref(255, 235, 235) as LPARAM);
        assert_eq!(allowed.text, TREE_VIEW_USE_SYSTEM_COLOR);
        assert_eq!(denied.text, TREE_VIEW_USE_SYSTEM_COLOR);
    }

    #[test]
    fn command_button_drop_target_uses_allowed_feedback_background() {
        let palette = ThemePalette::for_theme(ViewTheme::Light);

        assert_eq!(
            command_button_drop_target_background(palette),
            colorref(226, 241, 255)
        );
    }

    #[test]
    fn command_button_drop_target_blends_with_custom_theme_background() {
        let palette = ThemePalette::for_theme(ViewTheme::Graphite);
        let target_background = command_button_drop_target_background(palette);

        assert_ne!(target_background, palette.control_background);
        assert_ne!(target_background, colorref(226, 241, 255));
    }

    #[test]
    fn workspace_tree_drop_target_uses_allowed_feedback_background() {
        let palette = ThemePalette::for_theme(ViewTheme::Light);

        assert_eq!(
            workspace_tree_drop_target_background(palette),
            colorref(226, 241, 255)
        );
    }

    #[test]
    fn workspace_tree_drop_target_blends_with_custom_theme_background() {
        let palette = ThemePalette::for_theme(ViewTheme::Forest);
        let target_background = workspace_tree_drop_target_background(palette);

        assert_ne!(target_background, palette.control_background);
        assert_ne!(target_background, colorref(226, 241, 255));
    }

    #[test]
    fn dark_drop_feedback_keeps_text_and_line_contrast() {
        for theme in ViewTheme::options()
            .iter()
            .copied()
            .filter(|theme| theme.uses_dark_mode())
        {
            let palette = ThemePalette::for_theme(theme);

            for feedback in [DropFeedback::Allowed, DropFeedback::Denied] {
                let colors = TreeViewThemeColors::for_drop_feedback(feedback, palette);

                assert_ne!(colors.background, palette.control_background as LPARAM);
                assert_ne!(colors.background, colorref(226, 241, 255) as LPARAM);
                assert_ne!(colors.background, colorref(255, 235, 235) as LPARAM);
                assert_eq!(colors.text, palette.control_text as LPARAM);
                assert_eq!(colors.line, palette.tree_line as LPARAM);
            }
        }
    }

    #[test]
    fn tree_item_lparam_separates_categories_from_workspaces() {
        let workspace_lparam = workspace_tree_lparam(0).expect("workspace lparam should fit");
        let category_lparam = category_tree_lparam(0).expect("category lparam should fit");

        assert_eq!(workspace_index_from_tree_lparam(workspace_lparam), Some(0));
        assert_eq!(workspace_index_from_tree_lparam(category_lparam), None);
        assert_eq!(category_index_from_tree_lparam(category_lparam), Some(0));
        assert_eq!(category_index_from_tree_lparam(workspace_lparam), None);
        assert!(category_lparam < 0);
    }

    #[test]
    fn hdrop_path_count_capacity_rejects_excessive_counts() {
        assert_eq!(hdrop_path_count_capacity(MAX_HDROP_PATHS_TO_READ), Ok(16));
        assert_eq!(
            hdrop_path_count_capacity(MAX_HDROP_PATHS_TO_READ + 1),
            Err(DropDataError::TooManyHdropItems)
        );
    }

    #[test]
    fn hdrop_path_buffer_len_rejects_oversized_lengths() {
        assert_eq!(hdrop_path_buffer_len(0), Ok(1));
        assert_eq!(hdrop_path_buffer_len(MAX_HDROP_PATH_CHARS), Ok(32_768));
        assert_eq!(
            hdrop_path_buffer_len(MAX_HDROP_PATH_CHARS + 1),
            Err(DropDataError::InvalidHdropPathLength)
        );
    }

    #[test]
    fn hdrop_copied_path_len_rejects_unexpected_copy_lengths() {
        assert_eq!(hdrop_copied_path_len(4, 4), Ok(4));
        assert_eq!(
            hdrop_copied_path_len(5, 4),
            Err(DropDataError::InvalidHdropPathLength)
        );
    }

    #[test]
    fn minimum_client_size_keeps_tree_and_tabs_visible_at_large_font_and_dpi() {
        let dpi = 144;
        let layout = LayoutSpec::for_font_size_and_dpi(20, dpi);
        let minimum = minimum_main_client_size(layout, dpi);

        assert_eq!(minimum.width, 330);
        assert_eq!(minimum.height, 267);
    }

    #[test]
    fn startup_window_size_uses_window_pixel_settings_without_dpi_scaling() {
        let settings = AppSettings {
            view: ViewSettings::new(DEFAULT_FONT_FAMILY, DEFAULT_FONT_SIZE, "graphite")
                .with_window_layout(Some(484), Some(613), Some(160)),
            ..AppSettings::default()
        };
        let spec = MainWindowSpec::initial(AppState::from_settings(settings, Vec::new()));

        assert_eq!(
            startup_window_size(&spec),
            WindowSize {
                width: 484,
                height: 613,
            }
        );
    }

    #[test]
    fn startup_layout_restores_saved_tree_panel_pixels_without_dpi_scaling() {
        let saved = ViewSettings::new(DEFAULT_FONT_FAMILY, DEFAULT_FONT_SIZE, "graphite")
            .with_window_layout(Some(484), Some(613), Some(160));
        let default = ViewSettings::new(DEFAULT_FONT_FAMILY, DEFAULT_FONT_SIZE, "graphite");

        assert_eq!(startup_layout(&saved, 144).tree_panel_width, 160);
        assert_eq!(startup_layout(&default, 144).tree_panel_width, 240);
    }

    #[test]
    fn persisted_window_layout_keeps_current_window_pixels() {
        let current = ViewSettings::new(DEFAULT_FONT_FAMILY, DEFAULT_FONT_SIZE, "graphite");
        let view = view_settings_with_window_layout(
            &current,
            WindowSize {
                width: 726,
                height: 920,
            },
            240,
        );

        assert_eq!(view.window_width, Some(726));
        assert_eq!(view.window_height, Some(920));
        assert_eq!(view.tree_panel_width, Some(240));
    }

    #[test]
    fn scale_dimension_between_dpi_saturates_large_positive_dimensions() {
        let scaled = scale_dimension_between_dpi(i32::MAX, DEFAULT_DPI, DEFAULT_DPI * 2);

        assert_eq!(scaled, i32::MAX);
        assert!(scaled > 0);
    }

    #[test]
    fn dialog_font_scaling_keeps_default_size_and_expands_large_fonts() {
        let command_dialog_width = 570;

        assert_eq!(
            scale_dimension_for_dialog_font(command_dialog_width, DEFAULT_FONT_SIZE),
            command_dialog_width
        );
        assert_eq!(scale_dimension_for_dialog_font(24, 9), 24);
        assert_eq!(scale_dimension_for_dialog_font(24, 20), 36);
        assert_eq!(
            scale_dimension_for_dialog_font(command_dialog_width, 20),
            855
        );
    }

    #[test]
    fn dialog_layout_font_size_uses_larger_actual_font_size() {
        assert_eq!(
            dialog_layout_font_size_from_actual(DEFAULT_FONT_SIZE, None),
            DEFAULT_FONT_SIZE
        );
        assert_eq!(
            dialog_layout_font_size_from_actual(DEFAULT_FONT_SIZE, Some(20)),
            20
        );
        assert_eq!(dialog_layout_font_size_from_actual(20, Some(12)), 20);
        assert_eq!(dialog_layout_font_size_from_actual(8, Some(24)), 20);
    }

    #[test]
    fn font_size_from_logfont_height_uses_current_dpi() {
        assert_eq!(font_size_from_logfont_height(-24, 144), Some(12));
        assert_eq!(font_size_from_logfont_height(-40, 144), Some(20));
        assert_eq!(font_size_from_logfont_height(20, 96), Some(15));
        assert_eq!(font_size_from_logfont_height(0, 96), None);
    }

    #[test]
    fn dialog_rect_scaling_combines_font_size_and_dpi() {
        let rect = scale_rect_for_dialog(
            RectSpec {
                x: 132,
                y: 60,
                width: 80,
                height: 26,
            },
            20,
            144,
        );

        assert_eq!(
            rect,
            RectSpec {
                x: 297,
                y: 135,
                width: 180,
                height: 59,
            }
        );
    }

    #[test]
    fn splitter_width_clamps_to_tree_and_tabs_minimums() {
        let dpi = 96;
        let layout = LayoutSpec::for_font_size_and_dpi(DEFAULT_FONT_SIZE, dpi);
        let client = ClientSize {
            width: 500,
            height: 400,
        };

        assert_eq!(clamp_tree_panel_width(layout, client, dpi, 10), 72);
        assert_eq!(clamp_tree_panel_width(layout, client, dpi, 160), 160);
        assert_eq!(clamp_tree_panel_width(layout, client, dpi, 600), 358);
    }

    #[test]
    fn splitter_drag_width_uses_client_pointer_x() {
        let dpi = 96;
        let layout = LayoutSpec::for_font_size_and_dpi(DEFAULT_FONT_SIZE, dpi);
        let client = ClientSize {
            width: 500,
            height: 400,
        };

        assert_eq!(
            splitter_drag_tree_panel_width(layout, client, dpi, POINT { x: 232, y: 40 }),
            224
        );
    }

    #[test]
    fn splitter_hit_rect_covers_gap_between_main_panels() {
        let layout =
            LayoutSpec::for_font_size(DEFAULT_FONT_SIZE).arrange_main_content(ClientSize {
                width: 500,
                height: 400,
            });
        let hit_rect = main_splitter_hit_rect(layout);

        assert_eq!(
            hit_rect,
            RectSpec {
                x: 168,
                y: 6,
                width: 10,
                height: 386,
            }
        );
        assert!(point_in_rect_spec(POINT { x: 168, y: 6 }, hit_rect));
        assert!(point_in_rect_spec(POINT { x: 177, y: 391 }, hit_rect));
        assert!(!point_in_rect_spec(POINT { x: 178, y: 40 }, hit_rect));
    }

    #[test]
    fn command_button_rects_keep_grid_order_without_precollecting() {
        let panel = RectSpec {
            x: 20,
            y: 30,
            width: 320,
            height: 240,
        };
        let rects = command_button_rects(panel, 5, DEFAULT_FONT_SIZE, 96);
        assert_eq!(rects.len(), 5);

        let rects: Vec<_> = rects.collect();

        assert_eq!(rects[0].x, 30);
        assert_eq!(rects[0].y, 64);
        assert_eq!(rects[0].width, 132);
        assert_eq!(rects[0].height, 30);
        assert_eq!(rects[1].x, 170);
        assert_eq!(rects[1].y, 64);
        assert_eq!(rects[2].x, 30);
        assert_eq!(rects[2].y, 102);
        assert_eq!(rects[3].x, 170);
        assert_eq!(rects[3].y, 102);
        assert_eq!(rects[4].x, 30);
        assert_eq!(rects[4].y, 140);
    }

    #[test]
    fn command_button_rects_shrink_to_narrow_panel() {
        let panel = RectSpec {
            x: 20,
            y: 30,
            width: 116,
            height: 240,
        };

        let rects: Vec<_> = command_button_rects(panel, 2, DEFAULT_FONT_SIZE, 96).collect();

        assert_eq!(rects[0].x, 30);
        assert_eq!(rects[0].width, 96);
        assert_eq!(rects[1].x, 30);
        assert_eq!(rects[1].y, 102);
    }

    #[test]
    fn command_button_window_style_keeps_buttons_inside_page_panel() {
        let style = command_button_window_style();

        assert_ne!(style & BS_LEFT as u32, 0);
        assert_ne!(style & WS_CLIPSIBLINGS, 0);
    }

    #[test]
    fn command_tab_page_style_is_visible_page_container() {
        let style = command_tab_page_window_style();

        assert_ne!(style & WS_CHILD, 0);
        assert_ne!(style & WS_VISIBLE, 0);
        assert_ne!(style & WS_CLIPCHILDREN, 0);
        assert_ne!(style & WS_VSCROLL, 0);
        assert_eq!(style & WS_CLIPSIBLINGS, 0);
        assert_eq!(style & WS_TABSTOP, 0);
    }

    #[test]
    fn command_tab_selector_and_page_rect_reserve_dropdown_header() {
        let panel = RectSpec {
            x: 20,
            y: 30,
            width: 320,
            height: 240,
        };

        let selector = command_tab_selector_rect(panel, DEFAULT_FONT_SIZE, 96);
        let page = command_tab_page_rect(panel, DEFAULT_FONT_SIZE, 96);

        assert_eq!(selector.x, 20);
        assert_eq!(selector.y, 30);
        assert_eq!(selector.width, 320);
        assert_eq!(selector.height, 168);
        assert_eq!(page.x, 20);
        assert_eq!(page.y, 60);
        assert_eq!(page.width, 320);
        assert_eq!(page.height, 210);
    }

    #[test]
    fn command_button_rects_in_parent_keep_visual_position() {
        let panel = RectSpec {
            x: 20,
            y: 30,
            width: 320,
            height: 240,
        };
        let parent = RectSpec {
            x: 24,
            y: 60,
            width: 312,
            height: 206,
        };

        let rects: Vec<_> =
            command_button_rects_in_parent(panel, parent, 2, DEFAULT_FONT_SIZE, 96, 0).collect();

        assert_eq!(rects[0].x, 6);
        assert_eq!(rects[0].y, 4);
        assert_eq!(rects[1].x, 146);
        assert_eq!(rects[1].y, 4);
    }

    #[test]
    fn command_button_rects_in_parent_apply_scroll_offset() {
        let panel = RectSpec {
            x: 20,
            y: 30,
            width: 320,
            height: 240,
        };
        let parent = RectSpec {
            x: 24,
            y: 60,
            width: 312,
            height: 206,
        };

        let rects: Vec<_> =
            command_button_rects_in_parent(panel, parent, 2, DEFAULT_FONT_SIZE, 96, 20).collect();

        assert_eq!(rects[0].x, 6);
        assert_eq!(rects[0].y, -16);
        assert_eq!(rects[1].x, 146);
        assert_eq!(rects[1].y, -16);
    }

    #[test]
    fn command_button_rects_index_at_point_tracks_grid_cells_and_gaps() {
        let panel = RectSpec {
            x: 20,
            y: 30,
            width: 320,
            height: 240,
        };
        let parent = RectSpec {
            x: 24,
            y: 60,
            width: 312,
            height: 206,
        };

        let rects = command_button_rects_in_parent(panel, parent, 5, DEFAULT_FONT_SIZE, 96, 0);

        assert_eq!(rects.index_at_point(POINT { x: 6, y: 4 }), Some(0));
        assert_eq!(rects.index_at_point(POINT { x: 137, y: 33 }), Some(0));
        assert_eq!(rects.index_at_point(POINT { x: 138, y: 4 }), None);
        assert_eq!(rects.index_at_point(POINT { x: 146, y: 4 }), Some(1));
        assert_eq!(rects.index_at_point(POINT { x: 6, y: 34 }), None);
        assert_eq!(rects.index_at_point(POINT { x: 6, y: 42 }), Some(2));
        assert_eq!(rects.index_at_point(POINT { x: 146, y: 80 }), None);
    }

    #[test]
    fn command_button_rects_index_at_point_applies_scroll_offset() {
        let panel = RectSpec {
            x: 20,
            y: 30,
            width: 320,
            height: 240,
        };
        let parent = RectSpec {
            x: 24,
            y: 60,
            width: 312,
            height: 206,
        };

        let rects = command_button_rects_in_parent(panel, parent, 5, DEFAULT_FONT_SIZE, 96, 20);

        assert_eq!(rects.index_at_point(POINT { x: 6, y: 4 }), Some(0));
        assert_eq!(rects.index_at_point(POINT { x: 6, y: 14 }), None);
        assert_eq!(rects.index_at_point(POINT { x: 6, y: 22 }), Some(2));
    }

    #[test]
    fn command_button_max_scroll_offset_tracks_overflowing_rows() {
        let panel = RectSpec {
            x: 20,
            y: 30,
            width: 320,
            height: 240,
        };
        let parent = RectSpec {
            x: 24,
            y: 60,
            width: 312,
            height: 70,
        };

        assert_eq!(
            command_button_scroll_content_height(panel, parent, 5, DEFAULT_FONT_SIZE, 96),
            110
        );
        assert_eq!(
            command_button_max_scroll_offset(panel, parent, 5, DEFAULT_FONT_SIZE, 96),
            40
        );
        assert_eq!(
            command_button_max_scroll_offset(panel, parent, 2, DEFAULT_FONT_SIZE, 96),
            0
        );
    }

    #[test]
    fn command_button_effective_max_scroll_offset_aligns_up_to_rows() {
        let line_step = command_button_scroll_line_step(DEFAULT_FONT_SIZE, 96);

        assert_eq!(line_step, 38);
        assert_eq!(command_button_effective_max_scroll_offset(0, line_step), 0);
        assert_eq!(
            command_button_effective_max_scroll_offset(20, line_step),
            38
        );
        assert_eq!(
            command_button_effective_max_scroll_offset(50, line_step),
            76
        );
        assert_eq!(
            command_button_effective_max_scroll_offset(120, line_step),
            152
        );
    }

    #[test]
    fn command_button_aligned_scroll_offset_snaps_to_nearest_row() {
        let line_step = command_button_scroll_line_step(DEFAULT_FONT_SIZE, 96);
        let max_offset = command_button_effective_max_scroll_offset(120, line_step);

        assert_eq!(
            command_button_aligned_scroll_offset(18, max_offset, line_step),
            0
        );
        assert_eq!(
            command_button_aligned_scroll_offset(20, max_offset, line_step),
            38
        );
        assert_eq!(
            command_button_aligned_scroll_offset(75, max_offset, line_step),
            76
        );
        assert_eq!(
            command_button_aligned_scroll_offset(200, max_offset, line_step),
            152
        );
    }

    #[test]
    fn command_button_bottom_scroll_reveals_last_button() {
        let panel = RectSpec {
            x: 20,
            y: 30,
            width: 320,
            height: 240,
        };
        let parent = RectSpec {
            x: 24,
            y: 60,
            width: 312,
            height: 70,
        };
        let line_step = command_button_scroll_line_step(DEFAULT_FONT_SIZE, 96);
        let required_offset =
            command_button_max_scroll_offset(panel, parent, 5, DEFAULT_FONT_SIZE, 96);
        let max_offset = command_button_effective_max_scroll_offset(required_offset, line_step);

        let last =
            command_button_rects_in_parent(panel, parent, 5, DEFAULT_FONT_SIZE, 96, max_offset)
                .last()
                .expect("last command button should exist");

        assert!(last.y + last.height <= parent.height);
    }

    #[test]
    fn startup_font_validation_falls_back_when_saved_font_is_missing() {
        let settings = crate::domain::AppSettings {
            view: ViewSettings::new("Missing Codex Font", DEFAULT_FONT_SIZE, "system"),
            ..crate::domain::AppSettings::default()
        };
        let mut state = crate::domain::AppState::from_settings(settings, Vec::new());

        let warnings = validate_startup_font_settings_with(&mut state, |font_family| {
            font_family.eq_ignore_ascii_case(DEFAULT_FONT_FAMILY)
        });

        assert_eq!(state.settings().view.font_family, DEFAULT_FONT_FAMILY);
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].contains("Font not found"));
        assert!(state.status_message().contains("Missing Codex Font"));
    }

    #[test]
    fn font_family_normalization_sorts_and_deduplicates_without_case_sensitive_duplicates() {
        let fonts = normalize_font_family_list(vec![
            "Zed".to_owned(),
            "alpha".to_owned(),
            "ALPHA".to_owned(),
            "Beta".to_owned(),
        ]);

        assert_eq!(fonts, vec!["alpha", "Beta", "Zed"]);
    }

    #[test]
    fn failed_persist_restore_reverts_state_and_keeps_failure_status() {
        let previous_state = crate::domain::AppState::initial();
        let mut state = previous_state.clone();
        let workspace = Workspace::new("C:\\projects\\demo", "demo", "Rust")
            .expect("workspace should be valid");
        state
            .add_workspace(workspace)
            .expect("workspace should be added");
        assert_ne!(state.settings(), previous_state.settings());

        let error = settings::SettingsSaveError::Write {
            path: PathBuf::from("settings.toml"),
            source: io::Error::new(io::ErrorKind::PermissionDenied, "denied"),
        };

        let restore_point = SettingsRestorePoint::capture(&previous_state);

        restore_state_after_failed_persist(&mut state, restore_point, &error);

        assert_eq!(state.settings(), previous_state.settings());
        assert!(state.status_message().contains("Settings save failed"));
        assert!(state.status_message().contains("denied"));
    }

    #[test]
    fn workspace_drop_feedback_validation_allows_missing_single_paths_without_fs_access() {
        let paths = vec![PathBuf::from("Z:\\slow-or-missing\\workspace")];

        let result = validate_workspace_drop_paths_against_workspaces(&paths, &[]);

        assert_eq!(result, Ok(paths[0].as_path()));
    }

    #[test]
    fn workspace_drop_feedback_validation_rejects_duplicate_paths_without_fs_access() {
        let paths = vec![PathBuf::from("Z:\\slow-or-missing\\workspace")];
        let workspace = Workspace::new(paths[0].display().to_string(), "existing", "Rust")
            .expect("workspace should be valid");

        let result = validate_workspace_drop_paths_against_workspaces(&paths, &[workspace]);

        assert_eq!(
            result,
            Err(WorkspaceDropRejectReason::DuplicatePath {
                path: paths[0].clone(),
                name: "existing".to_owned(),
            })
        );
    }

    #[test]
    fn workspace_drop_validation_rejects_files() {
        let dir = unique_temp_dir("drop-file");
        fs::create_dir_all(&dir).expect("temp directory should be created");
        let file = dir.join("not-a-folder.txt");
        fs::write(&file, "not a folder").expect("temp file should be written");

        let result = inspect_workspace_drop_path_against_workspaces(&file, &[]).map(|_| ());

        assert_eq!(result, Err(WorkspaceDropRejectReason::NotFolder(file)));
        remove_temp_dir(&dir);
    }

    #[test]
    fn workspace_drop_validation_rejects_unreadable_or_missing_folders() {
        let path = unique_temp_dir("drop-missing").join("missing");

        let result = inspect_workspace_drop_path_against_workspaces(&path, &[]).map(|_| ());

        assert_eq!(
            result,
            Err(WorkspaceDropRejectReason::UnreadableFolder(path))
        );
    }

    #[test]
    fn workspace_drop_validation_rejects_duplicate_paths() {
        let dir = unique_temp_dir("drop-duplicate");
        fs::create_dir_all(&dir).expect("temp directory should be created");
        let workspace = Workspace::new(dir.display().to_string(), "existing", "Rust")
            .expect("workspace should be valid");

        let result = inspect_workspace_drop_path_against_workspaces(&dir, &[workspace]).map(|_| ());

        assert_eq!(
            result,
            Err(WorkspaceDropRejectReason::DuplicatePath {
                path: dir.clone(),
                name: "existing".to_owned(),
            })
        );
        remove_temp_dir(&dir);
    }

    #[test]
    fn workspace_language_inference_keeps_small_folder_detection() {
        let language =
            infer_workspace_language_from_limited_entry_names(vec![Some("Cargo.toml".to_owned())]);

        assert_eq!(language, Some("Rust"));
    }

    #[test]
    fn workspace_language_inference_does_not_read_past_entry_limit() {
        let entry_names = (0..WORKSPACE_LANGUAGE_INFERENCE_ENTRY_LIMIT)
            .map(|_| Some("notes.txt".to_owned()))
            .chain(std::iter::once_with(|| {
                panic!("entry past workspace language inference limit was read")
            }));

        assert!(infer_workspace_language_from_limited_entry_names(entry_names).is_none());
    }

    #[test]
    fn command_workspace_error_blocks_missing_context_and_inaccessible_paths() {
        let shell_without_tokens =
            CommandButton::new("Version", "tool.exe", "--version", ExecutionType::ShellApi)
                .expect("button should be valid");
        assert!(command_workspace_error(&shell_without_tokens, None).is_none());

        let shell_with_workspace_token =
            CommandButton::new("Name", "tool.exe", "{name}", ExecutionType::ShellApi)
                .expect("button should be valid");
        let shell_workspace_error = command_workspace_error(&shell_with_workspace_token, None)
            .expect("workspace token should require workspace");
        assert!(matches!(
            shell_workspace_error,
            CommandExecutionError::ArgumentResolution {
                source: ArgumentResolutionError::WorkspaceRequired
            }
        ));
        assert_eq!(
            shell_workspace_error.user_message(),
            "워크스페이스를 선택하세요."
        );

        let external_terminal =
            CommandButton::new("Check", "cargo", "check", ExecutionType::ExternalTerminal)
                .expect("button should be valid");
        let external_terminal_error = command_workspace_error(&external_terminal, None)
            .expect("external_terminal should require workspace");
        assert!(matches!(
            external_terminal_error,
            CommandExecutionError::MissingWorkspaceForExternalTerminal
        ));
        assert_eq!(
            external_terminal_error.user_message(),
            "워크스페이스를 선택하세요."
        );

        let missing_path = unique_temp_dir("missing-workspace").join("missing");
        let missing_workspace =
            Workspace::new(missing_path.display().to_string(), "missing", "Rust")
                .expect("workspace should be valid");
        let error = command_workspace_error(&shell_with_workspace_token, Some(&missing_workspace))
            .expect("missing workspace path should block execution");
        assert!(matches!(
            &error,
            CommandExecutionError::InaccessibleWorkspacePath { path }
                if path == &missing_workspace.path
        ));
        let message = error.user_message();

        assert!(message.contains("워크스페이스 폴더를 열 수 없습니다"));
        assert!(message.contains(&missing_workspace.path));
    }

    #[test]
    fn shell_api_argument_token_execution_values_are_quoted_as_single_windows_arguments() {
        assert_eq!(
            argument_token_execution_value(
                ExecutionType::ShellApi,
                ArgumentToken::InputText,
                "two words".to_owned(),
            ),
            "\"two words\""
        );
        assert_eq!(
            argument_token_execution_value(
                ExecutionType::ShellApi,
                ArgumentToken::Name,
                "Project".to_owned(),
            ),
            "Project"
        );
    }

    #[test]
    fn external_terminal_argument_token_execution_values_are_cmd_escaped() {
        assert_eq!(
            argument_token_execution_value(
                ExecutionType::ExternalTerminal,
                ArgumentToken::InputText,
                "two words".to_owned(),
            ),
            r#"^"two words^""#
        );
        assert_eq!(
            argument_token_execution_value(
                ExecutionType::ExternalTerminal,
                ArgumentToken::InputText,
                r#"say "hello" & %PATH% | !PATH!"#.to_owned(),
            ),
            r#"^"say \^"hello\^" ^& ^%PATH^% ^| ^!PATH^!^""#
        );
        assert_eq!(
            argument_token_execution_value(
                ExecutionType::ExternalTerminal,
                ArgumentToken::Name,
                "Project".to_owned(),
            ),
            "Project"
        );
    }

    #[test]
    fn prepare_command_arguments_does_not_replace_button_name_or_executable_path() {
        let workspace = Workspace::new(std::env::temp_dir().display().to_string(), "Temp", "Rust")
            .expect("workspace should be valid");
        let button = CommandButton::new(
            "{name}",
            "{path}",
            "--language {Language}",
            ExecutionType::ShellApi,
        )
        .expect("button should be valid");

        let arguments = prepare_command_arguments(
            null_mut(),
            null_mut(),
            null_mut(),
            DEFAULT_FONT_SIZE,
            UiLanguage::Korean,
            &button,
            Some(&workspace),
        )
        .expect("arguments should resolve")
        .expect("resolution should not be cancelled");

        assert_eq!(arguments, "--language Rust");
        assert_eq!(button.button_name, "{name}");
        assert_eq!(button.executable_path, "{path}");
    }

    #[test]
    fn prepare_command_arguments_returns_typed_unknown_token_error() {
        let button = CommandButton::new("Run", "tool.exe", "{missing}", ExecutionType::ShellApi)
            .expect("button should be valid");

        let error = prepare_command_arguments(
            null_mut(),
            null_mut(),
            null_mut(),
            DEFAULT_FONT_SIZE,
            UiLanguage::Korean,
            &button,
            None,
        )
        .expect_err("unknown token should fail argument preparation");

        match error {
            CommandExecutionError::ArgumentResolution { source } => {
                assert_eq!(
                    source,
                    ArgumentResolutionError::UnknownTokens(vec!["{missing}".to_owned()])
                );
            }
            other => panic!("unexpected command execution error: {other:?}"),
        }
    }

    #[test]
    fn quote_windows_command_argument_uses_windows_backslash_rules() {
        assert_eq!(quote_windows_command_argument("plain"), "plain");
        assert_eq!(
            quote_windows_command_argument(r"C:\Work Dir\"),
            r#""C:\Work Dir\\""#
        );
        assert_eq!(
            quote_windows_command_argument(r#"say "hello""#),
            r#""say \"hello\"""#
        );
    }

    #[test]
    fn external_terminal_parameters_use_cmd_s_wrapping_for_quoted_commands() {
        let command_line = command_line_from_executable_and_arguments(
            r"C:\Program Files\Tool\tool.exe",
            r#"--path "C:\Work Dir" --name "My App""#,
        );

        assert_eq!(
            command_line,
            r#""C:\Program Files\Tool\tool.exe" --path "C:\Work Dir" --name "My App""#
        );
        assert_eq!(
            external_terminal_parameters_for_command(&command_line),
            r#"/D /V:OFF /S /K ""C:\Program Files\Tool\tool.exe" --path "C:\Work Dir" --name "My App"""#
        );
    }

    #[test]
    fn shell_execute_open_rejects_interior_nul_before_win32_call() {
        assert_shell_execute_interior_nul_error(
            shell_execute_open(null_mut(), "cmd.exe\0evil.exe", "", None),
            "executable_path",
        );
        assert_shell_execute_interior_nul_error(
            shell_execute_open(null_mut(), "cmd.exe", "/D\0/C exit 0", None),
            "arguments",
        );
        assert_shell_execute_interior_nul_error(
            shell_execute_open(
                null_mut(),
                "cmd.exe",
                "/D /C exit 0",
                Some("C:\\Temp\0Other"),
            ),
            "directory",
        );
    }

    #[test]
    #[ignore = "starts a short-lived Windows process through ShellExecuteW"]
    fn shell_execute_open_smoke_starts_command_processor() {
        shell_execute_open(null_mut(), "cmd.exe", "/D /C exit 0", None)
            .expect("ShellExecuteW should start cmd.exe");
    }

    #[test]
    #[ignore = "starts a short-lived external_terminal cmd.exe process"]
    fn external_terminal_command_smoke_starts_command_processor_in_workspace() {
        let workspace = Workspace::new(std::env::temp_dir().display().to_string(), "temp", "Rust")
            .expect("workspace should be valid");

        execute_external_terminal_command(null_mut(), "exit", "", &workspace)
            .expect("external_terminal should start cmd.exe");
    }

    fn unique_temp_dir(label: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after Unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "j3devhelper-win32-{label}-{}-{nonce}",
            std::process::id()
        ))
    }

    fn assert_shell_execute_interior_nul_error(
        result: Result<(), CommandExecutionError>,
        expected_field: &'static str,
    ) {
        match result {
            Err(CommandExecutionError::InteriorNul { field, .. }) => {
                assert_eq!(field, expected_field);
            }
            other => panic!("unexpected shell execute result: {other:?}"),
        }
    }

    fn remove_temp_dir(path: &Path) {
        fs::remove_dir_all(path).expect("temp directory should be removed");
    }
}
