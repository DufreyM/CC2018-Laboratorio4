use raylib::prelude::*;

/// Shaders CPU-style para planetas divertidos y clásicos.
/// Solo se basa en `sphere-1.obj`, sin texturas externas.
/// Uniforms implícitos: time: f32, light_dir: Vector3

// ---------- PLANETA ROCOSO ----------
pub fn roca(pos: &Vector3, normal: &Vector3, time: f32) -> Color {
    let mountain = (pos.y * 5.0 + fbm_noise(pos.x * 3.0, pos.z * 3.0, 5) * 2.0).sin();
    let mut base = Color::new(
        (90.0 + mountain * 60.0) as u8,
        (70.0 + mountain * 40.0) as u8,
        (50.0 + mountain * 30.0) as u8,
        255,
    );

    let light_dir = Vector3::new(0.6, 0.9, 0.3).normalized();
    let brightness = normal.dot(light_dir).max(0.0) * 0.8 + 0.2;
    base = apply_brightness(base, brightness);

    let clouds = fbm_noise(pos.x * 6.0 + time*0.05, pos.z*6.0 + time*0.02, 4);
    blend_colors(base, Color::new(240, 240, 240, 255), clouds.powf(3.0) * 0.3)
}

// ---------- PLANETA GASEOSO ----------
pub fn gas(pos: &Vector3, normal: &Vector3, time: f32) -> Color {
    let r = pos.length();
    let swirl = ((pos.x*3.0 + pos.y*2.0 + time*0.15).sin() * 0.5 + 0.5).powf(1.2);

    let mut base = lerp_color(Color::new(20,30,80,255), Color::new(150,180,220,255), (1.0 - r).clamp(0.0,1.0));
    base = lerp_color(base, Color::new(220,200,100,255), swirl * 0.5);

    let light_dir = Vector3::new(0.3,0.7,0.8).normalized();
    let bright = normal.dot(light_dir).max(0.1);
    apply_brightness(base, bright * 0.9 + 0.1)
}

// ---------- MARCIANO ----------
pub fn marciano(pos: &Vector3, normal: &Vector3, time: f32) -> Color {
    let relieves = fbm_noise(pos.x * 6.0, pos.z * 6.0 + time * 0.05, 4);
    let mut base = lerp_color(
        Color::new(180, 50, 30, 255),
        Color::new(250, 100, 60, 255),
        relieves,
    );

    let glow = ((time * 2.0 + pos.y * 4.0).sin() * 0.5 + 0.5).powf(2.0);
    base = blend_colors(base, Color::new(0, 255, 120, 255), glow * 0.3);

    let light_dir = Vector3::new(0.4, 0.9, 0.2).normalized();
    let intensidad = normal.dot(light_dir).max(0.1);
    apply_brightness(base, intensidad * 0.9 + 0.1)
}

// ---------- PANQUEQUES ----------
pub fn panqueques(pos: &Vector3, normal: &Vector3, _time: f32) -> Color {
    let radio = (pos.x * pos.x + pos.z * pos.z).sqrt();
    let capas = (radio * 8.0).fract();
    let base = if capas < 0.5 {
        Color::new(200, 150, 90, 255)
    } else {
        Color::new(250, 200, 120, 255)
    };

    let light_dir = Vector3::new(0.5, 0.8, 0.3).normalized();
    let intensidad = normal.dot(light_dir).max(0.2);
    apply_brightness(base, intensidad)
}

// ---------- ARCOÍRIS ----------
pub fn arcoiris(pos: &Vector3, normal: &Vector3, time: f32) -> Color {
    let angle = pos.y.atan2(pos.x) + time * 0.5;
    let mut t = (angle / std::f32::consts::PI) % 1.0;
    if t < 0.0 { t += 1.0; }

    let base = if t < 1.0/7.0 { Color::RED }
               else if t < 2.0/7.0 { Color::ORANGE }
               else if t < 3.0/7.0 { Color::YELLOW }
               else if t < 4.0/7.0 { Color::GREEN }
               else if t < 5.0/7.0 { Color::BLUE }
               else if t < 6.0/7.0 { Color::PURPLE }
               else { Color::MAGENTA };

    let light_dir = Vector3::new(0.5, 0.7, 0.3).normalized();
    let intensidad = normal.dot(light_dir).max(0.2);
    apply_brightness(base, intensidad)
}

/* ---------------- UTILIDADES ---------------- */
fn fbm_noise(x: f32, y: f32, oct: u32) -> f32 {
    let mut sum = 0.0;
    let mut amp = 1.0;
    let mut freq = 1.0;
    let mut maxv = 0.0;
    for _ in 0..oct {
        sum += noise2d(x * freq, y * freq) * amp;
        maxv += amp;
        amp *= 0.5;
        freq *= 2.0;
    }
    ((sum / maxv) + 1.0) * 0.5
}

fn noise2d(x: f32, y: f32) -> f32 {
    let xi = x.floor() as i32;
    let yi = y.floor() as i32;
    let xf = x - x.floor();
    let yf = y - y.floor();
    let v00 = hash_to_float(xi, yi);
    let v10 = hash_to_float(xi + 1, yi);
    let v01 = hash_to_float(xi, yi + 1);
    let v11 = hash_to_float(xi + 1, yi + 1);
    lerp_f32(lerp_f32(v00, v10, fade(xf)), lerp_f32(v01, v11, fade(xf)), fade(yf)) * 2.0 - 1.0
}

fn fade(t: f32) -> f32 { t * t * t * (t * (t * 6.0 - 15.0) + 10.0) }
fn lerp_f32(a: f32, b: f32, t: f32) -> f32 { a + (b - a) * t }
fn hash_to_float(x: i32, y: i32) -> f32 {
    ((x.wrapping_mul(374761393) ^ y.wrapping_mul(668265263)).wrapping_add(1274126177) & 0xFFFF) as f32 / 65535.0
}
fn apply_brightness(c: Color, b: f32) -> Color {
    Color::new(
        (c.r as f32 * b).clamp(0.0, 255.0) as u8,
        (c.g as f32 * b).clamp(0.0, 255.0) as u8,
        (c.b as f32 * b).clamp(0.0, 255.0) as u8,
        c.a,
    )
}
fn lerp_color(a: Color, b: Color, t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    Color::new(
        (a.r as f32*(1.0-t)+b.r as f32*t) as u8,
        (a.g as f32*(1.0-t)+b.g as f32*t) as u8,
        (a.b as f32*(1.0-t)+b.b as f32*t) as u8,
        255,
    )
}
fn blend_colors(base: Color, top: Color, alpha: f32) -> Color {
    let a = alpha.clamp(0.0,1.0);
    Color::new(
        (base.r as f32*(1.0-a)+top.r as f32*a) as u8,
        (base.g as f32*(1.0-a)+top.g as f32*a) as u8,
        (base.b as f32*(1.0-a)+top.b as f32*a) as u8,
        255
    )
}
