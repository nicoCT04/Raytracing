use crate::{math::Vec3, ray::Ray, material::Hit};
use crate::shapes::Hittable;

pub struct Scene {
   pub objects: Vec<Box<dyn Hittable>>,
   pub light_dir: Vec3,
}
impl Scene {
   pub fn new(light_dir: Vec3) -> Self { Self { objects: Vec::new(), light_dir } }
   pub fn add(&mut self, o: Box<dyn Hittable>) { self.objects.push(o); }

   pub fn trace(&self, ray: Ray, tmin: f32, tmax: f32) -> Option<Hit> {
      let mut hit: Option<Hit> = None;
      let mut closest = tmax;
      for o in &self.objects {
         if let Some(h) = o.hit(ray, tmin, closest) {
               closest = h.t;
               hit = Some(h);
         }
      }
      hit
   }
}
