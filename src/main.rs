mod math;
mod ray;
mod camera;
mod material;
mod render;
mod scene;
mod shapes;

use raylib::prelude::*;
use math::Vec3;
use material::Material;
use shapes::{Plane, Cube};
use camera::OrbitCam;
use render::{render_scene, W, H, SCALE};

fn main() {
    let (mut rl, th) = raylib::init()
        .size(W*SCALE, H*SCALE)
        .title("Rust Raytracer — Paso 1 (modular)")
        .build();
    rl.set_target_fps(60);

    let mut image = Image::gen_image_color(W, H, Color::BLACK);
    let mut tex = rl.load_texture_from_image(&th, &image).unwrap();

    // ===== Escena base =====
    let mut scene = scene::Scene::new(Vec3::new(-0.3, -1.0, -0.2));
    // Plano
    scene.add(Box::new(Plane {
        y: 0.0,
        mat: Material { albedo: Vec3::new(0.9,0.9,0.9), specular: 0.1, transparency: 0.0, reflectivity: 0.0, ior: 1.0 }
    }));
    // Cubo
    scene.add(Box::new(Cube {
        min: Vec3::new(-0.5, 0.5, -0.5),
        max: Vec3::new( 0.5, 1.5,  0.5),
        mat: Material { albedo: Vec3::new(0.85,0.35,0.35), specular: 0.4, transparency: 0.0, reflectivity: 0.0, ior: 1.0 }
    }));

    // Cámara orbital
    let mut cam = OrbitCam { target: Vec3::new(0.0,1.0,0.0), yaw: 0.8, pitch: -0.2, dist: 3.0, fov_deg: 60.0 };

    let mut autorotate = false;

    while !rl.window_should_close() {
        // Controles
        let dt = rl.get_frame_time();
        if rl.is_key_down(KeyboardKey::KEY_LEFT)  { cam.yaw -= (90.0_f32).to_radians() * dt; }
        if rl.is_key_down(KeyboardKey::KEY_RIGHT) { cam.yaw += (90.0_f32).to_radians() * dt; }
        if rl.is_key_down(KeyboardKey::KEY_UP)    { cam.pitch += (60.0_f32).to_radians() * dt; }
        if rl.is_key_down(KeyboardKey::KEY_DOWN)  { cam.pitch -= (60.0_f32).to_radians() * dt; }
        cam.pitch = cam.pitch.clamp(-1.4, 1.4);
        let wheel = rl.get_mouse_wheel_move();
        if wheel.abs() > 0.0 { cam.dist = (cam.dist - wheel * 0.3).clamp(1.2, 10.0); }
        if rl.is_key_pressed(KeyboardKey::KEY_R) { autorotate = !autorotate; }
        if autorotate { cam.yaw += (30.0_f32).to_radians() * dt; }

        // Render CPU al image
        render_scene(&mut image, &scene, &cam);

        // Subir textura y dibujar
        {
            // get_image_data() -> ImageColors (no iterable directamente)
            let colors = image.get_image_data();

            // Convierte a slice &[Color]
            let slice: &[Color] = colors.as_ref().as_ref();

            // Empaqueta a RGBA8
            let mut bytes = Vec::<u8>::with_capacity((W * H * 4) as usize);
            for c in slice.iter() {
                bytes.push(c.r);
                bytes.push(c.g);
                bytes.push(c.b);
                bytes.push(c.a);
            }

            // Sube al GPU (Texture2D::update_texture)
            tex.update_texture(&bytes).expect("update_texture failed");
        }


        let mut d = rl.begin_drawing(&th);
        d.clear_background(Color::BLACK);
        d.draw_texture_pro(
            &tex,
            Rectangle { x:0.0, y:0.0, width: W as f32, height: H as f32 },
            Rectangle { x:0.0, y:0.0, width: (W*SCALE) as f32, height: (H*SCALE) as f32 },
            Vector2::zero(),
            0.0,
            Color::WHITE
        );
        d.draw_text("←/→ yaw, ↑/↓ pitch, rueda=zoom, R=auto-rotate", 8, 8, 16, Color::RAYWHITE);
        d.draw_text("Paso 1: Plano + Cubo + Luz + Cielo gradiente (modular)", 8, 28, 16, Color::RAYWHITE);
    }
}
