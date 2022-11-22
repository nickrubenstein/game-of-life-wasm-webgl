use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

use crate::utils;
use crate::universe::Universe;

pub struct Renderer {
    canvas: web_sys::HtmlCanvasElement,
    ctx: web_sys::WebGl2RenderingContext,
    universe: Rc<RefCell<Universe>>,
    view_scale: f32,
    view_position: (i32, i32),
    view_start_position: Option<(i32, i32)>,
    position_loc: u32,
    point_size_loc: u32,
    color_loc: web_sys::WebGlUniformLocation
}

const CELL_SIZE: f32 = 10.0;
const CANVAS_COLOR: [f32; 4] = [0.2, 0.2, 0.2, 1.0];
const UNIVERSE_COLOR: [f32; 4] = [0.3, 0.3, 0.3, 1.0];
const ALIVE_COLOR: [f32; 4] = [0.8, 0.8, 0.8, 1.0];

impl Renderer {
    pub fn new(
        canvas: web_sys::HtmlCanvasElement,
        universe: Rc<RefCell<Universe>>,
    ) -> Result<Renderer, JsValue> {

        let ctx: web_sys::WebGl2RenderingContext = canvas.get_context("webgl2")?
            .expect("canvas should have a webgl2")
            .dyn_into::<web_sys::WebGl2RenderingContext>()?;

        let vert_shader = Renderer::compile_shader(
            &ctx,
            web_sys::WebGl2RenderingContext::VERTEX_SHADER,
            r##"#version 300 es
    
            in vec4 position;
            in float pointSize;

            void main() {
                gl_Position = position;
                gl_PointSize = pointSize;
            }
            "##,
        ).expect("failed to compile vert shader");

        let frag_shader = Renderer::compile_shader(
            &ctx,
            web_sys::WebGl2RenderingContext::FRAGMENT_SHADER,
            r##"#version 300 es
        
            precision highp float;
            uniform vec4 color;
            out vec4 outColor;
            void main() {
                outColor = color;
            }
            "##,
        ).expect("falied to compile frag shader");
    
        let program = Renderer::link_program(&ctx, &vert_shader, &frag_shader).expect("Couldnt link shaders to program");
        ctx.use_program(Some(&program));
    
        let position_loc = ctx.get_attrib_location(&program, "position") as u32;
        let point_size_loc = ctx.get_attrib_location(&program, "pointSize") as u32;
        let color_loc = ctx.get_uniform_location(&program, "color").unwrap();

        let view_scale = 1.0;
        let view_position = (0, 0);
        let view_start_position = None;

        Ok(Renderer {
            canvas,
            universe,
            ctx,
            view_scale,
            view_position,
            view_start_position,
            position_loc,
            point_size_loc,
            color_loc
        })
    }

    fn compile_shader(
        context: &web_sys::WebGl2RenderingContext,
        shader_type: u32,
        source: &str,
    ) -> Result<web_sys::WebGlShader, String> {
        let shader = context
            .create_shader(shader_type)
            .ok_or_else(|| String::from("Unable to create shader object"))?;
        context.shader_source(&shader, source);
        context.compile_shader(&shader);
    
        if context
            .get_shader_parameter(&shader, web_sys::WebGl2RenderingContext::COMPILE_STATUS)
            .as_bool()
            .unwrap_or(false)
        {
            Ok(shader)
        } else {
            Err(context
                .get_shader_info_log(&shader)
                .unwrap_or_else(|| String::from("Unknown error creating shader")))
        }
    }
    
    fn link_program(
        context: &web_sys::WebGl2RenderingContext,
        vert_shader: &web_sys::WebGlShader,
        frag_shader: &web_sys::WebGlShader,
    ) -> Result<web_sys::WebGlProgram, String> {
        let program = context
            .create_program()
            .ok_or_else(|| String::from("Unable to create shader object"))?;
    
        context.attach_shader(&program, vert_shader);
        context.attach_shader(&program, frag_shader);
        context.link_program(&program);
    
        if context
            .get_program_parameter(&program, web_sys::WebGl2RenderingContext::LINK_STATUS)
            .as_bool()
            .unwrap_or(false)
        {
            Ok(program)
        } else {
            Err(context
                .get_program_info_log(&program)
                .unwrap_or_else(|| String::from("Unknown error creating program object")))
        }
    }

    pub fn draw(&self) {
        // let _timer = utils::Timer::new("Renderer::draw");
        let canvas_width = self.canvas.client_width() as i32;
        let canvas_height = self.canvas.client_height() as i32 - 1;
        self.canvas.set_width(canvas_width as u32);
        self.canvas.set_height(canvas_height as u32);
        // log!("({}, {})", canvas.width(), canvas.height());
        let canvas_small_side = canvas_width.min(canvas_height);

        let viewport_offset_x = (canvas_width - canvas_small_side) / 2 - self.view_position.0;
        let viewport_offset_y = (canvas_height - canvas_small_side) / 2 + self.view_position.1;
        self.ctx.viewport(viewport_offset_x, viewport_offset_y, canvas_small_side, canvas_small_side);

        self.ctx.clear_color(CANVAS_COLOR[0], CANVAS_COLOR[1], CANVAS_COLOR[2], CANVAS_COLOR[3]);
        self.ctx.clear(web_sys::WebGl2RenderingContext::COLOR_BUFFER_BIT);

        self.draw_universe(canvas_small_side as f32);
        self.draw_cells(canvas_small_side as f32);
    }

    pub fn reset_view(&mut self) {
        self.view_scale = 1.0;
        self.view_position = (0, 0);
        self.view_start_position = None;
    }

    pub fn start_position(&mut self, x: i32, y: i32) {
        self.view_start_position = Some((self.view_position.0 + x, self.view_position.1 + y));
        // utils::log!("start ({},{})", x, y);
    }

    pub fn set_position(&mut self, x: i32, y: i32) {
        if self.has_start_position() {
            let start = self.view_start_position.unwrap();
            self.view_position.0 = start.0 - x;
            self.view_position.1 = start.1 - y;
            // utils::log!("end   ({},{})", self.view_position.0, self.view_position.1);
        }
    }

    pub fn end_position(&mut self, x: i32, y: i32) {
        self.set_position(x, y);
        self.view_start_position = None;
    }

    pub fn has_start_position(&self) -> bool {
        self.view_start_position.is_some()
    }

    pub fn set_view_scale(&mut self, scale_delta: f32) {
        utils::log!("scale {} to {}", self.view_scale, self.view_scale + scale_delta);
        self.view_scale += scale_delta;
    }

    fn draw_universe(&self, canvas_small_side: f32) {
        self.ctx.uniform4f(Some(&self.color_loc), UNIVERSE_COLOR[0], UNIVERSE_COLOR[1], UNIVERSE_COLOR[2], UNIVERSE_COLOR[3]);
        self.draw_point(0.0, 0.0, canvas_small_side);
    }

    fn draw_cells(&self, canvas_small_side: f32) {
        let universe = self.universe.borrow();
        let universe_width = universe.width() as f32;
        let universe_height = universe.height() as f32;
        let universe_width_offset = -1.0 + (1.0 / universe_width);
        let universe_height_offset = -1.0 + (1.0 / universe_height);
        let scale = canvas_small_side / (universe_width.min(universe_height) + CELL_SIZE);
        self.ctx.uniform4f(Some(&self.color_loc), ALIVE_COLOR[0], ALIVE_COLOR[1], ALIVE_COLOR[2], ALIVE_COLOR[3]);
        for row in 0..universe.width() {
            let row_u = row as f32 / universe_width; 
            let row_n = row_u * 2.0 + universe_width_offset;
            for col in 0..universe.height() {
                if !universe.get_cell(row, col) {
                    continue;
                }
                let col_u = col as f32 / universe_height; 
                let col_n = col_u * 2.0 + universe_height_offset;
                self.draw_point(
                    col_n,
                    -row_n, // invert so that row 0 is at the top of the canvas
                    scale
                );
            }
        }
    }

    fn draw_point(&self, x: f32, y: f32, scale: f32) {
        self.ctx.vertex_attrib2f(self.position_loc, x, y);
        self.ctx.vertex_attrib1f(self.point_size_loc, scale);
        self.ctx.draw_arrays(web_sys::WebGl2RenderingContext::POINTS, 0, 1);
    }
}
