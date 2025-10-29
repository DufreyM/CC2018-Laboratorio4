use raylib::prelude::*;
use crate::framebuffer::Framebuffer;
use crate::shader::{roca, gas, cristal, lava, hielo};

#[derive(Copy, Clone)]
pub enum ShaderType {
    Rocky,
    Gas,
    Crystal,
    Lava,
    Ice,
}

/// Dibuja un triángulo relleno con shading perspectiva-correcto (mejor aproximación)
pub fn draw_filled_triangle(
    framebuffer: &mut Framebuffer,
    v0: Vector3,
    v1: Vector3,
    v2: Vector3,
    shader_type: ShaderType,
    time: f32,
) {
    let width = framebuffer.width as f32;
    let height = framebuffer.height as f32;
    let scale = 1.0;

    // Proyección simple perspectiva (igual que antes)
    let p0 = project(&v0, width, height, scale);
    let p1 = project(&v1, width, height, scale);
    let p2 = project(&v2, width, height, scale);

    // Normales por cara (si quieres normales por-vertex, hay que calcular otros datos)
    let edge1 = Vector3::new(v1.x - v0.x, v1.y - v0.y, v1.z - v0.z);
    let edge2 = Vector3::new(v2.x - v0.x, v2.y - v0.y, v2.z - v0.z);
    let normal = edge1.cross(edge2).normalized();

    // Backface culling: usar componente Z de la normal en cámara simple
    if normal.z >= 0.0 {
        return;
    }

    // Bounding box en pantalla
    let min_x = p0.x.min(p1.x).min(p2.x).max(0.0) as i32;
    let max_x = p0.x.max(p1.x).max(p2.x).min(width - 1.0) as i32;
    let min_y = p0.y.min(p1.y).min(p2.y).max(0.0) as i32;
    let max_y = p0.y.max(p1.y).max(p2.y).min(height - 1.0) as i32;

    let denom = ((p1.y - p2.y) * (p0.x - p2.x) + (p2.x - p1.x) * (p0.y - p2.y));
    if denom.abs() < 1e-6 { return; }

    // Para interpolación perspectiva-correcta, usamos 1/z weights
    let iz0 = 1.0 / (v0.z + 1e-6);
    let iz1 = 1.0 / (v1.z + 1e-6);
    let iz2 = 1.0 / (v2.z + 1e-6);

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let px = x as f32 + 0.5;
            let py = y as f32 + 0.5;

            let w0 = ((p1.y - p2.y) * (px - p2.x) + (p2.x - p1.x) * (py - p2.y)) / denom;
            let w1 = ((p2.y - p0.y) * (px - p2.x) + (p0.x - p2.x) * (py - p2.y)) / denom;
            let w2 = 1.0 - w0 - w1;

            const EPSILON: f32 = -0.0001;
            if w0 >= EPSILON && w1 >= EPSILON && w2 >= EPSILON {
                // Depth interpolación (perspectiva-correcta)
                let iz = w0 * iz0 + w1 * iz1 + w2 * iz2;
                let depth = 1.0 / iz;

                let idx = (y as u32 * framebuffer.width + x as u32) as usize;
                if depth < framebuffer.z_buffer[idx] {
                    framebuffer.z_buffer[idx] = depth;

                    // Interpolar posición 3D perspectiva-correcta
                    let px_x = (w0 * v0.x * iz0 + w1 * v1.x * iz1 + w2 * v2.x * iz2) / iz;
                    let px_y = (w0 * v0.y * iz0 + w1 * v1.y * iz1 + w2 * v2.y * iz2) / iz;
                    let px_z = (w0 * v0.z * iz0 + w1 * v1.z * iz1 + w2 * v2.z * iz2) / iz;
                    let pos = Vector3::new(px_x, px_y, px_z);

                    // Aplicar shader según tipo (usamos normal de cara)
                    let color = match shader_type {
                        ShaderType::Rocky => roca(&pos, &normal, time),
                        ShaderType::Gas => gas(&pos, &normal, time),
                        ShaderType::Crystal => cristal(&pos, &normal, time),
                        ShaderType::Lava => lava(&pos, &normal, time),
                        ShaderType::Ice => hielo(&pos, &normal, time),
                    };
                    framebuffer.set_pixel_with_color(x, y, color);
                }
            }
        }
    }
}

fn project(v: &Vector3, width: f32, height: f32, scale: f32) -> Vector2 {
    // Proyección simple: fov dependiente de z para dar sensación de profundidad.
    let fov = 1.0 / (v.z + 3.0);
    let x = width / 2.0 + v.x * scale * fov * width / 2.0;
    let y = height / 2.0 - v.y * scale * fov * height / 2.0;
    Vector2::new(x, y)
}
