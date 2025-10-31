use raylib::prelude::*;

/// Shaders "CPU-style" para planetas con mayor complejidad:
/// - Soporta hasta 4 capas de color por planeta (gradientes, bandas, nubes, brillo emissive)
/// - Iluminación simulada: Lambert + especular Blinn-Phong + rim lighting + AO aproximado
/// - Normal perturbation vía ruido para darle detalle a la iluminación
/// - Uniforms implícitos: time: f32, light_dir: Vector3

// ---------- CONFIG BÁSICA ----------
const MAX_LAYERS: usize = 4;

// ---------- PLANETA ROCOSO (roca) ----------
pub fn roca(pos: &Vector3, normal: &Vector3, time: f32) -> Color {
    // Capa 0: gradiente base según latitud
    let latitude = (pos.y).clamp(-1.0, 1.0) * 0.5 + 0.5; // 0..1
    let base_low = Color::new(70, 50, 40, 255);
    let base_high = Color::new(150, 130, 110, 255);
    let mut col = lerp_color(base_low, base_high, latitude);

    // Capa 1: detalle rocoso con FBM (pequeñas montañas y grietas)
    let mountain = fbm_noise(pos.x * 6.0, pos.z * 6.0 + time * 0.01, 5);
    let rock_layer = lerp_color(Color::new(90, 70, 60, 255), Color::new(120, 100, 90, 255), mountain);
    col = blend_colors(col, rock_layer, smoothstep(0.2, 0.8, mountain) * 0.8);

    // Capa 2: sedimentos / polvo (sutil, encima de montañas)
    let sediment = fbm_noise(pos.x * 12.0, pos.z * 12.0, 3).powf(2.0);
    let sediment_col = Color::new(200, 170, 140, 255);
    col = blend_colors(col, sediment_col, sediment * 0.25);

    // Capa 3: nubes o neblina de altura (transparente)
    let clouds = fbm_noise(pos.x * 3.0 + time * 0.03, pos.z * 3.0 + time * 0.02, 4);
    col = blend_colors(col, Color::new(235, 235, 235, 255), clouds.powf(3.0) * 0.35);

    // Iluminación (perturbar normal con ruido para detalle)
    let pert = perturb_normal(normal, pos, 0.6);
    let shaded = shading(col, &pert, Vector3::new(0.6, 0.9, 0.3).normalized(), 32.0, 0.25);

    // AO approximado y vignetting por poles
    let ao = 1.0 - fbm_noise(pos.x * 8.0, pos.z * 8.0, 3) * 0.25;
    apply_brightness(shaded, ao)
}

// ---------- PLANETA GASEOSO (gas) ----------
pub fn gas(pos: &Vector3, normal: &Vector3, time: f32) -> Color {
    // Capa 0: gradiente radial (más pálido hacia el borde)
    let r = pos.length().clamp(0.0, 1.0);
    let gradient = (1.0 - r).clamp(0.0, 1.0);
    let base_a = Color::new(20, 30, 80, 255);
    let base_b = Color::new(150, 180, 220, 255);
    let mut col = lerp_color(base_a, base_b, gradient);

    // Capa 1: bandas en la atmósfera (smoothed noise + sin)
    let band_noise = fbm_noise(pos.x * 4.0, pos.y * 3.0 + time * 0.1, 5);
    let bands = (pos.y * 6.0 + band_noise * 2.0).sin().abs();
    col = blend_colors(col, Color::new(200, 220, 240, 255), bands * 0.35);

    // Capa 2: remolinos (swirls)
    let swirl = ((pos.x * 3.0 + pos.y * 2.0 + time * 0.2).sin() * 0.5 + 0.5).powf(1.4);
    col = blend_colors(col, Color::new(220, 200, 100, 255), swirl * 0.25);

    // Capa 3: halo translúcido (simula scattering cerca del terminador)
    let rim = fresnel(normal, Vector3::new(0.0, 0.0, 1.0), 1.0).powf(2.0);
    col = blend_colors(col, Color::new(255, 245, 220, 255), rim * 0.25);

    // Iluminación suave + specular ancho para gas
    let pert = perturb_normal(normal, pos, 0.3);
    shading(col, &pert, Vector3::new(0.3, 0.7, 0.8).normalized(), 8.0, 0.12)
}

// ---------- MARCIANO (emissive veins + colores) ----------
pub fn marciano(pos: &Vector3, normal: &Vector3, time: f32) -> Color {
    // Capa 0: base rojiza con variación por ruido
    let base_noise = fbm_noise(pos.x * 6.0, pos.z * 6.0 + time * 0.02, 4);
    let mut col = lerp_color(Color::new(150, 40, 30, 255), Color::new(240, 90, 60, 255), base_noise);

    // Capa 1: venas emisivas (verde neón) siguiendo ruido agudo
    let veins = ridge(fbm_noise(pos.x * 20.0, pos.z * 20.0 + time * 0.1, 3));
    col = blend_colors(col, Color::new(0, 255, 120, 255), veins * 0.6);

    // Capa 2: manchas oscuras de cráteres
    let craters = fbm_noise(pos.x * 3.0 + time * 0.05, pos.z * 3.0, 4);
    col = blend_colors(col, Color::new(60, 30, 25, 255), smoothstep(0.4, 0.8, craters) * 0.45);

    // Capa 3: brillo pulsante local (emissive subtle)
    let pulse = ((time * 3.0 + pos.y * 6.0).sin() * 0.5 + 0.5).powf(3.0);
    let emissive = blend_colors(col, Color::new(255, 200, 120, 255), pulse * 0.12);

    // Iluminación y specular moderado
    let pert = perturb_normal(normal, pos, 0.4);
    shading(emissive, &pert, Vector3::new(0.4, 0.9, 0.2).normalized(), 20.0, 0.18)
}

// ---------- PANQUEQUES (banded planet con textura) ----------
pub fn panqueques(pos: &Vector3, normal: &Vector3, time: f32) -> Color {
    // Capa 0: bandas concentricas suaves según radio
    let radio = (pos.x * pos.x + pos.z * pos.z).sqrt();
    let bands = (radio * 6.0 + fbm_noise(pos.x * 4.0, pos.z * 4.0, 3) * 0.6).fract();
    let base1 = Color::new(200, 150, 90, 255);
    let base2 = Color::new(250, 200, 120, 255);
    let mut col = lerp_color(base1, base2, smoothstep(0.0, 1.0, bands));

    // Capa 1: grietas sutiles
    let cracks = fbm_noise(pos.x * 15.0, pos.z * 15.0, 4);
    col = blend_colors(col, Color::new(120, 80, 50, 255), cracks * 0.15);

    // Capa 2: borde más brillante (specular pancake)
    col = blend_colors(col, Color::new(255, 240, 200, 255), powf(normal.dot(Vector3::new(0.5, 0.9, 0.3).normalized()).max(0.0), 50.0) * 0.25);

    // Capa 3: sombra radial ligera hacia el ecuador
    let eq_shadow = (1.0 - (pos.y.abs()).clamp(0.0, 1.0)).powf(2.0);
    col = blend_colors(col, Color::new(80, 60, 40, 255), eq_shadow * 0.12);

    let pert = perturb_normal(normal, pos, 0.25);
    shading(col, &pert, Vector3::new(0.5, 0.8, 0.3).normalized(), 40.0, 0.2)
}

// ---------- ARCOÍRIS (suave, muchas capas de color) ----------
pub fn arcoiris(pos: &Vector3, normal: &Vector3, time: f32) -> Color {
    // Capa 0: rotación circular de colores (gradiente continuo)
    let angle = pos.y.atan2(pos.x) + time * 0.8;
    let mut t = (angle / std::f32::consts::PI) % 2.0;
    if t < 0.0 { t += 2.0; }
    t *= 0.5; // 0..1

    // base suave usando un polinomial para transiciones más suaves
    let col0 = rainbow_gradient(t);

    // Capa 1: brillo angular (specular colored)
    let rim = fresnel(normal, Vector3::new(0.0, 0.0, 1.0), 1.0);
    let spec_col = apply_brightness(col0, rim * 1.2 + 0.8);

    // Capa 2: bandas finas para acentuar el arco
    let bands = (pos.y * 12.0 + fbm_noise(pos.x * 8.0, pos.z * 8.0, 3) * 3.0).sin().abs();
    let layered = blend_colors(spec_col, Color::new(255, 255, 255, 255), bands * 0.18);

    // Capa 3: sutil brillo global
    let global_glow = (1.0 - pos.length()).clamp(0.0, 1.0).powf(1.5);
    let final_col = blend_colors(layered, Color::new(255, 240, 255, 255), global_glow * 0.08);

    let pert = perturb_normal(normal, pos, 0.2);
    shading(final_col, &pert, Vector3::new(0.5, 0.7, 0.3).normalized(), 64.0, 0.06)
}

/* ---------------- UTILIDADES AVANZADAS ---------------- */
fn perturb_normal(n: &Vector3, pos: &Vector3, scale: f32) -> Vector3 {
    // Perturba la normal usando derivadas aproximadas de FBM para dar relieve
    let eps = 0.001;
    let nx = fbm_noise((pos.x + eps) * 6.0, pos.z * 6.0, 3) - fbm_noise((pos.x - eps) * 6.0, pos.z * 6.0, 3);
    let nz = fbm_noise(pos.x * 6.0, (pos.z + eps) * 6.0, 3) - fbm_noise(pos.x * 6.0, (pos.z - eps) * 6.0, 3);
    let tangent = Vector3::new(nx, 0.0, nz) * scale;
    (*n + tangent).normalized()
}

fn shading(base: Color, normal: &Vector3, light_dir: Vector3, shininess: f32, specular_strength: f32) -> Color {
    let ndotl = normal.dot(light_dir).max(0.0);
    let ambient = 0.08;
    let mut lit = apply_brightness(base, ambient + ndotl * (1.0 - ambient));

    // Specular Blinn-Phong (suave)
    let view = Vector3::new(0.0, 0.0, 1.0); // cámara fija
    let half = (light_dir + view).normalized();
    let spec = normal.dot(half).max(0.0).powf(shininess) * specular_strength;
    lit = blend_colors(lit, Color::new(255, 255, 255, 255), spec as f32);

    // Rim lighting para accentuar bordes
    let rim = 1.0 - view.dot(*normal).clamp(0.0, 1.0);
    let rim_strength = rim.powf(2.0) * 0.12;
    blend_colors(lit, Color::new(255, 240, 210, 255), rim_strength)
}

fn ridge(x: f32) -> f32 { (1.0 - (2.0 * (x - 0.5)).abs()).max(0.0) }
fn smoothstep(a: f32, b: f32, x: f32) -> f32 { let t = ((x - a) / (b - a)).clamp(0.0, 1.0); t * t * (3.0 - 2.0 * t) }
fn powf(x: f32, p: f32) -> f32 { x.powf(p) }

fn fresnel(normal: &Vector3, view_dir: Vector3, power: f32) -> f32 {
    let vdotn = view_dir.dot(*normal).clamp(0.0, 1.0);
    (1.0 - vdotn).powf(power)
}

/* ---------------- RUIDO (FBM y Perlin-like) ---------------- */
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

/* ---------------- COLOR UTILITIES ---------------- */
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
        (a.r as f32 * (1.0 - t) + b.r as f32 * t) as u8,
        (a.g as f32 * (1.0 - t) + b.g as f32 * t) as u8,
        (a.b as f32 * (1.0 - t) + b.b as f32 * t) as u8,
        255,
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

fn rainbow_gradient(t: f32) -> Color {
    // t in 0..1 -> espectro suave
    let tt = (t * 6.0).clamp(0.0, 6.0);
    let idx = tt.floor() as i32;
    let frac = tt - idx as f32;
    match idx {
        0 => lerp_color(Color::new(255, 0, 0, 255), Color::new(255, 127, 0, 255), frac),
        1 => lerp_color(Color::new(255, 127, 0, 255), Color::new(255, 255, 0, 255), frac),
        2 => lerp_color(Color::new(255, 255, 0, 255), Color::new(0, 255, 0, 255), frac),
        3 => lerp_color(Color::new(0, 255, 0, 255), Color::new(0, 0, 255, 255), frac),
        4 => lerp_color(Color::new(0, 0, 255, 255), Color::new(75, 0, 130, 255), frac),
        5 => lerp_color(Color::new(75, 0, 130, 255), Color::new(148, 0, 211, 255), frac),
        _ => Color::MAGENTA,
    }
}
