use crate::domain::{
    AppSettings, Category, CommandButton, CommandTab, DomainValidationError, ExecutionType,
    TreeRootItem, UI_FONT_SIZE_OPTIONS, UiLanguage, ViewSettings, Workspace, category_names_equal,
    default_workspace_language_options, is_supported_ui_font_size, normalize_ui_language,
    normalize_workspace_language_options, workspace_paths_equal,
};

use super::stored_document::{
    CategoryDocument, CommandButtonDocument, CommandTabDocument, SettingsDocument,
    TreeRootItemDocument, ViewDocument, WorkspaceDocument,
};

pub(super) fn restore_settings(document: SettingsDocument) -> (AppSettings, Vec<String>) {
    document.into_settings()
}

impl SettingsDocument {
    fn into_settings(self) -> (AppSettings, Vec<String>) {
        let mut warnings = Vec::new();
        let mut category_names: Vec<String> = Vec::new();
        let mut workspace_paths: Vec<String> = Vec::new();
        let view = self.view.into_domain(&mut warnings);
        let languages = language_options_or_default(self.languages, &mut warnings);

        let categories = self
            .categories
            .into_iter()
            .enumerate()
            .filter_map(|(index, category)| match category.into_domain() {
                Ok(category) => {
                    if category_names
                        .iter()
                        .any(|name| category_names_equal(name, &category.name))
                    {
                        warnings.push(format!(
                            "categories[{index}]: 이미 있는 분류라 건너뜀: {}",
                            category.name
                        ));
                        None
                    } else {
                        category_names.push(category.name.clone());
                        Some(category)
                    }
                }
                Err(error) => {
                    warnings.push(format!("categories[{index}]: {}", error.user_message()));
                    None
                }
            })
            .collect();

        let workspaces = self
            .workspaces
            .into_iter()
            .enumerate()
            .filter_map(|(index, workspace)| {
                match workspace.into_domain(&languages, &mut warnings, index) {
                    Ok(mut workspace) => {
                        if workspace_paths
                            .iter()
                            .any(|path| workspace_paths_equal(path, &workspace.path))
                        {
                            warnings.push(format!(
                                "workspaces[{index}]: 이미 등록된 폴더라 건너뜀: {}",
                                workspace.path
                            ));
                            None
                        } else {
                            if let Some(category) = workspace.category.as_deref()
                                && !category_names
                                    .iter()
                                    .any(|name| category_names_equal(name, category))
                            {
                                warnings.push(format!(
                                    "workspaces[{index}]: 없는 분류라 최상위로 복원함: {category}"
                                ));
                                workspace.set_category(None);
                            }
                            workspace_paths.push(workspace.path.clone());
                            Some(workspace)
                        }
                    }
                    Err(error) => {
                        warnings.push(format!("workspaces[{index}]: {}", error.user_message()));
                        None
                    }
                }
            })
            .collect();

        let tree_order = self
            .tree_order
            .into_iter()
            .enumerate()
            .filter_map(|(index, item)| match item.into_domain() {
                Ok(item) => Some(item),
                Err(error) => {
                    warnings.push(format!("tree_order[{index}]: {}", error.user_message()));
                    None
                }
            })
            .collect();

        let command_tabs = self
            .command_tabs
            .into_iter()
            .enumerate()
            .filter_map(|(index, tab)| match tab.into_domain(&mut warnings, index) {
                Ok(tab) => Some(tab),
                Err(error) => {
                    warnings.push(format!("command_tabs[{index}]: {}", error.user_message()));
                    None
                }
            })
            .collect();

        let mut settings = AppSettings {
            view,
            languages,
            tree_order,
            categories,
            workspaces,
            command_tabs,
        };
        if !settings.tree_order.is_empty() {
            settings.sync_tree_order();
        }

        (settings, warnings)
    }
}

impl ViewDocument {
    fn into_domain(self, warnings: &mut Vec<String>) -> ViewSettings {
        let default = ViewSettings::default();
        let font_size = match self.font_size {
            Some(toml::Value::Integer(font_size)) => {
                font_size_or_default(font_size, default.font_size, warnings)
            }
            Some(value) => invalid_font_size_type_or_default(value, default.font_size, warnings),
            None => default.font_size,
        };
        let parsed_window_width =
            positive_i32_or_none(self.window_width, "view.window_width", warnings);
        let parsed_window_height =
            positive_i32_or_none(self.window_height, "view.window_height", warnings);
        let (window_width, window_height) =
            window_size_or_none(parsed_window_width, parsed_window_height, warnings);
        let tree_panel_width =
            positive_i32_or_none(self.tree_panel_width, "view.tree_panel_width", warnings);

        ViewSettings::new(
            usable_string_or_default(self.font_family, default.font_family),
            font_size,
            usable_string_or_default(self.theme, default.theme),
        )
        .with_ui_language(ui_language_or_default(
            self.ui_language,
            default.ui_language,
            warnings,
        ))
        .with_window_layout(window_width, window_height, tree_panel_width)
    }
}

impl CategoryDocument {
    fn into_domain(self) -> Result<Category, RestoreItemError> {
        Ok(Category::new(required_category_field(self.name, "name")?)?)
    }
}

impl TreeRootItemDocument {
    fn into_domain(self) -> Result<TreeRootItem, RestoreItemError> {
        let item_type = required_tree_order_field(self.item_type, "type")?;
        match item_type.as_str() {
            "category" => Ok(TreeRootItem::category(required_tree_order_field(
                self.name, "name",
            )?)),
            "workspace" => Ok(TreeRootItem::workspace(required_tree_order_field(
                self.path, "path",
            )?)),
            _ => Err(RestoreItemError::InvalidTreeOrderType(item_type)),
        }
    }
}

impl WorkspaceDocument {
    fn into_domain(
        self,
        languages: &[String],
        warnings: &mut Vec<String>,
        workspace_index: usize,
    ) -> Result<Workspace, RestoreItemError> {
        let mut workspace = Workspace::new_with_language_options(
            required_workspace_field(self.path, "path")?,
            required_workspace_field(self.name, "name")?,
            required_workspace_field(self.language, "Language")?,
            languages,
        )?;

        match optional_workspace_field(self.category, "category") {
            Ok(category) => workspace.set_category(category),
            Err(error) => warnings.push(format!(
                "workspaces[{workspace_index}]: {}",
                error.user_message()
            )),
        }

        Ok(workspace)
    }
}

impl CommandTabDocument {
    fn into_domain(
        self,
        warnings: &mut Vec<String>,
        tab_index: usize,
    ) -> Result<CommandTab, RestoreItemError> {
        let buttons = self
            .buttons
            .into_iter()
            .enumerate()
            .filter_map(|(button_index, button)| match button.into_domain() {
                Ok(button) => Some(button),
                Err(error) => {
                    warnings.push(format!(
                        "command_tabs[{tab_index}].buttons[{button_index}]: {}",
                        error.user_message()
                    ));
                    None
                }
            })
            .collect();

        let name = required_command_tab_field(self.name, "name")?;
        Ok(CommandTab::new(name, buttons)?)
    }
}

impl CommandButtonDocument {
    fn into_domain(self) -> Result<CommandButton, RestoreItemError> {
        let execution_type_value =
            required_command_button_field(self.execution_type, "execution_type")?;
        let execution_type = ExecutionType::from_config_value(&execution_type_value).ok_or(
            DomainValidationError::InvalidExecutionType(execution_type_value),
        )?;

        Ok(CommandButton::new(
            required_command_button_field(self.button_name, "button_name")?,
            required_command_button_field(self.executable_path, "executable_path")?,
            optional_command_button_field(self.arguments, "arguments")?,
            execution_type,
        )?)
    }
}

enum RestoreItemError {
    Domain(DomainValidationError),
    InvalidStringField { field: &'static str, actual: String },
    MissingTreeOrderField(&'static str),
    InvalidTreeOrderType(String),
}

impl RestoreItemError {
    fn user_message(&self) -> String {
        match self {
            Self::Domain(error) => error.user_message().to_string(),
            Self::InvalidStringField { field, actual } => {
                format!("{field}: 문자열이 아니라 건너뜀: {actual}")
            }
            Self::MissingTreeOrderField(field) => {
                format!("{field}: 필수 값이 없어 건너뜀")
            }
            Self::InvalidTreeOrderType(item_type) => {
                format!("지원하지 않는 트리 항목이라 건너뜀: {item_type}")
            }
        }
    }
}

impl From<DomainValidationError> for RestoreItemError {
    fn from(error: DomainValidationError) -> Self {
        Self::Domain(error)
    }
}

fn font_size_or_default(font_size: i64, default: u16, warnings: &mut Vec<String>) -> u16 {
    if (0..=i64::from(u16::MAX)).contains(&font_size) {
        let font_size = font_size as u16;
        if is_supported_ui_font_size(font_size) {
            return font_size;
        }
    }

    warnings.push(format!(
        "view.font_size: 지원 크기({})가 아니라 기본값으로 복원함: {font_size}",
        supported_font_size_labels()
    ));
    default
}

fn invalid_font_size_type_or_default(
    value: toml::Value,
    default: u16,
    warnings: &mut Vec<String>,
) -> u16 {
    warnings.push(format!(
        "view.font_size: 정수가 아니라 기본값으로 복원함: {}",
        toml_value_label(&value)
    ));
    default
}

fn ui_language_or_default(
    value: Option<String>,
    default: String,
    warnings: &mut Vec<String>,
) -> String {
    let Some(value) = value else {
        return default;
    };

    if value.trim().is_empty() {
        return default;
    }

    if UiLanguage::from_config_value(&value).is_some() {
        normalize_ui_language(value)
    } else {
        warnings.push(format!(
            "view.ui_language: 지원하지 않는 값이라 기본값으로 복원함: {value}"
        ));
        default
    }
}

fn positive_i32_or_none(
    value: Option<toml::Value>,
    field: &'static str,
    warnings: &mut Vec<String>,
) -> Option<i32> {
    match value {
        Some(toml::Value::Integer(value)) if (1..=i64::from(i32::MAX)).contains(&value) => {
            Some(value as i32)
        }
        Some(toml::Value::Integer(value)) => {
            warnings.push(format!("{field}: 양수 정수가 아니라 건너뜀: {value}"));
            None
        }
        Some(value) => {
            warnings.push(format!(
                "{field}: 정수가 아니라 건너뜀: {}",
                toml_value_label(&value)
            ));
            None
        }
        None => None,
    }
}

fn window_size_or_none(
    width: Option<i32>,
    height: Option<i32>,
    warnings: &mut Vec<String>,
) -> (Option<i32>, Option<i32>) {
    match (width, height) {
        (Some(width), Some(height)) => (Some(width), Some(height)),
        (None, None) => (None, None),
        _ => {
            warnings.push(
                "view.window_width/view.window_height: 두 값이 모두 있어야 창 크기를 복원합니다."
                    .to_owned(),
            );
            (None, None)
        }
    }
}

fn language_options_or_default(
    value: Option<toml::Value>,
    warnings: &mut Vec<String>,
) -> Vec<String> {
    let Some(value) = value else {
        return default_workspace_language_options();
    };

    let values = match value {
        toml::Value::Array(values) => values,
        value => {
            warnings.push(format!(
                "languages: 문자열 배열이 아니라 기본 언어 목록으로 복원함: {}",
                toml_value_label(&value)
            ));
            return default_workspace_language_options();
        }
    };

    let mut languages = Vec::with_capacity(values.len());
    for (index, value) in values.into_iter().enumerate() {
        match value {
            toml::Value::String(language) => languages.push(language),
            value => {
                warnings.push(format!(
                    "languages[{index}]: 문자열이 아니라 기본 언어 목록으로 복원함: {}",
                    toml_value_label(&value)
                ));
                return default_workspace_language_options();
            }
        }
    }

    match normalize_workspace_language_options(languages) {
        Ok(languages) => languages,
        Err(error) => {
            warnings.push(format!(
                "languages: {} 기본 언어 목록으로 복원함.",
                error.user_message()
            ));
            default_workspace_language_options()
        }
    }
}

fn required_workspace_field(
    value: Option<toml::Value>,
    field: &'static str,
) -> Result<String, RestoreItemError> {
    required_string_field(
        value,
        field,
        DomainValidationError::MissingWorkspaceField(field),
    )
}

fn required_tree_order_field(
    value: Option<toml::Value>,
    field: &'static str,
) -> Result<String, RestoreItemError> {
    string_field(value, field)?.ok_or(RestoreItemError::MissingTreeOrderField(field))
}

fn optional_workspace_field(
    value: Option<toml::Value>,
    field: &'static str,
) -> Result<Option<String>, RestoreItemError> {
    string_field(value, field)
}

fn required_category_field(
    value: Option<toml::Value>,
    field: &'static str,
) -> Result<String, RestoreItemError> {
    required_string_field(
        value,
        field,
        DomainValidationError::MissingCategoryField(field),
    )
}

fn required_command_tab_field(
    value: Option<toml::Value>,
    field: &'static str,
) -> Result<String, RestoreItemError> {
    required_string_field(
        value,
        field,
        DomainValidationError::MissingCommandTabField(field),
    )
}

fn required_command_button_field(
    value: Option<toml::Value>,
    field: &'static str,
) -> Result<String, RestoreItemError> {
    required_string_field(
        value,
        field,
        DomainValidationError::MissingCommandButtonField(field),
    )
}

fn optional_command_button_field(
    value: Option<toml::Value>,
    field: &'static str,
) -> Result<String, RestoreItemError> {
    Ok(string_field(value, field)?.unwrap_or_default())
}

fn required_string_field(
    value: Option<toml::Value>,
    field: &'static str,
    missing_error: DomainValidationError,
) -> Result<String, RestoreItemError> {
    string_field(value, field)?.ok_or_else(|| RestoreItemError::from(missing_error))
}

fn string_field(
    value: Option<toml::Value>,
    field: &'static str,
) -> Result<Option<String>, RestoreItemError> {
    match value {
        Some(toml::Value::String(value)) => Ok(Some(value)),
        Some(value) => Err(RestoreItemError::InvalidStringField {
            field,
            actual: toml_value_label(&value),
        }),
        None => Ok(None),
    }
}

fn usable_string_or_default(value: Option<String>, default: String) -> String {
    match value {
        Some(value) if !value.trim().is_empty() => value,
        _ => default,
    }
}

fn supported_font_size_labels() -> String {
    UI_FONT_SIZE_OPTIONS
        .iter()
        .map(u16::to_string)
        .collect::<Vec<_>>()
        .join(", ")
}

fn toml_value_label(value: &toml::Value) -> String {
    match value {
        toml::Value::String(value) => value.clone(),
        toml::Value::Integer(value) => value.to_string(),
        toml::Value::Float(value) => value.to_string(),
        toml::Value::Boolean(value) => value.to_string(),
        toml::Value::Datetime(value) => value.to_string(),
        toml::Value::Array(_) => "array".to_string(),
        toml::Value::Table(_) => "table".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn restore_settings_skips_command_button_with_interior_nul_argument() {
        let document = SettingsDocument {
            view: ViewDocument::default(),
            languages: None,
            tree_order: Vec::new(),
            categories: Vec::new(),
            workspaces: Vec::new(),
            command_tabs: vec![CommandTabDocument {
                name: Some(toml::Value::String("Tools".to_owned())),
                buttons: vec![CommandButtonDocument {
                    button_name: Some(toml::Value::String("Run".to_owned())),
                    executable_path: Some(toml::Value::String("tool.exe".to_owned())),
                    arguments: Some(toml::Value::String("--safe\0--evil".to_owned())),
                    execution_type: Some(toml::Value::String("shell_api".to_owned())),
                }],
            }],
        };

        let (settings, warnings) = restore_settings(document);

        assert_eq!(settings.command_tabs.len(), 1);
        assert!(settings.command_tabs[0].buttons.is_empty());
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].contains("command_tabs[0].buttons[0]"));
        assert!(warnings[0].contains("arguments"));
        assert!(warnings[0].contains("사용할 수 없는 문자"));
    }

    #[test]
    fn restore_settings_skips_display_names_with_interior_nul() {
        let document = SettingsDocument {
            view: ViewDocument::default(),
            languages: None,
            tree_order: Vec::new(),
            categories: vec![CategoryDocument {
                name: Some(toml::Value::String("Backend\0Hidden".to_owned())),
            }],
            workspaces: vec![WorkspaceDocument {
                path: Some(toml::Value::String("C:\\projects\\api".to_owned())),
                name: Some(toml::Value::String("api\0hidden".to_owned())),
                language: Some(toml::Value::String("Rust".to_owned())),
                category: None,
            }],
            command_tabs: vec![CommandTabDocument {
                name: Some(toml::Value::String("Tools\0Hidden".to_owned())),
                buttons: Vec::new(),
            }],
        };

        let (settings, warnings) = restore_settings(document);

        assert!(settings.categories.is_empty());
        assert!(settings.workspaces.is_empty());
        assert!(settings.command_tabs.is_empty());
        assert_eq!(warnings.len(), 3);
        assert!(warnings[0].contains("categories[0]"));
        assert!(warnings[0].contains("name"));
        assert!(warnings[0].contains("사용할 수 없는 문자"));
        assert!(warnings[1].contains("workspaces[0]"));
        assert!(warnings[1].contains("name"));
        assert!(warnings[1].contains("사용할 수 없는 문자"));
        assert!(warnings[2].contains("command_tabs[0]"));
        assert!(warnings[2].contains("name"));
        assert!(warnings[2].contains("사용할 수 없는 문자"));
    }
}
