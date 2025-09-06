use crate::math::Vec3;
use crate::skybox::Skybox;

pub struct Rng { state: u64 }
impl Rng {
   pub fn new(seed: u64) -> Self {
      let s = if seed == 0 { 0x9E37_79B9_7F4A_7C15 } else { seed };
      Self { state: s }
   }
   fn next_u32(&mut self) -> u32 {
      // xorshift64
      let mut x = self.state;
      x ^= x << 13;
      x ^= x >> 7;
      x ^= x << 17;
      self.state = x;
      x as u32
   }
   pub fn next_f32(&mut self) -> f32 {
      (self.next_u32() as f32) / (u32::MAX as f32)
   }
}

fn cosine_hemisphere_sample(r1: f32, r2: f32) -> (f32, f32, f32) {
   // muestreo con densidad ~ cos(theta)
   let phi = 2.0 * std::f32::consts::PI * r1;
   let r = r2.sqrt();
   let x = r * phi.cos();
   let z = r * phi.sin();
   let y = (1.0 - r2).sqrt();
   (x, y, z) // en espacio tangente: y = normal
}

fn build_onb(n: Vec3) -> (Vec3, Vec3, Vec3) {
   let n = n.normalize();
   let a = if n.x.abs() > 0.1 { Vec3::new(0.0, 1.0, 0.0) } else { Vec3::new(1.0, 0.0, 0.0) };
   let t = n.cross(a).normalize();
   let b = t.cross(n).normalize();
   (t, b, n)
}

/// Promedia el skybox sobre el hemisferio alrededor de `n` (cosine-weighted).
pub fn diffuse_env(n: Vec3, sky: &Skybox, rng: &mut Rng, samples: u32) -> Vec3 {
   if samples == 0 { return Vec3::new(0.0, 0.0, 0.0); }
   let (t, b, nn) = build_onb(n);
   let mut acc = Vec3::new(0.0, 0.0, 0.0);
   for _ in 0..samples {
      let r1 = rng.next_f32();
      let r2 = rng.next_f32();
      let (x, y, z) = cosine_hemisphere_sample(r1, r2);
      let dir = t.mul(x).add(nn.mul(y)).add(b.mul(z)).normalize();
      let c = sky.sample_dir(dir);
      acc = acc.add(c);
   }
   acc.mul(1.0 / samples as f32)
}
