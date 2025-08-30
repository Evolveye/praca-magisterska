use crate::{
    structure_tests::octree::Octree,
    world::{
        world_generator::WorldGenerative,
        world_holder::{ Voxel, VoxelDataset }
    }
};

pub struct GeneratorOfTest1Empty {
}

impl GeneratorOfTest1Empty {
    #[allow(dead_code)]
    pub fn new( _seed:u32) -> Self {
        Self {}
    }
}

impl WorldGenerative for GeneratorOfTest1Empty {
    fn generate_chunk( &self, _dataset:&mut VoxelDataset, _origin:(i64, i64, i64), size:u8 ) -> Octree<Voxel> {
        Octree::from_max_size( size as u32 )
    }
}
