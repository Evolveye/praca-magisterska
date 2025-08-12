use crate::{
    structure_tests::octree::Octree,
    world::world_holder::{ Voxel, VoxelDataset }
};

pub trait WorldGenerative: Send + Sync {
    fn generate_chunk( &self, dataset:&mut VoxelDataset, origin:(i64, i64, i64), size:u8 ) -> Octree<Voxel>;
}
