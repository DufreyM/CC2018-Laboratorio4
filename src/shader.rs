use raylib::prelude::*;

/// Shaders "CPU-style" para planetas con mayor complejidad:
/// - Soporta hasta 4 capas de color por planeta (gradientes, bandas, nubes, brillo emissive)
/// - Iluminación simulada: Lambert + especular Blinn-Phong + rim lighting + AO aproximado
/// - Normal perturbation vía ruido para darle detalle a la iluminación
/// - Uniforms implícitos: time: f32, light_dir: Vector3

// ---------- CONFIG BÁSICA ----------
const MAX_LAYERS: usize = 4;

// ---------- UTILIDADES DE CAPAS ----------
fn blend_layered(mut base: Color, layers: &[(Color, f32)]) -> Color {
    // layers: (color, weight) - se mezclan sobre base según weights normalizados
    let mut total_weight = 0.0;
    for &(_, w) in layers.iter() { total_weight += w.max(0.0); }
    if total_weight <= 0.0 { return base; }
    for &(col, w) in layers.iter() {
        let a = (w / total_weight).clamp(0.0, 1.0);
        base = blend_colors(base, col, a);
    }
    base
}

fn apply_emissive(base: Color, emissive: Color, strength: f32) -> Color {
    // simplistic additive emissive (clamped)
    let r = (base.r as f32 + emissive.r as f32 * strength).clamp(0.0, 255.0) as u8;
    let g = (base.g as f32 + emissive.g as f32 * strength).clamp(0.0, 255.0) as u8;
    let b = (base.b as f32 + emissive.b as f32 * strength).clamp(0.0, 255.0) as u8;
    Color::new(r, g, b, 255)
}

fn ring_mask(pos: &Vector3, inner: f32, outer: f32, tilt: f32) -> f32 {
    // pos: coordenadas en esfera; plane tilt en radianes. Regresa alpha 0..1 para anillo
    // proyectamos en plano ecuatorial rotado por tilt (simple)
    let x = pos.x;
    let z = pos.z * tilt.cos() - pos.y * tilt.sin(); // pequeña rotación para simular inclinación
    let r = (x * x + z * z).sqrt();
    smoothstep(inner, outer, r) * (1.0 - smoothstep(inner + 0.01, outer - 0.01, r)) // máscara suave
}

// ---------- EFECTO ATMOSFÉRICO GENERAL ----------
fn apply_atmosphere(color: Color, pos: &Vector3, normal: &Vector3, time: f32) -> Color {
    let view_dir = Vector3::new(0.0, 0.0, 1.0);
    let rim = fresnel(normal, view_dir, 2.5);
    let altitude = (1.0 - pos.length()).clamp(0.0, 1.0);
    let haze = (rim * 0.6 + altitude * 0.4).powf(1.5);
    let haze_color = Color::new(180, 210, 255, 255);
    blend_colors(color, haze_color, haze * 0.15 + (time * 0.1).sin().abs() * 0.05)
}

// ---------- PLANETA ROCOSO DETALLADO (AHORA 4 CAPAS + LAVA) ----------
pub fn roca(pos: &Vector3, normal: &Vector3, time: f32) -> Color {
    // Capa base: latitud + gradiente
    let latitude = (pos.y).clamp(-1.0, 1.0) * 0.5 + 0.5;
    let base_col = lerp_color(Color::new(40, 30, 25, 255), Color::new(210, 170, 120, 255), latitude);

    // Generamos 4 capas con pesos dinámicos:
    let relief = fbm_noise(pos.x * 8.0, pos.z * 8.0 + time * 0.02, 5);
    let veins = fbm_noise(pos.x * 24.0, pos.z * 24.0, 4).powf(1.2);
    let rust = fbm_noise(pos.x * 10.0, pos.z * 10.0, 3).powf(2.8);
    let moss = smoothstep(0.3, 0.8, relief) * (1.0 - latitude);

    let layer0 = (lerp_color(Color::new(90, 60, 50, 255), Color::new(240, 210, 180, 255), relief.powf(1.6)), 0.5); // rocas claras/obscuras
    let layer1 = (blend_colors(Color::new(255, 230, 200, 255), Color::new(190, 80, 40, 255), veins), 0.25); // vetas / óxidos
    let layer2 = (Color::new(60, 100, 70, 255), moss * 0.8); // musgo húmedo
    // capa 3: salpicaduras de material fundido (lava superficial)
    let lava_noise = fbm_noise(pos.x * 6.0, pos.z * 6.0 + time * 0.12, 4);
    let lava_mask = ridge(lava_noise).powf(2.0) * (1.0 - latitude).max(0.0);
    let lava_color = Color::new(255, 120, 40, 255);
    let layer3 = (lava_color, lava_mask * 0.8);

    let mut col = blend_layered(base_col, &[layer0, layer1, layer2, layer3]);

    // aplicar pequeñas grietas y brillo ecuatorial
    let cracks = fbm_noise(pos.x * 30.0, pos.z * 30.0, 4).powf(1.8);
    col = blend_colors(col, Color::new(30, 20, 18, 255), cracks * 0.25);

    // Emissive por lava: usar lava_mask para sumarlo
    let emissive_strength = (lava_mask * 2.0).clamp(0.0, 1.5);
    col = apply_emissive(col, Color::new(255, 80, 30, 255), emissive_strength);

    // Normal perturb y shading
    let pert = perturb_normal(normal, pos, 1.0);
    let light_dir = Vector3::new(0.6, 0.8, 0.5).normalized();
    let shaded = shading(col, &pert, light_dir, 64.0, 0.5);

    apply_atmosphere(shaded, pos, normal, time)
}

// ---------- PLANETA GASEOSO DETALLADO (BANDAS + ANILLO) ----------
pub fn gas(pos: &Vector3, normal: &Vector3, time: f32) -> Color {
    let r = pos.length().clamp(0.0, 1.0);
    let gradient = (1.0 - r).powf(0.5);

    // Base suave (gradiente radial + tendencia giratoria)
    let base_col = lerp_color(Color::new(10, 20, 60, 255), Color::new(220, 200, 160, 255), gradient);

    // Bandas primarias (hasta 3 capas de bandas)
    let band_noise = fbm_noise(pos.y * 3.0 + time * 0.08, pos.x * 3.0, 6);
    let bands_a = ((pos.y * 10.0 + band_noise * 4.0).sin() * 0.5 + 0.5).powf(1.6);
    let band_col_a = lerp_color(Color::new(255, 180, 90, 255), Color::new(180, 230, 255, 255), band_noise);

    let band_noise2 = fbm_noise(pos.y * 6.0 - time * 0.12, pos.z * 2.0, 5);
    let bands_b = ((pos.y * 6.0 + band_noise2 * 2.0).cos() * 0.5 + 0.5).powf(1.3);
    let band_col_b = lerp_color(Color::new(120, 80, 200, 255), Color::new(240, 220, 200, 255), band_noise2);

    // Nubes / remolinos locales
    let swirl = fbm_noise(pos.x * 12.0 + time * 0.4, pos.z * 12.0, 5).powf(1.3);
    let swirl_col = Color::new(255, 245, 210, 255);

    // Capa de neblina
    let haze_layer = (1.0 - r).powf(2.0) * 0.25;

    // Armamos capas (hasta 4)
    let layers = [
        (band_col_a, bands_a * 0.9),
        (band_col_b, bands_b * 0.5),
        (swirl_col, swirl * 0.4),
        (Color::new(180, 210, 255, 255), haze_layer), // alta atmósfera ligera
    ];

    let mut col = blend_layered(base_col, &layers);

    // Anillo: calculamos máscara y pintamos un anillo con gradiente y polvo
    let ring_alpha = ring_mask(pos, 1.05, 1.35, (time * 0.03 + 0.3).sin().abs() * 0.3 + 0.7);
    if ring_alpha > 0.0001 {
        // color del anillo: polvo + bandas
        let ring_noise = fbm_noise(pos.x * 80.0 + time * 0.6, pos.z * 60.0, 4);
        let ring_base = lerp_color(Color::new(220, 200, 170, 255), Color::new(120, 100, 80, 255), ring_noise);
        col = blend_colors(col, ring_base, ring_alpha * 0.85);
    }

    // Añadir brillo equatorial sutil
    col = blend_colors(col, Color::new(255, 255, 240, 255), ((1.0 - pos.y.abs()).powf(6.0)) * 0.06);

    // Perturbación menor (gaseoso suave)
    let pert = perturb_normal(normal, pos, 0.18);
    let shaded = shading(col, &pert, Vector3::new(0.4, 0.8, 0.9).normalized(), 20.0, 0.18);

    apply_atmosphere(shaded, pos, normal, time)
}

// ---------- MARCIANO MEJORADO (NOVEDAD: cristales/biolumin + campos magnéticos) ----------
pub fn marciano(pos: &Vector3, normal: &Vector3, time: f32) -> Color {
    let base_noise = fbm_noise(pos.x * 6.0, pos.z * 6.0 + time * 0.02, 4);
    let mut col = lerp_color(Color::new(140, 30, 25, 255), Color::new(250, 100, 70, 255), base_noise);

    // Vetas emisivas y pulsantes (bioluminiscencia sub-superficial)
    let veins = ridge(fbm_noise(pos.x * 22.0, pos.z * 22.0 + time * 0.15, 3));
    let pulsation = ((time * 2.2 + pos.y * 4.0).sin() * 0.5 + 0.5).powf(2.0);
    let emissive_col = Color::new(0, 255, 160, 255);
    col = blend_colors(col, emissive_col, veins * (0.45 + pulsation * 0.55));

    // Magma superficial
    let magma = fbm_noise(pos.x * 4.0, pos.z * 4.0, 3);
    col = blend_colors(col, Color::new(255, 80, 50, 255), magma.powf(3.0) * 0.2);

    // NUEVO: cristales reflectivos (puntos brillantes con normal perturb fuerte)
    let crystal_noise = fbm_noise(pos.x * 40.0 + time * 0.9, pos.z * 40.0, 3);
    let crystals = smoothstep(0.85, 0.98, crystal_noise);
    let crystal_col = Color::new(200, 230, 255, 255);
    col = blend_colors(col, crystal_col, crystals * 0.9);

    // NUEVO: sutil campo magnético visual como halo cercano al ecuador (efecto glow)
    let mag_field = (pos.y * 6.0 + (time * 0.5).sin() * 0.5).sin().abs();
    col = blend_colors(col, Color::new(90, 200, 160, 255), mag_field * 0.035);

    // Polvo marciano
    let dust = (1.0 - pos.y.abs()).powf(3.0);
    col = blend_colors(col, Color::new(80, 40, 30, 255), dust * 0.12);

    let pert = perturb_normal(normal, pos, 0.55);
    // Para cristales dejamos specular más alto localmente: aumentamos specular si crystals > 0
    let specular_strength = 0.2 + crystals * 0.6;
    let shaded = shading(col, &pert, Vector3::new(0.5, 0.9, 0.2).normalized(), 36.0, specular_strength);

    apply_atmosphere(shaded, pos, normal, time)
}

// ---------- PANQUEQUES MÁS TEXTURADO Y CAPAS (mantequilla, syrup, grano, crema) ----------
pub fn panqueques(pos: &Vector3, normal: &Vector3, time: f32) -> Color {
    let radio = (pos.x * pos.x + pos.z * pos.z).sqrt();
    let base1 = Color::new(200, 150, 90, 255);
    let base2 = Color::new(255, 210, 130, 255);

    // Anillos concéntricos (capas de panqueque)
    let bands = (radio * 7.0 + fbm_noise(pos.x * 4.0, pos.z * 4.0, 4) * 0.6).fract();
    let pancake_base = lerp_color(base1, base2, smoothstep(0.0, 1.0, bands));

    // texturas: grano, quemado, syrup
    let cracks = fbm_noise(pos.x * 18.0, pos.z * 18.0, 5).powf(1.0);
    let grain = fbm_noise(pos.x * 60.0, pos.z * 60.0, 3).powf(1.2);
    let syrup = fbm_noise(pos.x * 6.0 + time * 0.15, pos.z * 6.0, 4);

    // capas:
    let layer_butter = (Color::new(255, 240, 180, 255), (1.0 - pos.y.abs()).powf(3.0) * 0.35);
    let layer_syrup = (Color::new(130, 60, 30, 255), syrup.powf(1.5) * 0.45);
    let layer_grain = (Color::new(100, 70, 50, 255), grain * 0.25);
    let layer_base = (pancake_base, 0.9);

    let mut col = blend_layered(pancake_base, &[layer_base, layer_butter, layer_syrup, layer_grain]);

    // Grietas y sombras locales
    col = blend_colors(col, Color::new(80, 50, 30, 255), cracks * 0.18);

    let pert = perturb_normal(normal, pos, 0.32);
    let shaded = shading(col, &pert, Vector3::new(0.5, 0.8, 0.3).normalized(), 36.0, 0.25);

    apply_atmosphere(shaded, pos, normal, time)
}

// ---------- ARCOÍRIS (se mantiene, ligero ajuste para capas) ----------
pub fn arcoiris(pos: &Vector3, normal: &Vector3, time: f32) -> Color {
    let angle = pos.y.atan2(pos.x) + time * 0.7;
    let mut t = (angle / std::f32::consts::PI) % 2.0;
    if t < 0.0 { t += 2.0; }
    t *= 0.5;

    let col0 = rainbow_gradient(t);

    let rim = fresnel(normal, Vector3::new(0.0, 0.0, 1.0), 1.5);
    let rim_col = rainbow_gradient(((t + time * 0.2).sin() * 0.5 + 0.5) % 1.0);
    let rimmed = blend_colors(col0, rim_col, rim * 0.5);

    let pulse = ((time * 1.2).sin() * 0.5 + 0.5).powf(3.0);
    let layered = blend_colors(rimmed, Color::new(255, 255, 255, 255), pulse * 0.1);

    let pert = perturb_normal(normal, pos, 0.25);
    let shaded = shading(layered, &pert, Vector3::new(0.5, 0.7, 0.3).normalized(), 64.0, 0.08);
    apply_atmosphere(shaded, pos, normal, time)
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
