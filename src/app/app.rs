use std::{
    io::Write,
    time::{ Duration, Instant }
};

use anyhow::Result;
use cgmath::{ point3, vec3, Matrix4, SquareMatrix };
use winit::{
    event::{ Event, WindowEvent },
    event_loop::EventLoopWindowTarget,
    keyboard::{ KeyCode, PhysicalKey }
};
use crate::{
    app::{
        camera::{ Camera, FrustumVertex },
        control_manager::ControlManager,
        settings::AppSettings,
        window_manager::WindowManager,
    }, flags::FLAG_PROFILING_SHOW_FPS, rendering::{
        model::{ Model, ModelInstance },
        renderer::Renderer,
        vertex::{ Renderable, SimpleVertex },
    }, structure_tests::generate_world_as_world, world::{
        voxel_vertices::{ VOXEL_CORNERS, VOXEL_EDGES_INDICES, VOXEL_VERTICES },
        world::{ ChunkLoaderhandle, World, CHUNK_SIZE },
        world_renderer::WorldRenderer,
    }
};

pub struct App {
    pub window_manager: WindowManager,
    pub control_manager: ControlManager,
    pub camera: Camera,
    pub renderer: Renderer,
    pub world_renderer: WorldRenderer,
    pub world: World,
    pub camera_chunk_loader: ChunkLoaderhandle,
    pub settings: AppSettings,

    pub start_time: Instant,
    pub frame_times: Vec<f32>,
    pub last_tick_time: Instant,
    pub fps_time: Instant,
    pub frame_count: u64,

    frustum_model: Model<FrustumVertex>,
    world_border_model: Option<Model<SimpleVertex>>
}

impl App {
    pub fn new() -> Result<Self> {
        let _half_chunk_size = CHUNK_SIZE as f32 / 2.0;

        let window_manager = WindowManager::new()?;
        let control_manager = ControlManager::new(
            // point3( _half_chunk_size, 15.0, _half_chunk_size ), point3( _half_chunk_size, 0.0, 0.0 )
            // point3( _half_chunk_size, 45.0, _half_chunk_size ), point3( 0.0, 30.0, 0.0 )
            point3( -24.0, 70.0, -165.0 ), point3( 64.0, 60.0, 64.0 )
        );
        let window_size = window_manager.window.inner_size();
        let camera = Camera::new( control_manager.position, control_manager.rotation, window_size.width, window_size.height );
        let renderer = Renderer::create( &window_manager.window )?;
        let world_renderer = WorldRenderer::new( &renderer );
        let settings = AppSettings::new();
        let ( world, camera_chunk_loader ) = generate_world_as_world( control_manager.position );

        let model = unsafe {
            let mut model = Model::<FrustumVertex>::new( &renderer, VOXEL_VERTICES.map( |v| v.into() ).to_vec(), VOXEL_EDGES_INDICES.to_vec() ).unwrap();

            model.update_instance_buffer( &renderer, vec![
                ModelInstance {
                    instance_transform: Matrix4::identity()
                }
            ] ).unwrap();

            model
        };

        let border_model = if let Some( max_radius ) = world.max_radius {
            let scale = CHUNK_SIZE as f32 * max_radius as f32;
            let vertices = VOXEL_CORNERS.map( |v| {
                let pos = vec3(
                    (v.pos.x + 0.5) * scale - 0.5,
                    (v.pos.y + 0.5) * scale - 0.5,
                    (v.pos.z + 0.5) * scale - 0.5
                );

                SimpleVertex {
                    color: vec3( 1.0, 0.0, 0.0 ),
                    pos,
                }
            } ).into();

            unsafe {
                let mut border_model = Model::<SimpleVertex>::new( &renderer, vertices, VOXEL_EDGES_INDICES.to_vec() ).unwrap();

                border_model.update_instance_buffer( &renderer, vec![
                    ModelInstance {
                        instance_transform: Matrix4::identity()
                    }
                ] ).unwrap();

                Some( border_model )
            }
        } else {
            None
        };

        Ok( App {
            window_manager,
            control_manager,
            camera,
            renderer,
            world_renderer,
            world,
            camera_chunk_loader,
            settings,

            frame_times: Vec::with_capacity( 1000 ),
            frame_count: 0,
            start_time: Instant::now(),
            last_tick_time: Instant::now(),
            fps_time: Instant::now(),

            frustum_model: model,
            world_border_model: border_model,
        } )
    }

    pub fn tick( &mut self ) {
        let timestamp = Instant::now();
        let time_delta = timestamp.duration_since( self.last_tick_time );

        if FLAG_PROFILING_SHOW_FPS {
            self.last_tick_time = timestamp;
            self.frame_count += 1;

            self.frame_times.push( time_delta.as_secs_f32() );
            if self.frame_times.len() > 1000 {
                self.frame_times.remove( 0 );
            }

            if self.fps_time.elapsed() >= Duration::from_secs( 5 ) {
                let fps = 1.0 / time_delta.as_secs_f64();
                println!( "fps={} | times.len={}", fps as u32, self.frame_times.len() );

                self.fps_time = timestamp;
            }
        }

        self.world.move_chunk_loader_to( &self.camera_chunk_loader, self.control_manager.position.into(), self.control_manager.freezed );

        if !self.control_manager.freezed {
            self.world.update();
        }

        self.control_manager.update( &self.settings, time_delta.as_secs_f32() );
        self.camera.update_view( self.control_manager.position, self.control_manager.rotation, self.control_manager.freezed );

        self.world_renderer.update_instances_buffer(
            &self.renderer,
            if self.control_manager.freezed { self.world.debug_meshes.clone() } else { self.world.get_renderables( &self.camera ) }
        );

        unsafe { self.frustum_model.update_vertex_buffer::<FrustumVertex>( &self.renderer, self.camera.get_frustum_corners().into() ).unwrap() };
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
                        let mut models:Vec<&dyn Renderable> = if self.control_manager.freezed {
                            vec![ &self.world_renderer, &self.frustum_model ]
                        } else {
                            vec![ &self.world_renderer ]
                        };

                        if let Some( ref model ) = self.world_border_model {
                            models.push( model );
                        }

                        let _ = self.renderer.render( &mut self.window_manager, &self.camera, models );
                    },

                    WindowEvent::CloseRequested => App::destroy( elwt, self ),

                    WindowEvent::KeyboardInput { event, .. } => {
                        match event.physical_key {
                            PhysicalKey::Code( KeyCode::Escape ) => App::destroy( elwt, self ),
                            PhysicalKey::Code( KeyCode::KeyR ) => {
                                self.frame_times.clear();
                                self.control_manager.handle_keyboard_event( &self.settings, event );
                            },
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

    fn destroy( elwt:&EventLoopWindowTarget<()>, app:&mut App ) {
        elwt.exit();

        unsafe {
            app.renderer.device_wait_idle();
            app.world_renderer.model.destroy( &app.renderer.device );
            app.frustum_model.destroy( &app.renderer.device );
            if let Some( model ) = app.world_border_model.take() {
                model.destroy( &app.renderer.device );
            }
            app.renderer.destroy();
        }

        println!( "" );
        println!( "App uptime = {:?}", app.start_time.elapsed() );
        println!( "" );

        if app.frame_times.len() > 0 {
            Self::analyze_fps( &app.frame_times );
        }
    }

    fn analyze_fps( times:&Vec<f32> ) {
        let fps: Vec<f32> = times.iter().map(|dt| 1.0 / dt).collect();
        let mut sorted: Vec<f32> = fps.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let min_outlier = sorted.first().unwrap();
        let max_outlier = sorted.last().unwrap();

        let len = sorted.len();
        let lower_idx = (0.025 * len as f32).floor() as usize;
        let upper_idx = (0.975 * len as f32).ceil() as usize;
        let filtered: &[f32] = &sorted[lower_idx..upper_idx.min(len)];

        let mean = filtered.iter().copied().sum::<f32>() / filtered.len() as f32;
        let std_dev = (filtered.iter().map(|x| (x - mean).powi(2)).sum::<f32>() / filtered.len() as f32).sqrt();
        let min = *filtered.first().unwrap();
        let max = *filtered.last().unwrap();

        println!( "Avg FPS: {:.2}", mean );
        println!( "Std dev: {:.2}", std_dev );
        println!( "Min FPS: {:.2}", min );
        println!( "Max FPS: {:.2}", max );
        println!( "Min FPS outliers: {:.2}", min_outlier );
        println!( "Max FPS outliers: {:.2}", max_outlier );
        println!( "Outliers count: {:.2}", sorted.len() - filtered.len() );


        // Write CSV
        let file = std::fs::File::create( "frame_times.csv" ).unwrap();
        let mut writer = std::io::BufWriter::new(file);

        writeln!( writer, "frame,dt,fps" ).unwrap();

        for (i, dt) in times.iter().enumerate() {
            let fps = 1.0 / dt;
            writeln!( writer, "{},{:.6},{:.2}", i, dt, fps ).unwrap();
        }
    }
}
