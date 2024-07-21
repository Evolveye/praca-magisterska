mod model;
mod vertex;
mod renderer;
mod texture;

use anyhow::Result;

use winit::dpi::{ PhysicalPosition, LogicalSize };
use winit::event::{ ElementState, Event, WindowEvent };
use winit::event_loop::EventLoop;
use winit::keyboard::{ PhysicalKey, KeyCode };
use winit::window::WindowBuilder;

use renderer::App;

pub fn render() -> Result<()> {
  println!( "Renderer turned on" );
  pretty_env_logger::init();

  let event_loop = EventLoop::new()?;
  let window = WindowBuilder::new()
    .with_title( "Vulkan Tutorial (Rust)" )
    .with_inner_size( LogicalSize::new( 1536, 1152 ) )
    .build( &event_loop )?;

  window.set_cursor_visible( false );

  let window_size = window.inner_size();
  let center = PhysicalPosition::new(window_size.width as f64 / 2.0, window_size.height as f64 / 2.0);

  let mut app = unsafe { App::create( &window, |a| a )? };
  let mut minimized = false;

  unsafe { app.load_model( "src/resources/barrel.obj" )?; }

  event_loop.run( move |event, elwt| {
    match event {
      Event::AboutToWait => window.request_redraw(),

      Event::WindowEvent { event, .. } => match event {
        WindowEvent::RedrawRequested if !elwt.exiting() && !minimized => {
          if app.focused {
            window.set_cursor_position( center ).unwrap();
            app.control_manager.mouse_last_used_position = app.control_manager.mouse_position;
          }

          unsafe { app.render( &window ) }.unwrap();
        },

        WindowEvent::Resized( size ) => {
          if size.width == 0 || size.height == 0 {
            minimized = true;
          } else {
            minimized = false;
            app.resized = true;
          }
        }

        WindowEvent::Focused( focused ) => {
          app.focused = focused;
        }

        WindowEvent::CloseRequested => {
          elwt.exit();
          unsafe { app.destroy(); }
        }

        WindowEvent::KeyboardInput { event, .. } => {
          let pressed = event.state == ElementState::Pressed;
          let speed = 2.0;

          match event.physical_key {
            PhysicalKey::Code( KeyCode::ArrowLeft  ) | PhysicalKey::Code( KeyCode::KeyA ) => app.control_manager.velocity_left  = if pressed { speed } else { 0.0 },
            PhysicalKey::Code( KeyCode::ArrowRight ) | PhysicalKey::Code( KeyCode::KeyD ) => app.control_manager.velocity_right = if pressed { speed } else { 0.0 },
            PhysicalKey::Code( KeyCode::ShiftLeft  ) => app.control_manager.velocity_down = if pressed { speed } else { 0.0 },
            PhysicalKey::Code( KeyCode::Space      ) => app.control_manager.velocity_up   = if pressed { speed } else { 0.0 },
            PhysicalKey::Code( KeyCode::ArrowUp    ) | PhysicalKey::Code( KeyCode::KeyW ) => app.control_manager.velocity_forward  = if pressed { speed } else { 0.0 },
            PhysicalKey::Code( KeyCode::ArrowDown  ) | PhysicalKey::Code( KeyCode::KeyS ) => app.control_manager.velocity_backward = if pressed { speed } else { 0.0 },
            PhysicalKey::Code( KeyCode::Digit1 ) => app.models = 1,
            PhysicalKey::Code( KeyCode::Digit2 ) => app.models = 2,
            PhysicalKey::Code( KeyCode::Digit3 ) => app.models = 3,
            PhysicalKey::Code( KeyCode::Digit4 ) => app.models = 4,
            PhysicalKey::Code( KeyCode::Escape ) => {
              elwt.exit();
              unsafe { app.destroy(); }
            }
            _ => { }
          }
        }

        _ => {}
      }

      Event::DeviceEvent {
        event: winit::event::DeviceEvent::MouseMotion { delta },
        ..
    } => {
        let (dx, dy) = delta;
        app.control_manager.rotation.y += dx as f32 * app.settings.rotation_sensitivity;
        app.control_manager.rotation.x -= dy as f32 * app.settings.rotation_sensitivity;
        app.control_manager.rotation.x = app.control_manager.rotation.x.clamp(-std::f32::consts::FRAC_PI_2, std::f32::consts::FRAC_PI_2);
    }
      _ => {}
    }
  } )?;

  Ok(())
}
