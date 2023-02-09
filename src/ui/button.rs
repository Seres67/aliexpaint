use crate::util::point::Point;

#[derive(Clone, Debug)]
pub struct Button<F>
where
    F: FnMut(),
{
    pub origin: Point,
    pub size: u32,
    pub hovering: bool,
    pub on_click: F,
}

impl<F> Button<F>
where
    F: FnMut(),
{
    pub fn new(origin: Point, size: u32, on_click: F) -> Button<F> {
        Button {
            origin,
            size,
            hovering: false,
            on_click,
        }
    }

    pub fn contains(&self, point: Point) -> bool {
        point.x >= self.origin.x
            && point.x <= self.origin.x + self.size
            && point.y >= self.origin.y
            && point.y <= self.origin.y + self.size
    }

    pub fn draw(&self, canvas: &mut sdl2::render::Canvas<sdl2::video::Window>) {
        if self.hovering {
            canvas.set_draw_color(sdl2::pixels::Color::RGB(255, 0, 0));
        } else {
            canvas.set_draw_color(sdl2::pixels::Color::RGB(0, 0, 255));
        }
        canvas
            .fill_rect(sdl2::rect::Rect::new(
                self.origin.x as i32,
                self.origin.y as i32,
                self.size,
                self.size,
            ))
            .expect("TODO: panic message");
    }

    pub fn hover(&mut self, point: Point) {
        if self.contains(point) {
            self.hovering = true;
        } else {
            self.hovering = false;
        }
    }

    pub fn click(&mut self) {
        (self.on_click)();
    }
}
