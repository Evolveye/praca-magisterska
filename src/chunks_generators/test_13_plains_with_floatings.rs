use std::sync::Arc;

use rand::{ rngs::SmallRng, Rng, SeedableRng };

use crate::{
    chunks_generators::{
        floatings::GeneratorOfFloatings,
        utilities::{create_voxel, generate_unique}
    },
    noise::simplex_noise::SimplexNoise,
    structure_tests::{ octree::Octree, quadtree::Quadtree },
    world::{
        world::CHUNK_SIZE,
        world_generator::WorldGenerative,
        world_holder::{ fill_with, Color, Material, Voxel, VoxelDataset, WorldHolding }
    }
};

const CHUNK_SIZE_U32:u32 = CHUNK_SIZE as u32;

pub struct GeneratorOfTest13PlainsWithFloatings {
    clouds_generator: GeneratorOfFloatings,
    noise: SimplexNoise,
    hills_noise: SimplexNoise,
    noise_frequency: f64,
    noise_frequency_hills: f64,
    noise_amplitude: f64,
    noise_amplitude_hills: f64,
    hills_smoothing_length: i64,
}

impl GeneratorOfTest13PlainsWithFloatings {
    #[allow(unused)]
    pub fn new( seed:u32) -> Self {
        let mut clouds_generator = GeneratorOfFloatings::new( seed );
        clouds_generator.set_colors(
            (250, 250, 250),
            (150, 150, 250),
        );

        Self {
            clouds_generator,
            noise: SimplexNoise::new( seed ),
            hills_noise: SimplexNoise::new( !seed ),

            // noise_frequency: 0.1,
            // noise_frequency: 0.05,
            // noise_frequency: 0.025,
            // noise_frequency: 0.013,
            noise_frequency: 0.01,
            // noise_frequency: 0.005,
            noise_frequency_hills: 0.01,
            // noise_amplitude: 10.0,
            noise_amplitude: 12.0,
            // noise_amplitude: 20.0,
            noise_amplitude_hills: 100.0,
            // noise_amplitude_hills: 50.0,
            hills_smoothing_length: 100,
        }
    }
}

impl WorldGenerative for GeneratorOfTest13PlainsWithFloatings {
    fn generate_chunk( &self, dataset:&mut VoxelDataset, origin:(i64, i64, i64), size:u8 ) -> Octree<Voxel> {
        // println!( "Chunk generation {:?}, size={}", origin, size );
        let world_origin = (origin.0 * size as i64, origin.1 * size as i64, origin.2 * size as i64);
        let size_u32 = size as u32;
        let max_depth = Quadtree::get_max_depth_for( size_u32 );
        let mut world_holder = Octree::from_max_size( size_u32 );
        let grass_level = 8 - world_origin.1;

        let quadtree = Quadtree::from_terrain_generation( max_depth, &|x, z| {
            let coords = (
                (world_origin.0 + x as i64) as f64,
                1.0,
                (world_origin.2 + z as i64) as f64,
            );

            let mut noise = self.noise.noise3d( coords.0 * self.noise_frequency, coords.1, coords.2 * self.noise_frequency )
                * self.noise_amplitude;

            if world_origin.2 >= 0 {
                let min = world_origin.2 + z as i64;
                let mul = min.clamp( 0, self.hills_smoothing_length ) as f64 / self.hills_smoothing_length as f64;
                noise += (self.hills_noise.noise3d( coords.0 * self.noise_frequency_hills, coords.1, coords.2 * self.noise_frequency_hills ) + 0.5)
                    * self.noise_amplitude_hills * mul;
            }

            noise
        } );

        quadtree.proces_entire_tree( &mut |offset, size, noise_value| {
            let current_min = grass_level + noise_value as i64;
            if current_min < 0 || current_min < offset.1 as i64 { return offset.1 }

            let size = size - 1;
            let to = (offset.0 + size, current_min as u32, offset.2 + size);

            let below_water = current_min < grass_level - 8;
            let high = current_min > grass_level + 15;
            let peak = current_min > grass_level + 50;

            let color = Color {
                red: if peak {
                        if current_min % 2 == 0 { 150 } else { 190 }
                    } else if high && noise_value > 0.7 {
                        0.max( 50 - (noise_value * 1.0) as i32 ) as u8
                    } else if below_water {
                        if current_min % 2 == 0 { 10 } else { 25 }
                    } else {
                        30
                    },
                green: if peak {
                        if current_min % 2 == 0 { 150 } else { 190 }
                    } else if high && noise_value > 0.7 {
                        0.max( 220 - (noise_value * 4.0) as i32 ) as u8
                    } else if below_water {
                        255.min( (noise_value * 10.0) as i32 ) as u8
                    } else {
                        if current_min % 2 == 0 { 125 } else { 145 }
                    },
                blue: if peak {
                        if current_min % 2 == 0 { 150 } else { 190 }
                    } else if high && noise_value > 0.7 {
                        0.max( 50 - (noise_value * 1.0) as i32 ) as u8
                    } else if below_water {
                        if current_min % 2 == 0 { 175 } else { 200 }
                    } else {
                        20
                    },
            };

            dataset.expand( fill_with( offset, to, &mut world_holder, (&format!( "grass_{}", current_min ), color) ) );

            to.1 + 1
        } );

        if origin.1 == 0 && origin.0 < 0 {
        // if origin.1 == 0 && origin.0 == -1 && origin.2 == -1 {
            let size = size as u32;
            let uint_origin = (origin.0.abs(), origin.1.abs(), origin.2.abs());
            let randoms = generate_unique(
                (uint_origin.0 as u64) << 32 | (uint_origin.1 as u64) << 16 | (uint_origin.2 as u64),
                100
            );

            for random in randoms {
                let random_mod = random % 100;

                if random_mod < 20 {
                    let i = random % (size * size);
                    let x = i % size;
                    let z = (i / size) % size;

                    let mut div_size = size / 2;
                    let mut y = div_size;

                    loop {
                        div_size /= 2;

                        if world_holder.get( x, y, z ).is_some() {
                            y += div_size;
                        } else {
                            y -= div_size;
                        }

                        if div_size == 1 {
                            break
                        }
                    }

                    if y > 0 {
                        plant_tree( dataset, &mut world_holder, (x, y, z) );
                    }
                }
            }
        }

        // if origin.0 >= -1 && origin.0 <= 0 && origin.1 == 0 && origin.2 == 0 {
        //     world_holder = self.cube_generator.generate( dataset, world_holder, world_origin, size );
        // }
        if origin.1 >= 2 {
            let mut rng = SmallRng::seed_from_u64(
                world_origin.0
                .wrapping_add( world_origin.1.wrapping_mul( 13 ) )
                .wrapping_add( world_origin.2.wrapping_mul( 107 ) )
                as u64
            );

            let rng_val = rng.random::<u8>();

            if origin.1 >= 5 && rng_val > 250 {
                // world_holder = self.cube_generator.generate( dataset, world_holder, world_origin, size );
            } else if rng_val > 175 {
                world_holder = self.clouds_generator.generate( dataset, world_holder, world_origin, size );
                // world_holder = self.clouds_generator.generate( dataset, world_holder, world_origin, size );
            }
        }

        world_holder
    }
}

fn plant_tree( dataset:&mut VoxelDataset, world_holder:&mut Octree<Voxel>, coords:(u32, u32, u32) ) {
    if coords.0 < 2 || coords.1 > CHUNK_SIZE_U32 - 8 || coords.2 < 2 {
        return
    }

    let log = create_voxel( dataset, (String::from( "log" ), Material { _density:10 }), (String::from( "log" ), Color { red:175, green:40, blue:20 }) );
    let leaves = create_voxel( dataset, (String::from( "leaves" ), Material { _density:10 }), (String::from( "leaves" ), Color { red:20, green:100, blue:20 }) );

    world_holder.fill_voxels(
        (coords.0, coords.1, coords.2),
        (coords.0, coords.1 + 5, coords.2),
        Some( log )
    );

    world_holder.fill_voxels(
        (coords.0 - 2, coords.1 + 5, coords.2 - 2),
        (coords.0 + 2, coords.1 + 7, coords.2 + 2),
        Some( Arc::clone( &leaves ) )
    );

    world_holder.fill_voxels(
        (coords.0 - 1, coords.1 + 8, coords.2 - 1),
        (coords.0 + 1, coords.1 + 8, coords.2 + 1),
        Some( Arc::clone( &leaves ) )
    );
}