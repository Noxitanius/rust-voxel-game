mod block;
mod chunk;
mod command;
mod game;
mod gfx;
mod input;
mod mesh;
mod player;
mod voxel_mesher;
mod world;

use std::sync::Arc;
use std::time::{Duration, Instant};

use game::Game;
use gfx::Gfx;
use input::InputState;

use winit::event::{DeviceEvent, ElementState, Event, MouseButton, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{CursorGrabMode, WindowBuilder};

fn main() {
    env_logger::init();

    let event_loop = EventLoop::new().expect("create event loop");

    let window = Arc::new(
        WindowBuilder::new()
            .with_title("Voxel Engine – wgpu")
            .build(&event_loop)
            .expect("create window"),
    );

    let mut gfx = pollster::block_on(Gfx::new(window.clone()));
    let mut game = Game::new();
    let mut input = InputState::default();
    let mut mouse_locked = false;

    let tick_dt = Duration::from_millis(50); // 20 TPS
    let mut next_tick = Instant::now() + tick_dt;

    event_loop
        .run(move |event, elwt| {
            elwt.set_control_flow(ControlFlow::WaitUntil(next_tick));

            match event {
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CloseRequested => elwt.exit(),

                    WindowEvent::Resized(size) => {
                        gfx.resize(size);
                        window.request_redraw();
                    }

                    WindowEvent::RedrawRequested => match gfx.render() {
                        Ok(_) => {}
                        Err(wgpu::SurfaceError::Lost) => gfx.resize(gfx.size),
                        Err(wgpu::SurfaceError::OutOfMemory) => elwt.exit(),
                        Err(_) => {}
                    },

                    WindowEvent::KeyboardInput { event, .. } => {
                        let down = event.state == ElementState::Pressed;
                        match event.physical_key {
                            PhysicalKey::Code(KeyCode::Escape) if down => {
                                input.toggle_mouse_lock = true
                            }
                            PhysicalKey::Code(KeyCode::Space) if down => input.jump = true,

                            PhysicalKey::Code(KeyCode::KeyW) => input.move_fwd = down,
                            PhysicalKey::Code(KeyCode::KeyS) => input.move_back = down,
                            PhysicalKey::Code(KeyCode::KeyA) => input.move_left = down,
                            PhysicalKey::Code(KeyCode::KeyD) => input.move_right = down,
                            _ => {}
                        }
                    }

                    WindowEvent::MouseInput { state, button, .. } => {
                        if state == ElementState::Pressed {
                            match button {
                                MouseButton::Left => input.break_block = true,
                                MouseButton::Right => input.place_block = true,
                                _ => {}
                            }
                        }
                    }

                    _ => {}
                },

                Event::DeviceEvent {
                    event: DeviceEvent::MouseMotion { delta },
                    ..
                } => {
                    if mouse_locked {
                        let (dx, dy) = delta;
                        let sens = 0.002_f32;
                        game.look_delta((dx as f32) * sens, (dy as f32) * sens);
                    }
                }

                Event::AboutToWait => {
                    // Mouse lock toggle
                    if input.toggle_mouse_lock {
                        mouse_locked = !mouse_locked;
                        let _ = window.set_cursor_grab(if mouse_locked {
                            CursorGrabMode::Locked
                        } else {
                            CursorGrabMode::None
                        });
                        window.set_cursor_visible(!mouse_locked);
                    }

                    let now = Instant::now();
                    if now >= next_tick {
                        game.apply_input(input);
                        game.tick(input);

                        input.clear_one_shots();
                        next_tick += tick_dt;

                        let (pos, dir) = game.camera_pos_dir();
                        gfx.set_camera(pos, dir);

                        if let Some((verts, inds)) = game.mesh_loaded_chunks_if_dirty() {
                            gfx.set_mesh(&verts, &inds);
                        }

                        window.request_redraw();
                    }
                }

                _ => {}
            }
        })
        .expect("run event loop");
}
