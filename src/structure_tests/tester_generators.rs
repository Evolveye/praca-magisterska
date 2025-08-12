use crate::{
    noise::simplex_noise::SimplexNoise, structure_tests::{
        octree::Octree, quadtree::Quadtree
    },
    world::{
        world_generator::WorldGenerative,
        world_holder::{
            fill_with, Color, Voxel, VoxelDataset,
        }
    }
};

pub struct GeneratorOfRealisticallyTerrain {
    noise: SimplexNoise,
    noise_frequency: f64,
    noise_amplitude: f64,
}

impl GeneratorOfRealisticallyTerrain {
    pub fn new( seed:u32) -> Self {
        Self {
            noise: SimplexNoise::new( seed ),

            // noise_frequency: 0.1,
            // noise_frequency: 0.05,
            noise_frequency: 0.025,
            // noise_frequency: 0.013,
            // noise_frequency: 0.005,
            noise_amplitude: 10.0,
            // noise_amplitude: 15.0,
            // noise_amplitude: 20.0,
        }
    }
}

impl WorldGenerative for GeneratorOfRealisticallyTerrain {
    fn generate_chunk( &self, dataset:&mut VoxelDataset, origin:(i64, i64, i64), size:u8 ) -> Octree<Voxel> {
        // println!( "Chunk generation {:?}, size={}", origin, size );
        let origin = (origin.0 * size as i64, origin.1 * size as i64, origin.2 * size as i64);
        let size = size as u32;
        let max_depth = Quadtree::get_max_depth_for( size );
        let mut world_holder = Octree::from_max_size( size );
        let grass_level = 8 - origin.1;

        let quadtree = Quadtree::from_terrain_generation( max_depth, &|x, z| {
            self.noise.noise3d(
                // TODO Find out why "x" and "z" are swapped
                (origin.0 + z as i64) as f64 * self.noise_frequency,
                1.0,
                (origin.2 + x as i64) as f64 * self.noise_frequency,
            )
        } );

        quadtree.proces_entire_tree( &mut |offset, size, noise_value| {
            // if size < 2 { return offset.1 }

            let multiplied_noise = noise_value * self.noise_amplitude;
            let current_min = grass_level + multiplied_noise as i64;
            if current_min < 0 || current_min < offset.1 as i64 { return offset.1 }

            let size = size - 1;
            let to = (offset.0 + size, current_min as u32, offset.2 + size);

            {
                let below_water = current_min < grass_level - 5;
                let too_high = current_min > grass_level + 7;
                let grass_color = Color {
                    red: if current_min % 2 == 0 { 10 }
                        else if below_water { 20 }
                        else if too_high { 250 } else { 128 },
                    green: (127 + (multiplied_noise * 10.0) as i16) as u8,
                    blue: if below_water { 150 }
                        else if too_high { 250 }
                        else { 10 },
                };

                // let grass_color = Color {
                //     red: 10,
                //     green: 200,
                //     blue: 10,
                // };

                dataset.expand( fill_with( offset, to, &mut world_holder, (&format!( "grass_{}", current_min ), grass_color) ) );
            }

            to.1 + 1
        } );

        world_holder
    }
}
