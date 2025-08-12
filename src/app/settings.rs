#[derive(Clone, Debug)]
pub struct AppSettings {
  pub rotation_sensitivity: f32,
  pub movement_speed: f32,
  pub sprint_speed_x1: f32,
  pub sprint_speed_x2: f32,
}

impl AppSettings {
  pub fn new() -> Self {
    Self {
      rotation_sensitivity: 0.004,
      movement_speed: 3.0,
      sprint_speed_x1: 7.0,
      sprint_speed_x2: 12.0,
    }
  }
}
