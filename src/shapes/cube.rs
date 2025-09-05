use crate::{math::Vec3, ray::Ray, material::{Material, Hit}};
use super::Hittable;

pub struct Cube { pub min: Vec3, pub max: Vec3, pub mat: Material }

impl Hittable for Cube {
   fn hit(&self, ray: Ray, tmin: f32, tmax: f32) -> Option<Hit> {
      // AABB por slabs
      let inv = Vec3::new(1.0/ray.dir.x, 1.0/ray.dir.y, 1.0/ray.dir.z);

      let (tx1, tx2) = ((self.min.x - ray.origin.x) * inv.x, (self.max.x - ray.origin.x) * inv.x);
      let mut tmin_acc = tx1.min(tx2);
      let mut tmax_acc = tx1.max(tx2);

      let (ty1, ty2) = ((self.min.y - ray.origin.y) * inv.y, (self.max.y - ray.origin.y) * inv.y);
      tmin_acc = tmin_acc.max(ty1.min(ty2));
      tmax_acc = tmax_acc.min(ty1.max(ty2));

      let (tz1, tz2) = ((self.min.z - ray.origin.z) * inv.z, (self.max.z - ray.origin.z) * inv.z);
      tmin_acc = tmin_acc.max(tz1.min(tz2));
      tmax_acc = tmax_acc.min(tz1.max(tz2));

      if tmax_acc >= tmin_acc.max(tmin) && tmin_acc < tmax {
         let t = if tmin_acc > tmin { tmin_acc } else { tmax_acc };
         if !(t > tmin && t < tmax) { return None; }
         let p = ray.at(t);
         let eps = 1e-3;
         let (n, uv) = if (p.x - self.min.x).abs() < eps {
               // -X
               let u = (p.z - self.min.z) / (self.max.z - self.min.z);
               let v = (p.y - self.min.y) / (self.max.y - self.min.y);
               (Vec3::new(-1.0,0.0,0.0), (u, v))
         } else if (p.x - self.max.x).abs() < eps {
               // +X
               let u = 1.0 - (p.z - self.min.z) / (self.max.z - self.min.z);
               let v = (p.y - self.min.y) / (self.max.y - self.min.y);
               (Vec3::new(1.0,0.0,0.0), (u, v))
         } else if (p.y - self.min.y).abs() < eps {
               // -Y
               let u = (p.x - self.min.x) / (self.max.x - self.min.x);
               let v = 1.0 - (p.z - self.min.z) / (self.max.z - self.min.z);
               (Vec3::new(0.0,-1.0,0.0), (u, v))
         } else if (p.y - self.max.y).abs() < eps {
               // +Y
               let u = (p.x - self.min.x) / (self.max.x - self.min.x);
               let v = (p.z - self.min.z) / (self.max.z - self.min.z);
               (Vec3::new(0.0,1.0,0.0), (u, v))
         } else if (p.z - self.min.z).abs() < eps {
               // -Z
               let u = (p.x - self.min.x) / (self.max.x - self.min.x);
               let v = (p.y - self.min.y) / (self.max.y - self.min.y);
               (Vec3::new(0.0,0.0,-1.0), (u, v))
         } else {
               // +Z
               let u = 1.0 - (p.x - self.min.x) / (self.max.x - self.min.x);
               let v = (p.y - self.min.y) / (self.max.y - self.min.y);
               (Vec3::new(0.0,0.0,1.0), (u, v))
         };

         Some(Hit {
               t, p, n,
               mat_ptr: &self.mat as *const Material,
               uv
         })
      } else { None }
   }
}
