use super::BoundingBox;
use nalgebra::Vector2;

#[derive(Clone, Debug)]
pub struct BoundingBox2d {
    pub bottom_left: Vector2<f32>,
    pub top_right: Vector2<f32>,
}

impl BoundingBox2d {
    pub fn new(bottom_left: Vector2<f32>, top_right: Vector2<f32>) -> Self {
        Self {
            bottom_left,
            top_right,
        }
    }

    /// Calculate the area of the bounding box
    pub fn area(&self) -> f32 {
        (self.top_right.x - self.bottom_left.x) * (self.top_right.y - self.bottom_left.y)
    }

    // pub fn vertices(&self) -> [Vector2<f32>; 4] {
    //     [self.bottom_right, self.top_left]
    // }
}

// struct BoundingBox2dIteratorState {
//     bbox: BoundingBox2d,
//     index: usize,
// }

impl BoundingBox for BoundingBox2d {
    type Coorditate = Vector2<f32>;

    fn min(&self) -> Vector2<f32> {
        self.bottom_left
    }

    fn max(&self) -> Vector2<f32> {
        self.top_right
    }

    fn center(&self) -> Vector2<f32> {
        (self.bottom_left + self.top_right) / 2.0
    }

    fn contains(&self, point: &Vector2<f32>) -> bool {
        point.x >= self.bottom_left.x
            && point.x <= self.top_right.x
            && point.y >= self.bottom_left.y
            && point.y <= self.top_right.y
    }

    fn vertices(&self) -> Vec<Vector2<f32>> {
        let top_left = Vector2::new(self.bottom_left.x, self.top_right.y);
        let bottom_right = Vector2::new(self.top_right.x, self.bottom_left.y);
        vec![self.bottom_left, top_left, self.top_right, bottom_right]
    }

    // fn vertices(&self) -> impl Iterator<Item = Self::Coorditate> {
    //     self.vertices()
    // }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_bounding_box2d_area() {
        let bbox = BoundingBox2d::new(Vector2::new(0.0, 0.0), Vector2::new(1.0, 1.0));
        assert_eq!(bbox.area(), 1.0);

        let bbox = BoundingBox2d::new(Vector2::new(0.0, 0.0), Vector2::new(2.0, 2.0));
        assert_eq!(bbox.area(), 4.0);

        let bbox = BoundingBox2d::new(Vector2::new(-1.0, 2.0), Vector2::new(1.0, 3.0));
        assert_eq!(bbox.area(), 2.0);
    }

    #[test]
    fn test_bounding_box2d_center() {
        let bbox = BoundingBox2d::new(Vector2::new(0.0, 0.0), Vector2::new(1.0, 1.0));
        assert_eq!(bbox.center(), Vector2::new(0.5, 0.5));

        let bbox = BoundingBox2d::new(Vector2::new(0.0, 0.0), Vector2::new(2.0, 2.0));
        assert_eq!(bbox.center(), Vector2::new(1.0, 1.0));

        let bbox = BoundingBox2d::new(Vector2::new(-1.0, 2.0), Vector2::new(1.0, 3.0));
        assert_eq!(bbox.center(), Vector2::new(0.0, 2.5));
    }

    #[test]
    fn test_bounding_box2d_contains() {
        let bbox = BoundingBox2d::new(Vector2::new(0.0, 0.0), Vector2::new(1.0, 1.0));
        assert!(bbox.contains(&Vector2::new(0.5, 0.5)));
        assert!(!bbox.contains(&Vector2::new(1.5, 0.5)));

        let bbox = BoundingBox2d::new(Vector2::new(-1.0, 2.0), Vector2::new(1.0, 3.0));
        assert!(bbox.contains(&Vector2::new(0.0, 2.5)));
        assert!(!bbox.contains(&Vector2::new(0.0, 3.5)));
    }

    #[test]
    fn test_bounding_box2d_intersects() {
        let bbox1 = BoundingBox2d::new(Vector2::new(0.0, 0.0), Vector2::new(1.0, 1.0));
        let bbox2 = BoundingBox2d::new(Vector2::new(0.5, 0.5), Vector2::new(1.5, 1.5));
        assert!(bbox1.intersects(&bbox2));

        let bbox1 = BoundingBox2d::new(Vector2::new(0.0, 0.0), Vector2::new(1.0, 1.0));
        let bbox2 = BoundingBox2d::new(Vector2::new(1.5, 0.5), Vector2::new(2.5, 1.5));
        assert!(!bbox1.intersects(&bbox2));
    }

    #[test]
    fn test_bounding_box2d_vertices() {
        let bbox = BoundingBox2d::new(Vector2::new(0.0, 0.0), Vector2::new(1.0, 1.0));
        let vertices = bbox.vertices();
        assert_eq!(vertices.len(), 4);
        assert_eq!(vertices[0], bbox.bottom_left);
        assert_eq!(
            vertices[1],
            Vector2::new(bbox.bottom_left.x, bbox.top_right.y)
        );
        assert_eq!(vertices[2], bbox.top_right);
        assert_eq!(
            vertices[3],
            Vector2::new(bbox.top_right.x, bbox.bottom_left.y)
        );
    }
}
