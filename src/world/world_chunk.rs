use crate::{structure_tests::octree::Octree, world::world_holder::{Voxel, VoxelSide}};

#[allow(dead_code)]
pub struct WorldChunk {
    pub data: Octree<Voxel>,
    pub renderables: Vec<VoxelSide>,
}

impl WorldChunk {
    pub fn from_data( data:Octree<Voxel>, offset:(i64, i64, i64) ) -> Self {
        println!( " - from data: offset={:?}", offset );
        let renderables = data.get_visible_with_flood( (0, data.get_size() as u32 - 1, 0) )
            .into_iter()
            .filter_map( |mut s| {
                // if s.get_position().x != 0.0 { return None }
                s.move_by( (offset.0 as f32, offset.1 as f32, offset.2 as f32) );
                Some( s )
            } )
            .collect::<Vec<VoxelSide>>();

        Self { data, renderables }
    }
}
