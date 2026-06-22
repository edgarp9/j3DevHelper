mod arguments;
mod core;
mod localization;
mod navigation;
mod state;
mod ui;

pub use self::arguments::{
    ARGUMENT_TOKENS, ArgumentReplacementError, ArgumentResolutionError, ArgumentToken,
    argument_tokens_in_first_appearance_order, arguments_require_workspace,
    interactive_argument_tokens_in_first_appearance_order, replace_argument_tokens,
    resolve_argument_replacements, unknown_argument_tokens,
};
pub use self::core::{
    AppSettings, Category, CommandButton, CommandTab, DEFAULT_FONT_FAMILY, DEFAULT_FONT_SIZE,
    DEFAULT_THEME, DEFAULT_WORKSPACE_LANGUAGE, DomainValidationError, ExecutionType,
    MAX_UI_FONT_SIZE, MIN_UI_FONT_SIZE, TreeRootItem, TreeRootItemRef, UI_FONT_SIZE_OPTIONS,
    ViewSettings, ViewTheme, WORKSPACE_LANGUAGE_OPTIONS, Workspace, category_names_equal,
    current_ui_language, default_workspace_language_for_options,
    default_workspace_language_options, default_workspace_name_for_path,
    direct_reorder_drop_destination, indexed_reorder_drop_destination,
    infer_workspace_language_from_entry_names, is_supported_ui_font_size,
    is_supported_workspace_language, is_supported_workspace_language_in, normalize_ui_font_size,
    normalize_ui_language, normalize_workspace_language, normalize_workspace_language_options,
    workspace_paths_equal,
};
pub use self::localization::{DEFAULT_UI_LANGUAGE, UiLanguage};
pub use self::navigation::{
    CommandButtonMoveDirection, CommandTabMoveDirection, TreeKeyboardMoveDirection,
    WorkspaceTreeDropAction, WorkspaceTreeDropTarget, command_button_drop_destination,
    command_button_move_destination, command_tab_move_destination, tree_root_drop_destination,
    tree_root_keyboard_move_destination, workspace_belongs_to_category, workspace_category_index,
    workspace_keyboard_move_destination, workspace_tree_drop_action,
    workspace_tree_drop_destination, workspace_tree_visible_group_drop_destination,
};
pub use self::state::{
    AppState, CategoryMutationError, CommandButtonMutationError, CommandTabMutationError,
    INITIAL_STATUS_MESSAGE, INITIAL_STATUS_MESSAGE_EN, LanguageConfigMutationError,
    TreeRootMutationError, WorkspaceMutationError,
};
pub use self::ui::{
    ABOUT_FILE_NAME, APP_COPYRIGHT_NOTICE, APP_ICON_PNG_FILE_NAME, APP_ICON_SVG_FILE_NAME,
    APP_LICENSE_NOTICE, APP_LINUX_APPLICATION_ID, APP_LINUX_DESKTOP_ENTRY_NAME, APP_REPOSITORY_URL,
    APP_TITLE, APP_VERSION, ClientSize, DEFAULT_DPI, LayoutSpec, MainContentLayout, MainWindowSpec,
    MenuDefinition, MenuItemDefinition, PROJECT_LICENSE_FILE_NAME, RectSpec,
    THIRD_PARTY_LICENSE_NOTICE, THIRD_PARTY_LINUX_NATIVE_LICENSE_NOTICE,
    THIRD_PARTY_NOTICE_FILE_NAME, THIRD_PARTY_RESOURCE_LICENSE_NOTICE,
    THIRD_PARTY_RUST_LICENSE_NOTICE, WindowSize, about_license_heading, about_license_notice,
    default_about_text, main_menu_for_language, scale_dimension_for_dpi,
};

#[cfg(test)]
mod tests;
