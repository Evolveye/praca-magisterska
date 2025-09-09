use crate::{
    chunks_generators::utilities::create_voxel, noise::simplex_noise::SimplexNoise, structure_tests::{
        octree::Octree, quadtree::Quadtree
    }, world::{
        world_generator::WorldGenerative,
        world_holder::{
            fill_with, Color, Material, Voxel, VoxelDataset, WorldHolding
        }
    }
};

pub struct GeneratorOfTest11PeaksAndValleys {
    noise: SimplexNoise,
    noise_frequency: f64,
    noise_amplitude: f64,
}

impl GeneratorOfTest11PeaksAndValleys {
    #[allow(unused)]
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

impl WorldGenerative for GeneratorOfTest11PeaksAndValleys {
    fn generate_chunk( &self, dataset:&mut VoxelDataset, origin:(i64, i64, i64), size:u8 ) -> Octree<Voxel> {
        let mut world_holder = Octree::from_max_size( size as u32 );
        let grass_level = 8 - origin.1;

        let world_origin = (origin.0 * size as i64, origin.1 * size as i64, origin.2 * size as i64);
        let size = size as i64;

        if ((world_origin.1 + size) as f64) < -1.0 * self.noise_amplitude {
            let size = size as u32;
            let color = Color { red:204, green:204, blue:196 };
            let voxel = Some( create_voxel(
                dataset,
                (String::from( "grass" ), Material { _density:10 }),
                (format!( "pastel-r={},g={},b={}", color.red, color.green, color.blue ), color ),
            ) );

            world_holder.fill_voxels( (0, 0, 0), (size - 1, size - 1, size -1), voxel );

            return world_holder;
        }

        if (world_origin.1 as f64) > 1.0 * self.noise_amplitude {
            return world_holder;
        }

        for x in world_origin.0..world_origin.0 + size {
            for y in (world_origin.1..world_origin.1 + size).rev() {
                for z in world_origin.2..world_origin.2 + size {
                    let noise_value = self.noise.noise3d(
                        (x as f64 + 1.0) * self.noise_frequency,
                        (y as f64 + 1.0) * self.noise_frequency,
                        (z as f64 + 1.0) * self.noise_frequency,
                    );

                    let multiplied_noise = noise_value * self.noise_amplitude;
                    let current_min = grass_level + multiplied_noise as i64;

                    if y > current_min { continue }

                    let below_water = y < grass_level - 5;
                    let too_high = y > grass_level + 7;
                    let voxel = {
                        let color = Color {
                            red: if y % 2 == 0 { 10 }
                                else if below_water { 20 }
                                else if too_high { 250 } else { 128 },
                            green: (127 + (multiplied_noise * 10.0) as i16) as u8,
                            blue: if below_water { 150 }
                                else if too_high { 250 }
                                else { 10 },
                        };

                        Some( create_voxel(
                            dataset,
                            (String::from( "grass" ), Material { _density:10 }),
                            (format!( "pastel-r={},g={},b={}", color.red, color.green, color.blue ), color ),
                        ) )
                    };

                    world_holder.set_voxel(
                        (x - world_origin.0) as u32,
                        (y - world_origin.1) as u32,
                        (z - world_origin.2) as u32,
                        voxel
                    );
                }
            }
        }

        world_holder
    }
}
