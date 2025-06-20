use crate::world::{world_chunk::WorldChunk, world_holder::VoxelDataset};

pub trait WorldGenerative {
    fn generate_chunk( &self, origin:(i64, i64, i64), size:u8, dataset:&mut VoxelDataset ) -> WorldChunk;
}
