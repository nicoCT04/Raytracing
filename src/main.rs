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

// --- Imports útiles ---
use raylib::prelude::*;
use math::Vec3;
use material::Material;
use shapes::Cube;
use camera::OrbitCam;
use render::{render_scene, W, H, SCALE};
use texture::Texture;
use skybox::Skybox;

// ==========================================================
// Helpers personales (bloques y materiales)
// ==========================================================

const BLOCK: f32 = 1.0; // decidí que cada bloque mida 1×1×1

// Agrego un bloque 1×1×1 en coordenadas de grilla (gx,gy,gz)
fn add_block(scene: &mut scene::Scene, gx: i32, gy: i32, gz: i32, mat: Material) {
    let min = Vec3::new(gx as f32 * BLOCK, gy as f32 * BLOCK, gz as f32 * BLOCK);
    let max = Vec3::new((gx + 1) as f32 as f32 * BLOCK, (gy + 1) as f32 as f32 * BLOCK, (gz + 1) as f32 as f32 * BLOCK);
    scene.add(Box::new(Cube { min, max, mat }));
}

// A veces quiero “slabs” (capas finas). Con esto puedo poner, por ejemplo,
// una tapa de pasto delgadita arriba del bloque de tierra.
fn add_slab_top(scene: &mut scene::Scene, gx: i32, gy: i32, gz: i32, thickness: f32, mat: Material) {
    let y0 = gy as f32 * BLOCK + (BLOCK - thickness);
    let min = Vec3::new(gx as f32 * BLOCK, y0, gz as f32 * BLOCK);
    let max = Vec3::new((gx + 1) as f32 as f32 * BLOCK, gy as f32 as f32 * BLOCK + BLOCK, (gz + 1) as f32 as f32 * BLOCK);
    scene.add(Box::new(Cube { min, max, mat }));
}

// Materiales base (con fallback a damero si no existiera la imagen)
fn material_grass_top(tex: Option<Texture>) -> Material {
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

fn material_wood(tex: Option<Texture>) -> Material {
    Material {
        albedo: Vec3::new(1.0, 1.0, 1.0),
        kd: 1.0,
        specular: 0.10,
        transparency: 0.0,
        reflectivity: 0.0,
        ior: 1.0,
        texture: tex.unwrap_or(Texture::Checker {
            scale: 6.0,
            a: Vec3::new(0.60, 0.40, 0.20),
            b: Vec3::new(0.40, 0.25, 0.15),
        }),
    }
}

fn material_leaf(tex: Option<Texture>) -> Material {
    Material {
        albedo: Vec3::new(0.9, 1.0, 0.9),
        kd: 1.0,
        specular: 0.05,
        transparency: 0.0, // si más adelante quiero translucidez: 0.3 e ior 1.05
        reflectivity: 0.0,
        ior: 1.0,
        texture: tex.unwrap_or(Texture::Checker {
            scale: 10.0,
            a: Vec3::new(0.20, 0.45, 0.20),
            b: Vec3::new(0.15, 0.35, 0.15),
        }),
    }
}

// ==========================================================
// Main
// ==========================================================

fn main() {
    // Inicializo la ventana
    let (mut rl, th) = raylib::init()
        .size(W * SCALE, H * SCALE)
        .title("Rust Raytracer — Isla flotante voxel")
        .build();
    rl.set_target_fps(60);

    // Framebuffer (CPU) y textura (GPU)
    let mut image = Image::gen_image_color(W, H, Color::BLACK);
    let mut tex = rl.load_texture_from_image(&th, &image).unwrap();

    // Cargo el skybox (si no está, uso gradiente)
    let sky = Skybox::load("assets/sky.jpg");
    if sky.is_none() {
        eprintln!("(info) No se encontró assets/sky.jpg — usando gradiente.");
    }

    // Cargo texturas de imagen que quiero usar (si faltan, caen a damero)
    let tex_grass_top = Texture::from_file("assets/frontgrass.png"); // yo elegí una verde lisa para la tapa
    let tex_grass_side = Texture::from_file("assets/grass.png");     // NOTA: hoy no la uso per-cara (ver comentario abajo)
    let tex_dirt  = Texture::from_file("assets/dirt.png");
    let tex_stone = Texture::from_file("assets/stone.png");
    let tex_water = Texture::from_file("assets/water.png");
    let tex_wood  = Texture::from_file("assets/wood.png");
    let tex_leaf  = Texture::from_file("assets/leaf.png");

    // Creo la escena SIN plano (así mi isla realmente “flota”)
    let mut scene = scene::Scene::new(Vec3::new(-0.25, -1.0, -0.35)); // luz desde arriba-izq-atrás

    // Materiales base
    let mat_grass_top = material_grass_top(tex_grass_top);
    let mat_dirt       = material_dirt(tex_dirt);
    let mat_stone      = material_stone(tex_stone);
    let mat_water      = material_water(tex_water);
    let mat_wood       = material_wood(tex_wood);
    let mat_leaf       = material_leaf(tex_leaf);

    // ----------------------------------------------------------------
    // Genero la “isla” a partir de un heightmap pequeño (5×5)
    // Decidí elevar la base (base_y) para que se perciba flotando.
    // Para evitar el problema del “pasto de lado” sin tocar el sombreador,
    // hago cada columna como: bloque de stone/dirt + una “tapa” (slab) de pasto.
    // Así la parte superior es verde y los laterales quedan de tierra o piedra.
    // ----------------------------------------------------------------
    let hmap: [[i32; 5]; 5] = [
        [1, 2, 2, 2, 1],
        [2, 3, 3, 3, 2],
        [2, 3, 4, 3, 2],
        [2, 3, 3, 3, 2],
        [1, 2, 2, 2, 1],
    ];
    let base_y: i32 = 2; // altura de todo el bloque para que flote
    let grass_thickness = 0.50; // mi “slab” superior (en unidades de bloque)

    for gz in 0..5 {
        for gx in 0..5 {
            let height = hmap[gz as usize][gx as usize];
            if height <= 0 { continue; }
            let gxw = gx - 2; // centro la isla en el origen
            let gzw = gz - 2;

            for layer in 0..height {
                let gy = base_y + layer;

                // la capa más baja: piedra
                if layer == 0 {
                    add_block(&mut scene, gxw, gy, gzw, mat_stone.clone());
                } else {
                    // capas intermedias de tierra
                    add_block(&mut scene, gxw, gy, gzw, mat_dirt.clone());
                }

                // si esta es la última capa, le pongo una tapa de pasto finita
                if layer == height - 1 {
                    add_slab_top(&mut scene, gxw, gy, gzw, grass_thickness, mat_grass_top.clone());
                    // (NOTA: si en el futuro implemento textura por cara,
                    // podré usar grass_side para los laterales del mismo bloque)
                }
            }
        }
    }

    // Columna de agua “cascada” al costado izquierdo de la meseta
    let source_y = base_y + 2;         // un peldaño debajo del tope (según hmap actual)
    add_block(&mut scene, -2, source_y, -1, mat_water.clone());
    for gy in base_y..(base_y + 3) {
        add_block(&mut scene, -3, gy, -1, mat_water.clone());
    }

    // ----------------------------------------------------------------
    // Árbol sencillo: tronco de 3 bloques + copa 3×3 + 1 bloque extra arriba
    // Lo coloco sobre el “tile” central (0,0) de la isla
    // ----------------------------------------------------------------
    let tree_gx = 2;
    let tree_gz = 0;
    let tree_col_h = hmap[(tree_gz + 2) as usize][(tree_gx + 2) as usize]; // leo altura en esa celda
    let tree_top_y = base_y + tree_col_h - 1;

    // tronco de 3 bloques
    for i in 1..=3 {
        add_block(&mut scene, tree_gx, tree_top_y + i, tree_gz, mat_wood.clone());
    }

    // copa 3x3 alrededor del tope del tronco + tapa superior
    let crown_y = tree_top_y + 3;
    for dz in -1..=1 {
        for dx in -1..=1 {
            add_block(&mut scene, tree_gx + dx, crown_y, tree_gz + dz, mat_leaf.clone());
        }
    }
    add_block(&mut scene, tree_gx, crown_y + 1, tree_gz, mat_leaf.clone());

    // ----------------------------------------------------------------
    // Cámara: la alejo para que se aprecie completa y dejo rotación automática
    // ----------------------------------------------------------------
    let mut cam = OrbitCam {
        target: Vec3::new(0.0, base_y as f32 + 2.0, 0.0),
        yaw: 0.9,
        pitch: -0.35,
        dist: 8.0,      // la dejo más lejos para ver el diorama cómodo
        fov_deg: 60.0,
    };

    let mut autorotate = true;  // me gusta que rote sola para lucir “flotante”
    let mut world_angle = 0.0_f32;

    // Iluminación ambiental por skybox (puedo apagarla con F si quiero sombras más duras)
    let mut env_on = true;
    let env_samples: u32 = 4;
    let mut frame_id: u64 = 1;

    // ==========================================================
    // Loop principal
    // ==========================================================
    while !rl.window_should_close() {
        // Controles de cámara
        let dt = rl.get_frame_time();
        if rl.is_key_down(KeyboardKey::KEY_LEFT)  { cam.yaw   -= (90.0_f32).to_radians() * dt; }
        if rl.is_key_down(KeyboardKey::KEY_RIGHT) { cam.yaw   += (90.0_f32).to_radians() * dt; }
        if rl.is_key_down(KeyboardKey::KEY_UP)    { cam.pitch += (60.0_f32).to_radians() * dt; }
        if rl.is_key_down(KeyboardKey::KEY_DOWN)  { cam.pitch -= (60.0_f32).to_radians() * dt; }
        cam.pitch = cam.pitch.clamp(-1.4, 1.4);

        // Zoom del mouse (acá dejo un rango más amplio porque la escena es 3D “flotante”)
        let wheel = rl.get_mouse_wheel_move();
        if wheel.abs() > 0.0 {
            cam.dist = (cam.dist - wheel * 0.3).clamp(3.0, 14.0);
        }

        // Toggle de rotación automática
        if rl.is_key_pressed(KeyboardKey::KEY_R) { autorotate = !autorotate; }
        if autorotate { world_angle += (20.0_f32).to_radians() * dt; }

        // Toggle de IBL (por si quiero comparar look con/ sin relleno del cielo)
        if rl.is_key_pressed(KeyboardKey::KEY_F) { env_on = !env_on; }

        // Renderizo a la imagen de CPU
        let samples = if env_on { env_samples } else { 0 };
        render_scene(&mut image, &scene, &cam, world_angle, sky.as_ref(), samples, frame_id);
        frame_id = frame_id.wrapping_add(1);

        // Subo el framebuffer a la GPU y lo dibujo escalado en la ventana
        {
            let colors = image.get_image_data();
            let slice: &[Color] = colors.as_ref().as_ref();
            let mut bytes = Vec::<u8>::with_capacity((W * H * 4) as usize);
            for c in slice.iter() {
                bytes.push(c.r); bytes.push(c.g); bytes.push(c.b); bytes.push(c.a);
            }
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
        d.draw_text("R=toggle rotación | F=IBL on/off | Isla flotante voxel", 8, 8, 16, Color::RAYWHITE);
    }
}
