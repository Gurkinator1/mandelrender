#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![allow(unsafe_code)]

use eframe::{
    egui::{self, Vec2},
    egui_glow,
    glow::{self, ARRAY_BUFFER, ELEMENT_ARRAY_BUFFER, FLOAT, STATIC_DRAW, TRIANGLES, UNSIGNED_BYTE},
};

use egui::mutex::Mutex;
use std::{mem::size_of, sync::Arc};

fn main() -> Result<(), eframe::Error> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([350.0, 380.0]),
        multisampling: 4,
        renderer: eframe::Renderer::Glow,
        ..Default::default()
    };
    eframe::run_native(
        "Custom 3D painting in eframe using glow",
        options,
        Box::new(|cc| Box::new(MyApp::new(cc))),
    )
}

struct MyApp {
    /// Behind an `Arc<Mutex<…>>` so we can pass it to [`egui::PaintCallback`] and paint later.
    rotating_triangle: Arc<Mutex<RotatingTriangle>>,
    delta: Vec2,
    zoom: u32,
}

impl MyApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let gl = cc
            .gl
            .as_ref()
            .expect("You need to run eframe with the glow backend");
        Self {
            rotating_triangle: Arc::new(Mutex::new(RotatingTriangle::new(gl))),
            delta: Vec2::ZERO,
            zoom: 1
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                ui.label("The triangle is being painted using ");
                ui.hyperlink_to("glow", "https://github.com/grovesNL/glow");
                ui.label(" (OpenGL).");
            });

            egui::Frame::canvas(ui.style()).show(ui, |ui| {
                self.custom_painting(ui);
            });
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 10.0;
                if ui.button("smaller").clicked() {
                    self.zoom += 1;
                }

                if ui.button("larger").clicked() {
                    self.zoom += 1;
                }
            });
        });
    }

    fn on_exit(&mut self, gl: Option<&glow::Context>) {
        if let Some(gl) = gl {
            self.rotating_triangle.lock().destroy(gl);
        }
    }
}

impl MyApp {
    fn custom_painting(&mut self, ui: &mut egui::Ui) {
        let (rect, response) =
            ui.allocate_exact_size(egui::Vec2::splat(1000.0), egui::Sense::click_and_drag());

        self.delta += response.drag_delta() * 0.01;

        if response.clicked_by(egui::PointerButton::Primary) {
            self.zoom += 1;
        }
        else if response.clicked_by(egui::PointerButton::Secondary) {
            self.zoom -= 1;
        }
        
        
        // Clone locals so we can move them into the paint callback:
        let delta = self.delta;
        let zoom = self.zoom;
        let rotating_triangle = self.rotating_triangle.clone();

        let callback = egui::PaintCallback {
            rect,
            callback: std::sync::Arc::new(egui_glow::CallbackFn::new(move |_info, painter| {
                rotating_triangle.lock().paint(painter.gl(), delta, zoom);
            })),
        };
        ui.painter().add(callback);
    }
}

struct RotatingTriangle {
    program: glow::Program,
    vertex_array: glow::VertexArray,
}

impl RotatingTriangle {
    fn new(gl: &glow::Context) -> Self {
        use glow::HasContext as _;

        let shader_version = if cfg!(target_arch = "wasm32") {
            "#version 300 es"
        } else {
            "#version 330"
        };

        unsafe {
            let program = gl.create_program().expect("Cannot create program");

            let shader_sources = [
                (glow::VERTEX_SHADER, include_str!("./vertex_shader.vert")),
                (
                    glow::FRAGMENT_SHADER,
                    include_str!("./fragment_shader.frag"),
                ),
            ];

            let shaders: Vec<_> = shader_sources
                .iter()
                .map(|(shader_type, shader_source)| {
                    let shader = gl
                        .create_shader(*shader_type)
                        .expect("Cannot create shader");
                    gl.shader_source(shader, &format!("{shader_version}\n{shader_source}"));
                    gl.compile_shader(shader);
                    assert!(
                        gl.get_shader_compile_status(shader),
                        "Failed to compile {shader_type}: {}",
                        gl.get_shader_info_log(shader)
                    );
                    gl.attach_shader(program, shader);
                    shader
                })
                .collect();

            gl.link_program(program);
            assert!(
                gl.get_program_link_status(program),
                "{}",
                gl.get_program_info_log(program)
            );

            for shader in shaders {
                gl.detach_shader(program, shader);
                gl.delete_shader(shader);
            }

            //create vertex buffer
            let vao = gl
                .create_vertex_array()
                .expect("cannot create vertex arrray");
            gl.bind_vertex_array(Some(vao));

            //create & copy vertices data to buffer
            let vertices: Vec<f32> = vec![
                -1.0, -1.0,
                1.0, -1.0,
                1.0, 1.0,
                -1.0, 1.0,
            ];

            //"scale" vertices
            //for v in vertices.iter_mut() {
            //    *v *= 0.75;
            //}

            let buf = gl.create_buffer().expect("cannot create buffer");
            gl.bind_buffer(ARRAY_BUFFER, Some(buf));
            gl.buffer_data_u8_slice(
                ARRAY_BUFFER,
                std::slice::from_raw_parts(
                    vertices.as_ptr() as *const u8,
                    vertices.len() * size_of::<f32>(),
                ),
                STATIC_DRAW,
            );

            //buffer layout
            gl.enable_vertex_attrib_array(0);
            gl.vertex_attrib_pointer_f32(0, 2, FLOAT, false, 2 * size_of::<f32>() as i32, 0);
            
            //indices buffer
            let indices: Vec<u8> = vec![0,1,2,2,3,0];
            let ibo = gl.create_buffer().expect("cannot create buffer");
            gl.bind_buffer(ELEMENT_ARRAY_BUFFER, Some(ibo));
            gl.buffer_data_u8_slice(ELEMENT_ARRAY_BUFFER, &indices, STATIC_DRAW);

            
            gl.bind_vertex_array(None);

            Self {
                program,
                vertex_array: vao,
            }
        }
    }

    fn destroy(&self, gl: &glow::Context) {
        use glow::HasContext as _;
        unsafe {
            gl.delete_program(self.program);
            gl.delete_vertex_array(self.vertex_array);
        }
    }

    fn paint(&self, gl: &glow::Context, delta: Vec2, zoom: u32) {
        use glow::HasContext as _;
        unsafe {
            gl.use_program(Some(self.program));
            //update mouse delta
            gl.uniform_2_f32(
                gl.get_uniform_location(self.program, "u_mouse_delta")
                    .as_ref(),
                delta.x,
                delta.y,
            );
            //update zoom
            gl.uniform_1_u32(
                gl.get_uniform_location(self.program, "u_zoom").as_ref(),
                zoom
            );
            gl.bind_vertex_array(Some(self.vertex_array));
            gl.draw_elements(TRIANGLES, 6, UNSIGNED_BYTE, 0);
        }
    }
}
