use std::{
    cell::RefCell,
    collections::HashMap,
    rc::{ Rc, Weak },
    time::Instant
};

use crate::world::{
    world_chunk::{ WorldChunk, WorldChunkState},
    world_generator::WorldGenerative,
    world_holder::{ VoxelDataset, VoxelSide }
};

type WorldChunkCoord = i64;
pub type ChunkLoaderId = u16;
pub type GridPosition = (i64, i64, i64);
pub type Position = (f32, f32, f32);

pub static CHUNK_SIZE:usize = 64; // should be <= 64, because it is bit capacity of u64
pub static CHUNK_SIZE_X2:usize = CHUNK_SIZE * CHUNK_SIZE;
pub static CHUNK_SIZE_X3:usize = CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE;



pub struct ChunkLoader {
    id: ChunkLoaderId,
    render_distance: u8,
    position: Position,
}

pub type ChunkLoaderhandle = Rc<RefCell<ChunkLoader>>;



#[allow(dead_code)]
pub struct World {
    dataset: VoxelDataset,
    chunk_loaders: HashMap<ChunkLoaderId, Weak<RefCell<ChunkLoader>>>,
    chunks: HashMap<(WorldChunkCoord, WorldChunkCoord, WorldChunkCoord), WorldChunk>,
    default_generator: Box<dyn WorldGenerative>
}

impl World {
    pub fn new( default_generator:Box<dyn WorldGenerative> ) -> Self {
        debug_assert!( CHUNK_SIZE <= 64, "CHUNK_SIZE should be <= 64, because it is bit capacity of u64" );

        Self {
            chunks: HashMap::new(),
            dataset: VoxelDataset::new(),
            chunk_loaders:HashMap::new(),
            default_generator,
        }
    }

    pub fn create_chunk_loader( &mut self, position:Position, render_distance:u8 ) -> ChunkLoaderhandle {
        let id = self.chunk_loaders.len() as ChunkLoaderId;
        let chunk_loader = Rc::new( RefCell::new( ChunkLoader { id, position, render_distance } ) );
        // let chunk_loader = Rc::new( RefCell::new( ChunkLoader { id, position, render_distance } ) );

        self.chunk_loaders.insert( id, Rc::downgrade( &chunk_loader ) );

        self.load_chuks( WorldChunk::get_chunk_position_from_world_position( position ), render_distance );

        chunk_loader
    }

    pub fn move_chunk_loader_to( &mut self, chunk_loader:&ChunkLoaderhandle, move_to:Position ) {
        let mut loader = chunk_loader.borrow_mut();
        if !self.chunk_loaders.contains_key( &loader.id ) { return };

        let chunk_size = CHUNK_SIZE as i64;
        let render_distance = loader.render_distance as i64;
        let loader_chunk_x = (loader.position.0 as i64).div_euclid( chunk_size );
        let loader_chunk_y = (loader.position.1 as i64).div_euclid( chunk_size );
        let loader_chunk_z = (loader.position.2 as i64).div_euclid( chunk_size );

        let shift_x = (move_to.0 as i64).div_euclid( chunk_size ) - loader_chunk_x;
        let shift_y = (move_to.1 as i64).div_euclid( chunk_size ) - loader_chunk_y;
        let shift_z = (move_to.2 as i64).div_euclid( chunk_size ) - loader_chunk_z;

        // println!( "Updating chunks ({loader_chunk_x} {loader_chunk_y} {loader_chunk_z})" );
        // println!( "Updating chunks ({shift_x} {shift_y} {shift_z}), {:?} > {move_to:?}", loader.position );

        loader.position = move_to;

        if shift_x | shift_y | shift_z == 0 {
            return
        }

        let render_distance_addition_surrounding_x = shift_x.signum() * (render_distance + 1);
        let render_distance_addition_surrounding_y = shift_y.signum() * (render_distance + 1);
        let render_distance_addition_surrounding_z = shift_z.signum() * (render_distance + 1);

        let render_distance_addition_rendering_x = shift_x.signum() * render_distance;
        let render_distance_addition_rendering_y = shift_y.signum() * render_distance;
        let render_distance_addition_rendering_z = shift_z.signum() * render_distance;

        let from_s_x = loader_chunk_x - render_distance_addition_surrounding_x;
        let from_s_y = loader_chunk_y - render_distance_addition_surrounding_y;
        let from_s_z = loader_chunk_z - render_distance_addition_surrounding_z;

        let from_r_x = loader_chunk_x - render_distance_addition_rendering_x;
        let from_r_y = loader_chunk_y - render_distance_addition_rendering_y;
        let from_r_z = loader_chunk_z - render_distance_addition_rendering_z;

        // println!( "Updating chunks (shifts = {shift_x} {shift_y} {shift_z} | chunk_pos = {loader_chunk_x} {loader_chunk_y} {loader_chunk_z})" );

        let mut axis_processor = |from_sim, from_rend, shift, second_dim, third_dim, get_coords:&dyn Fn(i64, i64, i64) -> (i64, i64, i64) | {
            let ranges = if shift < 0 {
                ((from_sim + shift + 1)..(from_sim + 1), (from_rend + shift + 1)..(from_rend + 1) )
            } else {
                (from_sim..(from_sim + shift), from_rend..(from_rend + shift) )
            };

            // println!( "Ranges | simu = {:?}", ranges.0 );
            // println!( "Ranges | rend = {:?}", ranges.1 );

            // println!( "Logic" );
            for a in ranges.0 {
                for b in (second_dim - render_distance - 1)..=(second_dim + render_distance + 1) {
                    for c in (third_dim - render_distance - 1)..=(third_dim + render_distance + 1) {
                        // println!( "- {:?}", &get_coords( a, b, c ) );
                        self.chunks.remove( &get_coords( a, b, c ) );
                    }
                }
            }

            // println!( "render" );
            for a in ranges.1 {
                for b in (second_dim - render_distance)..=(second_dim + render_distance) {
                    for c in (third_dim - render_distance)..=(third_dim + render_distance) {
                        if let Some( chunk ) = self.chunks.get_mut( &get_coords( a, b, c ) ) {
                            // println!( "- {:?}", &get_coords( a, b, c ) );
                            chunk.state = WorldChunkState::Calculable;
                        }
                    }
                }
            }
        };

        axis_processor( from_s_x, from_r_x, shift_x, loader_chunk_y, loader_chunk_z, &|a, b, c| (a, b, c) );
        axis_processor( from_s_y, from_r_y, shift_y, loader_chunk_x, loader_chunk_z, &|a, b, c| (b, a, c) );
        axis_processor( from_s_z, from_r_z, shift_z, loader_chunk_x, loader_chunk_y, &|a, b, c| (b, c, a) );

        self.load_chuks( WorldChunk::get_chunk_position_from_world_position( move_to ), loader.render_distance );
    }

    fn load_chuks( &mut self, center_chunk_position:GridPosition, render_distance:u8 ) {
        println!();
        println!( "World generation. Position = {:?}, render distance = {}, chunk size = {}", center_chunk_position, render_distance, CHUNK_SIZE );

        let world_generation_time = Instant::now();
        let render_distance = render_distance as i64;

        let chunk_size = CHUNK_SIZE as i64;
        let enlarged_render_dist = render_distance + 1;
        let size = enlarged_render_dist as usize * 2 + 1;
        let mut chunks_arr = Vec::with_capacity( size * size * size );

        for y in -enlarged_render_dist..=enlarged_render_dist {
            for x in -enlarged_render_dist..=enlarged_render_dist {
                for z in -enlarged_render_dist..=enlarged_render_dist {
                    let chunk_pos = (center_chunk_position.0 + x, center_chunk_position.1 + y, center_chunk_position.2 + z);
                    let chunk = if let Some( chunk ) = self.chunks.remove( &chunk_pos ) {
                        chunk
                    } else {
                        self.default_generator.generate_chunk( chunk_pos, CHUNK_SIZE as u8, &mut self.dataset )
                    };

                    chunks_arr.push( (chunk_pos, chunk) );
                }
            }
        }

        let remeshing_time = Instant::now();
        let chunks_ptr = chunks_arr.as_mut_ptr();
        let mut remeshed_count = 0;
        for y in 1..=(render_distance as usize * 2 + 1) {
            for x in 1..=(render_distance as usize * 2 + 1) {
                for z in 1..=(render_distance as usize * 2 + 1) {
                    if unsafe {
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
                        ] )
                    } {
                        remeshed_count += 1;
                    }
                }
            }
        }


        self.chunks.extend( chunks_arr );

        // let print_chunk_coords = (0, 0, 0);
        println!( " - Chunks remeshing took {:?}", remeshing_time.elapsed() );
        println!( " - World generation took {:?}", world_generation_time.elapsed() );
        println!( " - Chunks range = {:?}", -enlarged_render_dist..=enlarged_render_dist );
        // println!( " - Chunks keys = {:?}", self.chunks.keys() );
        println!( " - World size = {}^3 = {}", chunk_size * enlarged_render_dist, (chunk_size * enlarged_render_dist).pow( 3 ) );
        println!( " - Remeshed chunks count = {}", remeshed_count );
        println!( " - All chunks count = {}", self.chunks.len() );
        println!();
        // println!( "Print of chunk {print_chunk_coords:?} 14th and 13th layers:" );
        // chunks.get( &print_chunk_coords ).unwrap().print_bitmask_layer( 14 );
        // chunks.get( &print_chunk_coords ).unwrap().print_bitmask_layer( 13 );
        // panic!();

    }

    pub fn get_renderables( &self ) -> Vec<VoxelSide> {
        self.chunks.values().flat_map( |c| {
            if matches!( c.state, WorldChunkState::Calculable ) { vec![] } else { c.renderables.clone() }
        } ).collect::<Vec<VoxelSide>>()
    }
}
