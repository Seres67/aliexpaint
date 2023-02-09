mod ui;
mod util;

use crate::util::line::Line;
use crate::util::point::Point;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::{Color, PixelFormatEnum};
use std::mem;
use std::sync::{Arc, Mutex};

const WIDTH: u32 = 640;
const HEIGHT: u32 = 480;

fn main() -> Result<(), String> {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window("Aliexpaint", WIDTH, HEIGHT)
        .position_centered()
        .opengl()
        .build()
        .unwrap();
    let mut canvas = window
        .into_canvas()
        .build()
        .map_err(|e| e.to_string())
        .unwrap();
    let texture_creator = canvas.texture_creator();
    let mut front_buffer = texture_creator
        .create_texture_streaming(PixelFormatEnum::RGBA8888, WIDTH, HEIGHT)
        .map_err(|e| e.to_string())
        .unwrap();
    let mut back_buffer = texture_creator
        .create_texture_streaming(PixelFormatEnum::RGBA8888, WIDTH, HEIGHT)
        .map_err(|e| e.to_string())
        .unwrap();
    let mut event_pump = sdl_context.event_pump()?;
    let mut mouse_down = false;
    let line_history = Arc::new(Mutex::new(vec![]));
    let removed_lines = Arc::new(Mutex::new(vec![]));
    let mut buttons = vec![ui::button::Button::new(Point { x: 0, y: 0 }, 50, || {
        line_history.lock().unwrap().clear();
        removed_lines.lock().unwrap().clear();
    })];

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                Event::KeyDown {
                    keycode, keymod, ..
                } => {
                    if keymod == sdl2::keyboard::Mod::LCTRLMOD {
                        if keycode == Some(Keycode::Z) {
                            if let Some(line) = line_history.lock().unwrap().pop() {
                                removed_lines.lock().unwrap().push(line);
                            }
                        } else if keycode == Some(Keycode::Y) {
                            if let Some(line) = removed_lines.lock().unwrap().pop() {
                                line_history.lock().unwrap().push(line);
                            }
                        }
                    }
                }
                Event::MouseButtonDown {
                    x, y, mouse_btn, ..
                } => {
                    for button in &mut buttons {
                        if button.hovering {
                            button.click();
                            break;
                        }
                    }
                    let mut handle_button = |color: Color| {
                        mouse_down = true;
                        removed_lines.lock().unwrap().clear();
                        line_history.lock().unwrap().push(Line {
                            points: Vec::new(),
                            color,
                        });
                        let mut line_history = line_history.lock().unwrap();
                        let line = line_history.last_mut().unwrap();
                        let points = &mut line.points;
                        points.push(Point {
                            x: x as u32,
                            y: y as u32,
                        });
                    };
                    match mouse_btn {
                        sdl2::mouse::MouseButton::Left => {
                            handle_button(Color::RGBA(255, 0, 0, 255))
                        }
                        sdl2::mouse::MouseButton::Right => {
                            handle_button(Color::RGBA(0, 255, 0, 255))
                        }
                        _ => {}
                    }
                }
                Event::MouseButtonUp { .. } => {
                    mouse_down = false;
                }
                Event::MouseMotion { x, y, .. } => {
                    if x < 0 || x > WIDTH as i32 || y < 51 || y > HEIGHT as i32 {
                        mouse_down = false;
                    }
                    for button in &mut buttons {
                        button.hover(Point::new(x as u32, y as u32));
                    }
                    if mouse_down {
                        line_history
                            .lock()
                            .unwrap()
                            .last_mut()
                            .unwrap()
                            .points
                            .push(Point {
                                x: x as u32,
                                y: y as u32,
                            });
                    }
                }
                _ => {}
            }
        }

        back_buffer
            .with_lock(None, |buffer: &mut [u8], pitch: usize| {
                for y in 0..HEIGHT {
                    for x in 0..WIDTH {
                        let offset = (y * pitch as u32 + x * 4) as usize;
                        buffer[offset] = 255;
                        buffer[offset + 1] = 255;
                        buffer[offset + 2] = 255;
                        buffer[offset + 3] = 255;
                    }
                }
                for x in 0..WIDTH {
                    let offset = (50 * pitch as u32 + x * 4) as usize;
                    buffer[offset] = 0;
                    buffer[offset + 1] = 0;
                    buffer[offset + 2] = 0;
                    buffer[offset + 3] = 0;
                }
                let line_history = line_history.lock().unwrap();
                for line in line_history.iter() {
                    let color = line.color;
                    for i in 0..line.points.len() - 1 {
                        let (x1, y1) = (line.points[i].x, line.points[i].y);
                        let (x2, y2) = (line.points[i + 1].x, line.points[i + 1].y);
                        let dx = (x2 as i32 - x1 as i32).abs();
                        let dy = (y2 as i32 - y1 as i32).abs();
                        let sx: i32 = if x1 < x2 { 1 } else { -1 };
                        let sy: i32 = if y1 < y2 { 1 } else { -1 };
                        let mut err = dx - dy;
                        let mut x: i32 = x1 as i32;
                        let mut y: i32 = y1 as i32;
                        loop {
                            let offset = y as usize * pitch + x as usize * 4;
                            let offset_above = (y - 1) as usize * pitch + x as usize * 4;
                            let offset_below = (y + 1) as usize * pitch + x as usize * 4;
                            buffer[offset - 4] = color.a;
                            buffer[offset - 3] = color.b;
                            buffer[offset - 2] = color.g;
                            buffer[offset - 1] = color.r;
                            buffer[offset] = color.a;
                            buffer[offset + 1] = color.b;
                            buffer[offset + 2] = color.g;
                            buffer[offset + 3] = color.r;
                            buffer[offset + 4] = color.a;
                            buffer[offset + 5] = color.b;
                            buffer[offset + 6] = color.g;
                            buffer[offset + 7] = color.r;
                            buffer[offset_above] = color.a;
                            buffer[offset_above + 1] = color.b;
                            buffer[offset_above + 2] = color.g;
                            buffer[offset_above + 3] = color.r;
                            buffer[offset_below] = color.a;
                            buffer[offset_below + 1] = color.b;
                            buffer[offset_below + 2] = color.g;
                            buffer[offset_below + 3] = color.r;
                            if x == x2 as i32 && y == y2 as i32 {
                                break;
                            }
                            let e2 = 2 * err;
                            if e2 > -dy {
                                err -= dy;
                                x += sx;
                            }
                            if e2 < dx {
                                err += dx;
                                y += sy;
                            }
                        }
                    }
                }
            })
            .unwrap();
        mem::swap(&mut front_buffer, &mut back_buffer);
        canvas.copy(&front_buffer, None, None).unwrap();
        for button in &buttons {
            button.draw(&mut canvas);
        }
        canvas.present();
        std::thread::sleep(std::time::Duration::from_millis(8));
    }
    Ok(())
}
