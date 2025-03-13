#![allow(unused_imports, unused_variables, dead_code)]
use iced::{
    application, color, keyboard,
    widget::{canvas::path, center},
    Element, Length, Point, Rectangle, Renderer, Theme,
};
use std::ops::Range;

use infinite::*;

fn main() -> iced::Result {
    application("H Fractal", Playground::update, Playground::view)
        .centered()
        .theme(|_| Theme::TokyoNight)
        .antialiasing(true)
        .run()
}

#[derive(Default)]
struct Playground;

#[derive(Debug, Clone)]
enum Message {}

impl Playground {
    fn update(&mut self, message: Message) {
        match message {}
    }

    fn view(&self) -> Element<Message> {
        let content = Infinite::new(Fractal).width(900).height(750);

        let content = center(content).width(Length::Fill).height(Length::Fill);

        content.into()
    }
}

const MAX_DEPTH: i32 = 100;
const MAX: Range<i32> = -MAX_DEPTH..MAX_DEPTH;

#[derive(Debug, Clone, Copy)]
struct Fractal;

#[derive(Debug, Clone, Copy, PartialEq)]
struct FractalState {
    count: i32,
    threshold: i32,
    depth: i32,
}

impl FractalState {
    const INIT_DEPTH: i32 = 8;

    fn new() -> Self {
        FractalState {
            depth: Self::INIT_DEPTH,
            threshold: 4,
            count: 0,
        }
    }

    fn zoom(&mut self, zoom_in: bool) {
        let diff = if zoom_in { 1 } else { -1 };
        self.count += diff;

        if self.count.abs() >= self.threshold {
            self.depth += self.count / self.threshold;
            self.count %= self.threshold;
        }
    }

    fn reset(&mut self) {
        self.depth = Self::INIT_DEPTH;
        self.count = 0;
    }
}

impl Program<Message, Theme, Renderer> for Fractal {
    type State = FractalState;

    fn init_state(&self) -> Self::State {
        FractalState::new()
    }

    fn draw<'a>(
        &self,
        state: &Self::State,
        theme: &Theme,
        bounds: Rectangle,
        _cursor: iced::mouse::Cursor,
        _infinite_cursor: iced::mouse::Cursor,
        _center: Point,
    ) -> Vec<Buffer<'a>> {
        let mut buffer = Buffer::new();

        let width = bounds.width / 4.0;

        let color = theme.extended_palette().primary.weak.color;

        draw(
            &mut buffer,
            color,
            Point::new(-width, 0.),
            Point::new(width, 0.),
            state.depth,
        );

        vec![buffer]
    }

    fn on_zoom(
        &self,
        state: &mut Self::State,
        _bounds: Rectangle,
        _cursor: iced::mouse::Cursor,
        _infinite_cursor: iced::mouse::Cursor,
        _focal_point: Point,
        _zoom: f32,
        diff: f32,
    ) -> Option<Message> {
        let zoom_in = diff > 0.0;

        state.zoom(zoom_in);

        None
    }

    fn on_zoom_reset(
        &self,
        state: &mut Self::State,
        _bounds: Rectangle,
        _cursor: iced::mouse::Cursor,
        _infinite_cursor: iced::mouse::Cursor,
        _zoom: f32,
    ) -> Option<Message> {
        state.reset();

        None
    }
}

fn draw(buffer: &mut Buffer<'_>, color: iced::Color, from: Point, to: Point, amount: i32) {
    if amount <= 0 {
        return;
    }

    buffer.stroke(
        Path::line(from, to),
        Stroke::default().with_color(color).with_width(3.5),
    );
    let factor = 1.0 / f32::sqrt(2.0);

    let stable_x = from.x == to.x;
    let distance = if stable_x {
        (from.y - to.y).abs()
    } else {
        (from.x - to.x).abs()
    };

    let distance = (distance * factor) / 2.0;

    let (new_from, new_to) = new_points(from, distance, stable_x);

    draw(buffer, color, new_from, new_to, amount - 1);

    let (new_from, new_to) = new_points(to, distance, stable_x);

    draw(buffer, color, new_from, new_to, amount - 1);
}

fn new_points(point: Point, distance: f32, stable_x: bool) -> (Point, Point) {
    if stable_x {
        let (x1, x2) = (point.x - distance, point.x + distance);
        (Point::new(x1, point.y), Point::new(x2, point.y))
    } else {
        let (y1, y2) = (point.y - distance, point.y + distance);
        (Point::new(point.x, y1), Point::new(point.x, y2))
    }
}
