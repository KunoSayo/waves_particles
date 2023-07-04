use std::env::current_exe;
use std::io::Write;
use std::iter::once;
use std::mem::size_of;
use std::num::NonZeroU64;
use std::path::PathBuf;
use std::time::Instant;

use anyhow::{anyhow, Context};
use clap::Parser;
use gif::{Encoder, Frame, Repeat};
use pollster::FutureExt;
use wgpu::*;
use wgpu::util::{DeviceExt, StagingBelt};

use crate::spell_card::GameRegion;

mod spell_card;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
/// Import image and output waves and particles gif
///
/// Default input image is "img.png"
struct Args {
    /// The input image path
    #[clap(short, long, default_value = "img.png")]
    input: String,

    /// The output image path
    #[clap(short, long, default_value = "output.gif")]
    output: String,

    /// The output gif width
    #[clap(long, default_value_t = 512)]
    width: u32,
    /// The output gif height
    #[clap(long, default_value_t = 512)]
    height: u32,

    /// The bullet width
    #[clap(long, default_value_t = 15.0)]
    bullet_width: f32,
    /// The bullet height
    #[clap(long, default_value_t = 15.0)]
    bullet_height: f32,

    /// The center image width
    #[clap(long, default_value_t = 50.0)]
    center_width: f32,
    /// The center image height
    #[clap(long, default_value_t = 50.0)]
    center_height: f32,

    /// The bullet ways
    #[clap(short, long, default_value_t = 8)]
    ways: u32,

    #[clap(short, long, default_value_t = 10.0)]
    fps: f32,

    /// The speed for gif. Should be [1, 30]
    #[arg(value_parser = clap::value_parser ! (u16).range(1..=30), default_value_t = 10)]
    speed_gif: u16,

    /// The init angle
    #[clap(long, default_value_t = 0.0)]
    angle: f32,

    /// The speed of angle delta increase
    #[clap(long, default_value_t = 0.5)]
    delta: f32,

    /// The speed for bullet move per second
    #[clap(long, default_value_t = 96)]
    speed: u32,

    /// The frames to skip at beginning
    #[clap(long, default_value_t = 40)]
    skip: u32,

    /// The frames to record.
    #[clap(long, default_value_t = 100)]
    frames: u32,

    /// The background color of red
    #[clap(short, long, default_value_t = 1.0)]
    red: f64,

    /// The background color of green
    #[clap(short, long, default_value_t = 1.0)]
    green: f64,

    /// The background color of blue
    #[clap(short, long, default_value_t = 1.0)]
    blue: f64,

    /// The background color of alpha
    #[clap(short, long, default_value_t = 1.0)]
    alpha: f64,
}


fn main() -> anyhow::Result<()> {
    let Args {
        input: image_path, output,
        width, height,
        bullet_width, bullet_height,
        center_width, center_height, ways, fps, speed_gif, angle, delta, speed, skip, frames,
        red, green, blue, alpha
    } = Args::parse();

    let shader_path = current_exe().unwrap().join("../shader.wgsl");
    let shader_source = std::fs::read_to_string(&shader_path)
        .context(format!("Read shader failed for path {:?}", shader_path))?;

    let overwrite = true;

    if !overwrite && PathBuf::from(&output).try_exists()
        .context(format!("Output file {}", &output))? {
        return Err(anyhow!("Output file {} exists!", output));
    }

    let mut out_data: Vec<u8> = Vec::new();

    let mut game = GameRegion {
        width: width as f32,
        height: height as f32,
        ways,
        speed_per_frame: speed as f32 / fps as f32,
        angle,
        a_angle: 0.0,
        a_a_angle: delta,
        bullets: vec![],
        half_bullet_width: bullet_width / 2.0,
        half_bullet_height: bullet_height / 2.0,
    };
    let mut gif_encoder = Encoder::new(&mut out_data, width as _, height as _, &[])?;

    let instance = Instance::new(InstanceDescriptor::default());
    let adapter = instance.request_adapter(&RequestAdapterOptions {
        power_preference: PowerPreference::HighPerformance,
        ..Default::default()
    }).block_on().expect("Get adapter failed");

    let (device, queue) = adapter.request_device(&DeviceDescriptor {
        limits: adapter.limits(),
        ..Default::default()
    }, None).block_on()?;

    let screen = device.create_texture(&TextureDescriptor {
        label: None,
        size: Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: TextureFormat::Rgba8Unorm,
        usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::COPY_SRC,
        view_formats: &[TextureFormat::Rgba8Unorm],
    });

    let image_data = std::fs::read(&image_path).context(format!("File {}", image_path))?;
    let image = image::load_from_memory(&image_data[..])?;
    let view = screen.create_view(&TextureViewDescriptor::default());

    let image_texture = device.create_texture_with_data(&queue, &TextureDescriptor {
        label: None,
        size: Extent3d {
            width: image.width(),
            height: image.height(),
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: TextureFormat::Rgba8Unorm,
        usage: TextureUsages::TEXTURE_BINDING,
        view_formats: &[TextureFormat::Rgba8Unorm],
    }, image.to_rgba8().as_ref());

    let image_view = image_texture.create_view(&TextureViewDescriptor::default());
    let sampler = device.create_sampler(&SamplerDescriptor {
        mag_filter: FilterMode::Linear,
        min_filter: FilterMode::Nearest,

        ..Default::default()
    });

    let uniform_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
        label: None,
        entries: &[BindGroupLayoutEntry {
            binding: 0,
            visibility: ShaderStages::FRAGMENT,
            ty: BindingType::Sampler(SamplerBindingType::Filtering),
            count: None,
        }, BindGroupLayoutEntry {
            binding: 1,
            visibility: ShaderStages::FRAGMENT,
            ty: BindingType::Texture {
                sample_type: TextureSampleType::Float { filterable: true },
                view_dimension: TextureViewDimension::D2,
                multisampled: false,
            },
            count: None,
        }],
    });

    let uniform_bind = device.create_bind_group(&BindGroupDescriptor {
        label: None,
        layout: &uniform_layout,
        entries: &[BindGroupEntry {
            binding: 0,
            resource: BindingResource::Sampler(&sampler),
        }, BindGroupEntry {
            binding: 1,
            resource: BindingResource::TextureView(&image_view),
        }],
    });
    let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[&uniform_layout],
        push_constant_ranges: &[],
    });

    let module = device.create_shader_module(ShaderModuleDescriptor {
        label: None,
        source: ShaderSource::Wgsl(shader_source.into()),
    });

    // Pos2
    let vertex_buffer_layout = [VertexBufferLayout {
        array_stride: size_of::<[f32; 2]>() as _,
        step_mode: VertexStepMode::Vertex,
        attributes: &[VertexAttribute {
            format: VertexFormat::Float32x2,
            offset: 0,
            shader_location: 0,
        }],
    }];
    let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
        label: None,
        layout: Some(&pipeline_layout),
        vertex: VertexState {
            module: &module,
            entry_point: "vs_main",
            buffers: &vertex_buffer_layout,
        },
        primitive: PrimitiveState {
            topology: PrimitiveTopology::TriangleStrip,
            strip_index_format: None,
            front_face: Default::default(),
            cull_mode: None,
            unclipped_depth: false,
            polygon_mode: Default::default(),
            conservative: false,
        },
        depth_stencil: None,
        multisample: Default::default(),
        fragment: Some(FragmentState {
            module: &module,
            entry_point: "fs_main",
            targets: &[Some(ColorTargetState {
                format: TextureFormat::Rgba8Unorm,
                blend: Some(BlendState {
                    color: BlendComponent {
                        src_factor: BlendFactor::SrcAlpha,
                        dst_factor: BlendFactor::OneMinusSrcAlpha,
                        operation: BlendOperation::Add,
                    },
                    alpha: Default::default(),
                }),
                write_mask: Default::default(),
            })],
        }),
        multiview: None,
    });

    let vertex_buffer = device.create_buffer(&BufferDescriptor {
        label: None,
        size: 1024 * 4 * 4 * 2,
        usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });


    let screen_buffer = device.create_buffer(&BufferDescriptor {
        label: None,
        size: (width * height * 4) as _,
        usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let mut staging_belt = StagingBelt::new(4096);

    for t in 0..skip {
        println!("Pre ticking {t}/{skip}");
        game.tick();
    }


    gif_encoder.set_repeat(Repeat::Infinite)?;
    let mut last_ms = 0.0;
    let frame_interval = 1000.0 / fps;
    for frame_idx in 1..=frames {
        let now = Instant::now();
        println!("🚀Ticking {frame_idx}/{frames}");
        game.tick();
        println!("🚀Ticked in {}ms", now.elapsed().as_millis());

        let now = Instant::now();
        let mut encoder = device.create_command_encoder(&Default::default());
        println!("❀Rendering frame {frame_idx}/{frames} with bullets {} ", game.bullets.len());
        let times = (game.bullets.len() + 1023) / 1024;
        for time in (0..times).rev() {
            let upper_bound = ((time + 1) * 1024).min(game.bullets.len());
            let lower_bound = time * 1024;
            let obj_count = upper_bound - lower_bound;
            for (idx, bullet) in game.bullets[lower_bound..upper_bound].iter().enumerate() {
                let buffer = staging_belt.write_buffer(&mut encoder,
                                                       &vertex_buffer, (idx * 4 * 4 * 2) as _,
                                                       NonZeroU64::new(2 * 4 * 4).unwrap(), &device);
                game.upload(buffer, bullet);
                // left-top right-top left-down right-down
            }
            let mut rp = encoder.begin_render_pass(&RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: Operations {
                        load: if time == times - 1 {
                            LoadOp::Clear(Color {
                                r: red,
                                g: green,
                                b: blue,
                                a: alpha,
                            })
                        } else { LoadOp::Load },
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            rp.set_pipeline(&pipeline);
            rp.set_vertex_buffer(0, vertex_buffer.slice(..));
            rp.set_bind_group(0, &uniform_bind, &[]);
            for i in (0..obj_count as u32).rev() {
                rp.draw(i * 4..i * 4 + 4, 0..1);
            }
        }
        // draw center.
        {
            let mut buffer = staging_belt.write_buffer(&mut encoder,
                                                       &vertex_buffer, 0 as _,
                                                       NonZeroU64::new(2 * 4 * 4).unwrap(), &device);
            let result = bytemuck::cast_slice_mut::<_, [f32; 2]>(&mut buffer[..]);
            debug_assert_eq!(result.len(), 4);
            for (idx, point) in result.into_iter().enumerate() {
                let x = if (idx & 1) == 0 {
                    (width as f32 - center_width) / 2.0
                } else {
                    (width as f32 + center_width) / 2.0
                };

                let y = if idx < 2 {
                    (height as f32 - center_height) / 2.0
                } else {
                    (height as f32 + center_height) / 2.0
                };

                let x = (x / width as f32) * 2.0 - 1.0;

                let y = 1.0 - (y / height as f32) * 2.0;
                point[0] = x;
                point[1] = y;
            }
            let mut rp = encoder.begin_render_pass(&RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Load,
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            rp.set_pipeline(&pipeline);
            rp.set_vertex_buffer(0, vertex_buffer.slice(..));
            rp.set_bind_group(0, &uniform_bind, &[]);
            rp.draw(0..4, 0..1);
        }

        staging_belt.finish();

        // let task = queue.submit(once(encoder.finish()));
        // device.poll(Maintain::WaitForSubmissionIndex(task));
        // let mut encoder = device.create_command_encoder(&Default::default());

        encoder.copy_texture_to_buffer(ImageCopyTexture {
            texture: &screen,
            mip_level: 0,
            origin: Origin3d {
                x: 0,
                y: 0,
                z: 0,
            },
            aspect: TextureAspect::All,
        }, ImageCopyBuffer {
            buffer: &screen_buffer,
            layout: ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * width),
                rows_per_image: Some(height),
            },
        }, Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        });

        let task = queue.submit(once(encoder.finish()));
        staging_belt.recall();
        device.poll(Maintain::WaitForSubmissionIndex(task));

        let slice = screen_buffer.slice(..);
        slice.map_async(MapMode::Read, |_| {});
        device.poll(Maintain::Wait);
        let mut data = slice.get_mapped_range().to_vec();
        screen_buffer.unmap();


        let mut frame = Frame::from_rgba_speed(width as _, height as _, &mut data[..], speed_gif as i32);

        let cur_ms = frame_idx as f32 * frame_interval;
        frame.delay = ((cur_ms - last_ms) / 10.0) as _;
        last_ms = (cur_ms / 10.0).floor() * 10.0;
        gif_encoder.write_frame(&frame)?;

        println!("❀Rendered in {}ms", now.elapsed().as_millis());
    }

    drop(gif_encoder);
    let mut out_file = std::fs::File::options()
        .write(true)
        .create(true)
        .truncate(true)
        .create_new(!overwrite)
        .open(&output)
        .context(format!("File {}", output))?;

    out_file.write_all(&out_data[..])?;

    Ok(())
}
