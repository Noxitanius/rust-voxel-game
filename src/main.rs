mod block;
mod command;
mod game;
mod input;
mod player;
mod render;
mod world;

use pixels::{Pixels, SurfaceTexture};
use render::DebugRenderer;

use std::sync::Arc;
use std::time::{Duration, Instant};

use game::Game;
use input::InputState;

use winit::event::DeviceEvent;
use winit::event::MouseButton;
use winit::window::CursorGrabMode;
use winit::{
    event::{ElementState, Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::WindowBuilder,
};

fn main() {
    // winit 0.29: new() gibt Result zurück
    let event_loop = EventLoop::new().expect("create event loop");

    // build() erwartet &EventLoopWindowTarget, nicht &Result<...>
    let window = Arc::new(
        WindowBuilder::new()
            .with_title("Voxel Engine – Phase 6")
            .build(&event_loop)
            .expect("create window"),
    );
    let window_for_loop = window.clone();
    let window_size = window.inner_size();
    let window_for_pixels = window.clone();
    let surface_texture =
        SurfaceTexture::new(window_size.width, window_size.height, &window_for_pixels);
    let mut pixels =
        Pixels::new(window_size.width, window_size.height, surface_texture).expect("create pixels");

    let mut renderer = DebugRenderer::new(window_size.width, window_size.height);

    let mut game = Game::new();
    let mut input = InputState::default();
    let mut mouse_locked = false;

    let tick_dt = Duration::from_millis(50); // 20 TPS
    let mut next_tick = Instant::now() + tick_dt;

    // winit 0.29: run() existiert, liefert Result, und Exit läuft über elwt.exit()
    event_loop
        .run(move |event, elwt| {
            // Default: warten bis zum nächsten Tick
            elwt.set_control_flow(ControlFlow::WaitUntil(next_tick));

            match event {
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CloseRequested => elwt.exit(),

                    WindowEvent::Resized(size) => {
                        let _ = pixels.resize_surface(size.width, size.height);
                        let _ = pixels.resize_buffer(size.width, size.height);
                        renderer.width = size.width;
                        renderer.height = size.height;
                    }

                    WindowEvent::ScaleFactorChanged {
                        scale_factor,
                        inner_size_writer,
                    } => {
                        let size = window_for_loop.inner_size();
                        let _ = pixels.resize_surface(size.width, size.height);
                        let _ = pixels.resize_buffer(size.width, size.height);
                        renderer.width = size.width;
                        renderer.height = size.height;
                    }

                    WindowEvent::RedrawRequested => {
                        renderer.draw(pixels.frame_mut(), &game);
                        if pixels.render().is_err() {
                            elwt.exit();
                        }
                    }

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
                    // Mouse lock toggle vor Tick anwenden
                    if input.toggle_mouse_lock {
                        mouse_locked = !mouse_locked;
                        let _ = window_for_loop.set_cursor_grab(if mouse_locked {
                            CursorGrabMode::Locked
                        } else {
                            CursorGrabMode::None
                        });

                        window_for_loop.set_cursor_visible(!mouse_locked);
                    }

                    let now = Instant::now();
                    if now >= next_tick {
                        game.apply_input(input);
                        game.tick(input);

                        window_for_loop.request_redraw();

                        // one-shot inputs zurücksetzen
                        input.clear_one_shots();

                        next_tick += tick_dt;
                    }
                }

                _ => {}
            }
        })
        .expect("run event loop");
}
