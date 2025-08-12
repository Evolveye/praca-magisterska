
use std::time::{Duration, Instant};

use cgmath::{ point2, vec2, vec3, InnerSpace, Point2, Point3, Vector3 };
use winit::{event::{ElementState, DeviceEvent::{ self, MouseMotion }, KeyEvent}, keyboard::{KeyCode, PhysicalKey}};

use crate::app::settings::AppSettings;

type Vec3 = cgmath::Vector3<f32>;
type Mat4 = cgmath::Matrix4<f32>;

#[derive(Clone, Debug)]
pub struct ControlManager {
    pub position: Point3<f32>,
    pub sprint_init: Option<Instant>,
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
    pub fn new( position:Point3<f32>, target:Point3<f32> ) -> Self {
        let mut instance = Self {
            position,
            rotation: vec2( 0.0, 0.0 ),
            sprint_init: None,
            velocity_left: 0.0,
            velocity_right: 0.0,
            velocity_down: 0.0,
            velocity_up: 0.0,
            velocity_backward: 0.0,
            velocity_forward: 0.0,
            mouse_position: point2( 0.0, 0.0 ),
            mouse_last_used_position: point2( 0.0, 0.0 ),
            lmb_pressed: false
        };

        instance.update_position( position, target );

        instance
    }

    pub fn update_position( &mut self, position:Point3<f32>, target:Point3<f32> ) {
        let direction = (target - position).normalize();
        let pitch = direction.y.asin();
        let yaw = direction.z.atan2( direction.x );

        self.position = position;
        self.rotation = vec2( pitch, yaw );
    }

    pub fn update_rotation( &mut self, settings:&AppSettings, delta_time:f32 ) {
        if self.lmb_pressed {
            self.rotation.x += (self.mouse_position.y - self.mouse_last_used_position.y) * -settings.rotation_sensitivity * delta_time;
            self.rotation.y += (self.mouse_position.x - self.mouse_last_used_position.x) *  settings.rotation_sensitivity * delta_time;
        }
    }

    pub fn update( &mut self, settings:&AppSettings, delta_time:f32 ) {
        self.update_rotation( settings, delta_time );

        let front = Vector3::new( self.rotation.y.cos(), 0.0, self.rotation.y.sin() ).normalize();
        let right = Vector3::new( front.z, 0.0, -front.x );

        let sprint_sign = self.velocity_forward - self.velocity_backward;
        let sprint_sign = if sprint_sign < 0.0 { -1.0 } else if sprint_sign > 0.0 { 1.0 } else { 0.0 };
        let sprint = if let Some( sprint_init ) = self.sprint_init {
            if sprint_init.elapsed() >= Duration::from_secs( 2 ) {
                settings.sprint_speed_x2 * sprint_sign
            } else {
                settings.sprint_speed_x1 * sprint_sign
            }
        } else { 0.0 };

        self.position += front * if sprint != 0.0 { sprint } else { self.velocity_forward - self.velocity_backward } * settings.movement_speed * delta_time;
        self.position += right * (self.velocity_left - self.velocity_right) * settings.movement_speed * delta_time;

        self.position.y += (self.velocity_up - self.velocity_down) * settings.movement_speed * delta_time;
    }

    pub fn get_view_matrix( &self ) -> Mat4 {
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

    pub fn handle_keyboard_event( &mut self, settings:&AppSettings, event:KeyEvent ) {
        let pressed = event.state == ElementState::Pressed;

        match event.physical_key {
            PhysicalKey::Code( KeyCode::ArrowLeft  ) | PhysicalKey::Code( KeyCode::KeyA ) => self.velocity_left  = if pressed { settings.movement_speed } else { 0.0 },
            PhysicalKey::Code( KeyCode::ArrowRight ) | PhysicalKey::Code( KeyCode::KeyD ) => self.velocity_right = if pressed { settings.movement_speed } else { 0.0 },
            PhysicalKey::Code( KeyCode::ShiftLeft  ) => self.velocity_down = if pressed { settings.movement_speed } else { 0.0 },
            PhysicalKey::Code( KeyCode::Space      ) => self.velocity_up   = if pressed { settings.movement_speed } else { 0.0 },
            PhysicalKey::Code( KeyCode::ArrowUp    ) | PhysicalKey::Code( KeyCode::KeyW ) => self.velocity_forward  = if pressed { settings.movement_speed } else { 0.0 },
            PhysicalKey::Code( KeyCode::ArrowDown  ) | PhysicalKey::Code( KeyCode::KeyS ) => self.velocity_backward = if pressed { settings.movement_speed } else { 0.0 },
            PhysicalKey::Code( KeyCode::ControlLeft ) => self.sprint_init = if pressed { Some( Instant::now() ) } else { None },

            _ => {}
        }
    }

    pub fn handle_device_event( &mut self, settings:&AppSettings, event:DeviceEvent ) {
        match event {
            MouseMotion { delta } => {
                let (dx, dy) = delta;
                self.rotation.y += dx as f32 * settings.rotation_sensitivity;
                self.rotation.x -= dy as f32 * settings.rotation_sensitivity;
                self.rotation.x = self.rotation.x.clamp(-std::f32::consts::FRAC_PI_2, std::f32::consts::FRAC_PI_2);
            }

            _ => {}
        }
    }
}
