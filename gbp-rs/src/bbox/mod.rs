mod bbox2d;
mod bbox3d;

pub use bbox2d::BoundingBox2d;
pub use bbox3d::BoundingBox3d;

// TODO: add better description
/// A bounding box is a rectangular region of space, defined by a minimum and maximum point.
pub trait BoundingBox {
    type Coorditate;

    fn min(&self) -> Self::Coorditate;
    fn max(&self) -> Self::Coorditate;
    fn center(&self) -> Self::Coorditate;
    fn contains(&self, point: &Self::Coorditate) -> bool;
    fn vertices(&self) -> Vec<Self::Coorditate>;
    /// Return the vertices of the bounding box lazily as an iterator
    // fn vertices(&self) -> impl Iterator<Item = Self::Coorditate>;
    fn intersects(&self, other: &Self) -> bool {
        self.vertices().iter().any(|v| other.contains(&v))
    }
}
