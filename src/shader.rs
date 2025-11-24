use crate::vertex::Vertex;
use std::fs;
use wgpu::{
    BlendState, ColorTargetState, ColorWrites, Device, Face, FragmentState, FrontFace,
    MultisampleState, PipelineCompilationOptions, PipelineLayout, PipelineLayoutDescriptor,
    PolygonMode, PrimitiveState, PrimitiveTopology, RenderPipeline, RenderPipelineDescriptor,
    ShaderModule, ShaderModuleDescriptor, ShaderSource, SurfaceConfiguration, VertexState,
};

pub struct Shader {
    pub module: ShaderModule,
    pub layout: PipelineLayout,
    pub pipeline: RenderPipeline,
}

impl Shader {
    pub fn new(device: &Device, config: &SurfaceConfiguration, path: &str) -> anyhow::Result<Self> {
        let src = fs::read_to_string(env!("CARGO_MANIFEST_DIR").to_owned() + path)?;
        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some(path),
            source: ShaderSource::Wgsl(src.into()),
        });
        let render_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some(path),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some(path),
            layout: Some(&render_pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::desc()],
                compilation_options: PipelineCompilationOptions::default(),
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(ColorTargetState {
                    format: config.format,
                    blend: Some(BlendState::REPLACE),
                    write_mask: ColorWrites::ALL,
                })],
                compilation_options: PipelineCompilationOptions::default(),
            }),
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                cull_mode: Some(Face::Back),
                unclipped_depth: false,
                polygon_mode: PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        Ok(Self {
            module: shader,
            layout: render_pipeline_layout,
            pipeline: render_pipeline,
        })
    }
}
