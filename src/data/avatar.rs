use alga::linear::AffineTransformation;
use alga::linear::Similarity;
use data::{Matrix4, PointLocator, Quaternion, RegionLocator, UnitQuaternion, Uuid, Vector2,
           Vector3};
use glium::glutin;
use std::f32::consts::PI;

/// Anything which can be rendered like an avatar.
pub trait Avatar {
    fn location(&self) -> &PointLocator;
    fn body_rotation(&self) -> &Quaternion<f32>;
    fn head_rotation(&self) -> &Quaternion<f32>;
}

/// The main avatar of the client, i.e. the one is used for interacting
/// with the world.
pub struct ClientAvatar {
    agent_id: Uuid,
    loc: PointLocator,
    // TODO: not used right now
    body_rotation: UnitQuaternion<f32>,
    head_rotation: UnitQuaternion<f32>,
    aspect_ratio: f32,

    pressed_left: bool,
    pressed_right: bool,
    pressed_up: bool,
    pressed_down: bool,
}

// TODO
pub struct OtherAvatar {}

lazy_static! {
    static ref WORLD_TO_DISPLAY: Matrix4<f32> = Matrix4::new(
        1.,
        0.,
        0.,
        0.,
        0.,
        0.,
        1.,
        0.,
        0.,
        1.,
        0.,
        0.,
        0.,
        0.,
        0.,
        1.
    );
}

// TODO: See opensim_networking::systems::agent_update.
// pub fn to_update_message(&self, session_id: Uuid) -> AgentUpdate
// (note: this belongs into the network module and not here)
impl ClientAvatar {
    pub fn new() -> Self {
        // TODO dummy

        let z_axis = Vector3::z_axis();

        ClientAvatar {
            agent_id: Uuid::nil(),
            loc: PointLocator {
                region: RegionLocator {
                    grid: "testgrid".to_string(),
                    reg_pos: Vector2::new(0, 0),
                },
                rel_pos: Vector3::new(5., 5., 5.),
            },
            // placeholder TODO
            body_rotation: UnitQuaternion::from_axis_angle(&z_axis, 0.),
            head_rotation: UnitQuaternion::from_axis_angle(&z_axis, 0.),
            aspect_ratio: 1024. / 768.,

            pressed_left: false,
            pressed_right: false,
            pressed_up: false,
            pressed_down: false,
        }
    }

    pub fn handle_key(&mut self, key: glutin::VirtualKeyCode, pressed: bool) -> bool {
        match key {
            glutin::VirtualKeyCode::Left => {
                self.pressed_left = pressed;
                true
            }
            glutin::VirtualKeyCode::Right => {
                self.pressed_right = pressed;
                true
            }
            glutin::VirtualKeyCode::Up => {
                self.pressed_up = pressed;
                true
            }
            glutin::VirtualKeyCode::Down => {
                self.pressed_down = pressed;
                true
            }
            _ => false,
        }
    }

    /// Updates according to local movement input.
    pub fn update(&mut self) {
        // y-axis in the world, z-axis in the rendering.
        let default_dir = Vector3::y_axis();
        let fwd = self.head_rotation.rotate_vector(&default_dir);
        //println!("fwd: {:?}", fwd);
        if self.pressed_up {
            self.loc.rel_pos += fwd;
        } else if self.pressed_down {
            self.loc.rel_pos -= fwd;
        }

        let axis = Vector3::z_axis();
        if self.pressed_left {
            self.head_rotation = self.head_rotation
                .append_rotation(&UnitQuaternion::from_axis_angle(&axis, 0.1));
        } else if self.pressed_right {
            self.head_rotation = self.head_rotation
                .append_rotation(&UnitQuaternion::from_axis_angle(&axis, -0.1));
        }
    }

    pub fn get_view_matrix(&self) -> Matrix4<f32> {
        // Translate world coordinates to coordinates relative to the eye.
        let trans = Matrix4::new_translation(&(self.loc.rel_pos * -1.));

        // Convert (x,y,z) world to (x,z,y) for display coordinates.
        &*WORLD_TO_DISPLAY * trans * self.head_rotation.to_homogeneous()
    }

    pub fn get_persp_matrix(&self) -> Matrix4<f32> {
        let fovy = PI / 2.;
        let near = 0.1;
        let far = 512.;
        let aspect = 1.0 / (fovy / 2.0).tan() / self.aspect_ratio;

        // This converts from camera relative coordinates to screen coordinates.
        Matrix4::new_perspective(aspect, fovy, near, far)
    }
}

impl Avatar for ClientAvatar {
    fn location(&self) -> &PointLocator {
        &self.loc
    }

    fn body_rotation(&self) -> &Quaternion<f32> {
        &self.body_rotation
    }

    fn head_rotation(&self) -> &Quaternion<f32> {
        &self.head_rotation
    }
}
