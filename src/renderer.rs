use std::{borrow::Cow, ops::Range, rc::Rc, rc::Weak};

use bytemuck::{Pod, Zeroable};
use log::{error, info};
use wgpu::{
    Adapter,
    BIND_BUFFER_ALIGNMENT,
    BackendBit,
    BindGroup,
    BindGroupDescriptor,
    BindGroupEntry,
    BindGroupLayout,
    BindGroupLayoutDescriptor,
    BindGroupLayoutEntry,
    BindingResource,
    BindingType,
    BlendState,
    Buffer,
    BufferAddress,
    BufferBinding,
    BufferBindingType,
    BufferDescriptor,
    BufferUsage,
    Color,
    ColorTargetState,
    ColorWrite,
    CommandEncoderDescriptor,
    CompareFunction,
    DepthBiasState,
    DepthStencilState,
    Device,
    DeviceDescriptor,
    DynamicOffset,
    Extent3d,
    Face,
    Features,
    FragmentState,
    FrontFace,
    IndexFormat,
    InputStepMode,
    Instance,
    Limits,
    LoadOp,
    MultisampleState,
    Operations,
    PipelineLayout,
    PipelineLayoutDescriptor,
    PolygonMode,
    PowerPreference,
    PresentMode,
    PrimitiveState,
    PrimitiveTopology,
    Queue,
    RenderPassColorAttachment,
    RenderPassDepthStencilAttachment,
    RenderPassDescriptor,
    RenderPipeline,
    RenderPipelineDescriptor,
    RequestAdapterOptions,
    ShaderFlags,
    ShaderModule,
    ShaderModuleDescriptor,
    ShaderSource,
    ShaderStage,
    StencilState,
    Surface,
    SwapChain,
    SwapChainDescriptor,
    Texture,
    TextureDescriptor,
    TextureDimension,
    TextureFormat,
    TextureUsage,
    TextureView,
    TextureViewDescriptor,
    VertexBufferLayout,
    VertexState,
    util::{BufferInitDescriptor, DeviceExt},
    vertex_attr_array
};

use crate::{content::model::Vertex};

use super::window::Window;

const VERTEX_BUFFER_SIZE: u64 = 32000000;
const INDEX_BUFFER_SIZE: u64 = 32000000;
const MAX_LOCAL_COUNT: u64 = 1 << 20;
const TEXTURE_FORMAT: TextureFormat = TextureFormat::Bgra8UnormSrgb;
const DEPTH_TEXTURE_FORMAT: TextureFormat = TextureFormat::Depth32Float;

#[repr(C, align(256))]
#[derive(Clone, Copy, Zeroable)]
struct Locals {
    view_pos: [f32; 4],
    view_proj: [[f32; 4]; 4],
    position: [f32; 3],
}

struct DrawCall {
    base_vertex: i32,
    indices: Range<u32>,
}

pub struct Renderer {
    instance: Instance,
    surface: Surface,
    adapter: Adapter,
    device: Device,
    queue: Queue,
    swap_chain_descriptor: SwapChainDescriptor,
    swap_chain: SwapChain,
    
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    local_buffer: Buffer,
    local_bind_group_layout: BindGroupLayout,
    local_bind_group: BindGroup,

    depth_texture: Texture,
    depth_texture_view: TextureView,
    shader_module: ShaderModule,
    pipeline_layout: PipelineLayout,
    pipeline: RenderPipeline,

    clear_color: [f64; 4],
    view_pos: [f32; 4],
    view_proj: [[f32; 4]; 4],
    bound_texture: Option<Weak<crate::content::texture::Texture>>,
    draw_calls: Vec<DrawCall>,
    vertex_data: Vec<Vertex>,
    index_data: Vec<u32>,
    local_data: Vec<Locals>,
}

impl Renderer {
    pub(crate) async fn new(window: &Window) -> Option<Renderer> {
        info!("Initializing rendering backend");

        let instance = Instance::new(BackendBit::all());
        
        let surface = unsafe { instance.create_surface(window) };

        let adapter = match instance.request_adapter(&RequestAdapterOptions {
            power_preference: PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
        }).await {
            Some(adapter) => adapter,
            None => {
                error!("Failed to find any suitable graphics adapter");
                return None;
            },
        };

        let (device, queue) = match adapter.request_device(
            &DeviceDescriptor {
                label: Some("device"),
                features: Features::empty(),
                limits: Limits::default(),
            },
            None,
        ).await {
            Ok(dq) => dq,
            Err(e) => {
                error!("Failed to acquire a graphics device: {}", e);
                return None;
            }
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

        let local_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("local_buffer"),
            size: MAX_LOCAL_COUNT as BufferAddress * BIND_BUFFER_ALIGNMENT,
            usage: BufferUsage::UNIFORM | BufferUsage::COPY_DST,
            mapped_at_creation: false,
        });

        let local_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStage::VERTEX,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: true,
                        min_binding_size: wgpu::BufferSize::new(std::mem::size_of::<Locals>() as _),
                    },
                    count: None
                },
            ]
        });

        let local_bind_group = device.create_bind_group(&BindGroupDescriptor {
            layout: &local_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::Buffer(BufferBinding {
                        buffer: &local_buffer,
                        offset: 0,
                        size: wgpu::BufferSize::new(std::mem::size_of::<Locals>() as _),
                    },
                )},
            ],
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
            bind_group_layouts: &[&local_bind_group_layout],
            push_constant_ranges: &[]
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
                }]
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
                depth_compare: CompareFunction::Less,
                stencil: StencilState::default(),
                bias: DepthBiasState::default(),
            }),
            multisample: MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(FragmentState {
                module: &shader_module,
                entry_point: "main",
                targets: &[ColorTargetState {
                    format: swap_chain_descriptor.format,
                    blend: Some(BlendState::REPLACE),
                    write_mask: ColorWrite::ALL,
                }]
            }),
        });

        Some(Renderer {
            instance,
            surface,
            adapter,
            device,
            queue,
            swap_chain_descriptor,
            swap_chain,

            vertex_buffer,
            index_buffer,
            local_buffer,
            local_bind_group_layout,
            local_bind_group,

            depth_texture,
            depth_texture_view,
            shader_module,
            pipeline_layout,
            pipeline,

            clear_color: [0., 0., 0., 1.],
            view_pos: [0., 0., 0., 1.],
            view_proj: [
                [1., 0., 0., 0.],
                [0., 1., 0., 0.],
                [0., 0., 1., 0.],
                [0., 0., 0., 1.],
            ],
            bound_texture: None,
            vertex_data: vec![],
            index_data: vec![],
            local_data: vec![],
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

    pub fn set_view_position(&mut self, view_position: [f32; 3]) -> &mut Self {
        self.view_pos = [view_position[0], view_position[1], view_position[2], 1.];
        self
    }

    pub fn set_view_projection(&mut self, view_projection: [[f32; 4]; 4]) -> &mut Self {
        self.view_proj = view_projection;
        self
    }

    pub fn bind_texture(&mut self, texture: &crate::Texture) -> &mut Self {
        self.bound_texture = Some(Rc::downgrade(&texture.0));
        self
    }

    pub fn draw_model(&mut self, model: &crate::Model, position: [f32; 3]) -> &mut Self {
        for mesh in &model.meshes {
            self.draw_calls.push(DrawCall {
                base_vertex: self.vertex_data.len() as i32,
                indices: self.index_data.len() as u32..(self.index_data.len() + mesh.indices.len()) as u32,
            });
            self.vertex_data.extend(&mesh.vertices);
            self.index_data.extend(&mesh.indices);
        }
        self.local_data.push(Locals {
            view_pos: self.view_pos,
            view_proj: self.view_proj, 
            position,
        });
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
                    }
                }
            }
        };

        let render_texture = frame.output;

        let vertex_data = bytemuck::cast_slice(&self.vertex_data);
        let index_data = bytemuck::cast_slice(&self.index_data);
        let local_data = unsafe {
            std::slice::from_raw_parts(
                self.local_data.as_ptr() as *const u8,
                self.local_data.len() * BIND_BUFFER_ALIGNMENT as usize,
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
                    depth_ops: Some(Operations {
                        load: LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });

            if self.draw_calls.len() > 0 {
                render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(0..vertex_data.len() as u64));
                render_pass.set_index_buffer(self.index_buffer.slice(0..index_data.len() as u64), IndexFormat::Uint32);
                render_pass.set_pipeline(&self.pipeline);
                for i in 0..self.draw_calls.len() {
                    let offset = (i as DynamicOffset) * (BIND_BUFFER_ALIGNMENT as DynamicOffset);
                    render_pass.set_bind_group(0, &self.local_bind_group, &[offset]);
                    render_pass.draw_indexed(self.draw_calls[i].indices.clone(), self.draw_calls[i].base_vertex, 0..1);
                }
            }
        }

        self.queue.write_buffer(&self.vertex_buffer, 0, vertex_data);
        self.queue.write_buffer(&self.index_buffer, 0, index_data);
        self.queue.write_buffer(&self.local_buffer, 0, local_data);
        self.queue.submit(Some(encoder.finish()));

        self.vertex_data.clear();
        self.index_data.clear();
        self.local_data.clear();
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
