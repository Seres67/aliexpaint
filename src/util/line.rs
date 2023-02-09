use crate::util::point::Point;
use sdl2::pixels::Color;

pub struct Line {
    pub points: Vec<Point>,
    pub color: Color,
}
