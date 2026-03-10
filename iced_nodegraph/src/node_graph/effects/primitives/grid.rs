//! Grid background primitive for NodeGraph.
//!
//! Renders the background pattern (grid, dots, hex, etc.) behind all other elements.

use std::sync::Arc;

use encase::ShaderSize;
use iced::Rectangle;
use iced::wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, Buffer, BufferDescriptor, BufferUsages, Device,
    Queue, TextureFormat,
};
use iced_wgpu::graphics::Viewport;
use iced_wgpu::primitive::{Pipeline, Primitive};

use crate::style::BackgroundStyle;

use super::super::pipeline::types;
use super::super::shared::SharedNodeGraphResources;
use super::RenderContext;

/// Primitive for rendering the background grid pattern.
#[derive(Debug, Clone)]
pub struct GridPrimitive {
    /// Shared rendering context
    pub context: RenderContext,
    /// Background style configuration
    pub background_style: BackgroundStyle,
}

/// Pipeline for GridPrimitive rendering.
pub struct GridPipeline {
    /// Shared resources (shader, pipeline, layout)
    shared: Arc<SharedNodeGraphResources>,
    /// Uniform buffer
    uniforms: Buffer,
    /// Grid storage buffer
    grids: Buffer,
    /// Bind group for rendering
    bind_group: BindGroup,
}

impl Pipeline for GridPipeline {
    fn new(device: &Device, _queue: &Queue, format: TextureFormat) -> Self {
        let shared = SharedNodeGraphResources::get_or_init(device, format);

        let uniforms = device.create_buffer(&BufferDescriptor {
            label: Some("grid_uniforms"),
            size: <types::Uniforms as ShaderSize>::SHADER_SIZE.get(),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let grids = device.create_buffer(&BufferDescriptor {
            label: Some("grid_grids_buffer"),
            size: <types::Grid as ShaderSize>::SHADER_SIZE.get(),
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("grid_bind_group"),
            layout: &shared.bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: uniforms.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: grids.as_entire_binding(),
                },
            ],
        });

        Self {
            shared,
            uniforms,
            grids,
            bind_group,
        }
    }
}

impl Primitive for GridPrimitive {
    type Pipeline = GridPipeline;

    fn prepare(
        &self,
        pipeline: &mut Self::Pipeline,
        _device: &Device,
        queue: &Queue,
        bounds: &Rectangle,
        viewport: &Viewport,
    ) {
        let scale = viewport.scale_factor();
        let style = &self.background_style;

        let uniforms = types::Uniforms {
            os_scale_factor: scale,
            camera_zoom: self.context.camera_zoom,
            camera_position: glam::Vec2::new(
                self.context.camera_position.x,
                self.context.camera_position.y,
            ),
            num_nodes: 0,
            time: self.context.time,
            bounds_origin: glam::Vec2::new(bounds.x * scale, bounds.y * scale),
            bounds_size: glam::Vec2::new(bounds.width * scale, bounds.height * scale),
            _pad0: glam::Vec2::ZERO,
        };

        let mut uniform_buffer = encase::UniformBuffer::new(Vec::new());
        uniform_buffer
            .write(&uniforms)
            .expect("Failed to write uniforms");
        queue.write_buffer(&pipeline.uniforms, 0, uniform_buffer.as_ref());

        let grid = types::Grid {
            pattern_type: style.pattern.type_id(),
            flags: (if style.adaptive_zoom { 1u32 } else { 0 })
                | (if style.hex_pointy_top { 2u32 } else { 0 }),
            minor_spacing: style.minor_spacing,
            major_ratio: style
                .major_spacing
                .map(|m| m / style.minor_spacing)
                .unwrap_or(0.0),
            line_widths: glam::Vec2::new(style.minor_width, style.major_width),
            opacities: glam::Vec2::new(style.minor_opacity, style.major_opacity),
            primary_color: glam::Vec4::new(
                style.primary_color.r,
                style.primary_color.g,
                style.primary_color.b,
                style.primary_color.a,
            ),
            secondary_color: glam::Vec4::new(
                style.secondary_color.r,
                style.secondary_color.g,
                style.secondary_color.b,
                style.secondary_color.a,
            ),
            pattern_params: glam::Vec4::new(
                style.dot_radius,
                style.line_angle,
                style.crosshatch_angle,
                0.0,
            ),
            adaptive_params: glam::Vec4::new(
                style.adaptive_min_spacing,
                style.adaptive_max_spacing,
                style.adaptive_fade_range,
                0.0,
            ),
        };

        let mut grid_buffer = encase::StorageBuffer::new(Vec::new());
        grid_buffer.write(&grid).expect("Failed to write grid");
        queue.write_buffer(&pipeline.grids, 0, grid_buffer.as_ref());
    }

    fn draw(
        &self,
        pipeline: &Self::Pipeline,
        render_pass: &mut iced::wgpu::RenderPass<'_>,
    ) -> bool {
        render_pass.set_pipeline(&pipeline.shared.grid_pipeline);
        render_pass.set_bind_group(0, &pipeline.bind_group, &[]);
        render_pass.draw(0..3, 0..1); // Fullscreen triangle
        true
    }
}
