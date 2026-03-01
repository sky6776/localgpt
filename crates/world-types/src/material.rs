//! PBR material definition.

use serde::{Deserialize, Serialize};

/// PBR material properties.  All fields mirror the Bevy `StandardMaterial`
/// subset that the gen tools expose.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MaterialDef {
    /// Base color (RGBA, linear).
    #[serde(default = "default_color")]
    pub color: [f32; 4],
    /// Metallic factor (0.0 = dielectric, 1.0 = metal).
    #[serde(default)]
    pub metallic: f32,
    /// Roughness factor (0.0 = mirror, 1.0 = matte).
    #[serde(default = "default_roughness")]
    pub roughness: f32,
    /// Emissive color (RGBA, linear). Non-zero = self-illuminating.
    #[serde(default)]
    pub emissive: [f32; 4],
}

impl Default for MaterialDef {
    fn default() -> Self {
        Self {
            color: default_color(),
            metallic: 0.0,
            roughness: default_roughness(),
            emissive: [0.0, 0.0, 0.0, 0.0],
        }
    }
}

fn default_color() -> [f32; 4] {
    [0.8, 0.8, 0.8, 1.0]
}

fn default_roughness() -> f32 {
    0.5
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_material() {
        let m = MaterialDef::default();
        assert_eq!(m.color, [0.8, 0.8, 0.8, 1.0]);
        assert_eq!(m.metallic, 0.0);
        assert_eq!(m.roughness, 0.5);
        assert_eq!(m.emissive, [0.0, 0.0, 0.0, 0.0]);
    }

    #[test]
    fn material_roundtrip() {
        let m = MaterialDef {
            color: [1.0, 0.0, 0.0, 1.0],
            metallic: 0.9,
            roughness: 0.1,
            emissive: [0.5, 0.5, 0.0, 1.0],
        };
        let json = serde_json::to_string(&m).unwrap();
        let back: MaterialDef = serde_json::from_str(&json).unwrap();
        assert_eq!(m, back);
    }
}
