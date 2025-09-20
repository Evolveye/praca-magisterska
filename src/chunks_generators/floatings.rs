use crate::{
    chunks_generators::utilities::create_voxel, noise::simplex_noise::SimplexNoise, structure_tests::octree::Octree, world::{
        world_generator::WorldGenerative,
        world_holder::{ Material, Voxel, VoxelDataset, WorldHolding }
    }
};

pub struct GeneratorOfFloatings {
    noise: SimplexNoise,
    noise_frequency: f64,
    color_top: (u8, u8, u8),
    color: (u8, u8, u8),
    generation_treeeshold: f64,
}

impl GeneratorOfFloatings {
    #[allow(dead_code)]
    pub fn new( seed:u32) -> Self {
        Self {
            noise: SimplexNoise::new( seed ),
            noise_frequency: 0.025,
            color_top: (25, 150, 15),
            color: (100, 50, 15),
            generation_treeeshold: 0.9,
        }
    }

    pub fn set_colors( &mut self, color:(u8,u8,u8), color_top:(u8,u8,u8) ) {
        self.color = color;
        self.color_top = color_top;
    }

    pub fn generate( &self, dataset:&mut VoxelDataset, mut world_holder:Octree<Voxel>, world_origin:(i64, i64, i64), size:u8 ) -> Octree<Voxel> {
        let size = size as i64;

        for x in world_origin.0..world_origin.0 + size {
            for y in (world_origin.1..world_origin.1 + size).rev() {
                for z in world_origin.2..world_origin.2 + size {
                    let value = self.noise.noise3d(
                        (x as f64 + 1.0) * self.noise_frequency,
                        (y as f64 + 1.0) * self.noise_frequency,
                        (z as f64 + 1.0) * self.noise_frequency,
                    );

                    let voxel = if value < self.generation_treeeshold { None } else {
                        let value_above = self.noise.noise3d(
                            (x as f64 + 1.0) * self.noise_frequency,
                            (y as f64 + 2.0) * self.noise_frequency,
                            (z as f64 + 1.0) * self.noise_frequency,
                        );

                        let (color, density) = if value_above < self.generation_treeeshold {
                            (self.color_top, 10)
                        } else {
                            (self.color, 20)
                        };

                        Some( create_voxel(
                            dataset,
                            (String::from( "grass" ), Material { _density:density }),
                            (format!( "pastel-r={},g={},b={}", color.0, color.1, color.2 ), color.into() ),
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

impl WorldGenerative for GeneratorOfFloatings {
    fn generate_chunk( &self, dataset:&mut VoxelDataset, origin:(i64, i64, i64), size:u8 ) -> Octree<Voxel> {
        let world_holder = Octree::from_max_size( size as u32 );
        let world_origin = (origin.0 * size as i64, origin.1 * size as i64, origin.2 * size as i64);

        self.generate( dataset, world_holder, world_origin, size )
    }
}
