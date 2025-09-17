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
use shapes::Cube;
use camera::OrbitCam;
use render::{render_scene, W, H, SCALE};
use texture::Texture;
use skybox::Skybox;

// ---- helpers ----
const BLOCK: f32 = 1.0;

fn add_block(scene: &mut scene::Scene, gx: i32, gy: i32, gz: i32, mat: Material) {
    // grid (gx,gy,gz) -> world AABB
    let min = Vec3::new(gx as f32 * BLOCK, gy as f32 * BLOCK, gz as f32 * BLOCK);
    let max = Vec3::new((gx as f32 + 1.0) * BLOCK, (gy as f32 + 1.0) * BLOCK, (gz as f32 + 1.0) * BLOCK);
    scene.add(Box::new(Cube { min, max, mat }));
}

fn material_grass(tex: Option<Texture>) -> Material {
    Material {
        albedo: Vec3::new(1.0, 1.0, 1.0),
        kd: 1.1,
        specular: 0.05,
        transparency: 0.0,
        reflectivity: 0.0,
        ior: 1.0,
        texture: tex.unwrap_or(Texture::Checker {
            scale: 8.0,
            a: Vec3::new(0.30, 0.60, 0.30),
            b: Vec3::new(0.20, 0.45, 0.20),
        }),
    }
}

fn material_dirt(tex: Option<Texture>) -> Material {
    Material {
        albedo: Vec3::new(1.0, 1.0, 1.0),
        kd: 1.0,
        specular: 0.03,
        transparency: 0.0,
        reflectivity: 0.0,
        ior: 1.0,
        texture: tex.unwrap_or(Texture::Checker {
            scale: 6.0,
            a: Vec3::new(0.45, 0.25, 0.15),
            b: Vec3::new(0.30, 0.18, 0.10),
        }),
    }
}

fn material_stone(tex: Option<Texture>) -> Material {
    Material {
        albedo: Vec3::new(0.95, 0.95, 0.95),
        kd: 1.1,
        specular: 0.02,
        transparency: 0.0,
        reflectivity: 0.0,
        ior: 1.0,
        texture: tex.unwrap_or(Texture::Checker {
            scale: 10.0,
            a: Vec3::new(0.65, 0.65, 0.70),
            b: Vec3::new(0.40, 0.40, 0.45),
        }),
    }
}

fn material_water(tex: Option<Texture>) -> Material {
    Material {
        albedo: Vec3::new(0.85, 0.95, 1.0),
        kd: 0.1,
        specular: 0.2,
        transparency: 0.9,
        reflectivity: 0.05,
        ior: 1.33,
        texture: tex.unwrap_or(Texture::Checker {
            scale: 12.0,
            a: Vec3::new(0.92, 0.98, 1.0),
            b: Vec3::new(0.84, 0.94, 1.0),
        }),
    }
}

fn main() {
    // ventana
    let (mut rl, th) = raylib::init()
        .size(W * SCALE, H * SCALE)
        .title("Rust Raytracer — Isla flotante voxel")
        .build();
    rl.set_target_fps(60);

    // framebuffer
    let mut image = Image::gen_image_color(W, H, Color::BLACK);
    let mut tex = rl.load_texture_from_image(&th, &image).unwrap();

    // skybox (opcional)
    let sky = Skybox::load("assets/sky.jpg");
    if sky.is_none() {
        eprintln!("(info) No se encontró assets/sky.jpg — usando gradiente.");
    }

    // === texturas de imagen (opcionales) ===
    let grass_img = Texture::from_file("assets/grass.png");
    let dirt_img  = Texture::from_file("assets/dirt.png");
    let stone_img = Texture::from_file("assets/stone.png");
    let water_img = Texture::from_file("assets/water.png");

    // === escena (SIN plano; isla flotando) ===
    let mut scene = scene::Scene::new(Vec3::new(-0.25, -1.0, -0.35)); // luz direccional: desde arriba-izq-atrás

    // materiales
    let mat_grass = material_grass(grass_img);
    let mat_dirt  = material_dirt(dirt_img);
    let mat_stone = material_stone(stone_img);
    let mat_water = material_water(water_img);

    // ---- Heightmap 5x5 (capas por columna) ----
    // centro en (0,0). Valores = altura total (piedra+dirt+grass).
    // forma redondeada y cómoda de renderizar.
    let hmap: [[i32; 5]; 5] = [
        [1, 2, 2, 2, 1],
        [2, 3, 3, 3, 2],
        [2, 3, 4, 3, 2],
        [2, 3, 3, 3, 2],
        [1, 2, 2, 2, 1],
    ];

    // elevamos todo para que "flote" (base_y >= 2 deja hueco visible bajo la isla)
    let base_y: i32 = 2;

    // generamos la isla: por cada celda, apilamos piedra->tierra->pasto
    for gz in 0..5 {
        for gx in 0..5 {
            let height = hmap[gz as usize][gx as usize];
            if height <= 0 { continue; }
            // coordenadas centradas (que quede al centro de la escena):
            let gxw = gx - 2;
            let gzw = gz - 2;

            for layer in 0..height {
                let gy = base_y + layer;
                // regla sencilla de materiales:
                // última capa = grass; capas intermedias = dirt; capa 0 = stone
                let mat = if layer == height - 1 {
                    mat_grass.clone()
                } else if layer == 0 {
                    mat_stone.clone()
                } else {
                    mat_dirt.clone()
                };
                add_block(&mut scene, gxw, gy, gzw, mat);
            }
        }
    }

    // Columna de "agua" al borde (cascada)
    // posición: a la izquierda del centro, pegada al borde de la meseta
    for gy in base_y..(base_y + 3) {
        add_block(&mut scene, -3, gy, -1, mat_water.clone());
    }

    // Cámara mirando al centro de la isla
    let mut cam = OrbitCam {
        target: Vec3::new(0.0, (base_y as f32 + 2.0), 0.0),
        yaw: 0.9,
        pitch: -0.35,
        dist: 8.0,
        fov_deg: 60.0,
    };

    let mut autorotate = true; // que rote sola para lucir "flotante"
    let mut world_angle = 0.0_f32;

    // IBL (difusa ambiental)
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
        if wheel.abs() > 0.0 {
            cam.dist = (cam.dist - wheel * 0.3).clamp(3.0, 14.0); // <- rango más amplio
        }

        if rl.is_key_pressed(KeyboardKey::KEY_R) { autorotate = !autorotate; }
        if autorotate { world_angle += (20.0_f32).to_radians() * dt; }

        if rl.is_key_pressed(KeyboardKey::KEY_F) { env_on = !env_on; }

        let samples = if env_on { env_samples } else { 0 };
        render_scene(&mut image, &scene, &cam, world_angle, sky.as_ref(), samples, frame_id);
        frame_id = frame_id.wrapping_add(1);

        // subir y dibujar
        {
            let colors = image.get_image_data();
            let slice: &[Color] = colors.as_ref().as_ref();
            let mut bytes = Vec::<u8>::with_capacity((W * H * 4) as usize);
            for c in slice.iter() { bytes.push(c.r); bytes.push(c.g); bytes.push(c.b); bytes.push(c.a); }
            tex.update_texture(&bytes).expect("update_texture failed");
        }
        let mut d = rl.begin_drawing(&th);
        d.clear_background(Color::BLACK);
        d.draw_texture_pro(
            &tex,
            Rectangle { x:0.0, y:0.0, width: W as f32, height: H as f32 },
            Rectangle { x:0.0, y:0.0, width: (W*SCALE) as f32, height: (H*SCALE) as f32 },
            Vector2::zero(), 0.0, Color::WHITE
        );
        d.draw_text("R=toggle rotación | F=IBL on/off | Isla flotante voxel", 8, 8, 16, Color::RAYWHITE);
    }
}
