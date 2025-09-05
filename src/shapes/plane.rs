use crate::{math::Vec3, ray::Ray, material::{Material, Hit}};
use super::Hittable;

#[derive(Copy, Clone)]
pub struct Plane { pub y: f32, pub mat: Material }

impl Hittable for Plane {
   fn hit(&self, ray: Ray, tmin: f32, tmax: f32) -> Option<Hit> {
      let denom = ray.dir.y;
      if denom.abs() < 1e-4 { return None; }
      let t = (self.y - ray.origin.y) / denom;
      if t > tmin && t < tmax {
         let p = ray.at(t);
         Some(Hit { t, p, n: Vec3::new(0.0,1.0,0.0), mat: self.mat })
      } else { None }
   }
}
