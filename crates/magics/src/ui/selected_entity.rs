use bevy::{input::common_conditions::input_just_pressed, prelude::*};

use crate::planner::FactorGraph;

pub struct SelectedEntityPlugin;

impl Plugin for SelectedEntityPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<DeselectEntityEvent>()
            .init_resource::<SelectedEntity>()
            .add_systems(
                Update,
                (
                    // render_hover_window.run_if(resource_equals(SelectedEntity(Some(..)))),
                    render_hover_window.run_if(not(resource_equals(SelectedEntity(None)))),
                    // unselect_entity.run_if(input_just_pressed(KeyCode::Escape)),
                    unselect_entity.run_if(on_event::<DeselectEntityEvent>()),
                ),
            );
    }
}

#[derive(Debug, Resource, Deref, DerefMut, Default, PartialEq, Eq)]
pub struct SelectedEntity(Option<SelectableEntity>);

impl SelectedEntity {
    #[inline]
    fn deselect(&mut self) {
        self.0 = None;
    }

    #[inline]
    fn select(&mut self, selectable: SelectableEntity) {
        self.0 = Some(selectable);
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum SelectableEntity {
    Robot(Entity),
    VariableVisualiser(Entity),
}
fn render_hover_window(query_robot: Query<(Entity, &FactorGraph)>) {}

#[derive(Event)]
pub struct DeselectEntityEvent;

fn unselect_entity(mut selected: ResMut<SelectedEntity>) {
    if selected.is_none() {
        return;
    }
    debug!("deselecting entity: {:?}", selected);
    selected.deselect();
}
