// TODO: For now the goal of this is to be just a demo rendering a terrain,
//       then wander around on the map, and only then actual networking code will be added.

// Target: OpenGL 3.1 for now, so GLSL 1.40 can be used.

#[macro_use]
extern crate glium;
extern crate nalgebra;
extern crate opensim_networking;
extern crate typenum;

use glium::{glutin, Surface};
use glium::index::PrimitiveType;

mod data;
use self::data::*;
use self::data::terrain::*;

fn main() {
    let mut events_loop = glutin::EventsLoop::new();
    let window = glutin::WindowBuilder::new();
    let context = glutin::ContextBuilder::new();
    let display = glium::Display::new(window, context, &events_loop).unwrap();

    // Get the data to be displayed.
    let mut terrain_manager = TerrainManager::new();
    let terrain_loc = PatchLocator {
        region: RegionLocator {
            grid: "dummy".to_string(),
            reg_pos: Vector2::new(0,0),
        },
        patch_pos: Vector2::new(0,0),
    };
    let terrain = terrain_manager.get_patch(&terrain_loc).unwrap();

    // building the vertex buffer, which contains all the vertices that we will draw
    let vertex_buffer = {
        #[derive(Copy, Clone)]
        struct Vertex {
            position: [f32; 3],
            color: [f32; 3],
        }

        implement_vertex!(Vertex, position, color);

        // Convert the heightmap to vertices.
        let mut vertices = Vec::new();
        for x in 0..256 {
            for y in 0..256 {
                vertices.push(Vertex {
                    position: [x as f32, y as f32, terrain.land[(x,y)]],
                    color: [0.0, 1.0, 0.0]
                });
            }
        }

        glium::VertexBuffer::new(&display, &vertices[..]).unwrap()
    };

    // building the index buffer
    let index_buffer = glium::IndexBuffer::new(&display, PrimitiveType::TrianglesList,
                                               &[0u16, 1, 2]).unwrap();

    // compiling shaders and linking them together
    let program = program!(&display,
        140 => {
            vertex: "
                #version 140
                uniform mat4 matrix;
                in vec3 position;
                in vec3 color;
                out vec3 vColor;
                void main() {
                    gl_Position = vec4(position, 0.0) * matrix;
                    vColor = color;
                }
            ",

            fragment: "
                #version 140
                in vec3 vColor;
                out vec4 f_color;
                void main() {
                    f_color = vec4(vColor, 1.0);
                }
            "
        },

    ).unwrap();

    // Here we draw the black background and triangle to the screen using the previously
    // initialised resources.
    //
    // In this case we use a closure for simplicity, however keep in mind that most serious
    // applications should probably use a function that takes the resources as an argument.
    let draw = || {
        // building the uniforms
        let uniforms = uniform! {
            matrix: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0f32]
            ]
        };

        // drawing a frame
        let mut target = display.draw();
        target.clear_color(0.0, 0.0, 0.0, 0.0);
        target.draw(&vertex_buffer, &index_buffer, &program, &uniforms, &Default::default()).unwrap();
        target.finish().unwrap();
    };

    // Draw the triangle to the screen.
    draw();

    // the main loop
    events_loop.run_forever(|event| {
        match event {
            glutin::Event::WindowEvent { event, .. } => match event {
                // Break from the main loop when the window is closed.
                glutin::WindowEvent::Closed => return glutin::ControlFlow::Break,
                // Redraw the triangle when the window is resized.
                glutin::WindowEvent::Resized(..) => draw(),
                _ => (),
            },
            _ => (),
        }
        glutin::ControlFlow::Continue
    });
}
