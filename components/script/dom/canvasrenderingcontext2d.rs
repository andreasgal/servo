/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::CanvasRenderingContext2DBinding;
use dom::bindings::codegen::Bindings::CanvasRenderingContext2DBinding::CanvasRenderingContext2DMethods;
use dom::bindings::global::{GlobalRef, GlobalField};
use dom::bindings::js::{JS, JSRef, LayoutJS, Temporary};
use dom::bindings::utils::{Reflector, reflect_dom_object};
use dom::htmlcanvaselement::HTMLCanvasElement;

use geom::point::Point2D;
use geom::rect::Rect;
use geom::size::Size2D;

use canvas::canvas_paint_task::{CanvasMsg, CanvasPaintTask};
use canvas::canvas_paint_task::CanvasMsg::{ClearRect, Close, FillRect, Recreate, StrokeRect};

use azure::azure_hl::{CompositionOp, JoinStyle, CapStyle};

use std::sync::mpsc::Sender;

#[jstraceable]
enum TextAlign {
    Start,
    End,
    Left,
    Right,
    Center,
}

#[jstraceable]
enum TextBaseline {
    Top,
    Hanging,
    Middle,
    Alphabetic,
    Ideographic,
    Bottom,
}

#[jstraceable]
struct State {
    textAlign: TextAlign,
    textBaseline: TextBaseline,
    lineWidth: f64,
    lineJoin: JoinStyle,
    lineCap: CapStyle,
    miterLimit: f64,
    dashOffset: f64,
    globalAlpha: f64,
    shadowBlur: f64,
    op: CompositionOp,
}

impl State {
    pub fn new() -> State {
        State {
            textAlign: TextAlign::Start,
            textBaseline: TextBaseline::Alphabetic,
            lineWidth: 1.0,
            lineJoin: JoinStyle::MiterOrBevel,
            lineCap: CapStyle::Butt,
            miterLimit: 10.0,
            globalAlpha: 1.0,
            shadowBlur: 0.0,
            dashOffset: 0.0,
            op: CompositionOp::Over,
        }
    }
}

#[dom_struct]
pub struct CanvasRenderingContext2D<'a> {
    reflector_: Reflector,
    global: GlobalField,
    renderer: Sender<CanvasMsg>,
    canvas: JS<HTMLCanvasElement>,
    state: Vec<&'a mut State>,
}

impl<'a> CanvasRenderingContext2D<'a> {
    fn new_inherited(global: GlobalRef, canvas: JSRef<HTMLCanvasElement>, size: Size2D<i32>) -> CanvasRenderingContext2D<'a> {
        CanvasRenderingContext2D {
            reflector_: Reflector::new(),
            global: GlobalField::from_rooted(&global),
            renderer: CanvasPaintTask::start(size),
            canvas: JS::from_rooted(canvas),
            state: vec![State::new()],
        }
    }

    pub fn new(global: GlobalRef, canvas: JSRef<HTMLCanvasElement>, size: Size2D<i32>) -> Temporary<CanvasRenderingContext2D<'a>> {
        reflect_dom_object(box CanvasRenderingContext2D::new_inherited(global, canvas, size),
                           global, CanvasRenderingContext2DBinding::Wrap)
    }

    pub fn recreate(&self, size: Size2D<i32>) {
        self.renderer.send(Recreate(size)).unwrap();
    }

    fn currentState(&self) -> &mut State {
        self.state[self.state.len() - 1]
    }
}

pub trait LayoutCanvasRenderingContext2DHelpers<'a> {
    unsafe fn get_renderer(&self) -> Sender<CanvasMsg>;
}

impl<'a> LayoutCanvasRenderingContext2DHelpers for LayoutJS<CanvasRenderingContext2D<'a>> {
    unsafe fn get_renderer(&self) -> Sender<CanvasMsg> {
        (*self.unsafe_get()).renderer.clone()
    }
}

impl<'a, 'b> CanvasRenderingContext2DMethods for JSRef<'a, CanvasRenderingContext2D<'b>> {
    fn Canvas(self) -> Temporary<HTMLCanvasElement> {
        Temporary::new(self.canvas)
    }

    fn FillRect(self, x: f64, y: f64, width: f64, height: f64) {
        let rect = Rect(Point2D(x as f32, y as f32), Size2D(width as f32, height as f32));
        self.renderer.send(FillRect(rect)).unwrap();
    }

    fn ClearRect(self, x: f64, y: f64, width: f64, height: f64) {
        let rect = Rect(Point2D(x as f32, y as f32), Size2D(width as f32, height as f32));
        self.renderer.send(ClearRect(rect)).unwrap();
    }

    fn StrokeRect(self, x: f64, y: f64, width: f64, height: f64) {
        let rect = Rect(Point2D(x as f32, y as f32), Size2D(width as f32, height as f32));
        self.renderer.send(StrokeRect(rect)).unwrap();
    }
}

#[unsafe_destructor]
impl<'a> Drop for CanvasRenderingContext2D<'a> {
    fn drop(&mut self) {
        self.renderer.send(Close).unwrap();
    }
}
