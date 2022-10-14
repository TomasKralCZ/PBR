use glam::{vec3, Mat4, Vec3};

pub trait Camera {
    /// Returns the current view matrix
    fn view_mat(&mut self) -> Mat4;
    /// Updates the latest (x,y) mouse position without adjusting look
    fn track_mouse_pos(&mut self, _new_x: f32, _new_y: f32) {}
    /// Adjust the looking direction
    fn adjust_look(&mut self, _new_x: f32, _new_y: f32) {}
    fn on_forward(&mut self) {}
    fn on_backward(&mut self) {}
    fn on_right(&mut self) {}
    fn on_left(&mut self) {}
    fn on_mouse_wheel(&mut self, _d: f32) {}
    fn get_pos(&self) -> Vec3;
}

pub enum CameraTyp {
    Flycam,
    Orbital,
}

pub struct Flycam {
    pos: Vec3,
    dir: Vec3,
    up: Vec3,
    pub move_speed: f32,
    pub look_sensitivity: f32,
    /// Last x position of the mouse
    current_x: f32,
    /// Last y position of the mouse
    current_y: f32,
    azimuth: f32,
    zenith: f32,
    /// Signals that the view transformation needs to be recomputed
    changed: bool,
    view_matrix: Mat4,
}

impl Flycam {
    pub fn new(
        pos: Vec3,
        move_speed: f32,
        look_sensitivity: f32,
        window_width: u32,
        window_height: u32,
    ) -> Self {
        Self {
            pos,
            dir: Vec3::new(0., 0., -1.),
            up: Vec3::new(0., 1., 0.),
            move_speed,
            look_sensitivity,
            current_x: window_width as f32 / 2.,
            current_y: window_height as f32 / 2.,
            azimuth: 0.,
            zenith: 0.,
            changed: true,
            view_matrix: Mat4::IDENTITY,
        }
    }

    #[allow(dead_code)]
    pub fn set_pos(&mut self, pos: Vec3) {
        self.pos = pos;
        self.changed = true;
    }

    pub fn move_forward(&mut self, d: f32) {
        self.pos += self.dir * d * self.move_speed;
        self.changed = true;
    }

    pub fn move_backward(&mut self) {
        self.move_forward(-1.);
    }

    pub fn strafe_right(&mut self, d: f32) {
        let dir = self.dir.cross(self.up);
        self.pos += dir * d * self.move_speed;
        self.changed = true;
    }

    pub fn strafe_left(&mut self) {
        self.strafe_right(-1.0);
    }

    /// Update the azimuth and zenith
    fn adjust_dir(&mut self) {
        let rad_azimuth = (270. + self.azimuth).to_radians();
        let rad_zenith = self.zenith.to_radians();

        let x = rad_azimuth.cos() * rad_zenith.cos();
        let y = rad_zenith.sin();
        let z = rad_azimuth.sin() * rad_zenith.cos();

        self.dir = Vec3::new(x as f32, y as f32, z as f32).normalize();
        self.changed = true;
    }
}

impl Camera for Flycam {
    fn view_mat(&mut self) -> Mat4 {
        if self.changed {
            self.changed = false;
            self.view_matrix = Mat4::look_at_rh(self.pos, self.pos + self.dir, self.up);
        }

        self.view_matrix
    }

    fn track_mouse_pos(&mut self, new_x: f32, new_y: f32) {
        self.current_x = new_x;
        self.current_y = new_y;
    }

    fn adjust_look(&mut self, new_x: f32, new_y: f32) {
        let dx = new_x - self.current_x;
        let dy = self.current_y - new_y;

        self.current_x = new_x;
        self.current_y = new_y;

        let x_offset = dx * self.look_sensitivity;
        let y_offset = dy * self.look_sensitivity;

        self.azimuth += x_offset;
        self.zenith += y_offset;

        self.zenith = self.zenith.clamp(-89., 89.);

        self.adjust_dir();
    }

    fn on_forward(&mut self) {
        self.move_forward(1.0);
    }

    fn on_backward(&mut self) {
        self.move_backward();
    }

    fn on_right(&mut self) {
        self.strafe_right(1.0);
    }

    fn on_left(&mut self) {
        self.strafe_left();
    }

    fn get_pos(&self) -> Vec3 {
        self.pos
    }
}

pub struct Orbitalcam {
    /// Distance from the origin
    dist: f32,
    pos: Vec3,
    sensitivity: f32,
    /// Last x position of the mouse
    current_x: f32,
    /// Last y position of the mouse
    current_y: f32,
    azimuth: f32,
    zenith: f32,
}

impl Orbitalcam {
    pub fn new(dist: f32, sensitivity: f32, window_width: u32, window_height: u32) -> Self {
        let pos = vec3(0., 0., dist);

        Self {
            dist,
            sensitivity,
            pos,
            current_x: window_width as f32 / 2.,
            current_y: window_height as f32 / 2.,
            azimuth: 180.,
            zenith: 0.,
        }
    }

    /// Update the azimuth and zenith
    fn adjust_dir(&mut self) {
        let rad_azimuth = (270. + self.azimuth).to_radians();
        let rad_zenith = self.zenith.to_radians();

        let x = rad_azimuth.cos() * rad_zenith.cos();
        let y = rad_zenith.sin();
        let z = rad_azimuth.sin() * rad_zenith.cos();

        self.pos = Vec3::new(x as f32, y as f32, z as f32).normalize();
    }

    fn clamp_angle(angle: &mut f32) {
        if *angle < 0. {
            *angle += 360.;
        } else if *angle > 360. {
            *angle -= 360.;
        }
    }
}

impl Camera for Orbitalcam {
    fn view_mat(&mut self) -> Mat4 {
        let up = if (self.zenith > 270. && self.zenith < 360.)
            || (self.zenith >= 0. && self.zenith < 90.)
        {
            vec3(0., 1., 0.)
        } else {
            vec3(0., -1., 0.)
        };

        Mat4::look_at_rh(self.pos * self.dist, vec3(0., 0., 0.), up)
    }

    fn on_forward(&mut self) {
        self.dist -= self.sensitivity;
    }

    fn on_backward(&mut self) {
        self.dist += self.sensitivity;
    }

    fn on_right(&mut self) {}

    fn on_left(&mut self) {}

    fn track_mouse_pos(&mut self, new_x: f32, new_y: f32) {
        self.current_x = new_x;
        self.current_y = new_y;
    }

    fn adjust_look(&mut self, new_x: f32, new_y: f32) {
        let dx = new_x - self.current_x;
        let dy = new_y - self.current_y;

        self.current_x = new_x;
        self.current_y = new_y;

        let x_offset = dx * self.sensitivity;
        let y_offset = dy * self.sensitivity;

        self.azimuth += x_offset;
        self.zenith += y_offset;

        Self::clamp_angle(&mut self.azimuth);
        Self::clamp_angle(&mut self.zenith);

        self.adjust_dir();
    }

    fn on_mouse_wheel(&mut self, d: f32) {
        self.dist += d;
        self.dist = self.dist.clamp(0.1, f32::MAX);
    }

    fn get_pos(&self) -> Vec3 {
        self.pos * self.dist
    }
}
