use std::fs::File;
use std::io::{BufRead, BufReader};
use raylib::prelude::*;

/// Modelo OBJ muy simple: lee vértices y caras (no usa normales ni UVs).
/// Las caras en el OBJ pueden venir como "f v/vt/vn" - sólo tomo el índice de vértice.
#[derive(Debug, Clone)]
pub struct ObjModel {
    pub vertices: Vec<Vector3>,
    pub faces: Vec<Vec<usize>>, // índices 0-based
}

impl ObjModel {
    pub fn load(path: &str) -> Result<Self, std::io::Error> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        let mut vertices = Vec::new();
        let mut faces = Vec::new();

        for line in reader.lines() {
            let line = line?;
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.is_empty() { continue; }

            match parts[0] {
                "v" => {
                    if parts.len() >= 4 {
                        let x: f32 = parts[1].parse().unwrap_or(0.0);
                        let y: f32 = parts[2].parse().unwrap_or(0.0);
                        let z: f32 = parts[3].parse().unwrap_or(0.0);
                        vertices.push(Vector3::new(x, y, z));
                    }
                }
                "f" => {
                    let mut face_indices = Vec::new();
                    for p in &parts[1..] {
                        // formato posible: "v", "v/vt", "v/vt/vn"
                        let idx = p.split('/').next().unwrap_or("0").parse::<usize>().unwrap_or(0);
                        if idx > 0 {
                            face_indices.push(idx - 1); // convertir a 0-based
                        }
                    }
                    if face_indices.len() >= 3 {
                        faces.push(face_indices);
                    }
                }
                _ => {}
            }
        }

        Ok(ObjModel { vertices, faces })
    }
}
