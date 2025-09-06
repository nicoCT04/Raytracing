use crate::{math::Vec3, ray::Ray, material::Hit, scene::Scene, camera::OrbitCam};
use crate::skybox::Skybox;
use crate::lighting::{self, Rng};
use raylib::prelude::*;

pub const W: i32 = 320;
pub const H: i32 = 180;
pub const SCALE: i32 = 4;

fn sky_fallback(dir: Vec3) -> Vec3 {
   let t = 0.5 * (dir.y + 1.0);
   let top = Vec3::new(0.5, 0.7, 1.0);
   let bottom = Vec3::new(1.0, 1.0, 1.0);
   bottom.mul(1.0 - t).add(top.mul(t))
}

/// Devuelve true si `p` está en sombra respecto a una luz direccional `light_dir`.
/// Convención: `light_dir` apunta DESDE el sol hacia la escena (dirección en la que viaja la luz).
/// Para el rayo de sombra, usamos la dirección HACIA la luz: `-light_dir`.
fn in_shadow(scene: &Scene, p: Vec3, n: Vec3, light_dir: Vec3) -> bool {
   let bias = 5e-3; // aumenta si ves acné
   let origin = p.add(n.normalize().mul(bias));
   let dir_to_light = light_dir.normalize().mul(-1.0); // del punto hacia la fuente
   let shadow_ray = Ray { origin, dir: dir_to_light };
   scene.trace(shadow_ray, 0.001, 1e9).is_some()
}

/// Sombrea un punto con luz directa (Lambert + Phong), sombras duras y difusa ambiental (IBL).
/// `light_dir` es dirección DEL sol hacia la escena (rayos de luz).
fn shade(
   scene: &Scene,
   hit: &Hit,
   light_dir: Vec3,
   sky: Option<&Skybox>,
   rng: &mut Rng,
   env_samples: u32,
) -> Vec3 {
   let n = hit.n.normalize();
   let mat = hit.material();

   // Albedo base * textura
   let tex_color = mat.texture.sample(hit.uv);
   let base = mat.albedo.hadamard(tex_color);

   // ----- LUZ DIRECTA con SOMBRA -----
   // Vector hacia la luz: usamos -light_dir (convención coherente)
   let l = light_dir.normalize().mul(-1.0);
   let mut direct = Vec3::new(0.0, 0.0, 0.0);

   // Solo hay directa si el punto "ve" la luz y no está en sombra
   let ndotl = (n.dot(l)).max(0.0);
   if ndotl > 0.0 && !in_shadow(scene, hit.p, n, light_dir) {
      // Difuso (Lambert) con kd
      let diffuse_direct = base.mul(ndotl * mat.kd);

      // Especular Phong sencillo
      let r = n.mul(2.0 * n.dot(l)).sub(l).normalize();
      let view_dir = Vec3::new(0.0, 0.0, 1.0); // aproximación
      let spec = mat.specular * (r.dot(view_dir).max(0.0)).powf(32.0);
      let specular = Vec3::new(spec, spec, spec);

      direct = diffuse_direct.add(specular);
   }

   // ----- DIFUSA AMBIENTAL (IBL) -----
   // Más baja para que las sombras resalten; puedes subirla si quieres.
   let env = if let (Some(sb), s) = (sky, env_samples) {
      lighting::diffuse_env(n, sb, rng, s)
   } else {
      Vec3::new(0.0, 0.0, 0.0)
   };
   let env_strength = 0.25; // antes 0.6
   let diffuse_env = base.hadamard(env).mul(env_strength);

   // Ambient mínimo residual (muy bajo)
   let ambient_min = 0.01;
   let ambient = base.mul(ambient_min);

   direct.add(diffuse_env).add(ambient)
}

pub fn render_scene(
   image: &mut Image,
   scene: &Scene,
   cam: &OrbitCam,
   world_angle: f32,
   sky: Option<&Skybox>,
   env_samples: u32,
   frame_id: u64,
) {
   let eye = cam.eye();
   let (fwd, right, up) = cam.basis();
   let aspect = (W as f32) / (H as f32);
   let half_h = (cam.fov_deg.to_radians() * 0.5).tan();
   let half_w = aspect * half_h;

   // Convención: light_dir es dirección del sol hacia la escena.
   // La rotamos con el mundo para que las sombras se muevan cuando presionas R.
   let light_dir = scene.light_dir.normalize().rot_y(world_angle);

   for y in 0..H {
      for x in 0..W {
         let u = ((x as f32 + 0.5) / (W as f32)) * 2.0 - 1.0;
         let v = 1.0 - ((y as f32 + 0.5) / (H as f32)) * 2.0;

         let dir = fwd.add(right.mul(u * half_w)).add(up.mul(v * half_h)).normalize();

         // Rotar el mundo alrededor del target (equivalente a rotar rayos)
         let center = cam.target;
         let origin_rel = eye.sub(center).rot_y(world_angle).add(center);
         let dir_rot = dir.rot_y(world_angle);

         let ray = Ray { origin: origin_rel, dir: dir_rot };

         // RNG por pixel + frame (para IBL)
         let seed = (frame_id << 32) ^ ((y as u64) << 16) ^ (x as u64);
         let mut rng = Rng::new(seed);

         let rgb = if let Some(h) = scene.trace(ray, 0.001, 1e9) {
               shade(scene, &h, light_dir, sky, &mut rng, env_samples).clamp01()
         } else {
               if let Some(sb) = sky { sb.sample_dir(dir_rot).clamp01() } else { sky_fallback(dir_rot).clamp01() }
         };

         let col = Color::new(
               (rgb.x * 255.0) as u8,
               (rgb.y * 255.0) as u8,
               (rgb.z * 255.0) as u8,
               255
         );
         image.draw_pixel(x, y, col);
      }
   }
}
