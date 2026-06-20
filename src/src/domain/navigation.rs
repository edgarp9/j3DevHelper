use super::core::{
    Category, TreeRootItemRef, Workspace, category_names_equal, direct_reorder_drop_destination,
    indexed_reorder_drop_destination,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorkspaceTreeDropTarget {
    Workspace { index: usize, insert_after: bool },
    Category { index: usize },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorkspaceTreeDropAction {
    MoveWorkspace { destination_index: usize },
    MoveRootItem { destination_index: usize },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TreeKeyboardMoveDirection {
    Up,
    Down,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommandTabMoveDirection {
    Left,
    Right,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommandButtonMoveDirection {
    Previous,
    Next,
}

pub fn workspace_belongs_to_category(workspace: &Workspace, category: &Category) -> bool {
    workspace
        .category
        .as_deref()
        .is_some_and(|name| category_names_equal(name, &category.name))
}

pub fn workspace_category_index(workspace: &Workspace, categories: &[Category]) -> Option<usize> {
    let category = workspace.category.as_deref()?;
    categories
        .iter()
        .position(|candidate| category_names_equal(category, &candidate.name))
}

pub fn workspace_keyboard_move_destination(
    workspaces: &[Workspace],
    categories: &[Category],
    source_index: usize,
    direction: TreeKeyboardMoveDirection,
) -> Option<usize> {
    let source = workspaces.get(source_index)?;
    let source_category_index = workspace_category_index(source, categories);
    let same_visible_group = |workspace: &Workspace| {
        workspace_category_index(workspace, categories) == source_category_index
    };

    match direction {
        TreeKeyboardMoveDirection::Up => workspaces
            .iter()
            .enumerate()
            .take(source_index)
            .rev()
            .find(|(_, workspace)| same_visible_group(workspace))
            .map(|(index, _)| index),
        TreeKeyboardMoveDirection::Down => workspaces
            .iter()
            .enumerate()
            .skip(source_index.saturating_add(1))
            .find(|(_, workspace)| same_visible_group(workspace))
            .map(|(index, _)| index),
    }
}

pub fn tree_root_keyboard_move_destination(
    root_items: &[TreeRootItemRef],
    source: TreeRootItemRef,
    direction: TreeKeyboardMoveDirection,
) -> Option<usize> {
    let source_index = root_items.iter().position(|item| *item == source)?;
    match direction {
        TreeKeyboardMoveDirection::Up => source_index.checked_sub(1),
        TreeKeyboardMoveDirection::Down => source_index
            .checked_add(1)
            .filter(|destination| *destination < root_items.len()),
    }
}

pub fn workspace_tree_drop_destination(
    source_index: usize,
    target: WorkspaceTreeDropTarget,
    len: usize,
) -> Option<usize> {
    let WorkspaceTreeDropTarget::Workspace {
        index: target_index,
        insert_after,
    } = target
    else {
        return None;
    };

    indexed_reorder_drop_destination(source_index, target_index, insert_after, len)
}

pub fn workspace_tree_visible_group_drop_destination(
    workspaces: &[Workspace],
    categories: &[Category],
    source_index: usize,
    target: WorkspaceTreeDropTarget,
) -> Option<usize> {
    let WorkspaceTreeDropTarget::Workspace {
        index: target_index,
        ..
    } = target
    else {
        return None;
    };

    let source = workspaces.get(source_index)?;
    let target_workspace = workspaces.get(target_index)?;
    if workspace_category_index(source, categories)
        != workspace_category_index(target_workspace, categories)
    {
        return None;
    }

    workspace_tree_drop_destination(source_index, target, workspaces.len())
}

pub fn workspace_tree_drop_action(
    workspaces: &[Workspace],
    categories: &[Category],
    root_items: &[TreeRootItemRef],
    source_index: usize,
    target: WorkspaceTreeDropTarget,
) -> Option<WorkspaceTreeDropAction> {
    let WorkspaceTreeDropTarget::Workspace {
        index: target_index,
        insert_after,
    } = target
    else {
        return None;
    };

    let source = workspaces.get(source_index)?;
    let target_workspace = workspaces.get(target_index)?;
    let source_category = workspace_category_index(source, categories);
    let target_category = workspace_category_index(target_workspace, categories);

    match (source_category, target_category) {
        (None, None) => tree_root_drop_destination(
            root_items,
            TreeRootItemRef::Workspace(source_index),
            TreeRootItemRef::Workspace(target_index),
            insert_after,
        )
        .map(|destination_index| WorkspaceTreeDropAction::MoveRootItem { destination_index }),
        (Some(source_category), Some(target_category)) if source_category == target_category => {
            workspace_tree_visible_group_drop_destination(
                workspaces,
                categories,
                source_index,
                target,
            )
            .map(|destination_index| WorkspaceTreeDropAction::MoveWorkspace { destination_index })
        }
        _ => None,
    }
}

pub fn tree_root_drop_destination(
    root_items: &[TreeRootItemRef],
    source: TreeRootItemRef,
    target: TreeRootItemRef,
    insert_after: bool,
) -> Option<usize> {
    let source_index = root_items.iter().position(|item| *item == source)?;
    let target_index = root_items.iter().position(|item| *item == target)?;
    indexed_reorder_drop_destination(source_index, target_index, insert_after, root_items.len())
}

pub fn command_button_drop_destination(
    source_index: usize,
    target_index: usize,
    len: usize,
) -> Option<usize> {
    direct_reorder_drop_destination(source_index, target_index, len)
}

pub fn command_button_move_destination(
    index: usize,
    len: usize,
    direction: CommandButtonMoveDirection,
) -> Option<usize> {
    if index >= len || len <= 1 {
        return None;
    }

    match direction {
        CommandButtonMoveDirection::Previous => index.checked_sub(1),
        CommandButtonMoveDirection::Next => (index + 1 < len).then_some(index + 1),
    }
}

pub fn command_tab_move_destination(
    index: usize,
    len: usize,
    direction: CommandTabMoveDirection,
) -> Option<usize> {
    if index >= len || len <= 1 {
        return None;
    }

    match direction {
        CommandTabMoveDirection::Left => index.checked_sub(1),
        CommandTabMoveDirection::Right => (index + 1 < len).then_some(index + 1),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn workspace_category_index_requires_visible_category_match() {
        let categories = vec![Category::new("Backend").expect("category should be valid")];
        let matched_workspace = Workspace::new("C:\\projects\\api", "api", "Rust")
            .expect("workspace should be valid")
            .with_category(Some("backend".to_owned()));
        let stale_workspace = Workspace::new("C:\\projects\\old", "old", "Rust")
            .expect("workspace should be valid")
            .with_category(Some("Archived".to_owned()));
        let root_workspace = Workspace::new("C:\\projects\\root", "root", "Rust")
            .expect("workspace should be valid");

        assert_eq!(
            workspace_category_index(&matched_workspace, &categories),
            Some(0)
        );
        assert_eq!(
            workspace_category_index(&stale_workspace, &categories),
            None
        );
        assert_eq!(workspace_category_index(&root_workspace, &categories), None);
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
            workspace_keyboard_move_destination(
                &workspaces,
                &categories,
                2,
                TreeKeyboardMoveDirection::Up,
            ),
            Some(0)
        );
        assert_eq!(
            workspace_keyboard_move_destination(
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
            workspace_keyboard_move_destination(
                &workspaces,
                &categories,
                0,
                TreeKeyboardMoveDirection::Up,
            ),
            None
        );
        assert_eq!(
            workspace_keyboard_move_destination(
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
            tree_root_keyboard_move_destination(
                &root_items,
                TreeRootItemRef::Workspace(0),
                TreeKeyboardMoveDirection::Up,
            ),
            Some(0),
        );
        assert_eq!(
            tree_root_keyboard_move_destination(
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
            tree_root_keyboard_move_destination(
                &root_items,
                TreeRootItemRef::Category(0),
                TreeKeyboardMoveDirection::Up,
            ),
            None,
        );
        assert_eq!(
            tree_root_keyboard_move_destination(
                &root_items,
                TreeRootItemRef::Workspace(0),
                TreeKeyboardMoveDirection::Down,
            ),
            None,
        );
        assert_eq!(
            tree_root_keyboard_move_destination(
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
            tree_root_drop_destination(
                &root_items,
                TreeRootItemRef::Workspace(0),
                TreeRootItemRef::Category(1),
                false,
            ),
            Some(2),
        );
        assert_eq!(
            tree_root_drop_destination(
                &root_items,
                TreeRootItemRef::Category(1),
                TreeRootItemRef::Workspace(0),
                true,
            ),
            Some(2),
        );
    }

    #[test]
    fn workspace_tree_drop_destination_accounts_for_removed_source() {
        assert_eq!(
            workspace_tree_drop_destination(
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
            workspace_tree_drop_destination(
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
    fn workspace_tree_drop_destination_ignores_noop_and_category_targets() {
        assert_eq!(
            workspace_tree_drop_destination(
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
            workspace_tree_drop_destination(
                1,
                WorkspaceTreeDropTarget::Workspace {
                    index: 1,
                    insert_after: true,
                },
                4,
            ),
            None
        );
        assert_eq!(
            workspace_tree_drop_destination(0, WorkspaceTreeDropTarget::Category { index: 0 }, 2),
            None
        );
    }

    #[test]
    fn workspace_tree_visible_group_drop_destination_matches_only_same_visible_group() {
        let categories = vec![
            Category::new("Backend").expect("category should be valid"),
            Category::new("Frontend").expect("category should be valid"),
        ];
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
            Workspace::new("C:\\projects\\web", "web", "Rust")
                .expect("workspace should be valid")
                .with_category(Some("Frontend".to_owned())),
        ];

        assert_eq!(
            workspace_tree_visible_group_drop_destination(
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
            workspace_tree_visible_group_drop_destination(
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
        assert_eq!(
            workspace_tree_visible_group_drop_destination(
                &workspaces,
                &categories,
                0,
                WorkspaceTreeDropTarget::Workspace {
                    index: 4,
                    insert_after: true,
                },
            ),
            None
        );
        assert_eq!(
            workspace_tree_visible_group_drop_destination(
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
    }

    #[test]
    fn workspace_tree_drop_action_distinguishes_root_and_group_moves() {
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
        let root_items = vec![
            TreeRootItemRef::Category(0),
            TreeRootItemRef::Workspace(1),
            TreeRootItemRef::Workspace(3),
        ];

        assert_eq!(
            workspace_tree_drop_action(
                &workspaces,
                &categories,
                &root_items,
                1,
                WorkspaceTreeDropTarget::Workspace {
                    index: 3,
                    insert_after: true,
                },
            ),
            Some(WorkspaceTreeDropAction::MoveRootItem {
                destination_index: 2
            })
        );
        assert_eq!(
            workspace_tree_drop_action(
                &workspaces,
                &categories,
                &root_items,
                0,
                WorkspaceTreeDropTarget::Workspace {
                    index: 2,
                    insert_after: true,
                },
            ),
            Some(WorkspaceTreeDropAction::MoveWorkspace {
                destination_index: 2
            })
        );
        assert_eq!(
            workspace_tree_drop_action(
                &workspaces,
                &categories,
                &root_items,
                0,
                WorkspaceTreeDropTarget::Workspace {
                    index: 1,
                    insert_after: false,
                },
            ),
            None
        );
    }

    #[test]
    fn command_button_drop_destination_uses_target_position() {
        assert_eq!(command_button_drop_destination(0, 2, 4), Some(2));
        assert_eq!(command_button_drop_destination(3, 1, 4), Some(1));
    }

    #[test]
    fn command_button_drop_destination_ignores_invalid_or_same_targets() {
        assert_eq!(command_button_drop_destination(1, 1, 4), None);
        assert_eq!(command_button_drop_destination(4, 1, 4), None);
        assert_eq!(command_button_drop_destination(1, 4, 4), None);
        assert_eq!(command_button_drop_destination(0, 1, 1), None);
    }

    #[test]
    fn command_button_move_destination_moves_one_step_vertically() {
        assert_eq!(
            command_button_move_destination(1, 3, CommandButtonMoveDirection::Previous),
            Some(0)
        );
        assert_eq!(
            command_button_move_destination(1, 3, CommandButtonMoveDirection::Next),
            Some(2)
        );
    }

    #[test]
    fn command_button_move_destination_ignores_edges_and_invalid_indices() {
        assert_eq!(
            command_button_move_destination(0, 3, CommandButtonMoveDirection::Previous),
            None
        );
        assert_eq!(
            command_button_move_destination(2, 3, CommandButtonMoveDirection::Next),
            None
        );
        assert_eq!(
            command_button_move_destination(3, 3, CommandButtonMoveDirection::Previous),
            None
        );
        assert_eq!(
            command_button_move_destination(0, 1, CommandButtonMoveDirection::Next),
            None
        );
    }

    #[test]
    fn command_tab_move_destination_moves_one_step_horizontally() {
        assert_eq!(
            command_tab_move_destination(1, 3, CommandTabMoveDirection::Left),
            Some(0)
        );
        assert_eq!(
            command_tab_move_destination(1, 3, CommandTabMoveDirection::Right),
            Some(2)
        );
    }

    #[test]
    fn command_tab_move_destination_ignores_edges_and_invalid_indices() {
        assert_eq!(
            command_tab_move_destination(0, 3, CommandTabMoveDirection::Left),
            None
        );
        assert_eq!(
            command_tab_move_destination(2, 3, CommandTabMoveDirection::Right),
            None
        );
        assert_eq!(
            command_tab_move_destination(3, 3, CommandTabMoveDirection::Left),
            None
        );
        assert_eq!(
            command_tab_move_destination(0, 1, CommandTabMoveDirection::Right),
            None
        );
    }
}
