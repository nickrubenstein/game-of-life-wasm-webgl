use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

use crate::renderer::Renderer;
use crate::universe::Universe;
// use crate::utils;


const FRAME_DURATION_MAX: usize = 10;

pub struct RenderLoop {
    window: web_sys::Window,
    playpause_button: web_sys::HtmlButtonElement,
    fps_label: web_sys::HtmlDivElement,
    universe: Rc<RefCell<Universe>>,
    renderer: Rc<RefCell<Renderer>>,
    ticks_per_frame: usize,
    animation_id: Option<i32>,
    then: f64,
    render_interval: f64,
    frame_durations: Vec<f64>,
    pub closure: Option<Closure<dyn Fn(f64)>>,
}

impl RenderLoop {
    pub fn new(
        window: web_sys::Window,
        playpause_button: web_sys::HtmlButtonElement,
        fps_label: web_sys::HtmlDivElement,
        universe: Rc<RefCell<Universe>>,
        renderer: Rc<RefCell<Renderer>>,
    ) -> RenderLoop {
        RenderLoop {
            window,
            playpause_button,
            fps_label,
            universe,
            renderer,
            ticks_per_frame: 1,
            then: f64::NEG_INFINITY,
            render_interval: 1.0,
            frame_durations: Vec::new(),
            animation_id: None,
            closure: None,
        }
    }

    pub fn render_loop(&mut self, now: f64) -> () {
        let elapsed = now - self.then;
        if elapsed > self.render_interval {
            self.add_frame_duration(elapsed);
            for _ in 0..self.ticks_per_frame {
                self.universe.borrow_mut().tick();
            }
            self.renderer.borrow().draw();
            self.then = now;
        }
        
        self.animation_id = if let Some(ref closure) = self.closure {
            Some(
                self.window
                    .request_animation_frame(closure.as_ref().unchecked_ref())
                    .expect("cannot set animation frame"),
            )
        } else {
            None
        }
    }

    pub fn is_paused(&self) -> bool {
        self.animation_id.is_none()
    }

    pub fn play(&mut self) -> Result<(), JsValue> {
        (self.playpause_button.as_ref() as &web_sys::Node).set_text_content(Some("⏸"));
        self.render_loop(0.0);
        Ok(())
    }

    pub fn pause(&mut self) -> Result<(), JsValue> {
        (self.playpause_button.as_ref() as &web_sys::Node).set_text_content(Some("▶"));
        if let Some(id) = self.animation_id {
            self.window.cancel_animation_frame(id)?;
            self.animation_id = None;
        }
        Ok(())
    }

    pub fn play_pause(&mut self) -> Result<(), JsValue> {
        if self.is_paused() {
            self.play()?;
        } else {
            self.pause()?;
        }
        Ok(())
    }

    pub fn set_ticks_per_frame(&mut self, ticks_per_frame: usize) {
        self.ticks_per_frame = ticks_per_frame;
    }

    pub fn set_render_interval(&mut self, render_interval: f64) {
        self.render_interval = render_interval;
        self.frame_durations.clear();
    }

    fn add_frame_duration(&mut self, elapsed: f64) {
        self.frame_durations.push(elapsed);
        if self.frame_durations.len() > FRAME_DURATION_MAX {
            self.update_fps_label();
        }
    }

    fn update_fps_label(&mut self) {
        let avg = self.frame_durations.iter().sum::<f64>() / self.frame_durations.len() as f64;
        let string = format!("FPS {}", (1000.0 / avg).round() as usize);
        (self.fps_label.as_ref() as &web_sys::Node).set_text_content(Some(&string));
        self.frame_durations.clear();
    }
}