#![allow(dead_code, unused_imports)]
use iced::{
    application, color,
    widget::{
        canvas::{Path, Text},
        center, text,
    },
    Element, Length, Renderer, Theme,
};

use canavs::*;

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

struct Graph;

impl<Message> Program<Message, Theme, Renderer> for Graph {
    type State = ();

    fn draw<'a>(
        &self,
        _state: &Self::State,
        _theme: &Theme,
        _bounds: iced::Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> Vec<Buffer<'a>> {
        let color2 = color!(128, 0, 128);
        let color = color!(0, 128, 128);

        let mut buffer = Buffer::new();

        buffer.draw_text(
            Text {
                content: "Testing Infinite".into(),
                position: (15., 45.).into(),
                size: 20.0.into(),
                ..Default::default()
            },
            Anchor::None,
        );

        {
            let path = Path::circle((15., 45.).into(), 5.0.into());
            buffer.fill(path, color2, Anchor::None);
        }

        buffer.fill_rounded_rect((120.0, 120.), (150., 100.), 10., color, Anchor::None);

        vec![buffer]
    }
}

mod canavs {
    use std::marker::PhantomData;

    use iced::{
        advanced::{self, layout, widget::tree, Widget},
        border::Radius,
        color, event as iced_event, keyboard, mouse,
        widget::canvas::{
            path::lyon_path::geom::euclid::Transform2D, Fill, Frame, Path, Stroke, Text,
        },
        Background, Border, Color, Element, Length, Point, Rectangle, Shadow, Size, Theme, Vector,
    };

    use iced_graphics::geometry;

    const DEFAULT_BACKGROUND: Background = Background::Color(color!(203, 213, 240));

    pub mod event {
        #[derive(Debug, Default, Clone, Copy, PartialEq)]
        pub enum Status {
            Captured,
            #[default]
            Ignored,
        }

        impl Status {
            pub fn merge(self, other: Self) -> Self {
                match (self, other) {
                    (Status::Captured, _) => Status::Captured,
                    (_, Status::Captured) => Status::Captured,
                    _ => Status::Ignored,
                }
            }
        }

        impl From<Status> for iced::event::Status {
            fn from(value: Status) -> Self {
                match value {
                    Status::Captured => iced::event::Status::Captured,
                    Status::Ignored => iced::event::Status::Captured,
                }
            }
        }

        #[derive(Debug, Clone, PartialEq)]
        pub enum Event {
            Mouse(iced::mouse::Event),
            Keyboard(iced::keyboard::Event),
            Touch(iced::touch::Event),
        }

        impl From<Event> for iced::Event {
            fn from(value: Event) -> Self {
                match value {
                    Event::Mouse(event) => iced::Event::Mouse(event),
                    Event::Touch(event) => iced::Event::Touch(event),
                    Event::Keyboard(event) => iced::Event::Keyboard(event),
                }
            }
        }
    }

    pub trait Program<Message, Theme = iced::Theme, Renderer = iced::Renderer>
    where
        Renderer: iced_graphics::geometry::Renderer,
    {
        type State: Default + 'static;

        fn draw<'a>(
            &self,
            state: &Self::State,
            theme: &Theme,
            bounds: Rectangle,
            cursor: mouse::Cursor,
        ) -> Vec<Buffer<'a>>;

        fn update(
            &self,
            _state: &mut Self::State,
            _event: event::Event,
            _bounds: Rectangle,
            _cursor: mouse::Cursor,
        ) -> (event::Status, Option<Message>) {
            (event::Status::Ignored, None)
        }

        fn mouse_interaction(
            &self,
            _state: &Self::State,
            _bounds: Rectangle,
            _cursor: mouse::Cursor,
        ) -> mouse::Interaction {
            mouse::Interaction::default()
        }
    }

    /// Determines the degree by which points on the canvas are fixed
    #[derive(Debug, Default, Clone, Copy, PartialEq)]
    pub enum Anchor {
        /// Both x and y coordinates are fixed and do not move in any direction
        Both,
        /// The x coordinate is fixed while the y coordinate can
        /// freely move
        X,
        /// The y coordinate  is fixed while the x coordinate can
        /// freely move
        Y,
        /// Both x and y coordinates are not anchored and are free to move in
        /// any direction
        #[default]
        None,
    }

    #[derive(Debug, Clone)]
    pub struct Buffer<'a> {
        fills: Vec<(Path, Fill, Anchor)>,
        strokes: Vec<(Path, Stroke<'a>, Anchor)>,
        text: Vec<(Text, Anchor)>,
        anchor: Option<Anchor>,
    }

    impl<'a> Default for Buffer<'a> {
        fn default() -> Self {
            Self::new()
        }
    }

    impl<'a> Buffer<'a> {
        pub fn new() -> Self {
            Self {
                fills: Vec::new(),
                strokes: Vec::new(),
                text: Vec::new(),
                anchor: None,
            }
        }

        /// Creates a [`Buffer`] with all canvas items anchored
        pub fn anchor_all(mut self, anchor: Anchor) -> Self {
            self.anchor = Some(anchor);
            self
        }

        pub fn draw_stroke(&mut self, path: Path, stroke: impl Into<Stroke<'a>>, anchor: Anchor) {
            self.strokes.push((path, stroke.into(), anchor))
        }

        pub fn draw_fill(&mut self, path: Path, fill: impl Into<Fill>, anchor: Anchor) {
            self.fills.push((path, fill.into(), anchor))
        }

        pub fn draw_text(&mut self, text: impl Into<Text>, anchor: Anchor) {
            self.text.push((text.into(), anchor))
        }

        pub fn fill(&mut self, path: Path, fill: impl Into<Fill>, anchor: Anchor) {
            self.fills.push((path, fill.into(), anchor))
        }

        pub fn stroke(&mut self, path: Path, stroke: impl Into<Stroke<'a>>, anchor: Anchor) {
            self.strokes.push((path, stroke.into(), anchor))
        }

        pub fn fill_rect(
            &mut self,
            top_left: impl Into<Point>,
            size: impl Into<Size>,
            fill: impl Into<Fill>,
            anchor: Anchor,
        ) {
            let size: Size = size.into();
            let point = top_left.into();

            //let bottom_left = point - Into::<Vector>::into(size);
            let bottom_left = point - Vector::new(0., size.height);

            let path = Path::rectangle(bottom_left, size);

            self.fill(path, fill, anchor)
        }

        pub fn fill_rounded_rect(
            &mut self,
            top_left: impl Into<Point>,
            size: impl Into<Size>,
            radius: impl Into<Radius>,
            fill: impl Into<Fill>,
            anchor: Anchor,
        ) {
            let size: Size = size.into();
            let point = top_left.into();

            let top_left = point - Vector::new(0., size.height);

            let path = Path::rounded_rectangle(top_left, size, radius.into());

            self.fill(path, fill, anchor);
        }

        pub fn stroke_rect(
            &mut self,
            top_left: impl Into<Point>,
            size: impl Into<Size>,
            stroke: impl Into<Stroke<'a>>,
            anchor: Anchor,
        ) {
            let size: Size = size.into();
            let point = top_left.into();

            //let bottom_left = point - Into::<Vector>::into(size);
            let bottom_left = point - Vector::new(0., size.height);

            let path = Path::rectangle(bottom_left, size);

            self.stroke(path, stroke, anchor)
        }

        pub fn stroke_rounded_rect(
            &mut self,
            top_left: impl Into<Point>,
            size: impl Into<Size>,
            radius: impl Into<Radius>,
            stroke: impl Into<Stroke<'a>>,
            anchor: Anchor,
        ) {
            let size: Size = size.into();
            let point = top_left.into();

            let top_left = point - Vector::new(0., size.height);

            let path = Path::rounded_rectangle(top_left, size, radius.into());

            self.stroke(path, stroke, anchor);
        }

        fn draw_fills<State, Renderer: geometry::Renderer>(
            &self,
            frame: &mut Frame<Renderer>,
            state: &InfiniteState<State>,
            center: Point,
        ) {
            self.fills
                .iter()
                .map(|(path, fill, anchor)| {
                    let path = transform_path(state, center, path, self.anchor.unwrap_or(*anchor));
                    (path, *fill)
                })
                .for_each(|(path, fill)| frame.fill(&path, fill));
        }

        fn draw_strokes<State, Renderer: geometry::Renderer>(
            &self,
            frame: &mut Frame<Renderer>,
            state: &InfiniteState<State>,
            center: Point,
        ) {
            self.strokes
                .iter()
                .map(|(path, stroke, anchor)| {
                    let path = transform_path(state, center, path, self.anchor.unwrap_or(*anchor));
                    (path, *stroke)
                })
                .for_each(|(path, stroke)| frame.stroke(&path, stroke));
        }

        fn draw_texts<State, Renderer: geometry::Renderer>(
            &self,
            frame: &mut Frame<Renderer>,
            state: &InfiniteState<State>,
            center: Point,
        ) {
            self.text
                .iter()
                .map(|(text, anchor)| {
                    transform_text(state, center, text, self.anchor.unwrap_or(*anchor))
                })
                .for_each(|text| frame.fill_text(text));
        }

        fn draw<State, Renderer: geometry::Renderer>(
            &self,
            frame: &mut Frame<Renderer>,
            state: &InfiniteState<State>,
            center: Point,
        ) {
            self.draw_fills(frame, state, center);
            self.draw_strokes(frame, state, center);
            self.draw_texts(frame, state, center);
        }
    }

    /// Determines which directions the canvas can be scrolled
    #[derive(Debug, Default, Clone, Copy, PartialEq)]
    pub enum ScrollDirection {
        /// Scroll in only X direction
        X,
        /// Scroll in only the Y direction
        Y,
        #[default]
        /// Scroll in both x and y directions
        Both,
    }

    pub struct Infinite<'a, P, Message, Theme = iced::Theme, Renderer = iced::Renderer>
    where
        Theme: Catalog,
        P: Program<Message, Theme, Renderer>,
        Renderer: geometry::Renderer,
    {
        width: Length,
        height: Length,
        direction: ScrollDirection,
        _message: PhantomData<Message>,
        _renderer: PhantomData<Renderer>,
        program: P,
        style: <Theme as Catalog>::Class<'a>,
    }

    impl<'a, P, Message, Theme, Renderer> Infinite<'a, P, Message, Theme, Renderer>
    where
        Theme: Catalog,
        P: Program<Message, Theme, Renderer>,
        Renderer: geometry::Renderer,
    {
        const DEFAULT_SIZE: f32 = 300.0;

        pub fn new(program: P) -> Self {
            Self {
                width: Length::Fixed(Self::DEFAULT_SIZE),
                height: Length::Fixed(Self::DEFAULT_SIZE),
                direction: ScrollDirection::default(),
                program,
                _message: PhantomData::default(),
                _renderer: PhantomData::default(),
                style: Theme::default(),
            }
        }

        pub fn height(mut self, height: impl Into<Length>) -> Self {
            self.height = height.into();
            self
        }

        pub fn width(mut self, width: impl Into<Length>) -> Self {
            self.width = width.into();
            self
        }

        pub fn scroll_direction(mut self, direction: ScrollDirection) -> Self {
            self.direction = direction;
            self
        }

        pub fn style(mut self, style: impl Fn(&Theme, InfiniteStatus) -> InfiniteStyle + 'a) -> Self
        where
            Theme::Class<'a>: From<StyleFn<'a, Theme>>,
        {
            self.style = (Box::new(style) as StyleFn<'a, Theme>).into();
            self
        }
    }

    impl<'a, P, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
        for Infinite<'a, P, Message, Theme, Renderer>
    where
        Theme: Catalog,
        P: Program<Message, Theme, Renderer>,
        Renderer: geometry::Renderer,
    {
        fn size(&self) -> Size<Length> {
            Size {
                width: self.width,
                height: self.height,
            }
        }

        fn tag(&self) -> tree::Tag {
            tree::Tag::of::<InfiniteState<P::State>>()
        }

        fn state(&self) -> tree::State {
            tree::State::new(InfiniteState::<P::State>::new())
        }

        fn on_event(
            &mut self,
            state: &mut tree::Tree,
            event: iced::Event,
            layout: layout::Layout<'_>,
            cursor: advanced::mouse::Cursor,
            _renderer: &Renderer,
            _clipboard: &mut dyn advanced::Clipboard,
            shell: &mut advanced::Shell<'_, Message>,
            _viewport: &Rectangle,
        ) -> iced_event::Status {
            let bounds = layout.bounds();

            let canvas_event = match event.clone() {
                iced::Event::Mouse(event) => Some(event::Event::Mouse(event)),
                iced::Event::Keyboard(event) => Some(event::Event::Keyboard(event)),
                iced::Event::Touch(event) => Some(event::Event::Touch(event)),
                _ => None,
            };

            if let Some(canvas_event) = canvas_event {
                let state = &mut state.state.downcast_mut::<InfiniteState<P::State>>().state;

                let (status, message) = self.program.update(state, canvas_event, bounds, cursor);

                if let Some(message) = message {
                    shell.publish(message);
                }

                if status == event::Status::Captured {
                    return status.into();
                }
            }

            if !cursor.is_over(bounds) {
                return iced_event::Status::Ignored;
            }

            match event {
                iced::Event::Mouse(mouse::Event::WheelScrolled { delta }) => {
                    let state = state.state.downcast_mut::<InfiniteState<P::State>>();

                    match delta {
                        // Zoom
                        mouse::ScrollDelta::Lines { y, .. } if state.keyboard_modifier.shift() => {
                            state.scale += y;

                            iced_event::Status::Captured
                        }
                        mouse::ScrollDelta::Pixels { y, .. } if state.keyboard_modifier.shift() => {
                            state.scale += y;
                            iced_event::Status::Captured
                        }

                        // Translation
                        mouse::ScrollDelta::Pixels { x, y } => {
                            let offset = match self.direction {
                                ScrollDirection::X => Vector::new(x, 0.),
                                ScrollDirection::Y => Vector::new(0., y),
                                ScrollDirection::Both => Vector::new(x, y),
                            };

                            state.offset = state.offset - offset;

                            iced_event::Status::Captured
                        }
                        mouse::ScrollDelta::Lines { x, y } => {
                            let mult = 100.0;
                            let offset = match self.direction {
                                ScrollDirection::X => Vector::new(x, 0.),
                                ScrollDirection::Y => Vector::new(0., y),
                                ScrollDirection::Both => Vector::new(x, y),
                            } * mult;

                            state.offset = state.offset - offset;

                            iced_event::Status::Captured
                        }
                    }
                }
                iced::Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. }) => {
                    let state = state.state.downcast_mut::<InfiniteState<P::State>>();
                    let translation = 25.0;
                    let zoom = 0.1;
                    match key {
                        // Translations
                        keyboard::Key::Named(keyboard::key::Named::ArrowUp)
                            if modifiers.command() =>
                        {
                            let offset = match self.direction {
                                ScrollDirection::X => Vector::new(0., 0.),
                                ScrollDirection::Y => Vector::new(0., translation),
                                ScrollDirection::Both => Vector::new(0., translation),
                            };

                            state.offset = state.offset - offset;

                            iced_event::Status::Captured
                        }

                        keyboard::Key::Named(keyboard::key::Named::ArrowDown)
                            if modifiers.command() =>
                        {
                            let offset = match self.direction {
                                ScrollDirection::X => Vector::new(0., 0.),
                                ScrollDirection::Y => Vector::new(0., translation),
                                ScrollDirection::Both => Vector::new(0., translation),
                            };
                            state.offset = state.offset + offset;

                            iced_event::Status::Captured
                        }

                        keyboard::Key::Named(keyboard::key::Named::ArrowLeft)
                            if modifiers.command() =>
                        {
                            let offset = match self.direction {
                                ScrollDirection::X => Vector::new(translation, 0.),
                                ScrollDirection::Y => Vector::new(0., 0.),
                                ScrollDirection::Both => Vector::new(translation, 0.),
                            };
                            state.offset = state.offset - offset;

                            iced_event::Status::Captured
                        }
                        keyboard::Key::Named(keyboard::key::Named::ArrowRight)
                            if modifiers.command() =>
                        {
                            let offset = match self.direction {
                                ScrollDirection::X => Vector::new(translation, 0.),
                                ScrollDirection::Y => Vector::new(0., 0.),
                                ScrollDirection::Both => Vector::new(translation, 0.),
                            };
                            state.offset = state.offset + offset;

                            iced_event::Status::Captured
                        }

                        // Zoom
                        keyboard::Key::Named(keyboard::key::Named::ArrowUp)
                            if modifiers.shift() =>
                        {
                            state.scale += zoom;

                            iced_event::Status::Captured
                        }

                        keyboard::Key::Named(keyboard::key::Named::ArrowDown)
                            if modifiers.shift() =>
                        {
                            state.scale -= zoom;

                            iced_event::Status::Captured
                        }

                        // Resets
                        keyboard::Key::Named(keyboard::key::Named::Home) if modifiers.command() => {
                            state.reset_all();
                            iced_event::Status::Captured
                        }

                        keyboard::Key::Named(keyboard::key::Named::Home) if modifiers.shift() => {
                            state.reset_scale();
                            iced_event::Status::Captured
                        }

                        keyboard::Key::Named(keyboard::key::Named::Home) => {
                            state.reset_offset();
                            iced_event::Status::Captured
                        }

                        _ => iced_event::Status::Ignored,
                    }
                }

                iced::Event::Keyboard(keyboard::Event::ModifiersChanged(modifiers)) => {
                    let state = state.state.downcast_mut::<InfiniteState<P::State>>();
                    state.keyboard_modifier = modifiers;

                    iced_event::Status::Captured
                }

                _ => iced_event::Status::Ignored,
            }
        }

        fn mouse_interaction(
            &self,
            state: &tree::Tree,
            layout: layout::Layout<'_>,
            cursor: advanced::mouse::Cursor,
            _viewport: &Rectangle,
            _renderer: &Renderer,
        ) -> advanced::mouse::Interaction {
            let bounds = layout.bounds();
            let state = &state.state.downcast_ref::<InfiniteState<P::State>>().state;

            self.program.mouse_interaction(&state, bounds, cursor)
        }

        fn layout(
            &self,
            _tree: &mut iced::advanced::widget::Tree,
            _renderer: &Renderer,
            limits: &iced::advanced::layout::Limits,
        ) -> layout::Node {
            layout::atomic(limits, self.width, self.height)
        }

        fn draw(
            &self,
            tree: &iced::advanced::widget::Tree,
            renderer: &mut Renderer,
            theme: &Theme,
            _style: &iced::advanced::renderer::Style,
            layout: layout::Layout<'_>,
            cursor: iced::advanced::mouse::Cursor,
            _viewport: &iced::Rectangle,
        ) {
            let bounds = layout.bounds();
            let is_mouse_over = cursor.is_over(bounds);

            if bounds.width < 1.0 || bounds.height < 1.0 {
                return;
            }

            let status = if is_mouse_over {
                InfiniteStatus::Hovered
            } else {
                InfiniteStatus::Active
            };

            let style = theme.style(&self.style, status);

            let state = tree.state.downcast_ref::<InfiniteState<P::State>>();

            renderer.fill_quad(
                advanced::renderer::Quad {
                    bounds,
                    border: style.border,
                    shadow: style.shadow,
                },
                style.background,
            );

            let position = bounds.position();

            renderer.with_translation(Vector::new(position.x, position.y), |renderer| {
                let mut frame = Frame::new(renderer, bounds.size());
                let center = frame.center();

                let buffers = self.program.draw(&state.state, theme, bounds, cursor);

                for buffer in buffers {
                    buffer.draw(&mut frame, state, center);
                }

                if state.scale != 1.0 {
                    let pos = (bounds.width * 0.9, bounds.height * 0.95).into();
                    let background = style.details_background;
                    let radius = style.details_border_radius;
                    let color = style.details_text;

                    let scale = state.scale * 100.;
                    let digs = digits(scale.abs() as u32) * 11;
                    let neg = if scale < 0. { 5. } else { 0. };
                    let digits = neg + (digs as f32) + 10.;

                    let padding = 12.5;

                    let rect =
                        Path::rounded_rectangle(pos, (digits + 2. * padding, 30.).into(), radius);

                    frame.fill(&rect, background);

                    let text = Text {
                        content: format!("{:.0}%", scale),
                        position: (pos.x + padding, pos.y + 5.).into(),
                        color,
                        ..Default::default()
                    };

                    frame.fill_text(text);
                }

                if state.offset != Vector::new(0., 0.) {
                    let pos = (bounds.width * 0.01, bounds.height * 0.95).into();
                    let background = style.details_background;
                    let radius = style.details_border_radius;
                    let color = style.details_text;

                    let x = state.offset.x;
                    let y = state.offset.y * -1.;

                    // 16: x: y:
                    // each digit: 9
                    // point: 3
                    // - : 5
                    // , : 9
                    // total:
                    // 16 + (x_num + x_neg) + 12 + 9 + 16 + (y_num + y_neg) + 12

                    let x_num = digits(x.abs() as u32) * 9;
                    let x_neg = if x < 0. { 5. } else { 0. };
                    let y_num = digits(y.abs() as u32) * 9;
                    let y_neg = if y < 0. { 5. } else { 0. };

                    let digits = 16.
                        + (x_num as f32)
                        + x_neg
                        + 12.
                        + 9.
                        + 16.
                        + (y_num as f32)
                        + y_neg
                        + 12.;
                    let padding = 12.5;

                    let rect =
                        Path::rounded_rectangle(pos, (digits + 2. * padding, 30.).into(), radius);

                    frame.fill(&rect, background);

                    let text = Text {
                        content: format!("x: {x:.1}, y: {y:.1}",),
                        position: (pos.x + padding, pos.y + 5.).into(),
                        color,
                        ..Default::default()
                    };

                    frame.fill_text(text);
                }

                let center = center - state.offset;

                let color1 = color!(128, 0, 128);
                //let color = color!(0, 128, 128);

                let x_axis = {
                    let y = center.y;
                    Path::line((0., y).into(), (bounds.width, y).into())
                };

                frame.stroke(
                    &x_axis,
                    Stroke::default().with_color(color1).with_width(2.0),
                );

                let y_axis = {
                    let x = center.x;
                    Path::line((x, 0.).into(), (x, bounds.height).into())
                };

                frame.stroke(
                    &y_axis,
                    Stroke::default().with_color(color1).with_width(2.0),
                );

                let geoms = frame.into_geometry();

                renderer.draw_geometry(geoms);
            });
        }
    }

    impl<'a, P, Message, Theme, Renderer> From<Infinite<'a, P, Message, Theme, Renderer>>
        for Element<'a, Message, Theme, Renderer>
    where
        Message: 'a,
        Theme: Catalog + 'a,
        P: Program<Message, Theme, Renderer> + 'a,
        Renderer: geometry::Renderer + 'a,
    {
        fn from(value: Infinite<'a, P, Message, Theme, Renderer>) -> Self {
            Element::new(value)
        }
    }

    struct InfiniteState<State> {
        offset: Vector,
        scale: f32,
        keyboard_modifier: keyboard::Modifiers,
        state: State,
    }

    impl<State> InfiniteState<State>
    where
        State: Default,
    {
        fn new() -> Self {
            Self {
                offset: Vector::new(0., 0.),
                scale: 1.0,
                state: State::default(),
                keyboard_modifier: keyboard::Modifiers::default(),
            }
        }

        fn reset_all(&mut self) {
            self.reset_offset();
            self.reset_scale();
        }

        fn reset_offset(&mut self) {
            self.offset = Vector::new(0., 0.)
        }

        fn reset_scale(&mut self) {
            self.scale = 1.0;
        }
    }

    impl<State> Default for InfiniteState<State>
    where
        State: Default,
    {
        fn default() -> Self {
            Self::new()
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct InfiniteStyle {
        border: Border,
        background: Background,
        shadow: Shadow,
        details_border_radius: Radius,
        details_background: Color,
        details_text: Color,
    }

    #[derive(Debug, Clone, Copy, Default, PartialEq)]
    pub enum InfiniteStatus {
        #[default]
        Active,
        Hovered,
    }

    pub trait Catalog {
        /// The item class of the [`Catalog`].
        type Class<'a>;

        /// The default class produced by the [`Catalog`].
        fn default<'a>() -> Self::Class<'a>;

        /// The [`Style`] of a class with the given status.
        fn style(&self, class: &Self::Class<'_>, status: InfiniteStatus) -> InfiniteStyle;
    }

    /// A styling function for an [`Infinite`].
    pub type StyleFn<'a, Theme> = Box<dyn Fn(&Theme, InfiniteStatus) -> InfiniteStyle + 'a>;

    impl Catalog for Theme {
        type Class<'a> = StyleFn<'a, Self>;

        fn default<'a>() -> Self::Class<'a> {
            Box::new(default)
        }

        fn style(&self, class: &Self::Class<'_>, status: InfiniteStatus) -> InfiniteStyle {
            class(self, status)
        }
    }

    pub fn default(theme: &Theme, status: InfiniteStatus) -> InfiniteStyle {
        let palette = theme.extended_palette();
        let shadow = Shadow::default();
        let border_width = 2.5;

        let background = palette.background.base;
        let details_background = Color {
            a: 0.9,
            ..background.color
        };
        let details_text = background.text;

        let border = match status {
            InfiniteStatus::Active => Border::default()
                .width(border_width)
                .color(palette.background.base.color),
            InfiniteStatus::Hovered => Border::default()
                .width(border_width)
                .color(palette.primary.strong.color),
        };

        InfiniteStyle {
            shadow,
            border,
            background: DEFAULT_BACKGROUND,
            details_background,
            details_border_radius: 5.into(),
            details_text,
        }
    }

    fn digits(num: u32) -> u32 {
        if num == 0 {
            return 1;
        }

        let mut output = 0;
        let mut num = num;

        while num > 0 {
            output += 1;
            num /= 10;
        }

        return output;
    }

    fn transform_path<State>(
        state: &InfiniteState<State>,
        center: Point,
        path: &Path,
        anchor: Anchor,
    ) -> Path {
        let offset = match anchor {
            Anchor::None => state.offset,
            Anchor::X => Vector::new(0., state.offset.y),
            Anchor::Y => Vector::new(state.offset.x, 0.),
            Anchor::Both => Vector::new(0., 0.),
        };
        let center = center - offset;
        let trans_x = center.x;
        let trans_y = center.y;
        let scale = state.scale;

        let transform = Transform2D::new(scale, 0.0, 0.0, -scale, trans_x, trans_y);

        path.transform(&transform)
    }

    fn translate_point<State>(
        state: &InfiniteState<State>,
        center: Point,
        point: impl Into<Point>,
        anchor: Anchor,
    ) -> Point {
        let offset = match anchor {
            Anchor::Both => Vector::new(0., 0.),
            Anchor::X => Vector::new(0., state.offset.y),
            Anchor::Y => Vector::new(state.offset.x, 0.),
            Anchor::None => state.offset,
        };
        let center = center - offset;
        let point = {
            let point: Point = point.into();
            Point::new(point.x * state.scale, point.y * state.scale)
        };
        let x = center.x + point.x;
        let y = center.y - point.y;

        Point::new(x, y)
    }

    fn transform_text<State>(
        state: &InfiniteState<State>,
        center: Point,
        text: &Text,
        anchor: Anchor,
    ) -> Text {
        let position = translate_point(state, center, text.position, anchor);

        Text {
            content: text.content.clone(),
            position,
            size: text.size,
            color: text.color,
            font: text.font,
            horizontal_alignment: text.horizontal_alignment,
            vertical_alignment: text.vertical_alignment,
            line_height: text.line_height,
            shaping: text.shaping,
        }
    }
}
