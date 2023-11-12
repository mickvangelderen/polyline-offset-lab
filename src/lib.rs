use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::prelude::*;

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

struct Point<A>(A);

impl<T, const N: usize> std::ops::Index<usize> for Point<[T; N]> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl<T, const N: usize> std::ops::IndexMut<usize> for Point<[T; N]> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
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

    rendering_context.clear_rect(0.0, 0.0, dimensions[0] as f64, dimensions[1] as f64);

    // Draw finished polylines.
    rendering_context.begin_path();
    for polyline in &state.polylines {
        let mut vertices = polyline.vertices.iter();
        if let Some(vertex) = vertices.next() {
            rendering_context.move_to(vertex[0], vertex[1]);

            for vertex in vertices {
                rendering_context.line_to(vertex[0], vertex[1]);
            }
        }
    }
    rendering_context.set_stroke_style(&"#000000".into());
    rendering_context.stroke();
    rendering_context.close_path();

    // Draw to-be-drawn polyline segment.
    if let (Some(a), Some(b)) = (
        state.polylines.first().and_then(|x| x.vertices.last()),
        &state.mouse_position,
    ) {
        rendering_context.begin_path();
        rendering_context.move_to(a[0], a[1]);
        rendering_context.line_to(b[0], b[1]);
        rendering_context.set_stroke_style(&"#ff0000".into());
        rendering_context.stroke();
        rendering_context.close_path();
    }

    // Draw vertices
    for vertex in state.polylines.iter().flat_map(|x| &x.vertices) {
        rendering_context.begin_path();
        rendering_context
            .ellipse(vertex[0], vertex[1], 5.0, 5.0, 0.0, 0.0, 360.0)
            .unwrap();
        rendering_context.set_fill_style(&"rgb(100, 100, 200)".into());
        rendering_context.fill();
        rendering_context.close_path();
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
