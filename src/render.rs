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

fn in_shadow(scene: &Scene, p: Vec3, n: Vec3, light_dir: Vec3) -> bool {
   let bias = 5e-3;
   let origin = p.add(n.normalize().mul(bias));
   let dir_to_light = light_dir.normalize().mul(-1.0);
   let shadow_ray = Ray { origin, dir: dir_to_light };
   scene.trace(shadow_ray, 0.001, 1e9).is_some()
}

fn miss_color(dir: Vec3, sky: Option<&Skybox>) -> Vec3 {
   if let Some(sb) = sky { sb.sample_dir(dir).clamp01() } else { sky_fallback(dir).clamp01() }
}

fn local_shade(scene: &Scene, hit: &Hit, light_dir: Vec3, sky: Option<&Skybox>, rng: &mut Rng, env_samples: u32) -> Vec3 {
   let n = hit.n.normalize();
   let mat = hit.material();

   // *** AQUÍ el cambio: usar atlas top/side/bottom según la normal ***
   let tex_color = mat.texture.sample_with_normal(hit.uv, n);
   let base = mat.albedo.hadamard(tex_color);

   // Luz directa + sombras
   let l = light_dir.normalize().mul(-1.0);
   let mut direct = Vec3::new(0.0, 0.0, 0.0);
   let ndotl = (n.dot(l)).max(0.0);
   if ndotl > 0.0 && !in_shadow(scene, hit.p, n, light_dir) {
      let diffuse_direct = base.mul(ndotl * mat.kd);
      let r = crate::math::Vec3::reflect(l.mul(-1.0), n).normalize();
      let view_dir = Vec3::new(0.0, 0.0, 1.0);
      let spec = mat.specular * (r.dot(view_dir).max(0.0)).powf(32.0);
      let specular = Vec3::new(spec, spec, spec);
      direct = diffuse_direct.add(specular);
   }

   // IBL difusa (bajita para que se note la sombra)
   let env = if let (Some(sb), s) = (sky, env_samples) { lighting::diffuse_env(n, sb, rng, s) } else { Vec3::new(0.0,0.0,0.0) };
   let diffuse_env = base.hadamard(env).mul(0.25);
   let ambient = base.mul(0.01);

   direct.add(diffuse_env).add(ambient)
}

fn trace_color(scene: &Scene, ray: Ray, depth: u32, sky: Option<&Skybox>, env_samples: u32, light_dir: Vec3, rng: &mut Rng) -> Vec3 {
   if depth == 0 { return Vec3::new(0.0,0.0,0.0); }

   if let Some(hit) = scene.trace(ray, 0.001, 1e9) {
      let n = hit.n.normalize();
      let mat = hit.material();
      let local = local_shade(scene, &hit, light_dir, sky, rng, env_samples);

      // Fresnel para mezcla
      let view = ray.dir.mul(-1.0);
      let f0 = mat.reflectivity.max(0.02);
      let fresnel = crate::math::Vec3::fresnel_schlick(view.dot(n).max(0.0), f0);

      let mut accum = local;

      if mat.reflectivity > 0.0 {
         let refl_dir = crate::math::Vec3::reflect(ray.dir, n).normalize();
         let refl_origin = hit.p.add(n.mul(1e-3));
         let refl_col = trace_color(scene, Ray { origin: refl_origin, dir: refl_dir }, depth - 1, sky, env_samples, light_dir, rng);
         accum = accum.add(refl_col.mul(mat.reflectivity * fresnel));
      }

      if mat.transparency > 0.0 {
         let (n1, n2) = (1.0f32, mat.ior.max(1.0));
         let mut n_out = n;
         let mut eta = n1 / n2;
         let mut cosi = (-ray.dir.dot(n)).clamp(-1.0, 1.0);
         if cosi < 0.0 { cosi = -cosi; n_out = n.mul(-1.0); eta = n2 / n1; }
         if let Some(refr_dir) = crate::math::Vec3::refract(ray.dir, n_out, eta) {
               let refr_origin = hit.p.sub(n_out.mul(1e-3));
               let refr_col = trace_color(scene, Ray { origin: refr_origin, dir: refr_dir.normalize() }, depth - 1, sky, env_samples, light_dir, rng);
               let k_trans = mat.transparency * (1.0 - fresnel);
               accum = accum.add(refr_col.mul(k_trans));
         }
      }

      return accum.clamp01();
   }

   miss_color(ray.dir, sky)
}

pub fn render_scene(image: &mut Image, scene: &Scene, cam: &OrbitCam, world_angle: f32, sky: Option<&Skybox>, env_samples: u32, frame_id: u64) {
   let eye = cam.eye();
   let (fwd, right, up) = cam.basis();
   let aspect = (W as f32) / (H as f32);
   let half_h = (cam.fov_deg.to_radians() * 0.5).tan();
   let half_w = aspect * half_h;

   let light_dir = scene.light_dir.normalize().rot_y(world_angle);
   let max_depth = 4;

   for y in 0..H {
      for x in 0..W {
         let u = ((x as f32 + 0.5) / (W as f32)) * 2.0 - 1.0;
         let v = 1.0 - ((y as f32 + 0.5) / (H as f32)) * 2.0;

         let dir = fwd.add(right.mul(u * half_w)).add(up.mul(v * half_h)).normalize();

         let center = cam.target;
         let origin_rel = eye.sub(center).rot_y(world_angle).add(center);
         let dir_rot = dir.rot_y(world_angle);

         let seed = (frame_id << 32) ^ ((y as u64) << 16) ^ (x as u64);
         let mut rng = Rng::new(seed);

         let color = trace_color(scene, Ray { origin: origin_rel, dir: dir_rot }, max_depth, sky, env_samples, light_dir, &mut rng).clamp01();

         let col = Color::new((color.x*255.0) as u8, (color.y*255.0) as u8, (color.z*255.0) as u8, 255);
         image.draw_pixel(x, y, col);
      }
   }
}
