use std::{collections::HashMap, time::Instant};

use crate::world::{
    world_chunk::WorldChunk, world_generator::WorldGenerative,
    world_holder::{ VoxelDataset, VoxelSide }
};

type WorldChunkCoord = i64;

pub static CHUNK_SIZE:usize = 64; // should be <= 64, because it is bit capacity of u64
pub static CHUNK_SIZE_X2:usize = CHUNK_SIZE * CHUNK_SIZE;
pub static CHUNK_SIZE_X3:usize = CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE;

#[allow(dead_code)]
pub struct World {
    dataset: VoxelDataset,
    chunks: HashMap<(WorldChunkCoord, WorldChunkCoord, WorldChunkCoord), WorldChunk>,
}

impl World {
    pub fn new( render_distance:u16, default_generator:impl WorldGenerative ) -> Self {
        debug_assert!( CHUNK_SIZE <= 64, "CHUNK_SIZE should be <= 64, because it is bit capacity of u64" );
        println!();
        println!( "World generation. Render distance = {}, chunk size = {}", render_distance, CHUNK_SIZE );

        let world_generation_time = Instant::now();
        let render_distance = render_distance as i64;
        let mut dataset = VoxelDataset::new();
        let mut chunks = HashMap::with_capacity( (render_distance as usize + 1).pow( 3 ) );

        // chunks.insert( (0, 0,  0), default_generator.generate_chunk( (0, 0,  0), 16, &mut dataset ) );
        // chunks.insert( (0, 0, -1), default_generator.generate_chunk( (0, 0, -1), 16, &mut dataset ) );
        // chunks.insert( (0, 0, -2), default_generator.generate_chunk( (0, 0, -2), 16, &mut dataset ) );
        // chunks.insert( (1, 0,  0), default_generator.generate_chunk( (1, 0,  0), 16, &mut dataset ) );
        // chunks.insert( (0, 0, -1), default_generator.generate_chunk( (0, 0, -1), 16, &mut dataset ) );

        let enlarged_render_dist = render_distance + 1;
        let size = enlarged_render_dist as usize * 2 + 1;
        let mut chunks_arr = Vec::with_capacity( size * size * size );

        for y in -enlarged_render_dist..=enlarged_render_dist {
            for x in -enlarged_render_dist..=enlarged_render_dist {
                for z in -enlarged_render_dist..=enlarged_render_dist {
                    let chunk_pos = (x, y, z);
                    chunks_arr.push( (chunk_pos, default_generator.generate_chunk( chunk_pos, CHUNK_SIZE as u8, &mut dataset )) );
                }
            }
        }

        let remeshing_time = Instant::now();
        let chunks_ptr = chunks_arr.as_mut_ptr();
        for y in 1..=(render_distance as usize * 2 + 1) {
            for x in 1..=(render_distance as usize * 2 + 1) {
                for z in 1..=(render_distance as usize * 2 + 1) {
                    unsafe {
                        let (chunk_pos, ref mut chunk) = *chunks_ptr.add( y * size * size + x * size + z );

                        chunk.remesh( chunk_pos, [
                            /*  0  1  2 */ &chunks_arr[ (y - 1) * size * size + (x - 1) * size + (z - 1) ].1, &chunks_arr[ (y - 1) * size * size + x * size + (z - 1) ].1, &chunks_arr[ (y - 1) * size * size + (x + 1) * size + (z - 1) ].1,
                            /*  3  4  5 */ &chunks_arr[ (y - 1) * size * size + (x - 1) * size +  z      ].1, &chunks_arr[ (y - 1) * size * size + x * size +  z      ].1, &chunks_arr[ (y - 1) * size * size + (x + 1) * size +  z      ].1,
                            /*  6  7  8 */ &chunks_arr[ (y - 1) * size * size + (x - 1) * size + (z + 1) ].1, &chunks_arr[ (y - 1) * size * size + x * size + (z + 1) ].1, &chunks_arr[ (y - 1) * size * size + (x + 1) * size + (z + 1) ].1,

                            /*  9 10 11 */ &chunks_arr[  y      * size * size + (x - 1) * size + (z - 1) ].1, &chunks_arr[  y      * size * size + x * size + (z - 1) ].1, &chunks_arr[  y      * size * size + (x + 1) * size + z ].1,
                            /* 12    13 */ &chunks_arr[  y      * size * size + (x - 1) * size +  z      ].1, /* center; remeshed chunk                                */  &chunks_arr[  y      * size * size + (x + 1) * size + z ].1,
                            /* 14 15 16 */ &chunks_arr[  y      * size * size + (x - 1) * size + (z + 1) ].1, &chunks_arr[  y      * size * size + x * size + (z + 1) ].1, &chunks_arr[  y      * size * size + (x + 1) * size + z ].1,

                            /* 17 18 19 */ &chunks_arr[ (y + 1) * size * size + (x - 1) * size + (z - 1) ].1, &chunks_arr[ (y + 1) * size * size + x * size + (z - 1) ].1, &chunks_arr[ (y + 1) * size * size + (x + 1) * size + (z - 1) ].1,
                            /* 20 21 22 */ &chunks_arr[ (y + 1) * size * size + (x - 1) * size +  z      ].1, &chunks_arr[ (y + 1) * size * size + x * size +  z      ].1, &chunks_arr[ (y + 1) * size * size + (x + 1) * size +  z      ].1,
                            /* 23 24 25 */ &chunks_arr[ (y + 1) * size * size + (x - 1) * size + (z + 1) ].1, &chunks_arr[ (y + 1) * size * size + x * size + (z + 1) ].1, &chunks_arr[ (y + 1) * size * size + (x + 1) * size + (z + 1) ].1,
                        ] );
                    }
                }
            }
        }


        chunks.extend( chunks_arr );

        // let print_chunk_coords = (0, 0, 0);
        println!( " - Chunks remeshing took {:?}", remeshing_time.elapsed() );
        println!( " - World generation took {:?}", world_generation_time.elapsed() );
        println!( " - World size = {}^3 = {}", CHUNK_SIZE as i64 * enlarged_render_dist, (CHUNK_SIZE as i64 * enlarged_render_dist).pow( 3 ) );
        println!( " - Chunks count = {}", chunks.len() );
        println!();
        // println!( "Print of chunk {print_chunk_coords:?} 14th and 13th layers:" );
        // chunks.get( &print_chunk_coords ).unwrap().print_bitmask_layer( 14 );
        // chunks.get( &print_chunk_coords ).unwrap().print_bitmask_layer( 13 );
        // panic!();

        Self { chunks, dataset }
    }

    pub fn get_renderables( &self ) -> Vec<VoxelSide> {
        self.chunks.values().flat_map( |c| c.renderables.clone() ).collect::<Vec<VoxelSide>>()
    }
}