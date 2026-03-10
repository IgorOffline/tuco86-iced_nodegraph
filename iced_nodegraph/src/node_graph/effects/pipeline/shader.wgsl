// ============================================================================
// GRID BACKGROUND SHADER
// ============================================================================
// Only the grid/background pattern remains as a custom WGPU pipeline.
// Nodes, edges, pins, and overlays have been migrated to iced_sdf.

struct Uniforms {
    os_scale_factor: f32,
    camera_zoom: f32,
    camera_position: vec2<f32>,

    num_nodes: u32,
    time: f32,

    bounds_origin: vec2<f32>,
    bounds_size: vec2<f32>,

    _pad0: vec2<f32>,
};

struct Grid {
    pattern_type: u32,      // 0=None, 1=Grid, 2=Hex, 3=Triangle, 4=Dots, 5=Lines, 6=Crosshatch
    flags: u32,             // bit 0 = adaptive_zoom, bit 1 = hex_pointy_top
    minor_spacing: f32,
    major_ratio: f32,

    line_widths: vec2<f32>,
    opacities: vec2<f32>,

    primary_color: vec4<f32>,
    secondary_color: vec4<f32>,

    pattern_params: vec4<f32>,
    adaptive_params: vec4<f32>,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@group(0) @binding(1)
var<storage, read> grids: array<Grid>;

// ============================================================================
// BACKGROUND PATTERN FUNCTIONS
// ============================================================================

fn compute_adaptive_spacing(base_spacing: f32, zoom: f32, min_screen: f32, max_screen: f32) -> vec2<f32> {
    var screen_spacing = base_spacing * zoom;
    var level_multiplier = 1.0;

    for (var i = 0; i < 6; i = i + 1) {
        if (screen_spacing >= min_screen || level_multiplier >= 64.0) { break; }
        screen_spacing = screen_spacing * 2.0;
        level_multiplier = level_multiplier * 2.0;
    }

    for (var i = 0; i < 6; i = i + 1) {
        if (screen_spacing <= max_screen || level_multiplier <= 0.0625) { break; }
        screen_spacing = screen_spacing * 0.5;
        level_multiplier = level_multiplier * 0.5;
    }

    return vec2(base_spacing * level_multiplier, level_multiplier);
}

fn compute_fade_factor(screen_spacing: f32, min_screen: f32, max_screen: f32, fade_range: f32) -> f32 {
    if (fade_range <= 0.0) { return 1.0; }

    let low_threshold = min_screen * (1.0 + fade_range);
    let high_threshold = max_screen * (1.0 - fade_range);

    if (screen_spacing < low_threshold) {
        return smoothstep(min_screen, low_threshold, screen_spacing);
    } else if (screen_spacing > high_threshold) {
        return smoothstep(max_screen, high_threshold, screen_spacing);
    }
    return 1.0;
}

fn pattern_grid(uv: vec2<f32>, minor: f32, major: f32, zoom: f32) -> vec2<f32> {
    let grid = grids[0];
    let minor_width = grid.line_widths.x;
    let major_width = grid.line_widths.y;

    let coord_minor = abs(uv % minor);
    let dist_minor_x = min(coord_minor.x, minor - coord_minor.x);
    let dist_minor_y = min(coord_minor.y, minor - coord_minor.y);
    let dist_minor = min(dist_minor_x, dist_minor_y);

    var dist_major = 1e10;
    if (major > 0.0) {
        let coord_major = abs(uv % major);
        let dist_major_x = min(coord_major.x, major - coord_major.x);
        let dist_major_y = min(coord_major.y, major - coord_major.y);
        dist_major = min(dist_major_x, dist_major_y);
    }

    let aa = 1.0 / zoom;
    let minor_intensity = 1.0 - smoothstep(0.0, minor_width + aa, dist_minor);
    let major_intensity = 1.0 - smoothstep(0.0, major_width + aa * 1.5, dist_major);

    return vec2(minor_intensity, major_intensity);
}

fn pattern_hex(uv: vec2<f32>, size: f32, pointy_top: bool) -> f32 {
    var p = uv;
    if (pointy_top) { p = vec2(uv.y, uv.x); }

    let hex_width = size * 1.732050808;
    let hex_height = size * 2.0;

    let row = floor(p.y / (hex_height * 0.75));
    var col = floor(p.x / hex_width);

    if (i32(row) % 2 == 1) {
        col = floor((p.x - hex_width * 0.5) / hex_width);
    }

    var hex_center = vec2(
        (col + 0.5) * hex_width,
        (row * 0.75 + 0.5) * hex_height
    );
    if (i32(row) % 2 == 1) {
        hex_center.x = hex_center.x + hex_width * 0.5;
    }

    let rel = abs(p - hex_center);
    let dist = max(rel.x, rel.y * 0.866 + rel.x * 0.5) - size * 0.866;

    let line_width = grids[0].line_widths.x;
    let aa = 1.0 / uniforms.camera_zoom;

    return 1.0 - smoothstep(0.0, line_width + aa, abs(dist));
}

fn pattern_triangle(uv: vec2<f32>, size: f32) -> f32 {
    let h = size * 0.866;

    let row = floor(uv.y / h);
    let col = floor(uv.x / size);

    let local = vec2(
        uv.x - col * size,
        uv.y - row * h
    );

    var dist = min(local.y, h - local.y);
    let d1 = abs(local.x - local.y * 0.577) * 0.866;
    let d2 = abs(size - local.x - local.y * 0.577) * 0.866;
    dist = min(dist, min(d1, d2));

    let line_width = grids[0].line_widths.x;
    let aa = 1.0 / uniforms.camera_zoom;

    return 1.0 - smoothstep(0.0, line_width + aa, dist);
}

fn pattern_dots(uv: vec2<f32>, spacing: f32) -> f32 {
    let cell = floor(uv / spacing);
    let center = (cell + 0.5) * spacing;
    let dist = length(uv - center);

    let radius = grids[0].pattern_params.x;
    let aa = 1.0 / uniforms.camera_zoom;

    return 1.0 - smoothstep(radius - aa, radius + aa, dist);
}

fn pattern_lines(uv: vec2<f32>, spacing: f32, angle: f32) -> f32 {
    let c = cos(angle);
    let s = sin(angle);
    let rotated = vec2(uv.x * c + uv.y * s, -uv.x * s + uv.y * c);

    let dist = abs(rotated.y % spacing);
    let line_dist = min(dist, spacing - dist);

    let line_width = grids[0].line_widths.x;
    let aa = 1.0 / uniforms.camera_zoom;

    return 1.0 - smoothstep(0.0, line_width + aa, line_dist);
}

fn pattern_crosshatch(uv: vec2<f32>, spacing: f32, angle1: f32, angle2: f32) -> f32 {
    let lines1 = pattern_lines(uv, spacing, angle1);
    let lines2 = pattern_lines(uv, spacing, angle2);
    return max(lines1, lines2);
}

fn compute_background_pattern(uv: vec2<f32>) -> vec4<f32> {
    let grid = grids[0];
    let pattern_type = grid.pattern_type;
    let minor_spacing = grid.minor_spacing;
    let zoom = uniforms.camera_zoom;

    var effective_spacing = minor_spacing;
    var fade = 1.0;

    if ((grid.flags & 1u) != 0u) {
        let adaptive = compute_adaptive_spacing(
            minor_spacing,
            zoom,
            grid.adaptive_params.x,
            grid.adaptive_params.y
        );
        effective_spacing = adaptive.x;

        let screen_spacing = effective_spacing * zoom;
        fade = compute_fade_factor(
            screen_spacing,
            grid.adaptive_params.x,
            grid.adaptive_params.y,
            grid.adaptive_params.z
        );
    }

    let major_spacing = effective_spacing * grid.major_ratio;
    let minor_opacity = grid.opacities.x * fade;
    let major_opacity = grid.opacities.y;

    var intensity = 0.0;
    var is_major = false;

    switch (pattern_type) {
        case 0u: {
            return grid.primary_color;
        }
        case 1u: {
            let grid_pattern = pattern_grid(uv, effective_spacing, major_spacing, zoom);
            if (grid_pattern.y > 0.01) {
                intensity = grid_pattern.y;
                is_major = true;
            } else {
                intensity = grid_pattern.x;
            }
        }
        case 2u: {
            let pointy = (grid.flags & 2u) != 0u;
            intensity = pattern_hex(uv, effective_spacing, pointy);
            is_major = true;
        }
        case 3u: {
            intensity = pattern_triangle(uv, effective_spacing);
            is_major = true;
        }
        case 4u: {
            intensity = pattern_dots(uv, effective_spacing);
            is_major = true;
        }
        case 5u: {
            intensity = pattern_lines(uv, effective_spacing, grid.pattern_params.y);
            is_major = true;
        }
        case 6u: {
            intensity = pattern_crosshatch(
                uv,
                effective_spacing,
                grid.pattern_params.y,
                grid.pattern_params.z
            );
            is_major = true;
        }
        default: {
            return grid.primary_color;
        }
    }

    let opacity = select(minor_opacity, major_opacity, is_major);
    let color = select(grid.secondary_color, grid.primary_color, is_major);

    let pattern_alpha = color.a * intensity * opacity;
    let bg = grid.primary_color;
    let result_rgb = bg.rgb * (1.0 - pattern_alpha) + color.rgb * pattern_alpha;

    return vec4(result_rgb, 1.0);
}

// ============================================================================
// ENTRY POINTS
// ============================================================================

@vertex
fn vs_background(@builtin(vertex_index) vertex_index: u32) -> @builtin(position) vec4<f32> {
    var positions = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(3.0, -1.0),
        vec2<f32>(-1.0, 3.0)
    );
    return vec4<f32>(positions[vertex_index], 0.0, 1.0);
}

@fragment
fn fs_background(@builtin(position) frag_coord: vec4<f32>) -> @location(0) vec4<f32> {
    let local_coord = frag_coord.xy - uniforms.bounds_origin;
    let uv = (local_coord / (uniforms.os_scale_factor * uniforms.camera_zoom)) - uniforms.camera_position;

    return compute_background_pattern(uv);
}
