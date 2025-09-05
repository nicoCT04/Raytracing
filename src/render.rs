use crate::{math::Vec3, ray::Ray, material::Hit, scene::Scene, camera::OrbitCam};
use raylib::prelude::*;

pub const W: i32 = 320;
pub const H: i32 = 180;
pub const SCALE: i32 = 4;

fn sky(dir: Vec3) -> Vec3 {
   let t = 0.5 * (dir.y + 1.0);
   let top = Vec3::new(0.5, 0.7, 1.0);
   let bottom = Vec3::new(1.0, 1.0, 1.0);
   bottom.mul(1.0 - t).add(top.mul(t))
}

fn shade(hit: &Hit, light_dir: Vec3) -> Vec3 {
   let ambient = 0.05;
   let n = hit.n.normalize();
   let l = light_dir.normalize();
   let diff = (n.dot(l)).max(0.0);

   let mat = hit.material();
   let tex_color = mat.texture.sample(hit.uv);
   let base = mat.albedo.hadamard(tex_color);

   let diffuse = base.mul(diff);

   let r = n.mul(2.0 * n.dot(l)).sub(l).normalize();
   let view_dir = Vec3::new(0.0, 0.0, 1.0);
   let spec = mat.specular * (r.dot(view_dir).max(0.0)).powf(32.0);

   diffuse.add(Vec3::new(spec, spec, spec)).add(base.mul(ambient))
}

pub fn render_scene(image: &mut Image, scene: &Scene, cam: &OrbitCam, world_angle: f32) {
   let eye = cam.eye();
   let (fwd, right, up) = cam.basis();
   let aspect = (W as f32)/(H as f32);
   let half_h = (cam.fov_deg.to_radians()*0.5).tan();
   let half_w = aspect * half_h;

   let light_dir = scene.light_dir.normalize().rot_y(world_angle);

   for y in 0..H {
      for x in 0..W {
         let u = ((x as f32 + 0.5)/(W as f32)) * 2.0 - 1.0;
         let v = 1.0 - ((y as f32 + 0.5)/(H as f32)) * 2.0;

         let dir = fwd.add(right.mul(u*half_w)).add(up.mul(v*half_h)).normalize();

         // Rotar el mundo (equivalente a rotar origen y dirección del rayo alrededor del target)
         let center = cam.target;
         let origin_rel = eye.sub(center).rot_y(world_angle).add(center);
         let dir_rot = dir.rot_y(world_angle);

         let ray = Ray { origin: origin_rel, dir: dir_rot };

         let rgb = if let Some(h) = scene.trace(ray, 0.001, 1e9) {
               shade(&h, light_dir).clamp01()
         } else {
               sky(dir_rot).clamp01()
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
