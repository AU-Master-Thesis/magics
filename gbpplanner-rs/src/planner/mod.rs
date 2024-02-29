mod factor;
mod factorgraph;
mod marginalise_factor_distance;
mod multivariate_normal;
mod robot;
mod spawner;
mod variable;

pub type Timestep = u32;

// pub trait Scalar: num_traits::Float + Copy + std::fmt::Debug {}
// impl Scalar for f32 {}
// impl Scalar for f64 {}

// only available on nightly :(
// pub type Vector<T> = ndarray::Array1<T: Scalar>;
// pub type Matrix<T> = ndarray::Array2<T: Scalar>;
pub type Vector<T> = ndarray::Array1<T>;
pub type Matrix<T> = ndarray::Array2<T>;

// TODO: finish and implement
pub trait VectorNorm {
    type Scalar: NdFloat;
    fn euclidean_norm(&self) -> Self::Scalar;
    fn l1_norm(&self) -> Self::Scalar;

    #[inline(always)]
    fn l2_norm(&self) -> Self::Scalar {
        self.euclidean_norm()
    }
}

macro_rules! vector_norm_impl {
    ($float:ty) => {
        impl VectorNorm for Vector<$float> {
            type Scalar = $float;
            fn euclidean_norm(&self) -> Self::Scalar {
                <$float>::sqrt(self.fold(0.0, |acc, x| acc + x * x))
                // self.dot(&self).sqrt()
            }
            fn l1_norm(&self) -> Self::Scalar {
                self.fold(0.0, |acc, x| acc + x.abs())
            }
        }
    };
}

vector_norm_impl!(f32);
vector_norm_impl!(f64);

pub trait NdarrayVectorExt: Clone {
    type Scalar: NdFloat;
    fn normalize(&mut self);
    fn normalized(self) -> Self {
        let mut copy = self.clone();
        copy.normalize();
        copy
    }
    // fn euclidian_norm(&self) -> Self::Scalar;
}

macro_rules! ndarray_vector_ext_impl {
    ($float:ty) => {
        impl NdarrayVectorExt for Vector<$float> {
            type Scalar = $float;
            fn normalize(&mut self) {
                let len = self.len() as $float;
                self.map_mut(|x| *x / len);
            }
        }
    };
}

// TODO: write test cases for impls
ndarray_vector_ext_impl!(f32);
ndarray_vector_ext_impl!(f64);

// impl NdarrayVectorExt for Vector<f32> {
//     type Scalar = f32;
//     fn normalize(self) -> Self {
//         let len = self.len() as f32;
//         self.mapv_into(|x| x / len)
//     }
// }

// pub trait NdarrayVectorExt<T>
// where
//     T: NdFloat,
// {
//     fn normalize(self) -> Self {
//         // Provide a default implementation using generics
//         let norm = self.dot(self).sqrt();
//         self.mapv_into(|x| x / norm)
//     }
// }

// impl<T> NdarrayVectorExt<T> for Vector<T> where T: NdFloat {}

// impl<T> TryFrom<Vector<T>> for bevy::math::Vec2 {
//     type Error = &'static str;

//     fn try_from(value: Vector<T>) -> Result<Self, Self::Error> {
//         todo!()
//     }
// }

use self::robot::RobotPlugin;
use self::spawner::SpawnerPlugin;
use bevy::prelude::*;
use ndarray::NdFloat;
// use ndarray_linalg::Lapack;

pub struct PlannerPlugin;

impl Plugin for PlannerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(
            (
                // RobotPlugin,
                SpawnerPlugin
            ),
        );
    }
}
