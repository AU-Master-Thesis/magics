use bevy::prelude::*;

pub struct GoalAreaPlugin;

impl Plugin for GoalAreaPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<events::GoalAreaReached>()
            .add_systems(Startup, setup_goal_areas_for_junction_scenario)
            .add_systems(FixedUpdate, detect_collisions);
        //.add_systems(
        //    Update,
        //    (
        //        render_goal_areas,
        //         log_collisions
        //    ),
        //);
    }
}

pub mod components {
    use super::*;

    #[derive(Component)]
    pub struct GoalArea {
        pub aabb: parry2d::bounding_volume::Aabb,
        pub(super) history: std::collections::HashMap<Entity, f32>,
        pub(super) shape: parry2d::shape::Cuboid,
    }

    impl GoalArea {
        pub fn new(aabb: parry2d::bounding_volume::Aabb) -> Self {
            let half_extents = aabb.half_extents();
            let shape = parry2d::shape::Cuboid::new(half_extents);
            Self {
                aabb,
                history: Default::default(),
                shape,
            }
        }

        pub fn reached_by(&mut self, entity: Entity) -> bool {
            self.history.contains_key(&entity)
        }

        pub fn history(&self) -> &std::collections::HashMap<Entity, f32> {
            &self.history
        }
    }

    #[derive(Component)]
    pub struct Collider(pub Box<dyn parry2d::shape::Shape>);
}

pub mod events {
    use super::*;

    #[derive(Event)]
    pub struct GoalAreaReached {
        pub area:       Entity,
        pub reached_by: Entity,
    }
}

fn detect_collisions(
    mut goal_areas: Query<(Entity, &mut components::GoalArea)>,
    colliders: Query<(Entity, &Transform, &components::Collider)>,
    time_fixed: Res<Time<Fixed>>,
    mut evw_goal_area_reached: EventWriter<events::GoalAreaReached>,
) {
    for (collider_entity, tf, collider) in &colliders {
        for (goal_area_entity, mut goal_area) in &mut goal_areas {
            if goal_area.history.contains_key(&collider_entity) {
                continue;
            }

            let translation = parry2d::na::Vector2::new(tf.translation.x, tf.translation.z);
            let collider_pos = parry2d::na::Isometry2::new(translation, 0.0);
            let area_pos = goal_area.aabb.center().into();

            match parry2d::query::intersection_test(
                &collider_pos,
                collider.0.as_ref(),
                &area_pos,
                &goal_area.shape,
            ) {
                Ok(true) => {
                    goal_area
                        .history
                        .insert(collider_entity, time_fixed.elapsed_seconds());

                    evw_goal_area_reached.send(events::GoalAreaReached {
                        area:       goal_area_entity,
                        reached_by: collider_entity,
                    });
                }
                _ => {}
            }
        }
    }
}

fn setup_goal_areas_for_junction_scenario(mut commands: Commands) {
    commands.spawn(components::GoalArea::new(
        parry2d::bounding_volume::Aabb::new(
            parry2d::na::Point2::new(-8.0, -49.5),
            parry2d::na::Point2::new(8.0, -50.5),
        ),
    ));

    commands.spawn(components::GoalArea::new(
        parry2d::bounding_volume::Aabb::new(
            parry2d::na::Point2::new(49.5, -8.0),
            parry2d::na::Point2::new(50.5, 8.0),
        ),
    ));
}

fn render_goal_areas(mut gizmos: Gizmos, goal_areas: Query<&components::GoalArea>) {
    for goal_area in &goal_areas {
        let center = goal_area.aabb.center();
        let position: Vec3 = Vec3::new(center.x, -1.0, center.y);
        let half_extents = goal_area.aabb.half_extents();
        let size = Vec2::new(half_extents.x * 2.0, half_extents.y * 2.0);

        gizmos.rect(position, Quat::IDENTITY, size, Color::SEA_GREEN);
    }
}

fn log_collisions(mut evr_goal_area_reached: EventReader<events::GoalAreaReached>) {
    for event in evr_goal_area_reached.read() {
        info!(
            "goal area {:?} reached by {:?}",
            event.area, event.reached_by
        );
    }
}
