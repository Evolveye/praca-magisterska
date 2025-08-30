use crate::{
    chunks_generators::utilities::{create_voxel, get_pastel_color, hash_u32},
    structure_tests::octree::Octree,
    world::{
        world_generator::WorldGenerative,
        world_holder::{ Material, Voxel, VoxelDataset, WorldHolding }
    }
};

pub struct GeneratorOfTest8FullWithDifferenties {
    colors_count: u8,
}

impl GeneratorOfTest8FullWithDifferenties {
    #[allow(dead_code)]
    pub fn new( _seed:u32) -> Self {
        Self {
            colors_count: 250,
        }
    }
}

impl WorldGenerative for GeneratorOfTest8FullWithDifferenties {
    fn generate_chunk( &self, dataset:&mut VoxelDataset, _origin:(i64, i64, i64), size:u8 ) -> Octree<Voxel> {
        let mut world_holder = Octree::from_max_size( size as u32 );
        let size = size as u32;
        let colors_count = self.colors_count as u32;

        for x in 0..size {
            for y in 0..size {
                for z in 0..size {
                    let i = x + y * size + z * size * size;
                    let color_seed = hash_u32( i );
                    let color = get_pastel_color( color_seed, colors_count );

                    world_holder.set_voxel(
                        x,
                        y,
                        z,
                        Some( create_voxel(
                            dataset,
                            (String::from( "pastel" ), Material { _density:10 }),
                            (format!( "pastel-r={},g={},b={}", color.0, color.1, color.2 ), color.into() ),
                        ) ),
                    );
                }
            }
        }

        world_holder
    }
}
