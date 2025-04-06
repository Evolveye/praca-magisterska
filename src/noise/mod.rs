pub mod simplex_noise;
use std::path::Path;

use cgmath::num_traits::Pow;
use image::{GrayImage, Luma};
use simplex_noise::SimplexNoise;

// const WIDTH: u32 = 512;
// const HEIGHT: u32 = 512;
// const WIDTH: u32 = 128;
// const HEIGHT: u32 = 128;
const FREQUENCY: f64 = 0.05;
const CERTAIN_TRESHOLD: f64 = 0.85;
const UNCERTAIN_TRESHOLD: f64 = 0.7;

pub fn generate_simplex_noise_image<F:FnMut(u32, u32, f64)>( width:u32, height:u32, mut noise_value_handler:F ) {
    let noise = SimplexNoise::new( 0 );

    for y in 0..height {
        for x in 0..width {
            let nx = x as f64 * FREQUENCY;
            let ny = y as f64 * FREQUENCY;

            let noise_value = noise.noise3d( nx, ny, 0.0 );
            // let slope = (dx * dx + dy * dy + dz * dz).sqrt();
            // noise_value = (1.0 as f64).powf( 20.0 ).clamp( 0.0, 1.0 );
            // noise_value = ((noise_value + 0.2) - 0.0).powf( 5.0 ).clamp( -1.0, 1.0 );
            // noise_value = ((noise_value + 0.2).powf( 5.0 ) - 0.2);
            // noise_value = ((noise_value + 0.2).min( 1.0 ) - 0.2);
            // noise_value = noise_value.powf( 3.0 );

            // let pixel_value = if noise_value >= CERTAIN_TRESHOLD { 0 }
            // else if noise_value >= UNCERTAIN_TRESHOLD && slope > 0.3 { 0 }
            // else { 255 };
            // let pixel_value = if noise_value >= CERTAIN_TRESHOLD && laplacian.abs() > 1.75 { 0 } else { 255 };
            // let pixel_value = ((noise_value + 1.0) * 0.5 * 255.0) as u8;

            // if noise_value >= UNCERTAIN_TRESHOLD {
            //     println!( "{} | {} | laplacian={} x={}, y={}", noise_value, pixel_value, laplacian, x, y );
            // }

            noise_value_handler( x, y, noise_value );
        }
    }
}

pub fn generate_simplex_noise_image_with_octaves<F:FnMut(u32, u32, f64)>( width:u32, height:u32, mut noise_value_handler:F ) {
    let noise = SimplexNoise::new( 1 );
    let noises = [
        SimplexNoise::new( 10 ),
        SimplexNoise::new( 20 ),
        SimplexNoise::new( 30 ),
        SimplexNoise::new( 40 ),
    ];

    let octaves_count = 3;
    let mut img = GrayImage::new( width, height );

    let mut min_v = f64::MAX;
    let mut max_v = f64::MIN;

    for y in 0..height {
        for x in 0..width {
            let mut value = 0.0;
            let mut amplitude = 1.0 / octaves_count as f64;
            // let mut amplitude = 1.0;
            // let mut amplitude = 1.0;
            let mut frequency = FREQUENCY; // + y as f64 * 0.002;

            for i in 0..octaves_count {
                let noise_value = noises[ i ].noise3d( x as f64 * frequency, y as f64 * frequency, 0.0) * amplitude;
                value += noise_value;
                // amplitude *= 1.5;
                // frequency *= 1.001;

                // if noise_value.abs() > 0.97 {
                //     println!( "{} | x={}, y={}", noise_value, x, y );
                // }
            }

            min_v = min_v.min( value );
            max_v = max_v.max( value );

            // value = ((value - 0.1).max( 0.0 ).powf( 1.0 ) - 0.0).clamp( 0.0, 1.0 );
            // let pixel_value = if value > 0.8 { 0 } else { 255 };
            // let pixel_value = ((value + 1.0) * 127.5) as u8;
            // let pixel_value = if x % 100 == 0 { 0 } else { ((value + 1.0) * 127.5) as u8 };
            // let pixel_value = value as u8;

            noise_value_handler( x, y, value );
        }
    }
}
