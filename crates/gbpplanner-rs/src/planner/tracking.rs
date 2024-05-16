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

#[derive(Clone, Copy)]
pub struct PositionMeasurement {
    pub position:  Vec3,
    pub timestamp: Instant,
}

/// A component that tracks position data of an entity using a ring buffer.
///
/// It stores position vectors (`Vec3`) and utilizes a timer to determine when
/// to capture and store an entity's current position into the ring buffer.
#[derive(Component)]
pub struct PositionTracker {
    ringbuf: HeapRb<PositionMeasurement>,
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
    pub fn ringbuf(&self) -> &HeapRb<PositionMeasurement> {
        &self.ringbuf
    }

    /// Returns a reference to the internal timer.
    pub fn timer(&self) -> &Timer {
        &self.timer
    }

    pub fn measurements(&self) -> impl Iterator<Item = &PositionMeasurement> + '_ {
        self.ringbuf.iter()
    }

    // // pub fn positions(&self) -> impl Iterator<Item = Vec3> + '_ {
    // pub fn positions(&self) -> impl Iterator<Item = Vec3> + '_ {
    //     self.ringbuf.iter().cloned().map(|m| m.position)
    // }

    /// Provides an iterator over the positions stored in the ring buffer.
    pub fn positions(&self) -> impl Iterator<Item = Vec2> + '_ {
        self.ringbuf
            .iter()
            .cloned()
            .map(|m| Vec2::new(m.position.x, m.position.z))
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
fn track_positions(
    mut q: Query<(&Transform, &mut PositionTracker), Changed<Transform>>,
    time: Res<Time>,
) {
    for (transform, mut tracker) in &mut q {
        tracker.timer.tick(time.delta());
        if tracker.timer.just_finished() {
            let measurement = PositionMeasurement {
                position:  transform.translation,
                timestamp: Instant::now(),
            };
            // tracker.ringbuf.push_overwrite(transform.translation);
            tracker.ringbuf.push_overwrite(measurement);

            if tracker.first_measurement_at.is_none() {
                tracker.first_measurement_at = Some(Instant::now());
            }
        }
    }
}

#[derive(Clone, Copy, serde::Serialize)]
pub struct VelocityMeasurement {
    pub velocity:      Vec3,
    // pub timestamp:     Instant,
    pub timestamp:     f64,
    pub measured_over: Duration,
}

#[derive(Clone, Copy, serde::Serialize)]
struct PreviousPosition {
    position:      Vec3,
    pub timestamp: f64,
    // timestamp: Instant,
}

/// A component that tracks velocity data of an entity with a transform using a
/// ring buffer.
#[derive(Component)]
pub struct VelocityTracker {
    ringbuf: HeapRb<VelocityMeasurement>,
    // last_position: Option<Vec3>,
    timer: Timer,
    previous_position: Option<PreviousPosition>,
    // first_measurement_at: Option<Instant>,
    first_measurement_at: Option<f64>,
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
            // last_position: None,
            timer: Timer::new(duration, TimerMode::Repeating),
            // previous_measurement: Some(VelocityMeasurement {
            //     velocity:      Vec3::ZERO,
            //     timestamp:     Instant::now(),
            //     measured_over: Duration::ZERO,
            // }),
            previous_position: None,
            first_measurement_at: None,
        }
    }

    pub fn measurements(&self) -> impl Iterator<Item = VelocityMeasurement> + '_ {
        self.ringbuf.iter().cloned()
    }

    // /// Provides an iterator over the velocities stored in the ring buffer.
    // pub fn velocities(&self) -> impl Iterator<Item = Vec3> + '_ {
    //     self.ringbuf.iter().cloned().map(|v| v.velocity)
    // }

    /// Provides an iterator over the velocities stored in the ring buffer.
    pub fn velocities(&self) -> impl Iterator<Item = Vec2> + '_ {
        self.ringbuf
            .iter()
            .cloned()
            .map(|v| Vec2::new(v.velocity.x, v.velocity.z))
    }
}

/// System function to update `VelocityTracker` components for entities whose
/// `Transform` has changed.
///
/// It checks if the update interval specified by the internal timer has elapsed
/// and updates the ring buffer with the current velocity of the entity.
fn track_velocities(
    mut q: Query<(&Transform, &mut VelocityTracker), Changed<Transform>>,
    time: Res<Time>,
) {
    for (transform, mut tracker) in &mut q {
        tracker.timer.tick(time.delta());
        if tracker.timer.just_finished() {
            // let now = Instant::now();
            let now = time.elapsed_seconds_f64();

            if let Some(previous_position) = tracker.previous_position {
                let dt = now - previous_position.timestamp;
                let measurement = VelocityMeasurement {
                    velocity:      (transform.translation - previous_position.position) / dt as f32,
                    timestamp:     now,
                    measured_over: Duration::from_secs_f64(dt),
                };
                // tracker.ringbuf.push_overwrite(transform.translation);
                tracker.ringbuf.push_overwrite(measurement);
            }
            tracker.previous_position = Some(PreviousPosition {
                position:  transform.translation,
                timestamp: now,
            });

            if tracker.first_measurement_at.is_none() {
                tracker.first_measurement_at = Some(now);
            }
        }
    }
}
