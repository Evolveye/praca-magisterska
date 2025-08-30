use crate::{
    chunks_generators::cube::GeneratorOfCube,
    structure_tests::octree::Octree,
    world::{
        world_generator::WorldGenerative,
        world_holder::{ Voxel, VoxelDataset }
    }
};

pub struct GeneratorOfTest9Natural {
    cube_generator: GeneratorOfCube,
}

impl GeneratorOfTest9Natural {
    #[allow(dead_code)]
    pub fn new( seed:u32) -> Self {
        Self {
            cube_generator: GeneratorOfCube::new( seed )
        }
    }
}

impl WorldGenerative for GeneratorOfTest9Natural {
    fn generate_chunk( &self, dataset:&mut VoxelDataset, origin:(i64, i64, i64), size:u8 ) -> Octree<Voxel> {
        self.cube_generator.generate_chunk( dataset, origin, size )
    }
}
