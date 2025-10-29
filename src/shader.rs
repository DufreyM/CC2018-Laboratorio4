use raylib::prelude::*;

/// SHADERS (implementados como funciones "fragment shader" en CPU).
/// Documentación de uniforms (parámetros globales usados por los shaders):
///
/// - time: f32 -> tiempo global en segundos (animaciones, bandas, nubes).
/// - light_dir: Vector3 -> dirección de luz (normalizada).
/// - view_dir: Vector3 -> dirección de la cámara (para especular).
/// - params (opcional): se pueden añadir floats para controlar intensidad,
///   frecuencia de ruido, mezcla, etc. (en main.rs los puedes exponer).

// ---------- Planeta Rocoso ----------
pub fn rocky_planet_shader(pos: &Vector3, normal: &Vector3, time: f32) -> Color {
    // uniforms implícitos: time, light_dir (hardcodeado)
    let height = pos.y;
    let water_threshold = -0.2 + (pos.x * 5.0).sin() * 0.08;

    let noise = fbm_noise(pos.x * 3.0 + time * 0.01, pos.z * 3.0, 4);

    let light_dir = Vector3::new(0.5, 0.8, 0.3).normalized();
    let brightness = normal.dot(light_dir).max(0.0);

    let base_color = if height < water_threshold {
        let deep_blue = Color::new(10, 50, 120, 255);
        let shallow_blue = Color::new(40, 100, 180, 255);
        lerp_color(deep_blue, shallow_blue, ((height + 1.0) / 2.0).clamp(0.0,1.0))
    } else {
        if noise > 0.55 {
            Color::new(34, 139, 34, 255) // vegetación
        } else {
            Color::new(139, 90, 43, 255) // roca
        }
    };

    // Nubes
    let cloud_noise = fbm_noise(pos.x * 8.0 + time * 0.08, pos.z * 8.0, 3);
    let cloud_alpha = if cloud_noise > 0.68 { 0.35 } else { 0.0 };

    let lit = apply_brightness(base_color, brightness * 0.7 + 0.3);
    blend_colors(lit, Color::WHITE, cloud_alpha)
}

// ---------- Gigante gaseoso ----------
pub fn gas_giant_shader(pos: &Vector3, normal: &Vector3, time: f32) -> Color {
    let r = (pos.x*pos.x + pos.y*pos.y + pos.z*pos.z).sqrt();
    let radial_gradient = 1.0 - r.min(1.0);

    let band_freq = 8.0;
    let band_offset = time * 0.12;
    let bands = ((pos.y + band_offset) * band_freq).sin() * 0.5 + 0.5;

    let turbulence = fbm_noise(pos.x * 4.0 + time*0.02, pos.y * 6.0, 4);

    let light_dir = Vector3::new(0.5, 0.3, 0.8).normalized();
    let brightness = normal.dot(light_dir).max(0.12);

    let dark = Color::new(30, 40, 100, 255);
    let light = Color::new(120, 150, 200, 255);
    let band_color = Color::new(200, 170, 120, 255);

    let mut color = lerp_color(dark, light, radial_gradient);
    color = lerp_color(color, band_color, bands * 0.45);
    let turb = (turbulence - 0.5) * 0.5;
    color = apply_brightness(color, 1.0 + turb);
    apply_brightness(color, brightness * 0.9 + 0.15)
}

// ---------- Planeta cristal ----------
pub fn crystal_planet_shader(pos: &Vector3, normal: &Vector3, time: f32) -> Color {
    let base_purple = Color::new(138, 43, 226, 255);
    let deep_purple = Color::new(75, 0, 130, 255);

    let crystal_pattern = voronoi_pattern(pos.x * 6.0 + time * 0.02, pos.y * 6.0, pos.z * 6.0);

    let view_dir = Vector3::new(0.0, 0.0, 1.0);
    let light_dir = Vector3::new((time * 0.25).cos(), 0.6, (time * 0.25).sin()).normalized();
    let reflect_dir = reflect(&light_dir.scale_by(-1.0), normal);
    let specular = view_dir.dot(reflect_dir).max(0.0).powf(32.0);

    let diffuse = normal.dot(light_dir).max(0.0);
    let mut color = lerp_color(deep_purple, base_purple, crystal_pattern);
    color = apply_brightness(color, diffuse * 0.6 + 0.35);
    blend_colors(color, Color::WHITE, (specular * 0.85).clamp(0.0,1.0))
}

// ---------- Lava ----------
pub fn lava_planet_shader(pos: &Vector3, normal: &Vector3, time: f32) -> Color {
    let flow = fbm_noise(pos.x * 3.8 + time * 0.25, pos.z * 3.8 + time * 0.18, 4);
    let cracks = ((pos.x * 25.0).sin() * (pos.z * 25.0).cos()).abs();
    let is_crack = cracks < 0.09;
    let pulse = ((time * 2.5).sin() * 0.5 + 0.5) * 0.25;

    let dark_rock = Color::new(40, 20, 10, 255);
    let hot_lava = Color::new(255, 90, 0, 255);
    let bright_lava = Color::new(255, 200, 60, 255);

    let mut color = if is_crack {
        bright_lava
    } else if flow > 0.45 {
        lerp_color(hot_lava, bright_lava, ((flow - 0.45) / 0.55).clamp(0.0,1.0))
    } else {
        lerp_color(dark_rock, hot_lava, (flow / 0.45).clamp(0.0,1.0))
    };

    color = apply_brightness(color, 1.0 + pulse);
    let light_dir = Vector3::new(0.5, 0.8, 0.3).normalized();
    let brightness = normal.dot(light_dir).max(0.25);
    apply_brightness(color, brightness)
}

// ---------- Hielo ----------
pub fn ice_planet_shader(pos: &Vector3, normal: &Vector3, time: f32) -> Color {
    let ice_color = Color::new(200, 230, 255, 255);
    let deep_ice = Color::new(150, 180, 220, 255);

    let crack_pattern = fbm_noise(pos.x * 12.0 + time*0.01, pos.z * 12.0, 3);
    let has_crack = crack_pattern > 0.72;

    let view_dir = Vector3::new(0.0, 0.0, 1.0);
    let light_dir = Vector3::new(0.5, 0.8, 0.3).normalized();
    let reflect_dir = reflect(&light_dir.scale_by(-1.0), normal);
    let specular = view_dir.dot(reflect_dir).max(0.0).powf(16.0);

    let depth = (pos.y + 1.0) / 2.0;
    let mut color = lerp_color(deep_ice, ice_color, depth.clamp(0.0,1.0));

    if has_crack {
        color = apply_brightness(color, 0.75);
    }
    color = blend_colors(color, Color::WHITE, (specular * 0.65).clamp(0.0,1.0));
    let brightness = normal.dot(light_dir).max(0.18);
    apply_brightness(color, brightness)
}

/* ----------------- UTILIDADES (ruido, mezcla, reflect, etc.) ----------------- */

fn fbm_noise(x: f32, y: f32, octaves: u32) -> f32 {
    let mut value = 0.0;
    let mut amplitude = 1.0;
    let mut frequency = 1.0;
    let mut max_value = 0.0;
    for _ in 0..octaves {
        value += noise2d(x * frequency, y * frequency) * amplitude;
        max_value += amplitude;
        amplitude *= 0.5;
        frequency *= 2.0;
    }
    // Normalizar a [0,1]
    ((value / max_value) + 1.0) * 0.5
}

fn noise2d(x: f32, y: f32) -> f32 {
    // Hash-based value noise, suave pero rápido
    let xi = x.floor() as i32;
    let yi = y.floor() as i32;
    let xf = x - x.floor();
    let yf = y - y.floor();

    // Corners random
    let v00 = hash_to_float(xi, yi);
    let v10 = hash_to_float(xi + 1, yi);
    let v01 = hash_to_float(xi, yi + 1);
    let v11 = hash_to_float(xi + 1, yi + 1);

    // Smooth interpolation (fade)
    let u = fade(xf);
    let v = fade(yf);

    let a = lerp_f32(v00, v10, u);
    let b = lerp_f32(v01, v11, u);
    lerp_f32(a, b, v) * 2.0 - 1.0 // volver a [-1,1]
}

fn fade(t: f32) -> f32 {
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

fn lerp_f32(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

fn hash_to_float(x: i32, y: i32) -> f32 {
    let n = (x.wrapping_mul(374761393) ^ y.wrapping_mul(668265263)).wrapping_add(1274126177);
    let n = (n ^ (n >> 13)).wrapping_mul(1274126177);
    ((n & 0xFFFF) as f32) / 65535.0 // [0,1]
}

fn voronoi_pattern(x: f32, y: f32, z: f32) -> f32 {
    // Voronoi simplificado: toma la distancia mínima a semillas pseudoaleatorias
    let xi = x.floor() as i32;
    let yi = y.floor() as i32;
    let zi = z.floor() as i32;
    let mut min_dist = f32::INFINITY;
    for dx in -1..=1 {
        for dy in -1..=1 {
            for dz in -1..=1 {
                let nx = (xi + dx) as f32 + hash_to_float(xi + dx, yi + dy) as f32 * 0.9;
                let ny = (yi + dy) as f32 + hash_to_float(yi + dy, zi + dz) as f32 * 0.9;
                let nz = (zi + dz) as f32 + hash_to_float(zi + dz, xi + dx) as f32 * 0.9;
                let dist = ((x - nx).powi(2) + (y - ny).powi(2) + (z - nz).powi(2)).sqrt();
                if dist < min_dist { min_dist = dist; }
            }
        }
    }
    (min_dist.min(1.0)) // valor entre 0 y 1 (aprox)
}

fn reflect(incident: &Vector3, normal: &Vector3) -> Vector3 {
    let dot = incident.dot(*normal);
    Vector3::new(
        incident.x - 2.0 * dot * normal.x,
        incident.y - 2.0 * dot * normal.y,
        incident.z - 2.0 * dot * normal.z,
    )
}

fn lerp_color(a: Color, b: Color, t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    Color::new(
        (a.r as f32 * (1.0 - t) + b.r as f32 * t) as u8,
        (a.g as f32 * (1.0 - t) + b.g as f32 * t) as u8,
        (a.b as f32 * (1.0 - t) + b.b as f32 * t) as u8,
        255,
    )
}

fn apply_brightness(color: Color, brightness: f32) -> Color {
    Color::new(
        ((color.r as f32 * brightness).min(255.0).max(0.0)) as u8,
        ((color.g as f32 * brightness).min(255.0).max(0.0)) as u8,
        ((color.b as f32 * brightness).min(255.0).max(0.0)) as u8,
        color.a,
    )
}

fn blend_colors(base: Color, top: Color, alpha: f32) -> Color {
    let a = alpha.clamp(0.0, 1.0);
    Color::new(
        (base.r as f32 * (1.0 - a) + top.r as f32 * a) as u8,
        (base.g as f32 * (1.0 - a) + top.g as f32 * a) as u8,
        (base.b as f32 * (1.0 - a) + top.b as f32 * a) as u8,
        255,
    )
}
