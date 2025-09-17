use crate::math::Vec3;
use raylib::prelude::Color;

#[derive(Debug, Clone)]
pub enum Texture {
   None,
   Checker { scale: f32, a: Vec3, b: Vec3 },
   Image   { width: i32, height: i32, pixels: Vec<Color> },
   // Atlas para bloques: top / side / bottom
   BlockAtlas {
      top:   Box<Texture>,
      side:  Box<Texture>,
      bottom:Box<Texture>,
   },
}

impl Texture {
   /// Muestréo normal (una sola textura)
   pub fn sample(&self, uv: (f32, f32)) -> Vec3 {
      self.sample_impl(uv)
   }

   /// Muestréo eligiendo top/side/bottom según la normal (sin tocar cube.rs)
   pub fn sample_with_normal(&self, uv: (f32, f32), n: Vec3) -> Vec3 {
      match self {
         Texture::BlockAtlas { top, side, bottom } => {
               // si la cara es "arriba/abajo" usamos top/bottom; si no, side
               let ny = n.y;
               let chosen = if ny > 0.5 { top.as_ref() }
               else if ny < -0.5 { bottom.as_ref() }
               else { side.as_ref() };
               chosen.sample_impl(uv)
         }
         _ => self.sample_impl(uv),
      }
   }

   fn sample_impl(&self, uv: (f32, f32)) -> Vec3 {
      match self {
         Texture::None => Vec3::new(1.0, 1.0, 1.0),
         Texture::Checker { scale, a, b } => {
               let u = uv.0 * *scale;
               let v = uv.1 * *scale;
               let iu = u.floor() as i32;
               let iv = v.floor() as i32;
               if ((iu + iv) & 1) == 0 { *a } else { *b }
         }
         Texture::Image { width, height, pixels } => {
               let mut u = uv.0 - uv.0.floor();
               let mut v = uv.1 - uv.1.floor();
               if u < 0.0 { u += 1.0; }
               if v < 0.0 { v += 1.0; }
               let x = (u * (*width as f32)) as i32;
               let y = ((1.0 - v) * (*height as f32)) as i32;
               let xi = x.rem_euclid(*width).max(0) as usize;
               let yi = y.clamp(0, *height - 1) as usize;
               let idx = yi * (*width as usize) + xi;
               let c = pixels[idx];
               Vec3::new(c.r as f32 / 255.0, c.g as f32 / 255.0, c.b as f32 / 255.0)
         }
         Texture::BlockAtlas { .. } => unreachable!(), // desviado arriba
      }
   }

   pub fn from_file(path: &str) -> Option<Self> {
      let img = raylib::prelude::Image::load_image(path).ok()?;
      let width = img.width();
      let height = img.height();
      let colors = img.get_image_data();
      let slice: &[Color] = colors.as_ref().as_ref();
      Some(Texture::Image { width, height, pixels: slice.to_vec() })
   }

   /// Crea el atlas desde 3 archivos
   pub fn block_atlas_from_files(top: &str, side: &str, bottom: &str) -> Option<Self> {
      Some(Texture::BlockAtlas {
         top:    Box::new(Texture::from_file(top)?),
         side:   Box::new(Texture::from_file(side)?),
         bottom: Box::new(Texture::from_file(bottom)?),
      })
   }
}
