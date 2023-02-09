use std::cell::RefCell;
use std::rc::Rc;

use js_sys::WebAssembly;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
// use crate::utils;
use crate::universe::Universe;

pub struct Renderer {
    canvas: web_sys::HtmlCanvasElement,
    ctx: web_sys::WebGl2RenderingContext,
    universe: Rc<RefCell<Universe>>,
    view_scale: f64,
    view_position: (i32, i32),
    view_start_position: Option<(i32, i32)>,
    cell_program: web_sys::WebGlProgram,
    bg_program: web_sys::WebGlProgram,
    cell_position_loc: u32,
    bg_position_loc: u32,
    point_size_loc: web_sys::WebGlUniformLocation,
    universe_width_loc: web_sys::WebGlUniformLocation,
    universe_height_loc: web_sys::WebGlUniformLocation,
    universe_width_offset_loc: web_sys::WebGlUniformLocation,
    universe_height_offset_loc: web_sys::WebGlUniformLocation
}

const CELL_SIZE: f32 = 10.0;
const CANVAS_COLOR: [f32; 4] = [0.2, 0.2, 0.2, 1.0];
// const UNIVERSE_COLOR: [f32; 4] = [0.3, 0.3, 0.3, 1.0];
// const ALIVE_COLOR: [f32; 4] = [0.8, 0.8, 0.8, 1.0];

impl Renderer {
    pub fn new(
        canvas: web_sys::HtmlCanvasElement,
        universe: Rc<RefCell<Universe>>,
    ) -> Result<Renderer, JsValue> {

        let ctx: web_sys::WebGl2RenderingContext = canvas.get_context("webgl2")?
            .expect("canvas should have a webgl2")
            .dyn_into::<web_sys::WebGl2RenderingContext>()?;

        let vert_cell_shader = Renderer::compile_shader(
            &ctx,
            web_sys::WebGl2RenderingContext::VERTEX_SHADER,
            r##"#version 300 es
    
            in vec2 position;
            uniform float universeWidth;
            uniform float universeHeight;
            uniform float universeWidthOffset;
            uniform float universeHeightOffset;
            uniform float pointSize;

            void main() {

                float x = 2.0 * ((position[0] / universeWidth) - 0.5) + universeWidthOffset;
                float y = 2.0 * ((position[1] / universeHeight) - 0.5) + universeHeightOffset;

                gl_Position = vec4(y, -x, 0.0, 1.0);
                gl_PointSize = pointSize;
            }
            "##,
        ).expect("failed to compile vert cell shader");

        let vert_bg_shader = Renderer::compile_shader(
            &ctx,
            web_sys::WebGl2RenderingContext::VERTEX_SHADER,
            r##"#version 300 es

            in vec2 position;

            void main() {
                gl_Position = vec4(position, 0.0, 1.0);
            }
            "##,
        ).expect("failed to compile vert bg shader");

        let frag_cell_shader = Renderer::compile_shader(
            &ctx,
            web_sys::WebGl2RenderingContext::FRAGMENT_SHADER,
            r##"#version 300 es
        
            precision lowp float;
            out vec4 outColor;
            void main() {
                outColor = vec4(0.8, 0.8, 0.8, 1.0);
            }
            "##,
        ).expect("falied to compile frag shader");

        let frag_bg_shader = Renderer::compile_shader(
            &ctx,
            web_sys::WebGl2RenderingContext::FRAGMENT_SHADER,
            r##"#version 300 es
        
            precision lowp float;
            out vec4 outColor;
            void main() {
                outColor = vec4(0.3, 0.3, 0.3, 1.0);
            }
            "##,
        ).expect("falied to compile frag shader");
    
        let cell_program = Renderer::link_program(&ctx, &vert_cell_shader, &frag_cell_shader).expect("Couldnt link shaders to cell_program");
        let cell_position_loc = ctx.get_attrib_location(&cell_program, "position") as u32;
        let point_size_loc = ctx.get_uniform_location(&cell_program, "pointSize").unwrap();
        let universe_width_loc = ctx.get_uniform_location(&cell_program, "universeWidth").unwrap();
        let universe_height_loc = ctx.get_uniform_location(&cell_program, "universeHeight").unwrap();
        let universe_width_offset_loc = ctx.get_uniform_location(&cell_program, "universeWidthOffset").unwrap();
        let universe_height_offset_loc = ctx.get_uniform_location(&cell_program, "universeHeightOffset").unwrap();

        let bg_program = Renderer::link_program(&ctx, &vert_bg_shader, &frag_bg_shader).expect("Couldnt link shaders to bg_program");
        let bg_position_loc = ctx.get_attrib_location(&bg_program, "position") as u32;

        ctx.use_program(Some(&bg_program));
        Renderer::init_background(&ctx).unwrap();

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
            cell_program,
            bg_program,
            cell_position_loc,
            bg_position_loc,
            point_size_loc,
            universe_width_loc,
            universe_height_loc,
            universe_width_offset_loc,
            universe_height_offset_loc
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

    fn init_background(context: &web_sys::WebGl2RenderingContext) -> Result<(), JsValue> {
        let vertices: [f32; 8] = [
            -1.0, 1.0, 
            -1.0, -1.0, 
            1.0, 1.0,
            1.0, -1.0
        ];
        let vertex_array = {
            let memory_buffer = wasm_bindgen::memory()
                .dyn_into::<WebAssembly::Memory>()?
                .buffer();
            let vertices_location = vertices.as_ptr() as u32 / 4;
            js_sys::Float32Array::new(&memory_buffer)
                .subarray(vertices_location, vertices_location + vertices.len() as u32)
        };
        let vertex_buffer: web_sys::WebGlBuffer = match context.create_buffer() {
            Some(buffer) => buffer,
            None => return Err(JsValue::from_str("Failed to create the buffer object"))
        };
        context.bind_buffer(web_sys::WebGl2RenderingContext::ARRAY_BUFFER, Some(&vertex_buffer));
        context.buffer_data_with_array_buffer_view(web_sys::WebGl2RenderingContext::ARRAY_BUFFER, &vertex_array, web_sys::WebGl2RenderingContext::STATIC_DRAW);
        Ok(())
    }

    pub fn draw(&self) {
        // let _timer = utils::Timer::new("Renderer::draw");
        let canvas_width = self.canvas.client_width() as i32;
        let canvas_height = self.canvas.client_height() as i32 - 1;
        self.canvas.set_width(canvas_width as u32);
        self.canvas.set_height(canvas_height as u32);
        // log!("({}, {})", canvas.width(), canvas.height());
        let canvas_small_side = canvas_width.min(canvas_height);
        let view_scale = ((canvas_small_side as f64) * self.view_scale) as i32;

        let viewport_offset_x = (canvas_width - view_scale) / 2 - self.view_position.0;
        let viewport_offset_y = (canvas_height - view_scale) / 2 + self.view_position.1;
        self.ctx.viewport(viewport_offset_x, viewport_offset_y, view_scale, view_scale);

        self.ctx.clear_color(CANVAS_COLOR[0], CANVAS_COLOR[1], CANVAS_COLOR[2], CANVAS_COLOR[3]);
        self.ctx.clear(web_sys::WebGl2RenderingContext::COLOR_BUFFER_BIT);

        self.draw_background();
        self.draw_cells(view_scale as f32);
    }

    pub fn reset_view(&mut self) {
        self.view_scale = 1.0;
        self.view_position = (0, 0);
        self.view_start_position = None;
    }

    pub fn start_position(&mut self, x: i32, y: i32) {
        self.view_start_position = Some((self.view_position.0 + x, self.view_position.1 + y));
        // utils::log!("start ({},{})", self.view_position.0, self.view_position.1);
    }

    pub fn set_position(&mut self, x: i32, y: i32) {
        if self.has_start_position() {
            let start = self.view_start_position.unwrap();
            self.view_position.0 = start.0 - x;
            self.view_position.1 = start.1 - y;
            // utils::log!("set   ({},{})", self.view_position.0, self.view_position.1);
        }
    }

    pub fn end_position(&mut self, x: i32, y: i32) {
        self.set_position(x, y);
        self.view_start_position = None;
        // utils::log!("end   ({},{})", self.view_position.0, self.view_position.1);
    }

    pub fn has_start_position(&self) -> bool {
        self.view_start_position.is_some()
    }

    pub fn set_view_scale(&mut self, scale: f64) {
        self.view_scale = f64::max(scale, 0.1);
    }

    pub fn set_view_scale_delta(&mut self, scale_delta: f64) {
        self.view_scale = f64::max(self.view_scale * scale_delta, 0.1);
    }

    pub fn get_view_position(&self) -> (i32, i32) {
        self.view_position
    }

    pub fn get_view_scale(&self) -> f64 {
        self.view_scale
    }

    fn draw_background(&self) {
        self.ctx.use_program(Some(&self.bg_program));
        self.ctx.enable_vertex_attrib_array(self.bg_position_loc);
        self.ctx.vertex_attrib_pointer_with_i32(self.bg_position_loc, 2, web_sys::WebGl2RenderingContext::FLOAT, false, 0, 0);

        self.ctx.draw_arrays(web_sys::WebGl2RenderingContext::TRIANGLE_STRIP, 0, 4);

        self.ctx.disable_vertex_attrib_array(self.bg_position_loc);
    }

    fn draw_cells(&self, size: f32) {
        self.ctx.use_program(Some(&self.cell_program));
        let universe = self.universe.borrow();
        let universe_width = universe.width() as f32;
        let universe_height = universe.height() as f32;
        let cell_size = size / (universe_width.min(universe_height) + CELL_SIZE);
        self.ctx.uniform1f(Some(&self.point_size_loc), cell_size);
        self.ctx.uniform1f(Some(&self.universe_width_loc), universe_width);
        self.ctx.uniform1f(Some(&self.universe_height_loc), universe_height);
        self.ctx.uniform1f(Some(&self.universe_width_offset_loc),  1.0 / universe_width);
        self.ctx.uniform1f(Some(&self.universe_height_offset_loc), 1.0 / universe_height);
        let live_cells = universe.get_live_cells();
        for (row, col) in live_cells {
            self.ctx.vertex_attrib2f(self.cell_position_loc, *row, *col);
            self.ctx.draw_arrays(web_sys::WebGl2RenderingContext::POINTS, 0, 1);
        }
    }
}
