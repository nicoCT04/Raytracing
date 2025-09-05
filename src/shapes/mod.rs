use crate::{ray::Ray, material::Hit};

pub trait Hittable {
   fn hit(&self, ray: Ray, tmin: f32, tmax: f32) -> Option<Hit>;
}

pub mod plane;
pub mod cube;

pub use plane::Plane;
pub use cube::Cube;
