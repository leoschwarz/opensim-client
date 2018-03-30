//! This module is responsible for rendering the World.
//!
//! Targets OpenGL 3.1 and GLSL 1.40 for now.

use data::avatar::ClientAvatar;
use data::{self, Storage};
use glium::index::PrimitiveType;
use glium::{self, glutin, Surface};
use parking_lot::RwLock;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};
use typed_rwlock::{RwLockReader, RwLockWriter};
use types::Vector3;

pub mod terrain_land {
    use data::{self, ids};
    use data::region::RegionDimensions;
    use data::terrain::{self, TerrainStorage};
    use types::{nalgebra, DMatrix, Vector2};
    use std::sync::Arc;
    use failure::Error;

    #[derive(Copy, Clone)]
    struct Vertex {
        position: [f32; 3],
        //color: [f32; 3],
    }

    implement_vertex!(Vertex, position);

    /// Render state for land layer terrain data of one region.
    struct RenderState {
        region_id: ids::RegionId,
        vertices: Vec<Vertex>,

        /// Patches which yet have to be added the vertices vector.
        patches_pending: Vec<data::terrain::PatchPosition>,
    }

    impl RenderState {
        fn new(region_id: ids::RegionId, reg_dims: &RegionDimensions) -> Self {
            let pps = reg_dims.patches_per_side;

            let mut patches_pending = Vec::new();
            for patch_x in 0..pps {
                for patch_y in 0..pps {
                    patches_pending.push(Vector2::new(patch_x, patch_y));
                }
            }

            RenderState {
                region_id,
                vertices: Vec::new(),
                patches_pending,
            }
        }

        pub fn update(&mut self, storage: Arc<TerrainStorage>) -> Result<(), Error> {
            let mut res = Ok(());

            let region_id = self.region_id.clone();
            let mut patches = Vec::new();
            self.patches_pending.retain(|&pos| {
                match storage.get_patch(&(region_id, pos)) {
                    Ok(patch) => { patches.push(patch); false }
                    Err(terrain::StorageError::NotFound) => { true }
                    Err(terrain::StorageError::Cache(e)) => {
                        res = Err(e.into());
                        true
                    }
                }
            });

            for patch in patches.iter() {
                add_vertices(patch, &mut self.vertices);
            }

            res
        }
    }

    /// Add vertices for the provided patch to the vertices vector.
    ///
    /// For each grid cell two triangles, i.e. 6 vertices, are inserted.
    fn add_vertices(patch: &data::terrain::TerrainPatch, vertices: &mut Vec<Vertex>)
    {
        let size = patch.size();
        let offset = Vector2::new(patch.position()[0] as usize * size, patch.position()[1] as usize * size);

        for x in 0..size {
            for y in 0..size {
                let mut add_vertex = |x: usize, y: usize| {
                    vertices.push(Vertex {
                        position: [x as f32, y as f32, patch.land_heightmap()[(x, y)]]
                    });
                };

                let x1 = x + offset[0] as usize;
                let y1 = y + offset[1] as usize;
                let x2 = x1 + 1;
                let y2 = y1 + 1;

                add_vertex(x1, y1);
                add_vertex(x2, y1);
                add_vertex(x1, y2);

                add_vertex(x2, y2);
                add_vertex(x1, y2);
                add_vertex(x2, y1);
            }
        }
    }
}


pub fn render_world(storage: Storage) {
    // Setup display.
    // TODO: Maybe this does not belong into the render world method?
    let mut events_loop = glutin::EventsLoop::new();
    let window = glutin::WindowBuilder::new();
    let context = glutin::ContextBuilder::new().with_depth_buffer(24);
    let display = glium::Display::new(window, context, &events_loop).unwrap();

    // TODO
    let terrain_patch = data::terrain::TerrainPatch::dummy();

    // Build the vertex buffer.
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
                let mut add_vertex = |x: usize, y: usize| {
                    vertices.push(Vertex {
                        position: [x as f32, y as f32, terrain_patch.land_heightmap()[(x, y)]],
                    });
                };

                let x2 = x1 + 1;
                let y2 = y1 + 1;

                add_vertex(x1, y1);
                add_vertex(x2, y1);
                add_vertex(x1, y2);

                add_vertex(x2, y2);
                add_vertex(x1, y2);
                add_vertex(x2, y1);
            }
        }

        glium::VertexBuffer::new(&display, &vertices[..]).unwrap()
    };

    // compiling shaders and linking them together
    let program = program!(&display,
        140 => {
            vertex: include_str!("../../shader/terrain_land.vert"),
            fragment: include_str!("../../shader/terrain_land.frag"),
        },

    ).unwrap();

    // let mut camera = camera::CameraState::new();
    let index_buffer = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);
    let params = glium::DrawParameters {
        depth: glium::Depth {
            test: glium::DepthTest::IfLess,
            write: true,
            ..Default::default()
        },
        ..Default::default()
    };

    let redraw = |avatar: &Arc<RwLock<ClientAvatar>>| {
        // Compute he uniforms.
        let uniforms = uniform! {
            persp_matrix: avatar.read().get_persp_matrix().as_ref().clone(),
            view_matrix: avatar.read().get_view_matrix().as_ref().clone(),
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
    redraw(&storage.client_avatar);

    // Main loop.
    let mut accumulator = Duration::new(0, 0);
    let mut previous_clock = Instant::now();
    loop {
        // Draw the frame.
        // camera.update();
        redraw(&storage.client_avatar);

        // Handle events.
        let mut exit = false;
        events_loop.poll_events(|event| match event {
            glutin::Event::WindowEvent { event, .. } => match event {
                glutin::WindowEvent::Closed => exit = true,
                glutin::WindowEvent::KeyboardInput { input, .. } => {
                    let pressed = input.state == glutin::ElementState::Pressed;
                    match input.virtual_keycode {
                        Some(glutin::VirtualKeyCode::Escape) => {exit = true;}
                        Some(key) => {storage.client_avatar.write().handle_key(key, pressed);}
                        _ => {}
                    }
                }
                _ => {}
                //ev => camera.process_input(&ev),
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
            storage.client_avatar.write().update();
        }

        thread::sleep(fixed_time_step - accumulator);
    }
}
