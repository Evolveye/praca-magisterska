use std::time::{ Duration, Instant };
use anyhow::Result;
use cgmath::{ point2, point3, vec2, vec3, InnerSpace, Point2, Point3, Vector3 };
use winit::{
  dpi::PhysicalPosition,
  event::{ ElementState, Event, WindowEvent },
  keyboard::{ PhysicalKey, KeyCode },
};

use super::{
  rendering::renderer::Renderer,
  window_manager::WindowManager,
};

type Vec3 = cgmath::Vector3<f32>;
type Mat4 = cgmath::Matrix4<f32>;

#[derive(Clone, Debug)]
pub struct ControlManager {
  pub position: Point3<f32>,
  pub velocity_left: f32,
  pub velocity_right: f32,
  pub velocity_up: f32,
  pub velocity_down: f32,
  pub velocity_forward: f32,
  pub velocity_backward: f32,
  pub rotation: cgmath::Vector2<f32>,
  pub mouse_position: Point2<f32>,
  pub mouse_last_used_position: Point2<f32>,
  pub lmb_pressed: bool,
}

impl ControlManager {
  fn new( position:Point3<f32>, target:Point3<f32>) -> Self {
    let direction = (target - position).normalize();
    let pitch = direction.y.asin();
    let yaw = direction.z.atan2( direction.x );

    Self {
      position,
      velocity_right: 0.0,
      velocity_left: 0.0,
      velocity_up: 0.0,
      velocity_down: 0.0,
      velocity_forward: 0.0,
      velocity_backward: 0.0,
      rotation: vec2( pitch, yaw ),
      mouse_position: point2( 0.0, 0.0 ),
      mouse_last_used_position: point2( 0.0, 0.0 ),
      lmb_pressed: false
    }
  }

  fn update_rotation( &mut self, settings:&AppSettings, delta_time:f32 ) {
    if self.lmb_pressed {
      self.rotation.x += (self.mouse_position.y - self.mouse_last_used_position.y) * -settings.rotation_sensitivity * delta_time;
      self.rotation.y += (self.mouse_position.x - self.mouse_last_used_position.x) *  settings.rotation_sensitivity * delta_time;
    }
  }

  fn update( &mut self, settings:&AppSettings, delta_time:f32 ) {
    self.update_rotation( settings, delta_time );

    let speed = settings.movement_speed;
    let front = Vector3::new( self.rotation.y.cos(), 0.0, self.rotation.y.sin() ).normalize();
    let right = Vector3::new( front.z, 0.0, -front.x );

    self.position += front * (self.velocity_forward - self.velocity_backward) * speed * delta_time;
    self.position += right * -(self.velocity_right - self.velocity_left) * speed * delta_time;

    self.position.y += (self.velocity_up - self.velocity_down) * speed * delta_time;
  }

  fn get_view_matrix( &self ) -> Mat4 {
    Mat4::look_at_rh(
      self.position,
      self.position + vec3(
        self.rotation.y.cos() * self.rotation.x.cos(),
        self.rotation.x.sin(),
        self.rotation.y.sin() * self.rotation.x.cos(),
      ),
      Vec3::unit_y(),
    )
  }
}


#[derive(Clone, Debug)]
pub struct AppSettings {
  pub rotation_sensitivity: f32,
  pub movement_speed: f32,
}

impl AppSettings {
  fn new() -> Self {
    Self {
      rotation_sensitivity: 0.004,
      movement_speed: 3.0,
    }
  }
}

pub struct Simulation {
  renderer: Renderer,
  window_manager: WindowManager,
  // world: world::World,
  settings: AppSettings,
  control_manager: ControlManager,

  start_time: Instant,
  last_tick_time: Instant,
  fps_time: Instant,
  fps_count: u32,
}

impl Simulation {
  pub fn new() -> Result<Self> {
    pretty_env_logger::init();

    let window_manager = WindowManager::new()?;
    let mut renderer = unsafe { Renderer::create( &window_manager.window )? };

    unsafe { renderer.load_cube()?; }

    Ok( Simulation {
      window_manager,
      renderer,
      // world: world::World::new(),
      control_manager: ControlManager::new( point3( 0.0, 20.0, -35.0 ), point3( 0.0, 0.0, -8.0 ) ),
      settings: AppSettings::new(),

      start_time: Instant::now(),
      last_tick_time: Instant::now(),
      fps_time: Instant::now(),
      fps_count: 0,
    } )
  }

  pub fn run_window_event_loop( &mut self ) {
    let window_manager = &mut self.window_manager;

    let window_size = window_manager.window.inner_size();
    let center = PhysicalPosition::new( window_size.width as f64 / 2.0, window_size.height as f64 / 2.0 );
    let mut minimized = false;

    window_manager.event_loop.take().expect( "Event loop has been used in the past" ).run( |event, elwt| {
      match event {
        Event::AboutToWait => window_manager.window.request_redraw(),

        Event::WindowEvent { event, .. } => match event {
          WindowEvent::RedrawRequested if !elwt.exiting() && !minimized => {
            if window_manager.focused {
              window_manager.window.set_cursor_position( center ).unwrap();
              self.control_manager.mouse_last_used_position = self.control_manager.mouse_position;
            }

            let timestamp = Instant::now();
            let time_delta = timestamp.duration_since( self.last_tick_time );

            self.last_tick_time = timestamp;
            self.fps_count += 1;

            if self.fps_time.elapsed() >= Duration::from_secs( 1 ) {
              let fps = 1.0 / time_delta.as_secs_f64();
              println!( "fps={}", self.fps_count );

              self.fps_time = timestamp;
              self.fps_count = 0
            }

            self.control_manager.update( &self.settings, time_delta.as_secs_f32() );
            let view_matrix = self.control_manager.get_view_matrix();
            unsafe { self.renderer.render( window_manager, view_matrix ) }.unwrap();
          },

          WindowEvent::Resized( size ) => {
            if size.width == 0 || size.height == 0 {
              minimized = true;
            } else {
              minimized = false;
              window_manager.resized = true;
            }
          }

          WindowEvent::Focused( focused ) => {
            window_manager.focused = focused;
          }

          WindowEvent::CloseRequested => {
            elwt.exit();
            unsafe { self.renderer.destroy(); }
          }

          WindowEvent::KeyboardInput { event, .. } => {
            let pressed = event.state == ElementState::Pressed;
            let speed = 2.0;

            match event.physical_key {
              PhysicalKey::Code( KeyCode::ArrowLeft  ) | PhysicalKey::Code( KeyCode::KeyA ) => self.control_manager.velocity_left  = if pressed { speed } else { 0.0 },
              PhysicalKey::Code( KeyCode::ArrowRight ) | PhysicalKey::Code( KeyCode::KeyD ) => self.control_manager.velocity_right = if pressed { speed } else { 0.0 },
              PhysicalKey::Code( KeyCode::ShiftLeft  ) => self.control_manager.velocity_down = if pressed { speed } else { 0.0 },
              PhysicalKey::Code( KeyCode::Space      ) => self.control_manager.velocity_up   = if pressed { speed } else { 0.0 },
              PhysicalKey::Code( KeyCode::ArrowUp    ) | PhysicalKey::Code( KeyCode::KeyW ) => self.control_manager.velocity_forward  = if pressed { speed } else { 0.0 },
              PhysicalKey::Code( KeyCode::ArrowDown  ) | PhysicalKey::Code( KeyCode::KeyS ) => self.control_manager.velocity_backward = if pressed { speed } else { 0.0 },
              PhysicalKey::Code( KeyCode::Escape ) => {
                elwt.exit();
                unsafe { self.renderer.destroy(); }
              }
              _ => { }
            }
          }

          _ => {}
        }

        Event::DeviceEvent { event: winit::event::DeviceEvent::MouseMotion { delta }, .. } => {
          let (dx, dy) = delta;
          self.control_manager.rotation.y += dx as f32 * self.settings.rotation_sensitivity;
          self.control_manager.rotation.x -= dy as f32 * self.settings.rotation_sensitivity;
          self.control_manager.rotation.x = self.control_manager.rotation.x.clamp(-std::f32::consts::FRAC_PI_2, std::f32::consts::FRAC_PI_2);
        }

        _ => {}
      }
    } ).unwrap();
  }
}
