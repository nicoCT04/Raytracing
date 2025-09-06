mod math;
mod ray;
mod camera;
mod material;
mod render;
mod scene;
mod shapes;
mod texture;
mod skybox;
mod lighting;

use raylib::prelude::*;
use math::Vec3;
use material::Material;
use shapes::{Plane, Cube};
use camera::OrbitCam;
use render::{render_scene, W, H, SCALE};
use texture::Texture;
use skybox::Skybox;

fn main() {
    // ----- Init ventana -----
    let (mut rl, th) = raylib::init()
        .size(W * SCALE, H * SCALE)
        .title("Rust Raytracer — Difuso con kd")
        .build();
    rl.set_target_fps(60);

    // Framebuffer
    let mut image = Image::gen_image_color(W, H, Color::BLACK);
    let mut tex = rl.load_texture_from_image(&th, &image).unwrap();

    // Skybox (opcional)
    let sky = Skybox::load("assets/sky.jpg");
    if sky.is_none() {
        eprintln!("(info) No se encontró assets/sky.jpg — usando gradiente como fallback.");
    }

    // ----- Escena base -----
    let mut scene = scene::Scene::new(Vec3::new(-0.3, -1.0, -0.2));

    // Suelo (ligeramente difuso, con algo de especular)
    let plane_mat = Material {
        albedo: Vec3::new(1.0, 1.0, 1.0),
        kd: 0.9,
        specular: 0.05,
        transparency: 0.0,
        reflectivity: 0.0,
        ior: 1.0,
        texture: Texture::Checker {
            scale: 1.5,
            a: Vec3::new(0.9, 0.9, 0.9),
            b: Vec3::new(0.6, 0.6, 0.6),
        },
    };
    scene.add(Box::new(Plane { y: 0.0, mat: plane_mat }));

    // Cubo 100% mate (solo difuso), más “presencia” difusa con kd
    let cube_mat = Material {
        albedo: Vec3::new(1.0, 1.0, 1.0),
        kd: 1.2,
        specular: 0.0, // mate
        transparency: 0.0,
        reflectivity: 0.0,
        ior: 1.0,
        texture: Texture::Checker {
            scale: 4.0,
            a: Vec3::new(0.85, 0.35, 0.35),
            b: Vec3::new(0.25, 0.10, 0.10),
        },
    };
    scene.add(Box::new(Cube {
        min: Vec3::new(-0.5, 0.5, -0.5),
        max: Vec3::new( 0.5, 1.5,  0.5),
        mat: cube_mat
    }));

    // Cámara
    let mut cam = OrbitCam {
        target: Vec3::new(0.0, 1.0, 0.0),
        yaw: 0.8,
        pitch: -0.2,
        dist: 3.0,
        fov_deg: 60.0,
    };

    let mut autorotate = false;
    let mut world_angle = 0.0_f32;

    // Difusa ambiental (IBL) — opcional
    let mut env_on = true;
    let mut env_samples: u32 = 4; // 0,2,4,8,16 recomendado
    let mut frame_id: u64 = 1;

    // ----- Loop -----
    while !rl.window_should_close() {
        // Controles de cámara
        let dt = rl.get_frame_time();
        if rl.is_key_down(KeyboardKey::KEY_LEFT)  { cam.yaw   -= (90.0_f32).to_radians() * dt; }
        if rl.is_key_down(KeyboardKey::KEY_RIGHT) { cam.yaw   += (90.0_f32).to_radians() * dt; }
        if rl.is_key_down(KeyboardKey::KEY_UP)    { cam.pitch += (60.0_f32).to_radians() * dt; }
        if rl.is_key_down(KeyboardKey::KEY_DOWN)  { cam.pitch -= (60.0_f32).to_radians() * dt; }
        cam.pitch = cam.pitch.clamp(-1.4, 1.4);

        let wheel = rl.get_mouse_wheel_move();
        if wheel.abs() > 0.0 { cam.dist = (cam.dist - wheel * 0.3).clamp(1.2, 10.0); }

        if rl.is_key_pressed(KeyboardKey::KEY_R) { autorotate = !autorotate; }
        if autorotate { world_angle += (30.0_f32).to_radians() * dt; }

        // Toggles IBL
        if rl.is_key_pressed(KeyboardKey::KEY_F) { env_on = !env_on; }
        if rl.is_key_pressed(KeyboardKey::KEY_E) {
            env_samples = (env_samples.saturating_mul(2)).min(16).max(1);
        }
        if rl.is_key_pressed(KeyboardKey::KEY_Q) {
            env_samples = (env_samples / 2).min(16); // puede llegar a 0
        }

        // Render
        let sky_ref = sky.as_ref();
        let samples = if env_on { env_samples } else { 0 };
        render_scene(&mut image, &scene, &cam, world_angle, sky_ref, samples, frame_id);
        frame_id = frame_id.wrapping_add(1);

        // Subir framebuffer a GPU
        {
            let colors = image.get_image_data();
            let slice: &[Color] = colors.as_ref().as_ref();
            let mut bytes = Vec::<u8>::with_capacity((W * H * 4) as usize);
            for c in slice.iter() {
                bytes.push(c.r); bytes.push(c.g); bytes.push(c.b); bytes.push(c.a);
            }
            tex.update_texture(&bytes).expect("update_texture failed");
        }

        // Dibujar
        let mut d = rl.begin_drawing(&th);
        d.clear_background(Color::BLACK);
        d.draw_texture_pro(
            &tex,
            Rectangle { x: 0.0, y: 0.0, width: W as f32, height: H as f32 },
            Rectangle { x: 0.0, y: 0.0, width: (W*SCALE) as f32, height: (H*SCALE) as f32 },
            Vector2::zero(),
            0.0,
            Color::WHITE
        );
        d.draw_text("←/→ yaw, ↑/↓ pitch, rueda=zoom, R=rotar diorama", 8, 8, 16, Color::RAYWHITE);
        d.draw_text(&format!("F=IBL {}  Q/E muestras={}  (kd controla difuso)",
            if env_on { "ON" } else { "OFF" }, samples), 8, 28, 16, Color::RAYWHITE);
    }
}
