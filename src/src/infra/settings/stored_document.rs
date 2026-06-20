use serde::{Deserialize, Serialize};

use crate::domain::{
    AppSettings, Category, CommandButton, CommandTab, TreeRootItem, ViewSettings, Workspace,
};

pub(super) fn parse_settings(content: &str) -> Result<SettingsDocument, toml::de::Error> {
    toml::from_str::<SettingsDocument>(content)
}

pub(super) fn serialize_settings(settings: &AppSettings) -> Result<String, toml::ser::Error> {
    let document = StoredSettingsDocument::from(settings);
    toml::to_string_pretty(&document)
}

#[derive(Debug, Deserialize)]
pub(super) struct SettingsDocument {
    #[serde(default)]
    pub(super) view: ViewDocument,
    pub(super) languages: Option<toml::Value>,
    #[serde(default)]
    pub(super) tree_order: Vec<TreeRootItemDocument>,
    #[serde(default)]
    pub(super) categories: Vec<CategoryDocument>,
    #[serde(default)]
    pub(super) workspaces: Vec<WorkspaceDocument>,
    #[serde(default)]
    pub(super) command_tabs: Vec<CommandTabDocument>,
}

#[derive(Debug, Default, Deserialize)]
pub(super) struct ViewDocument {
    pub(super) font_family: Option<String>,
    pub(super) font_size: Option<toml::Value>,
    pub(super) theme: Option<String>,
    pub(super) ui_language: Option<String>,
    pub(super) window_width: Option<toml::Value>,
    pub(super) window_height: Option<toml::Value>,
    pub(super) tree_panel_width: Option<toml::Value>,
}

#[derive(Debug, Deserialize)]
pub(super) struct CategoryDocument {
    pub(super) name: Option<toml::Value>,
}

#[derive(Debug, Deserialize)]
pub(super) struct TreeRootItemDocument {
    #[serde(rename = "type")]
    pub(super) item_type: Option<toml::Value>,
    pub(super) name: Option<toml::Value>,
    pub(super) path: Option<toml::Value>,
}

#[derive(Debug, Deserialize)]
pub(super) struct WorkspaceDocument {
    pub(super) path: Option<toml::Value>,
    pub(super) name: Option<toml::Value>,
    #[serde(rename = "Language")]
    pub(super) language: Option<toml::Value>,
    pub(super) category: Option<toml::Value>,
}

#[derive(Debug, Deserialize)]
pub(super) struct CommandTabDocument {
    pub(super) name: Option<toml::Value>,
    #[serde(default)]
    pub(super) buttons: Vec<CommandButtonDocument>,
}

#[derive(Debug, Deserialize)]
pub(super) struct CommandButtonDocument {
    pub(super) button_name: Option<toml::Value>,
    pub(super) executable_path: Option<toml::Value>,
    pub(super) arguments: Option<toml::Value>,
    pub(super) execution_type: Option<toml::Value>,
}

#[derive(Debug, Serialize)]
struct StoredSettingsDocument {
    view: StoredViewSettings,
    languages: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    tree_order: Vec<StoredTreeRootItem>,
    categories: Vec<StoredCategory>,
    workspaces: Vec<StoredWorkspace>,
    command_tabs: Vec<StoredCommandTab>,
}

impl From<&AppSettings> for StoredSettingsDocument {
    fn from(settings: &AppSettings) -> Self {
        Self {
            view: StoredViewSettings::from(&settings.view),
            languages: settings.languages.clone(),
            tree_order: stored_tree_order(settings),
            categories: settings
                .categories
                .iter()
                .map(StoredCategory::from)
                .collect(),
            workspaces: settings
                .workspaces
                .iter()
                .map(StoredWorkspace::from)
                .collect(),
            command_tabs: settings
                .command_tabs
                .iter()
                .map(StoredCommandTab::from)
                .collect(),
        }
    }
}

fn stored_tree_order(settings: &AppSettings) -> Vec<StoredTreeRootItem> {
    if settings.tree_order.is_empty() {
        Vec::new()
    } else {
        settings
            .normalized_tree_order()
            .iter()
            .map(StoredTreeRootItem::from)
            .collect()
    }
}

#[derive(Debug, Serialize)]
struct StoredViewSettings {
    font_family: String,
    font_size: u16,
    theme: String,
    ui_language: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    window_width: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    window_height: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tree_panel_width: Option<i32>,
}

impl From<&ViewSettings> for StoredViewSettings {
    fn from(view: &ViewSettings) -> Self {
        Self {
            font_family: view.font_family.clone(),
            font_size: view.font_size,
            theme: view.theme.clone(),
            ui_language: view.ui_language.clone(),
            window_width: view.window_width,
            window_height: view.window_height,
            tree_panel_width: view.tree_panel_width,
        }
    }
}

#[derive(Debug, Serialize)]
struct StoredCategory {
    name: String,
}

impl From<&Category> for StoredCategory {
    fn from(category: &Category) -> Self {
        Self {
            name: category.name.clone(),
        }
    }
}

#[derive(Debug, Serialize)]
struct StoredTreeRootItem {
    #[serde(rename = "type")]
    item_type: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    path: Option<String>,
}

impl From<&TreeRootItem> for StoredTreeRootItem {
    fn from(item: &TreeRootItem) -> Self {
        match item {
            TreeRootItem::Category { name } => Self {
                item_type: "category",
                name: Some(name.clone()),
                path: None,
            },
            TreeRootItem::Workspace { path } => Self {
                item_type: "workspace",
                name: None,
                path: Some(path.clone()),
            },
        }
    }
}

#[derive(Debug, Serialize)]
struct StoredWorkspace {
    path: String,
    name: String,
    #[serde(rename = "Language")]
    language: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    category: Option<String>,
}

impl From<&Workspace> for StoredWorkspace {
    fn from(workspace: &Workspace) -> Self {
        Self {
            path: workspace.path.clone(),
            name: workspace.name.clone(),
            language: workspace.language.clone(),
            category: workspace.category.clone(),
        }
    }
}

#[derive(Debug, Serialize)]
struct StoredCommandTab {
    name: String,
    buttons: Vec<StoredCommandButton>,
}

impl From<&CommandTab> for StoredCommandTab {
    fn from(tab: &CommandTab) -> Self {
        Self {
            name: tab.name.clone(),
            buttons: tab.buttons.iter().map(StoredCommandButton::from).collect(),
        }
    }
}

#[derive(Debug, Serialize)]
struct StoredCommandButton {
    button_name: String,
    executable_path: String,
    arguments: String,
    execution_type: &'static str,
}

impl From<&CommandButton> for StoredCommandButton {
    fn from(button: &CommandButton) -> Self {
        Self {
            button_name: button.button_name.clone(),
            executable_path: button.executable_path.clone(),
            arguments: button.arguments.clone(),
            execution_type: button.execution_type.as_config_value(),
        }
    }
}
