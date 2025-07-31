#[derive(Clone, Debug)]
pub struct AppSettings {
  pub rotation_sensitivity: f32,
  pub movement_speed: f32,
}

impl AppSettings {
  pub fn new() -> Self {
    Self {
      rotation_sensitivity: 0.004,
      movement_speed: 3.0,
    }
  }
}
