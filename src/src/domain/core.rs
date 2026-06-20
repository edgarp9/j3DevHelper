use std::collections::HashMap;
use std::path::Path;

use super::localization::{DEFAULT_UI_LANGUAGE, UiLanguage};

pub const DEFAULT_FONT_FAMILY: &str = "Segoe UI";
pub const DEFAULT_FONT_SIZE: u16 = 12;
pub const MIN_UI_FONT_SIZE: u16 = 9;
pub const MAX_UI_FONT_SIZE: u16 = 20;
pub const UI_FONT_SIZE_OPTIONS: &[u16] = &[9, 10, 11, 12, 13, 14, 16, 18, 20];
pub const DEFAULT_THEME: &str = "graphite";
pub const DEFAULT_WORKSPACE_LANGUAGE: &str = "Other";
pub const WORKSPACE_LANGUAGE_OPTIONS: &[&str] = &[
    "Rust",
    "TypeScript",
    "JavaScript",
    "Python",
    "Go",
    "Java",
    "C#",
    "C",
    "C++",
    "Other",
];

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ViewTheme {
    System,
    Light,
    ClassicDark,
    SepiaTeal,
    #[default]
    Graphite,
    Forest,
    SteelBlue,
}

const VIEW_THEME_OPTIONS: [ViewTheme; 7] = [
    ViewTheme::System,
    ViewTheme::Light,
    ViewTheme::ClassicDark,
    ViewTheme::SepiaTeal,
    ViewTheme::Graphite,
    ViewTheme::Forest,
    ViewTheme::SteelBlue,
];

impl ViewTheme {
    pub fn display_name(self) -> &'static str {
        self.display_name_for(UiLanguage::Korean)
    }

    pub fn display_name_for(self, language: UiLanguage) -> &'static str {
        match language {
            UiLanguage::Korean => self.korean_display_name(),
            UiLanguage::English => self.english_display_name(),
        }
    }

    fn korean_display_name(self) -> &'static str {
        match self {
            Self::System => "시스템",
            Self::Light => "밝게",
            Self::ClassicDark => "어둡게",
            Self::SepiaTeal => "세피아",
            Self::Graphite => "그래파이트",
            Self::Forest => "숲",
            Self::SteelBlue => "스틸 블루",
        }
    }

    fn english_display_name(self) -> &'static str {
        match self {
            Self::System => "System",
            Self::Light => "Light",
            Self::ClassicDark => "Classic Dark",
            Self::SepiaTeal => "Sepia Teal",
            Self::Graphite => "Graphite",
            Self::Forest => "Forest",
            Self::SteelBlue => "Steel Blue",
        }
    }

    pub fn as_config_value(self) -> &'static str {
        match self {
            Self::System => "system",
            Self::Light => "light",
            Self::ClassicDark => "classic-dark",
            Self::SepiaTeal => "sepia-teal",
            Self::Graphite => "graphite",
            Self::Forest => "forest",
            Self::SteelBlue => "steel-blue",
        }
    }

    pub fn from_config_value(value: &str) -> Option<Self> {
        let normalized = value.trim().to_ascii_lowercase();
        match normalized.as_str() {
            "system" => Some(Self::System),
            "default" => Some(Self::default()),
            "light" => Some(Self::Light),
            "dark" | "classic-dark" | "classic_dark" => Some(Self::ClassicDark),
            "sepia-teal" | "sepia_teal" | "sepia" => Some(Self::SepiaTeal),
            "graphite" | "gray" | "grey" => Some(Self::Graphite),
            "forest" | "green" => Some(Self::Forest),
            "steel-blue" | "steel_blue" | "steel" => Some(Self::SteelBlue),
            _ => None,
        }
    }

    pub fn options() -> &'static [Self] {
        &VIEW_THEME_OPTIONS
    }

    pub fn uses_dark_mode(self) -> bool {
        !matches!(self, Self::System | Self::Light)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Workspace {
    pub path: String,
    pub name: String,
    pub language: String,
    pub category: Option<String>,
}

impl Workspace {
    pub fn new(
        path: impl Into<String>,
        name: impl Into<String>,
        language: impl Into<String>,
    ) -> Result<Self, DomainValidationError> {
        Self::new_with_language_options(path, name, language, &default_workspace_language_options())
    }

    pub fn new_with_language_options(
        path: impl Into<String>,
        name: impl Into<String>,
        language: impl Into<String>,
        language_options: &[String],
    ) -> Result<Self, DomainValidationError> {
        let path = path.into().trim().to_owned();
        let name = name.into().trim().to_owned();
        let language = language.into().trim().to_owned();

        if path.trim().is_empty() {
            return Err(DomainValidationError::MissingWorkspaceField("path"));
        }

        if name.trim().is_empty() {
            return Err(DomainValidationError::MissingWorkspaceField("name"));
        }

        reject_interior_nul(
            "name",
            &name,
            DomainValidationError::InteriorNulWorkspaceField,
        )?;

        if language.trim().is_empty() {
            return Err(DomainValidationError::MissingWorkspaceField("Language"));
        }

        reject_interior_nul(
            "Language",
            &language,
            DomainValidationError::InteriorNulWorkspaceField,
        )?;

        let Some(language) = normalize_workspace_language(&language, language_options) else {
            return Err(DomainValidationError::UnsupportedWorkspaceLanguage(
                language,
            ));
        };

        Ok(Self {
            path,
            name,
            language,
            category: None,
        })
    }

    pub fn with_category(mut self, category: Option<String>) -> Self {
        self.set_category(category);
        self
    }

    pub fn set_category(&mut self, category: Option<String>) {
        self.category = normalize_workspace_category(category);
    }
}

fn normalize_workspace_category(category: Option<String>) -> Option<String> {
    category
        .map(|category| category.trim().to_owned())
        .filter(|category| !category.is_empty())
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Category {
    pub name: String,
}

impl Category {
    pub fn new(name: impl Into<String>) -> Result<Self, DomainValidationError> {
        let name = name.into().trim().to_owned();

        if name.is_empty() {
            return Err(DomainValidationError::MissingCategoryField("name"));
        }

        reject_interior_nul(
            "name",
            &name,
            DomainValidationError::InteriorNulCategoryField,
        )?;

        Ok(Self { name })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TreeRootItem {
    Category { name: String },
    Workspace { path: String },
}

impl TreeRootItem {
    pub fn category(name: impl Into<String>) -> Self {
        Self::Category {
            name: name.into().trim().to_owned(),
        }
    }

    pub fn workspace(path: impl Into<String>) -> Self {
        Self::Workspace {
            path: path.into().trim().to_owned(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TreeRootItemRef {
    Category(usize),
    Workspace(usize),
}

pub fn indexed_reorder_drop_destination(
    source_index: usize,
    target_index: usize,
    insert_after: bool,
    len: usize,
) -> Option<usize> {
    if source_index >= len || target_index >= len || source_index == target_index || len <= 1 {
        return None;
    }

    let destination_index = if insert_after {
        if source_index < target_index {
            target_index
        } else {
            target_index.saturating_add(1).min(len - 1)
        }
    } else if source_index < target_index {
        target_index.saturating_sub(1)
    } else {
        target_index
    };

    (destination_index != source_index).then_some(destination_index)
}

pub fn direct_reorder_drop_destination(
    source_index: usize,
    target_index: usize,
    len: usize,
) -> Option<usize> {
    if source_index >= len || target_index >= len || source_index == target_index || len <= 1 {
        None
    } else {
        Some(target_index)
    }
}

pub fn default_workspace_name_for_path(path: &Path) -> String {
    path.file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.trim().is_empty())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| path.display().to_string())
}

pub fn is_supported_workspace_language(language: &str) -> bool {
    WORKSPACE_LANGUAGE_OPTIONS
        .iter()
        .any(|option| workspace_language_names_equal(option, language))
}

pub fn is_supported_workspace_language_in(language: &str, language_options: &[String]) -> bool {
    normalize_workspace_language(language, language_options).is_some()
}

pub fn is_supported_ui_font_size(font_size: u16) -> bool {
    UI_FONT_SIZE_OPTIONS.contains(&font_size)
}

pub fn default_workspace_language_options() -> Vec<String> {
    WORKSPACE_LANGUAGE_OPTIONS
        .iter()
        .map(|language| (*language).to_owned())
        .collect()
}

pub fn default_workspace_language_for_options(language_options: &[String]) -> String {
    normalize_workspace_language(DEFAULT_WORKSPACE_LANGUAGE, language_options)
        .or_else(|| language_options.first().cloned())
        .unwrap_or_else(|| DEFAULT_WORKSPACE_LANGUAGE.to_owned())
}

pub fn normalize_workspace_language_options(
    languages: impl IntoIterator<Item = impl AsRef<str>>,
) -> Result<Vec<String>, DomainValidationError> {
    let mut normalized: Vec<String> = Vec::new();

    for language in languages {
        let language = language.as_ref().trim();
        if language.is_empty() {
            continue;
        }

        reject_interior_nul(
            "Language",
            language,
            DomainValidationError::InteriorNulLanguageConfigField,
        )?;

        if normalized
            .iter()
            .any(|existing| workspace_language_names_equal(existing, language))
        {
            return Err(DomainValidationError::DuplicateWorkspaceLanguage(
                language.to_owned(),
            ));
        }

        normalized.push(language.to_owned());
    }

    if normalized.is_empty() {
        Err(DomainValidationError::MissingWorkspaceLanguageOptions)
    } else {
        Ok(normalized)
    }
}

pub fn normalize_workspace_language(language: &str, language_options: &[String]) -> Option<String> {
    let language = language.trim();
    language_options
        .iter()
        .find(|option| option.as_str() == language)
        .cloned()
        .or_else(|| {
            language_options
                .iter()
                .find(|option| workspace_language_names_equal(option, language))
                .cloned()
        })
}

fn workspace_language_names_equal(left: &str, right: &str) -> bool {
    left.trim().eq_ignore_ascii_case(right.trim())
}

pub fn infer_workspace_language_from_entry_names<'a>(
    entry_names: impl IntoIterator<Item = &'a str>,
) -> Option<&'static str> {
    let mut has_cargo_manifest = false;
    let mut has_rust_source = false;
    let mut has_tsconfig = false;
    let mut has_typescript_source = false;
    let mut has_package_manifest = false;
    let mut has_javascript_source = false;
    let mut has_python_project = false;
    let mut has_python_source = false;
    let mut has_go_module = false;
    let mut has_go_source = false;
    let mut has_java_project = false;
    let mut has_java_source = false;
    let mut has_csharp_project = false;
    let mut has_csharp_source = false;
    let mut has_c_source = false;
    let mut has_cpp_project = false;
    let mut has_cpp_source = false;

    for entry_name in entry_names {
        let normalized = entry_name.trim().to_ascii_lowercase();

        match normalized.as_str() {
            "cargo.toml" => has_cargo_manifest = true,
            "tsconfig.json" => has_tsconfig = true,
            "package.json" => has_package_manifest = true,
            "pyproject.toml" | "requirements.txt" | "setup.py" => has_python_project = true,
            "go.mod" => has_go_module = true,
            "pom.xml" | "build.gradle" | "build.gradle.kts" => has_java_project = true,
            "cmakelists.txt" => has_cpp_project = true,
            _ => {}
        }

        if normalized.ends_with(".rs") {
            has_rust_source = true;
        } else if normalized.ends_with(".ts") || normalized.ends_with(".tsx") {
            has_typescript_source = true;
        } else if normalized.ends_with(".js") || normalized.ends_with(".jsx") {
            has_javascript_source = true;
        } else if normalized.ends_with(".py") {
            has_python_source = true;
        } else if normalized.ends_with(".go") {
            has_go_source = true;
        } else if normalized.ends_with(".java") {
            has_java_source = true;
        } else if normalized.ends_with(".csproj") {
            has_csharp_project = true;
        } else if normalized.ends_with(".cs") {
            has_csharp_source = true;
        } else if normalized.ends_with(".c") {
            has_c_source = true;
        } else if [".cpp", ".cc", ".cxx", ".hpp", ".hxx", ".hh"]
            .iter()
            .any(|extension| normalized.ends_with(extension))
        {
            has_cpp_source = true;
        }
    }

    if has_cargo_manifest || has_rust_source {
        Some("Rust")
    } else if has_tsconfig || has_typescript_source {
        Some("TypeScript")
    } else if has_package_manifest || has_javascript_source {
        Some("JavaScript")
    } else if has_python_project || has_python_source {
        Some("Python")
    } else if has_go_module || has_go_source {
        Some("Go")
    } else if has_java_project || has_java_source {
        Some("Java")
    } else if has_csharp_project || has_csharp_source {
        Some("C#")
    } else if has_c_source {
        Some("C")
    } else if has_cpp_project || has_cpp_source {
        Some("C++")
    } else {
        None
    }
}

pub fn workspace_paths_equal(left: &str, right: &str) -> bool {
    let left = comparable_workspace_path(left);
    let right = comparable_workspace_path(right);

    left.eq_ignore_ascii_case(&right)
}

pub fn category_names_equal(left: &str, right: &str) -> bool {
    left.trim().eq_ignore_ascii_case(right.trim())
}

fn comparable_workspace_path(path: &str) -> String {
    let trimmed = path.trim();
    let without_trailing_separator = trimmed.trim_end_matches(['\\', '/']);
    if without_trailing_separator.is_empty() {
        trimmed.to_owned()
    } else {
        without_trailing_separator.to_owned()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommandTab {
    pub name: String,
    pub buttons: Vec<CommandButton>,
}

impl CommandTab {
    pub fn new(
        name: impl Into<String>,
        buttons: Vec<CommandButton>,
    ) -> Result<Self, DomainValidationError> {
        let name = name.into();
        if name.trim().is_empty() {
            return Err(DomainValidationError::MissingCommandTabField("name"));
        }

        let name = name.trim().to_owned();
        reject_interior_nul(
            "name",
            &name,
            DomainValidationError::InteriorNulCommandTabField,
        )?;

        Ok(Self { name, buttons })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommandButton {
    pub button_name: String,
    pub executable_path: String,
    pub arguments: String,
    pub execution_type: ExecutionType,
}

impl CommandButton {
    pub fn new(
        button_name: impl Into<String>,
        executable_path: impl Into<String>,
        arguments: impl Into<String>,
        execution_type: ExecutionType,
    ) -> Result<Self, DomainValidationError> {
        let button_name = button_name.into().trim().to_owned();
        let executable_path = executable_path.into().trim().to_owned();
        let arguments = arguments.into();

        if button_name.is_empty() {
            return Err(DomainValidationError::MissingCommandButtonField(
                "button_name",
            ));
        }

        if executable_path.is_empty() {
            return Err(DomainValidationError::MissingCommandButtonField(
                "executable_path",
            ));
        }

        reject_interior_nul(
            "button_name",
            &button_name,
            DomainValidationError::InteriorNulCommandButtonField,
        )?;
        reject_interior_nul(
            "executable_path",
            &executable_path,
            DomainValidationError::InteriorNulCommandButtonField,
        )?;
        reject_interior_nul(
            "arguments",
            &arguments,
            DomainValidationError::InteriorNulCommandButtonField,
        )?;

        Ok(Self {
            button_name,
            executable_path,
            arguments,
            execution_type,
        })
    }
}

fn reject_interior_nul(
    field: &'static str,
    value: &str,
    error: impl FnOnce(&'static str) -> DomainValidationError,
) -> Result<(), DomainValidationError> {
    if value.contains('\0') {
        Err(error(field))
    } else {
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExecutionType {
    ShellApi,
    ExternalTerminal,
}

impl ExecutionType {
    pub fn from_config_value(value: &str) -> Option<Self> {
        match value {
            "shell_api" => Some(Self::ShellApi),
            "external_terminal" => Some(Self::ExternalTerminal),
            _ => None,
        }
    }

    pub fn as_config_value(self) -> &'static str {
        match self {
            Self::ShellApi => "shell_api",
            Self::ExternalTerminal => "external_terminal",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ViewSettings {
    pub font_family: String,
    pub font_size: u16,
    pub theme: String,
    pub ui_language: String,
    pub window_width: Option<i32>,
    pub window_height: Option<i32>,
    pub tree_panel_width: Option<i32>,
}

impl ViewSettings {
    pub fn new(font_family: impl Into<String>, font_size: u16, theme: impl Into<String>) -> Self {
        let default = Self::default();
        let font_family = font_family.into();
        let theme = theme.into();

        Self {
            font_family: usable_view_text_or_default(font_family, default.font_family),
            font_size: normalize_ui_font_size(font_size),
            theme: ViewTheme::from_config_value(&theme)
                .unwrap_or_default()
                .as_config_value()
                .to_owned(),
            ui_language: default.ui_language,
            window_width: None,
            window_height: None,
            tree_panel_width: None,
        }
    }

    pub fn with_ui_language(mut self, ui_language: impl AsRef<str>) -> Self {
        self.ui_language = normalize_ui_language(ui_language);
        self
    }

    pub fn with_window_layout(
        mut self,
        window_width: Option<i32>,
        window_height: Option<i32>,
        tree_panel_width: Option<i32>,
    ) -> Self {
        self.window_width = positive_dimension(window_width);
        self.window_height = positive_dimension(window_height);
        self.tree_panel_width = positive_dimension(tree_panel_width);
        self
    }

    pub fn with_window_layout_from(mut self, other: &Self) -> Self {
        self.window_width = other.window_width;
        self.window_height = other.window_height;
        self.tree_panel_width = other.tree_panel_width;
        self
    }

    pub fn set_window_layout(
        &mut self,
        window_width: i32,
        window_height: i32,
        tree_panel_width: i32,
    ) {
        self.window_width = positive_dimension(Some(window_width));
        self.window_height = positive_dimension(Some(window_height));
        self.tree_panel_width = positive_dimension(Some(tree_panel_width));
    }
}

impl Default for ViewSettings {
    fn default() -> Self {
        Self {
            font_family: DEFAULT_FONT_FAMILY.to_owned(),
            font_size: DEFAULT_FONT_SIZE,
            theme: DEFAULT_THEME.to_owned(),
            ui_language: DEFAULT_UI_LANGUAGE.to_owned(),
            window_width: None,
            window_height: None,
            tree_panel_width: None,
        }
    }
}

pub fn normalize_ui_language(ui_language: impl AsRef<str>) -> String {
    UiLanguage::from_config_value(ui_language.as_ref())
        .unwrap_or_default()
        .as_config_value()
        .to_owned()
}

pub fn current_ui_language(view: &ViewSettings) -> UiLanguage {
    UiLanguage::from_config_value(&view.ui_language).unwrap_or_default()
}

pub fn normalize_ui_font_size(font_size: u16) -> u16 {
    font_size.clamp(MIN_UI_FONT_SIZE, MAX_UI_FONT_SIZE)
}

fn positive_dimension(value: Option<i32>) -> Option<i32> {
    value.filter(|value| *value > 0)
}

fn usable_view_text_or_default(value: String, default: String) -> String {
    let value = value.trim();
    if value.is_empty() {
        default
    } else {
        value.to_owned()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AppSettings {
    pub view: ViewSettings,
    pub languages: Vec<String>,
    pub tree_order: Vec<TreeRootItem>,
    pub categories: Vec<Category>,
    pub workspaces: Vec<Workspace>,
    pub command_tabs: Vec<CommandTab>,
}

impl AppSettings {
    pub fn root_tree_items(&self) -> Vec<TreeRootItemRef> {
        root_tree_items_from_order(&self.tree_order, &self.categories, &self.workspaces)
    }

    pub fn normalized_tree_order(&self) -> Vec<TreeRootItem> {
        self.root_tree_items()
            .into_iter()
            .filter_map(|item| match item {
                TreeRootItemRef::Category(index) => self
                    .categories
                    .get(index)
                    .map(|category| TreeRootItem::category(category.name.clone())),
                TreeRootItemRef::Workspace(index) => self
                    .workspaces
                    .get(index)
                    .map(|workspace| TreeRootItem::workspace(workspace.path.clone())),
            })
            .collect()
    }

    pub fn sync_tree_order(&mut self) {
        self.tree_order = self.normalized_tree_order();
    }
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            view: ViewSettings::default(),
            languages: default_workspace_language_options(),
            tree_order: Vec::new(),
            categories: Vec::new(),
            workspaces: Vec::new(),
            command_tabs: Vec::new(),
        }
    }
}

fn root_tree_items_from_order(
    tree_order: &[TreeRootItem],
    categories: &[Category],
    workspaces: &[Workspace],
) -> Vec<TreeRootItemRef> {
    let indexes = TreeRootIndexes::new(categories, workspaces);
    let mut root_items = Vec::with_capacity(categories.len() + indexes.root_workspace_count);
    let mut used_categories = vec![false; categories.len()];
    let mut used_workspaces = vec![false; workspaces.len()];

    for requested in tree_order {
        match requested {
            TreeRootItem::Category { name } => {
                if let Some(index) = category_index_by_name(
                    &indexes.category_indexes_by_name,
                    name,
                    used_categories.as_slice(),
                ) {
                    used_categories[index] = true;
                    root_items.push(TreeRootItemRef::Category(index));
                }
            }
            TreeRootItem::Workspace { path } => {
                if let Some(index) = root_workspace_index_by_path(
                    &indexes.root_workspace_indexes_by_path,
                    path,
                    used_workspaces.as_slice(),
                ) {
                    used_workspaces[index] = true;
                    root_items.push(TreeRootItemRef::Workspace(index));
                }
            }
        }
    }

    for (index, used) in used_categories.iter().copied().enumerate() {
        if !used {
            root_items.push(TreeRootItemRef::Category(index));
        }
    }

    for (index, workspace) in workspaces.iter().enumerate() {
        if !used_workspaces[index]
            && workspace_category_index(workspace, &indexes.category_indexes_by_name).is_none()
        {
            root_items.push(TreeRootItemRef::Workspace(index));
        }
    }

    root_items
}

struct TreeRootIndexes {
    category_indexes_by_name: HashMap<String, Vec<usize>>,
    root_workspace_indexes_by_path: HashMap<String, Vec<usize>>,
    root_workspace_count: usize,
}

impl TreeRootIndexes {
    fn new(categories: &[Category], workspaces: &[Workspace]) -> Self {
        let mut category_indexes_by_name: HashMap<String, Vec<usize>> = HashMap::new();
        for (index, category) in categories.iter().enumerate() {
            category_indexes_by_name
                .entry(category_name_lookup_key(&category.name))
                .or_default()
                .push(index);
        }

        let mut root_workspace_indexes_by_path: HashMap<String, Vec<usize>> = HashMap::new();
        let mut root_workspace_count = 0;
        for (index, workspace) in workspaces.iter().enumerate() {
            if workspace_category_index(workspace, &category_indexes_by_name).is_some() {
                continue;
            }

            root_workspace_indexes_by_path
                .entry(workspace_path_lookup_key(&workspace.path))
                .or_default()
                .push(index);
            root_workspace_count += 1;
        }

        Self {
            category_indexes_by_name,
            root_workspace_indexes_by_path,
            root_workspace_count,
        }
    }
}

fn category_index_by_name(
    category_indexes_by_name: &HashMap<String, Vec<usize>>,
    name: &str,
    used_categories: &[bool],
) -> Option<usize> {
    category_indexes_by_name
        .get(&category_name_lookup_key(name))
        .and_then(|indexes| {
            indexes
                .iter()
                .copied()
                .find(|index| !used_categories[*index])
        })
}

fn root_workspace_index_by_path(
    root_workspace_indexes_by_path: &HashMap<String, Vec<usize>>,
    path: &str,
    used_workspaces: &[bool],
) -> Option<usize> {
    root_workspace_indexes_by_path
        .get(&workspace_path_lookup_key(path))
        .and_then(|indexes| {
            indexes
                .iter()
                .copied()
                .find(|index| !used_workspaces[*index])
        })
}

fn workspace_category_index(
    workspace: &Workspace,
    category_indexes_by_name: &HashMap<String, Vec<usize>>,
) -> Option<usize> {
    let category = workspace.category.as_deref()?;
    category_indexes_by_name
        .get(&category_name_lookup_key(category))
        .and_then(|indexes| indexes.first().copied())
}

fn category_name_lookup_key(name: &str) -> String {
    name.trim().to_ascii_lowercase()
}

fn workspace_path_lookup_key(path: &str) -> String {
    comparable_workspace_path(path).to_ascii_lowercase()
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DomainValidationError {
    MissingWorkspaceLanguageOptions,
    MissingCategoryField(&'static str),
    MissingWorkspaceField(&'static str),
    MissingCommandTabField(&'static str),
    MissingCommandButtonField(&'static str),
    InteriorNulLanguageConfigField(&'static str),
    InteriorNulCategoryField(&'static str),
    InteriorNulWorkspaceField(&'static str),
    InteriorNulCommandTabField(&'static str),
    InteriorNulCommandButtonField(&'static str),
    DuplicateWorkspaceLanguage(String),
    UnsupportedWorkspaceLanguage(String),
    InvalidExecutionType(String),
}

impl DomainValidationError {
    pub fn user_message(&self) -> String {
        self.user_message_for_language(UiLanguage::Korean)
    }

    pub fn user_message_for_language(&self, language: UiLanguage) -> String {
        match self {
            Self::MissingWorkspaceLanguageOptions => language
                .text(
                    "언어를 하나 이상 입력하세요.",
                    "Enter at least one language.",
                )
                .to_owned(),
            Self::MissingCategoryField(field) => {
                format!(
                    "{}: {field}",
                    language.text("분류 값을 입력하세요", "Enter a category value")
                )
            }
            Self::MissingWorkspaceField(field) => {
                format!(
                    "{}: {field}",
                    language.text("워크스페이스 값을 입력하세요", "Enter a workspace value")
                )
            }
            Self::MissingCommandTabField(field) => {
                format!(
                    "{}: {field}",
                    language.text("명령 그룹 값을 입력하세요", "Enter a command group value")
                )
            }
            Self::MissingCommandButtonField(field) => {
                format!(
                    "{}: {field}",
                    language.text("명령 값을 입력하세요", "Enter a command value")
                )
            }
            Self::InteriorNulLanguageConfigField(field) => {
                format!(
                    "{}: {field}",
                    language.text(
                        "언어에 사용할 수 없는 문자가 있습니다",
                        "The language contains an unsupported character"
                    )
                )
            }
            Self::InteriorNulCategoryField(field) => {
                format!(
                    "{}: {field}",
                    language.text(
                        "분류 값에 사용할 수 없는 문자가 있습니다",
                        "The category value contains an unsupported character"
                    )
                )
            }
            Self::InteriorNulWorkspaceField(field) => {
                format!(
                    "{}: {field}",
                    language.text(
                        "워크스페이스 값에 사용할 수 없는 문자가 있습니다",
                        "The workspace value contains an unsupported character"
                    )
                )
            }
            Self::InteriorNulCommandTabField(field) => {
                format!(
                    "{}: {field}",
                    language.text(
                        "명령 그룹 값에 사용할 수 없는 문자가 있습니다",
                        "The command group value contains an unsupported character"
                    )
                )
            }
            Self::InteriorNulCommandButtonField(field) => {
                format!(
                    "{}: {field}",
                    language.text(
                        "명령 값에 사용할 수 없는 문자가 있습니다",
                        "The command value contains an unsupported character"
                    )
                )
            }
            Self::DuplicateWorkspaceLanguage(value) => {
                format!(
                    "{}: {value}",
                    language.text("이미 있는 언어입니다", "This language already exists")
                )
            }
            Self::UnsupportedWorkspaceLanguage(value) => {
                format!(
                    "{}: {value}",
                    language.text("목록에 없는 언어입니다", "This language is not in the list")
                )
            }
            Self::InvalidExecutionType(value) => {
                format!(
                    "{}: {value}",
                    language.text(
                        "지원하지 않는 실행 방식입니다",
                        "Unsupported execution type"
                    )
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_button_rejects_interior_nul_in_execution_strings() {
        assert_command_button_interior_nul_error(
            CommandButton::new("Run\0Hidden", "tool.exe", "", ExecutionType::ShellApi),
            "button_name",
        );
        assert_command_button_interior_nul_error(
            CommandButton::new("Run", "tool.exe\0evil.exe", "", ExecutionType::ShellApi),
            "executable_path",
        );
        assert_command_button_interior_nul_error(
            CommandButton::new("Run", "tool.exe", "--safe\0--evil", ExecutionType::ShellApi),
            "arguments",
        );
    }

    fn assert_command_button_interior_nul_error(
        result: Result<CommandButton, DomainValidationError>,
        expected_field: &'static str,
    ) {
        match result {
            Err(DomainValidationError::InteriorNulCommandButtonField(field)) => {
                assert_eq!(field, expected_field);
            }
            other => panic!("unexpected command button validation result: {other:?}"),
        }
    }

    #[test]
    fn display_names_reject_interior_nul() {
        assert_domain_interior_nul_field(
            Workspace::new("C:\\projects\\api", "api\0hidden", "Rust"),
            |error| match error {
                DomainValidationError::InteriorNulWorkspaceField(field) => Some(field),
                _ => None,
            },
        );
        assert_domain_interior_nul_field(Category::new("Backend\0Hidden"), |error| match error {
            DomainValidationError::InteriorNulCategoryField(field) => Some(field),
            _ => None,
        });
        assert_domain_interior_nul_field(CommandTab::new("Tools\0Hidden", Vec::new()), |error| {
            match error {
                DomainValidationError::InteriorNulCommandTabField(field) => Some(field),
                _ => None,
            }
        });
    }

    fn assert_domain_interior_nul_field<T>(
        result: Result<T, DomainValidationError>,
        field_from_error: impl FnOnce(DomainValidationError) -> Option<&'static str>,
    ) {
        match result {
            Err(error) => assert_eq!(field_from_error(error), Some("name")),
            Ok(_) => panic!("interior NUL validation unexpectedly accepted value"),
        }
    }

    #[test]
    fn workspace_paths_follow_windows_case_insensitive_comparison() {
        assert!(workspace_paths_equal(
            " C:\\Projects\\Api\\ ",
            "c:\\projects\\api"
        ));
        assert!(workspace_paths_equal(
            "/tmp/J3DevHelper/",
            "/tmp/j3devhelper"
        ));
        assert!(!workspace_paths_equal(
            "/tmp/j3devhelper",
            "/tmp/j3devhelper2"
        ));
    }

    #[test]
    fn root_tree_items_uses_normalized_order_keys_without_losing_duplicates() {
        let settings = AppSettings {
            tree_order: vec![
                TreeRootItem::workspace(" C:\\Projects\\Api "),
                TreeRootItem::category(" backend "),
                TreeRootItem::workspace("C:\\Projects\\Api\\"),
                TreeRootItem::category("BACKEND"),
            ],
            categories: vec![test_category("Backend"), test_category("backend")],
            workspaces: vec![
                test_workspace("C:\\Projects\\Api", "api", None),
                test_workspace("C:\\Projects\\Api", "api duplicate", None),
                test_workspace("C:\\Projects\\Ui", "ui", Some("Backend")),
            ],
            ..AppSettings::default()
        };

        assert_eq!(
            settings.root_tree_items(),
            vec![
                TreeRootItemRef::Workspace(0),
                TreeRootItemRef::Category(0),
                TreeRootItemRef::Workspace(1),
                TreeRootItemRef::Category(1),
            ]
        );
    }

    fn test_category(name: &str) -> Category {
        Category {
            name: name.to_owned(),
        }
    }

    fn test_workspace(path: &str, name: &str, category: Option<&str>) -> Workspace {
        Workspace {
            path: path.to_owned(),
            name: name.to_owned(),
            language: DEFAULT_WORKSPACE_LANGUAGE.to_owned(),
            category: category.map(ToOwned::to_owned),
        }
    }
}
