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
    let (mut rl, th) = raylib::init()
        .size(W * SCALE, H * SCALE)
        .title("Rust Raytracer — Reflexión + Refracción + 5 materiales")
        .build();
    rl.set_target_fps(60);

    let mut image = Image::gen_image_color(W, H, Color::BLACK);
    let mut tex = rl.load_texture_from_image(&th, &image).unwrap();

    let sky = Skybox::load("assets/sky.jpg");

    // === Escena ===
    let mut scene = scene::Scene::new(Vec3::new(-0.3, -1.0, -0.2)); // luz direccional

    // 0) Suelo (material 0)
    let plane_mat = Material {
        albedo: Vec3::new(1.0, 1.0, 1.0),
        kd: 0.9,
        specular: 0.05,
        transparency: 0.0,
        reflectivity: 0.0,
        ior: 1.0,
        texture: Texture::Checker { scale: 1.5, a: Vec3::new(0.9,0.9,0.9), b: Vec3::new(0.6,0.6,0.6) },
    };
    scene.add(Box::new(Plane { y: 0.0, mat: plane_mat }));

    // 1) Madera (imagen) — difuso moderado
    let wood_tex = Texture::from_file("assets/cube.png")
        .or_else(|| Texture::from_file("assets/crate.png"))
        .unwrap_or(Texture::Checker { scale: 4.0, a: Vec3::new(0.8,0.6,0.4), b: Vec3::new(0.4,0.3,0.2) });
    let wood = Material {
        albedo: Vec3::new(1.0, 1.0, 1.0), kd: 1.0,
        specular: 0.15, transparency: 0.0, reflectivity: 0.0, ior: 1.0,
        texture: wood_tex,
    };
    scene.add(Box::new(Cube { min: Vec3::new(-1.8,0.5,-0.5), max: Vec3::new(-0.8,1.5,0.5), mat: wood }));

    // 2) Metal (reflectivo)
    let metal = Material {
        albedo: Vec3::new(0.9, 0.9, 0.9), kd: 0.2,
        specular: 0.9, transparency: 0.0, reflectivity: 0.8, ior: 1.0,
        texture: Texture::Checker { scale: 8.0, a: Vec3::new(0.8,0.8,0.8), b: Vec3::new(0.6,0.6,0.6) },
    };
    scene.add(Box::new(Cube { min: Vec3::new(0.3,0.5,-1.2), max: Vec3::new(1.3,1.5,-0.2), mat: metal }));

    // 3) Vidrio (refractivo)
    let glass = Material {
        albedo: Vec3::new(1.0, 1.0, 1.0), kd: 0.05,
        specular: 0.1, transparency: 1.0, reflectivity: 0.08, ior: 1.52,
        texture: Texture::Checker { scale: 6.0, a: Vec3::new(0.95,0.98,1.0), b: Vec3::new(0.9,0.95,1.0) },
    };
    scene.add(Box::new(Cube { min: Vec3::new(-0.5,0.5,-1.8), max: Vec3::new(0.5,1.5,-0.8), mat: glass }));

    // 4) Piedra (difusa)
    let stone = Material {
        albedo: Vec3::new(0.9, 0.9, 0.9), kd: 1.1,
        specular: 0.05, transparency: 0.0, reflectivity: 0.0, ior: 1.0,
        texture: Texture::Checker { scale: 5.0, a: Vec3::new(0.6,0.6,0.65), b: Vec3::new(0.35,0.35,0.4) },
    };
    scene.add(Box::new(Cube { min: Vec3::new(-0.6,0.5,0.6), max: Vec3::new(0.4,1.5,1.6), mat: stone }));

    // 5) Agua (refracción suave, tinte)
    let water = Material {
        albedo: Vec3::new(0.8, 0.9, 1.0), kd: 0.1,
        specular: 0.2, transparency: 0.9, reflectivity: 0.05, ior: 1.33,
        texture: Texture::Checker { scale: 10.0, a: Vec3::new(0.95,1.0,1.0), b: Vec3::new(0.85,0.95,1.0) },
    };
    scene.add(Box::new(Cube { min: Vec3::new(1.2,0.5,0.8), max: Vec3::new(2.2,1.5,1.8), mat: water }));

    // Cámara
    let mut cam = OrbitCam { target: Vec3::new(0.0,1.0,0.0), yaw: 0.8, pitch: -0.2, dist: 4.2, fov_deg: 60.0 };
    let mut autorotate = false;
    let mut world_angle = 0.0_f32;

    // IBL
    let mut env_on = true;
    let env_samples: u32 = 4;
    let mut frame_id: u64 = 1;

    while !rl.window_should_close() {
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
        if rl.is_key_pressed(KeyboardKey::KEY_F) { env_on = !env_on; }

        let samples = if env_on { env_samples } else { 0 };
        render_scene(&mut image, &scene, &cam, world_angle, sky.as_ref(), samples, frame_id);
        frame_id = frame_id.wrapping_add(1);

        // Subir y dibujar
        {
            let colors = image.get_image_data();
            let slice: &[Color] = colors.as_ref().as_ref();
            let mut bytes = Vec::<u8>::with_capacity((W * H * 4) as usize);
            for c in slice.iter() { bytes.push(c.r); bytes.push(c.g); bytes.push(c.b); bytes.push(c.a); }
            tex.update_texture(&bytes).expect("update_texture failed");
        }
        let mut d = rl.begin_drawing(&th);
        d.clear_background(Color::BLACK);
        d.draw_texture_pro(&tex,
            Rectangle { x:0.0, y:0.0, width: W as f32, height: H as f32 },
            Rectangle { x:0.0, y:0.0, width: (W*SCALE) as f32, height: (H*SCALE) as f32 },
            Vector2::zero(), 0.0, Color::WHITE
        );
        d.draw_text("R=rotar | F=IBL on/off | Reflexión+Refracción activos", 8, 8, 16, Color::RAYWHITE);
    }
}
