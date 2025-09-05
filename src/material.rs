use crate::math::Vec3;

#[derive(Copy, Clone, Debug)]
pub struct Material {
   pub albedo: Vec3,
   pub specular: f32,
   pub transparency: f32,
   pub reflectivity: f32,
   pub ior: f32,
}
impl Default for Material {
   fn default() -> Self {
      Self {
         albedo: Vec3::new(0.8,0.8,0.8),
         specular: 0.2,
         transparency: 0.0,
         reflectivity: 0.0,
         ior: 1.5,
      }
   }
}

#[derive(Copy, Clone, Debug)]
pub struct Hit {
   pub t: f32,
   pub p: Vec3,
   pub n: Vec3,
   pub mat: Material,
}
