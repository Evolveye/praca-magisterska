use std::collections::HashMap;

use crate::world::{
    world_chunk::WorldChunk, world_generator::WorldGenerative,
    world_holder::{ VoxelDataset, VoxelSide }
};

type WorldChunkCoord = i64;

#[allow(dead_code)]
pub struct World {
    dataset: VoxelDataset,
    chunks: HashMap<(WorldChunkCoord, WorldChunkCoord, WorldChunkCoord), WorldChunk>,
}

impl World {
    pub fn new( render_distance:u16, default_generator:impl WorldGenerative ) -> Self {
        let render_distance = render_distance as i64;

        let mut dataset = VoxelDataset::new();
        let mut chunks = HashMap::with_capacity( (render_distance as usize + 1).pow( 2 ) );

        // chunks.insert( (0, 0,  0), default_generator.generate_chunk( (0, 0,  0), 16, &mut dataset ) );
        // chunks.insert( (0, 0, -1), default_generator.generate_chunk( (0, 0, -1), 16, &mut dataset ) );
        // chunks.insert( (0, 0, -2), default_generator.generate_chunk( (0, 0, -2), 16, &mut dataset ) );
        // chunks.insert( (1, 0,  0), default_generator.generate_chunk( (1, 0,  0), 16, &mut dataset ) );
        // chunks.insert( (0, 0, -1), default_generator.generate_chunk( (0, 0, -1), 16, &mut dataset ) );

        for x in -render_distance..=render_distance {
            for y in -render_distance..=render_distance {
                for z in -render_distance..=render_distance {
                    println!( "Generating chunk {x} {y} {z}" );
                    chunks.insert( (x, y, z), default_generator.generate_chunk( (x, y, z), 16, &mut dataset ) );
                }
            }
        }

        Self { chunks, dataset }
    }

    pub fn get_renderables( &self ) -> Vec<VoxelSide> {
        self.chunks.values().flat_map( |c| c.renderables.clone() ).collect::<Vec<VoxelSide>>()
    }
}