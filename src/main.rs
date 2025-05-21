mod world;
mod noise;
mod structure_tests;
mod rendering;
mod simulation;
mod window_manager;

use image::{GrayImage, Luma};
use simulation::Simulation;
use world::world_holder::WorldHolder;
use std::path::Path;
use noise::generate_simplex_noise_image;
use structure_tests::{generate_world, tester::{WORLD_X, WORLD_Y, WORLD_Z}};

fn main() {
    render_world();
    // generate_world();
    // let mut world_struct = Octree::new( 18 );
    // let dataset = Tester::fill_50pc_realistically( &mut world_struct );
    // generate_img();
}

fn generate_img() {
    const WIDTH: u32 = 1024 * 4;
    const HEIGHT: u32 = 1024 * 4;
    let mut img = GrayImage::new( WIDTH, HEIGHT );

    generate_simplex_noise_image( WIDTH, HEIGHT, |x, y, v| {
        img.put_pixel( x, y, Luma([ if v > 0.85 { 0 } else { 255 } ]) )
    } );

    let path = Path::new( "simplex_noise_test.png" );
    img.save( path ).expect( "Failed to save image" );
    println!( "Image saved: {:?}", path );
}

fn render_world() {
    let world = generate_world();
    // println!( "(0, 15, 0) = {:?}", world.get_voxel( 0, 15, 0 ) );
    // world.get_all_visible_voxels_from( (0, WORLD_Y, 0) );
    let mut simulation = Simulation::new().unwrap();
    // simulation.update_instances_with_defaults();
    simulation.move_camera_to(
        (WORLD_X as f32 / 2.0, if WORLD_Y > 175 { 175 } else { WORLD_Y } as f32, 32f32.min( WORLD_Z as f32 * 2.0 )),
        (WORLD_X as f32 / 2.0, 0.0, WORLD_Z as f32)
    );
    simulation.update_instances_with_world_holder( world );
    simulation.run_window_event_loop();
}
