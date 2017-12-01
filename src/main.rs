// TODO: For now the goal of this is to be just a demo rendering a terrain,
//       then wander around on the map, and only then actual networking code will be added.

/// Target: OpenGL 3.1 for now, so GLSL 1.40 can be used.

#[macro_use]
extern crate glium;
extern crate nalgebra;
extern crate opensim_networking;
extern crate typenum;

use glium::{glutin, Surface};
use glium::index::PrimitiveType;
use std::time::{Duration, Instant};
use std::thread;

mod data;
use self::data::*;
use self::data::terrain::*;

mod camera;

fn main() {
    let mut events_loop = glutin::EventsLoop::new();
    let window = glutin::WindowBuilder::new();
    let context = glutin::ContextBuilder::new().with_depth_buffer(24);
    let display = glium::Display::new(window, context, &events_loop).unwrap();

    // Get the data to be displayed.
    let mut terrain_manager = TerrainManager::new();
    let terrain_loc = PatchLocator {
        region: RegionLocator {
            grid: "dummy".to_string(),
            reg_pos: Vector2::new(0, 0),
        },
        patch_pos: Vector2::new(0, 0),
    };
    let terrain = terrain_manager.get_patch(&terrain_loc).unwrap();

    // building the vertex buffer, which contains all the vertices that we will draw
    let vertex_buffer = {
        #[derive(Copy, Clone)]
        struct Vertex {
            position: [f32; 3],
            //color: [f32; 3],
        }

        implement_vertex!(Vertex, position);

        // Convert the heightmap to vertices, each grid cell is represented by
        // two triangles, i.e. 6 vertices.
        let mut vertices = Vec::new();
        for x1 in 0..255 {
            for y1 in 0..255 {
                let x2 = x1 + 1;
                let y2 = y1 + 1;

                let mut add_vertex = |x: usize, y: usize| {
                    vertices.push(Vertex {
                        position: [x as f32, y as f32, terrain.land[(x, y)]],
                    });
                };

                add_vertex(x1, y1);
                add_vertex(x2, y1);
                add_vertex(x1, y2);

                add_vertex(x2, y2);
                add_vertex(x1, y2);
                add_vertex(x2, y1);

                /*
                vertices.push(Vertex {
                    position: [x as f32, y as f32, terrain.land[(x, y)]],
                    color: [0.0, 1.0, 0.0],
                });
                */
            }
        }

        glium::VertexBuffer::new(&display, &vertices[..]).unwrap()
    };

    // compiling shaders and linking them together
    let program = program!(&display,
        140 => {
            vertex: "
                #version 140

                uniform mat4 persp_matrix;
                uniform mat4 view_matrix;

                in vec3 position;
                //in vec3 normal;
                out vec3 v_position;
                out vec3 v_normal;
                out float v_color;

                void main() {
                    v_position = position;
                    //v_normal = normal;
                    v_normal = vec3(1.0, 0.0, 0.0);
                    gl_Position = persp_matrix * view_matrix * vec4(v_position * 0.005, 1.0);
                    v_color = position.z / 24.8;
                }
            ",

            fragment: "
                #version 140

                in vec3 v_normal;
                in float v_color;
                out vec4 f_color;

                const vec3 LIGHT = vec3(-0.2, 0.8, 0.1);

                void main() {
                    //float lum = max(dot(normalize(v_normal), normalize(LIGHT)), 0.0);
                    //vec3 color = (0.3 + 0.7 * lum) * vec3(1.0, 1.0, 1.0);
                    //f_color = vec4(color, 1.0);

                    f_color = vec4(v_color, 0.5, v_color, 1.0);
                }
            "
        },

    ).unwrap();

    let mut camera = camera::CameraState::new();
    camera.set_direction((1., 0., 0.));
    //camera.set_position((0., 128., 0.));
    camera.set_position((0., 0., 0.));

    let index_buffer = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);
    let params = glium::DrawParameters {
        depth: glium::Depth {
            test: glium::DepthTest::IfLess,
            write: true,
            ..Default::default()
        },
        ..Default::default()
    };

    let redraw = |camera: &camera::CameraState| {
        // Compute he uniforms.
        let uniforms = uniform! {
            persp_matrix: camera.get_perspective(),
            view_matrix: camera.get_view(),
        };

        // Draw a frame.
        let mut target = display.draw();
        target.clear_color_and_depth((0.0, 0.0, 0.0, 0.0), 1.0);
        target
            .draw(&vertex_buffer, &index_buffer, &program, &uniforms, &params)
            .unwrap();
        target.finish().unwrap();
    };

    // Draw the triangle to the screen.
    redraw(&camera);

    // Main loop.
    let mut accumulator = Duration::new(0, 0);
    let mut previous_clock = Instant::now();
    loop {
        // Draw the frame.
        camera.update();
        redraw(&camera);

        // Handle events.
        let mut exit = false;
        events_loop.poll_events(|event| match event {
            glutin::Event::WindowEvent { event, .. } => match event {
                glutin::WindowEvent::Closed => exit = true,
                ev => camera.process_input(&ev),
            },
            _ => {}
        });
        if exit {
            break;
        }

        // Update clock.
        let now = Instant::now();
        accumulator += now - previous_clock;
        previous_clock = now;

        let fixed_time_step = Duration::new(0, 16666667);
        while accumulator >= fixed_time_step {
            accumulator -= fixed_time_step;

            // Update world state.
        }

        thread::sleep(fixed_time_step - accumulator);
    }
}
