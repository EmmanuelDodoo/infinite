#![allow(dead_code, unused_imports)]
use iced::{
    application, color,
    widget::{
        canvas::{Path, Text},
        center,
    },
    Element, Length, Point, Renderer, Theme,
};

use infinite::*;

fn main() -> iced::Result {
    application("Playground", Playground::update, Playground::view)
        .centered()
        .theme(|_| Theme::TokyoNight)
        .antialiasing(true)
        .run()
}

#[derive(Default)]
struct Playground;

#[derive(Debug)]
enum Message {}

impl Playground {
    fn update(&mut self, message: Message) {
        match message {}
    }

    fn graph(&self) -> Infinite<'_, Graph, Message, Theme, Renderer> {
        let infinite = Infinite::new(Graph);
        infinite
    }

    fn view(&self) -> Element<Message> {
        let content = self.graph().width(900).height(750);
        //let content = text("Work In Progress");

        let content = center(content).width(Length::Fill).height(Length::Fill);

        content.into()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
/// End points may be 0 but interval should never be 0
/// Negative steps not supported, step of zero not supported
struct Scale {
    start: f32,
    og_start: f32,
    og_end: f32,
    end: f32,
    step: f32,
    og_step: f32,
}

impl Scale {
    /// Bounds should never be 0
    fn new(bounds: f32) -> Self {
        assert_ne!(bounds, 0.0);
        Self {
            start: -bounds,
            og_start: -bounds,
            end: bounds,
            og_end: bounds,
            step: 1.,
            og_step: 1.,
        }
    }

    fn scroll(&mut self, amount: f32) {
        let scroll = self.step * amount;
        self.start += scroll;
        self.end += scroll;
    }

    /// Negative steps not supported, step of zero not supported
    fn step(mut self, step: f32) -> Self {
        assert!(step >= 0.0);

        self.step = step;
        self
    }

    fn zoom(&mut self, expand: bool) {
        let step = self.scale(expand);
        self.step = step;

        self.start = 1.0 * self.og_start * step;
        self.end = 1.0 * self.og_end * step;
    }

    fn reset(&mut self) {
        self.start = self.og_start;
        self.end = self.og_end;
        self.step = self.og_step;
    }

    fn scale(&self, expand: bool) -> f32 {
        let step = self.step.log10();

        let exp = step;
        let fract = step.fract().abs();

        let (base, exp) = if expand {
            Self::grow_step(fract, exp)
        } else {
            Self::shrink_step(fract, exp)
        };

        let exp = exp.floor();

        (base as f32) * 10_f32.powf(exp)
    }

    fn grow_step(fract: f32, exp: f32) -> (f32, f32) {
        // Frac has not been truncated
        let mut exp = exp;

        let base = if exp >= 0. {
            match fract {
                x if x >= 0. && x < 0.3 => 2,
                x if x < 0.697 => 5,
                _ => {
                    exp += 1.;
                    1
                }
            }
        } else {
            match fract {
                0. => 2,
                x if x < 0.31 => {
                    exp += 1.;
                    1
                }
                x if x < 0.699 => 5,
                _ => 2,
            }
        };

        (base as f32, exp)
    }

    fn shrink_step(fract: f32, exp: f32) -> (f32, f32) {
        let mut exp = exp;

        let base = if exp >= 0. {
            match fract {
                0. => {
                    exp -= 1.;
                    5
                }
                x if x < 0.31 => 1,
                x if x < 0.699 => 2,
                _ => 5,
            }
        } else {
            match fract {
                0. => {
                    exp -= 1.;
                    5
                }
                x if x < 0.3 => 5,
                x if x < 0.69 => 2,
                _ => 1,
            }
        };

        (base as f32, exp)
    }
}

impl From<f32> for Scale {
    fn from(value: f32) -> Self {
        Self::new(value)
    }
}

#[derive(Debug, Clone, Copy)]
struct ScaleIter {
    current: f32,
    end: f32,
    step: f32,
}

impl Iterator for ScaleIter {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current > self.end {
            return None;
        }

        let temp = self.current;
        self.current += self.step;

        Some(temp)
    }
}

impl From<Scale> for ScaleIter {
    fn from(value: Scale) -> Self {
        Self {
            current: value.start,
            end: value.end,
            step: value.step,
        }
    }
}

impl IntoIterator for Scale {
    type IntoIter = ScaleIter;
    type Item = f32;

    fn into_iter(self) -> Self::IntoIter {
        self.into()
    }
}

struct Graph;

#[derive(Debug, Clone, Copy, PartialEq)]
struct GraphState {
    x_scale: Scale,
    scroll: iced::Vector,
    zoom_state: ZoomState,
}

impl GraphState {
    fn range(&self) -> impl Iterator<Item = f32> {
        self.x_scale.into_iter()
    }

    /// Returns the physical distance between x axis points given the width of
    /// the bounds
    fn x_point_width(&self, width: f32) -> f32 {
        let k = self.zoom_state.kx;
        width * 0.5 / k
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
enum ZoomKind {
    #[default]
    None,
    ZoomedIn(u32),
    ZoomedOut(u32),
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct ZoomState {
    tracker: i16,
    threshold: i16,
    // Determines how far apart two points are from each other
    kx: f32,
    og_kx: f32,
    scale: f32,
    kind: ZoomKind,
}

impl Default for ZoomState {
    fn default() -> Self {
        Self {
            tracker: 0,
            threshold: 5,
            scale: 1.0,
            kx: 5.0,
            og_kx: 5.0,
            kind: ZoomKind::None,
        }
    }
}

impl ZoomState {
    fn on_zoom(&mut self, x_scale: &mut Scale, zoom: f32, diff: f32) {
        self.scale = zoom;

        let diff = (diff * 10.0) as i16;

        self.tracker += diff;

        let is_zoom_in = diff > 0;

        let kx_delta = 1.25;

        match self.kind {
            ZoomKind::None => {
                if self.tracker.abs() >= self.threshold {
                    self.tracker %= self.threshold;
                    x_scale.zoom(!is_zoom_in);

                    if is_zoom_in {
                        self.kind = ZoomKind::ZoomedIn(1);
                    } else {
                        self.kind = ZoomKind::ZoomedOut(1);
                    }
                }
            }
            ZoomKind::ZoomedIn(amt) => {
                let threshold = self.threshold;

                if is_zoom_in && self.tracker >= threshold {
                    self.threshold = threshold;
                    self.tracker %= self.threshold;
                    x_scale.zoom(!is_zoom_in);
                    self.kx = self.kx / kx_delta;

                    self.kind = ZoomKind::ZoomedIn(amt + 1);
                } else if !is_zoom_in && self.tracker < 0 {
                    x_scale.zoom(!is_zoom_in);

                    let amt = amt - 1;

                    self.kx = (self.kx * kx_delta).max(self.og_kx);
                    if amt == 0 {
                        self.tracker = threshold + self.tracker;
                        self.kind = ZoomKind::None;
                    } else {
                        self.tracker = threshold + self.tracker;
                        self.threshold = threshold;
                        self.kind = ZoomKind::ZoomedIn(amt);
                    }
                }
            }
            ZoomKind::ZoomedOut(amt) => {
                let threshold = self.threshold;

                if !is_zoom_in && self.tracker <= -threshold {
                    self.threshold = threshold;
                    self.tracker %= self.threshold;
                    x_scale.zoom(!is_zoom_in);
                    self.kx = self.kx * kx_delta;

                    self.kind = ZoomKind::ZoomedOut(amt + 1)
                } else if is_zoom_in && self.tracker > 0 {
                    x_scale.zoom(!is_zoom_in);

                    let amt = amt - 1;

                    self.kx = (self.kx / kx_delta).max(self.og_kx);
                    if amt == 0 {
                        self.tracker = -threshold + self.tracker;
                        self.kind = ZoomKind::None;
                    } else {
                        self.tracker = -threshold + self.tracker;
                        self.threshold = threshold;
                        self.kind = ZoomKind::ZoomedOut(amt);
                    }
                }
            }
        }
    }
}

impl<Message> Program<Message, Theme, Renderer> for Graph {
    type State = GraphState;

    fn init_state(&self) -> Self::State {
        GraphState {
            x_scale: Scale::new(10.0.into()),
            scroll: iced::Vector::new(0., 0.),
            zoom_state: ZoomState::default(),
        }
    }

    fn draw<'a>(
        &self,
        state: &Self::State,
        theme: &Theme,
        bounds: iced::Rectangle,
        _cursor: iced::mouse::Cursor,
        _infinite_cursor: iced::mouse::Cursor,
        center: iced::Point,
    ) -> Vec<Buffer<'a>> {
        use iced::widget::canvas::Stroke;
        let color2 = color!(128, 0, 128);
        let color = color!(0, 128, 128);
        let color1 = color!(102, 51, 153);

        let axis_color = theme.extended_palette().secondary.base.color;
        let axis_width = 2.5;

        let outline_color = theme.extended_palette().secondary.base.color;
        let point_outline_width = axis_width * 0.5;
        let outline_width = point_outline_width * 0.5;

        let dummies = {
            let mut buffer = Buffer::new();
            buffer.draw_text(Text {
                content: "Testing Infinite".into(),
                position: (15., 45.).into(),
                size: 20.0.into(),
                ..Default::default()
            });

            let path = Path::circle((0., 0.).into(), 5.0.into());
            buffer.fill(path, color2);

            buffer.fill_rounded_rectangle((120.0, 120.), (150., 100.), 10., color);

            buffer
        };

        let axis = {
            let mut buffer = Buffer::new().scale_all(false);

            let x_axis = {
                let x = bounds.width / 2.0;
                Path::line((-center.x + x, 0.).into(), (-center.x - x, 0.).into())
            };

            buffer.stroke(
                x_axis,
                Stroke::default()
                    .with_color(axis_color)
                    .with_width(axis_width),
            );

            let y_axis = {
                let y = bounds.height / 2.0;
                Path::line((0., center.y + y).into(), (0., center.y - y).into())
            };

            buffer.stroke(
                y_axis,
                Stroke::default()
                    .with_color(axis_color)
                    .with_width(axis_width),
            );

            buffer
        };

        let x_points = {
            let mut buffer = Buffer::new();

            let width = state.x_point_width(bounds.width);
            let height = bounds.height;
            let height = height / state.zoom_state.scale.max(0.01);
            let pad = 18.0;

            for point in state.range() {
                let x = width * point as f32;
                let _spacing = if x == 0. {
                    6.5
                } else if x < 0. {
                    3.0
                } else {
                    0.
                };

                buffer.draw_text(Text {
                    content: format!("{point:.2}"),
                    position: (x, 0.).into(),
                    horizontal_alignment: iced::alignment::Horizontal::Center,
                    ..Default::default()
                });

                let width = width / 5.0;

                for i in 0..=0 {
                    let outline_width = if i == 0 {
                        point_outline_width
                    } else {
                        outline_width
                    };

                    let pad = if i == 0 { pad } else { 0.0 };

                    if i == 0 && point == 0.0 {
                        continue;
                    }

                    let x = x + ((i as f32) * width);
                    let outline = Path::line((x, center.y + height).into(), (x, 0.).into());
                    buffer.stroke(
                        outline,
                        Stroke::default()
                            .with_color(outline_color)
                            .with_width(outline_width),
                    );

                    let outline = Path::line((x, -pad).into(), (x, center.y - height).into());
                    buffer.stroke(
                        outline,
                        Stroke::default()
                            .with_color(outline_color)
                            .with_width(outline_width),
                    );
                }
            }

            buffer
        };

        let temp = {
            let mut buffer = Buffer::new().scale_all(false).anchor_all(Anchor::Both);

            let line = {
                let y = bounds.height / 2.0;
                Path::line((0., center.y + y).into(), (0., center.y - y).into())
            };

            buffer.stroke(line, Stroke::default().with_width(3.0).with_color(color1));

            buffer
        };

        vec![axis, x_points, dummies, temp]
    }

    fn on_scroll(
        &self,
        state: &mut Self::State,
        bounds: iced::Rectangle,
        _cursor: iced::mouse::Cursor,
        _infinite_cursor: iced::mouse::Cursor,
        _scroll: iced::Vector,
        diff: iced::Vector,
    ) -> Option<Message> {
        let scroll = state.scroll;
        let mut scroll = scroll + diff;

        let x_width = state.x_point_width(bounds.width);

        if scroll.x.abs() >= x_width {
            let steps = (scroll.x / x_width).trunc();
            state.x_scale.scroll(steps);

            scroll.x = scroll.x % x_width;
        }

        state.scroll = scroll;

        None
    }

    fn on_scroll_reset(
        &self,
        state: &mut Self::State,
        _bounds: iced::Rectangle,
        _cursor: iced::mouse::Cursor,
        _infinite_cursor: iced::mouse::Cursor,
        scroll: iced::Vector,
    ) -> Option<Message> {
        state.scroll = scroll;
        state.x_scale.reset();
        None
    }

    fn on_zoom(
        &self,
        state: &mut Self::State,
        _bounds: iced::Rectangle,
        _cursor: iced::mouse::Cursor,
        _infinite_cursor: iced::mouse::Cursor,
        _focal_point: Point,
        zoom: f32,
        diff: f32,
    ) -> Option<Message> {
        state.zoom_state.on_zoom(&mut state.x_scale, zoom, diff);

        None
    }

    fn on_zoom_reset(
        &self,
        state: &mut Self::State,
        _bounds: iced::Rectangle,
        _cursor: iced::mouse::Cursor,
        _infinite_cursor: iced::mouse::Cursor,
        _zoom: f32,
    ) -> Option<Message> {
        state.x_scale.reset();
        state.zoom_state = ZoomState::default();
        None
    }
}
