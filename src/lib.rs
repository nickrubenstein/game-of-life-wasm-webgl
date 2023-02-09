use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

use std::cell::RefCell;
use std::rc::Rc;

mod renderer;
mod renderloop;
mod universe;
mod utils;

use renderer::Renderer;
use renderloop::RenderLoop;
use universe::Universe;

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    let window = window();
    let canvas = canvas();
    let universe: Rc<RefCell<Universe>> = Rc::new(RefCell::new(Universe::new(100, 100)));
    let renderer: Rc<RefCell<Renderer>> = Rc::new(RefCell::new(Renderer::new(
        canvas.clone(),
        universe.clone(),
    )?));

    // universe size apply button listener
    { 
        let universe_apply_btn = universe_apply_btn();
        let closure: Closure<dyn Fn() -> _> = {
            let universe = universe.clone();
            let renderer = renderer.clone();
            Closure::wrap(Box::new(move || -> Result<(), JsValue> {
                let row = row_input().value().parse::<usize>();
                // let col = col_input().value().parse::<usize>();
                if let Ok(r) = row /*&& let Ok(c) = col*/ {
                    universe.borrow_mut().set_size(Some(r), Some(r)/*Some(c)*/);
                    renderer.borrow().draw();
                }
                else {
                    utils::log!("Could not parse row or col");
                    utils::log!("row: {:?}", row);
                    // utils::log!("col: {:?}", col);
                }
                Ok(())
            }))
        };
        universe_apply_btn.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }

    // view reset apply button listener
    { 
        let view_apply_btn = view_apply_btn();
        let closure: Closure<dyn Fn() -> _> = {
            let renderer = renderer.clone();
            Closure::wrap(Box::new(move || -> Result<(), JsValue> {
                {
                    renderer.borrow_mut().reset_view();
                    view_scale_input().set_value("100");
                    renderer.borrow().draw();
                }
                Ok(())
            }))
        };
        view_apply_btn.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }

    // view scale input change listener
    { 
        let vsi = view_scale_input();
        let closure: Closure<dyn Fn() -> _> = {
            let renderer = renderer.clone();
            Closure::wrap(Box::new(move || -> Result<(), JsValue> {
                {
                    let scale = view_scale_input().value().parse::<f64>();
                    if let Ok(s) = scale {
                        renderer.borrow_mut().set_view_scale(s / 100.0);
                        renderer.borrow().draw();
                    }
                    else {
                        utils::log!("Could not parse zoom value");
                    }
                }
                Ok(())
            }))
        };
        vsi.add_event_listener_with_callback("change", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }

    // Mouse mousedown handler on canvas
    {
        let closure: Closure<dyn Fn(_)> = {
            let renderer = renderer.clone();
            Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
                let client_x = event.client_x();
                let client_y = event.client_y();
                renderer.borrow_mut().start_position(client_x, client_y);
            }))
        };
        (canvas.as_ref() as &web_sys::EventTarget)
            .add_event_listener_with_callback("mousedown", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }

    // Mouse move handler on canvas
    {
        let closure: Closure<dyn Fn(_)> = {
            let renderer = renderer.clone();
            Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
                {
                    if renderer.borrow().has_start_position() {
                        let client_x = event.client_x();
                        let client_y = event.client_y();
                        renderer.borrow_mut().set_position(client_x, client_y); 
                        renderer.borrow().draw();
                    }
                }
            }))
        };
        (canvas.as_ref() as &web_sys::EventTarget)
            .add_event_listener_with_callback("mousemove", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }

    // Mouse mouseup handler on canvas
    {
        let closure: Closure<dyn Fn(_)> = {
            let canvas = canvas.clone();
            let universe = universe.clone();
            let renderer = renderer.clone();
            Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
                {
                    let client_x = event.client_x();
                    let client_y = event.client_y();
                    renderer.borrow_mut().end_position(client_x, client_y);

                    let mut universe = universe.borrow_mut();
                    let w = universe.width() as isize;
                    let h = universe.height() as isize;
                    let view_offset = renderer.borrow().get_view_position();
                    let view_scale = renderer.borrow().get_view_scale();
                    let bounding_rect = (canvas.as_ref() as &web_sys::Element).get_bounding_client_rect();
                    let canvas_small_side = canvas.width().min(canvas.height()) as f64 * view_scale;
                    let viewport_offset_x = (canvas.width() as f64 - canvas_small_side) / 2.0 - view_offset.0 as f64;
                    let viewport_offset_y = (canvas.height() as f64 - canvas_small_side) / 2.0 - view_offset.1 as f64;
                    let rect_small_side = bounding_rect.width().min(bounding_rect.height());
                    let scale_x = w as f64 / rect_small_side / view_scale;
                    let scale_y = h as f64 / rect_small_side / view_scale;
                    let x = ((client_x as f64 - bounding_rect.left() - viewport_offset_x) * scale_x) as isize;
                    let y = ((client_y as f64 - bounding_rect.top() - viewport_offset_y) * scale_y) as isize;

                    let row = in_bounds(y, h);
                    let col = in_bounds(x, w);
                    if event.ctrl_key() {
                        if event.alt_key() {
                            let mut cells = Vec::new();
                            for r in 0..w {
                                cells.push((r as usize, col));
                            }
                            universe.toggle_cells(&cells);
                        }
                        else {
                            universe.toggle_cells(&[(row,                 col),
                                                       (row,                 in_bounds(x + 1, w)),
                                                       (in_bounds(y - 1, h), col),
                                                       (in_bounds(y - 1, h), in_bounds(x - 1, w)),
                                                       (in_bounds(y + 1, h), in_bounds(x - 1, w))]);
                        }
                    }
                    else {
                        universe.toggle_cell(row, col);
                    }
                }
                renderer.borrow().draw();
            }))
        };
        (canvas.as_ref() as &web_sys::EventTarget)
            .add_event_listener_with_callback("mouseup", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }

    // Mouse scroll handler on canvas
    {
        let closure: Closure<dyn Fn(_)> = {
            let canvas = canvas.clone();
            let renderer = renderer.clone();
            Closure::wrap(Box::new(move |event: web_sys::WheelEvent| {
                let old_scale = renderer.borrow().get_view_scale();
                if event.delta_y() >= 0.0 {
                    renderer.borrow_mut().set_view_scale_delta(1.1);
                }
                else {
                    renderer.borrow_mut().set_view_scale_delta(0.9);
                }
                let scale = renderer.borrow().get_view_scale();
                view_scale_input().set_value(&format!("{:.0}", scale * 100.0));
                let client_x = event.client_x() as f64;
                let client_y = event.client_y() as f64;
                let direction_x = (canvas.width() as f64 / 2.0) - client_x;
                let direction_y = (canvas.height() as f64 / 2.0) - client_y;
                renderer.borrow_mut().start_position((direction_x * old_scale) as i32, (direction_y * old_scale) as i32);
                renderer.borrow_mut().end_position((direction_x * scale) as i32, (direction_y * scale) as i32); 
                renderer.borrow().draw();
            }))
        };
        (canvas.as_ref() as &web_sys::EventTarget)
            .add_event_listener_with_callback("wheel", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }

    let play_pause_btn = play_pause_btn();
    let fps_label = fps_label();
    
    // Render loop handling
    let render_loop: Rc<RefCell<RenderLoop>> = Rc::new(RefCell::new(RenderLoop::new(
        window.clone(),
        play_pause_btn.clone(),
        fps_label.clone(),
        universe.clone(),
        renderer,
    )));
    render_loop.borrow_mut().closure = Some({
        let render_loop = render_loop.clone();
        Closure::wrap(Box::new(move |time: f64| {
            // let _timer = utils::Timer::new("Lib::render_loop");
            render_loop.borrow_mut().render_loop(time);
        }))
    });

    // play pause button listener
    { 
        let closure: Closure<dyn Fn() -> _> = {
            let render_loop = render_loop.clone();
            Closure::wrap(Box::new(move || -> Result<(), JsValue> {
                render_loop.borrow_mut().play_pause()?;
                Ok(())
            }))
        };
        play_pause_btn.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }

    // fps input listener
    {
        let fps_input = fps_input();
        let closure: Closure<dyn Fn(web_sys::Event) -> _> = {
            let render_loop = render_loop.clone();
            Closure::wrap(Box::new(move |e: web_sys::Event| -> Result<(), JsValue> {
                let input = e
                    .current_target()
                    .unwrap()
                    .dyn_into::<web_sys::HtmlInputElement>()
                    .unwrap()
                    .value()
                    .parse::<f64>();
                if let Ok(value) = input {
                    let value = 1000.0 / f64::powf(10.0, value);
                    // utils::log!("{}", value);
                    render_loop.borrow_mut().set_render_interval(value);
                }
                Ok(())
            }))
        };
        fps_input.add_event_listener_with_callback("input", &closure.as_ref().unchecked_ref())?;
        closure.forget();
    }

    // tpf input listener
    {
        let tpf_input = tpf_input();
        let closure: Closure<dyn Fn(web_sys::Event) -> _> = {
            let render_loop = render_loop.clone();
            Closure::wrap(Box::new(move |e: web_sys::Event| -> Result<(), JsValue> {
                let input = e
                    .current_target()
                    .unwrap()
                    .dyn_into::<web_sys::HtmlInputElement>()
                    .unwrap()
                    .value()
                    .parse::<f64>();
                if let Ok(value) = input {
                    // utils::log!("{}", value);
                    render_loop.borrow_mut().set_ticks_per_frame(value as usize);
                }
                Ok(())
            }))
        };
        tpf_input.add_event_listener_with_callback("input", &closure.as_ref().unchecked_ref())?;
        closure.forget();
    }

    // utils::log!("end of lib start");
    render_loop.borrow_mut().play()?;
    Ok(())
}

fn in_bounds(x: isize, cap: isize) -> usize {
    if x < 0 {
        0
    }
    else if x >= cap {
        (cap - 1) as usize
    }
    else {
        x as usize
    }
}

fn window() -> web_sys::Window {
    web_sys::window().expect("no global `window` exists")
}

fn document() -> web_sys::Document {
    window().document().expect("should have a document on window")
}

fn canvas() -> web_sys::HtmlCanvasElement {
    let canvas = document().get_element_by_id("canvas").expect("document should have a canvas");
    canvas.dyn_into::<web_sys::HtmlCanvasElement>().expect("dyn_into for canvas failed")
}

fn row_input() -> web_sys::HtmlInputElement {
    let btn = document().get_element_by_id("row-input").expect("document should have a row input");
    btn.dyn_into::<web_sys::HtmlInputElement>().expect("dyn_into for row input failed")
}

// fn col_input() -> web_sys::HtmlInputElement {
//     let btn = document().get_element_by_id("col-input").expect("document should have a col input");
//     btn.dyn_into::<web_sys::HtmlInputElement>().expect("dyn_into for col input failed")
// }

fn universe_apply_btn() -> web_sys::HtmlButtonElement {
    let btn = document().get_element_by_id("universe-apply").expect("document should have a universe-apply button");
    btn.dyn_into::<web_sys::HtmlButtonElement>().expect("dyn_into for universe-apply button failed")
}

fn view_scale_input() -> web_sys::HtmlInputElement {
    let btn = document().get_element_by_id("scale-input").expect("document should have a scale input");
    btn.dyn_into::<web_sys::HtmlInputElement>().expect("dyn_into for scale input failed")
}

fn view_apply_btn() -> web_sys::HtmlButtonElement {
    let btn = document().get_element_by_id("view-apply").expect("document should have a view-apply button");
    btn.dyn_into::<web_sys::HtmlButtonElement>().expect("dyn_into for view-apply button failed")
}

fn play_pause_btn() -> web_sys::HtmlButtonElement {
    let btn = document().get_element_by_id("play-pause").expect("document should have a play-pause button");
    btn.dyn_into::<web_sys::HtmlButtonElement>().expect("dyn_into for play pause button failed")
}

fn fps_input() -> web_sys::HtmlInputElement {
    let btn = document().get_element_by_id("fps-range").expect("document should have a fps-range input");
    btn.dyn_into::<web_sys::HtmlInputElement>().expect("dyn_into for fps-range input failed")
}

fn fps_label() -> web_sys::HtmlDivElement {
    let btn = document().get_element_by_id("fps-label").expect("document should have a fps-label div");
    btn.dyn_into::<web_sys::HtmlDivElement>().expect("dyn_into for fps-label div failed")
}

fn tpf_input() -> web_sys::HtmlInputElement {
    let btn = document().get_element_by_id("tpf-range").expect("document should have a tpf-range input");
    btn.dyn_into::<web_sys::HtmlInputElement>().expect("dyn_into for tpf-range input failed")
}
