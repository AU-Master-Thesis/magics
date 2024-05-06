use std::path::Path;

use angle::Angle;
use bevy::{
    ecs::{component::Component, system::Resource},
    math::Vec2,
};
use derive_more::IntoIterator;
use gbp_geometry::RelativePoint;
use gbp_linalg::Float;
use serde::{Deserialize, Serialize};
use typed_floats::StrictlyPositiveFinite;

#[derive(Debug, Clone, Serialize, Deserialize, Component)]
#[serde(rename_all = "kebab-case")]
pub struct TileCoordinates {
    pub row: usize,
    pub col: usize,
}

impl TileCoordinates {
    #[must_use]
    pub const fn new(row: usize, col: usize) -> Self {
        Self { row, col }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, IntoIterator)]
#[into_iterator(owned, ref)]
#[serde(rename_all = "kebab-case")]
pub struct TileGrid(Vec<String>);

impl TileGrid {
    pub fn new(tiles: Vec<impl Into<String>>) -> Self {
        Self(tiles.into_iter().map(Into::into).collect())
    }

    pub fn iter(&self) -> std::slice::Iter<String> {
        self.0.iter()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns number of rows in the tilegrid
    #[inline]
    pub fn nrows(&self) -> usize {
        self.0.len()
    }

    /// Returns number of columns in the tilegrid
    #[inline]
    pub fn ncols(&self) -> usize {
        self.0[0].chars().count()
    }

    /// Returns the tile at the given coordinates
    pub fn get_tile(&self, row: usize, col: usize) -> Option<char> {
        self.0.get(row).and_then(|r| r.chars().nth(col))
    }

    // /// override the index operator to allow for easy access to the grid
    // pub fn get(&self, row: usize, col: usize) -> Option<char> {
    //     self.0.get(row).and_then(|r| r.chars().nth(col))
    // }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Rotation(Angle);

impl Rotation {
    /// Create a new `Rotation` from a given degree
    ///
    /// # Panics
    ///
    /// If `degree` is not in [0.0, 360.0]
    #[must_use]
    pub fn new(degree: Float) -> Self {
        Self(Angle::from_degrees(degree).expect("Invalid angle"))
    }
}

impl Rotation {
    /// Get the rotation in radians
    #[inline]
    pub const fn as_radians(&self) -> Float {
        self.0.as_radians()
    }

    /// Get the rotation in degrees
    #[inline]
    pub fn as_degrees(&self) -> Float {
        self.0.as_degrees()
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Cell {
    pub row: usize,
    pub col: usize,
}

/// A circle to be placed in the environment
/// - A [`PlaceableShape`] variant
#[derive(Debug, Serialize, Deserialize, Clone, derive_more::Constructor)]
#[serde(rename_all = "kebab-case")]
pub struct Circle {
    /// The radius of the circle
    /// This is a value in the range [0, 1]
    pub radius: StrictlyPositiveFinite<Float>,
    // /// The center of the circle,
    // pub center: RelativePoint,
}

impl Circle {
    /// Expand the circle's radius with a give factor `expansion`
    pub fn expanded(&self, expansion: Float) -> Self {
        // self.radius = StrictlyPositiveFinite::<Float>::new(self.radius.get() *
        // expansion).unwrap();
        Self {
            radius: StrictlyPositiveFinite::<Float>::new(self.radius.get() + expansion).unwrap(),
        }
    }

    /// Check if a given point is inside the circle
    /// Expects translation and rotation to be performed beforehand
    pub fn inside(&self, point: Vec2) -> bool {
        let squared_distance = point.length_squared();
        squared_distance <= self.radius.get().powi(2) as f32
    }
}

/// A triangle to be placed in the environment
/// - A [`PlaceableShape`] variant
#[derive(Debug, Serialize, Deserialize, Clone, derive_more::Constructor)]
#[serde(rename_all = "kebab-case")]
pub struct Triangle {
    /// The length of the base of the triangle
    /// This is a value in the range [0, 1]
    pub base_length: StrictlyPositiveFinite<Float>,
    /// The height of the triangle
    /// Intersects the base perpendicularly at the mid-point
    pub height:      StrictlyPositiveFinite<Float>,
    /// The mid-point of the base of the triangle
    /// This is a value in the range [0, 1]
    /// Defines where the height of the triangle intersects the base
    /// perpendicularly
    pub mid_point:   Float,
    // /// Where to place the center of the triangle
    // pub translation: RelativePoint,
}

impl Triangle {
    /// Expand the triangle's size with a give factor `expansion`, this
    /// includes:
    /// - `base_length`
    /// - `height`
    pub fn expanded(&self, expansion: Float) -> Self {
        let factor = expansion * 3.0;

        let expanded_height = self.height.get() + factor;
        let height_factor = expanded_height / self.height.get();
        let expanded_base_length = self.base_length.get() * height_factor;

        Self {
            base_length: StrictlyPositiveFinite::<Float>::new(expanded_base_length).unwrap(),
            height:      StrictlyPositiveFinite::<Float>::new(expanded_height).unwrap(),
            mid_point:   self.mid_point,
        }
    }

    pub fn points(&self) -> (Vec2, Vec2, Vec2) {
        (
            // bottom-left corner
            Vec2::new(
                -self.base_length.get() as f32 / 2.0,
                -self.height.get() as f32 / 2.0,
            ),
            // bottom-right corner
            Vec2::new(
                self.base_length.get() as f32 / 2.0,
                -self.height.get() as f32 / 2.0,
            ),
            // top corner
            Vec2::new(
                (self.mid_point as f32 - 0.5) * self.base_length.get() as f32,
                self.height.get() as f32 / 2.0,
            ),
        )
    }

    /// Check if a given point is inside the triangle
    /// Expects translation and rotation to be performed beforehand
    pub fn inside(&self, point: Vec2) -> bool {
        // find the three vertices of the triangle
        let (a, b, c) = self.points();

        let d1 = sign(Vec2::from(point), a, b);
        let d2 = sign(Vec2::from(point), b, c);
        let d3 = sign(Vec2::from(point), c, a);

        let has_neg = d1 < 0.0 || d2 < 0.0 || d3 < 0.0;
        let has_pos = d1 > 0.0 || d2 > 0.0 || d3 > 0.0;

        !(has_neg && has_pos)
    }
}

fn sign(p1: Vec2, p2: Vec2, p3: Vec2) -> f32 {
    (p1.x - p3.x) * (p2.y - p3.y) - (p2.x - p3.x) * (p1.y - p3.y)
}

// How to check if point is inside a triangle
// https://stackoverflow.com/questions/2049582/how-to-determine-if-a-point-is-in-a-2d-triangle
// float sign (fPoint p1, fPoint p2, fPoint p3)
// {
//     return (p1.x - p3.x) * (p2.y - p3.y) - (p2.x - p3.x) * (p1.y - p3.y);
// }

// bool PointInTriangle (fPoint pt, fPoint v1, fPoint v2, fPoint v3)
// {
//     float d1, d2, d3;
//     bool has_neg, has_pos;

//     d1 = sign(pt, v1, v2);
//     d2 = sign(pt, v2, v3);
//     d3 = sign(pt, v3, v1);

//     has_neg = (d1 < 0) || (d2 < 0) || (d3 < 0);
//     has_pos = (d1 > 0) || (d2 > 0) || (d3 > 0);

//     return !(has_neg && has_pos);
// }

/// A regular polygon to be placed in the environment
/// - A [`PlaceableShape`] variant
#[derive(Debug, Serialize, Deserialize, Clone, derive_more::Constructor)]
#[serde(rename_all = "kebab-case")]
pub struct RegularPolygon {
    /// The number of sides of the polygon
    pub sides:       usize,
    /// Side length of the polygon
    pub side_length: StrictlyPositiveFinite<Float>,
    // /// Where to place the center of the polygon
    // pub translation: RelativePoint,
}

impl RegularPolygon {
    /// Expand the polygon's `side_length` with a given factor `expansion`
    pub fn expanded(&self, expansion: Float) -> Self {
        // let factor = expansion * 2.0;

        RegularPolygon::new(
            self.sides,
            StrictlyPositiveFinite::<Float>::new(self.side_length.get() + expansion).unwrap(),
        )
    }

    /// Get the points of the polygon
    /// Calculate all the points of the polygon
    pub fn point_at(&self, i: usize) -> (Float, Float) {
        let angle = 2.0 * std::f64::consts::PI / self.sides as Float * i as Float
            + std::f64::consts::FRAC_PI_4;

        let x = angle.cos() * self.side_length.get();
        let y = angle.sin() * self.side_length.get();

        (x, y)
    }

    /// Check if a given point is inside the polygon
    /// Expects translation and rotation to be performed beforehand
    pub fn inside(&self, point: Vec2) -> bool {
        let mut inside = false;
        let (x, y) = (point.x as f64, point.y as f64);
        let mut j = self.sides - 1;
        for i in 0..self.sides {
            let (xi, yi) = self.point_at(i);
            let (xj, yj) = self.point_at(j);
            if yi < y && yj >= y || yj < y && yi >= y {
                if xi + (y - yi) / (yj - yi) * (xj - xi) < x {
                    inside = !inside;
                }
            }
            j = i;
        }
        inside
    }
}

/// A rectangle to be placed in the environment
/// - A [`PlaceableShape`] variant
#[derive(Debug, Serialize, Deserialize, Clone, derive_more::Constructor)]
#[serde(rename_all = "kebab-case")]
pub struct Rectangle {
    /// The width of the rectangle
    /// This is a value in the range [0, 1]
    pub width:  StrictlyPositiveFinite<Float>,
    /// The height of the rectangle
    /// This is a value in the range [0, 1]
    pub height: StrictlyPositiveFinite<Float>,
    // /// The center of the rectangle
    // pub translation: RelativePoint,
}

impl Rectangle {
    /// Expand the rectangle's size with a given factor `expansion`, this
    /// includes:
    /// - `width`
    /// - `height`
    pub fn expanded(&self, expansion: Float) -> Self {
        // self.width = StrictlyPositiveFinite::<Float>::new(self.width.get() *
        // expansion).unwrap(); self.height =
        // StrictlyPositiveFinite::<Float>::new(self.height.get() * expansion).unwrap();

        Rectangle::new(
            StrictlyPositiveFinite::<Float>::new(self.width.get() + expansion).unwrap(),
            StrictlyPositiveFinite::<Float>::new(self.height.get() + expansion).unwrap(),
        )
    }

    /// Check if a given point is inside the rectangle
    /// Expects translation and rotation to be performed beforehand
    pub fn inside(&self, point: Vec2) -> bool {
        let (x, y) = (point.x as f64, point.y as f64);

        let half_width = self.width.get() / 2.0;
        let half_height = self.height.get() / 2.0;

        if x >= -half_width && x <= half_width && y >= -half_height && y <= half_height {
            return true;
        }

        false
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PlaceableShape {
    Circle(Circle),
    Triangle(Triangle),
    RegularPolygon(RegularPolygon),
    Rectangle(Rectangle),
}

impl PlaceableShape {
    /// Create a new `Self::Circle`
    ///
    /// # Panics
    ///
    /// If `center` is not a relative point i.e. within interval ([0.0, 1.0],
    /// [0.0, 1.0])
    #[allow(clippy::unwrap_used)]
    pub fn circle(radius: StrictlyPositiveFinite<Float>) -> Self {
        Self::Circle(Circle::new(radius))
    }

    /// Create a new `Self::Triangle`
    ///
    /// # Panics
    ///
    /// If `translation` is not a relative point i.e. within interval ([0.0,
    /// 1.0], [0.0, 1.0])
    #[allow(clippy::unwrap_used)]
    pub fn triangle(
        // base_length: Float,
        base_length: StrictlyPositiveFinite<Float>,
        // height: Float,
        height: StrictlyPositiveFinite<Float>,
        mid_point: Float,
        // translation: (Float, Float),
    ) -> Self {
        Self::Triangle(Triangle::new(
            base_length,
            height,
            mid_point,
            // RelativePoint::new(translation.0, translation.1).unwrap(),
        ))
    }

    #[allow(clippy::unwrap_used, clippy::missing_panics_doc)]
    pub fn rectangle(width: Float, height: Float) -> Self {
        Self::Rectangle(Rectangle::new(
            StrictlyPositiveFinite::<Float>::new(width).unwrap(),
            StrictlyPositiveFinite::<Float>::new(height).unwrap(),
            // RelativePoint::new(center.0, center.1).unwrap(),
        ))
    }

    #[allow(clippy::unwrap_used, clippy::missing_panics_doc)]
    pub fn square(side_length: Float) -> Self {
        Self::RegularPolygon(RegularPolygon::new(
            4,
            StrictlyPositiveFinite::<Float>::new(side_length).unwrap(),
            // RelativePoint::new(center.0, center.1).unwrap(),
        ))
    }

    #[allow(clippy::unwrap_used, clippy::missing_panics_doc)]
    pub fn regular_polygon(sides: usize, side_length: Float) -> Self {
        Self::RegularPolygon(RegularPolygon::new(
            sides,
            StrictlyPositiveFinite::<Float>::new(side_length).unwrap(),
            // RelativePoint::new(translation.0, translation.1).unwrap(),
        ))
    }

    /// Expand the shape by a given factor `expansion`
    pub fn expanded(&self, expansion: Float) -> Self {
        let factor = expansion * 1.0;
        match self {
            Self::Circle(circle) => Self::Circle(circle.expanded(factor)),
            Self::Triangle(triangle) => Self::Triangle(triangle.expanded(factor)),
            Self::RegularPolygon(polygon) => Self::RegularPolygon(polygon.expanded(factor)),
            Self::Rectangle(rectangle) => Self::Rectangle(rectangle.expanded(factor)),
        }
    }

    /// Check if a given point is inside the shape
    pub fn inside(&self, point: Vec2) -> bool {
        match self {
            Self::Circle(circle) => circle.inside(point),
            Self::Triangle(triangle) => triangle.inside(point),
            Self::RegularPolygon(polygon) => polygon.inside(point),
            Self::Rectangle(rectangle) => rectangle.inside(point),
        }
    }

    /// Get the shape as a [`Circle`] if it is a [`Circle`]
    pub fn as_circle(&self) -> Option<&Circle> {
        if let Self::Circle(v) = self {
            Some(v)
        } else {
            None
        }
    }

    /// Get the shape as a [`Triangle`] if it is a [`Triangle`]
    pub fn as_triangle(&self) -> Option<&Triangle> {
        if let Self::Triangle(v) = self {
            Some(v)
        } else {
            None
        }
    }

    /// Get the shape as a [`RegularPolygon`] if it is a [`RegularPolygon`]
    pub fn as_regular_polygon(&self) -> Option<&RegularPolygon> {
        if let Self::RegularPolygon(v) = self {
            Some(v)
        } else {
            None
        }
    }

    /// Get the shape as a [`Rectangle`] if it is a [`Rectangle`]
    pub fn as_rectangle(&self) -> Option<&Rectangle> {
        if let Self::Rectangle(v) = self {
            Some(v)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Obstacle {
    /// The shape to be placed as an obstacle
    pub shape: PlaceableShape,
    /// Rotation of the obstacle in degrees around the up-axis
    pub rotation: Rotation,
    /// Translation of the obstacle within the tile
    pub translation: RelativePoint,
    /// Which tile in the grid the obstacle should be placed
    pub tile_coordinates: TileCoordinates,
}

impl Obstacle {
    /// Create a new `Obstacle`
    ///
    /// # Panics
    ///
    /// If `rotation` is not a normalized angle, i.e. within [0.0, 2pi]
    #[must_use]
    pub fn new(
        (row, col): (usize, usize),
        shape: PlaceableShape,
        rotation: Float,
        translation: (Float, Float),
    ) -> Self {
        Self {
            tile_coordinates: TileCoordinates::new(row, col),
            shape,
            rotation: Rotation(Angle::new(rotation).expect("Invalid angle")),
            translation: RelativePoint::new(translation.0, translation.1)
                .expect("Invalid relative point"),
        }
    }
}

/// Struct to represent a list of shapes that can be placed in the map [`Grid`]
/// Each entry contains a [`Cell`] and a [`PlaceableShape`]
/// - The [`Cell`] represents which tile in the [`Grid`] the shape should be
///   placed
/// - The [`PlaceableShape`] represents the shape to be placed, and the local
///   cell translation
#[derive(Debug, Clone, Serialize, Deserialize, IntoIterator)]
#[serde(rename_all = "kebab-case")]
#[into_iterator(owned, ref)]
pub struct Obstacles(Vec<Obstacle>);

impl Obstacles {
    /// Create a new empty vector of [`Obstacle`]
    #[must_use]
    pub const fn empty() -> Self {
        Self(Vec::new())
    }

    pub fn iter(&self) -> std::slice::Iter<Obstacle> {
        self.0.iter()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct TileSettings {
    pub tile_size:       f32,
    pub path_width:      f32,
    pub obstacle_height: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Tiles {
    pub grid:     TileGrid,
    pub settings: TileSettings,
}

impl Tiles {
    /// Create an empty `Tiles`
    #[must_use]
    pub fn empty() -> Self {
        Self {
            grid:     TileGrid::new(vec!["█"]),
            settings: TileSettings {
                tile_size:       0.0,
                path_width:      0.0,
                obstacle_height: 0.0,
            },
        }
    }

    /// Set the tile size
    #[must_use]
    pub const fn with_tile_size(mut self, tile_size: f32) -> Self {
        self.settings.tile_size = tile_size;
        self
    }

    /// Set the obstacle height
    #[must_use]
    pub const fn with_obstacle_height(mut self, obstacle_height: f32) -> Self {
        self.settings.obstacle_height = obstacle_height;
        self
    }
}

#[derive(Debug, clap::ValueEnum, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum EnvironmentType {
    #[default]
    Intersection,
    Intermediate,
    Complex,
    Circle,
    Maze,
    Test,
}

/// **Bevy** [`Resource`]
/// The environment configuration for the simulation
#[derive(Debug, Clone, Serialize, Deserialize, Resource)]
#[serde(rename_all = "kebab-case")]
pub struct Environment {
    pub tiles:     Tiles,
    pub obstacles: Obstacles,
}

impl Default for Environment {
    fn default() -> Self {
        Self::intersection()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("RON error: {0}")]
    Ron(#[from] ron::Error),
    #[error("TOML error: {0}")]
    Toml(#[from] toml::de::Error),
    #[error("YAML error: {0}")]
    Yaml(#[from] serde_yaml::Error),
    #[error("Validation error: {0}")]
    InvalidEnvironment(#[from] EnvironmentError),
}

#[derive(Debug, thiserror::Error)]
pub enum EnvironmentError {
    #[error("Environment matrix representation is empty")]
    EmptyGrid,
    #[error("Environment matrix representation has rows of different lengths")]
    DifferentLengthRows,
}

impl Environment {
    /// Attempt to parse an [`Environment`] from a YAML file at `path`
    ///
    /// # Errors
    ///
    /// Will return `Err` if:
    /// 1. `path` does not exist on the filesystem
    /// 2. The contents of `path` are not valid RON
    /// 3. The parsed data does not represent a valid [`Environment`]
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, ParseError> {
        std::fs::read_to_string(path)
            .map_err(Into::into)
            .and_then(|contents| Self::parse(contents.as_str()))
    }

    /// Attempt to parse an [`Environment`] from a YAML encoded string
    ///
    /// # Errors
    ///
    /// Will return `Err` if:
    /// 1. `path` does not exist on the filesystem
    /// 2. The contents of `path` are not valid RON
    /// 3. The parsed data does not represent a valid [`Environment`]
    pub fn parse(contents: &str) -> Result<Self, ParseError> {
        // ron::from_str::<Environment>(contents)
        //     .map_err(|span| span.code)?
        //     .validate()
        //     .map_err(Into::into)
        // with yaml

        serde_yaml::from_str::<Self>(contents)
            .map_err(Into::into)
            .and_then(|env| env.validate().map_err(Into::into))
    }

    /// Ensure that the [`Environment`] is valid
    ///
    /// # Errors
    ///
    /// Will return `Err` if:
    /// 1. The matrix representation is not empty
    /// 2. All rows in the matrix representation are the same length
    pub fn validate(self) -> Result<Self, EnvironmentError> {
        if self.tiles.grid.is_empty() {
            Err(EnvironmentError::EmptyGrid)
        } else if self
            .tiles
            .grid
            .iter()
            .any(|row| row.chars().count() != self.tiles.grid.ncols())
        {
            Err(EnvironmentError::DifferentLengthRows)
        } else {
            Ok(self)
        }
    }

    #[must_use]
    pub fn new(
        matrix_representation: Vec<String>,
        path_width: f32,
        obstacle_height: f32,
        tile_size: f32,
    ) -> Self {
        Self {
            tiles:     Tiles {
                grid:     TileGrid(matrix_representation),
                settings: TileSettings {
                    tile_size,
                    path_width,
                    obstacle_height,
                },
            },
            obstacles: Obstacles::empty(),
        }
    }

    #[must_use]
    pub fn intersection() -> Self {
        Self {
            tiles:     Tiles {
                grid:     TileGrid::new(vec!["┼"]),
                settings: TileSettings {
                    tile_size:       100.0,
                    path_width:      0.1325,
                    obstacle_height: 1.0,
                },
            },
            obstacles: Obstacles::empty(),
        }
    }

    #[must_use]
    #[rustfmt::skip]
    pub fn intermediate() -> Self {
        Self {
            tiles: Tiles {
                grid: TileGrid::new(vec![
                    "┌┬┐ ",
                    "┘└┼┬",
                    "  └┘",
                ]),
                settings: TileSettings {
                    tile_size: 50.0,
                    path_width: 0.1325,
                    obstacle_height: 1.0,
                }
            },
            obstacles: Obstacles::empty(),
        }
    }

    #[must_use]
    #[rustfmt::skip]
    pub fn complex() -> Self {
        Self {
            tiles: Tiles {
                grid: TileGrid::new(vec![
                    "┌─┼─┬─┐┌",
                    "┼─┘┌┼┬┼┘",
                    "┴┬─┴┼┘│ ",
                    "┌┴┐┌┼─┴┬",
                    "├─┴┘└──┘",
                ]),
                settings: TileSettings {
                    tile_size: 25.0,
                    path_width: 0.4,
                    obstacle_height: 1.0,
                },
            },
            obstacles: Obstacles::empty(),
        }
    }

    #[must_use]
    #[rustfmt::skip]
    pub fn maze() -> Self {
        Self {
            tiles: Tiles {
                grid: TileGrid::new(vec![
                    "               ",
                    " ╶─┬─┐┌─────┬┐ ",
                    " ┌─┤┌┤│╷╶──┬┘│ ",
                    " │╷│╵├┤├─┬┬┴┬┤ ",
                    " └┤├─┘││╷╵├─┘│ ",
                    " ╷│╵╷╶┤│├┐└╴┌┘ ",
                    " │├─┴╴│╵│└──┤╷ ",
                    " └┤┌─┐└┬┘┌─┐└┘ ",
                    " ┌┴┤╷├╴│┌┤╷└─┐ ",
                    " │┌┤├┘┌┘││└──┤ ",
                    " ╵│╵├┬┘┌┘└──┐╵ ",
                    " ┌┘╶┘├─┴─┐╷╷└┐ ",
                    " └─┬─┴──┐├┘├─┘ ",
                    " ┌┐│╷┌─╴││╶┘╶┐ ",
                    " │└┼┘├──┘├──┬┤ ",
                    " ╵╶┴─┘╶──┴──┴┘ ",
                    "               ",
                ]),
                settings: TileSettings {
                    tile_size: 10.0,
                    path_width: 0.75,
                    obstacle_height: 1.0,
                },
            },
            obstacles: Obstacles::empty(),
        }
    }

    #[must_use]
    #[rustfmt::skip]
    pub fn test() -> Self {
        Self {
            tiles: Tiles {
                grid: TileGrid::new(vec![
                    "┌┬┐├",
                    "└┴┘┤",
                    "│─ ┼",
                    "╴╵╶╷",
                ]),
                settings: TileSettings {
                    tile_size: 50.0,
                    path_width: 0.1325,
                    obstacle_height: 1.0,
                },
            },
            obstacles: Obstacles::empty(),
        }
    }

    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    pub fn circle() -> Self {
        Self {
            tiles:     Tiles::empty()
                .with_tile_size(100.0)
                .with_obstacle_height(1.0),
            obstacles: Obstacles(vec![
                Obstacle::new(
                    (0, 0),
                    PlaceableShape::regular_polygon(4, 0.0525),
                    0.0,
                    (0.625, 0.60125),
                ),
                Obstacle::new(
                    (0, 0),
                    PlaceableShape::regular_polygon(4, 0.035),
                    0.0,
                    (0.44125, 0.57125),
                ),
                Obstacle::new(
                    (0, 0),
                    PlaceableShape::regular_polygon(4, 0.0225),
                    0.0,
                    (0.4835, 0.428),
                ),
                Obstacle::new(
                    (0, 0),
                    PlaceableShape::rectangle(0.0875, 0.035),
                    0.0,
                    (0.589, 0.3965),
                ),
                Obstacle::new(
                    (0, 0),
                    PlaceableShape::triangle(
                        0.03.try_into().expect("positive and finite"),
                        0.0415.try_into().expect("positive and finite"),
                        0.575,
                    ),
                    0.0,
                    (0.5575, 0.5145),
                ),
                Obstacle::new(
                    (0, 0),
                    PlaceableShape::triangle(
                        0.012.try_into().expect("positive and finite"),
                        0.025.try_into().expect("positive and finite"),
                        1.25,
                    ),
                    5.225,
                    (0.38, 0.432),
                ),
            ]),
        }
    }

    pub const fn path_width(&self) -> f32 {
        self.tiles.settings.path_width
    }

    pub const fn obstacle_height(&self) -> f32 {
        self.tiles.settings.obstacle_height
    }

    pub const fn tile_size(&self) -> f32 {
        self.tiles.settings.tile_size
    }
}
