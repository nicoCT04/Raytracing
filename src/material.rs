use crate::math::Vec3;
use crate::texture::Texture;

// Material ya NO es Copy (porque guarda una Texture)
#[derive(Debug)]
pub struct Material {
   pub albedo: Vec3,
   pub specular: f32,
   pub transparency: f32,
   pub reflectivity: f32,
   pub ior: f32,
   pub texture: Texture,
}

impl Default for Material {
   fn default() -> Self {
      Self {
         albedo: Vec3::new(0.8, 0.8, 0.8),
         specular: 0.2,
         transparency: 0.0,
         reflectivity: 0.0,
         ior: 1.5,
         texture: Texture::None,
      }
   }
}

// Guardamos un puntero solo-lectura al material para mantener Hit como Copy
#[derive(Copy, Clone, Debug)]
pub struct Hit {
   pub t: f32,
   pub p: Vec3,
   pub n: Vec3,
   pub mat_ptr: *const Material,
   pub uv: (f32, f32),
}

impl Hit {
   pub fn material(&self) -> &Material {
      unsafe { &*self.mat_ptr }
   }
}
