use crate::util::line::Line;

pub mod button;

pub trait Drawable {
    fn draw(&self, canvas: &mut sdl2::render::Canvas<sdl2::video::Window>);
}

pub trait Clickable {
    fn click(&self, drawn_lines: &mut Vec<Line>, removed_lines: &mut Vec<Line>);
}
