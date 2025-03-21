//! A widget for an infinite 2D cartesian canvas.
//!
//! All points on the [`Infinite`] are considered as cartesian co-ordinates
//! with the origin at co-ord (0, 0).
//!
//! Functionality:
//!
//! All functionality requires the [`Infinite`] to be hovered on by the
//! cursor. These are currently implemented:
//!
//! - Cursor-focused scrolling: Mouse scroll or Cmd(Ctrl) + arrow direction.
//! - Origin-focused scrolling: Mouse scroll + Shift or Cmd(Ctrl) + Shift + arrow direction.
//! - Zoom: Shift + Mouse scroll or Shift + arrow direction.
//! - Reset Zoom: Shift + Home key.
//! - Reset Scroll: Home key.
//! - Reset Scroll and Zoom: Cmd(Ctrl) + Home key.
//!
//! Note:
//!
//! - Text cannot be zoomed (scaled up or down).
//! - Items on the canvas can be anchored on a single, both and no axis. An
//!   anchored Item does not move when scrolled on the anchoring axis.
//! - The Scrolling direction for the [`Infinite`] can be set using
//!       [`ScrollDirection`].
//! - Like the regualar Iced canvas, Items on an [`Infinite`] benefit
//!   from antialiasing being enabled.
//! - Unlike the regular Iced canvas, unless otherwise stated, shapes
//!   are drawn with respect to their bottom-left point.

use std::f32::consts::E;
use std::marker::PhantomData;

use iced::{
    advanced::{self, layout, mouse::Cursor, widget::tree, Widget},
    border::Radius,
    color, event as iced_event, keyboard, mouse, touch,
    widget::canvas::{path::lyon_path::geom::euclid::Transform2D, Frame},
    Background, Border, Color, Element, Length, Pixels, Point, Rectangle, Shadow, Size, Theme,
    Vector,
};

pub use iced::widget::canvas::{Fill, Path, Stroke, Text};

use iced_graphics::geometry;

use event::Event;
use style::*;

const DEFAULT_BACKGROUND: Background = Background::Color(color!(203, 213, 240));
const SCALE_STEP: f32 = 0.1;
const OFFSET_STEP: f32 = 25.0;

/// Handle [`Infinite`] canvas event.
pub mod event {
    /// The status of an [`Event`] after being processed.
    #[derive(Debug, Default, Clone, Copy, PartialEq)]
    pub enum Status {
        /// The [`Event`] was handled.
        Captured,
        #[default]
        /// The [`Event`] was not handled.
        Ignored,
    }

    impl Status {
        /// Merges two [`Status`].
        ///
        /// [`Status::Captured`] takes precedence over [`Status::Ignored`].
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
    /// An canvas event.
    pub enum Event {
        /// A mouse event.
        Mouse(iced::mouse::Event),
        /// A keyboard event.
        Keyboard(iced::keyboard::Event),
        /// A touch event.
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

/// The state and logic of a [`Infinite`].
///
/// A [`Program`] can mutate internal state and produce messages for an application.
pub trait Program<Message, Theme = iced::Theme, Renderer = iced::Renderer>
where
    Renderer: iced_graphics::geometry::Renderer,
{
    /// The internal state mutated by the [`Program`].
    type State: 'static;

    /// Returns the initial state of the [`Program`].
    fn init_state(&self) -> Self::State;

    /// Returns the scroll the [`Infinite`] starts with.
    ///
    /// Scrolling up in the Y direction pulls the canvas down, thus the Y vector
    /// component is negative.
    ///
    /// Resetting the [`Infinite`] returns the scroll back to this value
    fn init_scroll(&self) -> iced::Vector {
        Vector::new(0., 0.)
    }

    /// Returns the zoom the [`Infinite`] starts with.
    ///
    /// Resetting the [`Infinite`] returns the zoom back to this value
    fn init_zoom(&self) -> f32 {
        0.0
    }

    /// Draws the state of the [`Program`], returning a bunch of [`Buffer`]s.
    ///
    /// A cursor whose position is translated to fit the [`Infinite`] coordinate
    /// system is provided as `infinite_cursor`.
    fn draw<'a>(
        &self,
        state: &Self::State,
        theme: &Theme,
        bounds: Rectangle,
        cursor: mouse::Cursor,
        infinite_cursor: mouse::Cursor,
        center: Point,
    ) -> Vec<Buffer<'a>>;

    /// Updates the state of the [`Program`].
    ///
    /// Captured [`Event`]s do not trigger a scroll or zoom on the
    /// [`Infinite`].
    ///
    /// A cursor whose position is translated to fit the [`Infinite`] coordinate
    /// system is provided as `infinite_cursor`.
    ///
    /// This method can optionally return a Message to notify an application of any meaningful interactions.
    ///
    /// By default, this method does and returns nothing.
    fn update(
        &self,
        _state: &mut Self::State,
        _event: Event,
        _bounds: Rectangle,
        _cursor: mouse::Cursor,
        _infinite_cursor: mouse::Cursor,
    ) -> (event::Status, Option<Message>) {
        (event::Status::Ignored, None)
    }

    /// Returns the current mouse interaction of the [`Program`].
    ///
    /// A cursor whose position is translated to fit the [`Infinite`] coordinate
    /// system is provided as `infinite_cursor`.
    fn mouse_interaction(
        &self,
        _state: &Self::State,
        _bounds: Rectangle,
        _cursor: mouse::Cursor,
        _infinite_cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        mouse::Interaction::default()
    }

    /// Returns the overlay of the [`Infinite`], if there is any.
    ///
    /// A cursor whose position is translated to fit the [`Infinite`] coordinate
    /// system is provided as `infinite_cursor`.
    fn overlay<'a>(
        &self,
        _state: &'a mut Self::State,
        _bounds: Rectangle,
        _infinite_cursor: Point,
        _translation: Vector,
    ) -> Option<iced::advanced::overlay::Element<'a, Message, Theme, Renderer>> {
        None
    }

    /// Updates the state of the [`Program`] whenever a scroll occurs.
    ///
    /// The current scroll of the canvas is provided as `scroll` and the change
    /// is also provided as `diff`.
    ///
    /// A cursor whose position is translated to fit the [`Infinite`] coordinate
    /// system is provided as `infinite_cursor`.
    ///
    /// An optional Message can be returned to notify an application of any
    /// meaningful interactions.
    ///
    /// By default, this method does and returns nothing. source
    fn on_scroll(
        &self,
        _state: &mut Self::State,
        _bounds: Rectangle,
        _cursor: mouse::Cursor,
        _infinite_cursor: mouse::Cursor,
        _scroll: Vector,
        _diff: Vector,
    ) -> Option<Message> {
        None
    }

    /// Updates the state of the [`Program`] whenever a zoom occurs.
    ///
    /// The current zoom of the canvas is provided as `zoom` and the change
    /// is also provided as `diff`.
    ///
    /// A cursor whose position is translated to fit the [`Infinite`] coordinate
    /// system is provided as `infinite_cursor`.
    ///
    /// An optional Message can be returned to notify an application of any
    /// meaningful interactions.
    ///
    /// By default, this method does and returns nothing. source
    fn on_zoom(
        &self,
        _state: &mut Self::State,
        _bounds: Rectangle,
        _cursor: mouse::Cursor,
        _infinite_cursor: mouse::Cursor,
        _focal_point: Point,
        _zoom: f32,
        _diff: f32,
    ) -> Option<Message> {
        None
    }

    /// Updates the state of the [`Program`] when the scroll is reset to the
    /// starting value.
    ///
    /// A cursor whose position is translated to fit the [`Infinite`] coordinate
    /// system is provided as `infinite_cursor`.
    ///
    /// An optional Message can be returned to notify an application of any
    /// meaningful interactions.
    ///
    /// By default, this method does and returns nothing. source
    fn on_scroll_reset(
        &self,
        _state: &mut Self::State,
        _bounds: Rectangle,
        _cursor: mouse::Cursor,
        _infinite_cursor: mouse::Cursor,
        _scroll: Vector,
    ) -> Option<Message> {
        None
    }

    /// Updates the state of the [`Program`] when the zoom is reset to the
    /// starting value.
    ///
    /// A cursor whose position is translated to fit the [`Infinite`] coordinate
    /// system is provided as `infinite_cursor`.
    ///
    /// An optional Message can be returned to notify an application of any
    /// meaningful interactions.
    ///
    /// By default, this method does and returns nothing. source
    fn on_zoom_reset(
        &self,
        _state: &mut Self::State,
        _bounds: Rectangle,
        _cursor: mouse::Cursor,
        _infinite_cursor: mouse::Cursor,
        _zoom: f32,
    ) -> Option<Message> {
        None
    }
}

/// Determines the degree by which points on the canvas are fixed.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum Anchor {
    /// Both x and y coordinates are fixed and do not move in any direction.
    Both,
    /// The x coordinate is fixed while the y coordinate can
    /// freely move.
    X,
    /// The y coordinate  is fixed while the x coordinate can
    /// freely move.
    Y,
    /// Both x and y coordinates are not anchored and are free to move in
    /// any direction.
    #[default]
    None,
}

#[derive(Debug, Clone)]
/// A buffer which records the items on an [`Infinite`] canvas.
pub struct Buffer<'a> {
    fills: Vec<(Path, Fill, Anchor)>,
    strokes: Vec<(Path, Stroke<'a>, Anchor)>,
    text: Vec<(Text, Anchor)>,
    /// If `Some`, all items in this buffer inherit this anchor.
    anchor: Option<Anchor>,
    /// If true a scale transform is applied to all recorded Path.
    scale: bool,
}

impl<'a> Default for Buffer<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> Buffer<'a> {
    /// Creates a new [`Buffer`].
    pub fn new() -> Self {
        Self {
            fills: Vec::new(),
            strokes: Vec::new(),
            text: Vec::new(),
            anchor: None,
            scale: true,
        }
    }

    /// Creates a [`Buffer`] with all items having the same anchored.
    ///
    ///
    /// After calling this function, the all stored items, both past and
    /// future will have their anchors removed.
    pub fn anchor_all(mut self, anchor: Anchor) -> Self {
        self.anchor = Some(anchor);
        self
    }

    /// Sets whether all items in the [`Buffer`] should be scale transformed
    pub fn scale_all(mut self, scale: bool) -> Self {
        self.scale = scale;
        self
    }

    /// Draws the characters of the given [`Text`] on the [`Infinite`] canvas with the anchor.
    pub fn draw_text_anchored(&mut self, text: impl Into<Text>, anchor: Anchor) {
        self.text.push((text.into(), anchor))
    }

    /// Draws the characters of the given [`Text`] on the [`Infinite`] canvas using the anchor of the [`Buffer`].
    pub fn draw_text(&mut self, text: impl Into<Text>) {
        self.text
            .push((text.into(), self.anchor.unwrap_or_default()))
    }

    /// Draws the fill of the given [`Path`] on the [`Infinite`] canvas with an anchor by filling it with the provided style.
    pub fn fill_anchored(&mut self, path: Path, fill: impl Into<Fill>, anchor: Anchor) {
        self.fills.push((path, fill.into(), anchor))
    }

    /// Draws the fill of the given [`Path`] on the [`Infinite`] canvas with the [`Buffer`]'s anchor by filling it with the provided style.
    pub fn fill(&mut self, path: Path, fill: impl Into<Fill>) {
        self.fills
            .push((path, fill.into(), self.anchor.unwrap_or_default()))
    }

    /// Draws the stroke of the given [`Path`] on the [`Infinite`] canvas with the provided style and anchor.
    pub fn stroke_anchored(&mut self, path: Path, stroke: impl Into<Stroke<'a>>, anchor: Anchor) {
        self.strokes.push((path, stroke.into(), anchor))
    }

    /// Draws the stroke of the given [`Path`] on the [`Infinite`] canvas with the provided style and the [`Buffer`]'s anchor.
    pub fn stroke(&mut self, path: Path, stroke: impl Into<Stroke<'a>>) {
        self.strokes
            .push((path, stroke.into(), self.anchor.unwrap_or_default()))
    }

    /// Draws a rectangle given its bottom-left corner coordinate, [`Size`] and [`Anchor`] by filling it with the provided style.
    pub fn fill_rectangle_anchored(
        &mut self,
        bottom_left: impl Into<Point>,
        size: impl Into<Size>,
        fill: impl Into<Fill>,
        anchor: Anchor,
    ) {
        let size: Size = size.into();
        let bottom_left = bottom_left.into();

        let path = Path::rectangle(bottom_left, size);

        self.fill_anchored(path, fill, anchor)
    }

    /// Draws a rectangle given its bottom-left corner coordinate and its [`Size`] by filling it with the provided style and the [`Buffer`]'s anchor.
    pub fn fill_rectangle(
        &mut self,
        bottom_left: impl Into<Point>,
        size: impl Into<Size>,
        fill: impl Into<Fill>,
    ) {
        let size: Size = size.into();
        let bottom_left = bottom_left.into();

        let path = Path::rectangle(bottom_left, size);

        self.fill_anchored(path, fill, self.anchor.unwrap_or_default())
    }

    /// Draws a rounded rectangle given its bottom-left corner coordinate, [`Size`] and [`Anchor`] by filling it with the provided style.
    pub fn fill_rounded_rectangle_anchored(
        &mut self,
        bottom_left: impl Into<Point>,
        size: impl Into<Size>,
        radius: impl Into<Radius>,
        fill: impl Into<Fill>,
        anchor: Anchor,
    ) {
        let size: Size = size.into();
        let bottom_left = bottom_left.into();

        let path = Path::rounded_rectangle(bottom_left, size, radius.into());

        self.fill_anchored(path, fill, anchor);
    }

    /// Draws a rounded rectangle given its bottom-left corner coordinate and its [`Size`] by filling it with the provided style and the [`Buffer`]'s anchor.
    pub fn fill_rounded_rectangle(
        &mut self,
        bottom_left: impl Into<Point>,
        size: impl Into<Size>,
        radius: impl Into<Radius>,
        fill: impl Into<Fill>,
    ) {
        let size: Size = size.into();
        let bottom_left = bottom_left.into();

        let path = Path::rounded_rectangle(bottom_left, size, radius.into());

        self.fill(path, fill);
    }

    /// Draws the stroke of a rectangle with the provided style given its bottom-left corner coordinate and its [`Size`].
    pub fn stroke_rect_anchored(
        &mut self,
        bottom_left: impl Into<Point>,
        size: impl Into<Size>,
        stroke: impl Into<Stroke<'a>>,
        anchor: Anchor,
    ) {
        let size: Size = size.into();
        let bottom_left = bottom_left.into();

        let path = Path::rectangle(bottom_left, size);

        self.stroke_anchored(path, stroke, anchor)
    }

    /// Draws the stroke of a rectangle with the provided style given its bottom-left corner coordinate and its [`Size`] and the [`Buffer`]'s anchor.
    pub fn stroke_rectangle(
        &mut self,
        bottom_left: impl Into<Point>,
        size: impl Into<Size>,
        stroke: impl Into<Stroke<'a>>,
    ) {
        let size: Size = size.into();
        let bottom_left = bottom_left.into();

        let path = Path::rectangle(bottom_left, size);

        self.stroke(path, stroke)
    }

    /// Draws the stroke of a rounded rectangle with the provided style given its bottom-left corner coordinate and its [`Size`].
    pub fn stroke_rounded_rectangle_anchored(
        &mut self,
        bottom_left: impl Into<Point>,
        size: impl Into<Size>,
        radius: impl Into<Radius>,
        stroke: impl Into<Stroke<'a>>,
        anchor: Anchor,
    ) {
        let size: Size = size.into();
        let bottom_left = bottom_left.into();

        let path = Path::rounded_rectangle(bottom_left, size, radius.into());

        self.stroke_anchored(path, stroke, anchor);
    }

    /// Draws the stroke of a rounded rectangle with the provided style given its bottom-left corner coordinate and its [`Size`] and the [`Buffer`]'s anchor.
    pub fn stroke_rounded_rectangle(
        &mut self,
        bottom_left: impl Into<Point>,
        size: impl Into<Size>,
        radius: impl Into<Radius>,
        stroke: impl Into<Stroke<'a>>,
    ) {
        let size: Size = size.into();
        let bottom_left = bottom_left.into();

        let path = Path::rounded_rectangle(bottom_left, size, radius.into());

        self.stroke(path, stroke);
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
                let path = transform_path(
                    state,
                    center,
                    path,
                    self.anchor.unwrap_or(*anchor),
                    self.scale,
                );
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
                let path = transform_path(
                    state,
                    center,
                    path,
                    self.anchor.unwrap_or(*anchor),
                    self.scale,
                );
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
    /// No scroll in any direction. Scroll events are thus ignored.
    None,
}

/// A widget capable of drawing 2D graphics on an infinite Cartesian plane.
pub struct Infinite<'a, P, Message, Theme = iced::Theme, Renderer = iced::Renderer>
where
    Theme: Catalog,
    P: Program<Message, Theme, Renderer>,
    Renderer: geometry::Renderer,
{
    width: Length,
    height: Length,
    direction: ScrollDirection,
    allow_scale: bool,
    scale_step: Option<f32>,
    offset_step: Option<Vector>,
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

    /// Creates a new [`Infinite`].
    pub fn new(program: P) -> Self {
        Self {
            width: Length::Fixed(Self::DEFAULT_SIZE),
            height: Length::Fixed(Self::DEFAULT_SIZE),
            direction: ScrollDirection::default(),
            allow_scale: true,
            scale_step: None,
            offset_step: None,
            program,
            _message: PhantomData,
            _renderer: PhantomData,
            style: Theme::default(),
        }
    }

    /// Sets the height of the [`Infinite`].
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    /// Sets the width of the [`Infinite`].
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the supported scroll direction of the [`Infinite`].
    pub fn scroll_direction(mut self, direction: ScrollDirection) -> Self {
        self.direction = direction;
        self
    }

    /// Sets whether the [`Infinite`] can be zoomed in/out on.
    pub fn zoom(mut self, allow: bool) -> Self {
        self.allow_scale = allow;
        self
    }

    /// Sets the value of a single zoom on the [`Infinite`].
    pub fn zoom_step(mut self, step: f32) -> Self {
        self.scale_step = Some(step);
        self
    }

    /// Sets the value of a single scroll on the [`Infinite`].
    pub fn scroll_step(mut self, step: Vector) -> Self {
        self.offset_step = Some(step);
        self
    }

    /// Sets  the style of the [`Infinite`].
    pub fn style(mut self, style: impl Fn(&Theme, Status) -> Style + 'a) -> Self
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
        let state = self.program.init_state();
        let mut state = InfiniteState::<P::State>::new(state);

        state.offset = self.program.init_scroll();
        state.set_scale_level(self.program.init_zoom());

        tree::State::new(state)
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

        let canvas_event = {
            let state = state.state.downcast_ref::<InfiniteState<P::State>>();

            wrap_event(event.clone(), bounds, state.offset, state.scale)
        };

        if let Some(canvas_event) = canvas_event {
            let state = state.state.downcast_mut::<InfiniteState<P::State>>();
            let (cursor, infinite) = get_cursors(cursor, bounds, state.offset, state.scale);

            let (status, message) =
                self.program
                    .update(&mut state.state, canvas_event, bounds, cursor, infinite);

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
                let (cursor, infinite) = get_cursors(cursor, bounds, state.offset, state.scale);
                let modifiers = state.keyboard_modifier;
                let scale_step = self.scale_step.unwrap_or(SCALE_STEP);

                match delta {
                    // Zoom
                    mouse::ScrollDelta::Lines { y, .. }
                        if modifiers.shift() && modifiers.command() =>
                    {
                        if !self.allow_scale {
                            return iced_event::Status::Ignored;
                        };
                        let step = if y < 0. { -scale_step } else { scale_step };
                        handle_scale(self, state, shell, bounds, (cursor, infinite), step, true)
                    }
                    mouse::ScrollDelta::Pixels { y, .. }
                        if modifiers.shift() && modifiers.command() =>
                    {
                        if !self.allow_scale {
                            return iced_event::Status::Ignored;
                        };
                        let step = if y < 0. { -scale_step } else { scale_step };
                        handle_scale(self, state, shell, bounds, (cursor, infinite), step, true)
                    }
                    mouse::ScrollDelta::Lines { y, .. } if modifiers.shift() => {
                        if !self.allow_scale {
                            return iced_event::Status::Ignored;
                        };
                        let step = if y < 0. { -scale_step } else { scale_step };
                        handle_scale(self, state, shell, bounds, (cursor, infinite), step, false)
                    }
                    mouse::ScrollDelta::Pixels { y, .. } if modifiers.shift() => {
                        if !self.allow_scale {
                            return iced_event::Status::Ignored;
                        };
                        let step = if y < 0. { -scale_step } else { scale_step };
                        handle_scale(self, state, shell, bounds, (cursor, infinite), step, false)
                    }

                    // Translation
                    mouse::ScrollDelta::Pixels { x, y } => {
                        let (x, y) = match self.offset_step {
                            Some(offset) => (offset.x, offset.y),
                            None => (x, y),
                        };
                        let offset = match self.direction {
                            ScrollDirection::X => Vector::new(x, 0.),
                            ScrollDirection::Y => Vector::new(0., y),
                            ScrollDirection::Both => Vector::new(x, y),
                            ScrollDirection::None => return iced_event::Status::Ignored,
                        };

                        state.offset = state.offset - offset;
                        let msg = self.program.on_scroll(
                            &mut state.state,
                            bounds,
                            cursor,
                            infinite,
                            state.offset,
                            -offset,
                        );

                        if let Some(msg) = msg {
                            shell.publish(msg);
                        }

                        iced_event::Status::Captured
                    }
                    mouse::ScrollDelta::Lines { x, y } => {
                        let (x, y) = match self.offset_step {
                            Some(offset) => (offset.x, offset.y),
                            None => (x, y),
                        };
                        let mult = 100.0;
                        let offset = match self.direction {
                            ScrollDirection::X => Vector::new(x, 0.),
                            ScrollDirection::Y => Vector::new(0., y),
                            ScrollDirection::Both => Vector::new(x, y),
                            ScrollDirection::None => return iced_event::Status::Ignored,
                        } * mult;

                        state.offset = state.offset - offset;
                        let msg = self.program.on_scroll(
                            &mut state.state,
                            bounds,
                            cursor,
                            infinite,
                            state.offset,
                            -offset,
                        );

                        if let Some(msg) = msg {
                            shell.publish(msg);
                        }

                        iced_event::Status::Captured
                    }
                }
            }

            iced::Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. }) => {
                let state = state.state.downcast_mut::<InfiniteState<P::State>>();
                let (cursor, infinite) = get_cursors(cursor, bounds, state.offset, state.scale);
                let (offset_x, offset_y) = match self.offset_step {
                    Some(offset) => (offset.x, offset.y),
                    None => (OFFSET_STEP, OFFSET_STEP),
                };
                let scale_step = self.scale_step.unwrap_or(SCALE_STEP);

                match key {
                    // Zoom
                    keyboard::Key::Named(keyboard::key::Named::ArrowUp)
                        if modifiers.shift() && modifiers.command() =>
                    {
                        if !self.allow_scale {
                            return iced_event::Status::Ignored;
                        };
                        handle_scale(
                            self,
                            state,
                            shell,
                            bounds,
                            (cursor, infinite),
                            scale_step,
                            true,
                        )
                    }

                    keyboard::Key::Named(keyboard::key::Named::ArrowDown)
                        if modifiers.shift() && modifiers.command() =>
                    {
                        if !self.allow_scale {
                            return iced_event::Status::Ignored;
                        };
                        handle_scale(
                            self,
                            state,
                            shell,
                            bounds,
                            (cursor, infinite),
                            -scale_step,
                            true,
                        )
                    }

                    keyboard::Key::Named(keyboard::key::Named::ArrowUp) if modifiers.shift() => {
                        if !self.allow_scale {
                            return iced_event::Status::Ignored;
                        };
                        handle_scale(
                            self,
                            state,
                            shell,
                            bounds,
                            (cursor, infinite),
                            scale_step,
                            false,
                        )
                    }

                    keyboard::Key::Named(keyboard::key::Named::ArrowDown) if modifiers.shift() => {
                        if !self.allow_scale {
                            return iced_event::Status::Ignored;
                        };
                        handle_scale(
                            self,
                            state,
                            shell,
                            bounds,
                            (cursor, infinite),
                            -scale_step,
                            false,
                        )
                    }

                    // Translations
                    keyboard::Key::Named(keyboard::key::Named::ArrowUp) if modifiers.command() => {
                        let offset = match self.direction {
                            ScrollDirection::X => Vector::new(0., 0.),
                            ScrollDirection::Y => Vector::new(0., offset_y),
                            ScrollDirection::Both => Vector::new(0., offset_y),
                            ScrollDirection::None => return iced_event::Status::Ignored,
                        } * (1.0 / state.scale);

                        state.offset = state.offset - offset;
                        let msg = self.program.on_scroll(
                            &mut state.state,
                            bounds,
                            cursor,
                            infinite,
                            state.offset,
                            -offset,
                        );

                        if let Some(msg) = msg {
                            shell.publish(msg);
                        }

                        iced_event::Status::Captured
                    }

                    keyboard::Key::Named(keyboard::key::Named::ArrowDown)
                        if modifiers.command() =>
                    {
                        let offset = match self.direction {
                            ScrollDirection::X => Vector::new(0., 0.),
                            ScrollDirection::Y => Vector::new(0., offset_y),
                            ScrollDirection::Both => Vector::new(0., offset_y),
                            ScrollDirection::None => return iced_event::Status::Ignored,
                        } * (1.0 / state.scale);
                        state.offset = state.offset + offset;

                        let msg = self.program.on_scroll(
                            &mut state.state,
                            bounds,
                            cursor,
                            infinite,
                            state.offset,
                            offset,
                        );

                        if let Some(msg) = msg {
                            shell.publish(msg);
                        }

                        iced_event::Status::Captured
                    }

                    keyboard::Key::Named(keyboard::key::Named::ArrowLeft)
                        if modifiers.command() =>
                    {
                        let offset = match self.direction {
                            ScrollDirection::X => Vector::new(offset_x, 0.),
                            ScrollDirection::Y => Vector::new(0., 0.),
                            ScrollDirection::Both => Vector::new(offset_x, 0.),
                            ScrollDirection::None => return iced_event::Status::Ignored,
                        } * (1.0 / state.scale);
                        state.offset = state.offset - offset;

                        let msg = self.program.on_scroll(
                            &mut state.state,
                            bounds,
                            cursor,
                            infinite,
                            state.offset,
                            -offset,
                        );

                        if let Some(msg) = msg {
                            shell.publish(msg);
                        }

                        iced_event::Status::Captured
                    }
                    keyboard::Key::Named(keyboard::key::Named::ArrowRight)
                        if modifiers.command() =>
                    {
                        let offset = match self.direction {
                            ScrollDirection::X => Vector::new(offset_x, 0.),
                            ScrollDirection::Y => Vector::new(0., 0.),
                            ScrollDirection::Both => Vector::new(offset_x, 0.),
                            ScrollDirection::None => return iced_event::Status::Ignored,
                        } * (1.0 / state.scale);
                        state.offset = state.offset + offset;

                        let msg = self.program.on_scroll(
                            &mut state.state,
                            bounds,
                            cursor,
                            infinite,
                            state.offset,
                            offset,
                        );

                        if let Some(msg) = msg {
                            shell.publish(msg);
                        }
                        iced_event::Status::Captured
                    }

                    // Resets
                    keyboard::Key::Named(keyboard::key::Named::Home) if modifiers.command() => {
                        let init_offset = self.program.init_scroll();
                        let init_scale = self.program.init_zoom();

                        state.reset_all(init_offset, init_scale);

                        if let Some(msg) = self.program.on_scroll_reset(
                            &mut state.state,
                            bounds,
                            cursor,
                            infinite,
                            init_offset,
                        ) {
                            shell.publish(msg);
                        }

                        if let Some(msg) = self.program.on_zoom_reset(
                            &mut state.state,
                            bounds,
                            cursor,
                            infinite,
                            init_scale,
                        ) {
                            shell.publish(msg);
                        }

                        iced_event::Status::Captured
                    }

                    keyboard::Key::Named(keyboard::key::Named::Home) if modifiers.shift() => {
                        let init = self.program.init_zoom();
                        state.reset_scale(init);

                        let msg = self.program.on_zoom_reset(
                            &mut state.state,
                            bounds,
                            cursor,
                            infinite,
                            state.scale,
                        );

                        if let Some(msg) = msg {
                            shell.publish(msg);
                        }
                        iced_event::Status::Captured
                    }

                    keyboard::Key::Named(keyboard::key::Named::Home) => {
                        let init = self.program.init_scroll();
                        state.reset_offset(init);

                        let msg = self.program.on_scroll_reset(
                            &mut state.state,
                            bounds,
                            cursor,
                            infinite,
                            init,
                        );

                        if let Some(msg) = msg {
                            shell.publish(msg);
                        }

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

            iced::Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                let state = state.state.downcast_mut::<InfiniteState<P::State>>();
                let (_, cursor) = get_cursors(cursor, bounds, state.offset, state.scale);

                state.set_mouse_position(cursor.position());

                iced_event::Status::Captured
            }

            iced::Event::Mouse(mouse::Event::CursorLeft) => {
                let state = state.state.downcast_mut::<InfiniteState<P::State>>();
                state.set_mouse_position(None);

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
        let state = &state.state.downcast_ref::<InfiniteState<P::State>>();
        let (cursor, infinite) = get_cursors(cursor, bounds, state.offset, state.scale);

        self.program
            .mouse_interaction(&state.state, bounds, cursor, infinite)
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
            Status::Hovered
        } else {
            Status::Active
        };

        let style = theme.style(&self.style, status);

        let state = tree.state.downcast_ref::<InfiniteState<P::State>>();

        renderer.fill_quad(
            advanced::renderer::Quad {
                bounds,
                border: style.border,
                shadow: Shadow::default(),
            },
            style.background,
        );

        let border_width = style.border.width;

        let bounds = {
            let width = bounds.width - (2. * border_width);
            let height = bounds.height - (2.0 * border_width);

            let position = bounds.position();

            let top_left = Point::new(position.x + border_width, position.y + border_width);

            Rectangle::new(top_left, Size::new(width, height))
        };

        let position = bounds.position();

        renderer.with_translation(Vector::new(position.x, position.y), |renderer| {
            let mut frame = Frame::new(renderer, bounds.size());
            let center = frame.center();

            let (cursor, infinite) = get_cursors(cursor, bounds, state.offset, state.scale);

            let buffers = self.program.draw(
                &state.state,
                theme,
                bounds,
                cursor,
                infinite,
                Point::ORIGIN - state.offset,
            );

            for buffer in buffers {
                buffer.draw(&mut frame, state, center);
            }

            let top = 2.5;
            let left = 8.0;
            let details_padding = {
                let bottom = 2.5;
                let right = 8.0;
                Size::new(left + right, top + bottom)
            };
            let details_bounds = Size::INFINITY;
            let details_size = 16.0;

            if state.scale_level != 0.0 {
                let pos = (bounds.width * 0.9, bounds.height * 0.95).into();
                let background = style.details_background;
                let radius = style.details_border_radius;
                let color = style.details_text;

                let scale = (state.scale_level) * 100.;

                let scale_string = format!("{:.0}%", scale);
                let min_bounds = min_text_bounds(&scale_string, details_bounds, details_size);
                let bounds = min_bounds.expand(details_padding);

                let rect = Path::rounded_rectangle(pos, bounds, radius);

                frame.fill(&rect, background);

                let text = Text {
                    content: scale_string,
                    position: (pos.x + left, pos.y + top).into(),
                    color,
                    ..Default::default()
                };

                frame.fill_text(text);
            }

            if state.offset != Vector::ZERO {
                let pos = (bounds.width * 0.01, bounds.height * 0.95).into();
                let background = style.details_background;
                let radius = style.details_border_radius;
                let color = style.details_text;

                let x = state.offset.x;
                let y = state.offset.y * -1.;

                let offset_string = format!("x: {x:.1}, y: {y:.1}");
                let min_bounds = min_text_bounds(&offset_string, details_bounds, details_size);
                let bounds = min_bounds.expand(details_padding);

                let rect = Path::rounded_rectangle(pos, bounds, radius);

                frame.fill(&rect, background);

                let text = Text {
                    content: offset_string,
                    position: (pos.x + left, pos.y + top).into(),
                    color,
                    ..Default::default()
                };

                frame.fill_text(text);
            }

            let geoms = frame.into_geometry();

            renderer.draw_geometry(geoms);
        });
    }

    fn overlay<'b>(
        &'b mut self,
        state: &'b mut tree::Tree,
        layout: layout::Layout<'_>,
        _renderer: &Renderer,
        translation: Vector,
    ) -> Option<advanced::overlay::Element<'b, Message, Theme, Renderer>> {
        let bounds = layout.bounds();
        let state = state.state.downcast_mut::<InfiniteState<P::State>>();

        self.program.overlay(
            &mut state.state,
            bounds,
            state.mouse_position.unwrap_or_default(),
            translation,
        )
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

#[derive(Debug, Clone, Copy, PartialEq)]
struct InfiniteState<State> {
    offset: Vector,
    scale_level: f32,
    scale: f32,
    keyboard_modifier: keyboard::Modifiers,
    state: State,
    /// The virtual position of the cursor
    mouse_position: Option<Point>,
}

impl<State> InfiniteState<State> {
    fn new(state: State) -> Self {
        let scale_level = 0.0;
        let scale = E.powf(scale_level);
        Self {
            offset: Vector::new(0., 0.),
            scale_level,
            state,
            scale,
            keyboard_modifier: keyboard::Modifiers::default(),
            mouse_position: None,
        }
    }

    fn set_mouse_position(&mut self, position: Option<Point>) {
        self.mouse_position = position;
    }

    /// Adds to scale level
    fn add_level(&mut self, diff: f32, focal_origin: bool) -> Vector {
        self.scale_level += diff;
        let prev_scale = self.scale;
        self.scale = E.powf(self.scale_level);

        let delta = if focal_origin {
            let ratio = if diff < 0.0 {
                prev_scale / self.scale
            } else {
                self.scale / prev_scale
            };

            let diff = 1.0 - ratio;

            Vector::new(diff * self.offset.x, -diff * self.offset.y)
        } else {
            let diff = self.scale - prev_scale;
            let cursor = self.mouse_position.unwrap_or(Point::ORIGIN);

            Vector::new(diff * cursor.x, -diff * cursor.y)
        };

        self.offset = self.offset + delta;

        delta
    }

    fn set_scale_level(&mut self, level: f32) {
        self.scale_level = level;
        self.scale = E.powf(self.scale_level);
    }

    fn reset_all(&mut self, offset: Vector, scale: f32) {
        self.reset_scale(scale);
        self.reset_offset(offset);
    }

    fn reset_offset(&mut self, init: Vector) {
        self.offset = init;
    }

    fn reset_scale(&mut self, init: f32) {
        self.scale_level = init;
        let prev_scale = self.scale;
        self.scale = E.powf(self.scale_level);

        let delta = {
            let diff = self.scale - prev_scale;
            let mouse = self.mouse_position.unwrap_or_default();
            Vector::new(diff * mouse.x, -diff * mouse.y)
        };

        self.offset = self.offset + delta;
    }
}

/// Style an [`Infinite`] canvas.
pub mod style {
    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq)]
    /// The appearance of the [`Infinite`].
    pub struct Style {
        /// The [`Border`] of the [`Infinite`].
        pub border: Border,
        /// The [`Background`] of the [`Infinite`].
        pub background: Background,
        /// The border radius of the [`Infinite`]'s details.
        pub details_border_radius: Radius,
        /// The [`Background`] of the [`Infinite`]'s details.
        pub details_background: Color,
        /// The text [`Color`] of the [`Infinite`]'s details.
        pub details_text: Color,
    }

    #[derive(Debug, Clone, Copy, Default, PartialEq)]
    /// The possible status of an [`Infinite`].
    pub enum Status {
        #[default]
        /// The [`Infinite`] is not being hovered on.
        Active,
        /// The [`Infinite`] is being hovered on.
        Hovered,
    }

    /// The theme of an [`Infinite`].
    pub trait Catalog {
        /// The item class of the [`Catalog`].
        type Class<'a>;

        /// The default class produced by the [`Catalog`].
        fn default<'a>() -> Self::Class<'a>;

        /// The [`Style`] of a class with the given status.
        fn style(&self, class: &Self::Class<'_>, status: Status) -> Style;
    }

    /// A styling function for an [`Infinite`].
    pub type StyleFn<'a, Theme> = Box<dyn Fn(&Theme, Status) -> Style + 'a>;

    impl Catalog for Theme {
        type Class<'a> = StyleFn<'a, Self>;

        fn default<'a>() -> Self::Class<'a> {
            Box::new(default)
        }

        fn style(&self, class: &Self::Class<'_>, status: Status) -> Style {
            class(self, status)
        }
    }

    /// The default [`Theme`] styling of an [`Infinite`].
    pub fn default(theme: &Theme, status: Status) -> Style {
        let palette = theme.extended_palette();
        let border_width = 2.5;

        let background = palette.background.base;
        let details_background = Color {
            a: 0.9,
            ..background.color
        };
        let details_text = background.text;

        let border = match status {
            Status::Active => Border::default()
                .width(border_width)
                .color(palette.background.base.color),
            Status::Hovered => Border::default()
                .width(border_width)
                .color(palette.primary.strong.color),
        };

        Style {
            border,
            background: DEFAULT_BACKGROUND,
            details_background,
            details_border_radius: 5.into(),
            details_text,
        }
    }
}

/// Returns a pair of [`Cursor`]s with the second [`Cursor`]'s point translated
/// to fit within the [`Infinite`]'s coordinate system.
fn get_cursors(cursor: Cursor, bounds: Rectangle, offset: Vector, scale: f32) -> (Cursor, Cursor) {
    match cursor {
        Cursor::Available(point) => {
            let point = bounds.center() - point;
            let point = (point - offset) * (1. / scale);
            let point = Point::new(-point.x, point.y);

            (cursor, Cursor::Available(point))
        }
        Cursor::Unavailable => (cursor, cursor),
    }
}

/// Returns the minimum bounds that can fit `text`.
pub fn min_text_bounds(text: &str, bounds: Size, size: impl Into<Pixels>) -> Size {
    use iced::{
        advanced::{
            self,
            text::{self, Paragraph},
        },
        alignment, Font,
    };

    let text = advanced::Text {
        content: text,
        bounds,
        font: Font::default(),
        size: size.into(),
        line_height: text::LineHeight::default(),
        horizontal_alignment: alignment::Horizontal::Left,
        vertical_alignment: alignment::Vertical::Center,
        wrapping: text::Wrapping::default(),
        shaping: text::Shaping::default(),
    };

    let text = iced_graphics::text::Paragraph::with_text(text);

    text.min_bounds()
}

fn wrap_event(
    event: iced::Event,
    bounds: Rectangle,
    offset: Vector,
    scale: f32,
) -> Option<event::Event> {
    match event.clone() {
        iced::Event::Mouse(mouse::Event::CursorMoved { position }) => {
            let point = bounds.center() - position;
            let point = (point - offset) * (1. / scale);
            let position = Point::new(-point.x, point.y);
            Some(Event::Mouse(mouse::Event::CursorMoved { position }))
        }
        iced::Event::Mouse(event) => Some(Event::Mouse(event)),
        iced::Event::Keyboard(event) => Some(Event::Keyboard(event)),
        iced::Event::Touch(event) => {
            let event = match event {
                touch::Event::FingerLost { id, position } => {
                    let position = bounds.center() - position;
                    let position = (position - offset) * (1. / scale);
                    let position = Point::new(-position.x, position.y);
                    Event::Touch(touch::Event::FingerLost { id, position })
                }
                touch::Event::FingerMoved { id, position } => {
                    let position = bounds.center() - position;
                    let position = (position - offset) * (1. / scale);
                    let position = Point::new(-position.x, position.y);
                    Event::Touch(touch::Event::FingerMoved { id, position })
                }
                touch::Event::FingerLifted { id, position } => {
                    let position = bounds.center() - position;
                    let position = (position - offset) * (1. / scale);
                    let position = Point::new(-position.x, position.y);
                    Event::Touch(touch::Event::FingerLifted { id, position })
                }
                touch::Event::FingerPressed { id, position } => {
                    let position = bounds.center() - position;
                    let position = (position - offset) * (1. / scale);
                    let position = Point::new(-position.x, position.y);
                    Event::Touch(touch::Event::FingerPressed { id, position })
                }
            };

            Some(event)
        }

        _ => None,
    }
}

fn transform_path<State>(
    state: &InfiniteState<State>,
    center: Point,
    path: &Path,
    anchor: Anchor,
    scale: bool,
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
    let scale = if scale { state.scale } else { 1.0 };

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
    //dbg!(&text.content);
    //dbg!(text.position);
    let position = translate_point(state, center, text.position, anchor);
    //dbg!(position);

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

fn handle_scale<P, Message, Theme, Renderer>(
    canvas: &Infinite<P, Message, Theme, Renderer>,
    state: &mut InfiniteState<P::State>,
    shell: &mut advanced::Shell<'_, Message>,
    bounds: Rectangle,
    cursors: (Cursor, Cursor),
    zoom: f32,
    focal_origin: bool,
) -> iced::event::Status
where
    Theme: Catalog,
    P: Program<Message, Theme, Renderer>,
    Renderer: geometry::Renderer,
{
    let offset_diff = state.add_level(zoom, focal_origin);
    let focal_point = if focal_origin {
        Point::ORIGIN
    } else {
        state.mouse_position.unwrap_or(Point::ORIGIN)
    };

    let msg = canvas.program.on_zoom(
        &mut state.state,
        bounds,
        cursors.0,
        cursors.1,
        focal_point,
        state.scale,
        zoom,
    );

    if let Some(msg) = msg {
        shell.publish(msg);
    }

    if let Some(msg) = canvas.program.on_scroll(
        &mut state.state,
        bounds,
        cursors.0,
        cursors.1,
        state.offset,
        offset_diff,
    ) {
        shell.publish(msg);
    }

    iced_event::Status::Captured
}
