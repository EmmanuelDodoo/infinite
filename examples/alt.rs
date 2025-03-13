#![allow(dead_code, unused_imports, unused_variables)]
use iced::{
    advanced::{
        self, layout, overlay, renderer,
        renderer::Quad,
        text::{LineHeight, Shaping, Wrapping},
        Clipboard, Shell,
    },
    alignment::{Horizontal, Vertical},
    application, color, event, mouse,
    widget::{
        canvas::{Path, Text},
        center,
    },
    Background, Border, Element, Event, Length, Point, Rectangle, Renderer, Size, Theme, Vector,
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

#[derive(Debug, Clone)]
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
struct ScaleCopy {
    start: f32,
    end: f32,
    step: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Scale {
    start: f32,
    end: f32,
    step: f32,
    original: ScaleCopy,
    state: ZoomState,
    k: f32,
}

impl Scale {
    const SCALE_FACTORS: [f32; 3] = [1.0, 2.0, 5.0];

    fn new(start: f32, end: f32, step: f32) -> Self {
        Self {
            start,
            end,
            step,
            k: 5.0,
            original: ScaleCopy { start, end, step },
            state: ZoomState::new(),
        }
    }

    fn reset_scroll(&mut self) {
        self.start = self.original.start;
        self.end = self.original.end;
    }

    fn reset_zoom(&mut self) {
        self.state.reset();

        let og_width = self.original.end - self.original.start;
        let center = self.start + (self.end - self.start) / 2.0;

        let new = og_width * self.original.step / 2.0;

        self.start = center - new;
        self.end = center + new;
        self.step = self.original.step;
    }

    fn scroll(&mut self, amount: f32) {
        let scroll = self.step * amount;
        self.start += scroll;
        self.end += scroll;
    }

    fn compute_zoom_scaling(zoom_level: f32) -> f32 {
        let step = 3.0;

        let exponent = (zoom_level / step).trunc();
        let sub_index = (zoom_level.abs() as usize % Self::SCALE_FACTORS.len()) as usize;

        let factor = Self::SCALE_FACTORS[sub_index];

        if zoom_level >= 0.0 {
            1.0 / (10.0f32.powf(exponent) * factor)
        } else {
            10.0f32.powf(-exponent) * factor
        }
    }

    fn zoom(&mut self, center: f32, expand: bool) {
        let count = self.state.zoom(expand);

        let factor = Self::compute_zoom_scaling(count);

        if self.step != factor {
            self.k *= 1.25;
            //self.scroll(-20.0);
            self.adjust_width(center, factor);
        }

        self.step = factor;
    }

    fn adjust_width(&mut self, center: f32, factor: f32) {
        let og_width = self.original.end - self.original.start;

        let new = og_width * factor / 2.0;

        self.start = center - new;
        self.end = center + new;
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Iter {
    current: f32,
    step: f32,
    end: f32,
}

impl Iter {
    fn new(scale: Scale) -> Self {
        let step = scale.step;
        let bounds = (scale.start, scale.end);
        let (first, last) = Self::generate_bounds(bounds, step);

        Self {
            current: first,
            end: last,
            step,
        }
    }

    fn generate_bounds(bounds: (f32, f32), step: f32) -> (f32, f32) {
        let (min_x, max_x) = (f32::min(bounds.0, bounds.1), f32::max(bounds.0, bounds.1));

        let first = (min_x / step).floor() * step;
        let last = (max_x / step).ceil() * step;

        (first, last)
    }
}

impl Iterator for Iter {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current > self.end {
            return None;
        }

        let out = self.current;
        self.current += self.step;

        Some(out)
    }
}

impl IntoIterator for Scale {
    type Item = f32;
    type IntoIter = Iter;

    fn into_iter(self) -> Self::IntoIter {
        Iter::new(self)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Pending {
    expand: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct GraphState {
    x: Scale,
    scroll: Vector,
    canvas_offset: Vector,
    scale: f32,
    flag: bool,
    pending: Option<Pending>,
}

impl GraphState {
    fn range(&self) -> Iter {
        self.x.into_iter()
    }

    fn x_width(&self, width: f32) -> f32 {
        width * 0.5 / self.x.k
    }

    fn zoom(&mut self, center_x: f32, expand: bool) {
        self.x.zoom(center_x, expand);
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
    count: f32,
    threshold: f32,
}

impl ZoomState {
    fn new() -> Self {
        Self {
            count: 0.0,
            threshold: 5.0,
        }
    }

    fn zoom(&mut self, expand: bool) -> f32 {
        if expand {
            self.count += 1.0;
        } else {
            self.count -= 1.0;
        }

        self.count / self.threshold
    }

    fn reset(&mut self) {
        self.count = 0.0;
    }

    //fn on_zoom(&mut self, x_scale: &mut Scale, diff: f32) {
    //    let diff = (diff * 10.0) as i16;
    //
    //    self.count += diff;
    //
    //    let is_zoom_in = diff > 0;
    //
    //    let _kx_delta = 1.25;
    //
    //    match self.kind {
    //        ZoomKind::None => {
    //            if self.count.abs() >= self.threshold {
    //                self.count %= self.threshold;
    //                x_scale.zoom(!is_zoom_in);
    //
    //                if is_zoom_in {
    //                    self.kind = ZoomKind::ZoomedIn(1);
    //                } else {
    //                    self.kind = ZoomKind::ZoomedOut(1);
    //                }
    //            }
    //        }
    //        ZoomKind::ZoomedIn(amt) => {
    //            let threshold = self.threshold;
    //
    //            if is_zoom_in && self.count >= threshold {
    //                self.threshold = threshold;
    //                self.count %= self.threshold;
    //                x_scale.zoom(!is_zoom_in);
    //                //self.kx = self.kx / kx_delta;
    //
    //                self.kind = ZoomKind::ZoomedIn(amt + 1);
    //            } else if !is_zoom_in && self.count < 0 {
    //                x_scale.zoom(!is_zoom_in);
    //
    //                let amt = amt - 1;
    //
    //                //self.kx = (self.kx * kx_delta).max(self.og_kx);
    //                if amt == 0 {
    //                    self.count = threshold + self.count;
    //                    self.kind = ZoomKind::None;
    //                } else {
    //                    self.count = threshold + self.count;
    //                    self.threshold = threshold;
    //                    self.kind = ZoomKind::ZoomedIn(amt);
    //                }
    //            }
    //        }
    //        ZoomKind::ZoomedOut(amt) => {
    //            let threshold = self.threshold;
    //
    //            if !is_zoom_in && self.count <= -threshold {
    //                self.threshold = threshold;
    //                self.count %= self.threshold;
    //                x_scale.zoom(!is_zoom_in);
    //                //self.kx = self.kx * kx_delta;
    //
    //                self.kind = ZoomKind::ZoomedOut(amt + 1)
    //            } else if is_zoom_in && self.count > 0 {
    //                x_scale.zoom(!is_zoom_in);
    //
    //                let amt = amt - 1;
    //
    //                //self.kx = (self.kx / kx_delta).max(self.og_kx);
    //                if amt == 0 {
    //                    self.count = -threshold + self.count;
    //                    self.kind = ZoomKind::None;
    //                } else {
    //                    self.count = -threshold + self.count;
    //                    self.threshold = threshold;
    //                    self.kind = ZoomKind::ZoomedOut(amt);
    //                }
    //            }
    //        }
    //    }
    //}
}

struct Graph;

impl Program<Message, Theme, Renderer> for Graph {
    type State = GraphState;

    fn init_state(&self) -> Self::State {
        GraphState {
            x: Scale::new(-15.0, 15.0, 1.0),
            scroll: Vector::new(0., 0.),
            scale: 1.0,
            canvas_offset: Vector::ZERO,
            flag: false,
            pending: None,
        }
    }

    fn init_zoom(&self) -> f32 {
        0.0
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

            let path = Path::circle((150., 150.).into(), 15.0.into());
            buffer.fill(path, color1);

            let path = Path::circle((150., -150.).into(), 15.0.into());
            buffer.fill(path, color1);

            let path = Path::circle((-150., 150.).into(), 15.0.into());
            buffer.fill(path, color1);

            let path = Path::circle((-150., -150.).into(), 15.0.into());
            buffer.fill(path, color1);

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

        let points = {
            let mut buffer = Buffer::new();

            let width = state.x_width(bounds.width);
            //let width = 100.;
            let outlines_num = 1.0;
            let height = bounds.height;
            let height = height / state.scale.max(0.01);
            let pad = 18.0;

            for point in state.range() {
                let x = width * point;
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

                let width = width / outlines_num;
                //let width = width / 5.0;

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

        vec![axis, dummies, points]
    }

    fn on_scroll(
        &self,
        state: &mut Self::State,
        bounds: iced::Rectangle,
        _cursor: iced::mouse::Cursor,
        _infinite_cursor: iced::mouse::Cursor,
        scroll: Vector,
        diff: Vector,
    ) -> Option<Message> {
        state.canvas_offset = scroll;
        if state.flag {
            state.flag = false;
            return None;
        }

        let mut scroll = state.scroll + (diff * state.scale);
        let x_width = state.x_width(bounds.width);

        if scroll.x.abs() >= x_width {
            let steps = (scroll.x / x_width).trunc();
            state.x.scroll(steps);

            scroll.x = scroll.x % x_width;
        }

        state.scroll = scroll;

        if let Some(Pending { .. }) = state.pending.take() {
            let center_x = state.canvas_offset.x / (x_width * state.scale);
            state.x.adjust_width(center_x, state.x.step);
        }

        None
    }

    fn on_scroll_reset(
        &self,
        state: &mut Self::State,
        _bounds: iced::Rectangle,
        _cursor: iced::mouse::Cursor,
        _infinite_cursor: iced::mouse::Cursor,
        scroll: Vector,
    ) -> Option<Message> {
        state.scroll = scroll;
        state.canvas_offset = scroll;
        state.x.reset_scroll();

        None
    }

    fn on_zoom(
        &self,
        state: &mut Self::State,
        bounds: iced::Rectangle,
        _cursor: iced::mouse::Cursor,
        infinite_cursor: iced::mouse::Cursor,
        focal_point: Point,
        zoom: f32,
        diff: f32,
    ) -> Option<Message> {
        let is_origin_zoom = focal_point == Point::ORIGIN;

        state.flag = !is_origin_zoom;
        state.scale = zoom;

        let x_width = state.x_width(bounds.width);
        let refr = if is_origin_zoom {
            let temp = state.canvas_offset * (1.0 / zoom);
            (temp.x, temp.y)
        } else {
            let temp = infinite_cursor.position().unwrap_or_default();

            (temp.x, temp.y)
        };

        let center_x = (refr.0 / x_width).round();

        state.zoom(center_x, diff > 0.0);

        if is_origin_zoom {
            state.pending = Some(Pending { expand: diff > 0.0 });
        }

        None
    }

    fn on_zoom_reset(
        &self,
        state: &mut Self::State,
        _bounds: iced::Rectangle,
        _cursor: iced::mouse::Cursor,
        _infinite_cursor: iced::mouse::Cursor,
        zoom: f32,
    ) -> Option<Message> {
        state.scale = zoom;
        state.flag = false;
        state.x.reset_zoom();
        None
    }

    fn overlay<'a>(
        &self,
        state: &'a mut Self::State,
        bounds: iced::Rectangle,
        _infinite_cursor: Point,
        translation: Vector,
    ) -> Option<iced::advanced::overlay::Element<'a, Message, Theme, Renderer>> {
        let width = 150.0;
        let translation = {
            let other = Vector::new(bounds.width - width, 0.0);
            other + translation
        };

        let position = bounds.position() + translation;

        let overlay = Overlay::new(state, position, width);
        let overlay = overlay::Element::new(Box::new(overlay));

        Some(overlay)
    }
}

fn _round_down_to_power_of_ten(value: f32) -> f32 {
    let power = value.abs().log10().floor();
    let base = 10f32.powf(power);
    (value / base).floor() * base
}

struct Overlay<'a> {
    position: Point,
    height: f32,
    width: f32,
    state: &'a mut GraphState,
}

impl<'a> Overlay<'a> {
    pub fn new(state: &'a mut GraphState, position: Point, width: f32) -> Self {
        Self {
            width,
            height: 30.0,
            position,
            state,
        }
    }
}

impl<'a, Message> overlay::Overlay<Message, Theme, Renderer> for Overlay<'a>
where
    Message: Clone + 'a,
{
    fn on_event(
        &mut self,
        _event: Event,
        layout: layout::Layout<'_>,
        cursor: iced::mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        _shell: &mut Shell<'_, Message>,
    ) -> event::Status {
        let bounds = layout.bounds();

        if !cursor.is_over(bounds) {
            return event::Status::Ignored;
        }

        event::Status::Ignored
    }

    fn mouse_interaction(
        &self,
        _layout: layout::Layout<'_>,
        _cursor: iced::mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> iced::mouse::Interaction {
        iced::mouse::Interaction::Pointer
    }

    fn layout(&mut self, _renderer: &Renderer, _bounds: Size) -> layout::Node {
        let size = Size::new(self.width, self.height);

        let node = layout::Node::new(size);

        node.translate(Vector::new(self.position.x, self.position.y))
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        theme: &Theme,
        _style: &renderer::Style,
        layout: layout::Layout<'_>,
        _cursor: mouse::Cursor,
    ) {
        let bounds = layout.bounds();
        let palette = theme.extended_palette();

        // todo
        let pair = if true {
            palette.primary.weak
        } else {
            palette.primary.strong
        };

        let background = pair.color;

        let border = Border::default().width(0.0);

        <Renderer as advanced::Renderer>::fill_quad(
            renderer,
            Quad {
                bounds,
                border,
                ..Default::default()
            },
            Background::Color(background),
        );

        let color = pair.text;

        let font = <Renderer as advanced::text::Renderer>::default_font(renderer);

        let icon = advanced::text::Text {
            content: "Legend".to_string(),
            size: 18.0.into(),
            bounds: bounds.size(),
            font,
            horizontal_alignment: Horizontal::Center,
            vertical_alignment: Vertical::Center,
            line_height: LineHeight::default(),
            shaping: Shaping::Basic,
            wrapping: Wrapping::None,
        };

        <Renderer as advanced::text::Renderer>::fill_text(
            renderer,
            icon,
            bounds.center(),
            color,
            bounds,
        )
    }
}
