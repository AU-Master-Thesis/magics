use std::time::{Duration, Instant};

use bevy::prelude::*;
use ringbuf::{ring_buffer::RbBase, HeapRb, Rb};

// trait BevySchedule: ScheduleLabel + Clone {}
//
// impl<T: ScheduleLabel + Clone> BevySchedule for T {}

/// A Bevy plugin to track the positions of entities over time.
///
/// The `TrackingPlugin` integrates with the Bevy app and adds systems to track
/// positions of entities using a ring buffer to store historical data.
pub struct TrackingPlugin;
// pub schedule: Box<dyn BevySchedule>,

impl Plugin for TrackingPlugin {
    /// Adds the tracking system to the Bevy app.
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, (track_positions, track_velocities));
    }
}

// #[derive(Event)]
// pub struct ExportPositions;
//
// #[derive(Event)]
// pub struct ExportVelocities;

/// A Bevy bundle to track the positions and velocities of entities over time.
#[derive(Bundle)]
pub struct TrackingBundle {
    pub tracker: PositionTracker,
    pub velocity_tracker: VelocityTracker,
}

/// A component that tracks position data of an entity using a ring buffer.
///
/// It stores position vectors (`Vec3`) and utilizes a timer to determine when
/// to capture and store an entity's current position into the ring buffer.
#[derive(Component)]
pub struct PositionTracker {
    ringbuf: HeapRb<Vec3>,
    timer: Timer,
    first_measurement_at: Option<Instant>,
}

impl PositionTracker {
    /// Creates a new `PositionTracker` with specified buffer capacity and
    /// update interval.
    ///
    /// # Arguments
    /// * `capacity` - The number of position vectors the ring buffer can hold.
    /// * `duration` - The interval between position updates.
    pub fn new(capacity: usize, duration: Duration) -> Self {
        Self {
            ringbuf: HeapRb::new(capacity),
            timer: Timer::new(duration, TimerMode::Repeating),
            first_measurement_at: None,
        }
    }

    /// Returns a reference to the internal ring buffer.
    pub fn ringbuf(&self) -> &HeapRb<Vec3> {
        &self.ringbuf
    }

    /// Returns a reference to the internal timer.
    pub fn timer(&self) -> &Timer {
        &self.timer
    }

    /// Provides an iterator over the positions stored in the ring buffer.
    pub fn positions(&self) -> impl Iterator<Item = Vec3> + '_ {
        self.ringbuf.iter().cloned()
    }

    /// Clears all stored positions from the ring buffer.
    pub fn clear(&mut self) {
        self.ringbuf.clear();
    }

    /// Returns the number of positions currently stored in the ring buffer.
    pub fn len(&self) -> usize {
        self.ringbuf.len()
    }

    /// Determines whether the ring buffer is empty.
    pub fn is_empty(&self) -> bool {
        self.ringbuf.is_empty()
    }
}

/// System function to update `PositionTracker` components for entities whose
/// `Transform` has changed.
///
/// It checks if the update interval specified by the internal timer has elapsed
/// and updates the ring buffer with the current position of the entity.
fn track_positions(mut q: Query<(&Transform, &mut PositionTracker), Changed<Transform>>, time: Res<Time>) {
    for (transform, mut tracker) in &mut q {
        tracker.timer.tick(time.delta());
        if tracker.timer.just_finished() {
            tracker.ringbuf.push_overwrite(transform.translation);

            if tracker.first_measurement_at.is_none() {
                tracker.first_measurement_at = Some(Instant::now());
            }
        }
    }
}

/// A component that tracks velocity data of an entity with a transform using a
/// ring buffer.
#[derive(Component)]
pub struct VelocityTracker {
    ringbuf: HeapRb<Vec3>,
    last_position: Option<Vec3>,
    timer: Timer,
    first_measurement_at: Option<Instant>,
}

impl VelocityTracker {
    /// Creates a new `VelocityTracker` with specified buffer capacity and
    /// update interval.
    ///
    /// # Arguments
    /// * `capacity` - The number of velocity vectors the ring buffer can hold.
    /// * `duration` - The interval between velocity updates.
    pub fn new(capacity: usize, duration: Duration) -> Self {
        Self {
            ringbuf: HeapRb::new(capacity),
            last_position: None,
            timer: Timer::new(duration, TimerMode::Repeating),
            first_measurement_at: None,
        }
    }

    /// Provides an iterator over the velocities stored in the ring buffer.
    pub fn velocities(&self) -> impl Iterator<Item = Vec3> + '_ {
        self.ringbuf.iter().cloned()
    }
}

/// System function to update `VelocityTracker` components for entities whose
/// `Transform` has changed.
///
/// It checks if the update interval specified by the internal timer has elapsed
/// and updates the ring buffer with the current velocity of the entity.
fn track_velocities(mut q: Query<(&Transform, &mut VelocityTracker), Changed<Transform>>, time: Res<Time>) {
    for (transform, mut tracker) in &mut q {
        tracker.timer.tick(time.delta());
        if tracker.timer.just_finished() {
            if let Some(last_position) = tracker.last_position {
                let velocity = (transform.translation - last_position) / time.delta_seconds();
                tracker.ringbuf.push_overwrite(velocity);
            }
            tracker.last_position = Some(transform.translation);

            if tracker.first_measurement_at.is_none() {
                tracker.first_measurement_at = Some(Instant::now());
            }
        }
    }
}
