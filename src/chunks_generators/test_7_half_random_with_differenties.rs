use crate::{
    chunks_generators::utilities::{create_voxel, generate_unique, get_pastel_color, hash_u32},
    structure_tests::octree::Octree,
    world::{
        world_generator::WorldGenerative,
        world_holder::{ Material, Voxel, VoxelDataset, WorldHolding }
    }
};

pub struct GeneratorOfTest7HalfRanfomWithDifferenties {
    colors_count: u8,
}

impl GeneratorOfTest7HalfRanfomWithDifferenties {
    #[allow(dead_code)]
    pub fn new( _seed:u32) -> Self {
        Self {
            colors_count: 250,
        }
    }
}

impl WorldGenerative for GeneratorOfTest7HalfRanfomWithDifferenties {
    fn generate_chunk( &self, dataset:&mut VoxelDataset, origin:(i64, i64, i64), size:u8 ) -> Octree<Voxel> {
        let mut world_holder = Octree::from_max_size( size as u32 );
        let size = size as usize;
        let randoms = generate_unique(
            (origin.0.abs() as u64) << 8 | (origin.1.abs() as u64) << 4 | (origin.2.abs() as u64),
            size * size * size / 2
        );

        let size = size as u32;
        let colors_count = self.colors_count as u32;

        for i in randoms {
            let x = i % size;
            let y = (i / size) % size;
            let z = i / (size * size);

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

        world_holder
    }
}
