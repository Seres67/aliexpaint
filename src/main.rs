use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::{Color, PixelFormatEnum};
use std::mem;

const WIDTH: u32 = 640;
const HEIGHT: u32 = 480;

struct Point {
    x: i32,
    y: i32,
}

struct Line {
    points: Vec<Point>,
    color: Color,
}

fn main() -> Result<(), String> {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window("Example", WIDTH, HEIGHT)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
    let texture_creator = canvas.texture_creator();
    let mut front_buffer = texture_creator
        .create_texture_streaming(PixelFormatEnum::RGBA8888, WIDTH, HEIGHT)
        .map_err(|e| e.to_string())?;
    let mut back_buffer = texture_creator
        .create_texture_streaming(PixelFormatEnum::RGBA8888, WIDTH, HEIGHT)
        .map_err(|e| e.to_string())?;
    let mut line_history: Vec<Line> = vec![];
    let mut removed_lines: Vec<Line> = vec![];
    let mut event_pump = sdl_context.event_pump()?;
    let mut mouse_down = false;
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
                            if let Some(line) = line_history.pop() {
                                removed_lines.push(line);
                            }
                        } else if keycode == Some(Keycode::Y) {
                            if let Some(line) = removed_lines.pop() {
                                line_history.push(line);
                            }
                        }
                    }
                }
                Event::MouseButtonDown {
                    x, y, mouse_btn, ..
                } => {
                    let mut handle_button = |color: Color| {
                        mouse_down = true;
                        removed_lines.clear();
                        line_history.push(Line {
                            points: Vec::new(),
                            color,
                        });
                        let line = line_history.last_mut().unwrap();
                        line.points.push(Point { x, y });
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
                    if mouse_down {
                        line_history.last_mut().unwrap().points.push(Point { x, y });
                    }
                }
                _ => {}
            }
        }
        back_buffer.with_lock(None, |buffer: &mut [u8], pitch: usize| {
            for y in 0..HEIGHT {
                for x in 0..WIDTH {
                    let offset = (y * pitch as u32 + x * 4) as usize;
                    buffer[offset] = 255;
                    buffer[offset + 1] = 255;
                    buffer[offset + 2] = 255;
                    buffer[offset + 3] = 255;
                }
            }
            for line in line_history.iter() {
                let color = line.color;
                for i in 0..line.points.len() - 1 {
                    let (x1, y1) = (line.points[i].x, line.points[i].y);
                    let (x2, y2) = (line.points[i + 1].x, line.points[i + 1].y);
                    let dx = (x2 - x1).abs();
                    let dy = (y2 - y1).abs();
                    let sx = if x1 < x2 { 1 } else { -1 };
                    let sy = if y1 < y2 { 1 } else { -1 };
                    let mut err = dx - dy;
                    let mut x = x1;
                    let mut y = y1;
                    loop {
                        let offset = y as usize * pitch + x as usize * 4;
                        buffer[offset] = color.a;
                        buffer[offset + 1] = color.b;
                        buffer[offset + 2] = color.g;
                        buffer[offset + 3] = color.r;
                        if x == x2 && y == y2 {
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
        })?;
        mem::swap(&mut front_buffer, &mut back_buffer);
        canvas.copy(&front_buffer, None, None)?;
        canvas.present();
        std::thread::sleep(std::time::Duration::from_millis(16));
    }
    Ok(())
}
