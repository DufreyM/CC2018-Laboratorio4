use raylib::prelude::*;

/// Framebuffer simple con z-buffer y textura GPU opcional.
/// Ahora `texture` es pública para que `main`/UI pueda actualizarla.
pub struct Framebuffer {
    pub width: u32,
    pub height: u32,
    pub color_buffer: Image,
    pub z_buffer: Vec<f32>,
    pub background_color: Color,
    pub current_color: Color,
    pub texture: Option<Texture2D>, // pública para acceso desde main
}

impl Framebuffer {
    /// Crea un framebuffer nuevo y prepara Z-buffer.
    pub fn new(width: u32, height: u32, background_color: Color) -> Self {
        let color_buffer = Image::gen_image_color(width as i32, height as i32, background_color);
        let z_buffer = vec![f32::INFINITY; (width * height) as usize];
        Self {
            width,
            height,
            color_buffer,
            z_buffer,
            background_color,
            current_color: Color::WHITE,
            texture: None,
        }
    }

    /// Limpia color y Z-buffer
    pub fn clear(&mut self) {
        self.color_buffer.clear_background(self.background_color);
        self.z_buffer.fill(f32::INFINITY);
    }

    pub fn set_current_color(&mut self, color: Color) {
        self.current_color = color;
    }

    pub fn set_background_color(&mut self, color: Color) {
        self.background_color = color;
    }

    /// Dibuja un píxel sin profundidad (útil para wireframe)
    pub fn set_pixel(&mut self, x: i32, y: i32) {
        if x >= 0 && y >= 0 && x < self.width as i32 && y < self.height as i32 {
            self.color_buffer.draw_pixel(x, y, self.current_color);
        }
    }

    /// Dibuja un píxel con color explícito
    pub fn set_pixel_with_color(&mut self, x: i32, y: i32, color: Color) {
        if x >= 0 && y >= 0 && x < self.width as i32 && y < self.height as i32 {
            self.color_buffer.draw_pixel(x, y, color);
        }
    }

    /// Dibuja un píxel controlando profundidad.
    pub fn set_pixel_depth(&mut self, x: i32, y: i32, depth: f32) {
        if x >= 0 && y >= 0 && x < self.width as i32 && y < self.height as i32 {
            let idx = (y as u32 * self.width + x as u32) as usize;
            if depth < self.z_buffer[idx] {
                self.z_buffer[idx] = depth;
                self.color_buffer.draw_pixel(x, y, self.current_color);
            }
        }
    }

    /// Inicializa la textura GPU desde la imagen (una sola vez).
    /// Si ya existe, no la vuelve a crear.
    pub fn init_texture(&mut self, window: &mut RaylibHandle, thread: &RaylibThread) {
        if self.texture.is_none() {
            if let Ok(tex) = window.load_texture_from_image(thread, &self.color_buffer) {
                self.texture = Some(tex);
            }
        }
    }

    /// Actualiza la textura existente con los datos actuales del color buffer.
    /// Usa el método de Texture2D update (separado en main).
    pub fn image_data(&self) -> Vec<Color> {
        self.color_buffer.get_image_data().to_vec()
    }

    /// Exporta a archivo (para capturas)
    pub fn render_to_file(&self, path: &str) {
        self.color_buffer.export_image(path);
    }
}
