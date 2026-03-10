// Allow dead_code warnings from encase's ShaderType derive macro
#![allow(dead_code)]

use encase::ShaderType;

/// Global uniforms shared by all primitives.
///
/// Contains camera, viewport, and timing data needed by the grid shader.
#[derive(Clone, Debug, ShaderType)]
pub struct Uniforms {
    pub os_scale_factor: f32, // e.g. 1.0, 1.5
    pub camera_zoom: f32,
    pub camera_position: glam::Vec2,

    pub num_nodes: u32,
    pub time: f32, // Time in seconds for animations

    pub bounds_origin: glam::Vec2, // widget bounds origin in physical pixels
    pub bounds_size: glam::Vec2,   // widget bounds size in physical pixels

    pub _pad0: glam::Vec2, // padding for 16-byte uniform alignment
}

/// Grid background configuration.
///
/// Read from storage buffer by the grid shader pass.
#[derive(Clone, Debug, ShaderType)]
pub struct Grid {
    /// Pattern type: 0=None, 1=Grid, 2=Hex, 3=Triangle, 4=Dots, 5=Lines, 6=Crosshatch
    pub pattern_type: u32,
    /// Flags: bit 0 = adaptive_zoom, bit 1 = hex_pointy_top
    pub flags: u32,
    /// Minor spacing in world-space pixels
    pub minor_spacing: f32,
    /// Major spacing ratio (major_spacing / minor_spacing), 0 = no major grid
    pub major_ratio: f32,

    /// Line widths: (minor_width, major_width)
    pub line_widths: glam::Vec2,
    /// Opacities: (minor_opacity, major_opacity)
    pub opacities: glam::Vec2,

    /// Primary pattern color (background fill)
    pub primary_color: glam::Vec4,
    /// Secondary pattern color (grid lines)
    pub secondary_color: glam::Vec4,

    /// Pattern-specific params: (dot_radius, line_angle, crosshatch_angle, _padding)
    pub pattern_params: glam::Vec4,

    /// Adaptive zoom thresholds: (min_spacing, max_spacing, fade_range, _padding)
    pub adaptive_params: glam::Vec4,
}

#[cfg(test)]
mod tests {
    use super::*;
    use encase::ShaderSize;

    #[test]
    fn test_uniforms_shader_size() {
        let size = Uniforms::SHADER_SIZE.get();
        assert!(size > 0, "Uniforms size should be positive");
        assert!(size % 16 == 0, "Uniforms size should be 16-byte aligned");
    }

    #[test]
    fn test_grid_shader_size() {
        let size = Grid::SHADER_SIZE.get();
        assert!(size > 0, "Grid size should be positive");
        assert!(size % 16 == 0, "Grid size should be 16-byte aligned");
    }
}
