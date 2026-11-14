use crate::model::{Pane, Tool, Workspace};
use egui_dock::{DockState, NodeIndex};
#[derive(Clone, Copy)]
pub enum Preset {
    Columns,
    Rows,
    Grid,
}

pub fn workspace_view(workspace: &Workspace, expanded: bool) -> DockState<Pane> {
    let mut panes: Vec<_> = workspace
        .dock
        .iter_all_tabs()
        .map(|(_, pane)| pane.clone())
        .collect();
    let primary = workspace
        .terminals
        .iter()
        .find(|terminal| terminal.tool == Tool::OpenCode)
        .and_then(|terminal| {
            panes
                .iter()
                .position(|pane| *pane == Pane::Terminal(terminal.id.clone()))
        })
        .or_else(|| {
            panes
                .iter()
                .position(|pane| matches!(pane, Pane::Terminal(_)))
        });
    let Some(primary) = primary else {
        return DockState::new(panes);
    };
    let mut dock = DockState::new(vec![panes.remove(primary)]);
    if panes.is_empty() {
        return dock;
    }
    // Keep secondary shells ahead of previews when sharing one tab strip.
    panes.sort_by_key(|pane| matches!(pane, Pane::Image(_)));
    let previews = if expanded {
        Vec::new()
    } else {
        let first = panes.iter().position(|pane| matches!(pane, Pane::Image(_)));
        match first {
            Some(index) if index > 0 => panes.split_off(index),
            _ => Vec::new(),
        }
    };
    let tree = dock.main_surface_mut();
    let [_, right] = tree.split_right(NodeIndex::root(), 0.67, panes);
    if !previews.is_empty() {
        tree.split_below(right, 0.48, previews);
    }
    dock
}
pub fn preset(panes: Vec<Pane>, kind: Preset) -> DockState<Pane> {
    if panes.is_empty() {
        return DockState::new(panes);
    }
    let count = match kind {
        Preset::Grid => 4,
        _ => 2,
    }
    .min(panes.len());
    let mut groups = vec![Vec::new(); count];
    for (index, pane) in panes.into_iter().enumerate() {
        groups[index % count].push(pane);
    }
    let mut dock = DockState::new(groups.remove(0));
    let tree = dock.main_surface_mut();
    if count >= 2 {
        let sides = match kind {
            Preset::Rows => tree.split_below(NodeIndex::root(), 0.5, groups.remove(0)),
            _ => tree.split_right(NodeIndex::root(), 0.5, groups.remove(0)),
        };
        if count >= 3 {
            tree.split_below(sides[0], 0.5, groups.remove(0));
        }
        if count >= 4 {
            tree.split_below(sides[1], 0.5, groups.remove(0));
        }
    }
    dock
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn workspace_modes_keep_every_tab_and_group_the_preview_only_when_expanded() {
        let mut workspace = Workspace::new("Work".into(), "Z:/project".into());
        let primary = Pane::Terminal("a".into());
        let secondary = Pane::Terminal("b".into());
        let preview = Pane::Image("layout.png".into());
        workspace.dock = DockState::new(vec![primary.clone(), secondary.clone(), preview.clone()]);
        for expanded in [false, true] {
            let dock = workspace_view(&workspace, expanded);
            assert_eq!(dock.iter_all_tabs().count(), 3);
            let (primary_node, secondary_node, preview_node) = (
                dock.find_tab(&primary).unwrap().node,
                dock.find_tab(&secondary).unwrap().node,
                dock.find_tab(&preview).unwrap().node,
            );
            assert_ne!(primary_node, secondary_node);
            assert_eq!(secondary_node == preview_node, expanded);
        }
    }
    #[test]
    fn presets_preserve_every_existing_pane_including_overflow_tabs() {
        let original: Vec<_> = (0..7).map(|i| Pane::Terminal(i.to_string())).collect();
        for kind in [Preset::Columns, Preset::Rows, Preset::Grid] {
            let dock = preset(original.clone(), kind);
            let mut actual: Vec<_> = dock.iter_all_tabs().map(|(_, p)| p.clone()).collect();
            actual.sort_by_key(|p| format!("{p:?}"));
            assert_eq!(actual, original);
            assert_eq!(dock.surfaces_count(), 1);
        }
    }
    #[test]
    fn proportions_and_all_panes_survive_state_roundtrip() {
        let mut workspace = crate::model::Workspace::new("Work".into(), "Z:/project".into());
        workspace.dock = preset(
            vec![Pane::Terminal("a".into()), Pane::Terminal("b".into())],
            Preset::Columns,
        );
        let json = serde_json::to_vec(&workspace).unwrap();
        let restored: crate::model::Workspace = serde_json::from_slice(&json).unwrap();
        assert_eq!(restored.dock.iter_all_tabs().count(), 2);
    }
}
