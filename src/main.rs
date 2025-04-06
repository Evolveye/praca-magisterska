mod world;
mod noise;
mod structure_tests;
use image::{GrayImage, Luma};
use std::path::Path;
use noise::{generate_simplex_noise_image, generate_simplex_noise_image_with_octaves};
use structure_tests::run_test;

const WIDTH: u32 = 1024 * 4;
const HEIGHT: u32 = 1024 * 4;

fn main() {
    run_test();
    // generate_img();
}

fn generate_img() {
    let mut img = GrayImage::new( WIDTH, HEIGHT );

    generate_simplex_noise_image( WIDTH, HEIGHT, |x, y, v| {
        img.put_pixel( x, y, Luma([ if v > 0.85 { 0 } else { 255 } ]) )
    } );

    let path = Path::new( "simplex_noise_test.png" );
    img.save( path ).expect( "Failed to save image" );
    println!( "Image saved: {:?}", path );
}
