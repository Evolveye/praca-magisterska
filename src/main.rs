mod world;
mod noise;
mod structure_tests;
mod rendering;
mod app;

use image::{ GrayImage, Luma };
use std::path::Path;
use noise::generate_simplex_noise_image;
use crate::{app::app::App, world::chunk_region_iterator::ChunkRegionIterator};

fn main() {
    pretty_env_logger::init();
    let mut app = App::new().unwrap();
    app.run_loop();


    // test_cube_with_side( 1, ChunkRegionIterator::with_range( 20..27 ) );
    // test_cube_with_side( 1, ChunkRegionIterator::with_range( 1..27 ) );
    // test_cube_with_side( 1, ChunkRegionIterator::with_range( 0..2 ) );
    // test_cube_with_side( 2, ChunkRegionIterator::with_range( 27..(27 + 98) ) );
    // test_cube_with_side( 2, ChunkRegionIterator::with_range( 105..(27 + 98) ) );
    // test_cube_indices_with_side( 1, 0, 26 );
    // test_cube_indices_with_side( 2, 0, 98 );
    // test_cube_indices_with_side( 2, 78, 20 );
    // test_cube_indices_with_side( 2, 80, 20 );
}

#[allow(dead_code)]
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

#[allow(dead_code)]
fn test_cube_with_side( layer:i32, mut cube:ChunkRegionIterator ) {
    let mut max = ChunkRegionIterator::get_layer_size( layer ) as u32;
    let mut last_side = cube.side;

    max = cube.iterations.unwrap() + 1;
    println!( "Start ({max} iterations)\n - - -" );

    loop {
        let value = cube.next();

        if last_side != cube.side {
            last_side = cube.side;
            println!( " - - -" );
        }

        println!( "  [{max: >2}] {: >2?} | {value: >2?}, side={}", cube.iterations, cube.side );

        if max == 0 { break }
        max -= 1;
    }

    println!( " - - -\nEnd" );
}

#[allow(dead_code)]
fn test_cube_indices_with_side( mut layer:i32, offset:u32, limit:u32 ) {
    let mut prev_layer = 0;

    loop {
        layer -= 1;
        prev_layer += ChunkRegionIterator::get_layer_size( layer ) as u32;
        if layer == 0 { break }
    }

    let mut index = 0;
    println!( "Start ({limit} iterations)\n - - -" );

    loop {
        let pos_index = prev_layer + offset + index;
        let value = ChunkRegionIterator::get_pos_from_index( pos_index );

        println!( "  [{index: >2}] | {pos_index} -> {value: >2?}" );

        if index == limit { break }
        index += 1;
    }

    println!( " - - -\nEnd" );
}