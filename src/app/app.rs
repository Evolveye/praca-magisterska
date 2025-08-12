use std::time::{Duration, Instant};

use anyhow::Result;
use cgmath::point3;
use winit::{
    event::{ Event, WindowEvent },
    event_loop::EventLoopWindowTarget,
    keyboard::{ KeyCode, PhysicalKey }
};
use crate::{
    app::{ control_manager::ControlManager, settings::AppSettings, window_manager::WindowManager },
     rendering::renderer::Renderer,
     structure_tests::generate_world_as_world,
     world::{
        world::{ ChunkLoaderhandle, World, CHUNK_SIZE },
        world_renderer::WorldRenderer
    }
};

pub struct App {
  pub renderer: Renderer,
  pub window_manager: WindowManager,
  pub world_renderer: WorldRenderer,
  pub world: World,
  pub camera_chunk_loader: ChunkLoaderhandle,
  pub settings: AppSettings,
  pub control_manager: ControlManager,

  pub start_time: Instant,
  pub last_tick_time: Instant,
  pub fps_time: Instant,
}

impl App {
    pub fn new() -> Result<Self> {
        let half_chunk_size = CHUNK_SIZE as f32 / 2.0;

        let window_manager = WindowManager::new()?;
        let control_manager = ControlManager::new( point3( half_chunk_size, 15.0, half_chunk_size ), point3( half_chunk_size, 0.0, 0.0 ) );
        let renderer = unsafe { Renderer::create( &window_manager.window )? };
        let world_renderer = WorldRenderer::new( &renderer );
        let settings = AppSettings::new();
        let ( world, camera_chunk_loader ) = generate_world_as_world();

        Ok( App {
            window_manager,
            renderer,
            world_renderer,
            world,
            camera_chunk_loader,
            control_manager,
            settings,

            start_time: Instant::now(),
            last_tick_time: Instant::now(),
            fps_time: Instant::now(),
        } )
    }

    pub fn tick( &mut self ) {
        let timestamp = Instant::now();
        let time_delta = timestamp.duration_since( self.last_tick_time );

        self.last_tick_time = timestamp;

        if self.fps_time.elapsed() >= Duration::from_secs( 1 ) {
            // let fps = 1.0 / time_delta.as_secs_f64();
            // println!( "fps={}", fps as u32 );

            self.fps_time = timestamp;
        }

        self.world.move_chunk_loader_to( &self.camera_chunk_loader, self.control_manager.position.into() );
        self.world.update();

        self.control_manager.update( &self.settings, time_delta.as_secs_f32() );

        self.world_renderer.update_instances_buffer( &self.renderer, &self.world );
    }

    pub fn run_loop( &mut self ) {
        self.window_manager.event_loop.take().expect( "Event loop has been used in the past" ).run( |event, elwt| {
            elwt.set_control_flow( winit::event_loop::ControlFlow::Poll );

            match event {
                Event::AboutToWait => self.window_manager.window.request_redraw(),

                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::RedrawRequested if !elwt.exiting() && !self.window_manager.minimized => {
                        if self.window_manager.focused {
                            self.window_manager.window.set_cursor_position( self.window_manager.center ).unwrap();
                            self.control_manager.mouse_last_used_position = self.control_manager.mouse_position;
                        }

                        self.tick();

                        unsafe {
                            let view_matrix = self.control_manager.get_view_matrix();
                            self.renderer.render( &mut self.window_manager, view_matrix, vec![ &self.world_renderer ] ).unwrap();
                        }
                    },

                    WindowEvent::CloseRequested => App::destroy( elwt, self ),

                    WindowEvent::KeyboardInput { event, .. } => {
                        match event.physical_key {
                            PhysicalKey::Code( KeyCode::Escape ) => App::destroy( elwt, self ),
                            _ => self.control_manager.handle_keyboard_event( &self.settings, event ),
                        }
                    }

                    _ => self.window_manager.handle_window_event( event ),
                }

                Event::DeviceEvent { event, .. } => self.control_manager.handle_device_event( &self.settings, event ),

                _ => {}
            }
        } ).unwrap();
    }

    fn destroy( elwt:&EventLoopWindowTarget<()>, state:&mut App ) {
        elwt.exit();

        unsafe {
            state.renderer.device_wait_idle();
            state.world_renderer.model.destroy( &state.renderer.device );
            state.renderer.destroy();
        }

        println!( "App uptime = {:?}", state.start_time.elapsed() );
    }
}
