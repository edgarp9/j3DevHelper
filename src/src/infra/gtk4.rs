#![allow(deprecated)]

use std::cell::{Cell, RefCell};
use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::rc::Rc;
use std::sync::mpsc::TryRecvError;

use ::gtk4 as gtk;
use gtk::gdk::prelude::ToplevelExt;
use gtk::glib::variant::ToVariant;
use gtk::prelude::*;
use gtk::{
    Align, Application, ApplicationWindow, Box as GtkBox, Button, CheckButton, ComboBoxText,
    Dialog, Entry, FlowBox, FlowBoxChild, GestureClick, Grid, Label, ListBox, ListBoxRow,
    Orientation, Paned, Popover, PopoverMenuBar, ResponseType, ScrolledWindow, Separator, TextView,
};
use gtk::{gdk, gio, glib, pango};

#[cfg(test)]
use crate::domain::about_license_notice;
use crate::domain::{
    APP_ICON_PNG_FILE_NAME, APP_ICON_SVG_FILE_NAME, APP_LINUX_APPLICATION_ID, APP_REPOSITORY_URL,
    APP_TITLE, APP_VERSION, ARGUMENT_TOKENS, AppSettings, AppState, ArgumentResolutionError,
    ArgumentToken, Category, CommandButton, CommandButtonMoveDirection, CommandTab,
    CommandTabMoveDirection, DEFAULT_FONT_FAMILY, DEFAULT_FONT_SIZE, ExecutionType, LayoutSpec,
    MainWindowSpec, TreeKeyboardMoveDirection, TreeRootItemRef, UI_FONT_SIZE_OPTIONS, UiLanguage,
    ViewSettings, ViewTheme, WindowSize, Workspace, WorkspaceTreeDropAction,
    WorkspaceTreeDropTarget, about_license_heading, arguments_require_workspace,
    command_button_drop_destination, command_button_move_destination, command_tab_move_destination,
    current_ui_language, default_workspace_language_for_options,
    default_workspace_language_options, default_workspace_name_for_path,
    infer_workspace_language_from_entry_names, normalize_ui_font_size,
    normalize_workspace_language_options, replace_argument_tokens, resolve_argument_replacements,
    tree_root_keyboard_move_destination, unknown_argument_tokens, workspace_belongs_to_category,
    workspace_category_index, workspace_keyboard_move_destination, workspace_paths_equal,
    workspace_tree_drop_action,
};
use crate::error::AppResult;
use crate::infra::settings;

const CSS_PROVIDER_NAME: &str = "j3devhelper";
const TREE_ROW_DATA_PREFIX: &str = "tree:";
const COMMAND_ROW_DATA_PREFIX: &str = "command:";
const WORKSPACE_LANGUAGE_INFERENCE_ENTRY_LIMIT: usize = 256;
const MIN_TREE_PANEL_WIDTH: i32 = 72;
const MIN_COMMAND_TABS_PANEL_WIDTH: i32 = 116;
const MIN_MAIN_PANEL_HEIGHT: i32 = 160;
const COMMAND_BUTTON_PREFERRED_WIDTH: i32 = 132;
const COMMAND_BUTTON_HORIZONTAL_PADDING: i32 = 10;
const COMMAND_BUTTON_TOP_PADDING: i32 = 4;
const COMMAND_BUTTON_BASE_GAP: u32 = 8;
const ARGUMENT_TOKEN_COLUMNS: i32 = 3;
const ARGUMENT_TOKEN_BUTTON_WIDTH: i32 = 156;
const ARGUMENT_TOKEN_BUTTON_HEIGHT: i32 = 28;
const EXECUTABLE_FILE_PATTERNS: &[&str] = &["*.exe", "*.cmd", "*.bat", "*.ps1", "*.com"];
const ALL_FILE_PATTERNS: &[&str] = &["*"];
const ABOUT_DIALOG_WIDTH: i32 = 460;
const ABOUT_DIALOG_HEIGHT: i32 = 300;

pub fn run_main_window(spec: MainWindowSpec) -> AppResult<()> {
    let app = Application::builder()
        .application_id(APP_LINUX_APPLICATION_ID)
        .build();
    let spec = Rc::new(RefCell::new(Some(spec)));

    app.connect_activate(move |app| {
        let Some(spec) = spec.borrow_mut().take() else {
            return;
        };
        build_main_window(app, spec);
    });

    app.run_with_args(&[APP_LINUX_APPLICATION_ID]);
    Ok(())
}

#[derive(Clone)]
struct MenuActions {
    theme: gio::SimpleAction,
    ui_language: gio::SimpleAction,
    workspace_edit: gio::SimpleAction,
    workspace_move_up: gio::SimpleAction,
    workspace_move_down: gio::SimpleAction,
    workspace_delete: gio::SimpleAction,
    tab_rename: gio::SimpleAction,
    tab_move_up: gio::SimpleAction,
    tab_move_down: gio::SimpleAction,
    tab_delete: gio::SimpleAction,
    command_run: gio::SimpleAction,
    command_add: gio::SimpleAction,
    command_edit: gio::SimpleAction,
    command_move_previous: gio::SimpleAction,
    command_move_next: gio::SimpleAction,
    command_delete: gio::SimpleAction,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum TreeSelection {
    Workspace(usize),
    Category(usize),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum TreeRowRef {
    Workspace(usize),
    Category(usize),
}

struct GtkWindowContext {
    window: ApplicationWindow,
    menu_bar: PopoverMenuBar,
    paned: Paned,
    tree_list: ListBox,
    command_tab_selector: ComboBoxText,
    command_list: FlowBox,
    status_label: Label,
    state: AppState,
    tree_rows: Vec<TreeRowRef>,
    selected_tree: Option<TreeSelection>,
    command_rows: Vec<FlowBoxChild>,
    actions: Option<MenuActions>,
    syncing_tree_list: bool,
    syncing_command_tab_selector: bool,
    syncing_command_list: bool,
    active_workspace_drag_source: Option<usize>,
    active_command_drag_source: Option<usize>,
}

fn build_main_window(app: &Application, mut spec: MainWindowSpec) {
    let view = spec.state.settings().view.clone();
    let language = current_ui_language(&view);

    let window_builder = ApplicationWindow::builder()
        .application(app)
        .title(spec.title);
    let icon = configure_application_icon().unwrap_or_else(|| {
        gtk::Window::set_default_icon_name(APP_LINUX_APPLICATION_ID);
        ApplicationIcon {
            name: APP_LINUX_APPLICATION_ID.to_owned(),
            source_path: None,
        }
    });
    let window = window_builder.icon_name(&icon.name).build();
    install_titlebar_icon(&window, &icon.name, spec.title);
    if let Some(icon_path) = icon.source_path {
        install_toplevel_icon_list(&window, icon_path);
    }

    validate_startup_font_settings(&window, &mut spec.state);
    let view = spec.state.settings().view.clone();
    spec.layout = LayoutSpec::for_font_size(view.font_size);
    let startup_size = startup_window_size(spec.initial_size, spec.layout, &view);
    window.set_default_size(startup_size.width, startup_size.height);

    let root = GtkBox::new(Orientation::Vertical, 0);
    let menu_bar = PopoverMenuBar::from_model(Some(&main_menu_model(language)));
    root.append(&menu_bar);

    let paned = Paned::new(Orientation::Horizontal);
    paned.set_wide_handle(true);
    paned.set_vexpand(true);
    paned.set_hexpand(true);
    paned.set_position(clamp_tree_panel_width_for_window(
        spec.layout,
        startup_size.width,
        view.tree_panel_width
            .unwrap_or(spec.layout.tree_panel_width),
    ));

    let tree_list = ListBox::new();
    tree_list.set_selection_mode(gtk::SelectionMode::Single);
    tree_list.set_activate_on_single_click(false);
    tree_list.add_css_class("workspace-tree");

    let tree_scroll = ScrolledWindow::builder()
        .min_content_width(MIN_TREE_PANEL_WIDTH)
        .child(&tree_list)
        .build();

    let right_panel = GtkBox::new(Orientation::Vertical, 6);
    right_panel.set_margin_top(0);
    right_panel.set_margin_bottom(0);
    right_panel.set_margin_start(0);
    right_panel.set_margin_end(0);

    let command_tab_selector = ComboBoxText::new();
    command_tab_selector.set_hexpand(true);
    right_panel.append(&command_tab_selector);

    let command_list = FlowBox::new();
    command_list.set_selection_mode(gtk::SelectionMode::Single);
    command_list.set_activate_on_single_click(false);
    command_list.set_min_children_per_line(1);
    command_list.set_valign(Align::Start);
    command_list.set_margin_top(COMMAND_BUTTON_TOP_PADDING);
    command_list.set_margin_start(COMMAND_BUTTON_HORIZONTAL_PADDING);
    command_list.set_margin_end(COMMAND_BUTTON_HORIZONTAL_PADDING);
    apply_command_flow_metrics(&command_list, view.font_size);
    command_list.add_css_class("command-list");
    let command_scroll = ScrolledWindow::builder()
        .min_content_width(MIN_COMMAND_TABS_PANEL_WIDTH)
        .min_content_height(160)
        .vexpand(true)
        .child(&command_list)
        .build();
    right_panel.append(&command_scroll);

    paned.set_start_child(Some(&tree_scroll));
    paned.set_resize_start_child(true);
    paned.set_shrink_start_child(false);
    paned.set_end_child(Some(&right_panel));
    paned.set_resize_end_child(true);
    paned.set_shrink_end_child(false);

    let content = GtkBox::new(Orientation::Vertical, 6);
    content.set_vexpand(true);
    content.set_hexpand(true);
    content.set_margin_top(6);
    content.set_margin_bottom(8);
    content.set_margin_start(8);
    content.set_margin_end(8);
    content.append(&paned);

    let status_label = Label::new(Some(spec.state.status_message()));
    status_label.set_xalign(0.0);
    status_label.add_css_class("status-label");

    root.append(&content);
    window.set_child(Some(&root));

    let context = Rc::new(RefCell::new(GtkWindowContext {
        window: window.clone(),
        menu_bar,
        paned,
        tree_list,
        command_tab_selector,
        command_list,
        status_label,
        state: spec.state,
        tree_rows: Vec::new(),
        selected_tree: None,
        command_rows: Vec::new(),
        actions: None,
        syncing_tree_list: false,
        syncing_command_tab_selector: false,
        syncing_command_list: false,
        active_workspace_drag_source: None,
        active_command_drag_source: None,
    }));

    let actions = install_actions(app, context.clone());
    context.borrow_mut().actions = Some(actions);
    connect_main_signals(context.clone());
    install_folder_drop_target(context.clone());
    refresh_all(context.clone());
    apply_view_css(&context.borrow());
    show_startup_warnings(context.clone());
    window.present();
}

fn install_actions(app: &Application, context: Rc<RefCell<GtkWindowContext>>) -> MenuActions {
    add_action(app, "file-font", context.clone(), handle_font_settings);

    let theme = add_string_state_action(
        app,
        "theme",
        ViewTheme::from_config_value(&context.borrow().state.settings().view.theme)
            .unwrap_or_default()
            .as_config_value(),
        context.clone(),
        |context, value| {
            if let Some(theme) = ViewTheme::from_config_value(&value) {
                handle_theme_settings(context, theme);
            }
        },
    );

    let ui_language = add_string_state_action(
        app,
        "ui-language",
        context_language(&context.borrow()).as_config_value(),
        context.clone(),
        |context, value| {
            if let Some(language) = UiLanguage::from_config_value(&value) {
                handle_ui_language_settings(context, language);
            }
        },
    );

    add_action(
        app,
        "file-workspace-languages",
        context.clone(),
        handle_language_config,
    );
    add_action(app, "file-about", context.clone(), handle_about);
    add_action(app, "file-exit", context.clone(), handle_close_main_window);

    add_action(app, "workspace-add", context.clone(), handle_add_workspace);
    add_action(
        app,
        "workspace-add-category",
        context.clone(),
        handle_add_category,
    );
    let workspace_edit = add_action(
        app,
        "workspace-edit",
        context.clone(),
        handle_edit_tree_item,
    );
    let workspace_move_up = add_action(app, "workspace-move-up", context.clone(), |context| {
        handle_move_tree_item(context, MoveDirection::Up);
    });
    let workspace_move_down = add_action(app, "workspace-move-down", context.clone(), |context| {
        handle_move_tree_item(context, MoveDirection::Down);
    });
    let workspace_delete = add_action(
        app,
        "workspace-delete",
        context.clone(),
        handle_delete_tree_item,
    );

    add_action(app, "tab-add", context.clone(), handle_add_command_tab);
    let tab_rename = add_action(
        app,
        "tab-rename",
        context.clone(),
        handle_rename_command_tab,
    );
    let tab_move_up = add_action(app, "tab-move-up", context.clone(), |context| {
        handle_move_command_tab(context, MoveDirection::Up);
    });
    let tab_move_down = add_action(app, "tab-move-down", context.clone(), |context| {
        handle_move_command_tab(context, MoveDirection::Down);
    });
    let tab_delete = add_action(
        app,
        "tab-delete",
        context.clone(),
        handle_delete_command_tab,
    );

    let command_run = add_action(
        app,
        "command-run",
        context.clone(),
        handle_run_selected_command,
    );
    let command_add = add_action(
        app,
        "command-add",
        context.clone(),
        handle_add_command_button,
    );
    let command_edit = add_action(
        app,
        "command-edit",
        context.clone(),
        handle_edit_command_button,
    );
    let command_move_previous =
        add_action(app, "command-move-previous", context.clone(), |context| {
            handle_move_command_button(context, MoveDirection::Up);
        });
    let command_move_next = add_action(app, "command-move-next", context.clone(), |context| {
        handle_move_command_button(context, MoveDirection::Down);
    });
    let command_delete = add_action(
        app,
        "command-delete",
        context.clone(),
        handle_delete_command_button,
    );

    MenuActions {
        theme,
        ui_language,
        workspace_edit,
        workspace_move_up,
        workspace_move_down,
        workspace_delete,
        tab_rename,
        tab_move_up,
        tab_move_down,
        tab_delete,
        command_run,
        command_add,
        command_edit,
        command_move_previous,
        command_move_next,
        command_delete,
    }
}

fn add_action(
    app: &Application,
    name: &str,
    context: Rc<RefCell<GtkWindowContext>>,
    handler: fn(Rc<RefCell<GtkWindowContext>>),
) -> gio::SimpleAction {
    add_action_dynamic(app, name, context, handler)
}

fn add_action_dynamic<F>(
    app: &Application,
    name: &str,
    context: Rc<RefCell<GtkWindowContext>>,
    handler: F,
) -> gio::SimpleAction
where
    F: Fn(Rc<RefCell<GtkWindowContext>>) + 'static,
{
    let action = gio::SimpleAction::new(name, None);
    action.connect_activate(move |_, _| handler(context.clone()));
    app.add_action(&action);
    action
}

fn add_string_state_action<F>(
    app: &Application,
    name: &str,
    initial_state: &str,
    context: Rc<RefCell<GtkWindowContext>>,
    handler: F,
) -> gio::SimpleAction
where
    F: Fn(Rc<RefCell<GtkWindowContext>>, String) + 'static,
{
    let action = gio::SimpleAction::new_stateful(
        name,
        Some(glib::VariantTy::STRING),
        &initial_state.to_variant(),
    );
    action.connect_change_state(move |_, value| {
        let Some(value) = value.and_then(|value| value.get::<String>()) else {
            return;
        };
        handler(context.clone(), value);
    });
    app.add_action(&action);
    action
}

fn connect_main_signals(context: Rc<RefCell<GtkWindowContext>>) {
    let tree_list = context.borrow().tree_list.clone();
    tree_list.connect_row_selected({
        let context = context.clone();
        move |_, row| {
            if context.borrow().syncing_tree_list {
                return;
            }
            handle_tree_row_selected(context.clone(), row);
        }
    });
    tree_list.connect_row_activated({
        let context = context.clone();
        move |tree_list, row| {
            tree_list.select_row(Some(row));
            handle_tree_row_selected(context.clone(), Some(row));
            handle_edit_tree_item(context.clone());
        }
    });

    let key = gtk::EventControllerKey::new();
    key.connect_key_pressed({
        let context = context.clone();
        move |_, key, _, state| handle_tree_key_pressed(context.clone(), key, state).into()
    });
    tree_list.add_controller(key);

    let selector = context.borrow().command_tab_selector.clone();
    selector.connect_changed({
        let context = context.clone();
        move |selector| {
            if context.borrow().syncing_command_tab_selector {
                return;
            }
            let selected_index = selector.active().map(|index| index as usize);
            context
                .borrow_mut()
                .state
                .select_command_tab(selected_index);
            refresh_command_buttons(context.clone());
            update_menu_state(&context.borrow());
        }
    });

    let command_list = context.borrow().command_list.clone();
    command_list.connect_selected_children_changed({
        let context = context.clone();
        move |flow_box| {
            if context.borrow().syncing_command_list {
                return;
            }
            if let Some(child) = flow_box.selected_children().first() {
                context
                    .borrow_mut()
                    .state
                    .select_command_button(Some(child.index().max(0) as usize));
            } else {
                context.borrow_mut().state.select_command_button(None);
            }
            update_menu_state(&context.borrow());
        }
    });
    command_list.connect_child_activated({
        let context = context.clone();
        move |flow_box, child| {
            flow_box.select_child(child);
            let index = child.index().max(0) as usize;
            context
                .borrow_mut()
                .state
                .select_command_button(Some(index));
            update_menu_state(&context.borrow());
            handle_run_command_button(context.clone(), index);
        }
    });
    let command_key = gtk::EventControllerKey::new();
    command_key.connect_key_pressed({
        let context = context.clone();
        move |_, key, _, state| handle_command_key_pressed(context.clone(), key, state).into()
    });
    command_list.add_controller(command_key);

    let window = context.borrow().window.clone();
    window.connect_close_request({
        let context = context.clone();
        move |_| {
            if persist_window_layout(context.clone()) {
                glib::Propagation::Proceed
            } else {
                glib::Propagation::Stop
            }
        }
    });
}

struct ApplicationIcon {
    name: String,
    source_path: Option<PathBuf>,
}

fn configure_application_icon() -> Option<ApplicationIcon> {
    let icon_path = find_application_icon_path()?;
    let icon_name = icon_path
        .file_stem()
        .and_then(|value| value.to_str())
        .filter(|value| !value.trim().is_empty())?
        .to_owned();
    let icon_parent = icon_path.parent()?;

    if let Some(display) = gdk::Display::default() {
        let icon_theme = gtk::IconTheme::for_display(&display);
        icon_theme.add_search_path(icon_parent);
    }
    gtk::Window::set_default_icon_name(&icon_name);

    Some(ApplicationIcon {
        name: icon_name,
        source_path: Some(icon_path),
    })
}

fn install_titlebar_icon(window: &ApplicationWindow, icon_name: &str, title: &str) {
    let titlebar = gtk::WindowHandle::new();
    titlebar.add_css_class("j3-compact-titlebar");

    let layout = gtk::CenterBox::new();
    layout.add_css_class("j3-titlebar-layout");

    let start = GtkBox::new(Orientation::Horizontal, 0);
    start.add_css_class("j3-titlebar-start");
    let icon = gtk::Image::from_icon_name(icon_name);
    icon.set_pixel_size(12);
    icon.add_css_class("j3-titlebar-icon");
    start.append(&icon);

    let label = Label::new(Some(title));
    label.set_ellipsize(pango::EllipsizeMode::End);
    label.add_css_class("j3-titlebar-label");

    let controls = gtk::WindowControls::new(gtk::PackType::End);
    controls.add_css_class("j3-titlebar-controls");

    layout.set_start_widget(Some(&start));
    layout.set_center_widget(Some(&label));
    layout.set_end_widget(Some(&controls));
    titlebar.set_child(Some(&layout));
    window.set_titlebar(Some(&titlebar));
}

fn install_toplevel_icon_list(window: &ApplicationWindow, icon_path: PathBuf) {
    window.connect_realize(move |window| {
        let Ok(texture) = gdk::Texture::from_filename(&icon_path) else {
            return;
        };
        let Some(surface) = window.surface() else {
            return;
        };
        let Ok(toplevel) = surface.downcast::<gdk::Toplevel>() else {
            return;
        };
        toplevel.set_icon_list(&[texture]);
    });
}

fn find_application_icon_path() -> Option<PathBuf> {
    find_icon_path(APP_ICON_SVG_FILE_NAME).or_else(|| find_icon_path(APP_ICON_PNG_FILE_NAME))
}

fn find_icon_path(file_name: &str) -> Option<PathBuf> {
    let executable_icon = env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(|parent| parent.join(file_name)));
    executable_icon.filter(|path| path.is_file()).or_else(|| {
        env::current_dir()
            .ok()
            .map(|path| path.join(file_name))
            .filter(|path| path.is_file())
    })
}

fn main_menu_model(language: UiLanguage) -> gio::Menu {
    let menu = gio::Menu::new();
    menu.append_submenu(Some(tr(language, "파일", "File")), &file_menu(language));
    menu.append_submenu(
        Some(tr(language, "워크스페이스", "Workspace")),
        &workspace_menu(language),
    );
    menu.append_submenu(
        Some(tr(language, "명령 그룹", "Command Group")),
        &command_tab_menu(language),
    );
    menu.append_submenu(
        Some(tr(language, "명령", "Command")),
        &command_menu(language),
    );
    menu
}

fn file_menu(language: UiLanguage) -> gio::Menu {
    let menu = gio::Menu::new();
    menu.append(Some(tr(language, "글꼴", "Font")), Some("app.file-font"));
    menu.append_submenu(Some(tr(language, "테마", "Theme")), &theme_menu(language));
    menu.append_submenu(
        Some(tr(language, "UI 언어", "UI Language")),
        &ui_language_menu(language),
    );
    menu.append(
        Some(tr(language, "워크스페이스 언어", "Workspace Languages")),
        Some("app.file-workspace-languages"),
    );
    menu.append(Some(tr(language, "정보", "About")), Some("app.file-about"));
    menu.append(Some(tr(language, "종료", "Exit")), Some("app.file-exit"));
    menu
}

fn theme_menu(language: UiLanguage) -> gio::Menu {
    let menu = gio::Menu::new();
    for theme in ViewTheme::options() {
        let item = gio::MenuItem::new(Some(theme.display_name_for(language)), None);
        item.set_action_and_target_value(
            Some("app.theme"),
            Some(&theme.as_config_value().to_variant()),
        );
        menu.append_item(&item);
    }
    menu
}

fn ui_language_menu(language: UiLanguage) -> gio::Menu {
    let menu = gio::Menu::new();
    for option in UiLanguage::options() {
        let item = gio::MenuItem::new(Some(option.display_name_for(language)), None);
        item.set_action_and_target_value(
            Some("app.ui-language"),
            Some(&option.as_config_value().to_variant()),
        );
        menu.append_item(&item);
    }
    menu
}

fn workspace_menu(language: UiLanguage) -> gio::Menu {
    let menu = gio::Menu::new();
    menu.append(Some(tr(language, "추가", "Add")), Some("app.workspace-add"));
    menu.append(
        Some(tr(language, "분류 추가", "Add Category")),
        Some("app.workspace-add-category"),
    );
    menu.append(
        Some(tr(language, "편집", "Edit")),
        Some("app.workspace-edit"),
    );
    menu.append(
        Some(tr(language, "위로", "Move Up")),
        Some("app.workspace-move-up"),
    );
    menu.append(
        Some(tr(language, "아래로", "Move Down")),
        Some("app.workspace-move-down"),
    );
    menu.append(
        Some(tr(language, "삭제", "Delete")),
        Some("app.workspace-delete"),
    );
    menu
}

fn command_tab_menu(language: UiLanguage) -> gio::Menu {
    let menu = gio::Menu::new();
    menu.append(Some(tr(language, "추가", "Add")), Some("app.tab-add"));
    menu.append(
        Some(tr(language, "이름 변경", "Rename")),
        Some("app.tab-rename"),
    );
    menu.append(
        Some(tr(language, "위로", "Move Up")),
        Some("app.tab-move-up"),
    );
    menu.append(
        Some(tr(language, "아래로", "Move Down")),
        Some("app.tab-move-down"),
    );
    menu.append(Some(tr(language, "삭제", "Delete")), Some("app.tab-delete"));
    menu
}

fn command_menu(language: UiLanguage) -> gio::Menu {
    let menu = gio::Menu::new();
    menu.append(Some(tr(language, "실행", "Run")), Some("app.command-run"));
    menu.append(Some(tr(language, "추가", "Add")), Some("app.command-add"));
    menu.append(Some(tr(language, "편집", "Edit")), Some("app.command-edit"));
    menu.append(
        Some(tr(language, "앞으로", "Previous")),
        Some("app.command-move-previous"),
    );
    menu.append(
        Some(tr(language, "뒤로", "Next")),
        Some("app.command-move-next"),
    );
    menu.append(
        Some(tr(language, "삭제", "Delete")),
        Some("app.command-delete"),
    );
    menu
}

fn refresh_all(context: Rc<RefCell<GtkWindowContext>>) {
    refresh_tree(context.clone());
    refresh_command_tab_selector(context.clone());
    refresh_command_buttons(context.clone());
    update_status_label(&context.borrow());
    update_menu_state(&context.borrow());
}

fn refresh_tree(context: Rc<RefCell<GtkWindowContext>>) {
    let tree_list = context.borrow().tree_list.clone();
    context.borrow_mut().syncing_tree_list = true;
    clear_list_box(&tree_list);
    context.borrow_mut().tree_rows.clear();

    let rows = build_tree_rows(&context.borrow().state);
    for row_ref in rows {
        append_tree_row(context.clone(), row_ref);
    }
    let restored_selection = restore_tree_selection(context.clone());
    context.borrow_mut().syncing_tree_list = false;
    if !restored_selection {
        schedule_tree_selection_clear(context);
    }
}

fn build_tree_rows(state: &AppState) -> Vec<TreeRowRef> {
    let settings = state.settings();
    let mut rows = Vec::new();
    let groups = category_workspace_groups(settings);

    for item in settings.root_tree_items() {
        match item {
            TreeRootItemRef::Category(index) => {
                rows.push(TreeRowRef::Category(index));
                if let Some(workspace_indexes) = groups.get(index) {
                    rows.extend(workspace_indexes.iter().copied().map(TreeRowRef::Workspace));
                }
            }
            TreeRootItemRef::Workspace(index) => rows.push(TreeRowRef::Workspace(index)),
        }
    }

    rows
}

fn append_tree_row(context: Rc<RefCell<GtkWindowContext>>, row_ref: TreeRowRef) {
    let language = context_language(&context.borrow());
    let row = ListBoxRow::new();
    row.add_css_class("tree-row");

    let label = Label::new(None);
    label.set_xalign(0.0);
    label.set_hexpand(true);
    label.set_ellipsize(pango::EllipsizeMode::End);

    let container = GtkBox::new(Orientation::Horizontal, 6);
    container.set_margin_top(4);
    container.set_margin_bottom(4);
    container.set_margin_start(6);
    container.set_margin_end(6);

    match row_ref {
        TreeRowRef::Category(index) => {
            row.add_css_class("category-row");
            let name = context
                .borrow()
                .state
                .settings()
                .categories
                .get(index)
                .map(|category| category.name.clone())
                .unwrap_or_default();
            label.set_text(&format!("▾ {name}"));
        }
        TreeRowRef::Workspace(index) => {
            let workspace = context
                .borrow()
                .state
                .settings()
                .workspaces
                .get(index)
                .cloned();
            if let Some(workspace) = workspace {
                if workspace.category.is_some() {
                    container.set_margin_start(24);
                }
                label.set_text(&workspace.name);
                row.set_tooltip_text(Some(&workspace_tree_tooltip_text(language, &workspace)));
            }
        }
    }

    container.append(&label);
    row.set_child(Some(&container));

    let right_click = GestureClick::new();
    right_click.set_button(3);
    right_click.connect_pressed({
        let context = context.clone();
        let row = row.clone();
        move |_, _, x, y| {
            let tree_list = context.borrow().tree_list.clone();
            tree_list.select_row(Some(&row));
            tree_list.grab_focus();
            show_tree_context_menu(context.clone(), &row, x, y);
        }
    });
    row.add_controller(right_click);

    install_tree_row_drag_drop(context.clone(), &row, row_ref);

    let tree_list = context.borrow().tree_list.clone();
    tree_list.append(&row);
    context.borrow_mut().tree_rows.push(row_ref);
}

fn workspace_tree_tooltip_text(language: UiLanguage, workspace: &Workspace) -> String {
    match language {
        UiLanguage::Korean => format!("폴더: {}\n언어: {}", workspace.path, workspace.language),
        UiLanguage::English => format!(
            "Folder: {}\nLanguage: {}",
            workspace.path, workspace.language
        ),
    }
}

fn restore_tree_selection(context: Rc<RefCell<GtkWindowContext>>) -> bool {
    let target = {
        let context_ref = context.borrow();
        if let Some(index) = context_ref.state.selected_workspace_index() {
            Some(TreeSelection::Workspace(index))
        } else {
            match context_ref.selected_tree {
                Some(TreeSelection::Category(index))
                    if index < context_ref.state.settings().categories.len() =>
                {
                    Some(TreeSelection::Category(index))
                }
                _ => None,
            }
        }
    };
    let target_row_ref = target.map(|selection| match selection {
        TreeSelection::Workspace(index) => TreeRowRef::Workspace(index),
        TreeSelection::Category(index) => TreeRowRef::Category(index),
    });
    let (tree_list, row) = {
        let context_ref = context.borrow();
        let row_index = target_row_ref
            .and_then(|target| context_ref.tree_rows.iter().position(|row| *row == target));
        (
            context_ref.tree_list.clone(),
            row_index.and_then(|index| context_ref.tree_list.row_at_index(index as i32)),
        )
    };

    if row.is_none() {
        let mut context_mut = context.borrow_mut();
        context_mut.selected_tree = None;
        context_mut.state.select_workspace(None);
    }
    tree_list.select_row(row.as_ref());
    row.is_some()
}

fn schedule_tree_selection_clear(context: Rc<RefCell<GtkWindowContext>>) {
    glib::idle_add_local_once(move || clear_tree_selection(context));
}

fn clear_tree_selection(context: Rc<RefCell<GtkWindowContext>>) {
    let tree_list = context.borrow().tree_list.clone();
    {
        let mut context_mut = context.borrow_mut();
        context_mut.syncing_tree_list = true;
        context_mut.selected_tree = None;
        context_mut.state.select_workspace(None);
    }
    tree_list.select_row(None::<&ListBoxRow>);
    context.borrow_mut().syncing_tree_list = false;
    update_menu_state(&context.borrow());
}

fn refresh_command_tab_selector(context: Rc<RefCell<GtkWindowContext>>) {
    let (selector, tab_names, selected_index) = {
        let context_ref = context.borrow();
        (
            context_ref.command_tab_selector.clone(),
            context_ref
                .state
                .settings()
                .command_tabs
                .iter()
                .map(|tab| tab.name.clone())
                .collect::<Vec<_>>(),
            context_ref.state.selected_command_tab_index(),
        )
    };
    let selector_state = command_tab_selector_sync_state(tab_names.len(), selected_index);
    {
        let mut context_mut = context.borrow_mut();
        context_mut
            .state
            .select_command_tab(selector_state.domain_selection);
    }

    context.borrow_mut().syncing_command_tab_selector = true;
    selector.remove_all();

    for name in &tab_names {
        selector.append_text(name);
    }

    selector.set_active(selector_state.active_index);
    selector.set_sensitive(selector_state.sensitive);
    context.borrow_mut().syncing_command_tab_selector = false;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct CommandTabSelectorSyncState {
    active_index: Option<u32>,
    sensitive: bool,
    domain_selection: Option<usize>,
}

fn command_tab_selector_sync_state(
    tab_count: usize,
    selected_index: Option<usize>,
) -> CommandTabSelectorSyncState {
    let domain_selection = selected_index.filter(|index| *index < tab_count);
    CommandTabSelectorSyncState {
        active_index: domain_selection.and_then(|index| u32::try_from(index).ok()),
        sensitive: tab_count > 0,
        domain_selection,
    }
}

fn refresh_command_buttons(context: Rc<RefCell<GtkWindowContext>>) {
    let command_list = context.borrow().command_list.clone();
    context.borrow_mut().syncing_command_list = true;
    clear_flow_box(&command_list);
    context.borrow_mut().command_rows.clear();

    let buttons = context
        .borrow()
        .state
        .selected_command_tab()
        .map(|tab| tab.buttons.clone())
        .unwrap_or_default();
    let command_button_width =
        command_button_preferred_width_for_font(context.borrow().state.settings().view.font_size);
    let command_button_height =
        command_button_height_for_font(context.borrow().state.settings().view.font_size);

    for (index, button) in buttons.iter().enumerate() {
        let row = FlowBoxChild::new();
        row.set_valign(Align::Start);
        row.set_size_request(command_button_width, command_button_height);
        row.add_css_class("command-row");
        let command = Button::new();
        command.set_valign(Align::Start);
        let command_label = Label::new(Some(&button.button_name));
        command_label.set_xalign(0.0);
        command_label.set_ellipsize(pango::EllipsizeMode::End);
        command.set_child(Some(&command_label));
        command.set_size_request(command_button_width, command_button_height);
        command.set_tooltip_text(Some(&button.button_name));
        command.add_css_class("command-button");
        command.connect_clicked({
            let context = context.clone();
            let row = row.clone();
            let command = command.clone();
            move |_| {
                let command_list = context.borrow().command_list.clone();
                command_list.select_child(&row);
                command.grab_focus();
                handle_run_command_button(context.clone(), index);
            }
        });

        let right_click = GestureClick::new();
        right_click.set_button(3);
        right_click.connect_pressed({
            let context = context.clone();
            let row = row.clone();
            move |_, _, x, y| {
                let command_list = context.borrow().command_list.clone();
                command_list.select_child(&row);
                command_list.grab_focus();
                show_command_context_menu(context.clone(), &row, x, y);
            }
        });
        command.add_controller(right_click);

        let command_key = gtk::EventControllerKey::new();
        command_key.connect_key_pressed({
            let context = context.clone();
            let row = row.clone();
            move |_, key, _, state| {
                if !is_context_menu_key(key, state) {
                    return false.into();
                }
                let command_list = context.borrow().command_list.clone();
                command_list.select_child(&row);
                context
                    .borrow_mut()
                    .state
                    .select_command_button(Some(index));
                update_menu_state(&context.borrow());
                show_command_context_menu(
                    context.clone(),
                    &row,
                    12.0,
                    f64::from(row.allocated_height()) / 2.0,
                );
                true.into()
            }
        });
        command.add_controller(command_key);

        install_command_row_drag_drop(context.clone(), &row, index);
        row.set_child(Some(&command));
        command_list.append(&row);
        context.borrow_mut().command_rows.push(row);
    }

    let selected_row = {
        let context_ref = context.borrow();
        context_ref
            .state
            .selected_command_button_index()
            .and_then(|index| context_ref.command_rows.get(index).cloned())
    };
    if let Some(row) = selected_row {
        command_list.select_child(&row);
    } else {
        context.borrow_mut().state.select_command_button(None);
    }
    context.borrow_mut().syncing_command_list = false;
    update_menu_state(&context.borrow());
}

fn clear_list_box(list_box: &ListBox) {
    while let Some(child) = list_box.first_child() {
        list_box.remove(&child);
    }
}

fn clear_flow_box(flow_box: &FlowBox) {
    while let Some(child) = flow_box.first_child() {
        flow_box.remove(&child);
    }
}

fn apply_command_flow_metrics(flow_box: &FlowBox, font_size: u16) {
    let gap = command_button_gap_for_font(font_size);
    flow_box.set_column_spacing(gap);
    flow_box.set_row_spacing(gap);
}

fn command_button_preferred_width_for_font(font_size: u16) -> i32 {
    COMMAND_BUTTON_PREFERRED_WIDTH + command_button_font_extra(font_size) * 12
}

fn command_button_gap_for_font(font_size: u16) -> u32 {
    COMMAND_BUTTON_BASE_GAP + command_button_font_extra(font_size).min(4) as u32
}

fn command_button_height_for_font(font_size: u16) -> i32 {
    30 + command_button_font_extra(font_size) * 2
}

fn command_button_font_extra(font_size: u16) -> i32 {
    i32::from(normalize_ui_font_size(font_size).saturating_sub(DEFAULT_FONT_SIZE))
}

fn handle_tree_row_selected(context: Rc<RefCell<GtkWindowContext>>, row: Option<&ListBoxRow>) {
    let selection = row.and_then(|row| {
        context
            .borrow()
            .tree_rows
            .get(row.index().max(0) as usize)
            .copied()
    });

    let mut context_mut = context.borrow_mut();
    match selection {
        Some(TreeRowRef::Workspace(index)) => {
            context_mut.selected_tree = Some(TreeSelection::Workspace(index));
            context_mut.state.select_workspace(Some(index));
        }
        Some(TreeRowRef::Category(index)) => {
            context_mut.selected_tree = Some(TreeSelection::Category(index));
            context_mut.state.select_workspace(None);
        }
        None => {
            context_mut.selected_tree = None;
            context_mut.state.select_workspace(None);
        }
    }
    drop(context_mut);

    update_menu_state(&context.borrow());
}

fn handle_tree_key_pressed(
    context: Rc<RefCell<GtkWindowContext>>,
    key: gdk::Key,
    state: gdk::ModifierType,
) -> bool {
    if is_context_menu_key(key, state) {
        return show_selected_tree_context_menu(context);
    }

    if !state.contains(gdk::ModifierType::CONTROL_MASK) {
        return false;
    }

    if key == gdk::Key::Up {
        handle_move_tree_item(context, MoveDirection::Up);
        true
    } else if key == gdk::Key::Down {
        handle_move_tree_item(context, MoveDirection::Down);
        true
    } else if key == gdk::Key::Left {
        if selected_workspace_can_move_to_root(&context.borrow()) {
            handle_move_workspace_to_root(context);
            true
        } else {
            false
        }
    } else {
        false
    }
}

fn handle_command_key_pressed(
    context: Rc<RefCell<GtkWindowContext>>,
    key: gdk::Key,
    state: gdk::ModifierType,
) -> bool {
    if is_context_menu_key(key, state) {
        return show_selected_command_context_menu(context);
    }

    false
}

fn is_context_menu_key(key: gdk::Key, state: gdk::ModifierType) -> bool {
    key == gdk::Key::Menu || (key == gdk::Key::F10 && state.contains(gdk::ModifierType::SHIFT_MASK))
}

fn show_selected_tree_context_menu(context: Rc<RefCell<GtkWindowContext>>) -> bool {
    let (tree_list, row) = {
        let context_ref = context.borrow();
        (
            context_ref.tree_list.clone(),
            context_ref.tree_list.selected_row(),
        )
    };
    let Some(row) = row else {
        return false;
    };
    tree_list.grab_focus();
    show_tree_context_menu(context, &row, 12.0, f64::from(row.allocated_height()) / 2.0);
    true
}

fn show_selected_command_context_menu(context: Rc<RefCell<GtkWindowContext>>) -> bool {
    let (command_list, row) = {
        let context_ref = context.borrow();
        (
            context_ref.command_list.clone(),
            context_ref
                .command_list
                .selected_children()
                .first()
                .cloned(),
        )
    };
    let Some(row) = row else {
        return false;
    };
    command_list.grab_focus();
    show_command_context_menu(context, &row, 12.0, f64::from(row.allocated_height()) / 2.0);
    true
}

fn selected_workspace_can_move_to_root(context: &GtkWindowContext) -> bool {
    workspace_selection_can_move_to_root(context.state.settings(), context.selected_tree)
}

fn workspace_selection_can_move_to_root(
    settings: &AppSettings,
    selection: Option<TreeSelection>,
) -> bool {
    let Some(TreeSelection::Workspace(index)) = selection else {
        return false;
    };
    let Some(workspace) = settings.workspaces.get(index) else {
        return false;
    };

    workspace_category_index(workspace, &settings.categories).is_some()
}

fn show_tree_context_menu(
    context: Rc<RefCell<GtkWindowContext>>,
    row: &ListBoxRow,
    x: f64,
    y: f64,
) {
    let language = context_language(&context.borrow());
    let tree_state = current_tree_menu_state(&context.borrow());
    let popover = Popover::new();
    popover.set_parent(row);
    popover.set_pointing_to(Some(&gdk::Rectangle::new(x as i32, y as i32, 1, 1)));
    let actions = GtkBox::new(Orientation::Vertical, 0);

    append_popover_button(
        &actions,
        tr(language, "편집", "Edit"),
        context.clone(),
        popover.clone(),
        handle_edit_tree_item,
        tree_state.can_edit_tree_item,
    );
    append_popover_separator(&actions);
    append_popover_button(
        &actions,
        tr(language, "위로", "Move Up"),
        context.clone(),
        popover.clone(),
        |context| handle_move_tree_item(context, MoveDirection::Up),
        tree_state.can_move_up,
    );
    append_popover_button(
        &actions,
        tr(language, "아래로", "Move Down"),
        context.clone(),
        popover.clone(),
        |context| handle_move_tree_item(context, MoveDirection::Down),
        tree_state.can_move_down,
    );
    append_popover_separator(&actions);
    append_popover_button(
        &actions,
        tr(language, "워크스페이스 추가", "Add Workspace"),
        context.clone(),
        popover.clone(),
        handle_add_workspace,
        true,
    );
    append_popover_button(
        &actions,
        tr(language, "분류 추가", "Add Category"),
        context.clone(),
        popover.clone(),
        handle_add_category,
        true,
    );
    append_popover_separator(&actions);
    append_popover_button(
        &actions,
        tr(language, "삭제", "Delete"),
        context.clone(),
        popover.clone(),
        handle_delete_tree_item,
        tree_state.can_delete_tree_item,
    );

    popover.set_child(Some(&actions));
    popover.popup();
}

fn show_command_context_menu(
    context: Rc<RefCell<GtkWindowContext>>,
    row: &FlowBoxChild,
    x: f64,
    y: f64,
) {
    let language = context_language(&context.borrow());
    let popover = Popover::new();
    popover.set_parent(row);
    popover.set_pointing_to(Some(&gdk::Rectangle::new(x as i32, y as i32, 1, 1)));
    let actions = GtkBox::new(Orientation::Vertical, 0);
    let state = current_command_menu_state(&context.borrow());

    append_popover_button(
        &actions,
        tr(language, "실행", "Run"),
        context.clone(),
        popover.clone(),
        handle_run_selected_command,
        state.can_execute,
    );
    append_popover_separator(&actions);
    append_popover_button(
        &actions,
        tr(language, "편집", "Edit"),
        context.clone(),
        popover.clone(),
        handle_edit_command_button,
        state.can_edit,
    );
    append_popover_button(
        &actions,
        tr(language, "앞으로", "Previous"),
        context.clone(),
        popover.clone(),
        |context| handle_move_command_button(context, MoveDirection::Up),
        state.can_move_previous,
    );
    append_popover_button(
        &actions,
        tr(language, "뒤로", "Next"),
        context.clone(),
        popover.clone(),
        |context| handle_move_command_button(context, MoveDirection::Down),
        state.can_move_next,
    );
    append_popover_separator(&actions);
    append_popover_button(
        &actions,
        tr(language, "명령 추가", "Add Command"),
        context.clone(),
        popover.clone(),
        handle_add_command_button,
        state.can_add,
    );
    append_popover_separator(&actions);
    append_popover_button(
        &actions,
        tr(language, "삭제", "Delete"),
        context.clone(),
        popover.clone(),
        handle_delete_command_button,
        state.can_delete,
    );

    popover.set_child(Some(&actions));
    popover.popup();
}

fn append_popover_button<F>(
    container: &GtkBox,
    label: &str,
    context: Rc<RefCell<GtkWindowContext>>,
    popover: Popover,
    handler: F,
    enabled: bool,
) where
    F: Fn(Rc<RefCell<GtkWindowContext>>) + 'static,
{
    let button = Button::with_label(label);
    button.set_halign(Align::Fill);
    button.set_sensitive(enabled);
    button.connect_clicked(move |_| {
        popover.popdown();
        handler(context.clone());
    });
    container.append(&button);
}

fn append_popover_separator(container: &GtkBox) {
    let separator = Separator::new(Orientation::Horizontal);
    separator.set_margin_top(3);
    separator.set_margin_bottom(3);
    container.append(&separator);
}

#[derive(Clone, Copy)]
enum MoveDirection {
    Up,
    Down,
}

impl From<MoveDirection> for TreeKeyboardMoveDirection {
    fn from(direction: MoveDirection) -> Self {
        match direction {
            MoveDirection::Up => Self::Up,
            MoveDirection::Down => Self::Down,
        }
    }
}

impl From<MoveDirection> for CommandTabMoveDirection {
    fn from(direction: MoveDirection) -> Self {
        match direction {
            MoveDirection::Up => Self::Left,
            MoveDirection::Down => Self::Right,
        }
    }
}

impl From<MoveDirection> for CommandButtonMoveDirection {
    fn from(direction: MoveDirection) -> Self {
        match direction {
            MoveDirection::Up => Self::Previous,
            MoveDirection::Down => Self::Next,
        }
    }
}

fn handle_add_workspace(context: Rc<RefCell<GtkWindowContext>>) {
    let language = context_language(&context.borrow());
    let reserved_paths = workspace_paths_except(&context.borrow(), None);
    let options = context.borrow().state.settings().languages.clone();
    if let Some(workspace) = show_workspace_dialog(
        &context_window(&context),
        WorkspaceDialogMode::Add,
        None,
        reserved_paths,
        options,
        language,
    ) {
        let restore_point = GtkStateRestorePoint::capture(&context.borrow());
        let result = context.borrow_mut().state.add_workspace(workspace);
        match result {
            Ok(_) => {
                if persist_settings_or_restore(context.clone(), restore_point) {
                    refresh_all(context.clone());
                }
            }
            Err(error) => {
                show_error_message(
                    &context_window(&context),
                    tr(language, "워크스페이스", "Workspace"),
                    &error.user_message_for_language(language),
                );
            }
        }
    }
}

fn handle_edit_tree_item(context: Rc<RefCell<GtkWindowContext>>) {
    let language = context_language(&context.borrow());
    let selection = context.borrow().selected_tree;
    match selection {
        Some(TreeSelection::Workspace(index)) => handle_edit_workspace(context, index),
        Some(TreeSelection::Category(index)) => handle_edit_category(context, index),
        None => {
            show_workspace_warning(
                &context,
                language,
                tree_edit_selection_required_message(language),
            );
            update_menu_state(&context.borrow());
        }
    }
}

fn handle_edit_workspace(context: Rc<RefCell<GtkWindowContext>>, index: usize) {
    let language = context_language(&context.borrow());
    let workspace = context
        .borrow()
        .state
        .settings()
        .workspaces
        .get(index)
        .cloned();
    let Some(workspace) = workspace else {
        context.borrow_mut().state.select_workspace(None);
        context.borrow_mut().selected_tree = None;
        show_workspace_warning(
            &context,
            language,
            selected_workspace_missing_message(language),
        );
        refresh_tree(context.clone());
        update_menu_state(&context.borrow());
        return;
    };

    let reserved_paths = workspace_paths_except(&context.borrow(), Some(index));
    let options = context.borrow().state.settings().languages.clone();
    if let Some(updated) = show_workspace_dialog(
        &context_window(&context),
        WorkspaceDialogMode::Edit,
        Some(workspace),
        reserved_paths,
        options,
        language,
    ) {
        let restore_point = GtkStateRestorePoint::capture(&context.borrow());
        let result = context.borrow_mut().state.update_workspace(index, updated);
        match result {
            Ok(()) => {
                if persist_settings_or_restore(context.clone(), restore_point) {
                    refresh_all(context.clone());
                }
            }
            Err(error) => show_error_message(
                &context_window(&context),
                tr(language, "워크스페이스", "Workspace"),
                &error.user_message_for_language(language),
            ),
        }
    }
}

fn handle_add_category(context: Rc<RefCell<GtkWindowContext>>) {
    let language = context_language(&context.borrow());
    if let Some(name) = show_text_input_dialog(
        &context_window(&context),
        tr(language, "분류 추가", "Add Category"),
        tr(language, "새 분류 이름", "New category name"),
        "",
        false,
        language,
    ) {
        match Category::new(name) {
            Ok(category) => {
                let restore_point = GtkStateRestorePoint::capture(&context.borrow());
                let result = {
                    let mut context_mut = context.borrow_mut();
                    let result = context_mut.state.add_category(category);
                    if result.is_ok() {
                        context_mut.selected_tree = None;
                    }
                    result
                };
                match result {
                    Ok(_) => {
                        if persist_settings_or_restore(context.clone(), restore_point) {
                            refresh_all(context.clone());
                        }
                    }
                    Err(error) => show_error_message(
                        &context_window(&context),
                        tr(language, "분류", "Category"),
                        &error.user_message_for_language(language),
                    ),
                }
            }
            Err(error) => show_warning_message(
                &context_window(&context),
                tr(language, "분류", "Category"),
                &error.user_message_for_language(language),
            ),
        }
    }
}

fn handle_edit_category(context: Rc<RefCell<GtkWindowContext>>, index: usize) {
    let language = context_language(&context.borrow());
    let name = context
        .borrow()
        .state
        .settings()
        .categories
        .get(index)
        .map(|category| category.name.clone());
    let Some(name) = name else {
        context.borrow_mut().selected_tree = None;
        show_category_warning(
            &context,
            language,
            selected_category_missing_message(language),
        );
        refresh_tree(context.clone());
        update_menu_state(&context.borrow());
        return;
    };

    if let Some(next_name) = show_text_input_dialog(
        &context_window(&context),
        tr(language, "분류 편집", "Edit Category"),
        tr(language, "분류 이름", "Category name"),
        &name,
        false,
        language,
    ) {
        match Category::new(next_name) {
            Ok(category) => {
                let restore_point = GtkStateRestorePoint::capture(&context.borrow());
                let result = context.borrow_mut().state.rename_category(index, category);
                match result {
                    Ok(()) => {
                        if persist_settings_or_restore(context.clone(), restore_point) {
                            refresh_all(context.clone());
                        }
                    }
                    Err(error) => show_error_message(
                        &context_window(&context),
                        tr(language, "분류", "Category"),
                        &error.user_message_for_language(language),
                    ),
                }
            }
            Err(error) => show_warning_message(
                &context_window(&context),
                tr(language, "분류", "Category"),
                &error.user_message_for_language(language),
            ),
        }
    }
}

fn handle_delete_tree_item(context: Rc<RefCell<GtkWindowContext>>) {
    let language = context_language(&context.borrow());
    let selection = context.borrow().selected_tree;
    match selection {
        Some(TreeSelection::Workspace(index)) => handle_delete_workspace(context, index),
        Some(TreeSelection::Category(index)) => handle_delete_category(context, index),
        None => {
            show_workspace_warning(
                &context,
                language,
                tree_delete_selection_required_message(language),
            );
            update_menu_state(&context.borrow());
        }
    }
}

fn handle_delete_workspace(context: Rc<RefCell<GtkWindowContext>>, index: usize) {
    let language = context_language(&context.borrow());
    let workspace = context
        .borrow()
        .state
        .settings()
        .workspaces
        .get(index)
        .cloned();
    let Some(workspace) = workspace else {
        context.borrow_mut().state.select_workspace(None);
        context.borrow_mut().selected_tree = None;
        show_workspace_warning(
            &context,
            language,
            selected_workspace_missing_message(language),
        );
        refresh_tree(context.clone());
        update_menu_state(&context.borrow());
        return;
    };
    let message = match language {
        UiLanguage::Korean => format!(
            "워크스페이스를 삭제할까요?\n\n이름: {}\n폴더: {}",
            workspace.name, workspace.path
        ),
        UiLanguage::English => format!(
            "Delete this workspace?\n\nName: {}\nFolder: {}",
            workspace.name, workspace.path
        ),
    };
    if confirm(
        &context_window(&context),
        tr(language, "워크스페이스 삭제", "Delete Workspace"),
        &message,
    ) {
        let restore_point = GtkStateRestorePoint::capture(&context.borrow());
        context.borrow_mut().state.delete_workspace(index);
        if persist_settings_or_restore(context.clone(), restore_point) {
            refresh_all(context.clone());
        }
    }
}

fn handle_delete_category(context: Rc<RefCell<GtkWindowContext>>, index: usize) {
    let language = context_language(&context.borrow());
    let category = context
        .borrow()
        .state
        .settings()
        .categories
        .get(index)
        .cloned();
    let Some(category) = category else {
        context.borrow_mut().selected_tree = None;
        show_category_warning(
            &context,
            language,
            selected_category_missing_message(language),
        );
        refresh_tree(context.clone());
        update_menu_state(&context.borrow());
        return;
    };
    let workspace_count = context
        .borrow()
        .state
        .settings()
        .workspaces
        .iter()
        .filter(|workspace| workspace_belongs_to_category(workspace, &category))
        .count();
    let message = match language {
        UiLanguage::Korean => format!(
            "분류를 삭제할까요?\n\n이름: {}\n소속 워크스페이스: {}개\n\n워크스페이스는 삭제하지 않고 최상위로 이동합니다.",
            category.name, workspace_count
        ),
        UiLanguage::English => format!(
            "Delete this category?\n\nName: {}\nWorkspaces: {}\n\nWorkspaces will not be deleted and will move to the top level.",
            category.name, workspace_count
        ),
    };
    if confirm(
        &context_window(&context),
        tr(language, "분류 삭제", "Delete Category"),
        &message,
    ) {
        let restore_point = GtkStateRestorePoint::capture(&context.borrow());
        {
            let mut context_mut = context.borrow_mut();
            context_mut.state.delete_category(index);
            context_mut.selected_tree = None;
        }
        if persist_settings_or_restore(context.clone(), restore_point) {
            refresh_all(context.clone());
        }
    }
}

fn handle_move_tree_item(context: Rc<RefCell<GtkWindowContext>>, direction: MoveDirection) {
    let language = context_language(&context.borrow());
    let selection = context.borrow().selected_tree;
    if selection.is_none() {
        show_workspace_warning(
            &context,
            language,
            tree_move_selection_required_message(language),
        );
        update_menu_state(&context.borrow());
        return;
    }
    if let Some(message) =
        tree_selection_missing_message(context.borrow().state.settings(), selection, language)
    {
        {
            let mut context_mut = context.borrow_mut();
            context_mut.state.select_workspace(None);
            context_mut.selected_tree = None;
        }
        match selection {
            Some(TreeSelection::Category(_)) => show_category_warning(&context, language, message),
            _ => show_workspace_warning(&context, language, message),
        }
        refresh_tree(context.clone());
        update_menu_state(&context.borrow());
        return;
    }
    let restore_point = GtkStateRestorePoint::capture(&context.borrow());
    let result = match selection {
        Some(TreeSelection::Workspace(index)) => {
            if workspace_is_root(&context.borrow(), index) {
                move_root_tree_item(
                    context.clone(),
                    TreeRootItemRef::Workspace(index),
                    direction,
                    language,
                )
            } else {
                move_workspace_in_category(context.clone(), index, direction, language)
            }
        }
        Some(TreeSelection::Category(index)) => move_root_tree_item(
            context.clone(),
            TreeRootItemRef::Category(index),
            direction,
            language,
        ),
        None => Ok(false),
    };

    match result {
        Ok(true) => {
            if persist_settings_or_restore(context.clone(), restore_point) {
                refresh_all(context.clone());
            }
        }
        Ok(false) => {}
        Err(message) => show_error_message(
            &context_window(&context),
            match selection {
                Some(TreeSelection::Category(_)) => tr(language, "분류", "Category"),
                _ => tr(language, "워크스페이스", "Workspace"),
            },
            &message,
        ),
    }
}

fn move_root_tree_item(
    context: Rc<RefCell<GtkWindowContext>>,
    source: TreeRootItemRef,
    direction: MoveDirection,
    language: UiLanguage,
) -> Result<bool, String> {
    let root_items = context.borrow().state.settings().root_tree_items();
    let Some(destination) =
        tree_root_keyboard_move_destination(&root_items, source, direction.into())
    else {
        return Ok(false);
    };
    context
        .borrow_mut()
        .state
        .move_root_tree_item(source, destination)
        .map(|()| true)
        .map_err(|error| error.user_message_for_language(language).to_owned())
}

fn move_workspace_in_category(
    context: Rc<RefCell<GtkWindowContext>>,
    index: usize,
    direction: MoveDirection,
    language: UiLanguage,
) -> Result<bool, String> {
    let settings = context.borrow().state.settings().clone();
    let Some(destination) = workspace_keyboard_move_destination(
        &settings.workspaces,
        &settings.categories,
        index,
        direction.into(),
    ) else {
        return Ok(false);
    };
    context
        .borrow_mut()
        .state
        .move_workspace(index, destination)
        .map(|()| true)
        .map_err(|error| error.user_message_for_language(language).to_owned())
}

fn handle_move_workspace_to_root(context: Rc<RefCell<GtkWindowContext>>) {
    let Some(TreeSelection::Workspace(index)) = context.borrow().selected_tree else {
        return;
    };
    let can_move_to_root = {
        let context_ref = context.borrow();
        workspace_selection_can_move_to_root(
            context_ref.state.settings(),
            context_ref.selected_tree,
        )
    };
    if !can_move_to_root {
        return;
    }
    let language = context_language(&context.borrow());
    let restore_point = GtkStateRestorePoint::capture(&context.borrow());
    let result = context.borrow_mut().state.move_workspace_to_root(index);
    match result {
        Ok(()) => {
            if persist_settings_or_restore(context.clone(), restore_point) {
                refresh_all(context.clone());
            }
        }
        Err(error) => show_error_message(
            &context_window(&context),
            tr(language, "워크스페이스", "Workspace"),
            &error.user_message_for_language(language),
        ),
    }
}

fn handle_font_settings(context: Rc<RefCell<GtkWindowContext>>) {
    let language = context_language(&context.borrow());
    let fonts = installed_font_families(&context_window(&context));
    let view = context.borrow().state.settings().view.clone();
    if let Some(view) = show_font_dialog(&context_window(&context), &view, &fonts, language) {
        apply_view_settings(context.clone(), view);
    }
}

fn handle_theme_settings(context: Rc<RefCell<GtkWindowContext>>, theme: ViewTheme) {
    let mut view = context.borrow().state.settings().view.clone();
    if ViewTheme::from_config_value(&view.theme).unwrap_or_default() == theme {
        return;
    }
    view.theme = theme.as_config_value().to_owned();
    apply_view_settings(context, view);
}

fn handle_ui_language_settings(context: Rc<RefCell<GtkWindowContext>>, language: UiLanguage) {
    let mut view = context.borrow().state.settings().view.clone();
    if current_ui_language(&view) == language {
        return;
    }
    view.ui_language = language.as_config_value().to_owned();
    apply_view_settings(context, view);
}

fn apply_view_settings(context: Rc<RefCell<GtkWindowContext>>, view: ViewSettings) {
    let current = context.borrow().state.settings().view.clone();
    let view = applied_view_settings(&current, view);
    if current == view {
        return;
    }

    let restore_point = GtkStateRestorePoint::capture(&context.borrow());
    {
        let mut context_mut = context.borrow_mut();
        context_mut.state.set_view_settings(view);
    }
    if !persist_settings_or_restore(context.clone(), restore_point) {
        return;
    }

    update_view_presentation(&context.borrow());
    apply_view_css(&context.borrow());
    let (command_list, font_size) = {
        let context_ref = context.borrow();
        (
            context_ref.command_list.clone(),
            context_ref.state.settings().view.font_size,
        )
    };
    apply_command_flow_metrics(&command_list, font_size);
    clamp_current_tree_panel_width(&context.borrow());
    refresh_all(context);
}

fn applied_view_settings(current: &ViewSettings, view: ViewSettings) -> ViewSettings {
    ViewSettings::new(view.font_family, view.font_size, view.theme)
        .with_ui_language(view.ui_language)
        .with_window_layout_from(current)
}

fn update_view_presentation(context: &GtkWindowContext) {
    let language = context_language(context);
    context
        .menu_bar
        .set_menu_model(Some(&main_menu_model(language)));
    sync_view_action_state(context);
}

fn sync_view_action_state(context: &GtkWindowContext) {
    let Some(actions) = context.actions.as_ref() else {
        return;
    };
    let view = &context.state.settings().view;
    let theme = ViewTheme::from_config_value(&view.theme)
        .unwrap_or_default()
        .as_config_value();
    actions.theme.set_state(&theme.to_variant());
    actions
        .ui_language
        .set_state(&context_language(context).as_config_value().to_variant());
}

fn handle_language_config(context: Rc<RefCell<GtkWindowContext>>) {
    let language = context_language(&context.borrow());
    let languages = context.borrow().state.settings().languages.clone();
    if let Some(languages) =
        show_language_config_dialog(&context_window(&context), &languages, language)
    {
        let restore_point = GtkStateRestorePoint::capture(&context.borrow());
        let result = context
            .borrow_mut()
            .state
            .set_workspace_languages(languages);
        match result {
            Ok(()) => {
                if persist_settings_or_restore(context.clone(), restore_point) {
                    refresh_all(context.clone());
                }
            }
            Err(error) => show_warning_message(
                &context_window(&context),
                workspace_language_apply_error_title(language),
                &error.user_message_for_language(language),
            ),
        }
    }
}

fn handle_about(context: Rc<RefCell<GtkWindowContext>>) {
    let language = context_language(&context.borrow());
    let window = context.borrow().window.clone();
    show_about_dialog(&window, language);
}

fn about_message(app_title: &str, version: &str) -> String {
    format!("{app_title}  v{version}")
}

fn about_dialog_close_label(language: UiLanguage) -> &'static str {
    tr(language, "닫기", "Close")
}

fn about_link_markup(url: &str) -> String {
    format!("<a href=\"{url}\">{url}</a>")
}

fn show_about_dialog(parent: &ApplicationWindow, language: UiLanguage) {
    let dialog = Dialog::builder()
        .transient_for(parent)
        .modal(true)
        .title(tr(language, "정보", "About"))
        .default_width(ABOUT_DIALOG_WIDTH)
        .default_height(ABOUT_DIALOG_HEIGHT)
        .build();
    dialog.add_button(about_dialog_close_label(language), ResponseType::Ok);
    dialog.set_default_response(ResponseType::Ok);

    let content = dialog.content_area();
    content.set_margin_top(24);
    content.set_margin_bottom(18);
    content.set_margin_start(24);
    content.set_margin_end(24);
    content.set_spacing(8);

    let message = Label::new(Some(&about_message(APP_TITLE, APP_VERSION)));
    message.set_xalign(0.0);
    content.append(&message);

    let link = Label::new(None);
    link.set_markup(&about_link_markup(APP_REPOSITORY_URL));
    link.set_xalign(0.0);
    content.append(&link);

    let separator = Separator::new(Orientation::Horizontal);
    separator.set_margin_top(4);
    separator.set_margin_bottom(2);
    content.append(&separator);

    let license_heading = Label::new(Some(about_license_heading(language)));
    license_heading.set_xalign(0.0);
    content.append(&license_heading);

    let license_notice = Label::new(Some(&crate::infra::about::load_about_text()));
    license_notice.set_xalign(0.0);
    license_notice.set_valign(Align::Start);
    license_notice.set_selectable(true);
    license_notice.set_wrap(true);
    license_notice.set_wrap_mode(pango::WrapMode::WordChar);

    let license_scroll = ScrolledWindow::builder()
        .hexpand(true)
        .vexpand(true)
        .hscrollbar_policy(gtk::PolicyType::Never)
        .vscrollbar_policy(gtk::PolicyType::Automatic)
        .min_content_height(100)
        .build();
    license_scroll.set_child(Some(&license_notice));
    content.append(&license_scroll);

    run_dialog(&dialog);
    close_dialog_and_present_parent(&dialog);
}

fn handle_close_main_window(context: Rc<RefCell<GtkWindowContext>>) {
    let window = context.borrow().window.clone();
    if window.root().is_some() {
        window.close();
    } else if let Some(app) = window.application() {
        app.quit();
    }
}

fn show_command_warning(
    context: &Rc<RefCell<GtkWindowContext>>,
    language: UiLanguage,
    message: &str,
) {
    let window = context.borrow().window.clone();
    show_warning_message(&window, tr(language, "명령", "Command"), message);
}

fn show_command_group_warning(
    context: &Rc<RefCell<GtkWindowContext>>,
    language: UiLanguage,
    message: &str,
) {
    let window = context.borrow().window.clone();
    show_warning_message(&window, tr(language, "명령 그룹", "Command Group"), message);
}

fn show_workspace_warning(
    context: &Rc<RefCell<GtkWindowContext>>,
    language: UiLanguage,
    message: &str,
) {
    let window = context.borrow().window.clone();
    show_warning_message(&window, tr(language, "워크스페이스", "Workspace"), message);
}

fn show_category_warning(
    context: &Rc<RefCell<GtkWindowContext>>,
    language: UiLanguage,
    message: &str,
) {
    let window = context.borrow().window.clone();
    show_warning_message(&window, tr(language, "분류", "Category"), message);
}

fn workspace_language_apply_error_title(language: UiLanguage) -> &'static str {
    tr(language, "언어", "Workspace Languages")
}

fn tree_edit_selection_required_message(language: UiLanguage) -> &'static str {
    tr(
        language,
        "편집할 항목을 선택하세요.",
        "Select an item to edit.",
    )
}

fn tree_delete_selection_required_message(language: UiLanguage) -> &'static str {
    tr(
        language,
        "삭제할 항목을 선택하세요.",
        "Select an item to delete.",
    )
}

fn tree_move_selection_required_message(language: UiLanguage) -> &'static str {
    tr(
        language,
        "이동할 항목을 선택하세요.",
        "Select an item to move.",
    )
}

fn selected_workspace_missing_message(language: UiLanguage) -> &'static str {
    tr(
        language,
        "선택한 워크스페이스를 찾을 수 없습니다.",
        "The selected workspace could not be found.",
    )
}

fn selected_category_missing_message(language: UiLanguage) -> &'static str {
    tr(
        language,
        "선택한 분류를 찾을 수 없습니다.",
        "The selected category could not be found.",
    )
}

fn command_group_rename_selection_required_message(language: UiLanguage) -> &'static str {
    tr(
        language,
        "이름을 바꿀 그룹을 선택하세요.",
        "Select a group to rename.",
    )
}

fn command_group_delete_selection_required_message(language: UiLanguage) -> &'static str {
    tr(
        language,
        "삭제할 명령 그룹을 선택하세요.",
        "Select a command group to delete.",
    )
}

fn command_group_move_selection_required_message(language: UiLanguage) -> &'static str {
    tr(
        language,
        "이동할 명령 그룹을 선택하세요.",
        "Select a command group to move.",
    )
}

fn selected_command_group_missing_message(language: UiLanguage) -> &'static str {
    tr(
        language,
        "선택한 명령 그룹을 찾을 수 없습니다.",
        "The selected command group could not be found.",
    )
}

fn cannot_move_further_message(language: UiLanguage) -> &'static str {
    tr(
        language,
        "더 이동할 수 없습니다.",
        "Cannot move any further.",
    )
}

fn command_run_selection_required_message(language: UiLanguage) -> &'static str {
    tr(
        language,
        "실행할 명령을 선택하세요.",
        "Select a command to run.",
    )
}

fn command_add_group_required_message(language: UiLanguage) -> &'static str {
    tr(
        language,
        "명령 그룹을 먼저 선택하세요.",
        "Select a command group first.",
    )
}

fn command_edit_group_required_message(language: UiLanguage) -> &'static str {
    tr(
        language,
        "명령 그룹을 선택하세요.",
        "Select a command group.",
    )
}

fn command_edit_selection_required_message(language: UiLanguage) -> &'static str {
    tr(
        language,
        "편집할 명령을 선택하세요.",
        "Select a command to edit.",
    )
}

fn command_delete_group_required_message(language: UiLanguage) -> &'static str {
    tr(
        language,
        "명령 그룹을 선택하세요.",
        "Select a command group.",
    )
}

fn command_delete_selection_required_message(language: UiLanguage) -> &'static str {
    tr(
        language,
        "삭제할 명령을 선택하세요.",
        "Select a command to delete.",
    )
}

fn command_move_group_required_message(language: UiLanguage) -> &'static str {
    tr(
        language,
        "명령 그룹을 선택하세요.",
        "Select a command group.",
    )
}

fn command_move_selection_required_message(language: UiLanguage) -> &'static str {
    tr(
        language,
        "이동할 명령을 선택하세요.",
        "Select a command to move.",
    )
}

fn selected_command_missing_message(language: UiLanguage) -> &'static str {
    tr(
        language,
        "선택한 명령을 찾을 수 없습니다.",
        "The selected command could not be found.",
    )
}

fn handle_add_command_tab(context: Rc<RefCell<GtkWindowContext>>) {
    let language = context_language(&context.borrow());
    if let Some(name) = show_text_input_dialog(
        &context_window(&context),
        tr(language, "그룹 추가", "Add Group"),
        tr(language, "그룹 이름", "Group name"),
        "",
        false,
        language,
    ) {
        match CommandTab::new(name, Vec::new()) {
            Ok(tab) => {
                let restore_point = GtkStateRestorePoint::capture(&context.borrow());
                let result = context.borrow_mut().state.add_command_tab(tab);
                match result {
                    Ok(_) => {
                        if persist_settings_or_restore(context.clone(), restore_point) {
                            refresh_all(context.clone());
                        }
                    }
                    Err(error) => show_error_message(
                        &context_window(&context),
                        tr(language, "명령 그룹", "Command Group"),
                        &error.user_message_for_language(language),
                    ),
                }
            }
            Err(error) => show_warning_message(
                &context_window(&context),
                tr(language, "명령 그룹", "Command Group"),
                &error.user_message_for_language(language),
            ),
        }
    }
}

fn handle_rename_command_tab(context: Rc<RefCell<GtkWindowContext>>) {
    let language = context_language(&context.borrow());
    let index = context.borrow().state.selected_command_tab_index();
    let Some(index) = index else {
        show_command_group_warning(
            &context,
            language,
            command_group_rename_selection_required_message(language),
        );
        update_menu_state(&context.borrow());
        return;
    };
    let name = context
        .borrow()
        .state
        .settings()
        .command_tabs
        .get(index)
        .map(|tab| tab.name.clone());
    let Some(name) = name else {
        context.borrow_mut().state.select_command_tab(None);
        show_command_group_warning(
            &context,
            language,
            selected_command_group_missing_message(language),
        );
        refresh_command_tab_selector(context.clone());
        refresh_command_buttons(context.clone());
        update_menu_state(&context.borrow());
        return;
    };

    if let Some(next_name) = show_text_input_dialog(
        &context_window(&context),
        tr(language, "그룹 이름 변경", "Rename Group"),
        tr(language, "그룹 이름", "Group name"),
        &name,
        false,
        language,
    ) {
        let restore_point = GtkStateRestorePoint::capture(&context.borrow());
        let result = context
            .borrow_mut()
            .state
            .rename_command_tab(index, next_name);
        match result {
            Ok(()) => {
                if persist_settings_or_restore(context.clone(), restore_point) {
                    refresh_all(context.clone());
                }
            }
            Err(error) => show_error_message(
                &context_window(&context),
                tr(language, "명령 그룹", "Command Group"),
                &error.user_message_for_language(language),
            ),
        }
    }
}

fn handle_delete_command_tab(context: Rc<RefCell<GtkWindowContext>>) {
    let language = context_language(&context.borrow());
    let index = context.borrow().state.selected_command_tab_index();
    let Some(index) = index else {
        show_command_group_warning(
            &context,
            language,
            command_group_delete_selection_required_message(language),
        );
        update_menu_state(&context.borrow());
        return;
    };
    let tab = context
        .borrow()
        .state
        .settings()
        .command_tabs
        .get(index)
        .cloned();
    let Some(tab) = tab else {
        context.borrow_mut().state.select_command_tab(None);
        show_command_group_warning(
            &context,
            language,
            selected_command_group_missing_message(language),
        );
        refresh_command_tab_selector(context.clone());
        refresh_command_buttons(context.clone());
        update_menu_state(&context.borrow());
        return;
    };

    let message = match language {
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
    if confirm(
        &context_window(&context),
        tr(language, "명령 그룹 삭제", "Delete Command Group"),
        &message,
    ) {
        let restore_point = GtkStateRestorePoint::capture(&context.borrow());
        context.borrow_mut().state.delete_command_tab(index);
        if persist_settings_or_restore(context.clone(), restore_point) {
            refresh_all(context.clone());
        }
    }
}

fn handle_move_command_tab(context: Rc<RefCell<GtkWindowContext>>, direction: MoveDirection) {
    let language = context_language(&context.borrow());
    let index = context.borrow().state.selected_command_tab_index();
    let Some(index) = index else {
        show_command_group_warning(
            &context,
            language,
            command_group_move_selection_required_message(language),
        );
        update_menu_state(&context.borrow());
        return;
    };
    let len = context.borrow().state.settings().command_tabs.len();
    let Some(destination) = command_tab_move_destination(index, len, direction.into()) else {
        show_command_group_warning(&context, language, cannot_move_further_message(language));
        update_menu_state(&context.borrow());
        return;
    };
    let restore_point = GtkStateRestorePoint::capture(&context.borrow());
    let result = context
        .borrow_mut()
        .state
        .move_command_tab(index, destination);
    match result {
        Ok(()) => {
            if persist_settings_or_restore(context.clone(), restore_point) {
                refresh_all(context.clone());
            }
        }
        Err(error) => show_error_message(
            &context_window(&context),
            tr(language, "명령 그룹", "Command Group"),
            &error.user_message_for_language(language),
        ),
    }
}

fn handle_add_command_button(context: Rc<RefCell<GtkWindowContext>>) {
    let language = context_language(&context.borrow());
    let tab_index = context.borrow().state.selected_command_tab_index();
    let Some(tab_index) = tab_index else {
        show_command_warning(
            &context,
            language,
            command_add_group_required_message(language),
        );
        update_menu_state(&context.borrow());
        return;
    };
    if context.borrow().state.selected_command_tab().is_none() {
        context.borrow_mut().state.select_command_tab(None);
        show_command_warning(
            &context,
            language,
            selected_command_group_missing_message(language),
        );
        refresh_command_tab_selector(context.clone());
        refresh_command_buttons(context.clone());
        update_menu_state(&context.borrow());
        return;
    };
    show_command_button_dialog(
        context.clone(),
        CommandButtonDialogMode::Add {
            tab_index,
            button_index: None,
        },
        None,
        language,
    );
}

fn handle_edit_command_button(context: Rc<RefCell<GtkWindowContext>>) {
    let language = context_language(&context.borrow());
    let tab_index = context.borrow().state.selected_command_tab_index();
    let button_index = context.borrow().state.selected_command_button_index();
    let Some(tab_index) = tab_index else {
        show_command_warning(
            &context,
            language,
            command_edit_group_required_message(language),
        );
        update_menu_state(&context.borrow());
        return;
    };
    let Some(button_index) = button_index else {
        show_command_warning(
            &context,
            language,
            command_edit_selection_required_message(language),
        );
        update_menu_state(&context.borrow());
        return;
    };
    let button = context.borrow().state.selected_command_button().cloned();
    let Some(button) = button else {
        context.borrow_mut().state.select_command_button(None);
        show_command_warning(
            &context,
            language,
            selected_command_missing_message(language),
        );
        refresh_command_buttons(context.clone());
        return;
    };

    show_command_button_dialog(
        context.clone(),
        CommandButtonDialogMode::Edit {
            tab_index,
            button_index,
        },
        Some(button),
        language,
    );
}

fn handle_delete_command_button(context: Rc<RefCell<GtkWindowContext>>) {
    let language = context_language(&context.borrow());
    let tab_index = context.borrow().state.selected_command_tab_index();
    let button_index = context.borrow().state.selected_command_button_index();
    let Some(tab_index) = tab_index else {
        show_command_warning(
            &context,
            language,
            command_delete_group_required_message(language),
        );
        update_menu_state(&context.borrow());
        return;
    };
    let Some(button_index) = button_index else {
        show_command_warning(
            &context,
            language,
            command_delete_selection_required_message(language),
        );
        update_menu_state(&context.borrow());
        return;
    };
    let button = context.borrow().state.selected_command_button().cloned();
    let Some(button) = button else {
        context.borrow_mut().state.select_command_button(None);
        show_command_warning(
            &context,
            language,
            selected_command_missing_message(language),
        );
        refresh_command_buttons(context.clone());
        return;
    };
    let message = match language {
        UiLanguage::Korean => format!(
            "명령을 삭제할까요?\n\n이름: {}\n실행 대상: {}",
            button.button_name, button.executable_path
        ),
        UiLanguage::English => format!(
            "Delete this command?\n\nName: {}\nExecutable: {}",
            button.button_name, button.executable_path
        ),
    };
    if confirm(
        &context_window(&context),
        tr(language, "명령 삭제", "Delete Command"),
        &message,
    ) {
        let restore_point = GtkStateRestorePoint::capture(&context.borrow());
        context
            .borrow_mut()
            .state
            .delete_command_button(tab_index, button_index);
        if persist_settings_or_restore(context.clone(), restore_point) {
            refresh_command_buttons(context.clone());
        }
    }
}

fn handle_move_command_button(context: Rc<RefCell<GtkWindowContext>>, direction: MoveDirection) {
    let language = context_language(&context.borrow());
    let tab_index = context.borrow().state.selected_command_tab_index();
    let button_index = context.borrow().state.selected_command_button_index();
    let Some(tab_index) = tab_index else {
        show_command_warning(
            &context,
            language,
            command_move_group_required_message(language),
        );
        update_menu_state(&context.borrow());
        return;
    };
    let Some(button_index) = button_index else {
        show_command_warning(
            &context,
            language,
            command_move_selection_required_message(language),
        );
        update_menu_state(&context.borrow());
        return;
    };
    let tab = context.borrow().state.selected_command_tab().cloned();
    let Some(tab) = tab else {
        context.borrow_mut().state.select_command_tab(None);
        show_command_warning(
            &context,
            language,
            selected_command_group_missing_message(language),
        );
        refresh_command_tab_selector(context.clone());
        refresh_command_buttons(context.clone());
        update_menu_state(&context.borrow());
        return;
    };
    let len = tab.buttons.len();
    let Some(destination) = command_button_move_destination(button_index, len, direction.into())
    else {
        show_command_warning(
            &context,
            language,
            tr(
                language,
                "더 이동할 수 없습니다.",
                "Cannot move any further.",
            ),
        );
        update_menu_state(&context.borrow());
        return;
    };
    let restore_point = GtkStateRestorePoint::capture(&context.borrow());
    if context.borrow().state.selected_command_button().is_none() {
        context.borrow_mut().state.select_command_button(None);
        show_command_warning(
            &context,
            language,
            selected_command_missing_message(language),
        );
        refresh_command_buttons(context.clone());
        return;
    }
    let result =
        context
            .borrow_mut()
            .state
            .move_command_button(tab_index, button_index, destination);
    match result {
        Ok(()) => {
            if persist_settings_or_restore(context.clone(), restore_point) {
                refresh_command_buttons(context.clone());
            }
        }
        Err(error) => show_error_message(
            &context_window(&context),
            tr(language, "명령", "Command"),
            &error.user_message_for_language(language),
        ),
    }
}

fn handle_run_selected_command(context: Rc<RefCell<GtkWindowContext>>) {
    let index = context.borrow().state.selected_command_button_index();
    if let Some(index) = index {
        handle_run_command_button(context, index);
    } else {
        let language = context_language(&context.borrow());
        show_command_warning(
            &context,
            language,
            command_run_selection_required_message(language),
        );
        update_menu_state(&context.borrow());
    }
}

fn handle_run_command_button(context: Rc<RefCell<GtkWindowContext>>, index: usize) {
    if command_execution_blocked_by_tree_selection(&context.borrow()) {
        return;
    }

    {
        let mut context_mut = context.borrow_mut();
        context_mut.state.select_command_button(Some(index));
    }

    let language = context_language(&context.borrow());
    let button = context.borrow().state.selected_command_button().cloned();
    let Some(button) = button else {
        context.borrow_mut().state.select_command_button(None);
        show_warning_message(
            &context_window(&context),
            tr(language, "명령", "Command"),
            selected_command_missing_message(language),
        );
        update_menu_state(&context.borrow());
        return;
    };
    let workspace = context.borrow().state.selected_workspace().cloned();

    if let Some(message) = command_workspace_error(language, &button, workspace.as_ref()) {
        context
            .borrow_mut()
            .state
            .set_status_message(message.clone());
        update_status_label(&context.borrow());
        show_warning_message(
            &context_window(&context),
            tr(language, "명령 실행", "Run Command"),
            &message,
        );
        return;
    }

    let arguments = match prepare_command_arguments(
        &context_window(&context),
        language,
        &button,
        workspace.as_ref(),
    ) {
        Ok(Some(arguments)) => arguments,
        Ok(None) => {
            context.borrow_mut().state.set_status_message(tr(
                language,
                "실행 취소",
                "Run canceled",
            ));
            update_status_label(&context.borrow());
            return;
        }
        Err(message) => {
            context
                .borrow_mut()
                .state
                .set_status_message(message.clone());
            update_status_label(&context.borrow());
            show_error_message(
                &context_window(&context),
                tr(language, "명령 실행", "Run Command"),
                &message,
            );
            return;
        }
    };

    let result = match button.execution_type {
        ExecutionType::ShellApi => {
            execute_shell_api_command(language, &button.executable_path, &arguments)
        }
        ExecutionType::ExternalTerminal => match workspace.as_ref() {
            Some(workspace) => execute_external_terminal_command(
                language,
                &button.executable_path,
                &arguments,
                workspace,
            ),
            None => Err(tr(
                language,
                "워크스페이스를 선택하세요.",
                "Select a workspace.",
            )
            .to_owned()),
        },
    };

    match result {
        Ok(()) => {
            let message = match language {
                UiLanguage::Korean => format!("실행 완료: {}", button.button_name),
                UiLanguage::English => format!("Run completed: {}", button.button_name),
            };
            context.borrow_mut().state.set_status_message(message);
        }
        Err(message) => {
            context
                .borrow_mut()
                .state
                .set_status_message(message.clone());
            show_error_message(
                &context_window(&context),
                tr(language, "명령 실행", "Run Command"),
                &message,
            );
        }
    }
    update_status_label(&context.borrow());
}

fn install_folder_drop_target(context: Rc<RefCell<GtkWindowContext>>) {
    let target = gtk::DropTarget::new(gdk::FileList::static_type(), gdk::DragAction::COPY);
    target.connect_drop({
        let context = context.clone();
        move |_, value, _, _| {
            let Ok(file_list) = value.get::<gdk::FileList>() else {
                return false;
            };
            let paths = file_list
                .files()
                .into_iter()
                .filter_map(|file| file.path())
                .collect::<Vec<_>>();
            handle_workspace_drop_paths(context.clone(), paths)
        }
    });
    context.borrow().tree_list.add_controller(target);
}

fn install_tree_row_drag_drop(
    context: Rc<RefCell<GtkWindowContext>>,
    row: &ListBoxRow,
    row_ref: TreeRowRef,
) {
    if let TreeRowRef::Workspace(index) = row_ref {
        let source = gtk::DragSource::new();
        source.set_actions(gdk::DragAction::MOVE);
        source.connect_prepare({
            let context = context.clone();
            move |_, _, _| {
                context.borrow_mut().active_workspace_drag_source = Some(index);
                Some(gdk::ContentProvider::for_value(
                    &format!("{TREE_ROW_DATA_PREFIX}{index}").to_value(),
                ))
            }
        });
        source.connect_drag_cancel({
            let context = context.clone();
            move |_, _, _| {
                context.borrow_mut().active_workspace_drag_source = None;
                false
            }
        });
        source.connect_drag_end({
            let context = context.clone();
            move |_, _, _| {
                context.borrow_mut().active_workspace_drag_source = None;
            }
        });
        row.add_controller(source);
    }

    let target = gtk::DropTarget::new(String::static_type(), gdk::DragAction::MOVE);
    install_tree_drop_feedback(&target, context.clone(), row, row_ref);
    target.connect_drop({
        let context = context.clone();
        let row = row.clone();
        move |_, value, _x, y| {
            row.remove_css_class("drop-target");
            let Ok(data) = value.get::<String>() else {
                return false;
            };
            let Some(source_index) = data
                .strip_prefix(TREE_ROW_DATA_PREFIX)
                .and_then(|value| value.parse::<usize>().ok())
            else {
                return false;
            };
            let insert_after = y > f64::from(row.allocated_height()) / 2.0;
            handle_workspace_internal_drop(context.clone(), source_index, row_ref, insert_after)
        }
    });
    row.add_controller(target);
}

fn install_command_row_drag_drop(
    context: Rc<RefCell<GtkWindowContext>>,
    row: &FlowBoxChild,
    index: usize,
) {
    let source = gtk::DragSource::new();
    source.set_actions(gdk::DragAction::MOVE);
    source.connect_prepare({
        let context = context.clone();
        move |_, _, _| {
            context.borrow_mut().active_command_drag_source = Some(index);
            Some(gdk::ContentProvider::for_value(
                &format!("{COMMAND_ROW_DATA_PREFIX}{index}").to_value(),
            ))
        }
    });
    source.connect_drag_cancel({
        let context = context.clone();
        move |_, _, _| {
            context.borrow_mut().active_command_drag_source = None;
            false
        }
    });
    source.connect_drag_end({
        let context = context.clone();
        move |_, _, _| {
            context.borrow_mut().active_command_drag_source = None;
        }
    });
    row.add_controller(source);

    let target = gtk::DropTarget::new(String::static_type(), gdk::DragAction::MOVE);
    install_command_drop_feedback(&target, context.clone(), row, index);
    target.connect_drop({
        let context = context.clone();
        let row = row.clone();
        move |_, value, _, _| {
            row.remove_css_class("drop-target");
            let Ok(data) = value.get::<String>() else {
                return false;
            };
            let Some(source_index) = data
                .strip_prefix(COMMAND_ROW_DATA_PREFIX)
                .and_then(|value| value.parse::<usize>().ok())
            else {
                return false;
            };
            handle_command_internal_drop(context.clone(), source_index, index)
        }
    });
    row.add_controller(target);
}

fn install_tree_drop_feedback(
    target: &gtk::DropTarget,
    context: Rc<RefCell<GtkWindowContext>>,
    row: &ListBoxRow,
    row_ref: TreeRowRef,
) {
    target.connect_enter({
        let context = context.clone();
        let row = row.clone();
        move |_, _, y| tree_drop_feedback_action(&context, &row, row_ref, y)
    });
    target.connect_motion({
        let context = context.clone();
        let row = row.clone();
        move |_, _, y| tree_drop_feedback_action(&context, &row, row_ref, y)
    });
    target.connect_leave({
        let row = row.clone();
        move |_| {
            row.remove_css_class("drop-target");
        }
    });
}

fn install_command_drop_feedback(
    target: &gtk::DropTarget,
    context: Rc<RefCell<GtkWindowContext>>,
    row: &FlowBoxChild,
    target_index: usize,
) {
    target.connect_enter({
        let context = context.clone();
        let row = row.clone();
        move |_, _, _| command_drop_feedback_action(&context, &row, target_index)
    });
    target.connect_motion({
        let context = context.clone();
        let row = row.clone();
        move |_, _, _| command_drop_feedback_action(&context, &row, target_index)
    });
    target.connect_leave({
        let row = row.clone();
        move |_| {
            row.remove_css_class("drop-target");
        }
    });
}

fn tree_drop_feedback_action(
    context: &Rc<RefCell<GtkWindowContext>>,
    row: &ListBoxRow,
    row_ref: TreeRowRef,
    y: f64,
) -> gdk::DragAction {
    let insert_after = y > f64::from(row.allocated_height()) / 2.0;
    let is_valid = {
        let context_ref = context.borrow();
        context_ref
            .active_workspace_drag_source
            .is_some_and(|source| {
                workspace_internal_drop_action(
                    context_ref.state.settings(),
                    source,
                    row_ref,
                    insert_after,
                )
                .is_some()
            })
    };
    set_drop_feedback_class(row, is_valid);
    if is_valid {
        gdk::DragAction::MOVE
    } else {
        gdk::DragAction::empty()
    }
}

fn command_drop_feedback_action(
    context: &Rc<RefCell<GtkWindowContext>>,
    row: &FlowBoxChild,
    target_index: usize,
) -> gdk::DragAction {
    let is_valid = {
        let context_ref = context.borrow();
        let button_count = context_ref
            .state
            .selected_command_tab()
            .map(|tab| tab.buttons.len())
            .unwrap_or_default();
        command_internal_drop_destination(
            context_ref.active_command_drag_source,
            target_index,
            button_count,
        )
        .is_some()
    };
    set_drop_feedback_class(row, is_valid);
    if is_valid {
        gdk::DragAction::MOVE
    } else {
        gdk::DragAction::empty()
    }
}

fn command_internal_drop_destination(
    source_index: Option<usize>,
    target_index: usize,
    button_count: usize,
) -> Option<usize> {
    command_button_drop_destination(source_index?, target_index, button_count)
}

fn set_drop_feedback_class<W>(row: &W, is_valid: bool)
where
    W: IsA<gtk::Widget>,
{
    if is_valid {
        row.add_css_class("drop-target");
    } else {
        row.remove_css_class("drop-target");
    }
}

fn handle_workspace_drop_paths(
    context: Rc<RefCell<GtkWindowContext>>,
    paths: Vec<PathBuf>,
) -> bool {
    let language = context_language(&context.borrow());
    let folder = match validate_workspace_drop_paths(&context.borrow(), &paths) {
        Ok(folder) => folder,
        Err(reason) => {
            show_warning_message(
                &context_window(&context),
                tr(language, "워크스페이스", "Workspace"),
                &workspace_drop_reject_message(language, reason),
            );
            return false;
        }
    };

    let workspace = workspace_from_dropped_folder(&context.borrow(), &folder);
    let workspace = match workspace {
        Some(workspace) => workspace,
        None => {
            let reserved_paths = workspace_paths_except(&context.borrow(), None);
            let options = context.borrow().state.settings().languages.clone();
            let Some(workspace) = show_workspace_dialog(
                &context_window(&context),
                WorkspaceDialogMode::Add,
                Some(Workspace {
                    path: folder.path.display().to_string(),
                    name: folder.default_name,
                    language: default_workspace_language_for_options(&options),
                    category: None,
                }),
                reserved_paths,
                options,
                language,
            ) else {
                return true;
            };
            workspace
        }
    };

    let restore_point = GtkStateRestorePoint::capture(&context.borrow());
    let result = context.borrow_mut().state.add_workspace(workspace);
    match result {
        Ok(_) => {
            if persist_settings_or_restore(context.clone(), restore_point) {
                refresh_all(context.clone());
                true
            } else {
                false
            }
        }
        Err(error) => {
            show_error_message(
                &context_window(&context),
                tr(language, "워크스페이스", "Workspace"),
                &error.user_message_for_language(language),
            );
            false
        }
    }
}

fn handle_workspace_internal_drop(
    context: Rc<RefCell<GtkWindowContext>>,
    source_index: usize,
    target: TreeRowRef,
    insert_after: bool,
) -> bool {
    let action = {
        let context_ref = context.borrow();
        workspace_internal_drop_action(
            context_ref.state.settings(),
            source_index,
            target,
            insert_after,
        )
    };
    let Some(action) = action else {
        return false;
    };

    let language = context_language(&context.borrow());
    let restore_point = GtkStateRestorePoint::capture(&context.borrow());
    let result = match action {
        GtkWorkspaceDropAction::ToCategory(category_index) => context
            .borrow_mut()
            .state
            .move_workspace_to_category(source_index, category_index)
            .map_err(|error| error.user_message_for_language(language).to_owned()),
        GtkWorkspaceDropAction::MoveWorkspace(destination) => context
            .borrow_mut()
            .state
            .move_workspace(source_index, destination)
            .map_err(|error| error.user_message_for_language(language).to_owned()),
        GtkWorkspaceDropAction::MoveRoot(destination) => context
            .borrow_mut()
            .state
            .move_root_tree_item(TreeRootItemRef::Workspace(source_index), destination)
            .map_err(|error| error.user_message_for_language(language).to_owned()),
    };

    match result {
        Ok(()) => {
            if persist_settings_or_restore(context.clone(), restore_point) {
                refresh_all(context.clone());
            }
            true
        }
        Err(message) => {
            show_error_message(
                &context_window(&context),
                tr(language, "워크스페이스", "Workspace"),
                &message,
            );
            false
        }
    }
}

fn workspace_internal_drop_action(
    settings: &AppSettings,
    source_index: usize,
    target: TreeRowRef,
    insert_after: bool,
) -> Option<GtkWorkspaceDropAction> {
    match target {
        TreeRowRef::Category(category_index) => {
            let workspace = settings.workspaces.get(source_index)?;
            let category = settings.categories.get(category_index)?;
            if workspace_belongs_to_category(workspace, category) {
                None
            } else {
                Some(GtkWorkspaceDropAction::ToCategory(category_index))
            }
        }
        TreeRowRef::Workspace(target_index) => {
            let root_items = settings.root_tree_items();
            workspace_tree_drop_action(
                &settings.workspaces,
                &settings.categories,
                root_items.as_slice(),
                source_index,
                WorkspaceTreeDropTarget::Workspace {
                    index: target_index,
                    insert_after,
                },
            )
            .map(GtkWorkspaceDropAction::from)
        }
    }
}

fn handle_command_internal_drop(
    context: Rc<RefCell<GtkWindowContext>>,
    source_index: usize,
    target_index: usize,
) -> bool {
    let Some(tab_index) = context.borrow().state.selected_command_tab_index() else {
        return false;
    };
    let button_count = context
        .borrow()
        .state
        .selected_command_tab()
        .map(|tab| tab.buttons.len())
        .unwrap_or_default();
    let Some(destination_index) =
        command_button_drop_destination(source_index, target_index, button_count)
    else {
        return false;
    };
    let language = context_language(&context.borrow());
    let restore_point = GtkStateRestorePoint::capture(&context.borrow());
    let result =
        context
            .borrow_mut()
            .state
            .move_command_button(tab_index, source_index, destination_index);
    match result {
        Ok(()) => {
            if persist_settings_or_restore(context.clone(), restore_point) {
                refresh_command_buttons(context.clone());
            }
            true
        }
        Err(error) => {
            show_error_message(
                &context_window(&context),
                tr(language, "명령", "Command"),
                &error.user_message_for_language(language),
            );
            false
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum GtkWorkspaceDropAction {
    ToCategory(usize),
    MoveWorkspace(usize),
    MoveRoot(usize),
}

impl From<WorkspaceTreeDropAction> for GtkWorkspaceDropAction {
    fn from(action: WorkspaceTreeDropAction) -> Self {
        match action {
            WorkspaceTreeDropAction::MoveWorkspace { destination_index } => {
                Self::MoveWorkspace(destination_index)
            }
            WorkspaceTreeDropAction::MoveRootItem { destination_index } => {
                Self::MoveRoot(destination_index)
            }
        }
    }
}

#[derive(Clone, Debug)]
struct WorkspaceDropFolder {
    path: PathBuf,
    default_name: String,
    inferred_language: Option<String>,
}

enum WorkspaceDropRejectReason {
    Empty,
    MultipleItems(usize),
    NotFolder(PathBuf),
    UnreadableFolder(PathBuf),
    DuplicatePath { path: PathBuf, name: String },
}

fn validate_workspace_drop_paths(
    context: &GtkWindowContext,
    paths: &[PathBuf],
) -> Result<WorkspaceDropFolder, WorkspaceDropRejectReason> {
    if paths.is_empty() {
        return Err(WorkspaceDropRejectReason::Empty);
    }
    if paths.len() > 1 {
        return Err(WorkspaceDropRejectReason::MultipleItems(paths.len()));
    }
    let path = paths[0].clone();

    reject_duplicate_workspace_drop_path(context, &path)?;

    let metadata = std::fs::metadata(&path)
        .map_err(|_| WorkspaceDropRejectReason::UnreadableFolder(path.clone()))?;
    if !metadata.is_dir() {
        return Err(WorkspaceDropRejectReason::NotFolder(path));
    }

    let entry_names = std::fs::read_dir(&path)
        .map_err(|_| WorkspaceDropRejectReason::UnreadableFolder(path.clone()))?
        .take(WORKSPACE_LANGUAGE_INFERENCE_ENTRY_LIMIT)
        .filter_map(Result::ok)
        .filter_map(|entry| entry.file_name().into_string().ok())
        .collect::<Vec<_>>();

    let default_name = default_workspace_name_for_path(&path);
    let inferred_language = infer_workspace_language_from_entry_names(
        entry_names.iter().map(String::as_str),
    )
    .and_then(|language| {
        context
            .state
            .settings()
            .languages
            .iter()
            .find(|option| option.eq_ignore_ascii_case(language))
            .cloned()
    });
    Ok(WorkspaceDropFolder {
        path,
        default_name,
        inferred_language,
    })
}

fn reject_duplicate_workspace_drop_path(
    context: &GtkWindowContext,
    path: &Path,
) -> Result<(), WorkspaceDropRejectReason> {
    let path_text = path.display().to_string();
    if let Some(existing) = context
        .state
        .settings()
        .workspaces
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

fn workspace_from_dropped_folder(
    context: &GtkWindowContext,
    folder: &WorkspaceDropFolder,
) -> Option<Workspace> {
    let language = folder.inferred_language.clone()?;
    Workspace::new_with_language_options(
        folder.path.display().to_string(),
        folder.default_name.clone(),
        language,
        &context.state.settings().languages,
    )
    .ok()
}

fn infer_workspace_language_from_folder(path: &Path) -> Option<&'static str> {
    let entries = std::fs::read_dir(path).ok()?;
    let names = entries
        .take(WORKSPACE_LANGUAGE_INFERENCE_ENTRY_LIMIT)
        .filter_map(Result::ok)
        .filter_map(|entry| entry.file_name().into_string().ok())
        .collect::<Vec<_>>();
    infer_workspace_language_from_entry_names(names.iter().map(String::as_str))
}

fn command_workspace_error(
    language: UiLanguage,
    button: &CommandButton,
    workspace: Option<&Workspace>,
) -> Option<String> {
    let requires_workspace = button.execution_type == ExecutionType::ExternalTerminal
        || arguments_require_workspace(&button.arguments);
    if !requires_workspace {
        return None;
    }
    let Some(workspace) = workspace else {
        return Some(
            tr(
                language,
                "워크스페이스를 선택하세요.",
                "Select a workspace.",
            )
            .to_owned(),
        );
    };
    if is_accessible_folder(Path::new(&workspace.path)) {
        None
    } else {
        Some(format!(
            "{}: {}",
            tr(
                language,
                "워크스페이스 폴더를 열 수 없습니다",
                "Could not open the workspace folder"
            ),
            workspace.path
        ))
    }
}

fn prepare_command_arguments(
    window: &ApplicationWindow,
    language: UiLanguage,
    button: &CommandButton,
    workspace: Option<&Workspace>,
) -> Result<Option<String>, String> {
    let replacements = resolve_argument_replacements(&button.arguments, workspace, |token| {
        let value = match token {
            ArgumentToken::SelectFile => match select_file(window, language) {
                PathSelection::Selected(path) => path.display().to_string(),
                PathSelection::Canceled => return Ok(None),
                PathSelection::Failed(message) => return Err(message),
            },
            ArgumentToken::SelectDir => match select_folder(window, language) {
                PathSelection::Selected(path) => path.display().to_string(),
                PathSelection::Canceled => return Ok(None),
                PathSelection::Failed(message) => return Err(message),
            },
            ArgumentToken::InputText => match show_text_input_dialog(
                window,
                tr(language, "텍스트 입력", "Text Input"),
                tr(language, "텍스트", "Text"),
                "",
                true,
                language,
            ) {
                Some(value) => value,
                None => return Ok(None),
            },
            ArgumentToken::Path | ArgumentToken::Name | ArgumentToken::Language => {
                return Err(format!("unexpected interactive token: {}", token.literal()));
            }
        };
        Ok(Some(value))
    })
    .map_err(|error| error.user_message_for_language(language))?;

    let Some(replacements) = replacements else {
        return Ok(None);
    };
    let replacements = replacements
        .into_iter()
        .map(|(token, value)| {
            (
                token,
                argument_token_execution_value(button.execution_type, value),
            )
        })
        .collect::<Vec<_>>();

    replace_argument_tokens(&button.arguments, &replacements)
        .map(Some)
        .map_err(|error| error.user_message_for_language(language))
}

fn argument_token_execution_value(execution_type: ExecutionType, value: String) -> String {
    match execution_type {
        ExecutionType::ShellApi | ExecutionType::ExternalTerminal => {
            quote_posix_shell_argument(&value)
        }
    }
}

fn execute_shell_api_command(
    language: UiLanguage,
    executable_path: &str,
    arguments: &str,
) -> Result<(), String> {
    let executable = executable_path.trim();
    if executable.is_empty() {
        return Err(empty_executable_message(language));
    }
    reject_interior_nul(language, "executable_path", executable)?;
    reject_interior_nul(language, "arguments", arguments)?;

    let command_line = command_line_from_executable_and_arguments(executable, arguments);
    Command::new("sh")
        .arg("-c")
        .arg(command_line)
        .spawn()
        .map(|_| ())
        .map_err(|error| command_failed_message(language, &error.to_string()))
}

fn execute_external_terminal_command(
    language: UiLanguage,
    executable_path: &str,
    arguments: &str,
    workspace: &Workspace,
) -> Result<(), String> {
    let executable = executable_path.trim();
    if executable.is_empty() {
        return Err(empty_executable_message(language));
    }
    reject_interior_nul(language, "executable_path", executable)?;
    reject_interior_nul(language, "arguments", arguments)?;
    reject_interior_nul(language, "directory", &workspace.path)?;

    let command_line = command_line_from_executable_and_arguments(executable, arguments);
    let hold_command = format!(
        "{command_line}; status=$?; printf '\\n[exit %s] Press Enter to close...' \"$status\"; read _"
    );

    for candidate in terminal_candidates() {
        let result = spawn_terminal(&candidate, &workspace.path, &hold_command);
        match result {
            Ok(()) => return Ok(()),
            Err(TerminalSpawnError::NotFound) => continue,
            Err(TerminalSpawnError::Spawn { terminal, error }) => {
                return Err(terminal_launch_failed_message(language, &terminal, &error));
            }
        }
    }

    Err(no_supported_terminal_message(language))
}

fn command_line_from_executable_and_arguments(executable_path: &str, arguments: &str) -> String {
    let executable = quote_posix_shell_argument(executable_path);
    if arguments.trim().is_empty() {
        executable
    } else {
        format!("{executable} {arguments}")
    }
}

enum TerminalSpawnError {
    NotFound,
    Spawn { terminal: String, error: String },
}

fn terminal_candidates() -> Vec<String> {
    terminal_candidates_from_env(std::env::var("TERMINAL").ok())
}

fn terminal_candidates_from_env(terminal: Option<String>) -> Vec<String> {
    let mut candidates = Vec::new();
    if let Some(terminal) = terminal
        && !terminal.trim().is_empty()
    {
        candidates.push(terminal);
    }
    candidates.extend(
        [
            "x-terminal-emulator",
            "gnome-terminal",
            "konsole",
            "xfce4-terminal",
            "alacritty",
            "kitty",
            "xterm",
        ]
        .iter()
        .map(|value| (*value).to_owned()),
    );
    candidates
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct TerminalCommandSpec {
    program: String,
    args: Vec<String>,
    current_dir: Option<String>,
}

fn terminal_command_spec(
    terminal: &str,
    working_directory: &str,
    command_line: &str,
) -> TerminalCommandSpec {
    let (program, mut args) = split_terminal_launcher(terminal);
    let basename = Path::new(&program)
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or(&program);

    args.extend(match basename {
        "gnome-terminal" => [
            "--working-directory".to_owned(),
            working_directory.to_owned(),
            "--".to_owned(),
            "sh".to_owned(),
            "-lc".to_owned(),
            command_line.to_owned(),
        ]
        .into_iter()
        .collect::<Vec<_>>(),
        "konsole" => [
            "--workdir".to_owned(),
            working_directory.to_owned(),
            "-e".to_owned(),
            "sh".to_owned(),
            "-lc".to_owned(),
            command_line.to_owned(),
        ]
        .into_iter()
        .collect::<Vec<_>>(),
        "xfce4-terminal" => [
            "--working-directory".to_owned(),
            working_directory.to_owned(),
            "-e".to_owned(),
            format!("sh -lc {}", quote_posix_shell_argument(command_line)),
        ]
        .into_iter()
        .collect::<Vec<_>>(),
        _ => [
            "-e".to_owned(),
            "sh".to_owned(),
            "-lc".to_owned(),
            command_line.to_owned(),
        ]
        .into_iter()
        .collect::<Vec<_>>(),
    });
    let current_dir = (!matches!(basename, "gnome-terminal" | "konsole" | "xfce4-terminal"))
        .then(|| working_directory.to_owned());

    TerminalCommandSpec {
        program,
        args,
        current_dir,
    }
}

fn split_terminal_launcher(terminal: &str) -> (String, Vec<String>) {
    let mut parts = split_shell_like_words(terminal);
    if parts.is_empty() {
        return (terminal.trim().to_owned(), Vec::new());
    }
    let program = parts.remove(0);
    (program, parts)
}

fn split_shell_like_words(value: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut current = String::new();
    let mut chars = value.trim().chars().peekable();
    let mut quote: Option<char> = None;

    while let Some(character) = chars.next() {
        match (quote, character) {
            (Some('\''), '\'') | (Some('"'), '"') => quote = None,
            (None, '\'' | '"') => quote = Some(character),
            (Some('"'), '\\') => {
                if let Some(next) = chars.next() {
                    current.push(next);
                }
            }
            (None, '\\') => {
                if let Some(next) = chars.next() {
                    current.push(next);
                }
            }
            (None, character) if character.is_whitespace() => {
                if !current.is_empty() {
                    words.push(std::mem::take(&mut current));
                }
            }
            _ => current.push(character),
        }
    }

    if !current.is_empty() {
        words.push(current);
    }
    words
}

fn spawn_terminal(
    terminal: &str,
    working_directory: &str,
    command_line: &str,
) -> Result<(), TerminalSpawnError> {
    let spec = terminal_command_spec(terminal, working_directory, command_line);
    let mut command = Command::new(&spec.program);
    if let Some(current_dir) = spec.current_dir.as_deref() {
        command.current_dir(current_dir);
    }
    command.args(&spec.args);

    match command.spawn() {
        Ok(_) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            Err(TerminalSpawnError::NotFound)
        }
        Err(error) => Err(TerminalSpawnError::Spawn {
            terminal: terminal.to_owned(),
            error: error.to_string(),
        }),
    }
}

fn reject_interior_nul(
    language: UiLanguage,
    field: &'static str,
    value: &str,
) -> Result<(), String> {
    if value.contains('\0') {
        Err(unsupported_execution_value_message(language, field))
    } else {
        Ok(())
    }
}

fn empty_executable_message(language: UiLanguage) -> String {
    tr(language, "실행 대상을 입력하세요.", "Enter an executable.").to_owned()
}

fn unsupported_execution_value_message(language: UiLanguage, field: &'static str) -> String {
    format!(
        "{}: {field}",
        tr(
            language,
            "실행 값에 사용할 수 없는 문자가 있습니다",
            "The execution value contains an unsupported character"
        )
    )
}

fn command_failed_message(language: UiLanguage, detail: &str) -> String {
    format!(
        "{}\n\n{detail}",
        tr(language, "명령 실행 실패", "Command failed")
    )
}

fn terminal_launch_failed_message(language: UiLanguage, terminal: &str, error: &str) -> String {
    format!(
        "{}: {terminal} ({error})",
        tr(language, "터미널 실행 실패", "Terminal launch failed")
    )
}

fn no_supported_terminal_message(language: UiLanguage) -> String {
    tr(
        language,
        "지원되는 터미널 에뮬레이터를 찾을 수 없습니다.",
        "No supported terminal emulator was found.",
    )
    .to_owned()
}

fn quote_posix_shell_argument(value: &str) -> String {
    if value.is_empty() {
        return "''".to_owned();
    }
    if value
        .chars()
        .all(|character| character.is_ascii_alphanumeric() || "-_./:@%+=".contains(character))
    {
        return value.to_owned();
    }
    format!("'{}'", value.replace('\'', "'\\''"))
}

#[derive(Clone, Copy)]
enum WorkspaceDialogMode {
    Add,
    Edit,
}

fn show_workspace_dialog(
    parent: &ApplicationWindow,
    mode: WorkspaceDialogMode,
    initial: Option<Workspace>,
    reserved_paths: Vec<String>,
    language_options: Vec<String>,
    language: UiLanguage,
) -> Option<Workspace> {
    let title = match mode {
        WorkspaceDialogMode::Add => tr(language, "워크스페이스 추가", "Add Workspace"),
        WorkspaceDialogMode::Edit => tr(language, "워크스페이스 편집", "Edit Workspace"),
    };
    let dialog = Dialog::builder()
        .transient_for(parent)
        .modal(true)
        .title(title)
        .default_width(460)
        .default_height(236)
        .build();
    add_dialog_buttons(&dialog, language, SAVE_CANCEL_DIALOG_BUTTONS);

    let content = dialog.content_area();
    content.set_spacing(8);
    content.set_margin_top(12);
    content.set_margin_bottom(12);
    content.set_margin_start(12);
    content.set_margin_end(12);

    let path_entry = Entry::new();
    path_entry.set_editable(false);
    let name_entry = Entry::new();
    let language_combo = ComboBoxText::new();
    for option in &language_options {
        language_combo.append_text(option);
    }

    let default_language = default_workspace_language_for_options(&language_options);
    let initial_path = initial
        .as_ref()
        .map(|workspace| workspace.path.as_str())
        .unwrap_or("");
    let initial_name = initial
        .as_ref()
        .map(|workspace| workspace.name.as_str())
        .unwrap_or("");
    let initial_language = initial
        .as_ref()
        .map(|workspace| workspace.language.as_str())
        .unwrap_or(default_language.as_str());
    path_entry.set_text(initial_path);
    name_entry.set_text(initial_name);
    select_combo_text(&language_combo, initial_language);

    append_labeled_row(
        &content,
        tr(language, "이름", "Name"),
        &name_entry,
        None::<&Button>,
    );
    let browse_button = Button::with_label(tr(language, "찾기", "Browse"));
    append_labeled_row(
        &content,
        tr(language, "폴더", "Folder"),
        &path_entry,
        Some(&browse_button),
    );
    append_labeled_row(
        &content,
        tr(language, "언어", "Language"),
        &language_combo,
        None::<&Button>,
    );

    let previous_folder_default_name = Rc::new(RefCell::new(None::<String>));
    let language_inference_request = Rc::new(Cell::new(0_u32));
    browse_button.connect_clicked({
        let dialog = dialog.clone();
        let path_entry = path_entry.clone();
        let name_entry = name_entry.clone();
        let language_combo = language_combo.clone();
        let language_options = language_options.clone();
        let previous_folder_default_name = previous_folder_default_name.clone();
        let language_inference_request = language_inference_request.clone();
        move |_| match select_folder(&dialog, language) {
            PathSelection::Selected(path) => {
                let previous_default = previous_folder_default_name.borrow().clone();
                let update = workspace_folder_browse_update(
                    mode,
                    &path,
                    name_entry.text().as_str(),
                    previous_default.as_deref(),
                );
                path_entry.set_text(&update.path_text);
                if let Some(name) = update.replacement_name {
                    name_entry.set_text(&name);
                }
                if let Some(folder_name) = update.previous_folder_default_name {
                    *previous_folder_default_name.borrow_mut() = Some(folder_name);
                }
                if let Some(path) = update.language_inference_path {
                    begin_workspace_language_inference(
                        &dialog,
                        language_inference_request.clone(),
                        path,
                        language_options.clone(),
                        language_combo.clone(),
                    );
                }
            }
            PathSelection::Canceled => {}
            PathSelection::Failed(message) => show_warning_message(
                &dialog,
                tr(language, "폴더 선택", "Select Folder"),
                &message,
            ),
        }
    });

    loop {
        let response = run_dialog(&dialog);
        if response != ResponseType::Ok {
            close_dialog_and_present_parent(&dialog);
            return None;
        }

        let path = path_entry.text().trim().to_owned();
        let name = name_entry.text().trim().to_owned();
        let selected_language = language_combo
            .active_text()
            .map(|value| value.to_string())
            .filter(|value| !value.trim().is_empty());

        if path.is_empty() {
            show_warning_message(
                &dialog,
                tr(language, "워크스페이스", "Workspace"),
                tr(language, "폴더를 선택하세요.", "Select a folder."),
            );
            continue;
        }

        let Some(accessible) = workspace_path_accessible_with_pending_ui(
            &dialog,
            &name_entry,
            &path_entry,
            &browse_button,
            &language_combo,
            path.clone(),
        ) else {
            show_warning_message(
                &dialog,
                tr(language, "워크스페이스", "Workspace"),
                tr(
                    language,
                    "폴더를 확인할 수 없습니다.",
                    "Could not validate the folder.",
                ),
            );
            continue;
        };

        if !accessible {
            show_warning_message(
                &dialog,
                tr(language, "워크스페이스", "Workspace"),
                tr(
                    language,
                    "접근 가능한 폴더를 선택하세요.",
                    "Select an accessible folder.",
                ),
            );
            continue;
        }

        if name.is_empty() {
            show_warning_message(
                &dialog,
                tr(language, "워크스페이스", "Workspace"),
                tr(language, "이름을 입력하세요.", "Enter a name."),
            );
            continue;
        }

        let Some(selected_language) = selected_language else {
            show_warning_message(
                &dialog,
                tr(language, "워크스페이스", "Workspace"),
                tr(language, "언어를 선택하세요.", "Select a language."),
            );
            continue;
        };

        if reserved_paths
            .iter()
            .any(|reserved| workspace_paths_equal(reserved, &path))
        {
            show_warning_message(
                &dialog,
                tr(language, "워크스페이스", "Workspace"),
                tr(
                    language,
                    "이미 등록된 폴더입니다.",
                    "This folder is already registered.",
                ),
            );
            continue;
        }

        match Workspace::new_with_language_options(path, name, selected_language, &language_options)
        {
            Ok(workspace) => {
                close_dialog_and_present_parent(&dialog);
                return Some(workspace);
            }
            Err(error) => {
                show_warning_message(&dialog, title, &error.user_message_for_language(language))
            }
        }
    }
}

struct WorkspaceFolderBrowseUpdate {
    path_text: String,
    replacement_name: Option<String>,
    previous_folder_default_name: Option<String>,
    language_inference_path: Option<PathBuf>,
}

fn workspace_folder_browse_update(
    mode: WorkspaceDialogMode,
    path: &Path,
    current_name: &str,
    previous_folder_default_name: Option<&str>,
) -> WorkspaceFolderBrowseUpdate {
    let path_text = path.display().to_string();
    if !matches!(mode, WorkspaceDialogMode::Add) {
        return WorkspaceFolderBrowseUpdate {
            path_text,
            replacement_name: None,
            previous_folder_default_name: None,
            language_inference_path: None,
        };
    }

    let folder_name = default_workspace_name_for_path(path);
    let replacement_name =
        should_replace_workspace_default_name(current_name, previous_folder_default_name)
            .then(|| folder_name.clone());

    WorkspaceFolderBrowseUpdate {
        path_text,
        replacement_name,
        previous_folder_default_name: Some(folder_name),
        language_inference_path: Some(path.to_path_buf()),
    }
}

fn begin_workspace_language_inference(
    dialog: &Dialog,
    request_id: Rc<Cell<u32>>,
    path: PathBuf,
    language_options: Vec<String>,
    language_combo: ComboBoxText,
) {
    let id = request_id.get().wrapping_add(1);
    request_id.set(id);
    dialog.set_response_sensitive(ResponseType::Ok, false);

    let (sender, receiver) = std::sync::mpsc::channel();
    let spawn_result = std::thread::Builder::new()
        .name("workspace-language-infer".to_owned())
        .spawn(move || {
            let language = workspace_language_option_from_folder(&path, &language_options);
            let _ = sender.send(language);
        });

    if spawn_result.is_err() {
        dialog.set_response_sensitive(ResponseType::Ok, true);
        return;
    }

    let dialog = dialog.clone();
    glib::idle_add_local(move || match receiver.try_recv() {
        Ok(language) => {
            if request_id.get() == id {
                if let Some(language) = language {
                    select_combo_text(&language_combo, &language);
                }
                dialog.set_response_sensitive(ResponseType::Ok, true);
            }
            glib::ControlFlow::Break
        }
        Err(TryRecvError::Empty) => glib::ControlFlow::Continue,
        Err(TryRecvError::Disconnected) => {
            if request_id.get() == id {
                dialog.set_response_sensitive(ResponseType::Ok, true);
            }
            glib::ControlFlow::Break
        }
    });
}

fn workspace_language_option_from_folder(
    path: &Path,
    language_options: &[String],
) -> Option<String> {
    infer_workspace_language_from_folder(path).and_then(|inferred| {
        language_options
            .iter()
            .find(|option| option.eq_ignore_ascii_case(inferred))
            .cloned()
    })
}

fn workspace_path_accessible_with_pending_ui(
    dialog: &Dialog,
    name_entry: &Entry,
    path_entry: &Entry,
    browse_button: &Button,
    language_combo: &ComboBoxText,
    path: String,
) -> Option<bool> {
    set_workspace_dialog_path_validation_pending(
        dialog,
        name_entry,
        path_entry,
        browse_button,
        language_combo,
        true,
    );

    let (sender, receiver) = std::sync::mpsc::channel();
    let spawn_result = std::thread::Builder::new()
        .name("workspace-path-check".to_owned())
        .spawn(move || {
            let accessible = is_accessible_folder(Path::new(&path));
            let _ = sender.send(accessible);
        });

    if spawn_result.is_err() {
        set_workspace_dialog_path_validation_pending(
            dialog,
            name_entry,
            path_entry,
            browse_button,
            language_combo,
            false,
        );
        return None;
    }

    let result = Rc::new(RefCell::new(None));
    let loop_ = glib::MainLoop::new(None, false);
    let loop_clone = loop_.clone();
    let result_clone = result.clone();
    glib::idle_add_local(move || match receiver.try_recv() {
        Ok(accessible) => {
            *result_clone.borrow_mut() = Some(accessible);
            loop_clone.quit();
            glib::ControlFlow::Break
        }
        Err(TryRecvError::Empty) => glib::ControlFlow::Continue,
        Err(TryRecvError::Disconnected) => {
            loop_clone.quit();
            glib::ControlFlow::Break
        }
    });

    loop_.run();

    set_workspace_dialog_path_validation_pending(
        dialog,
        name_entry,
        path_entry,
        browse_button,
        language_combo,
        false,
    );
    result.borrow_mut().take()
}

fn set_workspace_dialog_path_validation_pending(
    dialog: &Dialog,
    name_entry: &Entry,
    path_entry: &Entry,
    browse_button: &Button,
    language_combo: &ComboBoxText,
    pending: bool,
) {
    let enabled = !pending;
    name_entry.set_sensitive(enabled);
    path_entry.set_sensitive(enabled);
    browse_button.set_sensitive(enabled);
    language_combo.set_sensitive(enabled);
    dialog.set_response_sensitive(ResponseType::Ok, enabled);
}

fn should_replace_workspace_default_name(
    current_name: &str,
    previous_folder_default_name: Option<&str>,
) -> bool {
    current_name.trim().is_empty()
        || previous_folder_default_name.is_some_and(|previous| previous == current_name)
}

#[derive(Clone, Copy)]
enum CommandButtonDialogMode {
    Add {
        tab_index: usize,
        button_index: Option<usize>,
    },
    Edit {
        tab_index: usize,
        button_index: usize,
    },
}

fn show_command_button_dialog(
    context: Rc<RefCell<GtkWindowContext>>,
    mode: CommandButtonDialogMode,
    initial: Option<CommandButton>,
    language: UiLanguage,
) {
    let parent = context.borrow().window.clone();
    let title = match mode {
        CommandButtonDialogMode::Add { .. } => tr(language, "명령 추가", "Add Command"),
        CommandButtonDialogMode::Edit { .. } => tr(language, "명령 편집", "Edit Command"),
    };
    let dialog = Dialog::builder()
        .transient_for(&parent)
        .modal(true)
        .title(title)
        .default_width(570)
        .default_height(360)
        .build();
    add_dialog_buttons(&dialog, language, COMMAND_BUTTON_DIALOG_BUTTONS);

    let content = dialog.content_area();
    content.set_spacing(8);
    content.set_margin_top(12);
    content.set_margin_bottom(12);
    content.set_margin_start(12);
    content.set_margin_end(12);

    let name_entry = Entry::new();
    let executable_entry = Entry::new();
    let arguments_entry = Entry::new();
    let shell_radio = CheckButton::with_label(tr(language, "직접", "Direct"));
    let terminal_radio = CheckButton::with_label(tr(language, "터미널", "Terminal"));
    terminal_radio.set_group(Some(&shell_radio));
    shell_radio.set_active(true);

    if let Some(button) = initial.as_ref() {
        name_entry.set_text(&button.button_name);
        executable_entry.set_text(&button.executable_path);
        arguments_entry.set_text(&button.arguments);
        match button.execution_type {
            ExecutionType::ShellApi => shell_radio.set_active(true),
            ExecutionType::ExternalTerminal => terminal_radio.set_active(true),
        }
    }

    append_labeled_row(
        &content,
        tr(language, "이름", "Name"),
        &name_entry,
        None::<&Button>,
    );
    let browse_button = Button::with_label(tr(language, "찾기", "Browse"));
    append_labeled_row(
        &content,
        tr(language, "실행 대상", "Executable"),
        &executable_entry,
        Some(&browse_button),
    );
    append_labeled_row(
        &content,
        tr(language, "인수", "Arguments"),
        &arguments_entry,
        None::<&Button>,
    );

    let token_grid = Grid::new();
    token_grid.set_column_spacing(12);
    token_grid.set_row_spacing(6);
    for (index, token) in ARGUMENT_TOKENS.iter().enumerate() {
        let token_button = Button::with_label(token);
        token_button.set_size_request(ARGUMENT_TOKEN_BUTTON_WIDTH, ARGUMENT_TOKEN_BUTTON_HEIGHT);
        token_button.connect_clicked({
            let arguments_entry = arguments_entry.clone();
            let token = (*token).to_owned();
            move |_| {
                insert_argument_token(&arguments_entry, &token);
            }
        });
        let (column, row) = argument_token_grid_position(index);
        token_grid.attach(&token_button, column, row, 1, 1);
    }
    content.append(&token_grid);

    let execution_box = GtkBox::new(Orientation::Horizontal, 12);
    execution_box.append(&Label::new(Some(tr(
        language,
        "실행 방식",
        "Execution Type",
    ))));
    execution_box.append(&shell_radio);
    execution_box.append(&terminal_radio);
    content.append(&execution_box);

    browse_button.connect_clicked({
        let dialog = dialog.clone();
        let executable_entry = executable_entry.clone();
        move |_| match select_executable_file(&dialog, language, executable_entry.text().as_str()) {
            PathSelection::Selected(path) => executable_entry.set_text(&path.display().to_string()),
            PathSelection::Canceled => {}
            PathSelection::Failed(message) => show_warning_message(&dialog, title, &message),
        }
    });

    let mut applied_add_button_index = match mode {
        CommandButtonDialogMode::Add { button_index, .. } => button_index,
        CommandButtonDialogMode::Edit { .. } => None,
    };

    loop {
        let response = run_dialog(&dialog);
        if response == ResponseType::Cancel || response == ResponseType::DeleteEvent {
            close_dialog_and_present_parent(&dialog);
            return;
        }

        let button = command_button_from_entries(
            &name_entry,
            &executable_entry,
            &arguments_entry,
            &shell_radio,
            language,
        );
        let button = match button {
            Ok(button) => button,
            Err(message) => {
                show_warning_message(&dialog, tr(language, "명령", "Command"), &message);
                continue;
            }
        };

        if response == ResponseType::Apply {
            if let Some(index) = apply_command_button_dialog_value(
                context.clone(),
                mode,
                applied_add_button_index,
                button.clone(),
                language,
            ) {
                applied_add_button_index = Some(index);
                refresh_command_buttons(context.clone());
            }
            continue;
        }

        if apply_command_button_dialog_value(
            context.clone(),
            mode,
            applied_add_button_index,
            button,
            language,
        )
        .is_some()
        {
            refresh_command_buttons(context.clone());
            close_dialog_and_present_parent(&dialog);
            return;
        }
    }
}

fn insert_argument_token(arguments_entry: &Entry, token: &str) {
    let mut position = match arguments_entry.selection_bounds() {
        Some((selection_start, selection_end)) => {
            let start = selection_start.min(selection_end);
            let end = selection_start.max(selection_end);
            arguments_entry.delete_text(start, end);
            start
        }
        None => arguments_entry.position(),
    };
    arguments_entry.insert_text(token, &mut position);
    arguments_entry.set_position(position);
    arguments_entry.grab_focus();
}

fn argument_token_grid_position(index: usize) -> (i32, i32) {
    let index = index as i32;
    (
        index % ARGUMENT_TOKEN_COLUMNS,
        index / ARGUMENT_TOKEN_COLUMNS,
    )
}

fn command_button_from_entries(
    name_entry: &Entry,
    executable_entry: &Entry,
    arguments_entry: &Entry,
    shell_radio: &CheckButton,
    language: UiLanguage,
) -> Result<CommandButton, String> {
    let button_name = name_entry.text().to_string();
    let executable_path = executable_entry.text().to_string();
    let arguments = arguments_entry.text().to_string();
    if let Some(message) =
        command_button_required_field_message(&button_name, &executable_path, language)
    {
        return Err(message.to_owned());
    }

    if let Some(message) = command_button_unknown_token_message(&arguments, language) {
        return Err(message);
    }
    CommandButton::new(
        button_name,
        executable_path,
        arguments,
        if shell_radio.is_active() {
            ExecutionType::ShellApi
        } else {
            ExecutionType::ExternalTerminal
        },
    )
    .map_err(|error| error.user_message_for_language(language))
}

fn command_button_unknown_token_message(arguments: &str, language: UiLanguage) -> Option<String> {
    let unknown = unknown_argument_tokens(arguments);
    if unknown.is_empty() {
        None
    } else {
        Some(ArgumentResolutionError::UnknownTokens(unknown).user_message_for_language(language))
    }
}

fn command_button_required_field_message(
    button_name: &str,
    executable_path: &str,
    language: UiLanguage,
) -> Option<&'static str> {
    if button_name.trim().is_empty() {
        Some(tr(language, "이름을 입력하세요.", "Enter a name."))
    } else if executable_path.trim().is_empty() {
        Some(tr(
            language,
            "실행 대상을 입력하세요.",
            "Enter an executable target.",
        ))
    } else {
        None
    }
}

fn apply_command_button_dialog_value(
    context: Rc<RefCell<GtkWindowContext>>,
    mode: CommandButtonDialogMode,
    applied_add_button_index: Option<usize>,
    button: CommandButton,
    language: UiLanguage,
) -> Option<usize> {
    let existing_button_index = match mode {
        CommandButtonDialogMode::Add { .. } => applied_add_button_index,
        CommandButtonDialogMode::Edit { button_index, .. } => Some(button_index),
    };
    if let Some(button_index) = existing_button_index {
        let tab_index = match mode {
            CommandButtonDialogMode::Add { tab_index, .. }
            | CommandButtonDialogMode::Edit { tab_index, .. } => tab_index,
        };
        match command_button_update_needed(
            &context.borrow().state,
            tab_index,
            button_index,
            &button,
        ) {
            Ok(false) => return Some(button_index),
            Ok(true) => {}
            Err(error) => {
                show_error_message(
                    &context_window(&context),
                    tr(language, "명령", "Command"),
                    &error.user_message_for_language(language),
                );
                return None;
            }
        }
    }

    let restore_point = GtkStateRestorePoint::capture(&context.borrow());
    let result = match mode {
        CommandButtonDialogMode::Add {
            tab_index,
            button_index: _,
        } => {
            if let Some(button_index) = applied_add_button_index {
                context
                    .borrow_mut()
                    .state
                    .update_command_button(tab_index, button_index, button)
                    .map(|_| button_index)
            } else {
                context
                    .borrow_mut()
                    .state
                    .add_command_button(tab_index, button)
            }
        }
        CommandButtonDialogMode::Edit {
            tab_index,
            button_index,
        } => context
            .borrow_mut()
            .state
            .update_command_button(tab_index, button_index, button)
            .map(|_| button_index),
    };

    match result {
        Ok(index) => persist_settings_or_restore(context, restore_point).then_some(index),
        Err(error) => {
            show_error_message(
                &context_window(&context),
                tr(language, "명령", "Command"),
                &error.user_message_for_language(language),
            );
            None
        }
    }
}

fn command_button_update_needed(
    state: &AppState,
    tab_index: usize,
    button_index: usize,
    button: &CommandButton,
) -> Result<bool, crate::domain::CommandButtonMutationError> {
    state
        .command_button_matches(tab_index, button_index, button)
        .map(|matches| !matches)
}

fn show_font_dialog(
    parent: &ApplicationWindow,
    current: &ViewSettings,
    fonts: &[String],
    language: UiLanguage,
) -> Option<ViewSettings> {
    let font_options = font_dialog_family_options(fonts, &current.font_family);
    let dialog = Dialog::builder()
        .transient_for(parent)
        .modal(true)
        .title(tr(language, "글꼴", "Font"))
        .default_width(500)
        .default_height(292)
        .build();
    add_dialog_buttons(&dialog, language, FONT_DIALOG_BUTTONS);

    let content = dialog.content_area();
    content.set_spacing(8);
    content.set_margin_top(12);
    content.set_margin_bottom(12);
    content.set_margin_start(12);
    content.set_margin_end(12);

    let font_combo = ComboBoxText::new();
    for font in &font_options {
        font_combo.append_text(font);
    }
    select_font_combo_text(&font_combo, &current.font_family);

    let size_combo = ComboBoxText::new();
    for size in UI_FONT_SIZE_OPTIONS {
        size_combo.append_text(&size.to_string());
    }
    select_combo_text(&size_combo, &current.font_size.to_string());

    let preview = Label::new(Some(tr(language, "미리보기 123 가나다", "Preview 123 ABC")));
    preview.add_css_class("font-preview");
    preview.set_margin_top(8);

    append_labeled_row(
        &content,
        tr(language, "글꼴", "Font"),
        &font_combo,
        None::<&Button>,
    );
    append_labeled_row(
        &content,
        tr(language, "크기", "Size"),
        &size_combo,
        None::<&Button>,
    );
    content.append(&preview);

    let update_preview = {
        let font_combo = font_combo.clone();
        let size_combo = size_combo.clone();
        let preview = preview.clone();
        move || {
            let family = font_combo
                .active_text()
                .map(|value| value.to_string())
                .unwrap_or_else(|| DEFAULT_FONT_FAMILY.to_owned());
            let size = size_combo
                .active_text()
                .and_then(|value| value.parse::<u16>().ok())
                .unwrap_or(DEFAULT_FONT_SIZE);
            let attrs = pango::AttrList::new();
            attrs.insert(pango::AttrFontDesc::new(
                &pango::FontDescription::from_string(&format!("{family} {size}")),
            ));
            preview.set_attributes(Some(&attrs));
        }
    };
    update_preview();
    font_combo.connect_changed({
        let update_preview = update_preview.clone();
        move |_| update_preview()
    });
    size_combo.connect_changed(move |_| update_preview());

    loop {
        let response = run_dialog(&dialog);
        if response == ResponseType::Cancel || response == ResponseType::DeleteEvent {
            close_dialog_and_present_parent(&dialog);
            return None;
        }
        if response == ResponseType::Other(1) {
            select_font_combo_text(&font_combo, DEFAULT_FONT_FAMILY);
            select_combo_text(&size_combo, &DEFAULT_FONT_SIZE.to_string());
            continue;
        }

        let mut next = current.clone();
        next.font_family = match selected_font_dialog_family(
            &font_options,
            font_combo.active_text().map(|value| value.to_string()),
            language,
        ) {
            Ok(font_family) => font_family,
            Err(message) => {
                show_warning_message(&dialog, tr(language, "글꼴", "Font"), &message);
                continue;
            }
        };
        next.font_size = normalize_ui_font_size(
            size_combo
                .active_text()
                .and_then(|value| value.parse::<u16>().ok())
                .unwrap_or(DEFAULT_FONT_SIZE),
        );
        close_dialog_and_present_parent(&dialog);
        return Some(next);
    }
}

fn font_dialog_family_options(fonts: &[String], current_family: &str) -> Vec<String> {
    let mut options = fonts.to_vec();
    if current_family.eq_ignore_ascii_case(DEFAULT_FONT_FAMILY)
        && !font_family_available_in_list(&options, DEFAULT_FONT_FAMILY)
    {
        options.push(DEFAULT_FONT_FAMILY.to_owned());
        options = normalize_font_family_list(options);
    }
    options
}

fn select_font_combo_text(combo: &ComboBoxText, value: &str) {
    if !select_combo_text(combo, value) {
        combo.set_active(Some(0));
    }
}

fn selected_font_dialog_family(
    fonts: &[String],
    selected_font: Option<String>,
    language: UiLanguage,
) -> Result<String, String> {
    let Some(font_family) = selected_font.filter(|value| !value.trim().is_empty()) else {
        return Err(font_dialog_selection_required_message(language).to_owned());
    };
    if font_family_available_in_list(fonts, &font_family) {
        Ok(font_family)
    } else {
        Err(font_dialog_selection_required_message(language).to_owned())
    }
}

fn font_dialog_selection_required_message(language: UiLanguage) -> &'static str {
    tr(
        language,
        "목록에서 글꼴을 선택하세요.",
        "Select a font from the list.",
    )
}

fn show_language_config_dialog(
    parent: &ApplicationWindow,
    current_languages: &[String],
    language: UiLanguage,
) -> Option<Vec<String>> {
    let dialog = Dialog::builder()
        .transient_for(parent)
        .modal(true)
        .title(tr(language, "워크스페이스 언어", "Workspace Languages"))
        .default_width(460)
        .default_height(330)
        .build();
    add_dialog_buttons(&dialog, language, LANGUAGE_CONFIG_DIALOG_BUTTONS);

    let content = dialog.content_area();
    content.set_spacing(8);
    content.set_margin_top(12);
    content.set_margin_bottom(12);
    content.set_margin_start(12);
    content.set_margin_end(12);

    let editor_label = Label::new(Some(tr(language, "언어 목록", "Language List")));
    editor_label.set_xalign(0.0);
    content.append(&editor_label);

    let editor = TextView::new();
    editor.set_vexpand(true);
    editor.buffer().set_text(&current_languages.join("\n"));
    content.append(&editor);

    loop {
        let response = run_dialog(&dialog);
        if response == ResponseType::Cancel || response == ResponseType::DeleteEvent {
            close_dialog_and_present_parent(&dialog);
            return None;
        }
        if response == ResponseType::Other(1) {
            editor
                .buffer()
                .set_text(&default_workspace_language_options().join("\n"));
            continue;
        }
        let buffer = editor.buffer();
        let text = buffer.text(&buffer.start_iter(), &buffer.end_iter(), true);
        let languages = parse_language_config_editor_text(text.as_str());
        match normalize_workspace_language_options(languages.clone()) {
            Ok(_) => {
                close_dialog_and_present_parent(&dialog);
                return Some(languages);
            }
            Err(error) => show_warning_message(
                &dialog,
                tr(language, "언어", "Language"),
                &error.user_message_for_language(language),
            ),
        }
    }
}

fn parse_language_config_editor_text(text: &str) -> Vec<String> {
    text.split([',', '\n', '\r'])
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum DialogButtonRole {
    Save,
    Cancel,
    Apply,
    Default,
}

const SAVE_CANCEL_DIALOG_BUTTONS: &[DialogButtonRole] =
    &[DialogButtonRole::Save, DialogButtonRole::Cancel];
const COMMAND_BUTTON_DIALOG_BUTTONS: &[DialogButtonRole] = &[
    DialogButtonRole::Save,
    DialogButtonRole::Cancel,
    DialogButtonRole::Apply,
];
const FONT_DIALOG_BUTTONS: &[DialogButtonRole] = &[
    DialogButtonRole::Default,
    DialogButtonRole::Apply,
    DialogButtonRole::Cancel,
];
const LANGUAGE_CONFIG_DIALOG_BUTTONS: &[DialogButtonRole] = &[
    DialogButtonRole::Default,
    DialogButtonRole::Save,
    DialogButtonRole::Cancel,
];

fn add_dialog_buttons(dialog: &Dialog, language: UiLanguage, buttons: &[DialogButtonRole]) {
    for button in buttons {
        dialog.add_button(
            dialog_button_label(*button, language),
            dialog_button_response(*button),
        );
    }
    if let Some(response) = dialog_default_response(buttons) {
        dialog.set_default_response(response);
    }
}

fn dialog_button_label(button: DialogButtonRole, language: UiLanguage) -> &'static str {
    match button {
        DialogButtonRole::Save => tr(language, "저장", "Save"),
        DialogButtonRole::Cancel => tr(language, "취소", "Cancel"),
        DialogButtonRole::Apply => tr(language, "적용", "Apply"),
        DialogButtonRole::Default => tr(language, "기본값", "Default"),
    }
}

fn dialog_button_response(button: DialogButtonRole) -> ResponseType {
    match button {
        DialogButtonRole::Save => ResponseType::Ok,
        DialogButtonRole::Cancel => ResponseType::Cancel,
        DialogButtonRole::Apply => ResponseType::Apply,
        DialogButtonRole::Default => ResponseType::Other(1),
    }
}

fn dialog_default_response(buttons: &[DialogButtonRole]) -> Option<ResponseType> {
    if buttons == FONT_DIALOG_BUTTONS {
        return Some(ResponseType::Apply);
    }

    buttons
        .contains(&DialogButtonRole::Save)
        .then_some(ResponseType::Ok)
}

fn show_text_input_dialog(
    parent: &ApplicationWindow,
    title: &str,
    prompt: &str,
    initial: &str,
    allow_empty: bool,
    language: UiLanguage,
) -> Option<String> {
    let dialog = Dialog::builder()
        .transient_for(parent)
        .modal(true)
        .title(title)
        .default_width(390)
        .default_height(154)
        .build();
    add_dialog_buttons(&dialog, language, SAVE_CANCEL_DIALOG_BUTTONS);

    let content = dialog.content_area();
    content.set_spacing(8);
    content.set_margin_top(12);
    content.set_margin_bottom(12);
    content.set_margin_start(12);
    content.set_margin_end(12);
    let label = Label::new(Some(prompt));
    label.set_xalign(0.0);
    let entry = Entry::new();
    entry.set_activates_default(true);
    entry.set_text(initial);
    content.append(&label);
    content.append(&entry);

    loop {
        let response = run_dialog(&dialog);
        if response != ResponseType::Ok {
            close_dialog_and_present_parent(&dialog);
            return None;
        }
        let raw_value = entry.text().to_string();
        let Some(value) = accepted_text_input_value(&raw_value, allow_empty) else {
            show_warning_message(
                &dialog,
                title,
                tr(language, "이름을 입력하세요.", "Enter a name."),
            );
            continue;
        };
        close_dialog_and_present_parent(&dialog);
        return Some(value);
    }
}

fn accepted_text_input_value(raw_value: &str, allow_empty: bool) -> Option<String> {
    if allow_empty {
        Some(raw_value.to_owned())
    } else {
        let value = raw_value.trim().to_owned();
        (!value.is_empty()).then_some(value)
    }
}

fn run_dialog(dialog: &Dialog) -> ResponseType {
    glib::MainContext::default().block_on(dialog.run_future())
}

fn close_dialog_and_present_parent(dialog: &Dialog) {
    if let Some(parent) = dialog.transient_for() {
        dialog.close();
        present_parent_window(&parent);
    } else {
        dialog.close();
    }
}

fn append_labeled_row<W, B>(container: &GtkBox, label: &str, widget: &W, button: Option<&B>)
where
    W: IsA<gtk::Widget>,
    B: IsA<gtk::Widget>,
{
    let row = GtkBox::new(Orientation::Horizontal, 8);
    let label_widget = Label::new(Some(label));
    label_widget.set_xalign(0.0);
    label_widget.set_width_chars(16);
    row.append(&label_widget);
    row.append(widget);
    widget.set_hexpand(true);
    if let Some(button) = button {
        row.append(button);
    }
    container.append(&row);
}

fn select_combo_text(combo: &ComboBoxText, value: &str) -> bool {
    let mut index = 0;
    loop {
        combo.set_active(Some(index));
        let Some(text) = combo.active_text() else {
            combo.set_active(None);
            return false;
        };
        if text.as_str() == value || text.as_str().eq_ignore_ascii_case(value) {
            return true;
        }
        index += 1;
    }
}

#[derive(Debug, Eq, PartialEq)]
enum PathSelection {
    Selected(PathBuf),
    Canceled,
    Failed(String),
}

fn select_file(parent: &impl IsA<gtk::Window>, language: UiLanguage) -> PathSelection {
    let dialog = gtk::FileDialog::builder()
        .title(tr(language, "파일 선택", "Select File"))
        .modal(true)
        .build();
    apply_file_dialog_filters(&dialog, all_files_filter_specs(language));
    open_file_dialog(parent, &dialog, language, SelectionKind::File)
}

fn select_executable_file(
    parent: &impl IsA<gtk::Window>,
    language: UiLanguage,
    current_path: &str,
) -> PathSelection {
    let dialog = gtk::FileDialog::builder()
        .title(tr(language, "실행 대상 선택", "Select Executable"))
        .modal(true)
        .build();
    apply_file_dialog_filters(&dialog, executable_file_filter_specs(language));
    configure_initial_file_dialog_path(&dialog, current_path);
    open_file_dialog(parent, &dialog, language, SelectionKind::File)
}

fn select_folder(parent: &impl IsA<gtk::Window>, language: UiLanguage) -> PathSelection {
    let dialog = gtk::FileDialog::builder()
        .title(tr(language, "폴더 선택", "Select Folder"))
        .modal(true)
        .build();
    select_folder_dialog(parent, &dialog, language)
}

#[derive(Clone, Copy)]
enum SelectionKind {
    File,
    Folder,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct FileDialogFilterSpec {
    name: &'static str,
    patterns: &'static [&'static str],
}

fn all_files_filter_specs(language: UiLanguage) -> Vec<FileDialogFilterSpec> {
    vec![FileDialogFilterSpec {
        name: tr(language, "모든 파일", "All Files"),
        patterns: ALL_FILE_PATTERNS,
    }]
}

fn executable_file_filter_specs(language: UiLanguage) -> Vec<FileDialogFilterSpec> {
    vec![
        FileDialogFilterSpec {
            name: tr(language, "실행 파일", "Executable Files"),
            patterns: EXECUTABLE_FILE_PATTERNS,
        },
        FileDialogFilterSpec {
            name: tr(language, "모든 파일", "All Files"),
            patterns: ALL_FILE_PATTERNS,
        },
    ]
}

fn apply_file_dialog_filters(dialog: &gtk::FileDialog, specs: Vec<FileDialogFilterSpec>) {
    let filters = gio::ListStore::new::<gtk::FileFilter>();
    let mut default_filter = None;
    for spec in specs {
        let filter = gtk_file_filter_from_spec(spec);
        if default_filter.is_none() {
            default_filter = Some(filter.clone());
        }
        filters.append(&filter);
    }
    dialog.set_filters(Some(&filters));
    dialog.set_default_filter(default_filter.as_ref());
}

fn gtk_file_filter_from_spec(spec: FileDialogFilterSpec) -> gtk::FileFilter {
    let filter = gtk::FileFilter::new();
    filter.set_name(Some(spec.name));
    for pattern in spec.patterns {
        filter.add_pattern(pattern);
    }
    filter
}

fn open_file_dialog(
    parent: &impl IsA<gtk::Window>,
    dialog: &gtk::FileDialog,
    language: UiLanguage,
    kind: SelectionKind,
) -> PathSelection {
    let result = glib::MainContext::default().block_on(dialog.open_future(Some(parent)));
    present_parent_window(parent);
    file_dialog_result_to_path_selection(result, language, kind)
}

fn select_folder_dialog(
    parent: &impl IsA<gtk::Window>,
    dialog: &gtk::FileDialog,
    language: UiLanguage,
) -> PathSelection {
    let result = glib::MainContext::default().block_on(dialog.select_folder_future(Some(parent)));
    present_parent_window(parent);
    file_dialog_result_to_path_selection(result, language, SelectionKind::Folder)
}

fn file_dialog_result_to_path_selection(
    result: Result<gio::File, glib::Error>,
    language: UiLanguage,
    kind: SelectionKind,
) -> PathSelection {
    match result {
        Ok(file) => file.path().map(PathSelection::Selected).unwrap_or_else(|| {
            PathSelection::Failed(match (language, kind) {
                (UiLanguage::Korean, SelectionKind::File) => {
                    "로컬 파일 경로를 선택하세요.".to_owned()
                }
                (UiLanguage::English, SelectionKind::File) => {
                    "Select a local file path.".to_owned()
                }
                (UiLanguage::Korean, SelectionKind::Folder) => {
                    "로컬 폴더 경로를 선택하세요.".to_owned()
                }
                (UiLanguage::English, SelectionKind::Folder) => {
                    "Select a local folder path.".to_owned()
                }
            })
        }),
        Err(error) if error.matches(gio::IOErrorEnum::Cancelled) => PathSelection::Canceled,
        Err(error) => PathSelection::Failed(match language {
            UiLanguage::Korean => format!("선택 대화상자 오류: {error}"),
            UiLanguage::English => format!("Selection dialog failed: {error}"),
        }),
    }
}

#[derive(Debug, Default, Eq, PartialEq)]
struct FileDialogInitialPath {
    file: Option<PathBuf>,
    folder: Option<PathBuf>,
    name: Option<String>,
}

fn configure_initial_file_dialog_path(dialog: &gtk::FileDialog, current_path: &str) {
    let initial = file_dialog_initial_path(current_path);
    if let Some(file) = initial.file {
        dialog.set_initial_file(Some(&gio::File::for_path(file)));
    }
    if let Some(folder) = initial.folder {
        dialog.set_initial_folder(Some(&gio::File::for_path(folder)));
    }
    if let Some(name) = initial.name {
        dialog.set_initial_name(Some(&name));
    }
}

fn file_dialog_initial_path(current_path: &str) -> FileDialogInitialPath {
    let current_path = current_path.trim();
    if current_path.is_empty() {
        return FileDialogInitialPath::default();
    }

    let path = Path::new(current_path);
    if path.is_file() {
        return FileDialogInitialPath {
            file: Some(path.to_path_buf()),
            folder: None,
            name: None,
        };
    }
    if path.is_dir() {
        return FileDialogInitialPath {
            file: None,
            folder: Some(path.to_path_buf()),
            name: None,
        };
    }

    let folder = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty() && parent.is_dir())
        .map(Path::to_path_buf);
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.is_empty())
        .map(ToOwned::to_owned);

    FileDialogInitialPath {
        file: None,
        folder,
        name,
    }
}

fn installed_font_families(window: &ApplicationWindow) -> Vec<String> {
    let fonts = window
        .pango_context()
        .list_families()
        .into_iter()
        .map(|family| family.name().to_string())
        .collect::<Vec<_>>();
    normalize_font_family_list(fonts)
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

fn font_family_available_in_list(fonts: &[String], font_family: &str) -> bool {
    fonts
        .iter()
        .any(|candidate| candidate.eq_ignore_ascii_case(font_family))
}

fn validate_startup_font_settings(window: &ApplicationWindow, state: &mut AppState) {
    validate_startup_font_settings_with(state, |font_family| {
        font_family_available(window, font_family)
    });
}

fn validate_startup_font_settings_with(
    state: &mut AppState,
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

    if !font_family_available(&normalized.font_family)
        && !normalized
            .font_family
            .eq_ignore_ascii_case(DEFAULT_FONT_FAMILY)
    {
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

fn font_family_available(window: &ApplicationWindow, font_family: &str) -> bool {
    window
        .pango_context()
        .list_families()
        .iter()
        .any(|candidate| candidate.name().eq_ignore_ascii_case(font_family))
}

fn apply_view_css(context: &GtkWindowContext) {
    let view = &context.state.settings().view;
    let theme = ViewTheme::from_config_value(&view.theme).unwrap_or_default();
    let palette = ThemePalette::for_theme(theme);
    let css = view_css(view, palette);

    let provider = gtk::CssProvider::new();
    provider.load_from_data(&css);
    if let Some(display) = gdk::Display::default() {
        gtk::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
    context.window.add_css_class(CSS_PROVIDER_NAME);
}

fn view_css(view: &ViewSettings, palette: ThemePalette) -> String {
    let css = format!(
        r#"
.{} * {{
    font-family: "{}";
    font-size: {}pt;
}}
.{} {{
    background: {};
    color: {};
}}
.workspace-tree, .command-list {{
    background: {};
    color: {};
}}
.tree-row, .command-row {{
    background: transparent;
    color: {};
}}
.tree-row:selected, .command-row:selected {{
    background: {};
    color: {};
}}
.tree-row:selected label, .command-row:selected label {{
    color: {};
}}
.tree-row.drop-target, .command-row.drop-target {{
    background: {};
}}
.category-row label {{
    font-weight: 700;
}}
.command-button {{
    min-height: {}px;
    margin: 0;
}}
.status-label {{
    color: {};
}}
.j3-compact-titlebar,
.j3-titlebar-layout {{
    min-height: 22px;
    padding-top: 0;
    padding-bottom: 0;
}}
.j3-titlebar-start {{
    min-width: 42px;
    min-height: 22px;
}}
.j3-titlebar-icon {{
    min-width: 12px;
    min-height: 12px;
    padding: 0;
    margin: 0;
}}
.j3-titlebar-controls {{
    min-height: 22px;
    padding: 0;
    margin: 0;
}}
.j3-titlebar-controls button {{
    min-width: 20px;
    min-height: 20px;
    padding: 0;
    margin: 0;
}}
.j3-titlebar-controls image {{
    -gtk-icon-size: 12px;
    min-width: 12px;
    min-height: 12px;
}}
.j3-titlebar-label {{
    min-height: 20px;
    padding-top: 0;
    padding-bottom: 0;
}}
"#,
        CSS_PROVIDER_NAME,
        css_string(&view.font_family),
        view.font_size,
        CSS_PROVIDER_NAME,
        palette.window_background,
        palette.text,
        palette.control_background,
        palette.text,
        palette.text,
        palette.selection_background,
        palette.selection_text,
        palette.selection_text,
        palette.drop_target_background,
        command_button_height_for_font(view.font_size),
        palette.muted_text,
    );
    css
}

fn css_string(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

struct ThemePalette {
    window_background: &'static str,
    control_background: &'static str,
    text: &'static str,
    muted_text: &'static str,
    selection_background: &'static str,
    selection_text: &'static str,
    drop_target_background: &'static str,
}

impl ThemePalette {
    fn for_theme(theme: ViewTheme) -> Self {
        match theme {
            ViewTheme::System | ViewTheme::Light => Self {
                window_background: "#f0f0f0",
                control_background: "#ffffff",
                text: "#000000",
                muted_text: "#4d4d4d",
                selection_background: "#0a64d6",
                selection_text: "#ffffff",
                drop_target_background: "#d8e9ff",
            },
            ViewTheme::ClassicDark => Self {
                window_background: "#1f2124",
                control_background: "#181a1d",
                text: "#e6e8eb",
                muted_text: "#b6bbc2",
                selection_background: "#4f6f99",
                selection_text: "#ffffff",
                drop_target_background: "#364a63",
            },
            ViewTheme::SepiaTeal => Self {
                window_background: "#181918",
                control_background: "#1f3438",
                text: "#ece8db",
                muted_text: "#c8bc9f",
                selection_background: "#4f766f",
                selection_text: "#ffffff",
                drop_target_background: "#456865",
            },
            ViewTheme::Graphite => Self {
                window_background: "#18191a",
                control_background: "#32373f",
                text: "#efece5",
                muted_text: "#c9c3b7",
                selection_background: "#627894",
                selection_text: "#ffffff",
                drop_target_background: "#4f627b",
            },
            ViewTheme::Forest => Self {
                window_background: "#161917",
                control_background: "#273b3f",
                text: "#ecefe5",
                muted_text: "#bad0bd",
                selection_background: "#3f7b61",
                selection_text: "#ffffff",
                drop_target_background: "#3d6b56",
            },
            ViewTheme::SteelBlue => Self {
                window_background: "#18191b",
                control_background: "#364050",
                text: "#eff0f2",
                muted_text: "#c4d0dd",
                selection_background: "#447aa8",
                selection_text: "#ffffff",
                drop_target_background: "#41688c",
            },
        }
    }
}

fn update_menu_state(context: &GtkWindowContext) {
    let Some(actions) = context.actions.as_ref() else {
        return;
    };
    let tree_state = current_tree_menu_state(context);
    actions
        .workspace_edit
        .set_enabled(tree_state.can_edit_tree_item);
    actions
        .workspace_delete
        .set_enabled(tree_state.can_delete_tree_item);
    actions
        .workspace_move_up
        .set_enabled(tree_state.can_move_up);
    actions
        .workspace_move_down
        .set_enabled(tree_state.can_move_down);

    let tab_index = context.state.selected_command_tab_index();
    let tab_len = context.state.settings().command_tabs.len();
    actions.tab_rename.set_enabled(tab_index.is_some());
    actions.tab_delete.set_enabled(tab_index.is_some());
    actions.tab_move_up.set_enabled(
        tab_index
            .and_then(|index| {
                command_tab_move_destination(index, tab_len, CommandTabMoveDirection::Left)
            })
            .is_some(),
    );
    actions.tab_move_down.set_enabled(
        tab_index
            .and_then(|index| {
                command_tab_move_destination(index, tab_len, CommandTabMoveDirection::Right)
            })
            .is_some(),
    );

    let command_state = current_command_menu_state(context);
    actions.command_run.set_enabled(command_state.can_execute);
    actions.command_add.set_enabled(command_state.can_add);
    actions.command_edit.set_enabled(command_state.can_edit);
    actions.command_delete.set_enabled(command_state.can_delete);
    actions
        .command_move_previous
        .set_enabled(command_state.can_move_previous);
    actions
        .command_move_next
        .set_enabled(command_state.can_move_next);
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct TreeMenuState {
    can_delete_tree_item: bool,
    can_edit_tree_item: bool,
    can_move_up: bool,
    can_move_down: bool,
}

fn current_tree_menu_state(context: &GtkWindowContext) -> TreeMenuState {
    let can_target_selection = selected_tree_exists(context);
    TreeMenuState {
        can_delete_tree_item: can_target_selection,
        can_edit_tree_item: can_target_selection,
        can_move_up: tree_can_move(context, MoveDirection::Up),
        can_move_down: tree_can_move(context, MoveDirection::Down),
    }
}

fn selected_tree_exists(context: &GtkWindowContext) -> bool {
    selected_tree_exists_in_settings(context.state.settings(), context.selected_tree)
}

fn tree_selection_missing_message(
    settings: &crate::domain::AppSettings,
    selection: Option<TreeSelection>,
    language: UiLanguage,
) -> Option<&'static str> {
    match selection {
        Some(TreeSelection::Workspace(index)) if settings.workspaces.get(index).is_none() => {
            Some(selected_workspace_missing_message(language))
        }
        Some(TreeSelection::Category(index)) if settings.categories.get(index).is_none() => {
            Some(selected_category_missing_message(language))
        }
        _ => None,
    }
}

fn selected_tree_exists_in_settings(
    settings: &crate::domain::AppSettings,
    selection: Option<TreeSelection>,
) -> bool {
    match selection {
        Some(TreeSelection::Workspace(index)) => settings.workspaces.get(index).is_some(),
        Some(TreeSelection::Category(index)) => settings.categories.get(index).is_some(),
        None => false,
    }
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

fn current_command_menu_state(context: &GtkWindowContext) -> CommandMenuState {
    let tab = context.state.selected_command_tab();
    let button_index = context.state.selected_command_button_index();
    let selected_button_exists = context.state.selected_command_button().is_some();
    let button_len = tab.map(|tab| tab.buttons.len()).unwrap_or_default();
    command_menu_state(
        tab.is_some(),
        selected_button_exists,
        button_index,
        button_len,
        command_execution_blocked_by_tree_selection(context),
    )
}

fn command_execution_blocked_by_tree_selection(context: &GtkWindowContext) -> bool {
    selected_tree_blocks_command_execution(context.state.settings(), context.selected_tree)
}

fn selected_tree_blocks_command_execution(
    settings: &crate::domain::AppSettings,
    selection: Option<TreeSelection>,
) -> bool {
    matches!(selection, Some(TreeSelection::Category(index)) if settings.categories.get(index).is_some())
}

fn command_menu_state(
    has_selected_tab: bool,
    has_selected_button: bool,
    selected_button_index: Option<usize>,
    selected_button_count: usize,
    tree_selection_blocks_execution: bool,
) -> CommandMenuState {
    let can_target_button = has_selected_tab && has_selected_button;
    CommandMenuState {
        can_execute: can_target_button && !tree_selection_blocks_execution,
        can_add: has_selected_tab,
        can_edit: can_target_button,
        can_delete: can_target_button,
        can_move_previous: can_target_button
            && selected_button_index
                .and_then(|index| {
                    command_button_move_destination(
                        index,
                        selected_button_count,
                        CommandButtonMoveDirection::Previous,
                    )
                })
                .is_some(),
        can_move_next: can_target_button
            && selected_button_index
                .and_then(|index| {
                    command_button_move_destination(
                        index,
                        selected_button_count,
                        CommandButtonMoveDirection::Next,
                    )
                })
                .is_some(),
    }
}

fn tree_can_move(context: &GtkWindowContext, direction: MoveDirection) -> bool {
    let settings = context.state.settings();
    match context.selected_tree {
        Some(TreeSelection::Workspace(index)) => {
            if workspace_is_root(context, index) {
                let root_items = settings.root_tree_items();
                tree_root_keyboard_move_destination(
                    &root_items,
                    TreeRootItemRef::Workspace(index),
                    direction.into(),
                )
                .is_some()
            } else {
                workspace_keyboard_move_destination(
                    &settings.workspaces,
                    &settings.categories,
                    index,
                    direction.into(),
                )
                .is_some()
            }
        }
        Some(TreeSelection::Category(index)) => {
            let root_items = settings.root_tree_items();
            tree_root_keyboard_move_destination(
                &root_items,
                TreeRootItemRef::Category(index),
                direction.into(),
            )
            .is_some()
        }
        None => false,
    }
}

fn workspace_is_root(context: &GtkWindowContext, index: usize) -> bool {
    let settings = context.state.settings();
    settings
        .workspaces
        .get(index)
        .and_then(|workspace| workspace_category_index(workspace, &settings.categories))
        .is_none()
}

fn category_workspace_groups(settings: &crate::domain::AppSettings) -> Vec<Vec<usize>> {
    let mut groups = vec![Vec::new(); settings.categories.len()];
    for (index, workspace) in settings.workspaces.iter().enumerate() {
        if let Some(category_index) = workspace_category_index(workspace, &settings.categories) {
            groups[category_index].push(index);
        }
    }
    groups
}

fn workspace_paths_except(context: &GtkWindowContext, except_index: Option<usize>) -> Vec<String> {
    context
        .state
        .settings()
        .workspaces
        .iter()
        .enumerate()
        .filter(|(index, _)| Some(*index) != except_index)
        .map(|(_, workspace)| workspace.path.clone())
        .collect()
}

#[derive(Clone)]
struct GtkStateRestorePoint {
    state: AppState,
    selected_tree: Option<TreeSelection>,
}

impl GtkStateRestorePoint {
    fn capture(context: &GtkWindowContext) -> Self {
        Self {
            state: context.state.clone(),
            selected_tree: context.selected_tree,
        }
    }
}

fn persist_settings_or_restore(
    context: Rc<RefCell<GtkWindowContext>>,
    restore_point: GtkStateRestorePoint,
) -> bool {
    let current_settings = context.borrow().state.settings().clone();
    match settings::save_user_settings_if_changed(&current_settings, restore_point.state.settings())
    {
        Ok(path) => {
            let language = context_language(&context.borrow());
            context.borrow_mut().state.set_status_message(format!(
                "{}: {}",
                tr(language, "설정 저장", "Settings saved"),
                path.display()
            ));
            update_status_label(&context.borrow());
            true
        }
        Err(error) => {
            let dialog_language = context_language(&context.borrow());
            let dialog_message = settings_save_failure_message(dialog_language, &error);
            let window = context.borrow().window.clone();
            show_error_message(
                &window,
                tr(dialog_language, "설정", "Settings"),
                &dialog_message,
            );

            let restored_language = current_ui_language(&restore_point.state.settings().view);
            let status_message = settings_save_failure_message(restored_language, &error);
            {
                let mut context_mut = context.borrow_mut();
                let mut restored_state =
                    AppState::from_settings(restore_point.state.settings().clone(), Vec::new());
                restored_state.select_workspace(restore_point.state.selected_workspace_index());
                restored_state.select_command_tab(restore_point.state.selected_command_tab_index());
                restored_state
                    .select_command_button(restore_point.state.selected_command_button_index());
                context_mut.state = restored_state;
                context_mut.selected_tree = restore_point.selected_tree;
                context_mut.state.set_status_message(status_message);
            }
            update_view_presentation(&context.borrow());
            apply_view_css(&context.borrow());
            refresh_all(context.clone());
            false
        }
    }
}

fn settings_save_failure_message(
    language: UiLanguage,
    error: &settings::SettingsSaveError,
) -> String {
    match language {
        UiLanguage::Korean => format!("설정 저장 실패\n\n{error}"),
        UiLanguage::English => format!("Settings save failed\n\n{error}"),
    }
}

fn persist_window_layout(context: Rc<RefCell<GtkWindowContext>>) -> bool {
    let (width, height, tree_width) = {
        let context = context.borrow();
        let layout = LayoutSpec::for_font_size(context.state.settings().view.font_size);
        let tree_width = clamp_tree_panel_width_for_window(
            layout,
            context.window.width(),
            context.paned.position(),
        );
        (context.window.width(), context.window.height(), tree_width)
    };
    let restore_point = GtkStateRestorePoint::capture(&context.borrow());
    {
        let mut context_mut = context.borrow_mut();
        let mut view = context_mut.state.settings().view.clone();
        view.set_window_layout(width, height, tree_width);
        if context_mut.state.settings().view == view {
            return true;
        }
        context_mut.state.set_view_settings(view);
    }
    persist_settings_or_restore(context, restore_point)
}

fn startup_window_size(
    initial_size: WindowSize,
    layout: LayoutSpec,
    view: &ViewSettings,
) -> WindowSize {
    let requested = WindowSize {
        width: view.window_width.unwrap_or(initial_size.width),
        height: view.window_height.unwrap_or(initial_size.height),
    };
    let minimum = minimum_main_window_size(layout);

    WindowSize {
        width: requested.width.max(minimum.width),
        height: requested.height.max(minimum.height),
    }
}

fn minimum_main_window_size(layout: LayoutSpec) -> WindowSize {
    WindowSize {
        width: layout.content_margin * 2
            + MIN_TREE_PANEL_WIDTH
            + layout.panel_gap
            + MIN_COMMAND_TABS_PANEL_WIDTH,
        height: layout.content_top_gap + MIN_MAIN_PANEL_HEIGHT + layout.content_margin,
    }
}

fn clamp_current_tree_panel_width(context: &GtkWindowContext) {
    let layout = LayoutSpec::for_font_size(context.state.settings().view.font_size);
    let width = context.window.width();
    let clamped = clamp_tree_panel_width_for_window(layout, width, context.paned.position());
    if context.paned.position() != clamped {
        context.paned.set_position(clamped);
    }
}

fn clamp_tree_panel_width_for_window(layout: LayoutSpec, window_width: i32, width: i32) -> i32 {
    let maximum_tree = (window_width
        - layout.content_margin * 2
        - layout.panel_gap
        - MIN_COMMAND_TABS_PANEL_WIDTH)
        .max(MIN_TREE_PANEL_WIDTH);
    width.clamp(MIN_TREE_PANEL_WIDTH, maximum_tree)
}

fn update_status_label(context: &GtkWindowContext) {
    context
        .status_label
        .set_text(context.state.status_message());
}

fn show_startup_warnings(context: Rc<RefCell<GtkWindowContext>>) {
    let (warnings, language, window) = {
        let context = context.borrow();
        (
            context.state.restore_warnings().to_vec(),
            context_language(&context),
            context.window.clone(),
        )
    };
    if warnings.is_empty() {
        return;
    }
    show_warning_message(
        &window,
        tr(language, "설정 복원", "Settings Restore"),
        &warnings.join("\n"),
    );
}

fn confirm(parent: &ApplicationWindow, title: &str, message: &str) -> bool {
    let dialog = gtk::MessageDialog::builder()
        .transient_for(parent)
        .modal(true)
        .message_type(gtk::MessageType::Warning)
        .buttons(gtk::ButtonsType::YesNo)
        .text(title)
        .secondary_text(message)
        .build();
    dialog.set_default_response(ResponseType::No);
    let response = glib::MainContext::default().block_on(dialog.run_future());
    dialog.close();
    present_parent_window(parent);
    response == ResponseType::Yes
}

fn show_error_message(parent: &impl IsA<gtk::Window>, title: &str, message: &str) {
    show_message(parent, title, message, gtk::MessageType::Error);
}

fn show_warning_message(parent: &impl IsA<gtk::Window>, title: &str, message: &str) {
    show_message(parent, title, message, gtk::MessageType::Warning);
}

fn show_message(
    parent: &impl IsA<gtk::Window>,
    title: &str,
    message: &str,
    message_type: gtk::MessageType,
) {
    let dialog = gtk::MessageDialog::builder()
        .transient_for(parent)
        .modal(true)
        .message_type(message_type)
        .buttons(gtk::ButtonsType::Ok)
        .text(title)
        .secondary_text(message)
        .build();
    glib::MainContext::default().block_on(dialog.run_future());
    dialog.close();
    present_parent_window(parent);
}

fn present_parent_window(parent: &impl IsA<gtk::Window>) {
    let parent = parent.as_ref();
    if parent.root().is_some() && parent.is_visible() {
        parent.present();
    }
}

fn is_accessible_folder(path: &Path) -> bool {
    path.is_dir() && std::fs::read_dir(path).is_ok()
}

fn context_language(context: &GtkWindowContext) -> UiLanguage {
    current_ui_language(&context.state.settings().view)
}

fn context_window(context: &Rc<RefCell<GtkWindowContext>>) -> ApplicationWindow {
    context.borrow().window.clone()
}

fn tr(language: UiLanguage, korean: &'static str, english: &'static str) -> &'static str {
    language.text(korean, english)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quote_posix_shell_argument_preserves_single_argument_values() {
        assert_eq!(quote_posix_shell_argument(""), "''");
        assert_eq!(
            quote_posix_shell_argument("cargo-check_1.2:/tmp"),
            "cargo-check_1.2:/tmp"
        );
        assert_eq!(
            quote_posix_shell_argument("/tmp/space dir"),
            "'/tmp/space dir'"
        );
        assert_eq!(quote_posix_shell_argument("it's ready"), "'it'\\''s ready'");
    }

    #[test]
    fn linux_command_line_quotes_executable_and_keeps_prepared_arguments() {
        assert_eq!(
            command_line_from_executable_and_arguments(
                "/opt/j3 tools/run script",
                "--path '/tmp/space dir'"
            ),
            "'/opt/j3 tools/run script' --path '/tmp/space dir'"
        );
        assert_eq!(
            command_line_from_executable_and_arguments("git", "   "),
            "git"
        );
    }

    #[test]
    fn linux_argument_token_values_use_posix_shell_quoting() {
        assert_eq!(
            argument_token_execution_value(ExecutionType::ShellApi, "/tmp/space dir".to_owned()),
            "'/tmp/space dir'"
        );
        assert_eq!(
            argument_token_execution_value(ExecutionType::ExternalTerminal, "it's".to_owned()),
            "'it'\\''s'"
        );
    }

    #[test]
    fn linux_execution_values_reject_interior_nul() {
        assert!(reject_interior_nul(UiLanguage::English, "arguments", "--flag").is_ok());
        assert_eq!(
            reject_interior_nul(UiLanguage::English, "arguments", "--safe\0--evil"),
            Err("The execution value contains an unsupported character: arguments".to_owned())
        );
        assert_eq!(
            reject_interior_nul(UiLanguage::Korean, "arguments", "--safe\0--evil"),
            Err("실행 값에 사용할 수 없는 문자가 있습니다: arguments".to_owned())
        );
    }

    #[test]
    fn linux_execution_error_messages_use_requested_language() {
        assert_eq!(
            execute_shell_api_command(UiLanguage::Korean, "  ", "").unwrap_err(),
            "실행 대상을 입력하세요."
        );
        assert_eq!(
            execute_shell_api_command(UiLanguage::English, "  ", "").unwrap_err(),
            "Enter an executable."
        );
        assert_eq!(
            command_failed_message(UiLanguage::Korean, "spawn error"),
            "명령 실행 실패\n\nspawn error"
        );
        assert_eq!(
            command_failed_message(UiLanguage::English, "spawn error"),
            "Command failed\n\nspawn error"
        );
    }

    #[test]
    fn linux_terminal_error_messages_use_requested_language() {
        assert_eq!(
            terminal_launch_failed_message(UiLanguage::Korean, "xterm", "permission denied"),
            "터미널 실행 실패: xterm (permission denied)"
        );
        assert_eq!(
            terminal_launch_failed_message(UiLanguage::English, "xterm", "permission denied"),
            "Terminal launch failed: xterm (permission denied)"
        );
        assert_eq!(
            no_supported_terminal_message(UiLanguage::Korean),
            "지원되는 터미널 에뮬레이터를 찾을 수 없습니다."
        );
        assert_eq!(
            no_supported_terminal_message(UiLanguage::English),
            "No supported terminal emulator was found."
        );
    }

    #[test]
    fn gtk_workspace_drop_feedback_uses_windows_validity_rules() {
        let mut state = AppState::initial();
        state
            .add_category(Category::new("Backend").expect("category should be valid"))
            .expect("category should be added");
        state
            .add_category(Category::new("Frontend").expect("category should be valid"))
            .expect("category should be added");
        state
            .add_workspace(Workspace::new("/tmp/api", "api", "Rust").expect("workspace"))
            .expect("workspace should be added");
        state
            .add_workspace(Workspace::new("/tmp/web", "web", "Rust").expect("workspace"))
            .expect("workspace should be added");
        state
            .add_workspace(Workspace::new("/tmp/cli", "cli", "Rust").expect("workspace"))
            .expect("workspace should be added");
        state
            .move_workspace_to_category(2, 0)
            .expect("workspace should move into category");

        let settings = state.settings();

        assert_eq!(
            workspace_internal_drop_action(settings, 0, TreeRowRef::Workspace(1), false),
            None
        );
        assert_eq!(
            workspace_internal_drop_action(settings, 0, TreeRowRef::Workspace(1), true),
            Some(GtkWorkspaceDropAction::MoveRoot(3))
        );
        assert_eq!(
            workspace_internal_drop_action(settings, 0, TreeRowRef::Workspace(2), true),
            None
        );
        assert_eq!(
            workspace_internal_drop_action(settings, 2, TreeRowRef::Category(0), false),
            None
        );
        assert_eq!(
            workspace_internal_drop_action(settings, 2, TreeRowRef::Category(1), false),
            Some(GtkWorkspaceDropAction::ToCategory(1))
        );
    }

    #[test]
    fn gtk_command_drop_feedback_requires_active_valid_command_drag() {
        assert_eq!(command_internal_drop_destination(None, 1, 3), None);
        assert_eq!(command_internal_drop_destination(Some(1), 1, 3), None);
        assert_eq!(command_internal_drop_destination(Some(4), 1, 3), None);
        assert_eq!(command_internal_drop_destination(Some(0), 2, 3), Some(2));
    }

    #[test]
    fn terminal_candidates_prefer_terminal_environment_value() {
        let candidates = terminal_candidates_from_env(Some("wezterm".to_owned()));

        assert_eq!(candidates.first().map(String::as_str), Some("wezterm"));
        assert!(candidates.iter().any(|candidate| candidate == "xterm"));
    }

    #[test]
    fn terminal_candidates_ignore_empty_environment_value() {
        let candidates = terminal_candidates_from_env(Some("  ".to_owned()));

        assert_eq!(
            candidates.first().map(String::as_str),
            Some("x-terminal-emulator")
        );
    }

    #[test]
    fn terminal_command_spec_uses_gnome_terminal_working_directory_argument() {
        assert_eq!(
            terminal_command_spec("gnome-terminal", "/tmp/work space", "cargo test"),
            TerminalCommandSpec {
                program: "gnome-terminal".to_owned(),
                args: vec![
                    "--working-directory".to_owned(),
                    "/tmp/work space".to_owned(),
                    "--".to_owned(),
                    "sh".to_owned(),
                    "-lc".to_owned(),
                    "cargo test".to_owned(),
                ],
                current_dir: None,
            }
        );
    }

    #[test]
    fn terminal_command_spec_uses_konsole_workdir_argument() {
        assert_eq!(
            terminal_command_spec("/usr/bin/konsole", "/tmp/work", "cargo test"),
            TerminalCommandSpec {
                program: "/usr/bin/konsole".to_owned(),
                args: vec![
                    "--workdir".to_owned(),
                    "/tmp/work".to_owned(),
                    "-e".to_owned(),
                    "sh".to_owned(),
                    "-lc".to_owned(),
                    "cargo test".to_owned(),
                ],
                current_dir: None,
            }
        );
    }

    #[test]
    fn terminal_command_spec_quotes_xfce4_terminal_shell_command() {
        assert_eq!(
            terminal_command_spec("xfce4-terminal", "/tmp/work", "cargo test -- 'it works'"),
            TerminalCommandSpec {
                program: "xfce4-terminal".to_owned(),
                args: vec![
                    "--working-directory".to_owned(),
                    "/tmp/work".to_owned(),
                    "-e".to_owned(),
                    "sh -lc 'cargo test -- '\\''it works'\\'''".to_owned(),
                ],
                current_dir: None,
            }
        );
    }

    #[test]
    fn terminal_command_spec_uses_current_dir_for_generic_terminals() {
        assert_eq!(
            terminal_command_spec("xterm", "/tmp/work", "cargo test"),
            TerminalCommandSpec {
                program: "xterm".to_owned(),
                args: vec![
                    "-e".to_owned(),
                    "sh".to_owned(),
                    "-lc".to_owned(),
                    "cargo test".to_owned(),
                ],
                current_dir: Some("/tmp/work".to_owned()),
            }
        );
    }

    #[test]
    fn terminal_command_spec_splits_terminal_environment_arguments() {
        assert_eq!(
            terminal_command_spec("wezterm start", "/tmp/work", "cargo test"),
            TerminalCommandSpec {
                program: "wezterm".to_owned(),
                args: vec![
                    "start".to_owned(),
                    "-e".to_owned(),
                    "sh".to_owned(),
                    "-lc".to_owned(),
                    "cargo test".to_owned(),
                ],
                current_dir: Some("/tmp/work".to_owned()),
            }
        );
    }

    #[test]
    fn terminal_command_spec_keeps_quoted_launcher_path() {
        assert_eq!(
            terminal_command_spec("'my terminal' --new-window", "/tmp/work", "cargo test"),
            TerminalCommandSpec {
                program: "my terminal".to_owned(),
                args: vec![
                    "--new-window".to_owned(),
                    "-e".to_owned(),
                    "sh".to_owned(),
                    "-lc".to_owned(),
                    "cargo test".to_owned(),
                ],
                current_dir: Some("/tmp/work".to_owned()),
            }
        );
    }

    #[test]
    fn language_config_editor_accepts_commas_and_newlines() {
        assert_eq!(
            parse_language_config_editor_text(" Rust, TypeScript\n\nGo\r\n C++ "),
            vec![
                "Rust".to_owned(),
                "TypeScript".to_owned(),
                "Go".to_owned(),
                "C++".to_owned(),
            ]
        );
    }

    #[test]
    fn dialog_button_order_matches_windows_layouts() {
        assert_eq!(
            SAVE_CANCEL_DIALOG_BUTTONS,
            &[DialogButtonRole::Save, DialogButtonRole::Cancel]
        );
        assert_eq!(
            COMMAND_BUTTON_DIALOG_BUTTONS,
            &[
                DialogButtonRole::Save,
                DialogButtonRole::Cancel,
                DialogButtonRole::Apply
            ]
        );
        assert_eq!(
            FONT_DIALOG_BUTTONS,
            &[
                DialogButtonRole::Default,
                DialogButtonRole::Apply,
                DialogButtonRole::Cancel
            ]
        );
        assert_eq!(
            LANGUAGE_CONFIG_DIALOG_BUTTONS,
            &[
                DialogButtonRole::Default,
                DialogButtonRole::Save,
                DialogButtonRole::Cancel
            ]
        );
    }

    #[test]
    fn dialog_button_labels_and_responses_follow_windows_copy() {
        assert_eq!(
            dialog_button_label(DialogButtonRole::Save, UiLanguage::Korean),
            "저장"
        );
        assert_eq!(
            dialog_button_label(DialogButtonRole::Cancel, UiLanguage::English),
            "Cancel"
        );
        assert_eq!(
            dialog_button_response(DialogButtonRole::Save),
            ResponseType::Ok
        );
        assert_eq!(
            dialog_button_response(DialogButtonRole::Default),
            ResponseType::Other(1)
        );
        assert_eq!(
            dialog_default_response(SAVE_CANCEL_DIALOG_BUTTONS),
            Some(ResponseType::Ok)
        );
        assert_eq!(
            dialog_default_response(COMMAND_BUTTON_DIALOG_BUTTONS),
            Some(ResponseType::Ok)
        );
        assert_eq!(
            dialog_default_response(FONT_DIALOG_BUTTONS),
            Some(ResponseType::Apply)
        );
        assert_eq!(
            dialog_default_response(LANGUAGE_CONFIG_DIALOG_BUTTONS),
            Some(ResponseType::Ok)
        );
    }

    #[test]
    fn file_dialog_filter_specs_match_windows_dialogs() {
        assert_eq!(
            executable_file_filter_specs(UiLanguage::English),
            vec![
                FileDialogFilterSpec {
                    name: "Executable Files",
                    patterns: &["*.exe", "*.cmd", "*.bat", "*.ps1", "*.com"],
                },
                FileDialogFilterSpec {
                    name: "All Files",
                    patterns: &["*"],
                },
            ]
        );
        assert_eq!(
            executable_file_filter_specs(UiLanguage::Korean)[0].name,
            "실행 파일"
        );
        assert_eq!(
            all_files_filter_specs(UiLanguage::English),
            vec![FileDialogFilterSpec {
                name: "All Files",
                patterns: &["*"],
            }]
        );
    }

    #[test]
    fn file_dialog_result_to_path_selection_matches_windows_boundaries() {
        let local_path = PathBuf::from("/tmp/demo.txt");

        assert_eq!(
            file_dialog_result_to_path_selection(
                Ok(gio::File::for_path(&local_path)),
                UiLanguage::English,
                SelectionKind::File,
            ),
            PathSelection::Selected(local_path)
        );
        assert_eq!(
            file_dialog_result_to_path_selection(
                Err(glib::Error::new(gio::IOErrorEnum::Cancelled, "cancelled")),
                UiLanguage::English,
                SelectionKind::File,
            ),
            PathSelection::Canceled
        );
        assert_eq!(
            file_dialog_result_to_path_selection(
                Ok(gio::File::for_uri("sftp://example.invalid/tmp/demo.txt")),
                UiLanguage::English,
                SelectionKind::File,
            ),
            PathSelection::Failed("Select a local file path.".to_owned())
        );
        assert_eq!(
            file_dialog_result_to_path_selection(
                Ok(gio::File::for_uri("sftp://example.invalid/tmp/demo")),
                UiLanguage::Korean,
                SelectionKind::Folder,
            ),
            PathSelection::Failed("로컬 폴더 경로를 선택하세요.".to_owned())
        );

        let failed = file_dialog_result_to_path_selection(
            Err(glib::Error::new(gio::IOErrorEnum::Failed, "boom")),
            UiLanguage::English,
            SelectionKind::Folder,
        );
        match failed {
            PathSelection::Failed(message) => {
                assert!(message.starts_with("Selection dialog failed:"));
                assert!(message.contains("boom"));
            }
            other => panic!("expected failed selection, got {other:?}"),
        }
    }

    #[test]
    fn executable_file_dialog_initial_path_matches_windows_prefill_rules() {
        let dir = unique_temp_dir("gtk-file-dialog-initial");
        std::fs::create_dir_all(&dir).expect("temp dir should be created");
        let file = dir.join("run.sh");
        std::fs::write(&file, "").expect("temp file should be created");
        let missing = dir.join("missing.sh");

        assert_eq!(
            file_dialog_initial_path("  "),
            FileDialogInitialPath::default()
        );
        assert_eq!(
            file_dialog_initial_path(&file.display().to_string()),
            FileDialogInitialPath {
                file: Some(file.clone()),
                folder: None,
                name: None,
            }
        );
        assert_eq!(
            file_dialog_initial_path(&dir.display().to_string()),
            FileDialogInitialPath {
                file: None,
                folder: Some(dir.clone()),
                name: None,
            }
        );
        assert_eq!(
            file_dialog_initial_path(&missing.display().to_string()),
            FileDialogInitialPath {
                file: None,
                folder: Some(dir.clone()),
                name: Some("missing.sh".to_owned()),
            }
        );
        assert_eq!(
            file_dialog_initial_path("tool"),
            FileDialogInitialPath {
                file: None,
                folder: None,
                name: Some("tool".to_owned()),
            }
        );

        remove_temp_dir(&dir);
    }

    #[test]
    fn applied_view_settings_matches_windows_noop_and_preserves_window_layout() {
        let current = ViewSettings::new(DEFAULT_FONT_FAMILY, DEFAULT_FONT_SIZE, "graphite")
            .with_ui_language("en")
            .with_window_layout(Some(800), Some(600), Some(180));

        assert_eq!(applied_view_settings(&current, current.clone()), current);

        let next = ViewSettings::new("Custom Font", DEFAULT_FONT_SIZE + 2, "light")
            .with_ui_language("ko")
            .with_window_layout(Some(1), Some(2), Some(3));
        let applied = applied_view_settings(&current, next);

        assert_eq!(applied.font_family, "Custom Font");
        assert_eq!(applied.font_size, DEFAULT_FONT_SIZE + 2);
        assert_eq!(applied.theme, "light");
        assert_eq!(applied.ui_language, "ko");
        assert_eq!(applied.window_width, Some(800));
        assert_eq!(applied.window_height, Some(600));
        assert_eq!(applied.tree_panel_width, Some(180));
    }

    #[test]
    fn view_css_keeps_tree_selection_visible() {
        let view = ViewSettings::new(DEFAULT_FONT_FAMILY, DEFAULT_FONT_SIZE, "graphite");
        let css = view_css(&view, ThemePalette::for_theme(ViewTheme::Graphite));

        assert!(css.contains(".tree-row:selected, .command-row:selected"));
        assert!(css.contains(".tree-row:selected label, .command-row:selected label"));
        assert!(css.contains("background: #627894;"));
        assert!(css.contains("color: #ffffff;"));
    }

    #[test]
    fn settings_save_failure_message_uses_requested_language() {
        let error = settings::SettingsSaveError::Write {
            path: PathBuf::from("settings.toml"),
            source: std::io::Error::new(std::io::ErrorKind::PermissionDenied, "denied"),
        };

        let korean = settings_save_failure_message(UiLanguage::Korean, &error);
        assert!(korean.starts_with("설정 저장 실패\n\n"));
        assert!(korean.contains("denied"));

        let english = settings_save_failure_message(UiLanguage::English, &error);
        assert!(english.starts_with("Settings save failed\n\n"));
        assert!(english.contains("denied"));
    }

    #[test]
    fn workspace_tree_tooltip_matches_windows_copy() {
        let workspace =
            Workspace::new("/tmp/demo", "demo", "Rust").expect("workspace should be valid");

        assert_eq!(
            workspace_tree_tooltip_text(UiLanguage::Korean, &workspace),
            "폴더: /tmp/demo\n언어: Rust"
        );
        assert_eq!(
            workspace_tree_tooltip_text(UiLanguage::English, &workspace),
            "Folder: /tmp/demo\nLanguage: Rust"
        );
    }

    #[test]
    fn startup_font_validation_matches_windows_missing_font_warning() {
        let settings = crate::domain::AppSettings {
            view: ViewSettings::new("Missing Codex Font", DEFAULT_FONT_SIZE, "system")
                .with_ui_language("en"),
            ..crate::domain::AppSettings::default()
        };
        let mut state = AppState::from_settings(settings, Vec::new());

        let warnings = validate_startup_font_settings_with(&mut state, |font_family| {
            font_family.eq_ignore_ascii_case(DEFAULT_FONT_FAMILY)
        });

        assert_eq!(state.settings().view.font_family, DEFAULT_FONT_FAMILY);
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].contains("Font not found"));
        assert!(state.status_message().contains("Missing Codex Font"));
    }

    #[test]
    fn startup_font_validation_skips_warning_when_default_font_is_missing() {
        let settings = crate::domain::AppSettings {
            view: ViewSettings::new(DEFAULT_FONT_FAMILY, DEFAULT_FONT_SIZE, "system")
                .with_ui_language("en"),
            ..crate::domain::AppSettings::default()
        };
        let mut state = AppState::from_settings(settings, Vec::new());

        let warnings = validate_startup_font_settings_with(&mut state, |_| false);

        assert_eq!(state.settings().view.font_family, DEFAULT_FONT_FAMILY);
        assert!(warnings.is_empty());
        assert!(state.restore_warnings().is_empty());
    }

    #[test]
    fn font_family_normalization_matches_windows_dialog_list_rules() {
        assert_eq!(
            normalize_font_family_list(vec![
                "Zed".to_owned(),
                "alpha".to_owned(),
                "ALPHA".to_owned(),
                "Beta".to_owned(),
            ]),
            vec!["alpha", "Beta", "Zed"]
        );
        assert_eq!(
            normalize_font_family_list(Vec::new()),
            vec![DEFAULT_FONT_FAMILY.to_owned()]
        );
    }

    #[test]
    fn font_dialog_rejects_missing_selection_like_windows() {
        let fonts = vec!["Alpha".to_owned(), "Beta".to_owned()];

        assert_eq!(
            selected_font_dialog_family(&fonts, Some("beta".to_owned()), UiLanguage::English),
            Ok("beta".to_owned())
        );
        assert_eq!(
            selected_font_dialog_family(&fonts, None, UiLanguage::English),
            Err("Select a font from the list.".to_owned())
        );
        assert_eq!(
            selected_font_dialog_family(
                &fonts,
                Some(DEFAULT_FONT_FAMILY.to_owned()),
                UiLanguage::Korean
            ),
            Err("목록에서 글꼴을 선택하세요.".to_owned())
        );
    }

    #[test]
    fn font_dialog_options_keep_missing_default_selectable_on_linux() {
        let fonts = vec!["Alpha".to_owned(), "Beta".to_owned()];
        let options = font_dialog_family_options(&fonts, DEFAULT_FONT_FAMILY);

        assert_eq!(options, vec!["Alpha", "Beta", DEFAULT_FONT_FAMILY]);
        assert_eq!(
            selected_font_dialog_family(
                &options,
                Some(DEFAULT_FONT_FAMILY.to_owned()),
                UiLanguage::English
            ),
            Ok(DEFAULT_FONT_FAMILY.to_owned())
        );
        assert_eq!(font_dialog_family_options(&fonts, "Missing Font"), fonts);
    }

    #[test]
    fn command_button_update_needed_matches_windows_dialog_noop_rule() {
        let button = CommandButton::new("Build", "cargo", "build", ExecutionType::ExternalTerminal)
            .expect("button should be valid");
        let changed = CommandButton::new("Build", "cargo", "test", ExecutionType::ExternalTerminal)
            .expect("button should be valid");
        let mut state = AppState::initial();
        state
            .add_command_tab(
                CommandTab::new("Tools", vec![button.clone()]).expect("tab should be valid"),
            )
            .expect("tab should be added");

        assert_eq!(
            command_button_update_needed(&state, 0, 0, &button),
            Ok(false)
        );
        assert_eq!(
            command_button_update_needed(&state, 0, 0, &changed),
            Ok(true)
        );
    }

    #[test]
    fn theme_menu_uses_stateful_action_targets_without_label_prefixes() {
        let menu = theme_menu(UiLanguage::English);

        assert_eq!(menu.n_items(), ViewTheme::options().len() as i32);
        assert_eq!(
            menu_string_attribute(&menu, 0, "action").as_deref(),
            Some("app.theme")
        );
        assert_eq!(
            menu_string_attribute(&menu, 0, "target").as_deref(),
            Some("system")
        );
        assert_eq!(
            menu_string_attribute(&menu, 0, "label").as_deref(),
            Some("System")
        );
    }

    #[test]
    fn ui_language_menu_uses_stateful_action_targets_without_label_prefixes() {
        let menu = ui_language_menu(UiLanguage::Korean);

        assert_eq!(menu.n_items(), UiLanguage::options().len() as i32);
        assert_eq!(
            menu_string_attribute(&menu, 0, "action").as_deref(),
            Some("app.ui-language")
        );
        assert_eq!(
            menu_string_attribute(&menu, 0, "target").as_deref(),
            Some("ko")
        );
        assert_eq!(
            menu_string_attribute(&menu, 0, "label").as_deref(),
            Some("한국어")
        );
    }

    #[test]
    fn main_menu_model_matches_domain_menu_labels_in_both_languages() {
        for language in [UiLanguage::English, UiLanguage::Korean] {
            let model = main_menu_model(language);
            let model = model.upcast_ref::<gio::MenuModel>();
            let expected = crate::domain::main_menu_for_language(language);

            assert_eq!(model.n_items(), expected.len() as i32);
            for (index, expected_menu) in expected.iter().enumerate() {
                let index = index as i32;
                assert_eq!(
                    menu_model_string_attribute(model, index, "label").as_deref(),
                    Some(expected_menu.label)
                );
                let submenu = model
                    .item_link(index, "submenu")
                    .expect("top-level menu should have submenu");
                let labels = menu_model_labels(&submenu);
                let expected_labels = expected_menu
                    .items
                    .iter()
                    .map(|item| item.label.to_owned())
                    .collect::<Vec<_>>();
                assert_eq!(labels, expected_labels);
            }
        }
    }

    #[test]
    fn main_menu_model_wires_expected_action_names() {
        let model = main_menu_model(UiLanguage::English);
        let model = model.upcast_ref::<gio::MenuModel>();

        assert_eq!(
            submenu_action_names(model, 0),
            vec![
                Some("app.file-font".to_owned()),
                None,
                None,
                Some("app.file-workspace-languages".to_owned()),
                Some("app.file-about".to_owned()),
                Some("app.file-exit".to_owned()),
            ]
        );
        assert_eq!(
            submenu_action_names(model, 1),
            vec![
                Some("app.workspace-add".to_owned()),
                Some("app.workspace-add-category".to_owned()),
                Some("app.workspace-edit".to_owned()),
                Some("app.workspace-move-up".to_owned()),
                Some("app.workspace-move-down".to_owned()),
                Some("app.workspace-delete".to_owned()),
            ]
        );
        assert_eq!(
            submenu_action_names(model, 2),
            vec![
                Some("app.tab-add".to_owned()),
                Some("app.tab-rename".to_owned()),
                Some("app.tab-move-up".to_owned()),
                Some("app.tab-move-down".to_owned()),
                Some("app.tab-delete".to_owned()),
            ]
        );
        assert_eq!(
            submenu_action_names(model, 3),
            vec![
                Some("app.command-run".to_owned()),
                Some("app.command-add".to_owned()),
                Some("app.command-edit".to_owned()),
                Some("app.command-move-previous".to_owned()),
                Some("app.command-move-next".to_owned()),
                Some("app.command-delete".to_owned()),
            ]
        );
    }

    #[test]
    fn command_menu_state_matches_windows_context_rules() {
        assert_eq!(
            command_menu_state(true, true, Some(1), 3, false),
            CommandMenuState {
                can_execute: true,
                can_add: true,
                can_edit: true,
                can_delete: true,
                can_move_previous: true,
                can_move_next: true,
            }
        );
        assert_eq!(
            command_menu_state(true, true, Some(0), 3, true),
            CommandMenuState {
                can_execute: false,
                can_add: true,
                can_edit: true,
                can_delete: true,
                can_move_previous: false,
                can_move_next: true,
            }
        );
        assert_eq!(
            command_menu_state(true, false, None, 0, false),
            CommandMenuState {
                can_execute: false,
                can_add: true,
                can_edit: false,
                can_delete: false,
                can_move_previous: false,
                can_move_next: false,
            }
        );
    }

    #[test]
    fn tree_menu_selection_state_requires_existing_item_like_windows() {
        let settings = crate::domain::AppSettings {
            categories: vec![Category::new("Tools").expect("category should be valid")],
            workspaces: vec![
                Workspace::new("/tmp/demo", "demo", "Rust").expect("workspace should be valid"),
            ],
            ..crate::domain::AppSettings::default()
        };

        assert!(selected_tree_exists_in_settings(
            &settings,
            Some(TreeSelection::Workspace(0))
        ));
        assert!(selected_tree_exists_in_settings(
            &settings,
            Some(TreeSelection::Category(0))
        ));
        assert!(!selected_tree_exists_in_settings(
            &settings,
            Some(TreeSelection::Workspace(9))
        ));
        assert!(!selected_tree_exists_in_settings(
            &settings,
            Some(TreeSelection::Category(9))
        ));
    }

    #[test]
    fn tree_selection_missing_messages_match_windows_guard_copy() {
        let settings = crate::domain::AppSettings {
            categories: vec![Category::new("Tools").expect("category should be valid")],
            workspaces: vec![
                Workspace::new("/tmp/demo", "demo", "Rust").expect("workspace should be valid"),
            ],
            ..crate::domain::AppSettings::default()
        };

        assert_eq!(
            tree_selection_missing_message(
                &settings,
                Some(TreeSelection::Workspace(0)),
                UiLanguage::English
            ),
            None
        );
        assert_eq!(
            tree_selection_missing_message(
                &settings,
                Some(TreeSelection::Category(0)),
                UiLanguage::English
            ),
            None
        );
        assert_eq!(
            tree_selection_missing_message(
                &settings,
                Some(TreeSelection::Workspace(9)),
                UiLanguage::English
            ),
            Some("The selected workspace could not be found.")
        );
        assert_eq!(
            tree_selection_missing_message(
                &settings,
                Some(TreeSelection::Category(9)),
                UiLanguage::Korean
            ),
            Some("선택한 분류를 찾을 수 없습니다.")
        );
    }

    #[test]
    fn workspace_move_to_root_requires_visible_category_like_windows() {
        let settings = crate::domain::AppSettings {
            categories: vec![Category::new("Tools").expect("category should be valid")],
            workspaces: vec![
                Workspace::new("/tmp/tool", "tool", "Rust")
                    .expect("workspace should be valid")
                    .with_category(Some("tools".to_owned())),
                Workspace::new("/tmp/stale", "stale", "Rust")
                    .expect("workspace should be valid")
                    .with_category(Some("Missing".to_owned())),
                Workspace::new("/tmp/root", "root", "Rust").expect("workspace should be valid"),
            ],
            ..crate::domain::AppSettings::default()
        };

        assert!(workspace_selection_can_move_to_root(
            &settings,
            Some(TreeSelection::Workspace(0))
        ));
        assert!(!workspace_selection_can_move_to_root(
            &settings,
            Some(TreeSelection::Workspace(1))
        ));
        assert!(!workspace_selection_can_move_to_root(
            &settings,
            Some(TreeSelection::Workspace(2))
        ));
        assert!(!workspace_selection_can_move_to_root(
            &settings,
            Some(TreeSelection::Category(0))
        ));
        assert!(!workspace_selection_can_move_to_root(
            &settings,
            Some(TreeSelection::Workspace(9))
        ));
    }

    #[test]
    fn command_execution_blocks_only_existing_category_selection_like_windows() {
        let settings = crate::domain::AppSettings {
            categories: vec![Category::new("Tools").expect("category should be valid")],
            workspaces: vec![
                Workspace::new("/tmp/demo", "demo", "Rust").expect("workspace should be valid"),
            ],
            ..crate::domain::AppSettings::default()
        };

        assert!(selected_tree_blocks_command_execution(
            &settings,
            Some(TreeSelection::Category(0))
        ));
        assert!(!selected_tree_blocks_command_execution(
            &settings,
            Some(TreeSelection::Workspace(0))
        ));
        assert!(!selected_tree_blocks_command_execution(
            &settings,
            Some(TreeSelection::Category(9))
        ));
        assert!(!selected_tree_blocks_command_execution(&settings, None));
    }

    #[test]
    fn command_tab_selector_sync_matches_windows_selection_rules() {
        assert_eq!(
            command_tab_selector_sync_state(3, Some(1)),
            CommandTabSelectorSyncState {
                active_index: Some(1),
                sensitive: true,
                domain_selection: Some(1),
            }
        );
        assert_eq!(
            command_tab_selector_sync_state(3, Some(9)),
            CommandTabSelectorSyncState {
                active_index: None,
                sensitive: true,
                domain_selection: None,
            }
        );
        assert_eq!(
            command_tab_selector_sync_state(0, Some(0)),
            CommandTabSelectorSyncState {
                active_index: None,
                sensitive: false,
                domain_selection: None,
            }
        );
    }

    #[test]
    fn context_menu_key_matches_windows_keyboard_entry_points() {
        assert!(is_context_menu_key(
            gdk::Key::Menu,
            gdk::ModifierType::empty()
        ));
        assert!(is_context_menu_key(
            gdk::Key::F10,
            gdk::ModifierType::SHIFT_MASK
        ));
        assert!(!is_context_menu_key(
            gdk::Key::F10,
            gdk::ModifierType::empty()
        ));
    }

    #[test]
    fn text_input_value_handling_matches_windows_argument_dialog_rules() {
        assert_eq!(
            accepted_text_input_value("  category  ", false),
            Some("category".to_owned())
        );
        assert_eq!(accepted_text_input_value("   ", false), None);
        assert_eq!(
            accepted_text_input_value("  keep spaces  ", true),
            Some("  keep spaces  ".to_owned())
        );
        assert_eq!(accepted_text_input_value("", true), Some(String::new()));
    }

    #[test]
    fn workspace_dialog_replaces_name_only_while_it_is_folder_default() {
        assert!(should_replace_workspace_default_name("", None));
        assert!(should_replace_workspace_default_name(
            "first",
            Some("first")
        ));
        assert!(!should_replace_workspace_default_name(
            "custom",
            Some("first")
        ));
        assert!(!should_replace_workspace_default_name("custom", None));
    }

    #[test]
    fn workspace_dialog_browse_update_matches_windows_add_rules() {
        let dir = unique_temp_dir("browse-add-rust");
        std::fs::create_dir_all(&dir).expect("temp workspace should be created");
        std::fs::write(dir.join("Cargo.toml"), "[package]\nname = \"demo\"\n")
            .expect("project marker should be written");
        let options = vec!["rust".to_owned(), "Other".to_owned()];

        let update = workspace_folder_browse_update(WorkspaceDialogMode::Add, &dir, "", None);

        assert_eq!(update.path_text, dir.display().to_string());
        assert_eq!(
            update.replacement_name,
            Some(default_workspace_name_for_path(&dir))
        );
        assert_eq!(
            update.previous_folder_default_name,
            Some(default_workspace_name_for_path(&dir))
        );
        assert_eq!(update.language_inference_path, Some(dir.clone()));
        assert_eq!(
            workspace_language_option_from_folder(&dir, &options),
            Some("rust".to_owned())
        );

        remove_temp_dir(&dir);
    }

    #[test]
    fn workspace_dialog_browse_update_matches_windows_edit_rules() {
        let dir = unique_temp_dir("browse-edit-rust");
        std::fs::create_dir_all(&dir).expect("temp workspace should be created");
        std::fs::write(dir.join("Cargo.toml"), "[package]\nname = \"demo\"\n")
            .expect("project marker should be written");

        let update =
            workspace_folder_browse_update(WorkspaceDialogMode::Edit, &dir, "Custom Name", None);

        assert_eq!(update.path_text, dir.display().to_string());
        assert_eq!(update.replacement_name, None);
        assert_eq!(update.previous_folder_default_name, None);
        assert_eq!(update.language_inference_path, None);

        remove_temp_dir(&dir);
    }

    #[test]
    fn command_button_dialog_required_field_messages_match_windows_labels() {
        assert_eq!(
            command_button_required_field_message("", "tool", UiLanguage::Korean),
            Some("이름을 입력하세요.")
        );
        assert_eq!(
            command_button_required_field_message("Run", "", UiLanguage::English),
            Some("Enter an executable target.")
        );
        assert_eq!(
            command_button_required_field_message("Run", "tool", UiLanguage::English),
            None
        );
    }

    #[test]
    fn command_button_dialog_unknown_token_message_matches_windows_domain_message() {
        assert_eq!(
            command_button_unknown_token_message("--lang {language}", UiLanguage::English),
            Some("Unknown token.\n\n{language}".to_owned())
        );
        assert_eq!(
            command_button_unknown_token_message("{bad} {also_bad}", UiLanguage::Korean),
            Some("알 수 없는 토큰입니다.\n\n{bad}, {also_bad}".to_owned())
        );
        assert_eq!(
            command_button_unknown_token_message("{Language}", UiLanguage::English),
            None
        );
    }

    #[test]
    fn command_button_dialog_argument_tokens_use_windows_grid_layout() {
        let positions = (0..ARGUMENT_TOKENS.len())
            .map(argument_token_grid_position)
            .collect::<Vec<_>>();

        assert_eq!(
            positions,
            vec![(0, 0), (1, 0), (2, 0), (0, 1), (1, 1), (2, 1)]
        );
        assert_eq!(ARGUMENT_TOKEN_COLUMNS, 3);
        assert_eq!(ARGUMENT_TOKEN_BUTTON_WIDTH, 156);
        assert_eq!(ARGUMENT_TOKEN_BUTTON_HEIGHT, 28);
    }

    #[test]
    fn command_run_guard_message_matches_windows_copy() {
        assert_eq!(
            command_run_selection_required_message(UiLanguage::Korean),
            "실행할 명령을 선택하세요."
        );
        assert_eq!(
            command_run_selection_required_message(UiLanguage::English),
            "Select a command to run."
        );
    }

    #[test]
    fn command_button_guard_messages_match_windows_copy() {
        assert_eq!(
            command_add_group_required_message(UiLanguage::English),
            "Select a command group first."
        );
        assert_eq!(
            command_edit_group_required_message(UiLanguage::Korean),
            "명령 그룹을 선택하세요."
        );
        assert_eq!(
            command_edit_selection_required_message(UiLanguage::English),
            "Select a command to edit."
        );
        assert_eq!(
            command_delete_group_required_message(UiLanguage::English),
            "Select a command group."
        );
        assert_eq!(
            command_delete_selection_required_message(UiLanguage::Korean),
            "삭제할 명령을 선택하세요."
        );
        assert_eq!(
            command_move_group_required_message(UiLanguage::English),
            "Select a command group."
        );
        assert_eq!(
            command_move_selection_required_message(UiLanguage::Korean),
            "이동할 명령을 선택하세요."
        );
        assert_eq!(
            selected_command_missing_message(UiLanguage::English),
            "The selected command could not be found."
        );
    }

    #[test]
    fn command_group_guard_messages_match_windows_copy() {
        assert_eq!(
            command_group_rename_selection_required_message(UiLanguage::English),
            "Select a group to rename."
        );
        assert_eq!(
            command_group_delete_selection_required_message(UiLanguage::Korean),
            "삭제할 명령 그룹을 선택하세요."
        );
        assert_eq!(
            command_group_move_selection_required_message(UiLanguage::English),
            "Select a command group to move."
        );
        assert_eq!(
            selected_command_group_missing_message(UiLanguage::English),
            "The selected command group could not be found."
        );
        assert_eq!(
            cannot_move_further_message(UiLanguage::Korean),
            "더 이동할 수 없습니다."
        );
    }

    #[test]
    fn workspace_tree_guard_messages_match_windows_copy() {
        assert_eq!(
            tree_edit_selection_required_message(UiLanguage::Korean),
            "편집할 항목을 선택하세요."
        );
        assert_eq!(
            tree_delete_selection_required_message(UiLanguage::English),
            "Select an item to delete."
        );
        assert_eq!(
            tree_move_selection_required_message(UiLanguage::English),
            "Select an item to move."
        );
        assert_eq!(
            selected_workspace_missing_message(UiLanguage::English),
            "The selected workspace could not be found."
        );
        assert_eq!(
            selected_category_missing_message(UiLanguage::Korean),
            "선택한 분류를 찾을 수 없습니다."
        );
    }

    #[test]
    fn workspace_language_apply_error_title_matches_windows_copy() {
        assert_eq!(
            workspace_language_apply_error_title(UiLanguage::Korean),
            "언어"
        );
        assert_eq!(
            workspace_language_apply_error_title(UiLanguage::English),
            "Workspace Languages"
        );
    }

    #[test]
    fn about_message_matches_windows_copy() {
        assert_eq!(about_message("j3DevHelper", "1.2.3"), "j3DevHelper  v1.2.3");
        assert_eq!(
            about_link_markup(APP_REPOSITORY_URL),
            "<a href=\"https://github.com/edgarp9\">https://github.com/edgarp9</a>"
        );
        assert_eq!(about_dialog_close_label(UiLanguage::Korean), "닫기");
        assert_eq!(about_dialog_close_label(UiLanguage::English), "Close");
        assert_eq!((ABOUT_DIALOG_WIDTH, ABOUT_DIALOG_HEIGHT), (460, 300));
        assert!(about_license_notice(UiLanguage::English).contains("GPL-3.0"));
    }

    #[test]
    fn command_button_metrics_follow_windows_font_scaling_at_default_dpi() {
        assert_eq!(
            command_button_preferred_width_for_font(DEFAULT_FONT_SIZE),
            COMMAND_BUTTON_PREFERRED_WIDTH
        );
        assert_eq!(
            command_button_preferred_width_for_font(DEFAULT_FONT_SIZE + 4),
            COMMAND_BUTTON_PREFERRED_WIDTH + 48
        );
        assert_eq!(
            command_button_gap_for_font(DEFAULT_FONT_SIZE),
            COMMAND_BUTTON_BASE_GAP
        );
        assert_eq!(
            command_button_gap_for_font(DEFAULT_FONT_SIZE + 10),
            COMMAND_BUTTON_BASE_GAP + 4
        );
        assert_eq!(command_button_height_for_font(DEFAULT_FONT_SIZE), 30);
        assert_eq!(command_button_height_for_font(DEFAULT_FONT_SIZE + 4), 38);
        assert_eq!(COMMAND_BUTTON_HORIZONTAL_PADDING, 10);
        assert_eq!(COMMAND_BUTTON_TOP_PADDING, 4);
    }

    #[test]
    fn tree_panel_width_clamps_to_keep_command_area_visible() {
        let layout = LayoutSpec::for_font_size(DEFAULT_FONT_SIZE);

        assert_eq!(clamp_tree_panel_width_for_window(layout, 484, 10), 72);
        assert_eq!(clamp_tree_panel_width_for_window(layout, 484, 160), 160);
        assert_eq!(clamp_tree_panel_width_for_window(layout, 484, 600), 342);
        assert_eq!(clamp_tree_panel_width_for_window(layout, 100, 600), 72);
    }

    #[test]
    fn startup_window_size_keeps_tree_and_command_area_visible() {
        let layout = LayoutSpec::for_font_size(DEFAULT_FONT_SIZE);
        let initial = WindowSize {
            width: 484,
            height: 613,
        };
        let tiny_saved = ViewSettings::new(DEFAULT_FONT_FAMILY, DEFAULT_FONT_SIZE, "graphite")
            .with_window_layout(Some(10), Some(10), Some(600));

        assert_eq!(
            startup_window_size(initial, layout, &tiny_saved),
            WindowSize {
                width: 214,
                height: 174,
            }
        );
        assert_eq!(
            startup_window_size(initial, layout, &ViewSettings::default()),
            initial
        );
    }

    #[test]
    fn workspace_drop_reject_messages_match_windows_copy() {
        assert_eq!(
            workspace_drop_reject_message(UiLanguage::English, WorkspaceDropRejectReason::Empty),
            "Drop one folder."
        );
        assert_eq!(
            workspace_drop_reject_message(
                UiLanguage::Korean,
                WorkspaceDropRejectReason::MultipleItems(3)
            ),
            "폴더 하나만 드롭하세요. (3개 선택됨)"
        );
        assert_eq!(
            workspace_drop_reject_message(
                UiLanguage::English,
                WorkspaceDropRejectReason::DuplicatePath {
                    path: PathBuf::from("/tmp/demo"),
                    name: "Demo".to_owned(),
                },
            ),
            "This folder is already registered.\n\nName: Demo\nFolder: /tmp/demo"
        );
    }

    fn menu_string_attribute(menu: &gio::Menu, index: i32, attribute: &str) -> Option<String> {
        menu_model_string_attribute(menu.upcast_ref::<gio::MenuModel>(), index, attribute)
    }

    fn menu_model_string_attribute(
        menu: &gio::MenuModel,
        index: i32,
        attribute: &str,
    ) -> Option<String> {
        menu.item_attribute_value(index, attribute, Some(glib::VariantTy::STRING))
            .and_then(|value| value.get::<String>())
    }

    fn menu_model_labels(menu: &gio::MenuModel) -> Vec<String> {
        (0..menu.n_items())
            .filter_map(|index| menu_model_string_attribute(menu, index, "label"))
            .collect()
    }

    fn submenu_action_names(menu: &gio::MenuModel, index: i32) -> Vec<Option<String>> {
        let submenu = menu
            .item_link(index, "submenu")
            .expect("top-level menu should have submenu");
        (0..submenu.n_items())
            .map(|index| menu_model_string_attribute(&submenu, index, "action"))
            .collect()
    }

    fn unique_temp_dir(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "j3devhelper-{label}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be after epoch")
                .as_nanos()
        ))
    }

    fn remove_temp_dir(path: &Path) {
        if let Err(error) = std::fs::remove_dir_all(path)
            && error.kind() != std::io::ErrorKind::NotFound
        {
            panic!("failed to remove temp dir {}: {error}", path.display());
        }
    }
}
