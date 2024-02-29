use super::BoundingBox;
use nalgebra::Vector3;

#[derive(Debug)]
pub struct NegativeExtentsError {
    x: Option<f32>,
    y: Option<f32>,
    z: Option<f32>,
}

impl std::fmt::Display for NegativeExtentsError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if let Some(x) = self.x {
            write!(f, "x half extent is negative: {} ", x)?;
        }
        if let Some(y) = self.y {
            write!(f, "y half extent is negative: {} ", y)?;
        }
        if let Some(z) = self.z {
            write!(f, "z half extent is negative: {}", z)?;
        }
        Ok(())
    }
}

impl std::error::Error for NegativeExtentsError {}

#[derive(Clone, Debug)]
pub struct BoundingBox3d {
    pub min: Vector3<f32>,
    pub max: Vector3<f32>,
}

impl BoundingBox3d {
    pub fn from_min_max(min: Vector3<f32>, max: Vector3<f32>) -> Self {
        Self { min, max }
    }

    /// Create a new 3D bounding box from a center point and half extents
    /// precondition: x, y, and z must be non-negative
    /// returns an error if any of the half extents are negative
    pub fn from_center_and_half_extents(
        center: Vector3<f32>,
        x: f32,
        y: f32,
        z: f32,
    ) -> Result<Self, NegativeExtentsError> {
        if x < 0.0 || y < 0.0 || z < 0.0 {
            return Err(NegativeExtentsError {
                x: if x < 0.0 { Some(x) } else { None },
                y: if y < 0.0 { Some(y) } else { None },
                z: if z < 0.0 { Some(z) } else { None },
            });
        }
        let min = center - Vector3::new(x, y, z);
        let max = center + Vector3::new(x, y, z);
        Ok(Self { min, max })
    }

    /// Calculate the volume of the bounding box
    pub fn volume(&self) -> f32 {
        f32::abs(self.max.x - self.min.x)
            * f32::abs(self.max.y - self.min.y)
            * f32::abs(self.max.z - self.min.z)
    }
}

impl BoundingBox for BoundingBox3d {
    type Coorditate = Vector3<f32>;

    fn min(&self) -> Self::Coorditate {
        self.min
    }

    fn max(&self) -> Self::Coorditate {
        self.max
    }

    fn center(&self) -> Self::Coorditate {
        (self.min + self.max) / 2.0
    }

    fn contains(&self, point: &Self::Coorditate) -> bool {
        (point.x >= self.min.x && point.x <= self.max.x)
            && (point.y >= self.min.y && point.y <= self.max.y)
            && (point.z >= self.min.z && point.z <= self.max.z)
    }

    /// Return the vertices of the bounding box
    /// The vertices are returned in in such a order, that if you trace a line from
    /// [0] -> [1] -> [2] -> [3] -> [0], you will get the bottom plane of the bounding box
    /// and if you trace a line from [4] -> [5] -> [6] -> [7] -> [4], you will get the top plane of the bounding box
    /// and if you trace a line from [0] -> [4], [1] -> [5], [2] -> [6], [3] -> [7], you will get the edges of the bounding box
    fn vertices(&self) -> Vec<Self::Coorditate> {
        let vs = vec![
            // bottom plane
            self.min,
            Vector3::new(self.min.x, self.min.y, self.max.z),
            Vector3::new(self.max.x, self.min.y, self.max.z),
            Vector3::new(self.max.x, self.min.y, self.min.z),
            // top plane
            Vector3::new(self.min.x, self.max.y, self.min.z),
            Vector3::new(self.min.x, self.max.y, self.max.z),
            self.max,
            Vector3::new(self.max.x, self.max.y, self.min.z),
        ];

        // There are 8 vertices in a 3D bounding box
        debug_assert_eq!(vs.len(), 8);
        vs
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_bounding_box3d_volume() {
        let bbox = BoundingBox3d::from_min_max(
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(1.0, 1.0, 1.0),
        );
        assert_eq!(bbox.volume(), 1.0);

        let bbox = BoundingBox3d::from_min_max(
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(2.0, 2.0, 2.0),
        );
        assert_eq!(bbox.volume(), 8.0);

        let bbox = BoundingBox3d::from_min_max(
            Vector3::new(-1.0, 2.0, 3.0),
            Vector3::new(1.0, 3.0, 4.0),
        );
        assert_eq!(bbox.volume(), 2.0);
    }

    #[test]
    fn test_bounding_box3d_center() {
        let bbox = BoundingBox3d::from_min_max(
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(1.0, 1.0, 1.0),
        );
        assert_eq!(bbox.center(), Vector3::new(0.5, 0.5, 0.5));

        let bbox = BoundingBox3d::from_min_max(
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(2.0, 2.0, 2.0),
        );
        assert_eq!(bbox.center(), Vector3::new(1.0, 1.0, 1.0));

        let bbox = BoundingBox3d::from_min_max(
            Vector3::new(-1.0, 2.0, 3.0),
            Vector3::new(1.0, 3.0, 4.0),
        );
        assert_eq!(bbox.center(), Vector3::new(0.0, 2.5, 3.5));
    }

    #[test]
    fn test_bounding_box3d_contains() {
        let bbox = BoundingBox3d::from_min_max(
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(1.0, 1.0, 1.0),
        );
        assert!(bbox.contains(&Vector3::new(0.5, 0.5, 0.5)));
        assert!(!bbox.contains(&Vector3::new(1.5, 0.5, 0.5)));
        assert!(!bbox.contains(&Vector3::new(0.5, 1.5, 0.5)));
        assert!(!bbox.contains(&Vector3::new(0.5, 0.5, 1.5)));

        let bbox = BoundingBox3d::from_min_max(
            Vector3::new(-1.0, -3.5, 0.0),
            Vector3::new(2.0, 2.0, 2.0),
        );
        assert!(bbox.contains(&Vector3::new(0.5, -2.0, 1.0)));
        assert!(!bbox.contains(&Vector3::new(0.5, -4.0, 1.0)));
        assert!(!bbox.contains(&Vector3::new(0.5, -2.0, 3.0)));
        assert!(!bbox.contains(&Vector3::new(3.0, -2.0, 1.0)));
    }

    #[test]
    fn test_bounding_box3d_vertices() {
        let bbox = BoundingBox3d::from_min_max(
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(1.0, 1.0, 1.0),
        );
        let vertices = bbox.vertices();
        assert_eq!(vertices.len(), 8);
        assert_eq!(vertices[0], bbox.min);
        assert_eq!(
            vertices[1],
            Vector3::new(bbox.min.x, bbox.min.y, bbox.max.z)
        );
        assert_eq!(
            vertices[2],
            Vector3::new(bbox.max.x, bbox.min.y, bbox.max.z)
        );
        assert_eq!(
            vertices[3],
            Vector3::new(bbox.max.x, bbox.min.y, bbox.min.z)
        );
        assert_eq!(
            vertices[4],
            Vector3::new(bbox.min.x, bbox.max.y, bbox.min.z)
        );
        assert_eq!(
            vertices[5],
            Vector3::new(bbox.min.x, bbox.max.y, bbox.max.z)
        );
        assert_eq!(vertices[6], bbox.max);
        assert_eq!(
            vertices[7],
            Vector3::new(bbox.max.x, bbox.max.y, bbox.min.z)
        );
    }

    #[test]
    fn test_bounding_box3d_intersects() {
        let bbox1 = BoundingBox3d::from_min_max(
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(1.0, 1.0, 1.0),
        );
        let bbox2 = BoundingBox3d::from_min_max(
            Vector3::new(0.5, 0.5, 0.5),
            Vector3::new(1.5, 1.5, 1.5),
        );
        assert!(bbox1.intersects(&bbox2));

        let bbox1 = BoundingBox3d::from_min_max(
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(1.0, 1.0, 1.0),
        );
        let bbox2 = BoundingBox3d::from_min_max(
            Vector3::new(1.5, 0.5, 0.5),
            Vector3::new(2.5, 1.5, 1.5),
        );
        assert!(!bbox1.intersects(&bbox2));
    }
}
