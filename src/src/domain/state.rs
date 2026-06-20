use super::core::{
    AppSettings, Category, CommandButton, CommandTab, DomainValidationError, TreeRootItem,
    TreeRootItemRef, Workspace, category_names_equal, current_ui_language,
    normalize_workspace_language, normalize_workspace_language_options, workspace_paths_equal,
};
use super::localization::UiLanguage;

pub const INITIAL_STATUS_MESSAGE: &str = "준비됨";
pub const INITIAL_STATUS_MESSAGE_EN: &str = "Ready";

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorkspaceMutationError {
    DuplicatePath(String),
    InvalidIndex(usize),
    InvalidCategoryIndex(usize),
    InvalidWorkspace(DomainValidationError),
}

impl WorkspaceMutationError {
    pub fn user_message(&self) -> String {
        self.user_message_for_language(UiLanguage::Korean)
    }

    pub fn user_message_for_language(&self, language: UiLanguage) -> String {
        match self {
            Self::DuplicatePath(path) => {
                format!(
                    "{}: {path}",
                    language.text(
                        "이미 등록된 폴더입니다",
                        "This folder is already registered"
                    )
                )
            }
            Self::InvalidIndex(index) => {
                format!(
                    "{}: {index}",
                    language.text("워크스페이스를 찾을 수 없습니다", "Workspace not found")
                )
            }
            Self::InvalidCategoryIndex(index) => {
                format!(
                    "{}: {index}",
                    language.text("분류를 찾을 수 없습니다", "Category not found")
                )
            }
            Self::InvalidWorkspace(error) => error.user_message_for_language(language),
        }
    }
}

impl From<DomainValidationError> for WorkspaceMutationError {
    fn from(error: DomainValidationError) -> Self {
        Self::InvalidWorkspace(error)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LanguageConfigMutationError {
    InvalidConfig(DomainValidationError),
    WorkspaceLanguageInUse {
        workspace_name: String,
        language: String,
    },
}

impl LanguageConfigMutationError {
    pub fn user_message(&self) -> String {
        self.user_message_for_language(UiLanguage::Korean)
    }

    pub fn user_message_for_language(&self, language: UiLanguage) -> String {
        match self {
            Self::InvalidConfig(error) => error.user_message_for_language(language),
            Self::WorkspaceLanguageInUse {
                workspace_name,
                language: used_language,
            } => format!(
                "{}: {workspace_name} ({used_language})",
                language.text(
                    "사용 중인 언어는 삭제할 수 없습니다",
                    "Cannot remove a language currently in use"
                )
            ),
        }
    }
}

impl From<DomainValidationError> for LanguageConfigMutationError {
    fn from(error: DomainValidationError) -> Self {
        Self::InvalidConfig(error)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CategoryMutationError {
    DuplicateName(String),
    InvalidIndex(usize),
    InvalidCategory(DomainValidationError),
}

impl CategoryMutationError {
    pub fn user_message(&self) -> String {
        self.user_message_for_language(UiLanguage::Korean)
    }

    pub fn user_message_for_language(&self, language: UiLanguage) -> String {
        match self {
            Self::DuplicateName(name) => {
                format!(
                    "{}: {name}",
                    language.text("이미 있는 분류입니다", "This category already exists")
                )
            }
            Self::InvalidIndex(index) => {
                format!(
                    "{}: {index}",
                    language.text("분류를 찾을 수 없습니다", "Category not found")
                )
            }
            Self::InvalidCategory(error) => error.user_message_for_language(language),
        }
    }
}

impl From<DomainValidationError> for CategoryMutationError {
    fn from(error: DomainValidationError) -> Self {
        Self::InvalidCategory(error)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TreeRootMutationError {
    InvalidSource,
    InvalidDestination(usize),
}

impl TreeRootMutationError {
    pub fn user_message(&self) -> String {
        self.user_message_for_language(UiLanguage::Korean)
    }

    pub fn user_message_for_language(&self, language: UiLanguage) -> String {
        match self {
            Self::InvalidSource => language
                .text("항목을 찾을 수 없습니다.", "Item not found.")
                .to_owned(),
            Self::InvalidDestination(index) => {
                format!(
                    "{}: {index}",
                    language.text("이동할 수 없습니다", "Cannot move to this position")
                )
            }
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CommandTabMutationError {
    InvalidIndex(usize),
    InvalidTab(DomainValidationError),
}

impl CommandTabMutationError {
    pub fn user_message(&self) -> String {
        self.user_message_for_language(UiLanguage::Korean)
    }

    pub fn user_message_for_language(&self, language: UiLanguage) -> String {
        match self {
            Self::InvalidIndex(index) => {
                format!(
                    "{}: {index}",
                    language.text("명령 그룹을 찾을 수 없습니다", "Command group not found")
                )
            }
            Self::InvalidTab(error) => error.user_message_for_language(language),
        }
    }
}

impl From<DomainValidationError> for CommandTabMutationError {
    fn from(error: DomainValidationError) -> Self {
        Self::InvalidTab(error)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CommandButtonMutationError {
    InvalidTabIndex(usize),
    InvalidButtonIndex(usize),
    InvalidButton(DomainValidationError),
}

impl CommandButtonMutationError {
    pub fn user_message(&self) -> String {
        self.user_message_for_language(UiLanguage::Korean)
    }

    pub fn user_message_for_language(&self, language: UiLanguage) -> String {
        match self {
            Self::InvalidTabIndex(index) => {
                format!(
                    "{}: {index}",
                    language.text("명령 그룹을 찾을 수 없습니다", "Command group not found")
                )
            }
            Self::InvalidButtonIndex(index) => {
                format!(
                    "{}: {index}",
                    language.text("명령을 찾을 수 없습니다", "Command not found")
                )
            }
            Self::InvalidButton(error) => error.user_message_for_language(language),
        }
    }
}

impl From<DomainValidationError> for CommandButtonMutationError {
    fn from(error: DomainValidationError) -> Self {
        Self::InvalidButton(error)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AppState {
    status_message: String,
    settings: AppSettings,
    restore_warnings: Vec<String>,
    selected_workspace_index: Option<usize>,
    selected_command_tab_index: Option<usize>,
    selected_command_button_index: Option<usize>,
}

impl AppState {
    pub fn initial() -> Self {
        Self::from_settings(AppSettings::default(), Vec::new())
    }

    pub fn from_settings(settings: AppSettings, restore_warnings: Vec<String>) -> Self {
        let mut settings = settings;
        settings.sync_tree_order();

        let language = current_ui_language(&settings.view);
        let status_message = if restore_warnings.is_empty() {
            initial_status_message(language).to_owned()
        } else if restore_warnings.len() == 1 {
            restore_warnings[0].clone()
        } else {
            match language {
                UiLanguage::Korean => format!(
                    "설정 경고 {}건: {}",
                    restore_warnings.len(),
                    restore_warnings[0]
                ),
                UiLanguage::English => format!(
                    "{} settings warnings: {}",
                    restore_warnings.len(),
                    restore_warnings[0]
                ),
            }
        };

        let selected_command_tab_index = if settings.command_tabs.is_empty() {
            None
        } else {
            Some(0)
        };

        Self {
            status_message,
            settings,
            restore_warnings,
            selected_workspace_index: None,
            selected_command_tab_index,
            selected_command_button_index: None,
        }
    }

    pub fn status_message(&self) -> &str {
        &self.status_message
    }

    pub fn settings(&self) -> &AppSettings {
        &self.settings
    }

    pub fn set_view_settings(&mut self, view: super::core::ViewSettings) {
        self.settings.view = view;
    }

    pub fn set_workspace_languages(
        &mut self,
        languages: Vec<String>,
    ) -> Result<(), LanguageConfigMutationError> {
        let languages = normalize_workspace_language_options(languages)?;
        let mut normalized_workspace_languages = Vec::with_capacity(self.settings.workspaces.len());

        for workspace in &self.settings.workspaces {
            let Some(language) = normalize_workspace_language(&workspace.language, &languages)
            else {
                return Err(LanguageConfigMutationError::WorkspaceLanguageInUse {
                    workspace_name: workspace.name.clone(),
                    language: workspace.language.clone(),
                });
            };
            normalized_workspace_languages.push(language);
        }

        self.settings.languages = languages;
        for (workspace, language) in self
            .settings
            .workspaces
            .iter_mut()
            .zip(normalized_workspace_languages)
        {
            workspace.language = language;
        }

        Ok(())
    }

    pub fn add_restore_warning(&mut self, warning: impl Into<String>) {
        let warning = warning.into();
        self.restore_warnings.push(warning.clone());
        self.status_message = warning;
    }

    pub fn selected_workspace_index(&self) -> Option<usize> {
        self.selected_workspace_index
    }

    pub fn selected_workspace(&self) -> Option<&Workspace> {
        self.selected_workspace_index
            .and_then(|index| self.settings.workspaces.get(index))
    }

    pub fn selected_command_tab_index(&self) -> Option<usize> {
        self.selected_command_tab_index
    }

    pub fn selected_command_tab(&self) -> Option<&CommandTab> {
        self.selected_command_tab_index
            .and_then(|index| self.settings.command_tabs.get(index))
    }

    pub fn selected_command_button_index(&self) -> Option<usize> {
        self.selected_command_button_index
    }

    pub fn selected_command_button(&self) -> Option<&CommandButton> {
        let tab = self.selected_command_tab()?;
        self.selected_command_button_index
            .and_then(|index| tab.buttons.get(index))
    }

    pub fn select_workspace(&mut self, index: Option<usize>) {
        self.selected_workspace_index =
            index.filter(|index| *index < self.settings.workspaces.len());
    }

    pub fn select_command_tab(&mut self, index: Option<usize>) {
        let previous = self.selected_command_tab_index;
        self.selected_command_tab_index =
            index.filter(|index| *index < self.settings.command_tabs.len());
        if previous != self.selected_command_tab_index {
            self.selected_command_button_index = None;
        } else {
            self.clamp_selected_command_button();
        }
    }

    pub fn select_command_button(&mut self, index: Option<usize>) {
        let Some(tab_index) = self.selected_command_tab_index else {
            self.selected_command_button_index = None;
            return;
        };

        let Some(tab) = self.settings.command_tabs.get(tab_index) else {
            self.selected_command_tab_index = None;
            self.selected_command_button_index = None;
            return;
        };

        self.selected_command_button_index = index.filter(|index| *index < tab.buttons.len());
    }

    pub fn add_workspace(
        &mut self,
        mut workspace: Workspace,
    ) -> Result<usize, WorkspaceMutationError> {
        if self.workspace_path_exists(&workspace.path, None) {
            return Err(WorkspaceMutationError::DuplicatePath(workspace.path));
        }

        workspace.language =
            normalize_workspace_language(&workspace.language, &self.settings.languages)
                .ok_or_else(|| {
                    WorkspaceMutationError::InvalidWorkspace(
                        DomainValidationError::UnsupportedWorkspaceLanguage(
                            workspace.language.clone(),
                        ),
                    )
                })?;

        self.settings.workspaces.push(workspace);
        self.settings.sync_tree_order();
        let index = self.settings.workspaces.len() - 1;
        self.selected_workspace_index = Some(index);
        Ok(index)
    }

    pub fn add_category(&mut self, category: Category) -> Result<usize, CategoryMutationError> {
        if self.category_name_exists(&category.name) {
            return Err(CategoryMutationError::DuplicateName(category.name));
        }

        self.settings.categories.push(category);
        self.settings.sync_tree_order();
        self.selected_workspace_index = None;
        Ok(self.settings.categories.len() - 1)
    }

    pub fn rename_category(
        &mut self,
        index: usize,
        category: Category,
    ) -> Result<(), CategoryMutationError> {
        if index >= self.settings.categories.len() {
            return Err(CategoryMutationError::InvalidIndex(index));
        }
        if self.category_name_exists_except(&category.name, Some(index)) {
            return Err(CategoryMutationError::DuplicateName(category.name));
        }

        let previous_name = self.settings.categories[index].name.clone();
        let next_name = category.name;

        for item in &mut self.settings.tree_order {
            if let TreeRootItem::Category { name } = item
                && category_names_equal(name, &previous_name)
            {
                *name = next_name.clone();
            }
        }

        for workspace in &mut self.settings.workspaces {
            if workspace
                .category
                .as_deref()
                .is_some_and(|name| category_names_equal(name, &previous_name))
            {
                workspace.set_category(Some(next_name.clone()));
            }
        }

        self.settings.categories[index].name = next_name;
        self.settings.sync_tree_order();
        self.selected_workspace_index = None;
        Ok(())
    }

    pub fn move_category(
        &mut self,
        index: usize,
        destination_index: usize,
    ) -> Result<(), CategoryMutationError> {
        let len = self.settings.categories.len();
        if index >= len {
            return Err(CategoryMutationError::InvalidIndex(index));
        }
        if destination_index >= len {
            return Err(CategoryMutationError::InvalidIndex(destination_index));
        }
        if index == destination_index {
            return Ok(());
        }

        let category = self.settings.categories.remove(index);
        self.settings.categories.insert(destination_index, category);
        self.settings.sync_tree_order();
        Ok(())
    }

    pub fn delete_category(&mut self, index: usize) -> Option<Category> {
        if index >= self.settings.categories.len() {
            return None;
        }

        let removed_name = self.settings.categories[index].name.clone();
        let root_order_before_delete = self.settings.normalized_tree_order();
        let released_workspace_paths = self
            .settings
            .workspaces
            .iter()
            .filter(|workspace| {
                workspace
                    .category
                    .as_deref()
                    .is_some_and(|name| category_names_equal(name, &removed_name))
            })
            .map(|workspace| workspace.path.clone())
            .collect::<Vec<_>>();

        let removed = self.settings.categories.remove(index);
        for workspace in &mut self.settings.workspaces {
            if workspace
                .category
                .as_deref()
                .is_some_and(|name| category_names_equal(name, &removed_name))
            {
                workspace.set_category(None);
            }
        }

        let mut next_tree_order =
            Vec::with_capacity(root_order_before_delete.len() + released_workspace_paths.len());
        let mut inserted_released_workspaces = false;
        for item in root_order_before_delete {
            if matches!(&item, TreeRootItem::Category { name } if category_names_equal(name, &removed_name))
            {
                next_tree_order.extend(
                    released_workspace_paths
                        .iter()
                        .cloned()
                        .map(TreeRootItem::workspace),
                );
                inserted_released_workspaces = true;
            } else {
                next_tree_order.push(item);
            }
        }
        if !inserted_released_workspaces {
            next_tree_order.extend(
                released_workspace_paths
                    .iter()
                    .cloned()
                    .map(TreeRootItem::workspace),
            );
        }

        self.settings.tree_order = next_tree_order;
        self.settings.sync_tree_order();
        self.selected_workspace_index = None;
        Some(removed)
    }

    pub fn move_root_tree_item(
        &mut self,
        source: TreeRootItemRef,
        destination_index: usize,
    ) -> Result<(), TreeRootMutationError> {
        self.settings.sync_tree_order();
        let root_items = self.settings.root_tree_items();
        let len = root_items.len();
        let Some(source_index) = root_items.iter().position(|item| *item == source) else {
            return Err(TreeRootMutationError::InvalidSource);
        };
        if destination_index >= len {
            return Err(TreeRootMutationError::InvalidDestination(destination_index));
        }
        if source_index == destination_index {
            select_tree_root_source(&mut self.selected_workspace_index, source);
            return Ok(());
        }

        let mut tree_order = self.settings.normalized_tree_order();
        let item = tree_order.remove(source_index);
        tree_order.insert(destination_index, item);
        self.settings.tree_order = tree_order;
        select_tree_root_source(&mut self.selected_workspace_index, source);
        Ok(())
    }

    pub fn update_workspace(
        &mut self,
        index: usize,
        mut workspace: Workspace,
    ) -> Result<(), WorkspaceMutationError> {
        if index >= self.settings.workspaces.len() {
            return Err(WorkspaceMutationError::InvalidIndex(index));
        }

        if self.workspace_path_exists(&workspace.path, Some(index)) {
            return Err(WorkspaceMutationError::DuplicatePath(workspace.path));
        }

        workspace.language =
            normalize_workspace_language(&workspace.language, &self.settings.languages)
                .ok_or_else(|| {
                    DomainValidationError::UnsupportedWorkspaceLanguage(workspace.language.clone())
                })?;
        let previous_path = self.settings.workspaces[index].path.clone();
        workspace.category = self.settings.workspaces[index].category.clone();
        self.replace_workspace_tree_order_path(&previous_path, &workspace.path);
        self.settings.workspaces[index] = workspace;
        self.settings.sync_tree_order();
        self.selected_workspace_index = Some(index);
        Ok(())
    }

    pub fn delete_workspace(&mut self, index: usize) -> Option<Workspace> {
        if index >= self.settings.workspaces.len() {
            return None;
        }

        let removed = self.settings.workspaces.remove(index);
        self.settings.sync_tree_order();
        self.selected_workspace_index = None;
        Some(removed)
    }

    pub fn move_workspace(
        &mut self,
        index: usize,
        destination_index: usize,
    ) -> Result<(), WorkspaceMutationError> {
        let len = self.settings.workspaces.len();
        if index >= len {
            return Err(WorkspaceMutationError::InvalidIndex(index));
        }
        if destination_index >= len {
            return Err(WorkspaceMutationError::InvalidIndex(destination_index));
        }
        if index == destination_index {
            self.selected_workspace_index = Some(index);
            return Ok(());
        }

        let workspace = self.settings.workspaces.remove(index);
        self.settings
            .workspaces
            .insert(destination_index, workspace);
        self.settings.sync_tree_order();
        self.selected_workspace_index = Some(destination_index);
        Ok(())
    }

    pub fn move_workspace_to_category(
        &mut self,
        index: usize,
        category_index: usize,
    ) -> Result<(), WorkspaceMutationError> {
        if index >= self.settings.workspaces.len() {
            return Err(WorkspaceMutationError::InvalidIndex(index));
        }

        let Some(category) = self.settings.categories.get(category_index) else {
            return Err(WorkspaceMutationError::InvalidCategoryIndex(category_index));
        };

        self.settings.workspaces[index].set_category(Some(category.name.clone()));
        self.settings.sync_tree_order();
        self.selected_workspace_index = Some(index);
        Ok(())
    }

    pub fn move_workspace_to_root(&mut self, index: usize) -> Result<(), WorkspaceMutationError> {
        if index >= self.settings.workspaces.len() {
            return Err(WorkspaceMutationError::InvalidIndex(index));
        }

        self.settings.workspaces[index].set_category(None);
        self.settings.sync_tree_order();
        self.selected_workspace_index = Some(index);
        Ok(())
    }

    pub fn add_command_tab(&mut self, tab: CommandTab) -> Result<usize, CommandTabMutationError> {
        self.settings.command_tabs.push(tab);
        let index = self.settings.command_tabs.len() - 1;
        self.selected_command_tab_index = Some(index);
        self.selected_command_button_index = None;
        Ok(index)
    }

    pub fn rename_command_tab(
        &mut self,
        index: usize,
        name: impl Into<String>,
    ) -> Result<(), CommandTabMutationError> {
        let Some(existing) = self.settings.command_tabs.get_mut(index) else {
            return Err(CommandTabMutationError::InvalidIndex(index));
        };

        let name = name.into();
        if name.trim().is_empty() {
            return Err(DomainValidationError::MissingCommandTabField("name").into());
        }

        existing.name = name.trim().to_owned();
        self.selected_command_tab_index = Some(index);
        Ok(())
    }

    pub fn move_command_tab(
        &mut self,
        index: usize,
        destination_index: usize,
    ) -> Result<(), CommandTabMutationError> {
        let len = self.settings.command_tabs.len();
        if index >= len {
            return Err(CommandTabMutationError::InvalidIndex(index));
        }
        if destination_index >= len {
            return Err(CommandTabMutationError::InvalidIndex(destination_index));
        }
        if index == destination_index {
            self.selected_command_tab_index = Some(index);
            return Ok(());
        }

        let tab = self.settings.command_tabs.remove(index);
        self.settings.command_tabs.insert(destination_index, tab);
        self.selected_command_tab_index = Some(destination_index);
        self.selected_command_button_index = None;
        Ok(())
    }

    pub fn delete_command_tab(&mut self, index: usize) -> Option<CommandTab> {
        if index >= self.settings.command_tabs.len() {
            return None;
        }

        let removed = self.settings.command_tabs.remove(index);
        self.selected_command_tab_index = if self.settings.command_tabs.is_empty() {
            None
        } else if index < self.settings.command_tabs.len() {
            Some(index)
        } else {
            Some(self.settings.command_tabs.len() - 1)
        };
        self.selected_command_button_index = None;
        Some(removed)
    }

    pub fn add_command_button(
        &mut self,
        tab_index: usize,
        button: CommandButton,
    ) -> Result<usize, CommandButtonMutationError> {
        let Some(tab) = self.settings.command_tabs.get_mut(tab_index) else {
            return Err(CommandButtonMutationError::InvalidTabIndex(tab_index));
        };

        tab.buttons.push(button);
        let button_index = tab.buttons.len() - 1;
        self.selected_command_tab_index = Some(tab_index);
        self.selected_command_button_index = Some(button_index);
        Ok(button_index)
    }

    pub fn command_button_matches(
        &self,
        tab_index: usize,
        button_index: usize,
        button: &CommandButton,
    ) -> Result<bool, CommandButtonMutationError> {
        let Some(tab) = self.settings.command_tabs.get(tab_index) else {
            return Err(CommandButtonMutationError::InvalidTabIndex(tab_index));
        };

        let Some(existing) = tab.buttons.get(button_index) else {
            return Err(CommandButtonMutationError::InvalidButtonIndex(button_index));
        };

        Ok(existing.button_name == button.button_name
            && existing.executable_path == button.executable_path
            && existing.arguments == button.arguments
            && existing.execution_type.as_config_value() == button.execution_type.as_config_value())
    }

    pub fn update_command_button(
        &mut self,
        tab_index: usize,
        button_index: usize,
        button: CommandButton,
    ) -> Result<(), CommandButtonMutationError> {
        let Some(tab) = self.settings.command_tabs.get_mut(tab_index) else {
            return Err(CommandButtonMutationError::InvalidTabIndex(tab_index));
        };

        let Some(existing) = tab.buttons.get_mut(button_index) else {
            return Err(CommandButtonMutationError::InvalidButtonIndex(button_index));
        };

        *existing = button;
        self.selected_command_tab_index = Some(tab_index);
        self.selected_command_button_index = Some(button_index);
        Ok(())
    }

    pub fn move_command_button(
        &mut self,
        tab_index: usize,
        button_index: usize,
        destination_index: usize,
    ) -> Result<(), CommandButtonMutationError> {
        let Some(tab) = self.settings.command_tabs.get_mut(tab_index) else {
            return Err(CommandButtonMutationError::InvalidTabIndex(tab_index));
        };

        let len = tab.buttons.len();
        if button_index >= len {
            return Err(CommandButtonMutationError::InvalidButtonIndex(button_index));
        }
        if destination_index >= len {
            return Err(CommandButtonMutationError::InvalidButtonIndex(
                destination_index,
            ));
        }
        if button_index == destination_index {
            self.selected_command_tab_index = Some(tab_index);
            self.selected_command_button_index = Some(button_index);
            return Ok(());
        }

        let button = tab.buttons.remove(button_index);
        tab.buttons.insert(destination_index, button);
        self.selected_command_tab_index = Some(tab_index);
        self.selected_command_button_index = Some(destination_index);
        Ok(())
    }

    pub fn delete_command_button(
        &mut self,
        tab_index: usize,
        button_index: usize,
    ) -> Option<CommandButton> {
        let tab = self.settings.command_tabs.get_mut(tab_index)?;
        if button_index >= tab.buttons.len() {
            return None;
        }

        let removed = tab.buttons.remove(button_index);
        self.selected_command_tab_index = Some(tab_index);
        self.selected_command_button_index = if tab.buttons.is_empty() {
            None
        } else if button_index < tab.buttons.len() {
            Some(button_index)
        } else {
            Some(tab.buttons.len() - 1)
        };
        Some(removed)
    }

    pub fn set_status_message(&mut self, message: impl Into<String>) {
        self.status_message = message.into();
    }

    pub fn restore_warnings(&self) -> &[String] {
        &self.restore_warnings
    }

    fn workspace_path_exists(&self, path: &str, except_index: Option<usize>) -> bool {
        self.settings
            .workspaces
            .iter()
            .enumerate()
            .any(|(index, workspace)| {
                Some(index) != except_index && workspace_paths_equal(&workspace.path, path)
            })
    }

    fn category_name_exists(&self, name: &str) -> bool {
        self.category_name_exists_except(name, None)
    }

    fn category_name_exists_except(&self, name: &str, except_index: Option<usize>) -> bool {
        self.settings
            .categories
            .iter()
            .enumerate()
            .any(|(index, category)| {
                Some(index) != except_index && category_names_equal(&category.name, name)
            })
    }

    fn replace_workspace_tree_order_path(&mut self, previous_path: &str, next_path: &str) {
        for item in &mut self.settings.tree_order {
            if let TreeRootItem::Workspace { path } = item
                && workspace_paths_equal(path, previous_path)
            {
                *path = next_path.to_owned();
            }
        }
    }

    fn clamp_selected_command_button(&mut self) {
        let Some(tab_index) = self.selected_command_tab_index else {
            self.selected_command_button_index = None;
            return;
        };

        let Some(tab) = self.settings.command_tabs.get(tab_index) else {
            self.selected_command_tab_index = None;
            self.selected_command_button_index = None;
            return;
        };

        self.selected_command_button_index = self
            .selected_command_button_index
            .filter(|index| *index < tab.buttons.len());
    }
}

fn initial_status_message(language: UiLanguage) -> &'static str {
    match language {
        UiLanguage::Korean => INITIAL_STATUS_MESSAGE,
        UiLanguage::English => INITIAL_STATUS_MESSAGE_EN,
    }
}

fn select_tree_root_source(selection: &mut Option<usize>, source: TreeRootItemRef) {
    *selection = match source {
        TreeRootItemRef::Workspace(index) => Some(index),
        TreeRootItemRef::Category(_) => None,
    };
}

#[cfg(test)]
mod tests {
    use super::super::core::ExecutionType;
    use super::*;

    fn command_button(arguments: &str) -> CommandButton {
        match CommandButton::new(
            "Build".to_owned(),
            "cargo".to_owned(),
            arguments.to_owned(),
            ExecutionType::ShellApi,
        ) {
            Ok(button) => button,
            Err(_) => panic!("valid command button fixture rejected"),
        }
    }

    fn state_with_button(button: CommandButton) -> AppState {
        let mut settings = AppSettings::default();
        settings.command_tabs.push(CommandTab {
            name: "General".to_owned(),
            buttons: vec![button],
        });
        AppState::from_settings(settings, Vec::new())
    }

    #[test]
    fn command_button_matches_reports_same_values_without_selection_change() {
        let button = command_button("check");
        let state = state_with_button(button.clone());

        assert_eq!(state.command_button_matches(0, 0, &button), Ok(true));
        assert_eq!(state.selected_command_tab_index(), Some(0));
        assert_eq!(state.selected_command_button_index(), None);
    }

    #[test]
    fn command_button_matches_reports_changed_values() {
        let state = state_with_button(command_button("check"));
        let changed = command_button("test");

        assert_eq!(state.command_button_matches(0, 0, &changed), Ok(false));
    }

    #[test]
    fn add_category_rejects_duplicate_names_and_clears_workspace_selection() {
        let mut state = AppState::initial();
        let workspace = Workspace::new("C:\\projects\\demo", "demo", "Rust")
            .expect("workspace should be valid");
        state
            .add_workspace(workspace)
            .expect("workspace should be added");
        assert_eq!(state.selected_workspace_index(), Some(0));

        let first = Category::new("Tools").expect("category should be valid");
        assert_eq!(state.add_category(first), Ok(0));
        assert_eq!(state.selected_workspace_index(), None);

        let duplicate = Category::new("tools").expect("category should be valid");
        assert_eq!(
            state.add_category(duplicate),
            Err(CategoryMutationError::DuplicateName("tools".to_owned()))
        );
    }

    #[test]
    fn rename_category_updates_tree_order_and_workspace_memberships() {
        let mut state = AppState::initial();
        state
            .add_category(Category::new("Backend").expect("category should be valid"))
            .expect("category should be added");
        state
            .add_workspace(
                Workspace::new("C:\\projects\\api", "api", "Rust")
                    .expect("workspace should be valid"),
            )
            .expect("workspace should be added");
        state
            .move_workspace_to_category(0, 0)
            .expect("workspace should move into category");
        state
            .add_category(Category::new("Frontend").expect("category should be valid"))
            .expect("category should be added");
        state
            .move_root_tree_item(TreeRootItemRef::Category(1), 0)
            .expect("category should move in root tree order");

        state
            .rename_category(
                0,
                Category::new("Services").expect("category should be valid"),
            )
            .expect("category should be renamed");

        assert_eq!(state.settings().categories[0].name, "Services");
        assert_eq!(
            state.settings().workspaces[0].category.as_deref(),
            Some("Services")
        );
        assert_eq!(
            state.settings().tree_order.as_slice(),
            &[
                TreeRootItem::category("Frontend"),
                TreeRootItem::category("Services"),
            ]
        );
        assert_eq!(state.selected_workspace_index(), None);
    }

    #[test]
    fn rename_category_rejects_duplicate_names() {
        let mut state = AppState::initial();
        state
            .add_category(Category::new("Backend").expect("category should be valid"))
            .expect("category should be added");
        state
            .add_category(Category::new("Tools").expect("category should be valid"))
            .expect("category should be added");

        assert_eq!(
            state.rename_category(
                1,
                Category::new("backend").expect("category should be valid")
            ),
            Err(CategoryMutationError::DuplicateName("backend".to_owned()))
        );
    }

    #[test]
    fn categories_can_be_reordered() {
        let mut state = AppState::initial();
        for name in ["Backend", "Frontend", "Tools"] {
            state
                .add_category(Category::new(name).expect("category should be valid"))
                .expect("category should be added");
        }

        state.move_category(2, 0).expect("category should be moved");

        let names = state
            .settings()
            .categories
            .iter()
            .map(|category| category.name.as_str())
            .collect::<Vec<_>>();
        assert_eq!(names, vec!["Tools", "Backend", "Frontend"]);
    }

    #[test]
    fn move_category_rejects_invalid_indices() {
        let mut state = AppState::initial();
        state
            .add_category(Category::new("Backend").expect("category should be valid"))
            .expect("category should be added");

        assert_eq!(
            state.move_category(1, 0),
            Err(CategoryMutationError::InvalidIndex(1))
        );
        assert_eq!(
            state.move_category(0, 1),
            Err(CategoryMutationError::InvalidIndex(1))
        );
    }

    #[test]
    fn delete_category_releases_workspaces_and_preserves_tree_position() {
        let mut state = AppState::initial();
        state
            .add_category(Category::new("Backend").expect("category should be valid"))
            .expect("category should be added");
        state
            .add_workspace(
                Workspace::new("C:\\projects\\api", "api", "Rust")
                    .expect("workspace should be valid"),
            )
            .expect("workspace should be added");
        state
            .move_workspace_to_category(0, 0)
            .expect("workspace should move into category");
        state
            .add_workspace(
                Workspace::new("C:\\projects\\cli", "cli", "Rust")
                    .expect("workspace should be valid"),
            )
            .expect("workspace should be added");
        state
            .add_category(Category::new("Tools").expect("category should be valid"))
            .expect("category should be added");
        state
            .move_root_tree_item(TreeRootItemRef::Workspace(1), 0)
            .expect("workspace should move before category");

        let removed = state
            .delete_category(0)
            .expect("category should be deleted");

        assert_eq!(removed.name, "Backend");
        assert_eq!(state.settings().categories[0].name, "Tools");
        assert_eq!(state.settings().workspaces[0].category, None);
        assert_eq!(state.settings().workspaces[1].category, None);
        assert_eq!(
            state.settings().tree_order.as_slice(),
            &[
                TreeRootItem::workspace("C:\\projects\\cli"),
                TreeRootItem::workspace("C:\\projects\\api"),
                TreeRootItem::category("Tools"),
            ]
        );
        assert_eq!(state.selected_workspace_index(), None);
    }

    #[test]
    fn delete_category_rejects_invalid_index() {
        let mut state = AppState::initial();
        state
            .add_category(Category::new("Backend").expect("category should be valid"))
            .expect("category should be added");
        let previous = state.clone();

        assert_eq!(state.delete_category(1), None);
        assert_eq!(state, previous);
    }

    #[test]
    fn tree_root_order_can_mix_categories_and_root_workspaces() {
        let mut state = AppState::initial();
        state
            .add_category(Category::new("Backend").expect("category should be valid"))
            .expect("category should be added");
        state
            .add_workspace(
                Workspace::new("C:\\projects\\cli", "cli", "Rust")
                    .expect("workspace should be valid"),
            )
            .expect("workspace should be added");
        state
            .add_workspace(
                Workspace::new("C:\\projects\\tools", "tools", "Rust")
                    .expect("workspace should be valid"),
            )
            .expect("workspace should be added");
        state
            .add_category(Category::new("Frontend").expect("category should be valid"))
            .expect("category should be added");

        state
            .move_root_tree_item(TreeRootItemRef::Category(1), 0)
            .expect("category should move in root tree order");

        assert_eq!(
            state.settings().tree_order.as_slice(),
            &[
                TreeRootItem::category("Frontend"),
                TreeRootItem::category("Backend"),
                TreeRootItem::workspace("C:\\projects\\cli"),
                TreeRootItem::workspace("C:\\projects\\tools"),
            ]
        );

        state
            .move_root_tree_item(TreeRootItemRef::Workspace(1), 1)
            .expect("workspace should move in root tree order");

        assert_eq!(
            state.settings().tree_order.as_slice(),
            &[
                TreeRootItem::category("Frontend"),
                TreeRootItem::workspace("C:\\projects\\tools"),
                TreeRootItem::category("Backend"),
                TreeRootItem::workspace("C:\\projects\\cli"),
            ]
        );
        assert_eq!(state.selected_workspace_index(), Some(1));
    }

    #[test]
    fn workspaces_can_be_reordered_and_keep_selection_on_destination() {
        let mut state = AppState::initial();
        for name in ["alpha", "beta", "gamma"] {
            let workspace = Workspace::new(format!("C:\\projects\\{name}"), name, "Rust")
                .expect("workspace should be valid");
            state
                .add_workspace(workspace)
                .expect("workspace should be added");
        }

        state
            .move_workspace(2, 0)
            .expect("workspace should be moved");

        let names = state
            .settings()
            .workspaces
            .iter()
            .map(|workspace| workspace.name.as_str())
            .collect::<Vec<_>>();
        assert_eq!(names, vec!["gamma", "alpha", "beta"]);
        assert_eq!(state.selected_workspace_index(), Some(0));
    }

    #[test]
    fn move_workspace_rejects_invalid_indices() {
        let mut state = AppState::initial();
        let workspace = Workspace::new("C:\\projects\\alpha", "alpha", "Rust")
            .expect("workspace should be valid");
        state
            .add_workspace(workspace)
            .expect("workspace should be added");

        assert_eq!(
            state.move_workspace(1, 0),
            Err(WorkspaceMutationError::InvalidIndex(1))
        );
        assert_eq!(
            state.move_workspace(0, 1),
            Err(WorkspaceMutationError::InvalidIndex(1))
        );
        assert_eq!(state.selected_workspace_index(), Some(0));
    }

    #[test]
    fn workspace_can_be_moved_to_category_without_changing_workspace_index() {
        let mut state = AppState::initial();
        let category = Category::new("Backend").expect("category should be valid");
        state
            .add_category(category)
            .expect("category should be added");
        let workspace =
            Workspace::new("C:\\projects\\api", "api", "Rust").expect("workspace should be valid");
        state
            .add_workspace(workspace)
            .expect("workspace should be added");

        state
            .move_workspace_to_category(0, 0)
            .expect("workspace should move into category");

        assert_eq!(
            state.settings().workspaces[0].category.as_deref(),
            Some("Backend")
        );
        assert_eq!(state.selected_workspace_index(), Some(0));
    }

    #[test]
    fn workspace_can_be_moved_to_root_from_category() {
        let mut state = AppState::initial();
        state
            .add_category(Category::new("Backend").expect("category should be valid"))
            .expect("category should be added");
        state
            .add_workspace(
                Workspace::new("C:\\projects\\api", "api", "Rust")
                    .expect("workspace should be valid"),
            )
            .expect("workspace should be added");
        state
            .move_workspace_to_category(0, 0)
            .expect("workspace should move into category");

        state
            .move_workspace_to_root(0)
            .expect("workspace should move to root");

        assert_eq!(state.settings().workspaces[0].category, None);
        assert_eq!(state.selected_workspace_index(), Some(0));
    }

    #[test]
    fn move_workspace_to_root_rejects_invalid_index() {
        let mut state = AppState::initial();

        assert_eq!(
            state.move_workspace_to_root(0),
            Err(WorkspaceMutationError::InvalidIndex(0))
        );
    }

    #[test]
    fn update_workspace_preserves_category_membership() {
        let mut state = AppState::initial();
        state
            .add_category(Category::new("Backend").expect("category should be valid"))
            .expect("category should be added");
        state
            .add_workspace(
                Workspace::new("C:\\projects\\api", "api", "Rust")
                    .expect("workspace should be valid"),
            )
            .expect("workspace should be added");
        state
            .move_workspace_to_category(0, 0)
            .expect("workspace should move into category");

        state
            .update_workspace(
                0,
                Workspace::new("C:\\projects\\api", "api-renamed", "Rust")
                    .expect("workspace should be valid"),
            )
            .expect("workspace should be updated");

        assert_eq!(state.settings().workspaces[0].name, "api-renamed");
        assert_eq!(
            state.settings().workspaces[0].category.as_deref(),
            Some("Backend")
        );
    }

    #[test]
    fn language_config_rejects_removing_languages_used_by_workspaces() {
        let mut state = AppState::initial();
        state
            .add_workspace(
                Workspace::new("C:\\projects\\api", "api", "Rust")
                    .expect("workspace should be valid"),
            )
            .expect("workspace should be added");

        assert_eq!(
            state.set_workspace_languages(vec!["Java".to_owned()]),
            Err(LanguageConfigMutationError::WorkspaceLanguageInUse {
                workspace_name: "api".to_owned(),
                language: "Rust".to_owned(),
            })
        );
    }

    #[test]
    fn language_config_case_change_updates_existing_workspaces() {
        let mut state = AppState::initial();
        state
            .add_workspace(
                Workspace::new("C:\\projects\\api", "api", "Rust")
                    .expect("workspace should be valid"),
            )
            .expect("workspace should be added");

        state
            .set_workspace_languages(vec!["rust".to_owned(), "java".to_owned()])
            .expect("case-only language config change should be accepted");

        assert_eq!(state.settings().languages, vec!["rust", "java"]);
        assert_eq!(state.settings().workspaces[0].language, "rust");
    }
}
