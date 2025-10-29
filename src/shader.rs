
use raylib::prelude::*;

/// Shaders CPU-style reinventados para planetas
/// uniforms implícitos:
/// - time: f32 -> animaciones dinámicas
/// - light_dir: Vector3 -> dirección de luz
/// - view_dir: Vector3 -> dirección de cámara

// ---------- Planeta Rocoso Reimaginado ----------
pub fn roca(pos: &Vector3, normal: &Vector3, time: f32) -> Color {
    let mountain = (pos.y * 5.0 + fbm_noise(pos.x * 3.0, pos.z * 3.0, 5) * 2.0).sin();
    let mut base = Color::new(
        (90.0 + mountain * 60.0) as u8,
        (70.0 + mountain * 40.0) as u8,
        (50.0 + mountain * 30.0) as u8,
        255,
    );

    // efecto de luz
    let light_dir = Vector3::new(0.6, 0.9, 0.3).normalized();
    let brightness = normal.dot(light_dir).max(0.0) * 0.8 + 0.2;
    base = apply_brightness(base, brightness);

    // nubes ligeras y dinámicas
    let clouds = fbm_noise(pos.x * 6.0 + time*0.05, pos.z*6.0 + time*0.02, 4);
    blend_colors(base, Color::new(240, 240, 240, 255), clouds.powf(3.0) * 0.3)
}

// ---------- Gigante gaseoso Reimaginado ----------
pub fn gas(pos: &Vector3, normal: &Vector3, time: f32) -> Color {
    let r = pos.length();
    let swirl = ((pos.x*3.0 + pos.y*2.0 + time*0.15).sin() * 0.5 + 0.5).powf(1.2);

    let mut base = lerp_color(Color::new(20,30,80,255), Color::new(150,180,220,255), (1.0 - r).clamp(0.0,1.0));
    base = lerp_color(base, Color::new(220,200,100,255), swirl * 0.5);

    let light_dir = Vector3::new(0.3,0.7,0.8).normalized();
    let bright = normal.dot(light_dir).max(0.1);
    apply_brightness(base, bright * 0.9 + 0.1)
}

// ---------- Planeta Cristal Reimaginado ----------
pub fn cristal(pos: &Vector3, normal: &Vector3, time: f32) -> Color {
    let crystal_noise = fbm_noise(pos.x*8.0 + time*0.03, pos.y*8.0, 3);
    let base = lerp_color(Color::new(50,0,100,255), Color::new(180,120,255,255), crystal_noise);

    // reflejo más dramático
    let light_dir = Vector3::new(time.cos()*0.5,0.7,time.sin()*0.5).normalized();
    let reflect_dir = reflect(&(-light_dir), normal);
    let spec = reflect_dir.dot(Vector3::new(0.0,0.0,1.0)).max(0.0).powf(50.0);

    blend_colors(base, Color::WHITE, spec * 0.7)
}

// ---------- Lava Reimaginada ----------
pub fn lava(pos: &Vector3, normal: &Vector3, time: f32) -> Color {
    let magma = ((pos.x*12.0 + time*2.5).sin() * (pos.z*12.0 + time*1.5).cos()).abs();
    let cracks = ((pos.y*10.0).sin() * (pos.x*15.0).cos()).abs() > 0.7;
    let mut color = if cracks { Color::new(255,180,50,255) } else { lerp_color(Color::new(80,30,10,255), Color::new(255,60,20,255), magma) };

    let light_dir = Vector3::new(0.5,0.8,0.3).normalized();
    let bright = normal.dot(light_dir).max(0.2) + (0.2*fbm_noise(pos.x*5.0,pos.z*5.0,3));
    apply_brightness(color, bright)
}

// ---------- Hielo Reimaginado ----------
pub fn hielo(pos: &Vector3, normal: &Vector3, time: f32) -> Color {
    let base = lerp_color(Color::new(140,190,250,255), Color::new(210,240,255,255), (pos.y+1.0)/2.0);
    let frost = fbm_noise(pos.x*15.0 + time*0.02, pos.z*15.0, 4).powf(3.0);
    let mut color = blend_colors(base, Color::new(255,255,255,255), frost*0.4);

    let light_dir = Vector3::new(0.5,0.8,0.3).normalized();
    let spec = normal.dot(light_dir).max(0.0).powf(12.0);
    blend_colors(color, Color::WHITE, spec*0.6)
}

/* ---------------- UTILIDADES ---------------- */
fn fbm_noise(x: f32, y: f32, oct: u32) -> f32 {
    let mut sum=0.0; let mut amp=1.0; let mut freq=1.0; let mut maxv=0.0;
    for _ in 0..oct { sum+=noise2d(x*freq,y*freq)*amp; maxv+=amp; amp*=0.5; freq*=2.0; }
    ((sum/maxv)+1.0)*0.5
}

fn noise2d(x: f32, y: f32) -> f32 {
    let xi=x.floor() as i32; let yi=y.floor() as i32;
    let xf=x-x.floor(); let yf=y-y.floor();
    let v00=hash_to_float(xi,yi); let v10=hash_to_float(xi+1,yi);
    let v01=hash_to_float(xi,yi+1); let v11=hash_to_float(xi+1,yi+1);
    lerp_f32(lerp_f32(v00,v10,fade(xf)), lerp_f32(v01,v11,fade(xf)), fade(yf))*2.0-1.0
}

fn fade(t: f32) -> f32 { t*t*t*(t*(t*6.0-15.0)+10.0) }
fn lerp_f32(a:f32,b:f32,t:f32)->f32 { a+(b-a)*t }
fn hash_to_float(x:i32,y:i32)->f32 { ((x.wrapping_mul(374761393)^y.wrapping_mul(668265263)).wrapping_add(1274126177) & 0xFFFF) as f32/65535.0 }

fn reflect(i:&Vector3,n:&Vector3)->Vector3 { let d=i.dot(*n); *i - *n * 2.0*d }
fn lerp_color(a:Color,b:Color,t:f32)->Color { let t=t.clamp(0.0,1.0); Color::new((a.r as f32*(1.0-t)+b.r as f32*t) as u8, (a.g as f32*(1.0-t)+b.g as f32*t) as u8, (a.b as f32*(1.0-t)+b.b as f32*t) as u8,255) }
fn apply_brightness(c:Color,b:f32)->Color { Color::new((c.r as f32*b).clamp(0.0,255.0) as u8,(c.g as f32*b).clamp(0.0,255.0) as u8,(c.b as f32*b).clamp(0.0,255.0) as u8,c.a) }
fn blend_colors(base:Color,top:Color,alpha:f32)->Color { let a=alpha.clamp(0.0,1.0); Color::new((base.r as f32*(1.0-a)+top.r as f32*a) as u8,(base.g as f32*(1.0-a)+top.g as f32*a) as u8,(base.b as f32*(1.0-a)+top.b as f32*a) as u8,255) }
