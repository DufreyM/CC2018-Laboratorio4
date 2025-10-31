#![allow(unused_imports)]
mod framebuffer;
mod line;
mod obj_loader;
mod shader;
mod triangle;
mod geometria;

use raylib::prelude::*;
use framebuffer::Framebuffer;
use obj_loader::ObjModel;
use triangle::ShaderType;
use geometria::{generate_moon, generate_rings, transform_model};
use std::f32::consts::PI;

fn main() {
    let (mut window, thread) = raylib::init()
        .size(800, 600)
        .title("Laboratorio 4 - Cuerpos Celestes Procedurales")
        .build();

    let mut fb = Framebuffer::new(800, 600, Color::new(5, 5, 15, 255));

    println!("Cargando sphere-1.obj ...");
    let model_sphere = ObjModel::load("sphere-1.obj")
        .expect("No se pudo cargar sphere-1.obj");

    let model_crystal = ObjModel::load("crystal_planet.obj")
        .unwrap_or_else(|_| model_sphere.clone());

    let moon_model = generate_moon(0.3, 24);
    let rings_model = generate_rings(1.35, 2.1, 128);

    println!(
        "Modelos listos. Vertices luna: {}, anillos: {}",
        moon_model.vertices.len(),
        rings_model.vertices.len()
    );

    let mut angle_y = 0.0f32;
    let mut scale = 1.5f32;
    let mut current_planet = 0usize;
    let mut auto_rotate = true;
    let mut time = 0.0f32;
    let mut orbital_angle = 0.0f32;

    window.set_target_fps(60);

    let planet_names = vec![
        "Rocos",
        "Gaseoso",
        "Adicional",
        "Puntos Extra",
        "Puntos Extra",
    ];

    let planet_models = vec![
        "sphere-1.obj + Luna Procedural",
        "sphere-1.obj + Anillos Procedurales",
        "crystal_planet.obj",
        "sphere-1.obj",
        "sphere-1.obj",
    ];

    println!("\n=== CONTROLES ===");
    println!("TAB: Cambiar planeta | P: Pausar rotación | W/S: Zoom | A/D: Rotar | R: Reiniciar | C: Captura");

    while !window.window_should_close() {
        fb.clear();
        time += 0.016;
        orbital_angle += 0.02;

        if window.is_key_pressed(KeyboardKey::KEY_TAB) {
            current_planet = (current_planet + 1) % planet_names.len();
            println!("Cambiado a: {}", planet_names[current_planet]);
        }

        if window.is_key_pressed(KeyboardKey::KEY_P) {
            auto_rotate = !auto_rotate;
            println!("Auto-rotación: {}", if auto_rotate { "ON" } else { "OFF" });
        }

        if window.is_key_down(KeyboardKey::KEY_A) {
            angle_y -= 0.02;
        }

        if window.is_key_down(KeyboardKey::KEY_D) {
            angle_y += 0.02;
        }

        if window.is_key_down(KeyboardKey::KEY_W) {
            scale *= 1.02;
        }

        if window.is_key_down(KeyboardKey::KEY_S) {
            scale /= 1.02;
        }

        if window.is_key_pressed(KeyboardKey::KEY_R) {
            angle_y = 0.0;
            scale = 1.5;
            println!("Vista reiniciada");
        }

        if window.is_key_pressed(KeyboardKey::KEY_C) {
            fb.render_to_file("capture.png");
            println!("Captura guardada: capture.png");
        }

        if auto_rotate {
            angle_y += 0.01;
        }

        // --- Render principal ---
        let current_model = if current_planet == 2 { &model_crystal } else { &model_sphere };

        let rotated = transform_model(current_model, Vector3::new(0.0, 0.0, 0.0), angle_y, 0.0, scale);

        let shader_type = match current_planet {
            0 => ShaderType::Rocky,
            1 => ShaderType::Gas,
            2 => ShaderType::Crystal,
            3 => ShaderType::Lava,
            _ => ShaderType::Ice,
        };

        for face in &current_model.faces {
            if face.len() < 3 { continue; }
            for i in 1..(face.len() - 1) {
                let v0 = rotated[face[0]];
                let v1 = rotated[face[i]];
                let v2 = rotated[face[i + 1]];
                triangle::draw_filled_triangle(&mut fb, v0, v1, v2, shader_type, time);
            }
        }

        if current_planet == 0 {
            let moon_distance = 2.5;
            let moon_x = orbital_angle.cos() * moon_distance;
            let moon_z = orbital_angle.sin() * moon_distance;
            let moon_transformed = transform_model(&moon_model, Vector3::new(moon_x * scale, 0.5 * scale, moon_z * scale), angle_y * 0.5, 0.0, scale * 0.6);
            for face in &moon_model.faces {
                if face.len() < 3 { continue; }
                for i in 1..(face.len() - 1) {
                    let v0 = moon_transformed[face[0]];
                    let v1 = moon_transformed[face[i]];
                    let v2 = moon_transformed[face[i + 1]];
                    triangle::draw_filled_triangle(&mut fb, v0, v1, v2, ShaderType::Ice, time);
                }
            }
        }

        if current_planet == 1 {
            let rings_transformed = transform_model(&rings_model, Vector3::new(0.0, 0.0, 0.0), angle_y * 0.3, 0.35, scale);
            for face in &rings_model.faces {
                if face.len() < 3 { continue; }
                for i in 1..(face.len() - 1) {
                    let v0 = rings_transformed[face[0]];
                    let v1 = rings_transformed[face[i]];
                    let v2 = rings_transformed[face[i + 1]];
                    triangle::draw_filled_triangle(&mut fb, v0, v1, v2, ShaderType::Crystal, time);
                }
            }
        }

        // --- Render final ---
        let pixels: Vec<Color> = fb.image_data();
        let mut raw: Vec<u8> = Vec::with_capacity(pixels.len() * 4);
        for c in pixels {
            raw.extend_from_slice(&[c.r, c.g, c.b, c.a]);
        }

        if fb.texture.is_none() {
            fb.init_texture(&mut window, &thread);
        }

        if let Some(tex) = &mut fb.texture {
            tex.update_texture(&raw);

            let mut d = window.begin_drawing(&thread);
            d.clear_background(Color::BLACK);
            d.draw_texture(tex, 0, 0, Color::WHITE);

            d.draw_text(&planet_names[current_planet], 10, 10, 20, Color::WHITE);
            d.draw_text(
                "Controles: TAB planeta | P pausa | W/S zoom | A/D rotar | R reiniciar | C captura",
                10,
                570,
                14,
                Color::LIGHTGRAY,
            );
        }
    }

    println!("Salida.");
}
