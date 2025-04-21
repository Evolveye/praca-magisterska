mod world;
mod noise;
mod structure_tests;
mod rendering;
mod simulation;
mod window_manager;

use image::{GrayImage, Luma};
use simulation::Simulation;
use std::path::Path;
use noise::generate_simplex_noise_image;
use structure_tests::generate_world;

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
    let mut simulation = Simulation::new().unwrap();
    // simulation.update_instances_with_defaults();
    simulation.update_instances_with_world_holder( world );
    simulation.run_window_event_loop();
}
