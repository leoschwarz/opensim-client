use glium::glutin;
use nalgebra::{Matrix4, Point3, Vector3};

// Note: world coordinate system is x=right, y=depth, z=up.

pub struct CameraState {
    aspect_ratio: f32,
    position: Vector3<f32>,
    direction: Vector3<f32>,

    moving_up: bool,
    moving_left: bool,
    moving_down: bool,
    moving_right: bool,
    moving_forward: bool,
    moving_backward: bool,
}

impl CameraState {
    pub fn new() -> CameraState {
        CameraState {
            aspect_ratio: 1024.0 / 768.0,
            position: Vector3::new(0.1, 0.1, 1.0),
            direction: Vector3::new(0.0, 0.0, -1.0),
            moving_up: false,
            moving_left: false,
            moving_down: false,
            moving_right: false,
            moving_forward: false,
            moving_backward: false,
        }
    }

    pub fn set_position(&mut self, pos: Vector3<f32>) {
        self.position = pos;
    }

    pub fn set_direction(&mut self, dir: Vector3<f32>) {
        self.direction = dir;
    }

    /// Returns the camera perspective projection.
    ///
    /// Its job is to convert form camere relative coordinates to screen coordinates.
    pub fn get_perspective(&self) -> [[f32; 4]; 4] {
        let fovy: f32 = 3.141592 / 2.0;
        let znear = 0.1;
        let zfar = 1024.;
        let aspect = 1.0 / (fovy / 2.0).tan() / self.aspect_ratio;

        Matrix4::new_perspective(aspect, fovy, znear, zfar)
            .as_ref()
            .clone()
    }

    /// Returns the view transformation matrix.
    ///
    /// Its job is to convert from world coordinates to camera relative coordinates.
    pub fn get_view(&self) -> [[f32; 4]; 4] {
        let trans = Matrix4::new_translation(&(self.position * -1.));
        // Rotate our (x,y,z) to OpenGL (x,z,y) coordinates.
        let rotate_csys = Matrix4::new(
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
            1.,
        );
        let view = trans * rotate_csys;

        view.as_ref().clone()
    }

    pub fn update(&mut self) {
        let f = self.direction.normalize();
        let up = Vector3::new(0.0, 1.0, 0.0);
        let s = f.cross(&up).normalize();
        let u = s.cross(&f);

        if self.moving_up {
            self.position += u * 0.01;
        }

        if self.moving_left {
            self.position -= s * 0.01;
        }

        if self.moving_down {
            self.position -= u * 0.01;
        }

        if self.moving_right {
            self.position += s * 0.01;
        }

        if self.moving_forward {
            self.position += f * 0.01;
        }

        if self.moving_backward {
            self.position -= f * 0.01;
        }
    }

    pub fn process_input(&mut self, event: &glutin::WindowEvent) {
        let input = match *event {
            glutin::WindowEvent::KeyboardInput { input, .. } => input,
            _ => return,
        };
        let pressed = input.state == glutin::ElementState::Pressed;
        let key = match input.virtual_keycode {
            Some(key) => key,
            None => return,
        };
        match key {
            glutin::VirtualKeyCode::Up => self.moving_up = pressed,
            glutin::VirtualKeyCode::Down => self.moving_down = pressed,
            glutin::VirtualKeyCode::Left => self.moving_left = pressed,
            glutin::VirtualKeyCode::Right => self.moving_right = pressed,
            _ => (),
        };
    }
}
