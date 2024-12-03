#![allow(dead_code, unused_imports)]
use iced::{
    application, color,
    widget::{
        canvas::{Path, Text},
        center,
    },
    Element, Length, Renderer, Theme,
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
    end: f32,
    step: f32,
}

#[allow(dead_code)]
impl Scale {
    /// Bounds should never be 0
    fn new(bounds: f32) -> Self {
        assert_ne!(bounds, 0.0);
        Self {
            start: -bounds,
            end: bounds,
            step: 1.,
        }
    }

    fn scroll(&mut self, scroll: f32) {
        let scroll = self.step * scroll;
        self.start += scroll;
        self.end += scroll;
    }

    fn start(mut self, start: f32) -> Self {
        self.start = start;
        self
    }

    fn end(mut self, end: f32) -> Self {
        self.end = end;
        self
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

        //self.start *= step;
        //self.end *= step;

        //let temp = self.start * step;
        //let is_negative = temp < 0.0;
        //let temp = temp.abs().max(1.0);
        //self.start = if is_negative { -temp } else { temp };
        //
        //let temp = self.end * step;
        //let is_negative = temp < 0.0;
        //let temp = temp.abs().max(1.0);
        //self.end = if is_negative { -temp } else { temp };
    }

    fn zoom_scale(&mut self, expand: bool) {
        if expand {
            self.start *= 2.0;
            self.end *= 2.0;
        } else {
            self.start /= 2.0;
            self.end /= 2.0;
        }
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

#[derive(Debug, Clone, Copy)]
struct GraphState {
    x_scale: Scale,
    kx: f32,
    scroll: iced::Vector,
    scale: f32,
    temp: f32,
    threshold: f32,
}

impl GraphState {
    fn range(&self) -> impl Iterator<Item = f32> {
        self.x_scale.into_iter()
    }

    fn x_point_width(&self, bounds_width: f32) -> f32 {
        let k = self.kx;
        bounds_width * 0.5 / k
    }

    // fn len(&self) -> u32 {
    //     (self.start.abs() + self.end.abs()) as u32
    // }
}

impl<Message> Program<Message, Theme, Renderer> for Graph {
    type State = GraphState;

    fn create_state(&self) -> Self::State {
        GraphState {
            x_scale: Scale::new(10.0),
            kx: 5.0,
            scroll: iced::Vector::new(0., 0.),
            scale: 1.0,
            temp: 0.0,
            threshold: 0.5,
        }
    }

    fn draw<'a>(
        &self,
        state: &Self::State,
        theme: &Theme,
        bounds: iced::Rectangle,
        _cursor: iced::mouse::Cursor,
        center: iced::Point,
    ) -> Vec<Buffer<'a>> {
        use iced::widget::canvas::Stroke;
        let color2 = color!(128, 0, 128);
        let color = color!(0, 128, 128);
        //let color1 = color!(102, 51, 153);

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
            let height = height / state.scale.max(0.01);
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

        vec![axis, x_points, dummies]
    }

    fn on_scroll(
        &self,
        state: &mut Self::State,
        bounds: iced::Rectangle,
        _cursor: iced::mouse::Cursor,
        diff: iced::Vector,
    ) -> Option<Message> {
        let mut scroll = state.scroll + diff;

        let x_width = state.x_point_width(bounds.width);

        if scroll.x.abs() >= x_width {
            let steps = (scroll.x / x_width).trunc();
            state.x_scale.scroll(steps);

            scroll.x = scroll.x % x_width;
        }

        state.scroll = scroll;
        None
    }

    fn on_zoom(
        &self,
        state: &mut Self::State,
        _bounds: iced::Rectangle,
        _cursor: iced::mouse::Cursor,
        diff: f32,
    ) -> Option<Message> {
        let zoom = state.temp + diff;
        state.temp = zoom;

        let threshold = if state.temp < 0.0 && diff > 0. || state.temp > 0. && diff < 0. {
            state.threshold / 2.5
        } else {
            state.threshold
        };

        if zoom.abs() >= threshold {
            state.x_scale.zoom(diff < 0.);
            if state.temp < 0.0 && diff > 0. || state.temp > 0. && diff < 0. {
                state.threshold /= 2.5;
            } else {
                state.threshold *= 2.5;
            }
            state.temp = zoom % state.threshold;

            //    if zoom < 0. {
            //        state.scale -= 0.5;
            //        state.x_scale.zoom(true);
            //        //state.x_scale.zoom_scale(true);
            //    } else {
            //        state.scale += 0.5;
            //        state.x_scale.zoom(false);
            //        //state.x_scale.zoom_scale(false);
            //    }
            //
        }

        //if zoom.abs() >= threshold.abs() {
        //    //dbg!(state.scale);
        //    //dbg!(threshold);
        //
        //    if threshold >= 0.0125 {
        //        state.scale = threshold;
        //        state.temp = state.scale;
        //        state.x_scale.zoom(diff);
        //        //state.x_scale.zoom_scale(diff);
        //    }
        //}

        None
    }
}
