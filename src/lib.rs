use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::prelude::*;
mod math;

use math::Point;

// When the `wee_alloc` feature is enabled, this uses `wee_alloc` as the global
// allocator.
//
// If you don't want to use `wee_alloc`, you can safely delete this.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[derive(Default)]
struct State {
    mouse_position: Option<Point<[f64; 2]>>,
    polylines: Vec<Polyline>,
}

#[derive(Default)]
struct Polyline {
    vertices: Vec<Point<[f64; 2]>>,
}

// This is like the `main` function, except for JavaScript.
#[wasm_bindgen(start)]
pub fn main_js() -> Result<(), JsValue> {
    // This provides better error messages in debug mode.
    // It's disabled in release mode so it doesn't bloat up the file size.
    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();

    let window = web_sys::window().expect("global window must be available");
    let document = window.document().expect("window should have a document");
    let canvas = document
        .get_element_by_id("canvas")
        .unwrap()
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .unwrap();
    let canvas_parent = canvas
        .parent_element()
        .unwrap()
        .dyn_into::<web_sys::HtmlElement>()
        .unwrap();

    let state = Rc::new(RefCell::new(State::default()));

    canvas
        .add_event_listener_with_callback(
            "mouseup",
            Closure::<dyn FnMut(web_sys::MouseEvent)>::new({
                let state = Rc::clone(&state);
                move |event: web_sys::MouseEvent| {
                    let polylines = &mut state.borrow_mut().polylines;
                    if polylines.is_empty() {
                        polylines.push(Polyline::default());
                    }
                    let polyline = &mut polylines[0];
                    polyline
                        .vertices
                        .push(Point([event.client_x() as f64, event.client_y() as f64]));
                }
            })
            .into_js_value()
            .unchecked_ref(),
        )
        .unwrap();

    canvas
        .add_event_listener_with_callback(
            "mouseenter",
            Closure::<dyn FnMut(web_sys::MouseEvent)>::new({
                let state = Rc::clone(&state);
                move |event: web_sys::MouseEvent| {
                    state.borrow_mut().mouse_position =
                        Some(Point([event.client_x() as f64, event.client_y() as f64]))
                }
            })
            .into_js_value()
            .unchecked_ref(),
        )
        .unwrap();

    canvas
        .add_event_listener_with_callback(
            "mousemove",
            Closure::<dyn FnMut(web_sys::MouseEvent)>::new({
                let state = Rc::clone(&state);
                move |event: web_sys::MouseEvent| {
                    if let Some(mouse_position) = state.borrow_mut().mouse_position.as_mut() {
                        mouse_position[0] = event.client_x() as f64;
                        mouse_position[1] = event.client_y() as f64;
                    }
                }
            })
            .into_js_value()
            .unchecked_ref(),
        )
        .unwrap();

    canvas
        .add_event_listener_with_callback(
            "mouseleave",
            Closure::<dyn FnMut(web_sys::MouseEvent)>::new({
                let state = Rc::clone(&state);
                move |_| {
                    state.borrow_mut().mouse_position = None;
                }
            })
            .into_js_value()
            .unchecked_ref(),
        )
        .unwrap();

    let rendering_context = canvas
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into::<web_sys::CanvasRenderingContext2d>()
        .unwrap();

    let context = Context {
        window,
        canvas,
        canvas_parent,
        rendering_context,
        state,
    };

    context
        .window
        .clone()
        .request_animation_frame(
            Closure::once(move || animation_frame_callback(context))
                .into_js_value()
                .unchecked_ref(),
        )
        .unwrap();

    Ok(())
}

#[derive(Clone)]
struct Context {
    window: web_sys::Window,
    canvas_parent: web_sys::HtmlElement,
    canvas: web_sys::HtmlCanvasElement,
    rendering_context: web_sys::CanvasRenderingContext2d,
    state: Rc<RefCell<State>>,
}

fn animation_frame_callback(context: Context) {
    let dimensions = Point([
        context.canvas_parent.client_width(),
        context.canvas_parent.client_height(),
    ]);

    for (index, &name) in ["width", "height"].iter().enumerate() {
        if context
            .canvas
            .get_attribute(name)
            .map(|x| x.parse::<i32>().unwrap())
            != Some(dimensions[index])
        {
            context
                .canvas
                .set_attribute(name, &dimensions[index].to_string())
                .unwrap();
        }
    }

    let rendering_context = &context.rendering_context;
    let state = context.state.borrow();

    // Compute offset line segments.
    let offset_polylines = state
        .polylines
        .iter()
        .map(|polyline| {
            let line_segments = polyline
                .vertices
                .windows(2)
                .map(|x| {
                    let a = x[0];
                    let b = x[1];
                    let normal = Point::normal(a, b);
                    let offset = normal * 50.0;
                    [a + offset, b + offset]
                })
                .collect::<Vec<_>>();

            let mut vertices = line_segments
                .windows(2)
                .filter_map(|line_segments| {
                    const X: usize = 0;
                    const Y: usize = 1;

                    let [p0, p1] = line_segments[0];
                    let [q0, q1] = line_segments[1];

                    let p0p1 = p1 - p0;
                    let q0q1 = q1 - q0;
                    let q0p0 = p0 - q0;

                    let d = q0q1[X] * p0p1[Y] - p0p1[X] * q0q1[Y];

                    if d.abs() < f64::EPSILON {
                        // Line segments are parallel, because of how these segments are constructed,
                        // this means that p1 should equal q1.
                        None
                    } else {
                        let t = (q0p0[X] * q0q1[Y] - q0q1[X] * q0p0[Y]) / d;
                        Some(p0 + p0p1 * t)
                    }
                })
                .collect::<Vec<_>>();

            if !line_segments.is_empty() {
                vertices.insert(0, line_segments.first().unwrap()[0]);
                vertices.push(line_segments.last().unwrap()[1]);
            }

            vertices
        })
        .collect::<Vec<_>>();

    rendering_context.clear_rect(0.0, 0.0, dimensions[0] as f64, dimensions[1] as f64);

    // Draw finished polylines.
    for polyline in &state.polylines {
        draw_polyline(rendering_context, polyline.vertices.iter().copied());
    }

    // Draw offset line segments
    for polyline in offset_polylines.iter() {
        draw_offset_polyline(rendering_context, polyline.iter().copied())
    }

    // Draw to-be-drawn polyline segment.
    if let (Some(&a), Some(b)) = (
        state.polylines.first().and_then(|x| x.vertices.last()),
        state.mouse_position,
    ) {
        draw_highlighted_line_segment(rendering_context, a, b);
    }

    // Draw vertices
    for &vertex in state.polylines.iter().flat_map(|x| &x.vertices) {
        draw_vertex(rendering_context, vertex)
    }

    drop(state);

    context
        .window
        .clone()
        .request_animation_frame(
            Closure::once(move || animation_frame_callback(context))
                .into_js_value()
                .unchecked_ref(),
        )
        .unwrap();
}

fn draw_offset_line_segment(
    rendering_context: &web_sys::CanvasRenderingContext2d,
    a: Point<[f64; 2]>,
    b: Point<[f64; 2]>,
) {
    rendering_context.begin_path();
    rendering_context.move_to(a[0], a[1]);
    rendering_context.line_to(b[0], b[1]);
    rendering_context.set_stroke_style(&"#00ff00".into());
    rendering_context.stroke();
}

fn draw_highlighted_line_segment(
    rendering_context: &web_sys::CanvasRenderingContext2d,
    a: Point<[f64; 2]>,
    b: Point<[f64; 2]>,
) {
    rendering_context.begin_path();
    rendering_context.move_to(a[0], a[1]);
    rendering_context.line_to(b[0], b[1]);
    rendering_context.set_stroke_style(&"#ff0000".into());
    rendering_context.stroke();
}

fn draw_vertex(rendering_context: &web_sys::CanvasRenderingContext2d, a: Point<[f64; 2]>) {
    rendering_context.begin_path();
    rendering_context
        .ellipse(a[0], a[1], 5.0, 5.0, 0.0, 0.0, 360.0)
        .unwrap();
    rendering_context.set_fill_style(&"rgb(100, 100, 200)".into());
    rendering_context.fill();
}

fn draw_polyline(
    rendering_context: &web_sys::CanvasRenderingContext2d,
    vertices: impl IntoIterator<Item = Point<[f64; 2]>>,
) {
    let mut vertices = vertices.into_iter();
    if let Some(vertex) = vertices.next() {
        rendering_context.begin_path();
        rendering_context.move_to(vertex[0], vertex[1]);
        for vertex in vertices {
            rendering_context.line_to(vertex[0], vertex[1]);
        }
        rendering_context.set_stroke_style(&"#000000".into());
        rendering_context.stroke();
    }
}

fn draw_offset_polyline(
    rendering_context: &web_sys::CanvasRenderingContext2d,
    vertices: impl IntoIterator<Item = Point<[f64; 2]>>,
) {
    let mut vertices = vertices.into_iter();
    if let Some(vertex) = vertices.next() {
        rendering_context.begin_path();
        rendering_context.move_to(vertex[0], vertex[1]);
        for vertex in vertices {
            rendering_context.line_to(vertex[0], vertex[1]);
        }
        rendering_context.set_stroke_style(&"#00ff00".into());
        rendering_context.stroke();
    }
}
