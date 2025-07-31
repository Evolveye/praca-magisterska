use anyhow::Result;
use winit::{
  dpi::{ LogicalSize, PhysicalPosition },
  event::WindowEvent,
  event_loop::EventLoop,
  window::{ Window, WindowBuilder }
};

pub struct WindowManager {
  pub event_loop: Option<EventLoop<()>>,
  pub window: Window,
  pub focused: bool,
  pub resized: bool,
  pub minimized: bool,
  pub center: PhysicalPosition<f64>,
}

impl WindowManager {
    pub fn new() -> Result<Self> {
        let event_loop = EventLoop::new()?;
        let window = WindowBuilder::new()
            .with_title( "Vulkan Tutorial (Rust)" )
            .with_inner_size( LogicalSize::new( 1024, 768 ) )
            .build( &event_loop )?;


        let window_size = window.inner_size();
        let center = PhysicalPosition::new( window_size.width as f64 / 2.0, window_size.height as f64 / 2.0 );

        window.set_cursor_visible( false );

        Ok( Self {
            window,
            focused: true,
            resized: false,
            minimized: false,
            event_loop: Some( event_loop ),
            center,
        } )
    }

    pub fn handle_window_event( &mut self, event:WindowEvent ) {
        match event {
            WindowEvent::Focused( focused ) => self.focused = focused,

            WindowEvent::Resized( size ) => {
                if size.width == 0 || size.height == 0 {
                    self.minimized = true;
                } else {
                    self.minimized = false;
                    self.resized = true;
                }
            }

            _ => {}
        }
    }
}