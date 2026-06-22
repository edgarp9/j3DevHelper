use super::*;

#[test]
fn layout_scales_with_dpi() {
    let base = LayoutSpec::for_font_size(DEFAULT_FONT_SIZE);
    let scaled = LayoutSpec::for_font_size_and_dpi(DEFAULT_FONT_SIZE, 144);

    assert_eq!(scaled.content_margin, 12);
    assert_eq!(scaled.tree_panel_width, base.tree_panel_width * 3 / 2);
    assert_eq!(scale_dimension_for_dpi(484, 144), 726);
}

#[test]
fn infers_workspace_language_from_project_markers() {
    assert_eq!(
        infer_workspace_language_from_entry_names(["Cargo.toml", "src"]),
        Some("Rust")
    );
    assert_eq!(
        infer_workspace_language_from_entry_names(["package.json", "tsconfig.json"]),
        Some("TypeScript")
    );
    assert_eq!(
        infer_workspace_language_from_entry_names(["pyproject.toml"]),
        Some("Python")
    );
    assert_eq!(
        infer_workspace_language_from_entry_names(["main.c"]),
        Some("C")
    );
}

#[test]
fn returns_none_when_language_cannot_be_inferred() {
    assert_eq!(
        infer_workspace_language_from_entry_names(["README.md", "docs"]),
        None
    );
}

#[test]
fn workspace_language_must_be_supported_option() {
    let error = Workspace::new(r"C:\dev\app", "app", "Haskell")
        .expect_err("unsupported language should be rejected");

    assert_eq!(
        error,
        DomainValidationError::UnsupportedWorkspaceLanguage("Haskell".to_owned())
    );
}

#[test]
fn default_workspace_language_is_non_committal() {
    assert_eq!(DEFAULT_WORKSPACE_LANGUAGE, "Other");
    assert!(is_supported_workspace_language(DEFAULT_WORKSPACE_LANGUAGE));
}

#[test]
fn workspace_language_config_normalizes_case_and_duplicates() {
    let languages = normalize_workspace_language_options(["rust", "c", "java"])
        .expect("language config should be valid");

    let workspace = Workspace::new_with_language_options(r"C:\dev\app", "app", "RUST", &languages)
        .expect("configured language should be accepted case-insensitively");

    assert_eq!(workspace.language, "rust");
    assert_eq!(
        normalize_workspace_language_options(["rust", "Rust"]),
        Err(DomainValidationError::DuplicateWorkspaceLanguage(
            "Rust".to_owned()
        ))
    );
}

#[test]
fn view_theme_options_preserve_system_and_j3_tree_text_palettes() {
    let values = ViewTheme::options()
        .iter()
        .map(|theme| theme.as_config_value())
        .collect::<Vec<_>>();

    assert_eq!(
        values,
        vec![
            "system",
            "light",
            "classic-dark",
            "sepia-teal",
            "graphite",
            "forest",
            "steel-blue",
        ]
    );
}

#[test]
fn view_theme_defaults_to_graphite() {
    assert_eq!(DEFAULT_THEME, "graphite");
    assert_eq!(ViewTheme::default(), ViewTheme::Graphite);
    assert_eq!(ViewSettings::default().theme, "graphite");
    assert_eq!(
        ViewSettings::new("Segoe UI", DEFAULT_FONT_SIZE, "default").theme,
        "graphite"
    );
}

#[test]
fn view_settings_normalizes_theme_storage_values() {
    assert_eq!(
        ViewSettings::new("Segoe UI", DEFAULT_FONT_SIZE, "dark").theme,
        "classic-dark"
    );
    assert_eq!(
        ViewSettings::new("Segoe UI", DEFAULT_FONT_SIZE, "unknown").theme,
        DEFAULT_THEME
    );
}

#[test]
fn view_settings_normalizes_ui_language_storage_values() {
    assert_eq!(DEFAULT_UI_LANGUAGE, "en");
    assert_eq!(
        ViewSettings::new("Segoe UI", DEFAULT_FONT_SIZE, "graphite")
            .with_ui_language("english")
            .ui_language,
        "en"
    );
    assert_eq!(
        ViewSettings::new("Segoe UI", DEFAULT_FONT_SIZE, "graphite")
            .with_ui_language("unknown")
            .ui_language,
        DEFAULT_UI_LANGUAGE
    );
}

#[test]
fn user_messages_follow_ui_language() {
    assert_eq!(
        DomainValidationError::MissingWorkspaceField("path")
            .user_message_for_language(UiLanguage::English),
        "Enter a workspace value: path"
    );
    assert_eq!(
        DomainValidationError::MissingWorkspaceField("path")
            .user_message_for_language(UiLanguage::Korean),
        "워크스페이스 값을 입력하세요: path"
    );
    assert_eq!(
        WorkspaceMutationError::DuplicatePath(r"C:\dev\app".to_owned())
            .user_message_for_language(UiLanguage::English),
        r"This folder is already registered: C:\dev\app"
    );
    assert_eq!(
        ArgumentResolutionError::WorkspaceRequired.user_message_for_language(UiLanguage::English),
        "Select a workspace."
    );
}

#[test]
fn app_state_starts_ready_without_restore_warnings() {
    let state = AppState::initial();

    assert_eq!(state.status_message(), "Ready");
}

#[test]
fn main_menu_groups_file_level_actions() {
    let spec = MainWindowSpec::initial(AppState::initial());
    let file_menu = spec
        .menus
        .iter()
        .find(|menu| menu.label == "File")
        .expect("file menu should exist");
    let labels = file_menu
        .items
        .iter()
        .map(|item| item.label)
        .collect::<Vec<_>>();

    assert_eq!(
        labels,
        vec![
            "Font",
            "Theme",
            "UI Language",
            "Workspace Languages",
            "About",
            "Exit",
        ]
    );
    assert!(file_menu.items.iter().all(|item| item.enabled));
}

#[test]
fn main_menu_uses_korean_labels_when_ui_language_is_korean() {
    let settings = AppSettings {
        view: ViewSettings::default().with_ui_language("ko"),
        ..AppSettings::default()
    };
    let spec = MainWindowSpec::initial(AppState::from_settings(settings, Vec::new()));
    let labels = spec.menus.iter().map(|menu| menu.label).collect::<Vec<_>>();
    let file_labels = spec.menus[0]
        .items
        .iter()
        .map(|item| item.label)
        .collect::<Vec<_>>();

    assert_eq!(labels, vec!["파일", "워크스페이스", "명령 그룹", "명령"]);
    assert_eq!(
        file_labels,
        vec![
            "글꼴",
            "테마",
            "UI 언어",
            "워크스페이스 언어",
            "정보",
            "종료",
        ]
    );
}

#[test]
fn tree_menu_uses_directional_move_actions() {
    let spec = MainWindowSpec::initial(AppState::initial());
    let tree_menu = spec
        .menus
        .iter()
        .find(|menu| menu.label == "Workspace")
        .expect("workspace menu should exist");
    let labels = tree_menu
        .items
        .iter()
        .map(|item| item.label)
        .collect::<Vec<_>>();

    assert_eq!(
        labels,
        vec![
            "Add",
            "Add Category",
            "Edit",
            "Move Up",
            "Move Down",
            "Delete"
        ]
    );
    assert!(!labels.contains(&"Move"));
}

#[test]
fn command_group_menu_uses_vertical_move_actions() {
    let spec = MainWindowSpec::initial(AppState::initial());
    let tabs_menu = spec
        .menus
        .iter()
        .find(|menu| menu.label == "Command Group")
        .expect("command group menu should exist");
    let labels = tabs_menu
        .items
        .iter()
        .map(|item| item.label)
        .collect::<Vec<_>>();

    assert_eq!(
        labels,
        vec!["Add", "Rename", "Move Up", "Move Down", "Delete"]
    );
    assert!(!labels.contains(&"Move"));
}

#[test]
fn commands_menu_uses_previous_next_move_actions() {
    let spec = MainWindowSpec::initial(AppState::initial());
    let commands_menu = spec
        .menus
        .iter()
        .find(|menu| menu.label == "Command")
        .expect("commands menu should exist");
    let labels = commands_menu
        .items
        .iter()
        .map(|item| item.label)
        .collect::<Vec<_>>();

    assert_eq!(
        labels,
        vec!["Run", "Add", "Edit", "Previous", "Next", "Delete"]
    );
    assert!(!labels.contains(&"Move"));
}

#[test]
fn main_menu_omits_view_and_help_menus() {
    let spec = MainWindowSpec::initial(AppState::initial());
    let labels = spec.menus.iter().map(|menu| menu.label).collect::<Vec<_>>();

    assert_eq!(
        labels,
        vec!["File", "Workspace", "Command Group", "Command"]
    );
    assert_eq!(APP_VERSION, env!("CARGO_PKG_VERSION"));
    assert_eq!(APP_REPOSITORY_URL, "https://github.com/edgarp9");
    assert_eq!(APP_LICENSE_NOTICE, "GPL-3.0-or-later");
    assert!(APP_COPYRIGHT_NOTICE.contains("edgarp9"));
    assert_eq!(ABOUT_FILE_NAME, "about.txt");
    assert_eq!(PROJECT_LICENSE_FILE_NAME, "LICENSE");
    assert_eq!(THIRD_PARTY_NOTICE_FILE_NAME, "THIRD_PARTY_NOTICES.txt");
    assert!(THIRD_PARTY_LICENSE_NOTICE.contains("MIT"));
    assert!(THIRD_PARTY_RUST_LICENSE_NOTICE.contains("Unicode-3.0"));
    assert!(THIRD_PARTY_RESOURCE_LICENSE_NOTICE.contains("Material Symbols"));
    assert!(THIRD_PARTY_RESOURCE_LICENSE_NOTICE.contains("SIL Open Font License"));
    assert!(THIRD_PARTY_RESOURCE_LICENSE_NOTICE.contains("OFL-1.1"));
    assert!(THIRD_PARTY_LINUX_NATIVE_LICENSE_NOTICE.contains("LGPL-2.1-or-later"));
    assert!(THIRD_PARTY_LINUX_NATIVE_LICENSE_NOTICE.contains("Cairo: LGPL-2.1-only OR MPL-1.1"));
    assert!(
        about_license_notice(UiLanguage::English).contains(&format!("j3DevHelper  v{APP_VERSION}"))
    );
    assert!(
        about_license_notice(UiLanguage::English)
            .contains("j3DevHelper is distributed under GPL-3.0-or-later.")
    );
    assert!(about_license_notice(UiLanguage::English).contains("No warranty"));
    assert!(about_license_notice(UiLanguage::English).contains("LICENSE"));
    assert!(about_license_notice(UiLanguage::English).contains(APP_COPYRIGHT_NOTICE));
    assert!(about_license_notice(UiLanguage::English).contains("Corresponding Source"));
    assert!(about_license_notice(UiLanguage::English).contains("j3devhelper-v0.2.0-source.zip"));
    assert!(about_license_notice(UiLanguage::English).contains("THIRD_PARTY_NOTICES.txt"));
    assert!(about_license_notice(UiLanguage::English).contains("Material Symbols"));
    assert!(about_license_notice(UiLanguage::English).contains("LGPL-2.1-only OR MPL-1.1"));
    assert_eq!(
        about_license_notice(UiLanguage::Korean),
        about_license_notice(UiLanguage::English)
    );
    assert_eq!(about_license_heading(UiLanguage::English), "Licenses");
}

#[test]
fn command_tabs_start_empty_and_can_be_managed() {
    let mut state = AppState::initial();

    assert!(state.settings().command_tabs.is_empty());
    assert_eq!(state.selected_command_tab_index(), None);

    let first = state
        .add_command_tab(CommandTab::new("first", Vec::new()).expect("tab should be valid"))
        .expect("tab should be added");
    let second = state
        .add_command_tab(CommandTab::new("second", Vec::new()).expect("tab should be valid"))
        .expect("tab should be added");

    assert_eq!(first, 0);
    assert_eq!(second, 1);
    assert_eq!(
        state.selected_command_tab().map(|tab| tab.name.as_str()),
        Some("second")
    );

    state
        .rename_command_tab(1, "renamed")
        .expect("tab should be renamed");
    state.move_command_tab(1, 0).expect("tab should be moved");

    assert_eq!(state.settings().command_tabs[0].name, "renamed");
    assert_eq!(state.selected_command_tab_index(), Some(0));

    let removed = state.delete_command_tab(0).expect("tab should be deleted");
    assert_eq!(removed.name, "renamed");
    assert_eq!(state.selected_command_tab_index(), Some(0));
    assert_eq!(state.settings().command_tabs[0].name, "first");
}

#[test]
fn command_buttons_can_be_managed_inside_selected_tab() {
    let mut state = AppState::initial();
    let tab_index = state
        .add_command_tab(CommandTab::new("common", Vec::new()).expect("tab should be valid"))
        .expect("tab should be added");

    let first = state
        .add_command_button(
            tab_index,
            CommandButton::new(
                "Cargo Check",
                "cargo",
                "check",
                ExecutionType::ExternalTerminal,
            )
            .expect("button should be valid"),
        )
        .expect("button should be added");
    let second = state
        .add_command_button(
            tab_index,
            CommandButton::new("Open", "explorer.exe", "{path}", ExecutionType::ShellApi)
                .expect("button should be valid"),
        )
        .expect("button should be added");

    assert_eq!(first, 0);
    assert_eq!(second, 1);
    assert_eq!(state.selected_command_button_index(), Some(1));

    state
        .move_command_button(tab_index, 1, 0)
        .expect("button should be moved");
    assert_eq!(
        state.settings().command_tabs[0].buttons[0].button_name,
        "Open"
    );
    assert_eq!(state.selected_command_button_index(), Some(0));

    state
        .update_command_button(
            tab_index,
            0,
            CommandButton::new(
                "Open Workspace",
                "explorer.exe",
                "{path}",
                ExecutionType::ShellApi,
            )
            .expect("button should be valid"),
        )
        .expect("button should be updated");
    assert_eq!(
        state
            .selected_command_button()
            .map(|button| button.button_name.as_str()),
        Some("Open Workspace")
    );

    let removed = state
        .delete_command_button(tab_index, 0)
        .expect("button should be deleted");
    assert_eq!(removed.button_name, "Open Workspace");
    assert_eq!(state.selected_command_button_index(), Some(0));
    assert_eq!(
        state.settings().command_tabs[0].buttons[0].button_name,
        "Cargo Check"
    );
}

#[test]
fn unknown_argument_tokens_are_reported_once() {
    assert_eq!(
        unknown_argument_tokens("{path} {missing} {Language} {missing} {language}"),
        vec!["{missing}".to_owned(), "{language}".to_owned()]
    );
}

#[test]
fn argument_tokens_are_reported_once_in_first_appearance_order() {
    assert_eq!(
        argument_tokens_in_first_appearance_order(
            "{selectdir} {path} {inputtext} {path} {selectfile} {inputtext}"
        ),
        vec![
            ArgumentToken::SelectDir,
            ArgumentToken::Path,
            ArgumentToken::InputText,
            ArgumentToken::SelectFile,
        ]
    );
}

#[test]
fn interactive_argument_tokens_keep_prompt_order() {
    assert_eq!(
        interactive_argument_tokens_in_first_appearance_order(
            "{name} {selectfile} {selectdir} {inputtext} {selectfile}"
        ),
        vec![
            ArgumentToken::SelectFile,
            ArgumentToken::SelectDir,
            ArgumentToken::InputText,
        ]
    );
}

#[test]
fn indexed_reorder_drop_destination_accounts_for_removed_source() {
    assert_eq!(indexed_reorder_drop_destination(1, 3, false, 4), Some(2));
    assert_eq!(indexed_reorder_drop_destination(3, 1, true, 4), Some(2));
    assert_eq!(indexed_reorder_drop_destination(3, 1, false, 4), Some(1));
    assert_eq!(indexed_reorder_drop_destination(0, 3, true, 4), Some(3));
}

#[test]
fn indexed_reorder_drop_destination_ignores_noop_or_invalid_targets() {
    assert_eq!(indexed_reorder_drop_destination(1, 2, false, 4), None);
    assert_eq!(indexed_reorder_drop_destination(1, 1, true, 4), None);
    assert_eq!(indexed_reorder_drop_destination(4, 1, true, 4), None);
    assert_eq!(indexed_reorder_drop_destination(1, 4, true, 4), None);
    assert_eq!(indexed_reorder_drop_destination(0, 1, true, 1), None);
}

#[test]
fn direct_reorder_drop_destination_uses_target_index() {
    assert_eq!(direct_reorder_drop_destination(0, 2, 4), Some(2));
    assert_eq!(direct_reorder_drop_destination(3, 1, 4), Some(1));
    assert_eq!(direct_reorder_drop_destination(1, 1, 4), None);
    assert_eq!(direct_reorder_drop_destination(4, 1, 4), None);
    assert_eq!(direct_reorder_drop_destination(1, 4, 4), None);
    assert_eq!(direct_reorder_drop_destination(0, 1, 1), None);
}

#[test]
fn replace_argument_tokens_reuses_one_value_for_repeated_tokens() {
    let replaced = replace_argument_tokens(
        "--cwd {path} --again {path} --language {Language}",
        &[
            (ArgumentToken::Path, r"C:\dev\space path".to_owned()),
            (ArgumentToken::Language, "Rust".to_owned()),
        ],
    )
    .expect("known tokens should be replaced");

    assert_eq!(
        replaced,
        r"--cwd C:\dev\space path --again C:\dev\space path --language Rust"
    );
}

#[test]
fn resolves_workspace_argument_tokens_before_replacement() {
    let workspace = Workspace::new(r"C:\dev\space path", "j3DevHelper", "Rust")
        .expect("workspace should be valid");
    let replacements =
        resolve_argument_replacements("{path}|{name}|{Language}", Some(&workspace), |_| {
            Err("interactive prompt should not run".to_owned())
        })
        .expect("workspace tokens should resolve")
        .expect("resolution should not be cancelled");

    let replaced = replace_argument_tokens("{path}|{name}|{Language}", &replacements)
        .expect("resolved values should replace known tokens");

    assert_eq!(replaced, r"C:\dev\space path|j3DevHelper|Rust");
}

#[test]
fn resolves_interactive_tokens_once_in_prompt_order_and_reuses_values() {
    let mut prompts = Vec::new();
    let replacements = resolve_argument_replacements(
        "{inputtext} {selectfile} {inputtext} {selectdir} {selectfile}",
        None,
        |token| {
            prompts.push(token);
            Ok(Some(
                match token {
                    ArgumentToken::SelectFile => r"C:\picked file.txt",
                    ArgumentToken::SelectDir => r"C:\picked dir",
                    ArgumentToken::InputText => "typed text",
                    ArgumentToken::Path | ArgumentToken::Name | ArgumentToken::Language => {
                        "unexpected"
                    }
                }
                .to_owned(),
            ))
        },
    )
    .expect("interactive tokens should resolve")
    .expect("resolution should not be cancelled");

    assert_eq!(
        prompts,
        vec![
            ArgumentToken::InputText,
            ArgumentToken::SelectFile,
            ArgumentToken::SelectDir,
        ]
    );
    assert_eq!(
        replace_argument_tokens(
            "{inputtext} {selectfile} {inputtext} {selectdir} {selectfile}",
            &replacements,
        )
        .expect("resolved values should replace known tokens"),
        r"typed text C:\picked file.txt typed text C:\picked dir C:\picked file.txt"
    );
}

#[test]
fn resolving_arguments_cancels_entire_execution_when_prompt_is_cancelled() {
    let mut prompts = Vec::new();
    let result =
        resolve_argument_replacements("{selectfile} {selectdir} {inputtext}", None, |token| {
            prompts.push(token);
            match token {
                ArgumentToken::SelectFile => Ok(Some(r"C:\picked file.txt".to_owned())),
                ArgumentToken::SelectDir => Ok(None),
                ArgumentToken::InputText => Ok(Some("should not be requested".to_owned())),
                ArgumentToken::Path | ArgumentToken::Name | ArgumentToken::Language => {
                    Ok(Some("unexpected".to_owned()))
                }
            }
        })
        .expect("prompt cancellation is not an error");

    assert_eq!(result, None);
    assert_eq!(
        prompts,
        vec![ArgumentToken::SelectFile, ArgumentToken::SelectDir]
    );
}

#[test]
fn replace_argument_tokens_rejects_case_mismatched_language_token() {
    let error = replace_argument_tokens(
        "{language}",
        &[(ArgumentToken::Language, "Rust".to_owned())],
    )
    .expect_err("{language} should not be accepted");

    assert_eq!(
        error,
        ArgumentReplacementError::UnknownToken("{language}".to_owned())
    );
}

#[test]
fn resolving_arguments_rejects_unknown_tokens_before_prompting() {
    let mut prompted = false;
    let error = resolve_argument_replacements("{language} {inputtext}", None, |_| {
        prompted = true;
        Ok(Some("typed".to_owned()))
    })
    .expect_err("unknown token should be rejected");

    assert_eq!(
        error,
        ArgumentResolutionError::UnknownTokens(vec!["{language}".to_owned()])
    );
    assert!(!prompted);
}

#[test]
fn arguments_require_workspace_only_for_workspace_tokens() {
    assert!(arguments_require_workspace("{path} {selectfile}"));
    assert!(arguments_require_workspace("{Language}"));
    assert!(!arguments_require_workspace("{selectfile} {inputtext}"));
}
