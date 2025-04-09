use bevy::prelude::*;
use gbp_config::formation::ReachedWhen;
use itertools::Itertools;
use std::time::Duration;

use super::bundle::StateVector; // Assuming bundle.rs contains StateVector

/// Represents a robotic route consisting of several waypoints that define
/// positions and velocities the robot should achieve as it progresses along the
/// path.
#[allow(clippy::similar_names)]
#[derive(Component, Debug, derive_more::Index)]
pub struct Route {
    /// A list of state vectors representing waypoints.
    #[index]
    waypoints:    Vec<StateVector>,
    /// The index of the next target waypoint in the waypoints vector.
    pub target_index: usize, // Made public
    /// The recorded time at the start of the route as a floating-point
    /// timestamp.
    started_at:   f64,
    /// Optional recorded time when the route was completed as a floating-point
    /// timestamp.
    finished_at:  Option<f64>,
}

impl std::fmt::Display for Route {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "waypoints:")?;
        for wp in &self.waypoints {
            writeln!(f, "  {}", wp)?;
        }

        writeln!(f, "target_index: {}", self.target_index)?;
        writeln!(f, "started_at: {}", self.started_at)?;
        writeln!(f, "finished_at: {:?}", self.finished_at)
    }
}

impl Route {
    /// Creates a new route from a specified set of waypoints and initial time.
    ///
    /// # Arguments
    /// * `waypoints` - A vector of `StateVector` that must contain at least two
    ///   elements.
    /// * `started_at` - The start time of the route as a floating-point
    ///   timestamp.
    pub fn new(
        waypoints: min_len_vec::TwoOrMore<StateVector>,
        started_at: f64,
    ) -> Self {
        Self {
            waypoints: waypoints.into(),
            target_index: 1, // skip first waypoint, as it is the initial pose
            started_at,
            finished_at: None,
        }
    }

    pub fn update_waypoints(&mut self, waypoints: min_len_vec::TwoOrMore<StateVector>) {
        self.waypoints = waypoints.into();
        self.target_index = 1;
    }

    /// Returns a reference to the next waypoint, if available.
    pub fn next_waypoint(&self) -> Option<&StateVector> {
        self.waypoints.get(self.target_index)
    }

    pub fn current_waypoint_index(&self) -> Option<usize> {
        if self.is_completed() {
            None
        } else {
            Some(self.target_index)
        }
    }

    /// Returns a reference to the last waypoint, if available.
    pub fn last_waypoint(&self) -> Option<&StateVector> {
        self.waypoints.get(self.target_index - 1)
    }

    pub fn next_waypoint_is_last(&self) -> bool {
        self.target_index == self.waypoints.len() - 1
    }

    /// Advances to the next waypoint, updating the finished time if the route
    /// is completed.
    ///
    /// # Arguments
    /// * `elapsed` - The current time as a `std::time::Duration` since the
    ///   start.
    pub fn advance(&mut self, elapsed: Duration) {
        if self.target_index < self.waypoints.len() {
            self.target_index += 1;
        }
        if self.is_completed() && self.finished_at.is_none() {
            self.finished_at = Some(elapsed.as_secs_f64() + self.started_at);
        }
    }

    /// Returns the total number of waypoints.
    #[inline]
    pub fn len(&self) -> usize {
        self.waypoints.len()
    }

    /// Provides a slice of all waypoints.
    #[inline]
    pub fn waypoints(&self) -> &[StateVector] {
        &self.waypoints
    }

    /// Returns the start time of the route.
    #[inline]
    pub fn started_at(&self) -> f64 {
        self.started_at
    }

    /// Returns the finish time of the route, if completed.
    #[inline]
    pub fn finished_at(&self) -> Option<f64> {
        self.finished_at
    }

    /// Returns a reference to the first waypoint.
    #[inline]
    pub fn first(&self) -> &StateVector {
        // waypoints are guaranteed to have at least two elements
        &self.waypoints[0]
    }

    /// Returns a reference to the last waypoint.
    #[inline]
    pub fn last(&self) -> &StateVector {
        // waypoints are guaranteed to have at least two elements
        &self.waypoints[self.waypoints.len() - 1]
    }

    /// Checks whether all waypoints have been reached.
    #[inline]
    pub fn is_completed(&self) -> bool {
        self.target_index >= self.waypoints.len()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MissionState {
    Idle { waiting_for_waypoints: bool },
    Active,
    Completed,
}

impl MissionState {
    pub fn idle(&self) -> bool {
        matches!(self, MissionState::Idle { .. })
    }
}

#[derive(Debug, Component)]
pub struct Mission {
    pub routes: Vec<Route>,
    pub taskpoints: Vec<StateVector>,
    pub active_route: usize, // Made public
    pub started_at: f64, // Made public
    pub finished_at: Option<f64>, // Made public
    pub state: MissionState,
    pub finished_when_intersects: ReachedWhen, // Made public
    pub taskpoint_reached_when_intersects: ReachedWhen, // Made public
    /// ID of the square currently being targeted, if applicable
    pub target_square_id: Option<String>,
}

impl Mission {
    pub fn local(
        waypoints: min_len_vec::TwoOrMore<StateVector>,
        started_at: f64,
        finished_when_intersects: ReachedWhen,
        waypoint_reached_when_intersects: ReachedWhen,
    ) -> Self {
        let route = Route::new(
            waypoints.iter().copied().collect_vec().try_into().unwrap(),
            started_at,
        );
        Self {
            routes: vec![route],
            taskpoints: vec![*waypoints.first(), *waypoints.last()],
            active_route: 0,
            started_at,
            finished_at: None,
            state: MissionState::Active,
            finished_when_intersects,
            taskpoint_reached_when_intersects: waypoint_reached_when_intersects,
            target_square_id: None, // Initialize as None
        }
    }

    pub fn global(
        waypoints: min_len_vec::TwoOrMore<StateVector>,
        started_at: f64,
        finished_when_intersects: ReachedWhen,
        waypoint_reached_when_intersects: ReachedWhen,
    ) -> Self {
        Self::new(
            waypoints,
            started_at,
            MissionState::Idle {
                waiting_for_waypoints: false,
            },
            finished_when_intersects,
            waypoint_reached_when_intersects,
        )
    }

    fn new(
        waypoints: min_len_vec::TwoOrMore<StateVector>,
        started_at: f64,
        state: MissionState,
        finished_when_intersects: ReachedWhen,
        waypoint_reached_when_intersects: ReachedWhen,
    ) -> Self {
        assert_ne!(state, MissionState::Completed);
        let first_route = Route::new(
            waypoints
                .iter()
                .copied()
                .take(2)
                .collect::<Vec<_>>()
                .try_into()
                .unwrap(),
            started_at,
        );
        Self {
            routes: vec![first_route],
            taskpoints: waypoints.into(),
            active_route: 0,
            started_at,
            finished_at: None,
            state,
            finished_when_intersects,
            taskpoint_reached_when_intersects: waypoint_reached_when_intersects,
            target_square_id: None, // Initialize as None
        }
    }

    /// Return the time at which the mission was started in seconds
    #[inline]
    pub fn started_at(&self) -> f64 {
        self.started_at
    }

    pub fn finished_at(&self) -> Option<f64> {
        self.finished_at
    }

    pub fn is_completed(&self) -> bool {
        self.state == MissionState::Completed
    }

    pub fn next_waypoint(&self) -> Option<&StateVector> {
        self.routes
            .get(self.active_route)
            .and_then(|r| r.next_waypoint())
    }

    pub fn current_waypoint_index(&self) -> Option<usize> {
        self.routes
            .get(self.active_route)
            .and_then(|r| r.current_waypoint_index())
    }

    pub fn last_waypoint(&self) -> Option<&StateVector> {
        self.routes
            .get(self.active_route)
            .and_then(|r| r.last_waypoint())
    }

    pub fn next_waypoint_is_last(&self) -> bool {
        self.active_route == self.routes.len() - 1
            && self
                .active_route()
                .is_some_and(|r| r.next_waypoint_is_last())
    }

    pub fn active_route(&self) -> Option<&Route> {
        if self.is_completed() {
            None
        } else {
            self.routes.get(self.active_route)
        }
    }

    pub fn active_route_mut(&mut self) -> Option<&mut Route> {
        if self.is_completed() {
            None
        } else {
            self.routes.get_mut(self.active_route)
        }
    }

    pub fn next_route(&mut self, time: &Time) {
        match self.state {
            MissionState::Completed => {}
            _ => {
                self.active_route += 1;
                if self.active_route >= self.taskpoints.len() - 1 {
                    self.state = MissionState::Completed;
                    self.finished_at = Some(time.elapsed().as_secs_f64());
                } else {
                    let waypoints: Vec<StateVector> = self
                        .taskpoints
                        .iter()
                        .skip(self.active_route)
                        .take(2)
                        .copied()
                        .collect_vec();
                    let next_route =
                        Route::new(waypoints.try_into().unwrap(), time.elapsed_seconds_f64());
                    self.routes.push(next_route);
                    self.state = MissionState::Idle {
                        waiting_for_waypoints: false,
                    }
                }
            }
        }
    }

    pub fn advance_to_next_waypoint(&mut self, time: &Time) {
        match self.state {
            MissionState::Active => {
                let current_route = self.routes.get_mut(self.active_route).unwrap();
                current_route.advance(time.elapsed());
                if current_route.is_completed() {
                    self.next_route(time);
                }
            }
            MissionState::Completed | MissionState::Idle { .. } => return,
        }
    }

    pub fn waypoints(&self) -> impl Iterator<Item = &StateVector> + '_ {
        self.routes.iter().flat_map(|r| r.waypoints())
    }
}
