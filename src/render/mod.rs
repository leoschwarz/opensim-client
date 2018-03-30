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
    use data::region::RegionDimensions;
    use data::terrain::{self, TerrainStorage};
    use data::{self, ids};
    use failure::Error;
    use std::sync::Arc;
    use types::{nalgebra, DMatrix, Vector2};

    #[derive(Copy, Clone)]
    pub struct Vertex {
        position: [f32; 3],
        //color: [f32; 3],
    }

    implement_vertex!(Vertex, position);

    /// Render state for land layer terrain data of one region.
    pub struct RenderState {
        region_id: ids::RegionId,
        vertices: Vec<Vertex>,
        /// Current write offset.
        vertices_offset: usize,

        /// Patches which yet have to be added the vertices vector.
        patches_pending: Vec<data::terrain::PatchPosition>,
    }

    impl RenderState {
        pub fn new(region_id: ids::RegionId, reg_dims: &RegionDimensions) -> Self {
            let pps = reg_dims.patches_per_side as usize;

            let mut patches_pending = Vec::new();
            for patch_x in 0..pps {
                for patch_y in 0..pps {
                    patches_pending.push(Vector2::new(patch_x as u8, patch_y as u8));
                }
            }

            let mut vertices = Vec::new();
            let v_per_patch = 6 * (reg_dims.patch_size_axis as usize - 1).pow(2);
            let v_tot = pps * pps * v_per_patch;
            for _ in 0..v_tot {
                vertices.push(Vertex {
                    position: [0., 0., 0.],
                });
            }

            RenderState {
                region_id,
                vertices,
                vertices_offset: 0,
                patches_pending,
            }
        }

        /// Tries to update the vertices with new available terrain patches.
        ///
        /// Returns an error if there was one. Otherwise, if and only if some
        /// new vertices are added `Ok(true)` is returned, else
        /// `Ok(false)`.
        pub fn update(&mut self, storage: Arc<TerrainStorage>) -> Result<bool, Error> {
            let mut res: Result<(), Error> = Ok(());

            let region_id = self.region_id.clone();
            let mut patches = Vec::new();
            self.patches_pending
                .retain(|&pos| match storage.get_patch(&(region_id, pos)) {
                    Ok(patch) => {
                        patches.push(patch);
                        false
                    }
                    Err(terrain::StorageError::NotFound) => true,
                    Err(terrain::StorageError::Cache(e)) => {
                        res = Err(e.into());
                        true
                    }
                });

            for patch in patches.iter() {
                self.add_vertices(patch);
            }

            res?;
            Ok(patches.len() > 0)
        }

        pub fn vertices(&self) -> &[Vertex] {
            &self.vertices[..]
        }

        /// For each grid cell two triangles, i.e. 6 vertices, are inserted.
        fn add_vertices(&mut self, patch: &data::terrain::TerrainPatch) {
            let size = patch.size();
            let offset = Vector2::new(
                patch.position()[0] as usize * size,
                patch.position()[1] as usize * size,
            );

            for x1 in 0..(size - 1) {
                for y1 in 0..(size - 1) {
                    let mut add_vertex = |x_rel: usize, y_rel: usize| {
                        let x_abs = x_rel + offset[0] as usize;
                        let y_abs = y_rel + offset[1] as usize;
                        self.vertices[self.vertices_offset] = Vertex {
                            position: [
                                x_abs as f32,
                                y_abs as f32,
                                patch.land_heightmap()[(x_rel, y_rel)],
                            ],
                        };
                        self.vertices_offset += 1;
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

    // Compile the shaders, and link them together.
    let program = program!(&display,
        140 => {
            vertex: include_str!("../../shader/terrain_land.vert"),
            fragment: include_str!("../../shader/terrain_land.frag"),
        },
    ).unwrap();

    // Wait for region connection. (TODO loading screen.)
    while storage.client_avatar.read().current_region().is_none() {
        thread::sleep(Duration::from_millis(50));
    }

    let region_id = storage
        .client_avatar
        .read()
        .current_region()
        .clone()
        .unwrap();
    let region_conn = storage.region.get(&region_id).unwrap();
    let region = region_conn.clone_region().unwrap();

    //let pps = region.dimensions().patches_per_side as usize;

    let mut render_state = terrain_land::RenderState::new(region_id, region.dimensions());
    let v_buffer =
        glium::VertexBuffer::empty_dynamic(&display, render_state.vertices().len()).unwrap();

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
            .draw(&v_buffer, &index_buffer, &program, &uniforms, &params)
            .unwrap();
        target.finish().unwrap();
    };

    // Draw the triangle to the screen.
    redraw(&storage.client_avatar);

    // Main loop.
    let mut accumulator = Duration::new(0, 0);
    let mut previous_clock = Instant::now();
    loop {
        // Update as needed.
        if render_state.update(Arc::clone(&storage.terrain)).unwrap() {
            v_buffer.write(render_state.vertices())
        }

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
