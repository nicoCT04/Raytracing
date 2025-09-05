#[derive(Copy, Clone, Debug, Default)]
pub struct Vec3 { pub x: f32, pub y: f32, pub z: f32 }

impl Vec3 {
   pub fn new(x: f32, y: f32, z: f32) -> Self { Self { x, y, z } }
   pub fn add(self, o: Vec3) -> Vec3 { Vec3::new(self.x+o.x, self.y+o.y, self.z+o.z) }
   pub fn sub(self, o: Vec3) -> Vec3 { Vec3::new(self.x-o.x, self.y-o.y, self.z-o.z) }
   pub fn mul(self, k: f32) -> Vec3 { Vec3::new(self.x*k, self.y*k, self.z*k) }
   pub fn dot(self, o: Vec3) -> f32 { self.x*o.x + self.y*o.y + self.z*o.z }
   pub fn cross(self, o: Vec3) -> Vec3 {
      Vec3::new(self.y*o.z - self.z*o.y, self.z*o.x - self.x*o.z, self.x*o.y - self.y*o.x)
   }
   pub fn length(self) -> f32 { self.dot(self).sqrt() }
   pub fn normalize(self) -> Vec3 { let l=self.length(); if l>0.0 { self.mul(1.0/l) } else { self } }
   pub fn clamp01(self) -> Vec3 {
      fn c(v:f32)->f32{ v.clamp(0.0,1.0) }
      Vec3::new(c(self.x), c(self.y), c(self.z))
   }
   pub fn hadamard(self, o: Vec3) -> Vec3 {
      Vec3::new(self.x * o.x, self.y * o.y, self.z * o.z)
   }
   pub fn rot_y(self, ang: f32) -> Vec3 {
      let c = ang.cos(); let s = ang.sin();
      Vec3::new(self.x * c + self.z * s, self.y, -self.x * s + self.z * c)
   }

}
