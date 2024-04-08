use std::{collections::VecDeque, time::Duration};

use bevy::{input::common_conditions::*, prelude::*};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if let Ok(rust_log) = std::env::var("RUST_LOG") {
        println!("RUST_LOG = {}", rust_log);
    }
    App::new()
        .add_plugins(DefaultPlugins)
        .add_event::<CreateNotificationEvent>()
        .add_event::<DespawnNotificationEvent>()
        .init_resource::<NotificationStore>()
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                advance_time,
                send_notification.run_if(input_just_pressed(KeyCode::Space)),
                listen_for_notifications,
                remove_notifications,
            ),
        )
        .run();

    Ok(())
}

#[derive(Debug, Component)]
pub struct NotificationCountdown(Timer);

impl NotificationCountdown {
    pub fn new(duration: Duration) -> Self {
        Self(Timer::from_seconds(duration.as_secs_f32(), TimerMode::Once))
    }
}

#[derive(Debug, Default)]
struct NotificationIdGenerator {
    current: NotificationId,
}

impl NotificationIdGenerator {
    fn next_id(&mut self) -> NotificationId {
        let next = self.current;
        self.current += 1;
        next
    }
}

#[derive(Resource, Default)]
struct NotificationStore {
    notifications: VecDeque<Notification>,
    id_generator: NotificationIdGenerator,
}

impl NotificationStore {
    pub fn len(&self) -> usize {
        self.notifications.len()
    }

    pub fn is_empty(&self) -> bool {
        self.notifications.is_empty()
    }

    pub fn insert(&mut self, event: CreateNotificationEvent) -> NotificationId {
        let id = self.id_generator.next_id();
        let notification =
            Notification::new(id, event.title, event.body, event.category, event.duration);
        self.notifications.push_back(notification);
        id
    }

    pub fn remove(&mut self, id: NotificationId) -> Option<Notification> {
        for i in 0..self.len() {
            if self.notifications[i].id == id {
                return self.notifications.remove(i);
            }
        }
        None
    }
}

type NotificationId = usize;

#[derive(Debug)]
struct Notification {
    id: NotificationId,
    title: String,
    body: String,
    category: NotificationCategory,
    duration: Duration,
}

impl Notification {
    fn new(
        id: NotificationId,
        title: impl Into<String>,
        body: impl Into<String>,
        category: NotificationCategory,
        duration: Duration,
    ) -> Self {
        Self {
            id,
            title: title.into(),
            body: body.into(),
            category,
            duration,
        }
    }
}

#[derive(Debug, Event, Clone)]
pub struct CreateNotificationEvent {
    title: String,
    body: String,
    category: NotificationCategory,
    duration: Duration,
}

#[derive(Debug, Default, PartialEq, Clone, Copy)]
pub enum NotificationCategory {
    #[default]
    Info,
    Warn,
    Error,
}

impl CreateNotificationEvent {
    pub fn new(
        title: impl Into<String>,
        body: impl Into<String>,
        category: NotificationCategory,
        duration: Duration,
    ) -> Self {
        Self {
            title: title.into(),
            body: body.into(),
            category,
            duration,
        }
    }
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
    // root node
    // commands.spawn(NodeBundle {
    //     style: Style {
    //         width: Val::Percent(100.0),
    //         height: Val::Percent(100.0),
    //         justify_content: JustifyContent::SpaceBetween,
    //         ..default()
    //     },
    //     ..default()
    // });
}

fn send_notification(mut ev_notification: EventWriter<CreateNotificationEvent>) {
    println!("sending notification");
    ev_notification.send(CreateNotificationEvent::new(
        "title",
        "body",
        NotificationCategory::Info,
        Duration::from_secs(5),
    ));
}

fn listen_for_notifications(
    mut commands: Commands,
    mut ev_notification: EventReader<CreateNotificationEvent>,
    mut store: ResMut<NotificationStore>,
) {
    for notification in ev_notification.read() {
        println!("received notification: {:?}", notification);
        let id = store.insert(notification.clone());
        println!("inserted notification into store with id: {:?}", id);
        println!("the store currently contains {} notifications", store.len());

        commands
            .spawn((
                NotificationCountdown::new(notification.duration),
                NotificationComponent(id),
                NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        justify_content: JustifyContent::SpaceBetween,
                        ..default()
                    },
                    ..default()
                },
            ))
            .with_children(|parent| {
                // left vertical fill (border)
                parent.spawn(NodeBundle {
                    style: Style {
                        width: Val::Px(200.),
                        height: Val::Px(200.0),
                        border: UiRect::all(Val::Px(2.)),
                        ..default()
                    },
                    background_color: Color::rgb(0.65, 0.65, 0.65).into(),
                    ..default()
                });
            });
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationColumnPlacement {
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationRowPlacement {
    Top,
    Bottom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NotificationPlacement {
    row: NotificationRowPlacement,
    column: NotificationColumnPlacement,
}

// #[derive(Debug, Component, Clone, Copy)]
#[derive(Component, Clone, Copy)]
pub struct NotificationComponent(pub NotificationId);

// #[derive(Component)]
// pub struct Foo;

#[derive(Debug, Event, Clone, Copy)]
pub struct DespawnNotificationEvent(NotificationId);

fn advance_time(
    time: Res<Time>,
    mut query: Query<(&NotificationComponent, &mut NotificationCountdown)>,
    // query: Query<NotificationComponen>,
    // query: Query<&Foo>,
    mut ev_despawn_notification: EventWriter<DespawnNotificationEvent>,
) {
    for (NotificationComponent(id), mut countdown) in query.iter_mut() {
        // for (component, mut countdown) in query.iter_mut() {
        countdown.0.tick(time.delta());

        if countdown.0.just_finished() {
            println!("countdown finished sending despawn event");
            ev_despawn_notification.send(DespawnNotificationEvent(*id));
        }
    }
}

fn remove_notifications(
    mut commands: Commands,
    mut ev_despawn_notification: EventReader<DespawnNotificationEvent>,
    mut store: ResMut<NotificationStore>,
) {
    for ev in ev_despawn_notification.read() {
        let Some(notification) = store.remove(ev.0) else {
            error!(
                "attempted to remove a notification {:?} that does not exist!",
                ev.0
            );
            continue;
        };

        // commands.get_entity(entity).despawn();

        println!("removed notification: {:?}", notification);
    }
}
