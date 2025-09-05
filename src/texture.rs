use crate::math::Vec3;
use raylib::prelude::Color;

#[derive(Debug)]
#[allow(dead_code)]
pub enum Texture {
   None,
   Checker { scale: f32, a: Vec3, b: Vec3 },
   Image { width: i32, height: i32, pixels: Vec<Color> }, 
}

impl Texture {
   pub fn sample(&self, uv: (f32, f32)) -> Vec3 {
      match self {
         Texture::None => Vec3::new(1.0, 1.0, 1.0),
         Texture::Checker { scale, a, b } => {
               // UV repeat con "scale", damero en 2D
               let u = uv.0 * scale;
               let v = uv.1 * scale;
               let iu = u.floor() as i32;
               let iv = v.floor() as i32;
               let checker = ((iu + iv) & 1) == 0;
               if checker { *a } else { *b }
         }
         Texture::Image { width, height, pixels } => {
               let mut u = uv.0.fract();
               let mut v = uv.1.fract();
               if u < 0.0 { u += 1.0; }
               if v < 0.0 { v += 1.0; }
               let x = (u * (*width as f32)) as i32;
               let y = ((1.0 - v) * (*height as f32)) as i32; // origen arriba
               let xi = x.clamp(0, *width - 1) as usize;
               let yi = y.clamp(0, *height - 1) as usize;
               let idx = yi * (*width as usize) + xi;
               let c = pixels[idx];
               Vec3::new(c.r as f32 / 255.0, c.g as f32 / 255.0, c.b as f32 / 255.0)
         }
      }
   }
}
