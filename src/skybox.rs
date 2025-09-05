use raylib::prelude::*;
use crate::math::Vec3;

pub struct Skybox {
   pub width: i32,
   pub height: i32,
   pub pixels: Vec<Color>,
}

impl Skybox {
   // Carga directa desde archivo a RAM (no crea Texture2D).
   pub fn load(path: &str) -> Option<Self> {
      let img = Image::load_image(path).ok()?;
      let width = img.width();
      let height = img.height();

      // Extrae colores a CPU
      let colors = img.get_image_data();      // ImageColors
      let slice: &[Color] = colors.as_ref().as_ref();
      Some(Self { width, height, pixels: slice.to_vec() })
   }

   // Mapea dirección -> (u,v) equirectangulares y muestrea.
   pub fn sample_dir(&self, dir: Vec3) -> Vec3 {
      let d = dir.normalize();
      let u = 0.5 + 0.5 * d.z.atan2(d.x) / std::f32::consts::PI;
      let v = (d.y.clamp(-1.0, 1.0)).acos() / std::f32::consts::PI;

      let x = (u * self.width as f32) as i32;
      let y = (v * self.height as f32) as i32;
      let xi = x.rem_euclid(self.width);
      let yi = y.clamp(0, self.height - 1);

      let idx = (yi as usize) * (self.width as usize) + (xi as usize);
      let c = self.pixels[idx];
      Vec3::new(c.r as f32 / 255.0, c.g as f32 / 255.0, c.b as f32 / 255.0)
   }
}
