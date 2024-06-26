#![allow(
  dead_code,
  unused_variables,
  clippy::too_many_arguments,
  clippy::unnecessary_wraps
)]



use anyhow::{anyhow, Result};
use cgmath::{ point2, Point2, point3, Point3, vec2, vec3, Vector3, Deg, InnerSpace };
use log::*;
use thiserror::Error;

use vulkanalia::prelude::v1_0::*;
use vulkanalia::Version;
use vulkanalia::loader::{ LibloadingLoader, LIBRARY };
use vulkanalia::window as vk_window;
use vulkanalia::vk::ExtDebugUtilsExtension;
use vulkanalia::bytecode::Bytecode;
use vulkanalia::vk::KhrSurfaceExtension;
use vulkanalia::vk::KhrSwapchainExtension;

use winit::dpi::{ PhysicalPosition, LogicalSize };
use winit::event::{ ElementState, Event, WindowEvent };
use winit::keyboard::{ PhysicalKey, KeyCode };
use winit::event_loop::EventLoop;
use winit::window::{ Window, WindowBuilder };

use std::collections::{ HashMap, HashSet };
use std::hash::{ Hash, Hasher };
use std::io::BufReader;
use std::fs::File;
use std::os::raw::c_void;
use std::ffi::CStr;
use std::mem::size_of;
use std::ptr::copy_nonoverlapping as memcpy;
use std::time::{ Instant, Duration };

type Vec2 = cgmath::Vector2<f32>;
type Vec3 = cgmath::Vector3<f32>;
type Mat4 = cgmath::Matrix4<f32>;



const PORTABILITY_MACOS_VERSION:Version = Version::new( 1, 3, 216 );
const VALIDATION_ENABLED:bool = cfg!( debug_assertions );
const VALIDATION_LAYER:vk::ExtensionName = vk::ExtensionName::from_bytes( b"VK_LAYER_KHRONOS_validation" );
const DEVICE_EXTENSIONS:&[ vk::ExtensionName ] = &[ vk::KHR_SWAPCHAIN_EXTENSION.name ];
const MAX_FRAMES_IN_FLIGHT: usize = 2;

static VERTICES:[ Vertex; 8 ] = [
  // Vertex::new( vec3( -1.0, -1.0, -1.0 ), vec3( 1.0, 0.0, 0.0 ), vec2( 1.0, 0.0 ) ),
  // Vertex::new( vec3(  1.0, -1.0, -1.0 ), vec3( 0.0, 1.0, 0.0 ), vec2( 0.0, 0.0 ) ),
  // Vertex::new( vec3(  1.0, -1.0,  1.0 ), vec3( 0.0, 1.0, 0.0 ), vec2( 0.0, 0.0 ) ),
  // Vertex::new( vec3( -1.0, -1.0,  1.0 ), vec3( 1.0, 0.0, 0.0 ), vec2( 1.0, 0.0 ) ),

  // Vertex::new( vec3( -1.0,  1.0, -1.0 ), vec3( 0.0, 0.0, 1.0 ), vec2( 0.0, 1.0 ) ),
  // Vertex::new( vec3(  1.0,  1.0, -1.0 ), vec3( 1.0, 1.0, 1.0 ), vec2( 1.0, 1.0 ) ),
  // Vertex::new( vec3(  1.0,  1.0,  1.0 ), vec3( 1.0, 1.0, 1.0 ), vec2( 1.0, 1.0 ) ),
  // Vertex::new( vec3( -1.0,  1.0,  1.0 ), vec3( 0.0, 0.0, 1.0 ), vec2( 0.0, 1.0 ) ),

  Vertex::new( vec3(  1.0, -1.0, -1.0 ), vec3( 0.0, 1.0, 0.0 ), vec2( 0.0, 0.0 ) ),
  Vertex::new( vec3(  1.0, -1.0,  1.0 ), vec3( 0.0, 1.0, 0.0 ), vec2( 0.0, 0.0 ) ),
  Vertex::new( vec3( -1.0, -1.0,  1.0 ), vec3( 1.0, 0.0, 0.0 ), vec2( 1.0, 0.0 ) ),
  Vertex::new( vec3( -1.0, -1.0, -1.0 ), vec3( 1.0, 0.0, 0.0 ), vec2( 1.0, 0.0 ) ),

  Vertex::new( vec3(  1.0,  1.0, -1.0 ), vec3( 1.0, 1.0, 1.0 ), vec2( 1.0, 1.0 ) ),
  Vertex::new( vec3(  1.0,  1.0,  1.0 ), vec3( 1.0, 1.0, 1.0 ), vec2( 1.0, 1.0 ) ),
  Vertex::new( vec3( -1.0,  1.0,  1.0 ), vec3( 0.0, 0.0, 1.0 ), vec2( 0.0, 1.0 ) ),
  Vertex::new( vec3( -1.0,  1.0, -1.0 ), vec3( 0.0, 0.0, 1.0 ), vec2( 0.0, 1.0 ) ),
];

const INDICES:&[ u32 ] = &[
  1, 2, 0, 2, 3, 0, // bottom
  5, 4, 6, 4, 7, 6, // up
  1, 0, 5, 0, 4, 5, // back
  0, 3, 4, 3, 7, 4, // left
  3, 2, 7, 2, 6, 7, // front
  2, 1, 6, 1, 5, 6, // right

  // 1, 2, 3,
  // 7, 6, 5,
  // 4, 5, 1,
  // 5, 6, 2,
];



pub fn render() -> Result<()> {
  println!( "Renderer turned on" );
  pretty_env_logger::init();

  // Window

  let event_loop = EventLoop::new()?;
  let window = WindowBuilder::new()
    .with_title( "Vulkan Tutorial (Rust)" )
    .with_inner_size( LogicalSize::new( 1536, 1152 ) )
    .build( &event_loop )?;

  window.set_cursor_visible( false );

  let window_size = window.inner_size();
  let center = PhysicalPosition::new(window_size.width as f64 / 2.0, window_size.height as f64 / 2.0);

  let mut app = unsafe { App::create( &window )? };
  let mut minimized = false;

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
        app.control_manager.rotation.y += dx as f32 * 0.001;
        app.control_manager.rotation.x -= dy as f32 * 0.001;
        app.control_manager.rotation.x = app.control_manager.rotation.x.clamp(-std::f32::consts::FRAC_PI_2, std::f32::consts::FRAC_PI_2);
    }
      _ => {}
    }
  } )?;

  Ok(())
}

#[derive(Clone, Debug)]
struct ControlManager {
  position: Point3<f32>,
  velocity_left: f32,
  velocity_right: f32,
  velocity_up: f32,
  velocity_down: f32,
  velocity_forward: f32,
  velocity_backward: f32,
  rotation: cgmath::Vector2<f32>,
  mouse_position: Point2<f32>,
  mouse_last_used_position: Point2<f32>,
  lmb_pressed: bool,
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
struct AppSettings {
  rotation_sensitivity: f32,
  movement_speed: f32,
}

impl AppSettings {
  fn new() -> Self {
    AppSettings {
      rotation_sensitivity: 0.02,
      movement_speed: 3.0,
    }
  }
}


#[derive(Clone, Debug)]
struct App {
  models: usize,
  entry: Entry,
  control_manager: ControlManager,
  instance: Instance,
  data: AppData,
  device: Device,
  frame: usize,
  resized: bool,
  last_tick_time: Instant,
  focused: bool,
  settings: AppSettings,
  fps_time: Instant,
  fps_count: u32,
}

impl App {
  /// Creates our Vulkan app.
  unsafe fn create( window:&Window ) -> Result<Self> {
    let loader = LibloadingLoader::new( LIBRARY )?;
    let entry = Entry::new( loader ).map_err( |b| anyhow!( "{}", b ) )?;
    let mut data = AppData::default();
    let instance = create_instance( window, &entry, &mut data )?;

    data.surface = vk_window::create_surface( &instance, &window, &window )?;
    pick_physical_device( &instance, &mut data )?;

    let device = create_logical_device( &entry, &instance, &mut data )?;
    create_swapchain( window, &instance, &device, &mut data )?;
    create_swapchain_image_views( &device, &mut data )?;
    create_render_pass( &instance, &device, &mut data )?;
    create_descriptor_set_layout( &device, &mut data )?;
    create_pipeline( &device, &mut data )?;
    create_command_pools( &instance, &device, &mut data )?;
    create_color_objects( &instance, &device, &mut data )?;
    create_depth_objects( &instance, &device, &mut data )?;
    create_framebuffers( &device, &mut data )?;
    // create_texture_image( &instance, &device, &mut data, "src/resources/viking_room.png" )?;
    create_texture_image( &instance, &device, &mut data, "src/resources/barrel.png" )?;
    create_texture_image_view( &device, &mut data )?;
    create_texture_sampler( &device, &mut data )?;
    // load_model( &mut data, "cube" )?;
    // load_model( &mut data, "src/resources/cube.obj" )?;
    // load_model( &mut data, "src/resources/viking_room.obj" )?;
    load_model( &mut data, "src/resources/barrel.obj" )?;
    // load_model( &mut data, "src/resources/bunny.obj" )?;
    create_vertex_buffer( &instance, &device, &mut data )?;
    create_index_buffer( &instance, &device, &mut data )?;
    create_uniform_buffers( &instance, &device, &mut data )?;
    create_descriptor_pool( &device, &mut data )?;
    create_descriptor_sets( &device, &mut data )?;
    create_command_buffers( &device, &mut data )?;
    create_sync_objects( &device, &mut data )?;

    Ok( Self {
      entry, instance, data, device,
      models: 4,
      frame: 0,
      resized: false,
      focused: true,
      last_tick_time: Instant::now(),
      fps_time: Instant::now(),
      fps_count: 0,
      control_manager: ControlManager::new( point3( 0.0, 0.0, 6.0 ), point3( 0.0, 0.0, 0.0 ) ),
      settings: AppSettings::new(),
    } )
  }

  unsafe fn destroy( &mut self ) {
    self.device.device_wait_idle().unwrap();

    self.destroy_swapchain();

    self.device.destroy_sampler( self.data.texture_sampler, None );
    self.device.destroy_image_view( self.data.texture_image_view, None );

    self.device.destroy_image( self.data.texture_image, None );
    self.device.free_memory( self.data.texture_image_memory, None );

    self.device.destroy_descriptor_set_layout( self.data.descriptor_set_layout, None );

    self.device.destroy_buffer( self.data.vertex_buffer, None );
    self.device.free_memory( self.data.vertex_buffer_memory, None );

    self.device.destroy_buffer( self.data.index_buffer, None );
    self.device.free_memory( self.data.index_buffer_memory, None );

    self.data.in_flight_fences.iter().for_each( |f| self.device.destroy_fence( *f, None ) );
    self.data.render_finished_semaphores.iter().for_each( |s| self.device.destroy_semaphore( *s, None ) );
    self.data.image_available_semaphores.iter().for_each( |s| self.device.destroy_semaphore( *s, None ) );
    self.data.command_pools.iter().for_each( |f| self.device.destroy_command_pool( *f, None ) );

    self.device.destroy_command_pool( self.data.command_pool, None );
    self.device.destroy_device( None );
    self.instance.destroy_surface_khr( self.data.surface, None );

    if VALIDATION_ENABLED {
      self.instance.destroy_debug_utils_messenger_ext( self.data.messenger, None );
    }

    self.instance.destroy_instance( None );
  }

  unsafe fn recreate_swapchain(&mut self, window:&Window) -> Result<()> {
    self.device.device_wait_idle()?;
    self.destroy_swapchain();

    create_swapchain(window, &self.instance, &self.device, &mut self.data)?;
    create_swapchain_image_views( &self.device, &mut self.data )?;
    create_render_pass( &self.instance, &self.device, &mut self.data )?;
    create_pipeline( &self.device, &mut self.data )?;
    create_color_objects( &self.instance, &self.device, &mut self.data )?;
    create_depth_objects( &self.instance, &self.device, &mut self.data )?;
    create_framebuffers( &self.device, &mut self.data )?;
    create_uniform_buffers( &self.instance, &self.device, &mut self.data )?;
    create_descriptor_pool( &self.device, &mut self.data )?;
    create_descriptor_sets( &self.device, &mut self.data )?;
    create_command_buffers( &self.device, &mut self.data )?;

    self.data.images_in_flight.resize( self.data.swapchain_images.len() , vk::Fence::null() );

    Ok(())
  }

  unsafe fn destroy_swapchain( &mut self ) {
    self.device.destroy_descriptor_pool( self.data.descriptor_pool, None );
    self.data.uniform_buffers_memory.iter().for_each( |m| self.device.free_memory( *m, None ) );
    self.data.uniform_buffers.iter().for_each( |b| self.device.destroy_buffer( *b, None ) );

    self.device.destroy_image_view( self.data.depth_image_view, None );
    self.device.free_memory( self.data.depth_image_memory, None );
    self.device.destroy_image( self.data.depth_image, None );

    self.device.destroy_image_view( self.data.color_image_view, None );
    self.device.free_memory( self.data.color_image_memory, None );
    self.device.destroy_image( self.data.color_image, None );

    self.data.framebuffers.iter().for_each( |f| self.device.destroy_framebuffer( *f, None ) );

    self.device.destroy_pipeline( self.data.pipeline, None );
    self.device.destroy_pipeline_layout( self.data.pipeline_layout, None );
    self.device.destroy_render_pass( self.data.render_pass, None );

    self.data.swapchain_image_views.iter().for_each( |v| self.device.destroy_image_view( *v, None ) );

    self.device.destroy_swapchain_khr( self.data.swapchain, None );
  }



  unsafe fn render( &mut self, window:&Window) -> Result<()> {
    self.device.wait_for_fences( &[ self.data.in_flight_fences[ self.frame ] ], true, u64::MAX )?;

    let result = self.device.acquire_next_image_khr(
      self.data.swapchain,
      u64::MAX,
      self.data.image_available_semaphores[ self.frame ],
      vk::Fence::null(),
    );

    let image_index = match result {
      Ok(( image_index, _ )) => image_index as usize,
      Err( vk::ErrorCode::OUT_OF_DATE_KHR ) => return self.recreate_swapchain( window ),
      Err( e ) => return Err( anyhow!( e ) ),
    };

    if !self.data.images_in_flight[ image_index ].is_null() {
      self.device.wait_for_fences( &[ self.data.images_in_flight[ image_index ] ], true, u64::MAX )?;
    }

    self.data.images_in_flight[ image_index ] = self.data.in_flight_fences[ self.frame ];

    self.update_command_buffer( image_index )?;
    self.update_uniform_buffer( image_index )?;

    let wait_semaphores = &[ self.data.image_available_semaphores[ self.frame ] ];
    let wait_stages = &[ vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT ];
    let command_buffers = &[ self.data.command_buffers[ image_index ] ];
    let signal_semaphores = &[ self.data.render_finished_semaphores[ self.frame ] ];
    let submit_info = vk::SubmitInfo::builder()
      .wait_semaphores( wait_semaphores )
      .wait_dst_stage_mask( wait_stages )
      .command_buffers( command_buffers )
      .signal_semaphores( signal_semaphores );

    self.device.reset_fences( &[ self.data.in_flight_fences[ self.frame ] ] )?;
    self.device.queue_submit(
      self.data.graphics_queue,
      &[ submit_info ],
      self.data.in_flight_fences[ self.frame ],
    )?;

    let swapchains = &[ self.data.swapchain ];
    let image_indices = &[ image_index as u32 ];
    let present_info = vk::PresentInfoKHR::builder()
      .wait_semaphores( signal_semaphores )
      .swapchains( swapchains )
      .image_indices( image_indices );

    let result = self.device.queue_present_khr( self.data.present_queue, &present_info );
    let changed = result == Ok( vk::SuccessCode::SUBOPTIMAL_KHR ) || result == Err( vk::ErrorCode::OUT_OF_DATE_KHR );

    if self.resized || changed {
      self.resized = false;
      self.recreate_swapchain( window )?;
    } else if let Err( e ) = result {
      return Err( anyhow!( e ) );
    }

    self.frame = (self.frame + 1) % MAX_FRAMES_IN_FLIGHT;

    Ok(())
  }

  unsafe fn update_uniform_buffer( &mut self, image_index:usize ) -> Result<()> {
    // let view = Mat4::look_at_rh(
    //   point3( 0.0, 0.0, 6.0 ),
    //   point3( 0.0, 0.0, 0.0 ),
    //   vec3( 0.0, 1.0, 0.0 ),
    // );

    // self.control_manager.update_position_by_velocity();
    // self.control_manager.update_target_position_by_mouse();

    // let view = Mat4::look_at_rh( self.control_manager.position, self.control_manager.target_position, Vec3::unit_y() );

    let frame_duration = Duration::from_secs_f64( (1 / 60) as f64 );
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

    let view = self.control_manager.get_view_matrix();
    let correction = Mat4::new(
      1.0,  0.0,       0.0, 0.0,
      0.0, -1.0,       0.0, 0.0,
      0.0,  0.0, 1.0 / 2.0, 0.0,
      0.0,  0.0, 1.0 / 2.0, 1.0,
    );

    let proj = correction * cgmath::perspective(
      Deg( 45.0 ),
      self.data.swapchain_extent.width as f32 / self.data.swapchain_extent.height as f32,
      0.1,
      100.0,
    );

    // self.control_manager.update_last_mouse_position();

    let ubo = UniformBufferObject { view, proj };

    let memory = self.device.map_memory(
      self.data.uniform_buffers_memory[ image_index ],
      0,
      size_of::<UniformBufferObject>() as u64,
      vk::MemoryMapFlags::empty()
    )?;

    memcpy( &ubo, memory.cast(), 1 );

    self.device.unmap_memory( self.data.uniform_buffers_memory[ image_index ] );

    Ok(())
  }

  unsafe fn update_command_buffer( &mut self, image_index:usize ) -> Result<()> {
    let command_pool = self.data.command_pools[ image_index ];
    self.device.reset_command_pool( command_pool, vk::CommandPoolResetFlags::empty() )?;

    let command_buffer = self.data.command_buffers[ image_index ];

    let inheritance = vk::CommandBufferInheritanceInfo::builder();

    let begin_info = vk::CommandBufferBeginInfo::builder()
      .flags( vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT );

    self.device.begin_command_buffer( command_buffer, &begin_info )?;

    let render_area = vk::Rect2D::builder()
      .offset( vk::Offset2D::default() )
      .extent( self.data.swapchain_extent );

    let color_clear_value = vk::ClearValue {
      color: vk::ClearColorValue {
        float32: [ 0.0, 0.0, 0.0, 1.0 ],
      },
    };

    let depth_clear_value = vk::ClearValue {
      depth_stencil: vk::ClearDepthStencilValue {
        depth: 1.0,
        stencil: 0,
      }
    };

    let clear_values = &[ color_clear_value, depth_clear_value ];
    let render_pass_begin = vk::RenderPassBeginInfo::builder()
      .render_pass( self.data.render_pass )
      .framebuffer( self.data.framebuffers[ image_index ])
      .render_area( render_area )
      .clear_values( clear_values );

    self.device.cmd_begin_render_pass( command_buffer, &render_pass_begin, vk::SubpassContents::SECONDARY_COMMAND_BUFFERS );

    let secondary_command_buffers = (0..self.models)
      .map( |i| self.update_secondary_command_buffer( image_index, i ) )
      .collect::<Result<Vec<_>, _>>()?;

    self.device.cmd_execute_commands( command_buffer, &secondary_command_buffers );
    self.device.cmd_end_render_pass( command_buffer );
    self.device.end_command_buffer( command_buffer )?;

    Ok(())
  }

  unsafe fn update_secondary_command_buffer( &mut self, image_index:usize, model_index:usize) -> Result<vk::CommandBuffer> {
    // self.data.secondary_command_buffers.resize_with( image_index + 1, Vec::new );

    let command_buffers = &mut self.data.secondary_command_buffers[ image_index ];

    while model_index >= command_buffers.len() {
      let allocate_info = vk::CommandBufferAllocateInfo::builder()
        .command_pool( self.data.command_pools[ image_index ] )
        .level( vk::CommandBufferLevel::SECONDARY )
        .command_buffer_count( 1 );

      let command_buffer = self.device.allocate_command_buffers( &allocate_info )?[ 0 ];
      command_buffers.push( command_buffer );
    }

    let command_buffer = command_buffers[ model_index ];

    // Model

    let x = (((model_index % 2) as f32) *  2.5) - 1.25;
    let y = (((model_index / 2) as f32) * -2.5) + 1.0;
    let time = self.last_tick_time.elapsed().as_secs_f32();

    let model = Mat4::from_translation( vec3( x, y, 0.0 ) );
    // * Mat4::from_axis_angle(
    //   vec3( 0.0, 0.0, 1.0 ),
    //   Deg( 90.0 ) * time
    // );

    let model_bytes = std::slice::from_raw_parts(
      &model as *const Mat4 as *const u8,
      size_of::<Mat4>()
    );

    let opacity = (model_index + 1) as f32 * 0.25;
    // let opacity = (4 - model_index) as f32 * 0.25;
    let opacity_bytes = &opacity.to_ne_bytes()[..];

    //

    let inheritance_info = vk::CommandBufferInheritanceInfo::builder()
      .render_pass( self.data.render_pass )
      .subpass( 0 )
      .framebuffer( self.data.framebuffers[ image_index ] );

    let info = vk::CommandBufferBeginInfo::builder()
      .flags( vk::CommandBufferUsageFlags::RENDER_PASS_CONTINUE )
      .inheritance_info( &inheritance_info );

    self.device.begin_command_buffer( command_buffer, &info )?;

    self.device.cmd_bind_pipeline( command_buffer, vk::PipelineBindPoint::GRAPHICS, self.data.pipeline );
    self.device.cmd_bind_vertex_buffers( command_buffer, 0, &[ self.data.vertex_buffer ], &[ 0 ] );
    self.device.cmd_bind_index_buffer( command_buffer, self.data.index_buffer, 0, vk::IndexType::UINT32 );

    self.device.cmd_bind_descriptor_sets(
      command_buffer,
      vk::PipelineBindPoint::GRAPHICS,
      self.data.pipeline_layout,
      0,
      &[ self.data.descriptor_sets[ image_index ] ],
      &[]
    );

    self.device.cmd_push_constants(
      command_buffer,
      self.data.pipeline_layout,
      vk::ShaderStageFlags::VERTEX,
      0,
      model_bytes,
    );

    self.device.cmd_push_constants(
      command_buffer,
      self.data.pipeline_layout,
      vk::ShaderStageFlags::FRAGMENT,
      64,
      opacity_bytes,
    );

    self.device.cmd_draw_indexed( command_buffer, self.data.indices.len() as u32, 1, 0, 0, 0 );
    self.device.end_command_buffer( command_buffer )?;

    Ok( command_buffer )
  }
}



#[derive(Clone, Debug, Default)]
struct AppData {
  surface: vk::SurfaceKHR,
  messenger: vk::DebugUtilsMessengerEXT,
  physical_device: vk::PhysicalDevice,
  msaa_samples: vk::SampleCountFlags,
  graphics_queue: vk::Queue,
  present_queue: vk::Queue,
  swapchain: vk::SwapchainKHR,
  swapchain_format: vk::Format,
  swapchain_extent: vk::Extent2D,
  swapchain_images: Vec<vk::Image>,
  swapchain_image_views: Vec<vk::ImageView>,
  render_pass: vk::RenderPass,
  descriptor_set_layout: vk::DescriptorSetLayout,
  pipeline_layout: vk::PipelineLayout,
  pipeline: vk::Pipeline,
  framebuffers: Vec<vk::Framebuffer>,
  command_pool: vk::CommandPool,
  command_pools: Vec<vk::CommandPool>,
  command_buffers: Vec<vk::CommandBuffer>,
  secondary_command_buffers: Vec<Vec<vk::CommandBuffer>>,
  image_available_semaphores: Vec<vk::Semaphore>,
  render_finished_semaphores: Vec<vk::Semaphore>,
  in_flight_fences: Vec<vk::Fence>,
  images_in_flight: Vec<vk::Fence>,
  vertices: Vec<Vertex>,
  indices: Vec<u32>,
  vertex_buffer: vk::Buffer,
  vertex_buffer_memory: vk::DeviceMemory,
  index_buffer: vk::Buffer,
  index_buffer_memory: vk::DeviceMemory,
  uniform_buffers: Vec<vk::Buffer>,
  uniform_buffers_memory: Vec<vk::DeviceMemory>,
  descriptor_pool: vk::DescriptorPool,
  descriptor_sets: Vec<vk::DescriptorSet>,
  mip_levels: u32,
  color_image: vk::Image,
  color_image_memory: vk::DeviceMemory,
  color_image_view: vk::ImageView,
  texture_image: vk::Image,
  texture_image_memory: vk::DeviceMemory,
  texture_image_view: vk::ImageView,
  texture_sampler: vk::Sampler,
  depth_image: vk::Image,
  depth_image_memory: vk::DeviceMemory,
  depth_image_view: vk::ImageView,
}



#[derive(Debug, Error)]
#[error("Missing {0}.")]
pub struct SuitabilityError(pub &'static str);



#[derive(Copy, Clone, Debug)]
struct QueueFamilyIndices {
  graphics: u32,
  present: u32,
}

impl QueueFamilyIndices {
  unsafe fn get( instance:&Instance, data:&AppData, physical_device:vk::PhysicalDevice ) -> Result<Self> {
    let properties = instance.get_physical_device_queue_family_properties( physical_device );

    let graphics = properties
      .iter()
      .position( |p| p.queue_flags.contains( vk::QueueFlags::GRAPHICS ) )
      .map( |i| i as u32 );

    let mut present = None;
    for (index, properties) in properties.iter().enumerate() {
      if instance.get_physical_device_surface_support_khr( physical_device, index as u32, data.surface )? {
        present = Some(index as u32);
        break;
      }
    }

    if let (Some( graphics ), Some( present )) = (graphics, present) {
      Ok( Self { graphics, present } )
    } else {
      Err( anyhow!( SuitabilityError( "Missing required queue families." ) ) )
    }
  }
}


#[derive(Clone, Debug)]
struct SwapchainSupport {
  capabilities: vk::SurfaceCapabilitiesKHR,
  formats: Vec<vk::SurfaceFormatKHR>,
  present_modes: Vec<vk::PresentModeKHR>,
}

impl SwapchainSupport {
  unsafe fn get( instance:&Instance, data:&AppData, physical_device:vk::PhysicalDevice ) -> Result<Self> {
    Ok( Self {
      capabilities: instance.get_physical_device_surface_capabilities_khr( physical_device, data.surface )?,
      formats: instance.get_physical_device_surface_formats_khr( physical_device, data.surface )?,
      present_modes: instance.get_physical_device_surface_present_modes_khr( physical_device, data.surface )?,
    } )
  }
}



#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct Vertex {
  pos: Vec3,
  color: Vec3,
  tex_coord: Vec2,
}

impl Vertex {
  const fn new( pos:Vec3, color:Vec3, tex_coord:Vec2 ) -> Self {
    Self { pos, color, tex_coord }
  }

  fn binding_description() -> vk::VertexInputBindingDescription {
    vk::VertexInputBindingDescription::builder()
      .binding( 0 )
      .stride( size_of::<Vertex>() as u32 )
      .input_rate( vk::VertexInputRate::VERTEX )
      .build()
  }

  fn attribute_description() -> [ vk::VertexInputAttributeDescription; 3 ] {
    let pos = vk::VertexInputAttributeDescription::builder()
      .binding( 0 )
      .location( 0 )
      .format( vk::Format::R32G32B32_SFLOAT )
      .offset( 0 )
      .build();

    let color = vk::VertexInputAttributeDescription::builder()
      .binding( 0 )
      .location( 1 )
      .format( vk::Format::R32G32B32_SFLOAT )
      .offset( size_of::<Vec3>() as u32 )
      .build();

    let tex_coord = vk::VertexInputAttributeDescription::builder()
      .binding( 0 )
      .location( 2 )
      .format( vk::Format::R32G32_SFLOAT )
      .offset( (size_of::<Vec3>() + size_of::<Vec3>()) as u32 )
      .build();

    [ pos, color, tex_coord ]
  }
}

impl PartialEq for Vertex {
  fn eq( &self, other:&Self ) -> bool {
    self.pos == other.pos && self.color == other.color && self.tex_coord == other.tex_coord
  }
}

impl Eq for Vertex {}

impl Hash for Vertex {
  fn hash<H:Hasher>( &self, state:&mut H ) {
    self.pos[ 0 ].to_bits().hash( state );
    self.pos[ 1 ].to_bits().hash( state );
    self.pos[ 2 ].to_bits().hash( state );
    self.color[ 0 ].to_bits().hash( state );
    self.color[ 1 ].to_bits().hash( state );
    self.color[ 2 ].to_bits().hash( state );
    self.tex_coord[ 0 ].to_bits().hash( state );
    self.tex_coord[ 1 ].to_bits().hash( state );
  }
}



#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct UniformBufferObject {
  view: Mat4,
  proj: Mat4,
}


fn load_model( data:&mut AppData, src:&str ) -> Result<()> {
  if src == "cube" {
    data.vertices = VERTICES.into();
    data.indices = INDICES.into();
    return Ok(())
  }

  let mut unique_vertices = HashMap::new();
  let mut reader = BufReader::new( File::open( src )? );

  let ( models, _ ) = tobj::load_obj_buf(
    &mut reader,
    &tobj::LoadOptions { triangulate:true, single_index:true, ..Default::default() },
    |_| std::result::Result::Ok( Default::default() ),
  )?;

  let get_min_max = |a:(f32, f32), b:f32| (
    if a.0 > b { b } else { a.0 },
    if a.1 > b { a.1 } else { b },
  );

  for model in models {
    let mut min_max_x = (0.0, 0.0);
    let mut min_max_y = (0.0, 0.0);
    let mut min_max_z = (0.0, 0.0);

    for index in model.mesh.indices.clone() {
      let pos_offset = (3 * index) as usize;

      min_max_x = get_min_max( min_max_x, model.mesh.positions[ pos_offset + 0 ] );
      min_max_y = get_min_max( min_max_x, model.mesh.positions[ pos_offset + 1 ] );
      min_max_z = get_min_max( min_max_x, model.mesh.positions[ pos_offset + 2 ] );
    }

    for index in model.mesh.indices {
      let pos_offset = (3 * index) as usize;
      let tex_coord_offset = (2 * index) as usize;

      let vertex = Vertex {
        pos: vec3(
          model.mesh.positions[ pos_offset + 0 ],
          model.mesh.positions[ pos_offset + 1 ],
          model.mesh.positions[ pos_offset + 2 ],
        ),
        color: vec3( 1.0, 1.0, 1.0 ),
        tex_coord: if model.mesh.texcoords.is_empty() { vec2( 0.0, 0.0 ) } else {
          vec2(
            model.mesh.texcoords[ tex_coord_offset + 0 ],
            1.0 - model.mesh.texcoords[ tex_coord_offset + 1 ],
          )
        },
      };

      if let Some( index ) = unique_vertices.get( &vertex ) {
        data.indices.push( *index as u32 )
      } else {
        let index = data.vertices.len();
        unique_vertices.insert( vertex, index );
        data.vertices.push( vertex );
        data.indices.push( index as u32 )
      }
    }
  }

  Ok(())
}

unsafe fn create_instance( window:&Window, entry:&Entry, data:&mut AppData ) -> Result<Instance> {
  let application_info = vk::ApplicationInfo::builder()
    .application_name( b"Vulkan Tutorial\0" )
    .application_version( vk::make_version( 1, 0, 0 ) )
    .engine_name( b"No Engine\0" )
    .engine_version( vk::make_version( 1, 0, 0 ) )
    .api_version( vk::make_version( 1, 0, 0 ) );

  let available_layers = entry
    .enumerate_instance_layer_properties()?
    .iter()
    .map( |l| l.layer_name )
    .collect::<HashSet<_>>();

  if VALIDATION_ENABLED && !available_layers.contains( &VALIDATION_LAYER ) {
    return Err( anyhow!( "Validation layer requested but not supported." ) );
  }

  let layers = if VALIDATION_ENABLED {
    vec![ VALIDATION_LAYER.as_ptr() ]
  } else {
    Vec::new()
  };

  let mut extensions = vk_window::get_required_instance_extensions( window )
    .iter()
    .map( |e| e.as_ptr() )
    .collect::<Vec<_>>();

  let flags = if cfg!(target_os = "macos") && entry.version()? >= PORTABILITY_MACOS_VERSION {
    extensions.push( vk::KHR_GET_PHYSICAL_DEVICE_PROPERTIES2_EXTENSION.name.as_ptr() );
    extensions.push( vk::KHR_PORTABILITY_ENUMERATION_EXTENSION.name.as_ptr() );
    vk::InstanceCreateFlags::ENUMERATE_PORTABILITY_KHR
  } else {
    vk::InstanceCreateFlags::empty()
  };

  if VALIDATION_ENABLED {
    extensions.push( vk::EXT_DEBUG_UTILS_EXTENSION.name.as_ptr() );
  }

  let mut info = vk::InstanceCreateInfo::builder()
    .application_info( &application_info )
    .enabled_extension_names( &extensions )
    .enabled_layer_names( &layers )
    .flags( flags );

  let mut debug_info = vk::DebugUtilsMessengerCreateInfoEXT::builder()
    .message_severity( vk::DebugUtilsMessageSeverityFlagsEXT::all() )
    .message_type( vk::DebugUtilsMessageTypeFlagsEXT::all() )
    .user_callback( Some( debug_callback ) );

  if VALIDATION_ENABLED {
    info = info.push_next( &mut debug_info );
  }

  let instance = entry.create_instance( &info, None )?;

  if VALIDATION_ENABLED {
    data.messenger = instance.create_debug_utils_messenger_ext( &debug_info, None )?;
  }

  Ok( instance )
}

unsafe fn get_max_msaa_samples( instance:&Instance, data:&mut AppData ) -> vk::SampleCountFlags {
  let properties = instance.get_physical_device_properties( data.physical_device );
  let counts = properties.limits.sampled_image_color_sample_counts & properties.limits.sampled_image_depth_sample_counts;

  [
    vk::SampleCountFlags::_64,
    vk::SampleCountFlags::_32,
    vk::SampleCountFlags::_16,
    vk::SampleCountFlags::_8,
    vk::SampleCountFlags::_4,
    vk::SampleCountFlags::_2,
    vk::SampleCountFlags::_1,
  ]
    .iter()
    .cloned()
    .find( |c| counts.contains( *c ) )
    .unwrap_or( vk::SampleCountFlags::_1 )
}

unsafe fn pick_physical_device( instance:&Instance, data:&mut AppData ) -> Result<()> {
  for physical_device in instance.enumerate_physical_devices()? {
    let properties = instance.get_physical_device_properties( physical_device );

    if let Err( error ) = check_physical_device( instance, data, physical_device ) {
      warn!( "Skipping physical device (`{}`): {}", properties.device_name, error )
    } else {
      info!( "Selected physical device (`{}`).", properties.device_name );
      data.physical_device = physical_device;
      data.msaa_samples = get_max_msaa_samples( instance, data );
      return Ok(());
    }
  }

  Err( anyhow!( "Failed to find suitable physical device." ) )
}

unsafe fn check_physical_device( instance:&Instance, data:&AppData, physical_device:vk::PhysicalDevice ) -> Result<()> {
  QueueFamilyIndices::get( instance, data, physical_device )?;
  check_physical_device_extensions( instance, physical_device )?;

  let support = SwapchainSupport::get( instance, data, physical_device )?;
  if support.formats.is_empty() || support.present_modes.is_empty() {
    return Err( anyhow!( SuitabilityError( "Insufficient swapchain support." ) ) );
  }

  let features = instance.get_physical_device_features( physical_device );
  if features.sampler_anisotropy != vk::TRUE {
    return Err( anyhow!( SuitabilityError( "No sampler anisotropy." ) ) );
  }

  Ok(())
}

unsafe fn check_physical_device_extensions( instance:&Instance, physical_device:vk::PhysicalDevice ) -> Result<()> {
  let extensions = instance
    .enumerate_device_extension_properties( physical_device, None )?
    .iter()
    .map( |e| e.extension_name )
    .collect::<HashSet<_>>();

  if DEVICE_EXTENSIONS.iter().all( |e| extensions.contains( e ) ) {
    Ok(())
  } else {
    Err( anyhow!( SuitabilityError( "Missing required device extensions." ) ) )
  }
}

unsafe fn create_logical_device( entry:&Entry, instance:&Instance, data:&mut AppData ) -> Result<Device> {
  let indices = QueueFamilyIndices::get( instance, data, data.physical_device )?;

  let mut unique_indices = HashSet::new();
  unique_indices.insert( indices.graphics );
  unique_indices.insert( indices.present );

  let queue_priorities = &[ 1.0 ];
  let queue_infos = unique_indices
    .iter()
    .map( |i| {
      vk::DeviceQueueCreateInfo::builder()
        .queue_family_index( *i )
        .queue_priorities( queue_priorities )
    } )
    .collect::<Vec<_>>();

  let layers = if VALIDATION_ENABLED {
    vec![ VALIDATION_LAYER.as_ptr() ]
  } else {
    vec![]
  };

  let mut extensions = DEVICE_EXTENSIONS
    .iter()
    .map( |n| n.as_ptr() )
    .collect::<Vec<_>>();

  // Required by Vulkan SDK on macOS since 1.3.216.
  if cfg!(target_os = "macos") && entry.version()? >= PORTABILITY_MACOS_VERSION {
      extensions.push( vk::KHR_PORTABILITY_SUBSET_EXTENSION.name.as_ptr() );
  }

  let features = vk::PhysicalDeviceFeatures::builder()
    .sampler_anisotropy( true )
    .sample_rate_shading( true );

  let info = vk::DeviceCreateInfo::builder()
    .queue_create_infos( &queue_infos )
    .enabled_layer_names( &layers )
    .enabled_extension_names( &extensions )
    .enabled_features( &features );

  let device = instance.create_device( data.physical_device, &info, None )?;

  data.graphics_queue = device.get_device_queue( indices.graphics, 0 );
  data.present_queue = device.get_device_queue( indices.present, 0 );

  Ok( device )
}

fn get_swapchain_surface_format( formats:&[vk::SurfaceFormatKHR] ) -> vk::SurfaceFormatKHR {
  formats
    .iter()
    .cloned()
    .find( |f| f.format == vk::Format::B8G8R8A8_SRGB && f.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR )
    .unwrap_or_else( || formats[ 0 ] )
}

fn get_swapchain_present_mode( present_modes:&[vk::PresentModeKHR] ) -> vk::PresentModeKHR {
  present_modes
    .iter()
    .cloned()
    .find( |m| *m == vk::PresentModeKHR::MAILBOX )
    .unwrap_or( vk::PresentModeKHR::FIFO )
}

fn get_swapchain_extent( window:&Window, capabilities:vk::SurfaceCapabilitiesKHR ) -> vk::Extent2D {
  if capabilities.current_extent.width != u32::MAX {
    capabilities.current_extent
} else {
    let size = window.inner_size();
    let clamp = |min:u32, max:u32, v:u32| min.max( max.min( v ) );

    vk::Extent2D::builder()
      .width( clamp(
        capabilities.min_image_extent.width,
        capabilities.max_image_extent.width,
        size.width,
      ) )
      .height( clamp(
        capabilities.min_image_extent.height,
        capabilities.max_image_extent.height,
        size.height,
      ) )
      .build()
  }
}

unsafe fn get_supported_format( instance:&Instance, data:&AppData, candidates: &[vk::Format], tiling: vk::ImageTiling, features:vk::FormatFeatureFlags ) -> Result<vk::Format> {
  candidates
    .iter()
    .cloned()
    .find( |f| {
      let properties = instance.get_physical_device_format_properties( data.physical_device, *f );

      match tiling {
        vk::ImageTiling::LINEAR => properties.linear_tiling_features.contains( features ),
        vk::ImageTiling::OPTIMAL => properties.optimal_tiling_features.contains( features ),
        _ => false,
      }
    } )
    .ok_or_else( || anyhow!( "Failed to find supported format!" ) )
}

unsafe fn get_depth_format( instance:&Instance, data:&AppData ) -> Result<vk::Format> {
  let candidates = &[
    vk::Format::D32_SFLOAT,
    vk::Format::D32_SFLOAT_S8_UINT,
    vk::Format::D24_UNORM_S8_UINT,
  ];

  get_supported_format( instance, data, candidates, vk::ImageTiling::OPTIMAL, vk::FormatFeatureFlags::DEPTH_STENCIL_ATTACHMENT )
}

unsafe fn get_memory_type_index( instance:&Instance, data:&mut AppData, properties:vk::MemoryPropertyFlags, requirements:vk::MemoryRequirements ) -> Result<u32> {
  let memory = instance.get_physical_device_memory_properties( data.physical_device );

  (0..memory.memory_type_count)
    .find( |i| {
      let suitable = (requirements.memory_type_bits & (1 << i)) != 0;
      let memory_type = memory.memory_types[ *i as usize ];
      suitable && memory_type.property_flags.contains( properties )
    } )
    .ok_or_else( || anyhow!( "Failed to find suitable memory type." ) )
}


unsafe fn create_swapchain( window:&Window, instance:&Instance, device:&Device, data:&mut AppData ) -> Result<()> {
  let indices = QueueFamilyIndices::get( instance, data, data.physical_device )?;
  let support = SwapchainSupport::get( instance, data, data.physical_device )?;

  let surface_format = get_swapchain_surface_format( &support.formats );
  let present_mode = get_swapchain_present_mode( &support.present_modes );
  let extent = get_swapchain_extent( window, support.capabilities );

  let mut image_count = support.capabilities.min_image_count + 1;
  if support.capabilities.max_image_count != 0 && image_count > support.capabilities.max_image_count {
    image_count = support.capabilities.max_image_count;
  }

  let mut queue_family_indices = vec![];
  let image_sharing_mode = if indices.graphics != indices.present {
    queue_family_indices.push( indices.graphics );
    queue_family_indices.push( indices.present );
    vk::SharingMode::CONCURRENT
  } else {
    vk::SharingMode::EXCLUSIVE
  };

  let info = vk::SwapchainCreateInfoKHR::builder()
    .surface( data.surface )
    .min_image_count( image_count )
    .image_format( surface_format.format )
    .image_color_space( surface_format.color_space )
    .image_extent( extent )
    .image_array_layers( 1 )
    .image_usage( vk::ImageUsageFlags::COLOR_ATTACHMENT )
    .image_sharing_mode( image_sharing_mode )
    .queue_family_indices( &queue_family_indices )
    .pre_transform( support.capabilities.current_transform )
    .composite_alpha( vk::CompositeAlphaFlagsKHR::OPAQUE )
    .present_mode( present_mode )
    .clipped( true )
    .old_swapchain( vk::SwapchainKHR::null() );

  data.swapchain = device.create_swapchain_khr( &info, None )?;
  data.swapchain_images = device.get_swapchain_images_khr( data.swapchain )?;
  data.swapchain_format = surface_format.format;
  data.swapchain_extent = extent;

  Ok(())
}

unsafe fn create_swapchain_image_views( device:&Device, data:&mut AppData ) -> Result<()> {
  data.swapchain_image_views = data
    .swapchain_images
    .iter()
    .map( |i| create_image_view( device, *i, data.swapchain_format, vk::ImageAspectFlags::COLOR, 1 ) )
    .collect::<Result<Vec<_>, _>>()?;

  Ok(())
}

unsafe fn create_render_pass( instance:&Instance, device:&Device, data:&mut AppData ) -> Result<()>{
  let color_attachment = vk::AttachmentDescription::builder()
    .format( data.swapchain_format )
    .samples( data.msaa_samples )
    .load_op( vk::AttachmentLoadOp::CLEAR )
    .store_op( vk::AttachmentStoreOp::STORE )
    .stencil_load_op( vk::AttachmentLoadOp::DONT_CARE )
    .stencil_store_op( vk::AttachmentStoreOp::DONT_CARE )
    .initial_layout( vk::ImageLayout::UNDEFINED )
    .final_layout( vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL );

    let color_attachment_ref = vk::AttachmentReference::builder()
      .attachment( 0 )
      .layout( vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL );

  let depth_stencil_attachment = vk::AttachmentDescription::builder()
    .format( get_depth_format( instance, data )? )
    .samples( data.msaa_samples )
    .load_op( vk::AttachmentLoadOp::CLEAR )
    .store_op( vk::AttachmentStoreOp::DONT_CARE )
    .stencil_load_op( vk::AttachmentLoadOp::DONT_CARE )
    .stencil_store_op( vk::AttachmentStoreOp::DONT_CARE )
    .initial_layout( vk::ImageLayout::UNDEFINED )
    .final_layout( vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL );

  let depth_stencil_attachment_ref = vk::AttachmentReference::builder()
    .attachment( 1 )
    .layout( vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL );

  let color_resolve_attachment = vk::AttachmentDescription::builder()
    .format( data.swapchain_format )
    .samples( vk::SampleCountFlags::_1 )
    .load_op( vk::AttachmentLoadOp::DONT_CARE )
    .store_op( vk::AttachmentStoreOp::STORE )
    .stencil_load_op( vk::AttachmentLoadOp::DONT_CARE )
    .stencil_store_op( vk::AttachmentStoreOp::DONT_CARE )
    .initial_layout( vk::ImageLayout::UNDEFINED )
    .final_layout( vk::ImageLayout::PRESENT_SRC_KHR );

  let color_resolve_attachment_ref = vk::AttachmentReference::builder()
    .attachment( 2 )
    .layout( vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL );

  let color_attachments = &[ color_attachment_ref ];
  let resolve_attachments = &[ color_resolve_attachment_ref ];
  let subpass = vk::SubpassDescription::builder()
    .pipeline_bind_point( vk::PipelineBindPoint::GRAPHICS )
    .color_attachments( color_attachments )
    .depth_stencil_attachment( &depth_stencil_attachment_ref )
    .resolve_attachments( resolve_attachments );

  let dependency = vk::SubpassDependency::builder()
    .src_subpass( vk::SUBPASS_EXTERNAL )
    .dst_subpass( 0 )
    .src_stage_mask( vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS )
    .src_access_mask( vk::AccessFlags::empty() )
    .dst_stage_mask( vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS )
    .dst_access_mask( vk::AccessFlags::COLOR_ATTACHMENT_WRITE | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE );

  let attachments = &[ color_attachment, depth_stencil_attachment, color_resolve_attachment ];
  let subpasses = &[ subpass ];
  let dependencies = &[ dependency ];
  let info = vk::RenderPassCreateInfo::builder()
    .attachments( attachments )
    .subpasses( subpasses )
    .dependencies( dependencies );

  data.render_pass = device.create_render_pass( &info, None )?;

  Ok(())
}

unsafe fn create_descriptor_set_layout( device:&Device, data:&mut AppData ) -> Result<()> {
  let ubo_binding = vk::DescriptorSetLayoutBinding::builder()
    .binding( 0 )
    .descriptor_type( vk::DescriptorType::UNIFORM_BUFFER )
    .descriptor_count( 1 )
    .stage_flags( vk::ShaderStageFlags::VERTEX );

  let sampler_binding = vk::DescriptorSetLayoutBinding::builder()
    .binding( 1 )
    .descriptor_type( vk::DescriptorType::COMBINED_IMAGE_SAMPLER )
    .descriptor_count( 1 )
    .stage_flags( vk::ShaderStageFlags::FRAGMENT );

  let bindings = &[ ubo_binding, sampler_binding ];
  let info = vk::DescriptorSetLayoutCreateInfo::builder()
    .bindings( bindings );

  data.descriptor_set_layout = device.create_descriptor_set_layout( &info, None )?;

  Ok(())
}

unsafe fn create_pipeline( device:&Device, data:&mut AppData ) -> Result<()> {
  let vert = include_bytes!( "./shaders/vert.spv" );
  let frag = include_bytes!( "./shaders/frag.spv" );

  let vert_shader_module = create_shader_module( device, vert )?;
  let frag_shader_module = create_shader_module( device, frag )?;

  let vert_stage = vk::PipelineShaderStageCreateInfo::builder()
    .stage( vk::ShaderStageFlags::VERTEX )
    .module( vert_shader_module )
    .name( b"main\0" );

  let frag_stage = vk::PipelineShaderStageCreateInfo::builder()
    .stage( vk::ShaderStageFlags::FRAGMENT )
    .module( frag_shader_module )
    .name( b"main\0" );

  let binding_descriptions = &[ Vertex::binding_description() ];
  let attribute_descriptions = Vertex::attribute_description();
  let vertex_input_state = vk::PipelineVertexInputStateCreateInfo::builder()
    .vertex_binding_descriptions( binding_descriptions )
    .vertex_attribute_descriptions( &attribute_descriptions );

  let input_assembly_state = vk::PipelineInputAssemblyStateCreateInfo::builder()
    .topology( vk::PrimitiveTopology::TRIANGLE_LIST )
    .primitive_restart_enable( false );

  let viewport = vk::Viewport::builder()
    .x( 0.0 )
    .y( 0.0 )
    .width( data.swapchain_extent.width as f32 )
    .height( data.swapchain_extent.height as f32 )
    .min_depth( 0.0 )
    .max_depth( 1.0 );

  let scissor = vk::Rect2D::builder()
    .offset( vk::Offset2D { x:0, y:0 } )
    .extent( data.swapchain_extent );

  let viewports = &[ viewport ];
  let scissors = &[ scissor ];
  let viewport_state = vk::PipelineViewportStateCreateInfo::builder()
    .viewports( viewports )
    .scissors( scissors );

  let rasterization_state = vk::PipelineRasterizationStateCreateInfo::builder()
    .depth_clamp_enable( false )
    .rasterizer_discard_enable( false )
    .polygon_mode( vk::PolygonMode::FILL )
    .line_width( 1.0 )
    .cull_mode( vk::CullModeFlags::BACK )
    .front_face( vk::FrontFace::COUNTER_CLOCKWISE )
    .depth_bias_enable( false );

  let multisample_state = vk::PipelineMultisampleStateCreateInfo::builder()
    .rasterization_samples( data.msaa_samples )
    .sample_shading_enable( true )
    .min_sample_shading( 0.2 );

  let depth_stencil_state = vk::PipelineDepthStencilStateCreateInfo::builder()
    .depth_test_enable( true )
    .depth_write_enable( true )
    .depth_compare_op( vk::CompareOp::LESS )
    .depth_bounds_test_enable( false )
    .min_depth_bounds( 0.0 )
    .max_depth_bounds( 1.0 )
    .stencil_test_enable( false );
    // .front( vk::StencilOpState )
    // .back( vk::StencilOpState );

  let attachment = vk::PipelineColorBlendAttachmentState::builder()
    .color_write_mask( vk::ColorComponentFlags::all() )
    .blend_enable( true )
    .src_color_blend_factor( vk::BlendFactor::SRC_ALPHA )
    .dst_color_blend_factor( vk::BlendFactor::ONE_MINUS_SRC_ALPHA )
    .color_blend_op( vk::BlendOp::ADD )
    .src_alpha_blend_factor( vk::BlendFactor::ONE )
    .dst_alpha_blend_factor( vk::BlendFactor::ZERO )
    .alpha_blend_op( vk::BlendOp::ADD );

  let attachments = &[ attachment ];
  let color_blend_state = vk::PipelineColorBlendStateCreateInfo::builder()
    .logic_op_enable( false )
    .logic_op( vk::LogicOp::COPY )
    .attachments( attachments )
    .blend_constants([ 0.0, 0.0, 0.0, 0.0 ]);

  let vert_push_constant_range = vk::PushConstantRange::builder()
    .stage_flags( vk::ShaderStageFlags::VERTEX )
    .offset( 0 )
    .size( 64 );

  let frag_push_constant_range = vk::PushConstantRange::builder()
    .stage_flags( vk::ShaderStageFlags::FRAGMENT )
    .offset( 64 )
    .size( 4 );

  let set_layouts = &[ data.descriptor_set_layout ];
  let push_constant_ranges = &[ vert_push_constant_range, frag_push_constant_range ];
  let layout_info = vk::PipelineLayoutCreateInfo::builder()
    .set_layouts( set_layouts )
    .push_constant_ranges( push_constant_ranges );

  data.pipeline_layout = device.create_pipeline_layout( &layout_info, None )?;

  let stages = &[ vert_stage, frag_stage ];
  let info = vk::GraphicsPipelineCreateInfo::builder()
    .stages( stages )
    .vertex_input_state( &vertex_input_state )
    .input_assembly_state( &input_assembly_state )
    .viewport_state( &viewport_state )
    .rasterization_state( &rasterization_state )
    .multisample_state( &multisample_state )
    .depth_stencil_state( &depth_stencil_state )
    .color_blend_state( &color_blend_state )
    .layout( data.pipeline_layout )
    .render_pass( data.render_pass )
    .subpass( 0 )
    .base_pipeline_handle( vk::Pipeline::null() )
    .base_pipeline_index( -1 );

  data.pipeline = device.create_graphics_pipelines( vk::PipelineCache::null(), &[ info ], None )?.0[ 0 ];

  device.destroy_shader_module( vert_shader_module, None );
  device.destroy_shader_module( frag_shader_module, None );

  Ok(())
}

unsafe fn create_shader_module( device:&Device, bytecode:&[u8] ) -> Result<vk::ShaderModule> {
  let bytecode = Bytecode::new( bytecode ).unwrap();
  let info = vk::ShaderModuleCreateInfo::builder()
    .code_size( bytecode.code_size() )
    .code( bytecode.code() );

  Ok( device.create_shader_module( &info, None )? )
}

unsafe fn create_framebuffers( device:&Device, data:&mut AppData ) -> Result<()> {
  data.framebuffers = data
    .swapchain_image_views
    .iter()
    .map( |i| {
      let attachments = &[ data.color_image_view, data.depth_image_view, *i ];
      let create_info = vk::FramebufferCreateInfo::builder()
        .render_pass( data.render_pass )
        .attachments( attachments )
        .width( data.swapchain_extent.width )
        .height( data.swapchain_extent.height )
        .layers( 1 );

      device.create_framebuffer( &create_info, None )
    } )
    .collect::<Result<Vec<_>,_>>()?;

  Ok(())
}

unsafe fn create_command_pool( instance:&Instance, device:&Device, data:&AppData ) -> Result<vk::CommandPool> {
  let indices = QueueFamilyIndices::get( instance, data, data.physical_device )?;

  let info = vk::CommandPoolCreateInfo::builder()
    .flags( vk::CommandPoolCreateFlags::TRANSIENT )
    .queue_family_index( indices.graphics );

  Ok( device.create_command_pool( &info, None )? )
}

unsafe fn create_command_pools( instance:&Instance, device:&Device, data:&mut AppData ) -> Result<()> {
  data.command_pool = create_command_pool( instance, device, data )?;

  for _ in 0..data.swapchain_images.len() {
    data.command_pools.push( create_command_pool( instance, device, data )? );
  }

  Ok(())
}

unsafe fn create_buffer( instance:&Instance, device:&Device, data:&mut AppData, size:vk::DeviceSize, usage:vk::BufferUsageFlags, properties:vk::MemoryPropertyFlags ) -> Result<(vk::Buffer, vk::DeviceMemory)>{
  let buffer_info = vk::BufferCreateInfo::builder()
    .size( size )
    .usage( usage )
    .sharing_mode( vk::SharingMode::EXCLUSIVE )
    .flags( vk::BufferCreateFlags::empty() );

  let buffer = device.create_buffer( &buffer_info, None )?;

  let requirements = device.get_buffer_memory_requirements( buffer );
  let memory_info = vk::MemoryAllocateInfo::builder()
    .allocation_size( requirements.size )
    .memory_type_index( get_memory_type_index( instance, data, properties, requirements )? );

  let buffer_memory = device.allocate_memory( &memory_info, None )?;

  device.bind_buffer_memory( buffer, buffer_memory, 0 )?;

  Ok(( buffer, buffer_memory ))
}

unsafe fn begin_single_time_commands( device:&Device, data:&mut AppData ) -> Result<vk::CommandBuffer> {
  let info = vk::CommandBufferAllocateInfo::builder()
    .level( vk::CommandBufferLevel::PRIMARY )
    .command_pool( data.command_pool )
    .command_buffer_count( 1 );

  let command_buffer = device.allocate_command_buffers( &info )?[ 0 ];

  let info = vk::CommandBufferBeginInfo::builder()
    .flags( vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT );

  device.begin_command_buffer( command_buffer, &info )?;

  Ok( command_buffer )
}

unsafe fn end_single_time_commands( device:&Device, data:&mut AppData, command_buffer:vk::CommandBuffer ) -> Result<()> {
  device.end_command_buffer( command_buffer )?;

  let command_buffers = &[ command_buffer ];
  let info = vk::SubmitInfo::builder()
    .command_buffers( command_buffers );

  device.queue_submit( data.graphics_queue, &[ info ], vk::Fence::null() )?;
  device.queue_wait_idle( data.graphics_queue )?;
  device.free_command_buffers( data.command_pool, &[ command_buffer ] );

  Ok(())
}

unsafe fn copy_buffer( device:&Device, data:&mut AppData, source:vk::Buffer, destination:vk::Buffer, size:vk::DeviceSize ) -> Result<()> {
  let command_buffer = begin_single_time_commands( device, data )?;

  let regions = vk::BufferCopy::builder().size( size );
  device.cmd_copy_buffer( command_buffer , source, destination, &[ regions ] );

  end_single_time_commands( device, data, command_buffer )?;

  Ok(())
}

unsafe fn copy_buffer_to_image( device:&Device, data:&mut AppData, buffer:vk::Buffer, image:vk::Image, width:u32, height:u32 ) -> Result<()> {
  let command_buffer = begin_single_time_commands( device, data )?;
  let subresource = vk::ImageSubresourceLayers::builder()
    .aspect_mask( vk::ImageAspectFlags::COLOR )
    .mip_level( 0 )
    .base_array_layer( 0 )
    .layer_count( 1 );

  let region = vk::BufferImageCopy::builder()
    .buffer_offset( 0 )
    .buffer_row_length( 0 )
    .buffer_image_height( 0 )
    .image_subresource( subresource )
    .image_offset( vk::Offset3D { x:0, y:0, z:0 } )
    .image_extent( vk::Extent3D { width, height, depth:1 } );

  device.cmd_copy_buffer_to_image( command_buffer, buffer, image, vk::ImageLayout::TRANSFER_DST_OPTIMAL, &[ region ] );

  end_single_time_commands( device, data, command_buffer )?;

  Ok(())
}

unsafe fn generate_mipmaps( instance:&Instance, device:&Device, data:&mut AppData, image:vk::Image, format:vk::Format, width:u32, height:u32, mip_levels:u32 ) -> Result<()> {
  if !instance.get_physical_device_format_properties( data.physical_device, format )
    .optimal_tiling_features
    .contains( vk::FormatFeatureFlags::SAMPLED_IMAGE_FILTER_LINEAR )
  {
    return Err( anyhow!( "Texture image format does not support linear blitting!" ) )
  }

  let command_buffer = begin_single_time_commands( device, data )?;

  let subresource = vk::ImageSubresourceRange::builder()
    .aspect_mask( vk::ImageAspectFlags::COLOR )
    .base_array_layer( 0 )
    .layer_count( 1 )
    .level_count( 1 );

  let mut barrier = vk::ImageMemoryBarrier::builder()
    .image( image )
    .src_queue_family_index( vk::QUEUE_FAMILY_IGNORED )
    .dst_queue_family_index( vk::QUEUE_FAMILY_IGNORED )
    .subresource_range( subresource );

  let mut mip_width = width;
  let mut mip_height = height;

  for i in 1..mip_levels {
    barrier.subresource_range.base_mip_level = i - 1;
    barrier.old_layout = vk::ImageLayout::TRANSFER_DST_OPTIMAL;
    barrier.new_layout = vk::ImageLayout::TRANSFER_SRC_OPTIMAL;
    barrier.src_access_mask = vk::AccessFlags::TRANSFER_WRITE;
    barrier.dst_access_mask = vk::AccessFlags::TRANSFER_READ;

    device.cmd_pipeline_barrier(
      command_buffer,
      vk::PipelineStageFlags::TRANSFER,
      vk::PipelineStageFlags::TRANSFER,
      vk::DependencyFlags::empty(),
      &[] as &[vk::MemoryBarrier],
      &[] as &[vk::BufferMemoryBarrier],
      &[ barrier ],
    );

    let src_subresource = vk::ImageSubresourceLayers::builder()
      .aspect_mask( vk::ImageAspectFlags::COLOR )
      .mip_level( i - 1 )
      .base_array_layer( 0 )
      .layer_count( 1 );

    let dst_subresource = vk::ImageSubresourceLayers::builder()
      .aspect_mask( vk::ImageAspectFlags::COLOR )
      .mip_level( i )
      .base_array_layer( 0 )
      .layer_count( 1 );

    let blit = vk::ImageBlit::builder()
      .src_offsets([
        vk::Offset3D { x:0, y:0, z:0 },
        vk::Offset3D {
          x: mip_width as i32,
          y: mip_height as i32,
          z: 1
        },
      ])
      .src_subresource( src_subresource )
      .dst_offsets([
        vk::Offset3D { x:0, y:0, z:0 },
        vk::Offset3D {
          x: (if mip_width  > 1 { mip_width  / 2 } else { 1 }) as i32,
          y: (if mip_height > 1 { mip_height / 2 } else { 1 }) as i32,
          z: 1
        },
      ])
      .dst_subresource( dst_subresource );

    device.cmd_blit_image(
      command_buffer,
      image,
      vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
      image,
      vk::ImageLayout::TRANSFER_DST_OPTIMAL,
      &[ blit ],
      vk::Filter::LINEAR,
    );

    barrier.old_layout = vk::ImageLayout::TRANSFER_SRC_OPTIMAL;
    barrier.new_layout = vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL;
    barrier.src_access_mask = vk::AccessFlags::TRANSFER_READ;
    barrier.dst_access_mask = vk::AccessFlags::SHADER_READ;

    device.cmd_pipeline_barrier(
      command_buffer,
      vk::PipelineStageFlags::TRANSFER,
      vk::PipelineStageFlags::FRAGMENT_SHADER,
      vk::DependencyFlags::empty(),
      &[] as &[vk::MemoryBarrier],
      &[] as &[vk::BufferMemoryBarrier],
      &[ barrier ],
    );

    if mip_width > 1 {
      mip_width /= 2;
    }

    if mip_height > 1 {
      mip_height /= 2;
    }
  }

  barrier.subresource_range.base_mip_level = mip_levels - 1;
  barrier.old_layout = vk::ImageLayout::TRANSFER_DST_OPTIMAL;
  barrier.new_layout = vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL;
  barrier.src_access_mask = vk::AccessFlags::TRANSFER_WRITE;
  barrier.dst_access_mask = vk::AccessFlags::SHADER_READ;

  device.cmd_pipeline_barrier(
    command_buffer,
    vk::PipelineStageFlags::TRANSFER,
    vk::PipelineStageFlags::FRAGMENT_SHADER,
    vk::DependencyFlags::empty(),
    &[] as &[vk::MemoryBarrier],
    &[] as &[vk::BufferMemoryBarrier],
    &[ barrier ],
  );

  end_single_time_commands( device, data, command_buffer )?;

  Ok(())
}

unsafe fn create_color_objects( instance:&Instance, device:&Device, data:&mut AppData ) -> Result<()> {
  let (color_image, color_image_memory) = create_image(
    instance, device, data,
    data.swapchain_extent.width,
    data.swapchain_extent.height,
    1,
    data.msaa_samples,
    data.swapchain_format,
    vk::ImageTiling::OPTIMAL,
    vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::TRANSIENT_ATTACHMENT,
    vk::MemoryPropertyFlags::DEVICE_LOCAL,
  )?;

  data.color_image = color_image;
  data.color_image_memory = color_image_memory;
  data.color_image_view = create_image_view(
    device,
    color_image,
    data.swapchain_format,
    vk::ImageAspectFlags::COLOR,
    1,
  )?;

  Ok(())
}

unsafe fn create_image_view( device:&Device, image:vk::Image, format:vk::Format, aspects:vk::ImageAspectFlags, mip_levels:u32 ) -> Result<vk::ImageView> {
  let subresource_range = vk::ImageSubresourceRange::builder()
    .aspect_mask( aspects )
    .base_mip_level( 0 )
    .level_count( mip_levels )
    .base_array_layer( 0 )
    .layer_count( 1 );

  let info = vk::ImageViewCreateInfo::builder()
    .image( image )
    .view_type( vk::ImageViewType::_2D )
    .format( format )
    .subresource_range( subresource_range );

  Ok( device.create_image_view( &info, None )? )
}
unsafe fn create_image(
  instance: &Instance,
  device: &Device,
  data: &mut AppData,
  width: u32,
  height: u32,
  mip_levels: u32,
  samples: vk::SampleCountFlags,
  format: vk::Format,
  tiling: vk::ImageTiling,
  usage: vk::ImageUsageFlags,
  properties: vk::MemoryPropertyFlags
) -> Result<( vk::Image, vk::DeviceMemory )> {
  let info = vk::ImageCreateInfo::builder()
    .image_type( vk::ImageType::_2D )
    .extent( vk::Extent3D { width, height, depth:1 } )
    .mip_levels( mip_levels )
    .array_layers( 1 )
    .format( format )
    .tiling( tiling )
    .initial_layout( vk::ImageLayout::UNDEFINED )
    .usage( usage )
    .sharing_mode( vk::SharingMode::EXCLUSIVE )
    .samples( samples );

  let image = device.create_image( &info, None )?;
  let requirements = device.get_image_memory_requirements( image );

  let info = vk::MemoryAllocateInfo::builder()
    .allocation_size( requirements.size )
    .memory_type_index( get_memory_type_index( instance, data, properties, requirements )? );

  let image_memory = device.allocate_memory( &info, None )?;

  device.bind_image_memory( image, image_memory, 0 )?;

  Ok((image, image_memory))
}

unsafe fn create_depth_objects( instance:&Instance, device:&Device, data:&mut AppData ) -> Result<()> {
  let format = get_depth_format( instance, data )?;

  let ( depth_image, depth_image_memory ) = create_image(
    instance, device, data,
    data.swapchain_extent.width,
    data.swapchain_extent.height,
    1,
    data.msaa_samples,
    format,
    vk::ImageTiling::OPTIMAL,
    vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
    vk::MemoryPropertyFlags::DEVICE_LOCAL,
  )?;

  data.depth_image = depth_image;
  data.depth_image_memory = depth_image_memory;
  data.depth_image_view = create_image_view( device, depth_image, format, vk::ImageAspectFlags::DEPTH, 1 )?;

  Ok(())
}

unsafe fn create_texture_image( instance:&Instance, device:&Device, data:&mut AppData, src:&str ) -> Result<()> {
  let image = match File::open( src ) {
    Ok( file ) => file,
    Err( err ) => {
      println!( "Current path: {:?}", std::env::current_dir() );
      println!( "{:?}", err );
      return Err( anyhow!( err ) );
    }
  };

  let decoder = png::Decoder::new( image );
  let mut reader = decoder.read_info()?;

  let mut pixels = vec![ 0; reader.info().raw_bytes() ];
  reader.next_frame( &mut pixels )?;

  let size = reader.info().raw_bytes() as u64;
  let ( width, height ) = reader.info().size();

  data.mip_levels = (width.max( height ) as f32).log2().floor() as u32 + 1;

  let (stagging_buffer, stagging_buffer_memory) = create_buffer(
    instance, device, data, size,
    vk::BufferUsageFlags::TRANSFER_SRC,
    vk::MemoryPropertyFlags::HOST_COHERENT | vk::MemoryPropertyFlags::HOST_VISIBLE,
  )?;

  let memory = device.map_memory( stagging_buffer_memory, 0, size, vk::MemoryMapFlags::empty() )?;

  memcpy( pixels.as_ptr(), memory.cast(), pixels.len() );

  device.unmap_memory( stagging_buffer_memory );

  let ( texture_image, texture_image_memory ) = create_image(
    instance, device, data, width, height,
    data.mip_levels,
    vk::SampleCountFlags::_1,
    vk::Format::R8G8B8A8_SRGB,
    vk::ImageTiling::OPTIMAL,
    vk::ImageUsageFlags::SAMPLED | vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::TRANSFER_SRC,
    vk::MemoryPropertyFlags::DEVICE_LOCAL
  )?;

  data.texture_image = texture_image;
  data.texture_image_memory = texture_image_memory;

  transition_image_layout(
    device, data, data.texture_image,
    vk::Format::R8G8B8A8_SRGB,
    vk::ImageLayout::UNDEFINED,
    vk::ImageLayout::TRANSFER_DST_OPTIMAL,
    data.mip_levels
  )?;

  copy_buffer_to_image( device, data, stagging_buffer, data.texture_image, width, height )?;

  device.destroy_buffer( stagging_buffer, None );
  device.free_memory( stagging_buffer_memory, None );

  generate_mipmaps( instance, device, data, data.texture_image, vk::Format::R8G8B8A8_SRGB, width, height, data.mip_levels )?;

  Ok(())
}

unsafe fn create_texture_image_view( device:&Device, data:&mut AppData ) -> Result<()> {
  data.texture_image_view = create_image_view( device, data.texture_image, vk::Format::R8G8B8A8_SRGB, vk::ImageAspectFlags::COLOR, data.mip_levels )?;
  Ok(())
}

unsafe fn create_texture_sampler( device:&Device, data:&mut AppData ) -> Result<()> {
  let info = vk::SamplerCreateInfo::builder()
    .mag_filter( vk::Filter::LINEAR )
    .min_filter( vk::Filter::LINEAR )
    .address_mode_u( vk::SamplerAddressMode::REPEAT )
    .address_mode_v( vk::SamplerAddressMode::REPEAT )
    .address_mode_w( vk::SamplerAddressMode::REPEAT )
    .anisotropy_enable( true )
    .max_anisotropy( 16.0 )
    .border_color( vk::BorderColor::INT_OPAQUE_BLACK )
    .unnormalized_coordinates( false )
    .compare_enable( false )
    .compare_op( vk::CompareOp::ALWAYS )
    .mipmap_mode( vk::SamplerMipmapMode::LINEAR )
    .min_lod( 0.0 )
    .max_lod( data.mip_levels as f32 )
    .mip_lod_bias( 0.0 );

  data.texture_sampler = device.create_sampler( &info, None )?;

  Ok(())
}

unsafe fn transition_image_layout( device:&Device, data:&mut AppData, image:vk::Image, format:vk::Format, old_layout:vk::ImageLayout, new_layout:vk::ImageLayout, mip_levels:u32 ) -> Result<()> {
  let (
    src_access_mask,
    dst_access_mask,
    src_stage_mask,
    dst_stage_mask,
  ) = match (old_layout, new_layout) {
    (vk::ImageLayout::UNDEFINED, vk::ImageLayout::TRANSFER_DST_OPTIMAL) => (
      vk::AccessFlags::empty(),
      vk::AccessFlags::TRANSFER_WRITE,
      vk::PipelineStageFlags::TOP_OF_PIPE,
      vk::PipelineStageFlags::TRANSFER,
    ),

    (vk::ImageLayout::TRANSFER_DST_OPTIMAL, vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL) => (
      vk::AccessFlags::TRANSFER_WRITE,
      vk::AccessFlags::SHADER_READ,
      vk::PipelineStageFlags::TRANSFER,
      vk::PipelineStageFlags::FRAGMENT_SHADER,
    ),

    (vk::ImageLayout::UNDEFINED, vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL) => (
      vk::AccessFlags::empty(),
      vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_READ | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
      vk::PipelineStageFlags::TOP_OF_PIPE,
      vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
    ),

    _ => return Err( anyhow!( "Unsupported image layout transition!" ) )
  };

  let command_buffer = begin_single_time_commands( device, data )?;
  let aspect_mask = if new_layout == vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL {
    match format {
      vk::Format::D32_SFLOAT_S8_UINT | vk::Format::D24_UNORM_S8_UINT => vk::ImageAspectFlags::DEPTH | vk::ImageAspectFlags::STENCIL,
      _ => vk::ImageAspectFlags::DEPTH,
    }
  } else {
    vk::ImageAspectFlags::COLOR
  };

  let subresource = vk::ImageSubresourceRange::builder()
    .aspect_mask( aspect_mask )
    .base_mip_level( 0 )
    .level_count( mip_levels )
    .base_array_layer( 0 )
    .layer_count( 1 );

  let barrier = vk::ImageMemoryBarrier::builder()
    .old_layout( old_layout )
    .new_layout( new_layout )
    .src_queue_family_index( vk::QUEUE_FAMILY_IGNORED )
    .dst_queue_family_index( vk::QUEUE_FAMILY_IGNORED )
    .image( image )
    .subresource_range( subresource )
    .src_access_mask( src_access_mask )
    .dst_access_mask( dst_access_mask );

  device.cmd_pipeline_barrier(
    command_buffer,
    src_stage_mask,
    dst_stage_mask,
    vk::DependencyFlags::empty(),
    &[] as &[ vk::MemoryBarrier ],
    &[] as &[ vk::BufferMemoryBarrier ],
    &[ barrier ],
  );

  end_single_time_commands( device, data, command_buffer )?;

  Ok(())
}

unsafe fn create_vertex_buffer( instance:&Instance, device:&Device, data:&mut AppData ) -> Result<()> {
  let size = (size_of::<Vertex>() * data.vertices.len()) as u64;
  let ( staging_buffer, staging_buffer_memory ) = create_buffer(
    instance,
    device,
    data,
    size,
    vk::BufferUsageFlags::TRANSFER_SRC,
    vk::MemoryPropertyFlags::HOST_COHERENT | vk::MemoryPropertyFlags::HOST_VISIBLE,
  )?;

  let memory = device.map_memory( staging_buffer_memory, 0, size, vk::MemoryMapFlags::empty() )?;

  memcpy( data.vertices.as_ptr(), memory.cast(), data.vertices.len() );
  device.unmap_memory( staging_buffer_memory );

  let ( vertex_buffer, vertex_buffer_memory ) = create_buffer(
    instance,
    device,
    data,
    size,
    vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::VERTEX_BUFFER,
    vk::MemoryPropertyFlags::DEVICE_LOCAL,
  )?;

  data.vertex_buffer = vertex_buffer;
  data.vertex_buffer_memory = vertex_buffer_memory;

  copy_buffer( device, data, staging_buffer, vertex_buffer, size )?;

  device.destroy_buffer( staging_buffer, None );
  device.free_memory( staging_buffer_memory, None );

  Ok(())
}

unsafe fn create_index_buffer( instance:&Instance, device:&Device, data:&mut AppData ) -> Result<()> {
  let size = (size_of::<u32>() * data.indices.len()) as u64;
  let ( staging_buffer, staging_buffer_memory ) = create_buffer(
    instance,
    device,
    data,
    size,
    vk::BufferUsageFlags::TRANSFER_SRC,
    vk::MemoryPropertyFlags::HOST_COHERENT | vk::MemoryPropertyFlags::HOST_VISIBLE,
  )?;

  let memory = device.map_memory( staging_buffer_memory, 0, size, vk::MemoryMapFlags::empty() )?;

  memcpy( data.indices.as_ptr(), memory.cast(), data.indices.len() );
  device.unmap_memory( staging_buffer_memory );

  let ( index_buffer, index_buffer_memory ) = create_buffer(
    instance,
    device,
    data,
    size,
    vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::INDEX_BUFFER,
    vk::MemoryPropertyFlags::DEVICE_LOCAL
  )?;

  data.index_buffer = index_buffer;
  data.index_buffer_memory = index_buffer_memory;

  copy_buffer( device, data, staging_buffer, index_buffer, size )?;

  device.destroy_buffer( staging_buffer, None );
  device.free_memory( staging_buffer_memory, None );

  Ok(())
}

unsafe fn create_uniform_buffers( instance:&Instance, device:&Device, data:&mut AppData ) -> Result<()> {
  data.uniform_buffers.clear();
  data.uniform_buffers_memory.clear();

  for _ in 0..data.swapchain_images.len() {
    let (uniform_buffer, uniform_buffer_memory) = create_buffer(
      instance,
      device,
      data,
      size_of::<UniformBufferObject>() as u64,
      vk::BufferUsageFlags::UNIFORM_BUFFER,
      vk::MemoryPropertyFlags::HOST_COHERENT | vk::MemoryPropertyFlags::HOST_VISIBLE
    )?;

    data.uniform_buffers.push( uniform_buffer );
    data.uniform_buffers_memory.push( uniform_buffer_memory );
  }

  Ok(())
}

unsafe fn create_descriptor_pool( device:&Device, data:&mut AppData ) -> Result<()> {
  let ubo_size = vk::DescriptorPoolSize::builder()
    .type_( vk::DescriptorType::UNIFORM_BUFFER )
    .descriptor_count( data.swapchain_images.len() as u32 );

  let sampler_size = vk::DescriptorPoolSize::builder()
    .type_( vk::DescriptorType::COMBINED_IMAGE_SAMPLER )
    .descriptor_count( data.swapchain_images.len() as u32 );

  let pool_sizes = &[ ubo_size, sampler_size ];
  let info = vk::DescriptorPoolCreateInfo::builder()
    .pool_sizes( pool_sizes )
    .max_sets( data.swapchain_images.len() as u32 );

  data.descriptor_pool = device.create_descriptor_pool( &info, None )?;

  Ok(())
}

unsafe fn create_descriptor_sets( device:&Device, data:&mut AppData ) -> Result<()> {
  let layouts = vec![ data.descriptor_set_layout; data.swapchain_images.len() ];
  let info = vk::DescriptorSetAllocateInfo::builder()
    .descriptor_pool( data.descriptor_pool )
    .set_layouts( &layouts );

  data.descriptor_sets = device.allocate_descriptor_sets( &info )?;

  for i in 0..data.swapchain_images.len() {
    let info = vk::DescriptorBufferInfo::builder()
      .buffer( data.uniform_buffers[ i ] )
      .offset( 0 )
      .range( size_of::<UniformBufferObject>() as u64 );

    let buffer_info = &[ info ];
    let ubo_write = vk::WriteDescriptorSet::builder()
      .dst_set( data.descriptor_sets[ i ] )
      .dst_binding( 0 )
      .dst_array_element( 0 )
      .descriptor_type( vk::DescriptorType::UNIFORM_BUFFER)
      .buffer_info( buffer_info );

    let info = vk::DescriptorImageInfo::builder()
      .image_layout( vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL )
      .image_view( data.texture_image_view )
      .sampler( data.texture_sampler );

    let image_info = &[ info ];
    let sampler_write = vk::WriteDescriptorSet::builder()
      .dst_set( data.descriptor_sets[ i ] )
      .dst_binding( 1 )
      .dst_array_element( 0 )
      .descriptor_type( vk::DescriptorType::COMBINED_IMAGE_SAMPLER )
      .image_info( image_info );

    device.update_descriptor_sets(
      &[ ubo_write, sampler_write ],
      &[] as &[ vk::CopyDescriptorSet ]
    );
  }

  Ok(())
}

unsafe fn create_command_buffers( device:&Device, data:&mut AppData ) -> Result<()> {
  for image_index in 0..data.swapchain_images.len() {
    let allocate_info = vk::CommandBufferAllocateInfo::builder()
      .command_pool( data.command_pools[ image_index ] )
      .level( vk::CommandBufferLevel::PRIMARY )
      .command_buffer_count( data.framebuffers.len() as u32 );

    data.command_buffers.push( device.allocate_command_buffers( &allocate_info )?[ 0 ] );
  }

  data.secondary_command_buffers = vec![ vec![]; data.swapchain_images.len() ];

  Ok(())
}

unsafe fn create_sync_objects( device:&Device, data:&mut AppData ) -> Result<()> {
  let semaphore_info = vk::SemaphoreCreateInfo::builder();
  let fence_info = vk::FenceCreateInfo::builder()
    .flags( vk::FenceCreateFlags::SIGNALED );

  for _ in 0..MAX_FRAMES_IN_FLIGHT {
    data.image_available_semaphores.push( device.create_semaphore( &semaphore_info, None )? );
    data.render_finished_semaphores.push( device.create_semaphore( &semaphore_info, None )? );

    data.in_flight_fences.push( device.create_fence( &fence_info, None )? );
  }

  data.images_in_flight = data.swapchain_images
    .iter()
    .map( |_| vk::Fence::null() )
    .collect();

  Ok(())
}


extern "system" fn debug_callback(
  severity: vk::DebugUtilsMessageSeverityFlagsEXT,
  type_: vk::DebugUtilsMessageTypeFlagsEXT,
  data: *const vk::DebugUtilsMessengerCallbackDataEXT,
  _: *mut c_void,
) -> vk::Bool32 {
  let data = unsafe { *data };
  let message = unsafe { CStr::from_ptr( data.message ) }.to_string_lossy();

  if severity >= vk::DebugUtilsMessageSeverityFlagsEXT::ERROR {
    error!("({:?}) {}", type_, message);
  } else if severity >= vk::DebugUtilsMessageSeverityFlagsEXT::WARNING {
    warn!("({:?}) {}", type_, message);
  } else if severity >= vk::DebugUtilsMessageSeverityFlagsEXT::INFO {
    debug!("({:?}) {}", type_, message);
  } else {
    trace!("({:?}) {}", type_, message);
  }

  vk::FALSE
}
