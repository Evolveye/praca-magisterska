use crate::{
    chunks_generators::utilities::{ create_voxel, generate_unique, get_pastel_color, generate_woksel_index, hash_u32 },
    structure_tests::octree::Octree,
    world::{
        world_generator::WorldGenerative,
        world_holder::{ Material, Voxel, VoxelDataset, WorldHolding }
    }
};

pub struct GeneratorOfTest7HalfRanfomWithDifferenties {
    colors_count: u16,
}

impl GeneratorOfTest7HalfRanfomWithDifferenties {
    #[allow(dead_code)]
    pub fn new( _seed:u32) -> Self {
        Self {
            colors_count: 1000,
        }
    }
}

impl WorldGenerative for GeneratorOfTest7HalfRanfomWithDifferenties {
    fn generate_chunk( &self, dataset:&mut VoxelDataset, origin:(i64, i64, i64), size:u8 ) -> Octree<Voxel> {
        let mut world_holder = Octree::from_max_size( size as u32 );
        let size = size as usize;
        let uint_origin = (origin.0.abs(), origin.1.abs(), origin.2.abs());

        let randoms = generate_unique(
            (uint_origin.0 as u64) << 32 | (uint_origin.1 as u64) << 16 | (uint_origin.2 as u64),
            size * size * size / 2
        );

        let size = size as u32;
        let colors_count = self.colors_count as u32;

        for i in randoms {
            let x = i % size;
            let y = (i / size) % size;
            let z = i / (size * size);

            let color_seed = hash_u32( generate_woksel_index( origin, (x, y, z), size ) );
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

        println!( "dataset lengths | voxels = {}, colors = {}", dataset.voxels.len(), dataset.colors.len() );

        world_holder
    }
}
