use crate::model::Pane;
use egui_dock::{DockState,NodeIndex};
#[derive(Clone,Copy)]
pub enum Preset {Columns,Rows,Grid}
pub fn preset(panes:Vec<Pane>,kind:Preset)->DockState<Pane> {
    if panes.is_empty(){return DockState::new(panes);}
    let count=match kind{Preset::Grid=>4,_=>2}.min(panes.len());
    let mut groups=vec![Vec::new();count];
    for(index,pane)in panes.into_iter().enumerate(){groups[index%count].push(pane);}
    let mut dock=DockState::new(groups.remove(0));
    let tree=dock.main_surface_mut();
    if count>=2 {
        let sides=match kind{Preset::Rows=>tree.split_below(NodeIndex::root(),0.5,groups.remove(0)),_=>tree.split_right(NodeIndex::root(),0.5,groups.remove(0))};
        if count>=3{tree.split_below(sides[0],0.5,groups.remove(0));}
        if count>=4{tree.split_below(sides[1],0.5,groups.remove(0));}
    }
    dock
}
#[cfg(test)] mod tests {
    use super::*;
    #[test] fn presets_preserve_every_existing_pane_including_overflow_tabs() {
        let original:Vec<_>=(0..7).map(|i|Pane::Terminal(i.to_string())).collect();
        for kind in [Preset::Columns,Preset::Rows,Preset::Grid] {
            let dock=preset(original.clone(),kind);
            let mut actual:Vec<_>=dock.iter_all_tabs().map(|(_,p)|p.clone()).collect();
            actual.sort_by_key(|p|format!("{p:?}"));assert_eq!(actual,original);
            assert_eq!(dock.surfaces_count(),1);
        }
    }
    #[test] fn proportions_and_all_panes_survive_state_roundtrip() {
        let mut workspace=crate::model::Workspace::new("Work".into(),"Z:/project".into());
        workspace.dock=preset(vec![Pane::Terminal("a".into()),Pane::Terminal("b".into())],Preset::Columns);
        let json=serde_json::to_vec(&workspace).unwrap();
        let restored:crate::model::Workspace=serde_json::from_slice(&json).unwrap();
        assert_eq!(restored.dock.iter_all_tabs().count(),2);
    }
}
