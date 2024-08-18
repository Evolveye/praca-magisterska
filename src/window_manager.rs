use anyhow::Result;
use winit::{
  dpi::LogicalSize,
  event_loop::EventLoop,
  window::{ Window, WindowBuilder}
};

pub struct WindowManager {
  pub event_loop: Option<EventLoop<()>>,
  pub window: Window,
  pub focused: bool,
  pub resized: bool,
}

impl WindowManager {
  pub fn new() -> Result<Self> {
    let event_loop = EventLoop::new()?;
    let window = WindowBuilder::new()
      .with_title( "Vulkan Tutorial (Rust)" )
      .with_inner_size( LogicalSize::new( 1536, 1152 ) )
      .build( &event_loop )?;

    window.set_cursor_visible( false );

    Ok( Self {
      window,
      focused: true,
      resized: false,
      event_loop: Some( event_loop ),
    } )
  }
}