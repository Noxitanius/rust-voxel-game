use std::sync::Arc;

use crate::mesh::Vertex;
use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec3};
use wgpu::util::DeviceExt;
use winit::dpi::PhysicalSize;
use winit::window::Window;

fn cube_mesh() -> (Vec<Vertex>, Vec<u32>) {
    let v = vec![
        Vertex {
            pos: [-1.0, -1.0, 1.0],
            color: [1.0, 0.2, 0.2],
        }, // 0
        Vertex {
            pos: [1.0, -1.0, 1.0],
            color: [0.2, 1.0, 0.2],
        }, // 1
        Vertex {
            pos: [1.0, 1.0, 1.0],
            color: [0.2, 0.2, 1.0],
        }, // 2
        Vertex {
            pos: [-1.0, 1.0, 1.0],
            color: [1.0, 1.0, 0.2],
        }, // 3
        Vertex {
            pos: [-1.0, -1.0, -1.0],
            color: [0.2, 1.0, 1.0],
        }, // 4
        Vertex {
            pos: [1.0, -1.0, -1.0],
            color: [1.0, 0.2, 1.0],
        }, // 5
        Vertex {
            pos: [1.0, 1.0, -1.0],
            color: [0.9, 0.9, 0.9],
        }, // 6
        Vertex {
            pos: [-1.0, 1.0, -1.0],
            color: [0.3, 0.3, 0.3],
        }, // 7
    ];

    let i: Vec<u32> = vec![
        0, 1, 2, 0, 2, 3, // front
        1, 5, 6, 1, 6, 2, // right
        5, 4, 7, 5, 7, 6, // back
        4, 0, 3, 4, 3, 7, // left
        3, 2, 6, 3, 6, 7, // top
        4, 5, 1, 4, 1, 0, // bottom
    ];

    (v, i)
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct CameraUniform {
    view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    fn new() -> Self {
        Self {
            view_proj: Mat4::IDENTITY.to_cols_array_2d(),
        }
    }
}

fn build_view_proj_from(pos: Vec3, dir: Vec3, aspect: f32) -> Mat4 {
    let eye = pos;
    let target = pos + dir;
    let up = Vec3::Y;

    let view = Mat4::look_at_rh(eye, target, up);
    let proj = Mat4::perspective_rh(45f32.to_radians(), aspect, 0.1, 200.0);
    proj * view
}

struct Depth {
    view: wgpu::TextureView,
    format: wgpu::TextureFormat,
}

impl Depth {
    fn create(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) -> Self {
        let format = wgpu::TextureFormat::Depth32Float;
        let size = wgpu::Extent3d {
            width: config.width.max(1),
            height: config.height.max(1),
            depth_or_array_layers: 1,
        };

        let tex = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("depth"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });

        let view = tex.create_view(&wgpu::TextureViewDescriptor::default());
        Self { view, format }
    }
}

pub struct Gfx {
    window: Arc<Window>,
    pub size: PhysicalSize<u32>,

    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,

    pipeline: wgpu::RenderPipeline,

    vertex_buf: Option<wgpu::Buffer>,
    index_buf: Option<wgpu::Buffer>,
    index_count: u32,

    camera_buf: wgpu::Buffer,
    camera_bg: wgpu::BindGroup,

    depth: Depth,
}

impl Gfx {
    pub async fn new(window: Arc<Window>) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::default();
        // Arc<Window> ist ok – vermeidet self-referential lifetime-Gefrickel :contentReference[oaicite:1]{index=1}
        let surface = instance
            .create_surface(window.clone())
            .expect("create surface");

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .expect("request adapter");

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                memory_hints: wgpu::MemoryHints::Performance,
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                trace: wgpu::Trace::Off,
            })
            .await
            .expect("request device");

        let caps = surface.get_capabilities(&adapter);
        let surface_format = caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: caps.present_modes[0],
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &config);

        // ----- Mesh -----
        let (verts, inds) = cube_mesh();

        let vertex_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("vertex buffer"),
            contents: bytemuck::cast_slice(&verts),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("index buffer"),
            contents: bytemuck::cast_slice(&inds),
            usage: wgpu::BufferUsages::INDEX,
        });

        let index_count = inds.len() as u32;

        // ----- Camera uniform -----
        let mut cam_u = CameraUniform::new();
        let aspect = config.width as f32 / config.height as f32;
        cam_u.view_proj = build_view_proj_from(
            Vec3::new(3.0, 2.0, 5.0),
            Vec3::new(-0.5, -0.2, -1.0),
            aspect,
        )
        .to_cols_array_2d();

        let camera_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("camera buffer"),
            contents: bytemuck::bytes_of(&cam_u),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("camera bgl"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let camera_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("camera bg"),
            layout: &camera_bgl,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buf.as_entire_binding(),
            }],
        });

        // ----- Pipeline -----
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("cube shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/cube.wgsl").into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("pipeline layout"),
            bind_group_layouts: &[&camera_bgl],
            immediate_size: 0,
        });

        let depth = Depth::create(&device, &config);

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("cube pipeline"),
            layout: Some(&pipeline_layout),

            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::layout()],
                compilation_options: Default::default(),
            },

            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),

            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                cull_mode: Some(wgpu::Face::Back),
                front_face: wgpu::FrontFace::Ccw,
                ..Default::default()
            },

            depth_stencil: Some(wgpu::DepthStencilState {
                format: depth.format,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: Default::default(),
                bias: Default::default(),
            }),

            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        Self {
            window,
            size,
            surface,
            device,
            queue,
            config,
            pipeline,
            vertex_buf: Some(vertex_buf),
            index_buf: Some(index_buf),
            index_count,
            camera_buf,
            camera_bg,
            depth,
        }
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width == 0 || new_size.height == 0 {
            return;
        }
        self.size = new_size;
        self.config.width = new_size.width;
        self.config.height = new_size.height;
        self.surface.configure(&self.device, &self.config);

        self.depth = Depth::create(&self.device, &self.config);

        // Kamera-Aspect aktualisieren
        let mut cam_u = CameraUniform::new();
        let aspect = self.config.width as f32 / self.config.height as f32;
        cam_u.view_proj = build_view_proj_from(
            Vec3::new(3.0, 2.0, 5.0),
            Vec3::new(-0.5, -0.2, -1.0),
            aspect,
        )
        .to_cols_array_2d();

        self.queue
            .write_buffer(&self.camera_buf, 0, bytemuck::bytes_of(&cam_u));
    }

    pub fn set_camera(&mut self, pos: (f32, f32, f32), dir: (f32, f32, f32)) {
        let pos = Vec3::new(pos.0, pos.1, pos.2);
        let mut dir = Vec3::new(dir.0, dir.1, dir.2);

        // Schutz: nie Nullrichtung
        if dir.length_squared() < 1e-6 {
            dir = Vec3::new(0.0, 0.0, -1.0);
        } else {
            dir = dir.normalize();
        }

        let aspect = self.config.width as f32 / self.config.height as f32;

        let mut cam_u = CameraUniform::new();
        cam_u.view_proj = build_view_proj_from(pos, dir, aspect).to_cols_array_2d();

        self.queue
            .write_buffer(&self.camera_buf, 0, bytemuck::bytes_of(&cam_u));
    }

    pub fn set_mesh(&mut self, vertices: &[Vertex], indices: &[u32]) {
        let vb = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("dynamic vertex buffer"),
                contents: bytemuck::cast_slice(vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });

        let ib = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("dynamic index buffer"),
                contents: bytemuck::cast_slice(indices),
                usage: wgpu::BufferUsages::INDEX,
            });

        self.vertex_buf = Some(vb);
        self.index_buf = Some(ib);
        self.index_count = indices.len() as u32;
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        eprintln!("RENDER");

        let frame = self.surface.get_current_texture()?;
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("render encoder"),
            });

        {
            let mut rp = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.0,
                            b: 0.2,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
                multiview_mask: None,
            });

            rp.set_pipeline(&self.pipeline);
            rp.set_bind_group(0, &self.camera_bg, &[]);
            if let (Some(vb), Some(ib)) = (&self.vertex_buf, &self.index_buf) {
                rp.set_vertex_buffer(0, vb.slice(..));
                rp.set_index_buffer(ib.slice(..), wgpu::IndexFormat::Uint32);
                rp.draw_indexed(0..self.index_count, 0, 0..1);
            }
        }

        self.queue.submit(Some(encoder.finish()));
        frame.present();
        Ok(())
    }
}

// wgpu::util::DeviceExt für create_buffer_init
