// --- Módulos del proyecto ---
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

// --- Imports ---
use raylib::prelude::*;
use math::Vec3;
use material::Material;
use shapes::Cube;
use camera::OrbitCam;
use render::{render_scene, W, H, SCALE};
use texture::Texture;
use skybox::Skybox;

// ==========================================================
// Helpers personales
// ==========================================================
const BLOCK: f32 = 1.0;

fn add_block(scene: &mut scene::Scene, gx: i32, gy: i32, gz: i32, mat: Material) {
    let min = Vec3::new(gx as f32 * BLOCK, gy as f32 * BLOCK, gz as f32 * BLOCK);
    let max = Vec3::new((gx + 1) as f32 as f32 * BLOCK, (gy + 1) as f32 as f32 * BLOCK, (gz + 1) as f32 as f32 * BLOCK);
    scene.add(Box::new(Cube { min, max, mat }));
}

// Materiales base (fallback a damero si falta imagen)
fn material_from(tex: Option<Texture>, albedo: Vec3, kd: f32, spec: f32, transp: f32, refl: f32, ior: f32, fallback_a: Vec3, fallback_b: Vec3, scale: f32) -> Material {
    Material {
        albedo, kd, specular: spec, transparency: transp, reflectivity: refl, ior,
        texture: tex.unwrap_or(Texture::Checker { scale, a: fallback_a, b: fallback_b }),
    }
}

// ==========================================================
// Main
// ==========================================================
fn main() {
    // Ventana
    let (mut rl, th) = raylib::init()
        .size(W * SCALE, H * SCALE)
        .title("Rust Raytracer — Isla flotante con cueva")
        .build();
    rl.set_target_fps(60);

    // Framebuffer
    let mut image = Image::gen_image_color(W, H, Color::BLACK);
    let mut tex = rl.load_texture_from_image(&th, &image).unwrap();

    // Skybox
    let sky = Skybox::load("assets/sky.jpg");
    if sky.is_none() { eprintln!("(info) No se encontró assets/sky.jpg — usando gradiente."); }

    // Cargo texturas
    let tex_grass_top  = Texture::from_file("assets/frontgrass.png");
    let tex_grass_side = Texture::from_file("assets/grass.png");
    let tex_dirt       = Texture::from_file("assets/dirt.png");
    let tex_stone      = Texture::from_file("assets/stone.png");
    let tex_water      = Texture::from_file("assets/water.png");
    let tex_wood       = Texture::from_file("assets/wood.png");
    let tex_leaf       = Texture::from_file("assets/leaf.png");
    let tex_diamond    = Texture::from_file("assets/diamond.png");
    let tex_tnt        = Texture::from_file("assets/tnt.png");

    // Atlas de pasto (top/side/bottom)
    let grass_atlas = Texture::block_atlas_from_files(
        "assets/frontgrass.png",
        "assets/grass.png",
        "assets/dirt.png",
    ).unwrap_or(Texture::Checker { scale: 8.0, a: Vec3::new(0.30,0.60,0.30), b: Vec3::new(0.20,0.45,0.20) });

    // Materiales
    let mat_grass = material_from(Some(grass_atlas), Vec3::new(1.0,1.0,1.0), 1.1, 0.05, 0.0, 0.0, 1.0, Vec3::new(0.3,0.6,0.3), Vec3::new(0.2,0.45,0.2), 8.0);
    let mat_dirt  = material_from(tex_dirt,  Vec3::new(1.0,1.0,1.0), 1.0, 0.03, 0.0, 0.0, 1.0, Vec3::new(0.45,0.25,0.15), Vec3::new(0.30,0.18,0.10), 6.0);
    let mat_stone = material_from(tex_stone, Vec3::new(0.95,0.95,0.95), 1.1, 0.02, 0.0, 0.0, 1.0, Vec3::new(0.65,0.65,0.70), Vec3::new(0.40,0.40,0.45), 10.0);
    let mat_water = material_from(tex_water, Vec3::new(0.85,0.95,1.0), 0.1, 0.2, 0.9, 0.05, 1.33, Vec3::new(0.92,0.98,1.0), Vec3::new(0.84,0.94,1.0), 12.0);
    let mat_wood  = material_from(tex_wood,  Vec3::new(1.0,1.0,1.0), 1.0, 0.10, 0.0, 0.0, 1.0, Vec3::new(0.60,0.40,0.20), Vec3::new(0.40,0.25,0.15), 6.0);
    let mat_leaf  = material_from(tex_leaf,  Vec3::new(0.9,1.0,0.9), 1.0, 0.05, 0.0, 0.0, 1.0, Vec3::new(0.20,0.45,0.20), Vec3::new(0.15,0.35,0.15), 10.0);
    let mat_diamond = material_from(tex_diamond, Vec3::new(1.0,1.0,1.0), 0.9, 0.4, 0.0, 0.2, 1.0, Vec3::new(0.6,0.9,1.0), Vec3::new(0.4,0.7,0.9), 8.0);
    let mat_tnt = material_from(tex_tnt, Vec3::new(1.0,1.0,1.0), 0.9, 0.1, 0.0, 0.0, 1.0, Vec3::new(0.9,0.3,0.3), Vec3::new(0.7,0.15,0.15), 8.0);

    // === Escena (sin plano; isla flotante) ===
    let mut scene = scene::Scene::new(Vec3::new(-0.25, -1.0, -0.35));

    // Heightmap 5x5 (capas por columna)
    let hmap: [[i32; 5]; 5] = [
        [1, 2, 2, 2, 1],
        [2, 3, 3, 3, 2],
        [2, 3, 4, 3, 2],
        [2, 3, 3, 3, 2],
        [1, 2, 2, 2, 1],
    ];
    let base_y: i32 = 2;

    for gz in 0..5 {
        for gx in 0..5 {
            let h = hmap[gz as usize][gx as usize];
            if h <= 0 { continue; }
            let gxw = gx - 2;
            let gzw = gz - 2;

            for layer in 0..h {
                let gy = base_y + layer;
                let is_top = layer == h - 1;
                let mat = if is_top { mat_grass.clone() } else if layer == 0 { mat_stone.clone() } else { mat_dirt.clone() };
                add_block(&mut scene, gxw, gy, gzw, mat);
            }
        }
    }

    // Cascada (columna) + bloque fuente pegado a tierra
    for gy in base_y..(base_y + 3) { add_block(&mut scene, -3, gy, -1, mat_water.clone()); }
    add_block(&mut scene, -2, base_y + 2, -1, mat_water.clone()); // “source” tocando la meseta

    // Árbol en esquina (frontal derecha: gx=2, gz=0)
    let tree_gx = 2;
    let tree_gz = 0;
    let tree_col_h = hmap[(tree_gz + 2) as usize][(tree_gx + 2) as usize];
    let tree_top_y = base_y + tree_col_h - 1;
    for i in 1..=3 { add_block(&mut scene, tree_gx, tree_top_y + i, tree_gz, mat_wood.clone()); }
    let crown_y = tree_top_y + 3;
    for dz in -1..=1 { for dx in -1..=1 { add_block(&mut scene, tree_gx + dx, crown_y, tree_gz + dz, mat_leaf.clone()); } }
    add_block(&mut scene, tree_gx, crown_y + 1, tree_gz, mat_leaf.clone());

    // === Mini cueva colgante bajo la isla ===
    // Caja de 3×2×3 (x,z,y) justo bajo la isla, con 2 aperturas.
    let top = base_y - 1;   // techo de la cueva
    let height = 2;         // dos niveles hacia abajo (ligero)
    let x0 = -1; let x1 = 1;
    let z0 =  0; let z1 = 2;

    // Reservo DOS huecos en la pared trasera para diamantes
    let diamond_slot_a = (x1, z1); // trasera derecha
    let diamond_slot_b = (x0, z1); // trasera izquierda

    // Muros/techo: solo perímetro. Aperturas:
    // - frontal (z=z0) de 2 bloques ancho, 1 de alto (en el nivel inferior)
    // - lateral izquierda (x=x0) 1×1 (ventana) en el nivel inferior
    for dy in 0..height {
        let gy = top - dy;
        for gz in z0..=z1 {
            for gx in x0..=x1 {
                let edge = gx == x0 || gx == x1 || gz == z0 || gz == z1;
                if !edge { continue; }

                // Aperturas visibles:
                let front_open = gz == z0 && (gx == 0 || gx == 1) && dy == 1;
                let side_open  = gx == x0 && gz == 1 && dy == 1;

                // Huecos para diamante (solo nivel inferior, dy==1)
                let diamond_hole = dy == 1 && ((gx, gz) == diamond_slot_a || (gx, gz) == diamond_slot_b);

                if front_open || side_open || diamond_hole { continue; }

                add_block(&mut scene, gx, gy, gz, mat_stone.clone());
            }
        }
    }

    // Suelo (loseta completa en el nivel inferior)
    for gz in (z0+1)..=(z1-1) {
        for gx in (x0+1)..=(x1-1) {
            add_block(&mut scene, gx, top - (height - 1), gz, mat_stone.clone());
        }
    }

    // TNT en la entrada frontal (visible)
    add_block(&mut scene, 0, top - 1, z0, mat_tnt.clone());

    // Diamantes en los huecos de la pared trasera (ahora SIN piedra detrás)
    add_block(&mut scene, diamond_slot_a.0, top - 1, diamond_slot_a.1, mat_diamond.clone());
    add_block(&mut scene, diamond_slot_b.0, top - 1, diamond_slot_b.1, mat_diamond.clone());

    // Cámara
    let mut cam = OrbitCam { target: Vec3::new(0.0, base_y as f32 + 2.0, 0.0), yaw: 0.9, pitch: -0.50, dist: 9.2, fov_deg: 60.0 };

    let mut autorotate = true;
    let mut world_angle = 0.0_f32;

    let mut env_on = true;
    let env_samples: u32 = 4;
    let mut frame_id: u64 = 1;

    // Loop
    while !rl.window_should_close() {
        let dt = rl.get_frame_time();
        if rl.is_key_down(KeyboardKey::KEY_LEFT)  { cam.yaw   -= (90.0_f32).to_radians() * dt; }
        if rl.is_key_down(KeyboardKey::KEY_RIGHT) { cam.yaw   += (90.0_f32).to_radians() * dt; }
        if rl.is_key_down(KeyboardKey::KEY_UP)    { cam.pitch += (60.0_f32).to_radians() * dt; }
        if rl.is_key_down(KeyboardKey::KEY_DOWN)  { cam.pitch -= (60.0_f32).to_radians() * dt; }
        cam.pitch = cam.pitch.clamp(-1.4, 1.4);

        let wheel = rl.get_mouse_wheel_move();
        if wheel.abs() > 0.0 { cam.dist = (cam.dist - wheel * 0.3).clamp(3.0, 14.0); }

        if rl.is_key_pressed(KeyboardKey::KEY_R) { autorotate = !autorotate; }
        if autorotate { world_angle += (20.0_f32).to_radians() * dt; }

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
        d.draw_text("R=rotación | F=IBL on/off | Isla flotante con cueva", 8, 8, 16, Color::RAYWHITE);
    }
}
