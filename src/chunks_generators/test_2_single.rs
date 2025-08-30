use crate::{
    chunks_generators::utilities::create_voxel,
    structure_tests::octree::Octree,
    world::{
        world_generator::WorldGenerative,
        world_holder::{ Material, Voxel, VoxelDataset, WorldHolding }
    }
};

pub struct GeneratorOfTest2Single {
    color: (u8, u8, u8),
}

impl GeneratorOfTest2Single {
    #[allow(dead_code)]
    pub fn new( _seed:u32) -> Self {
        Self {
            color: (25, 150, 15)
        }
    }
}

impl WorldGenerative for GeneratorOfTest2Single {
    fn generate_chunk( &self, dataset:&mut VoxelDataset, origin:(i64, i64, i64), size:u8 ) -> Octree<Voxel> {
        let mut world_holder = Octree::from_max_size( size as u32 );

        if origin.0 == 0 && origin.1 == 0 && origin.2 == 0 {
            world_holder.set_voxel(
                0,
                0,
                0,
                Some( create_voxel(
                    dataset,
                    (String::from( "grass" ), Material { _density:10 }),
                    (String::from( "grass" ), self.color.into() ),
                ) )
            );
        }

        world_holder
    }
}
