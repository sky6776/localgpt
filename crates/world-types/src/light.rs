//! Light definitions — directional, point, and spot lights.

use serde::{Deserialize, Serialize};

/// Light source attached to an entity.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LightDef {
    /// Type of light source.
    #[serde(default)]
    pub light_type: LightType,
    /// Light color (RGBA, linear).
    #[serde(default = "default_white")]
    pub color: [f32; 4],
    /// Intensity (lumens for Point/Spot, lux for Directional).
    #[serde(default = "default_intensity")]
    pub intensity: f32,
    /// Direction vector for directional/spot lights.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub direction: Option<[f32; 3]>,
    /// Whether this light casts shadows.
    #[serde(default = "default_true")]
    pub shadows: bool,
}

impl Default for LightDef {
    fn default() -> Self {
        Self {
            light_type: LightType::default(),
            color: default_white(),
            intensity: default_intensity(),
            direction: None,
            shadows: true,
        }
    }
}

/// Light source type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LightType {
    #[default]
    Directional,
    Point,
    Spot,
}

fn default_white() -> [f32; 4] {
    [1.0, 1.0, 1.0, 1.0]
}

fn default_intensity() -> f32 {
    1000.0
}

fn default_true() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn light_type_roundtrip() {
        for lt in [LightType::Directional, LightType::Point, LightType::Spot] {
            let json = serde_json::to_string(&lt).unwrap();
            let back: LightType = serde_json::from_str(&json).unwrap();
            assert_eq!(lt, back);
        }
    }

    #[test]
    fn light_def_defaults() {
        let l = LightDef::default();
        assert_eq!(l.light_type, LightType::Directional);
        assert!(l.shadows);
        assert_eq!(l.intensity, 1000.0);
    }
}
