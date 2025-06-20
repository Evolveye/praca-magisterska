pub mod simplex_noise;

use simplex_noise::SimplexNoise;

const FREQUENCY: f64 = 0.05;

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
