// Copyright 2021 Chay Nabors.

use std::borrow::Cow;
use std::ops::Range;
use std::sync::Weak;

use bytemuck::Zeroable;
use log::error;
use log::info;
use nalgebra::Isometry3;
use nalgebra::Matrix4;
use nalgebra::Point3;
use nalgebra::Translation3;
use nalgebra::UnitQuaternion;
use wgpu::vertex_attr_array;
use wgpu::Adapter;
use wgpu::BackendBit;
use wgpu::BindGroup;
use wgpu::BindGroupDescriptor;
use wgpu::BindGroupEntry;
use wgpu::BindGroupLayout;
use wgpu::BindGroupLayoutDescriptor;
use wgpu::BindGroupLayoutEntry;
use wgpu::BindingResource;
use wgpu::BindingType;
use wgpu::BlendState;
use wgpu::Buffer;
use wgpu::BufferAddress;
use wgpu::BufferBinding;
use wgpu::BufferBindingType;
use wgpu::BufferDescriptor;
use wgpu::BufferUsage;
use wgpu::Color;
use wgpu::ColorTargetState;
use wgpu::ColorWrite;
use wgpu::CommandEncoderDescriptor;
use wgpu::CompareFunction;
use wgpu::DepthBiasState;
use wgpu::DepthStencilState;
use wgpu::Device;
use wgpu::DeviceDescriptor;
use wgpu::DynamicOffset;
use wgpu::Extent3d;
use wgpu::Face;
use wgpu::Features;
use wgpu::FragmentState;
use wgpu::FrontFace;
use wgpu::IndexFormat;
use wgpu::InputStepMode;
use wgpu::Instance;
use wgpu::Limits;
use wgpu::LoadOp;
use wgpu::MultisampleState;
use wgpu::Operations;
use wgpu::PipelineLayout;
use wgpu::PipelineLayoutDescriptor;
use wgpu::PolygonMode;
use wgpu::PowerPreference;
use wgpu::PresentMode;
use wgpu::PrimitiveState;
use wgpu::PrimitiveTopology;
use wgpu::Queue;
use wgpu::RenderPassColorAttachment;
use wgpu::RenderPassDepthStencilAttachment;
use wgpu::RenderPassDescriptor;
use wgpu::RenderPipeline;
use wgpu::RenderPipelineDescriptor;
use wgpu::RequestAdapterOptions;
use wgpu::ShaderFlags;
use wgpu::ShaderModule;
use wgpu::ShaderModuleDescriptor;
use wgpu::ShaderSource;
use wgpu::ShaderStage;
use wgpu::StencilState;
use wgpu::Surface;
use wgpu::SwapChain;
use wgpu::SwapChainDescriptor;
use wgpu::Texture;
use wgpu::TextureDescriptor;
use wgpu::TextureDimension;
use wgpu::TextureFormat;
use wgpu::TextureUsage;
use wgpu::TextureView;
use wgpu::TextureViewDescriptor;
use wgpu::VertexBufferLayout;
use wgpu::VertexState;
use wgpu::BIND_BUFFER_ALIGNMENT;

use crate::model::Vertex;
use crate::Window;

const VERTEX_BUFFER_SIZE: u64 = 32000000;
const INDEX_BUFFER_SIZE: u64 = 32000000;
const MAX_UNIFORM_COUNT: u64 = 1 << 20;
const TEXTURE_FORMAT: TextureFormat = TextureFormat::Bgra8UnormSrgb;
const DEPTH_TEXTURE_FORMAT: TextureFormat = TextureFormat::Depth32Float;

#[repr(C, align(256))]
#[derive(Copy, Clone, Debug, Zeroable)]
struct Uniforms {
    mvp: [[f32; 4]; 4],
}

#[derive(Debug)]
struct DrawCall {
    base_vertex: i32,
    indices: Range<u32>,
}

#[derive(Debug)]
pub struct Renderer {
    _instance: Instance,
    surface: Surface,
    _adapter: Adapter,
    device: Device,
    queue: Queue,
    swap_chain_descriptor: SwapChainDescriptor,
    swap_chain: SwapChain,

    vertex_buffer: Buffer,
    index_buffer: Buffer,
    uniform_buffer: Buffer,
    _uniform_bind_group_layout: BindGroupLayout,
    uniform_bind_group: BindGroup,

    depth_texture: Texture,
    depth_texture_view: TextureView,
    _shader_module: ShaderModule,
    _pipeline_layout: PipelineLayout,
    pipeline: RenderPipeline,

    clear_color: [f64; 4],
    view: Isometry3<f32>,
    projection: Matrix4<f32>,
    _bound_texture: Option<Weak<crate::Texture>>,
    draw_calls: Vec<Vec<DrawCall>>,
    vertex_data: Vec<Vertex>,
    index_data: Vec<u32>,
    uniform_data: Vec<Uniforms>,
}

impl Renderer {
    pub(crate) async fn new(window: &Window) -> Option<Renderer> {
        info!("Initializing rendering backend");

        let instance = Instance::new(BackendBit::all());

        let surface = unsafe { instance.create_surface(window) };

        let adapter = match instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
            })
            .await
        {
            Some(adapter) => adapter,
            None => {
                error!("Failed to find any suitable graphics adapter");
                return None;
            },
        };

        let (device, queue) = match adapter
            .request_device(
                &DeviceDescriptor { label: Some("device"), features: Features::empty(), limits: Limits::default() },
                None,
            )
            .await
        {
            Ok(dq) => dq,
            Err(e) => {
                error!("Failed to acquire a graphics device: {}", e);
                return None;
            },
        };

        let window_size = window.size();

        let (swap_chain_descriptor, swap_chain) = create_swap_chain(&device, &surface, window_size);

        let vertex_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("vertex_buffer"),
            size: VERTEX_BUFFER_SIZE,
            usage: BufferUsage::VERTEX | BufferUsage::COPY_DST,
            mapped_at_creation: false,
        });

        let index_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("index_buffer"),
            size: INDEX_BUFFER_SIZE,
            usage: BufferUsage::INDEX | BufferUsage::COPY_DST,
            mapped_at_creation: false,
        });

        let uniform_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("uniform_buffer"),
            size: MAX_UNIFORM_COUNT as BufferAddress * BIND_BUFFER_ALIGNMENT,
            usage: BufferUsage::UNIFORM | BufferUsage::COPY_DST,
            mapped_at_creation: false,
        });

        let uniform_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: None,
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStage::VERTEX,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: true,
                    min_binding_size: wgpu::BufferSize::new(std::mem::size_of::<Uniforms>() as _),
                },
                count: None,
            }],
        });

        let uniform_bind_group = device.create_bind_group(&BindGroupDescriptor {
            layout: &uniform_bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: BindingResource::Buffer(BufferBinding {
                    buffer: &uniform_buffer,
                    offset: 0,
                    size: wgpu::BufferSize::new(std::mem::size_of::<Uniforms>() as _),
                }),
            }],
            label: None,
        });

        let (depth_texture, depth_texture_view) = create_depth_texture(&device, window_size);

        let shader_module = device.create_shader_module(&ShaderModuleDescriptor {
            label: Some("shader"),
            source: ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader.wgsl"))),
            flags: ShaderFlags::VALIDATION,
        });

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&uniform_bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader_module,
                entry_point: "main",
                buffers: &[VertexBufferLayout {
                    array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
                    step_mode: InputStepMode::Vertex,
                    attributes: &vertex_attr_array![
                        0 => Float32x3,
                        1 => Float32x2,
                        2 => Float32x3,
                    ],
                }],
            },
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                cull_mode: Some(Face::Back),
                clamp_depth: false,
                polygon_mode: PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: Some(DepthStencilState {
                format: DEPTH_TEXTURE_FORMAT,
                depth_write_enabled: true,
                depth_compare: CompareFunction::GreaterEqual,
                stencil: StencilState::default(),
                bias: DepthBiasState::default(),
            }),
            multisample: MultisampleState { count: 1, mask: !0, alpha_to_coverage_enabled: false },
            fragment: Some(FragmentState {
                module: &shader_module,
                entry_point: "main",
                targets: &[ColorTargetState {
                    format: swap_chain_descriptor.format,
                    blend: Some(BlendState::REPLACE),
                    write_mask: ColorWrite::ALL,
                }],
            }),
        });

        Some(Renderer {
            _instance: instance,
            surface,
            _adapter: adapter,
            device,
            queue,
            swap_chain_descriptor,
            swap_chain,

            vertex_buffer,
            index_buffer,
            uniform_buffer,
            _uniform_bind_group_layout: uniform_bind_group_layout,
            uniform_bind_group,

            depth_texture,
            depth_texture_view,
            _shader_module: shader_module,
            _pipeline_layout: pipeline_layout,
            pipeline,

            clear_color: [0., 0., 0., 1.],
            view: Isometry3::identity(),
            projection: Matrix4::identity(),
            _bound_texture: None,
            vertex_data: vec![],
            index_data: vec![],
            uniform_data: vec![],
            draw_calls: vec![],
        })
    }

    pub(crate) fn resize(&mut self, size: [u32; 2]) {
        let (swap_chain_descriptor, swap_chain) = create_swap_chain(&self.device, &self.surface, size);
        self.swap_chain_descriptor = swap_chain_descriptor;
        self.swap_chain = swap_chain;

        let (depth_texture, depth_texture_view) = create_depth_texture(&self.device, size);
        self.depth_texture = depth_texture;
        self.depth_texture_view = depth_texture_view;
    }

    pub fn set_clear_color(&mut self, clear_color: [f64; 4]) -> &mut Self {
        self.clear_color = clear_color;
        self
    }

    pub fn set_view(&mut self, view: Isometry3<f32>) -> &mut Self {
        self.view = view;
        self
    }

    pub fn set_projection(&mut self, projection: Matrix4<f32>) -> &mut Self {
        self.projection = projection;
        self
    }

    pub fn bind_texture(&mut self, _texture: &crate::Texture) -> &mut Self {
        self
    }

    pub fn draw_model(&mut self, model: &crate::Model, position: Point3<f32>, rotation: UnitQuaternion<f32>) -> &mut Self {
        let mut draw_call = vec![];
        for mesh in &model.meshes {
            draw_call.push(DrawCall {
                base_vertex: self.vertex_data.len() as i32,
                indices: self.index_data.len() as u32..(self.index_data.len() + mesh.indices.len()) as u32,
            });
            self.vertex_data.extend(&mesh.vertices);
            self.index_data.extend(&mesh.indices);
        }
        self.draw_calls.push(draw_call);

        let model = Translation3::from(position) * rotation;
        let mvp = self.projection * (self.view * model).to_homogeneous();
        self.uniform_data.push(Uniforms { mvp: mvp.into() });

        self
    }

    pub fn submit(&mut self) {
        let frame = match self.swap_chain.get_current_frame() {
            Ok(frame) => frame,
            Err(_) => {
                self.swap_chain = self.device.create_swap_chain(&self.surface, &self.swap_chain_descriptor);
                match self.swap_chain.get_current_frame() {
                    Ok(frame) => frame,
                    Err(e) => {
                        error!("Failed to acquire swapchain frame: {}", e);
                        return ();
                    },
                }
            },
        };

        let render_texture = frame.output;

        let vertex_data = bytemuck::cast_slice(&self.vertex_data);
        let index_data = bytemuck::cast_slice(&self.index_data);
        let uniform_data = unsafe {
            std::slice::from_raw_parts(
                self.uniform_data.as_ptr() as *const u8,
                self.uniform_data.len() * BIND_BUFFER_ALIGNMENT as usize,
            )
        };

        let mut encoder = self.device.create_command_encoder(&CommandEncoderDescriptor { label: None });

        {
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("render_pass"),
                color_attachments: &[RenderPassColorAttachment {
                    view: &render_texture.view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color {
                            r: self.clear_color[0],
                            g: self.clear_color[1],
                            b: self.clear_color[2],
                            a: self.clear_color[3],
                        }),
                        store: true,
                    },
                }],
                depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                    view: &self.depth_texture_view,
                    depth_ops: Some(Operations { load: LoadOp::Clear(0.0), store: true }),
                    stencil_ops: None,
                }),
            });

            if self.draw_calls.len() > 0 {
                render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(0..vertex_data.len() as u64));
                render_pass.set_index_buffer(self.index_buffer.slice(0..index_data.len() as u64), IndexFormat::Uint32);
                render_pass.set_pipeline(&self.pipeline);
                for i in 0..self.draw_calls.len() {
                    let offset = (i as DynamicOffset) * (BIND_BUFFER_ALIGNMENT as DynamicOffset);
                    render_pass.set_bind_group(0, &self.uniform_bind_group, &[offset]);
                    for k in 0..self.draw_calls[i].len() {
                        render_pass.draw_indexed(
                            self.draw_calls[i][k].indices.clone(),
                            self.draw_calls[i][k].base_vertex,
                            0..1,
                        );
                    }
                }
            }
        }

        self.queue.write_buffer(&self.vertex_buffer, 0, vertex_data);
        self.queue.write_buffer(&self.index_buffer, 0, index_data);
        self.queue.write_buffer(&self.uniform_buffer, 0, uniform_data);
        self.queue.submit(Some(encoder.finish()));

        self.vertex_data.clear();
        self.index_data.clear();
        self.uniform_data.clear();
        self.draw_calls.clear();
    }
}

fn create_swap_chain(device: &Device, surface: &Surface, size: [u32; 2]) -> (SwapChainDescriptor, SwapChain) {
    let swap_chain_descriptor = SwapChainDescriptor {
        usage: TextureUsage::RENDER_ATTACHMENT,
        format: TEXTURE_FORMAT,
        width: size[0],
        height: size[1],
        present_mode: PresentMode::Fifo,
    };

    let swap_chain = device.create_swap_chain(surface, &swap_chain_descriptor);

    (swap_chain_descriptor, swap_chain)
}

fn create_depth_texture(device: &Device, size: [u32; 2]) -> (Texture, TextureView) {
    let texture = device.create_texture(&TextureDescriptor {
        label: Some("depth texture"),
        size: Extent3d { width: size[0], height: size[1], depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: DEPTH_TEXTURE_FORMAT,
        usage: TextureUsage::RENDER_ATTACHMENT | TextureUsage::SAMPLED,
    });

    let view = texture.create_view(&TextureViewDescriptor::default());

    (texture, view)
}
