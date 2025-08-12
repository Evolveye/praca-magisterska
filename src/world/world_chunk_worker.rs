use std::{
    collections::HashMap, sync::{ mpsc,Arc, RwLock }, thread, vec
};

use crate::world::{chunk_region_iterator::ChunkRegionIterator, world::{ GridPosition, Position, CHUNK_SIZE as CHUNK_SIZE_USIZE }, world_chunk::{WorldChunk, WorldChunkState}, world_generator::WorldGenerative, world_holder::VoxelDataset};

const CHUNK_SIZE:i64 = CHUNK_SIZE_USIZE as i64;

type WorldChunkCoord = i64;

pub struct ChunksDataset {
    pub chunks: RwLock<HashMap<(WorldChunkCoord, WorldChunkCoord, WorldChunkCoord), RwLock<WorldChunk>>>,
    pub default_generator: Box<dyn WorldGenerative>,
    pub worker_tasks: Vec<ChunkCmd>,
}

impl ChunksDataset {
    pub fn new( default_generator:Box<dyn WorldGenerative> ) -> Self {
        Self {
            chunks: RwLock::new( HashMap::new() ),
            default_generator,
            worker_tasks: vec![],
        }
    }
}

#[allow(dead_code)]
pub enum ChunkCmd {
    GenerateChunks( GridPosition, u8 ),
    EnsureChunks( GridPosition, u8 ),
    FillChunks( GridPosition, u8 ),
    UpdateChunkLoaderChunks( Position, Position, u8 ),
}

#[allow(dead_code)]
pub enum ChunkRes {
    ChunksStateUpdate( Vec<GridPosition>, Vec<GridPosition> ),
    NewChunks( Vec<(GridPosition, RwLock<WorldChunk>)>, GridPosition, u8 ),
}

pub fn start_chunk_worker( chunks_dataset:&Arc<ChunksDataset>, rx:mpsc::Receiver<ChunkCmd>, tx:mpsc::Sender<ChunkRes> ) {
    thread::Builder::new()
        .name( "chunk-processor".into() )
        .spawn( {
            let chunks_dataset = Arc::clone( chunks_dataset );

            move || {
                while let Ok( cmd ) = rx.recv() {
                    // main            gen          main
                    // EnsureChunks -> NewChunks -> FillChunks

                    // main                       gen
                    // UpdateChunkLoaderChunks -> ChunksStateUpdate

                    match cmd {
                        ChunkCmd::GenerateChunks( position, render_distance ) => {
                            generate_chunks( position, 0 );
                        }
                        ChunkCmd::EnsureChunks( position, render_distance ) => {
                            create_missing_chunks( &tx, &chunks_dataset, position, render_distance as i64 );
                        },
                        ChunkCmd::FillChunks( position, render_distance ) => {
                            generate_chunk_filling( &chunks_dataset, position, render_distance as i64 );
                        }
                        ChunkCmd::UpdateChunkLoaderChunks( position, move_to, render_distance ) => {
                            update_chunk_loader_chunks( &tx, &chunks_dataset, position, move_to, render_distance );
                        }
                    }
                }
            }
        } )
        .expect( "Failed to spawn thread" );
}

fn generate_chunks( position:GridPosition, from:usize ) {
    let mut cube_layer_iter = ChunkRegionIterator::with_range( 0..100 );
    let mut i = 0;

    loop {
        let Some( next ) = cube_layer_iter.next() else { break };
        println!( "{i: >2} | side={}, {:?}", cube_layer_iter.side, next );

        if i > 30 { break }
        i += 1;
    }
}

fn iterate_throught_cube_layer( layer:u8 ) {

}

fn update_chunk_loader_chunks( tx:&mpsc::Sender<ChunkRes>, chunks_dataset:&Arc<ChunksDataset>, position:Position, move_to:Position, render_distance:u8 ) {
    let render_distance = render_distance as i64;
    let loader_chunk_x = (position.0 as i64).div_euclid( CHUNK_SIZE );
    let loader_chunk_y = (position.1 as i64).div_euclid( CHUNK_SIZE );
    let loader_chunk_z = (position.2 as i64).div_euclid( CHUNK_SIZE );

    let move_to_chunk_x = (move_to.0 as i64).div_euclid( CHUNK_SIZE );
    let move_to_chunk_y = (move_to.1 as i64).div_euclid( CHUNK_SIZE );
    let move_to_chunk_z = (move_to.2 as i64).div_euclid( CHUNK_SIZE );

    let shift_x = move_to_chunk_x - loader_chunk_x;
    let shift_y = move_to_chunk_y - loader_chunk_y;
    let shift_z = move_to_chunk_z - loader_chunk_z;

    if shift_x | shift_y | shift_z == 0 {
        return
    }

    let move_to_chunk = (move_to_chunk_x, move_to_chunk_y, move_to_chunk_z);
    // println!( "gen: New chunk {move_to_chunk:?}" );

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

    let mut update_coords = (vec![], vec![]);

    axis_processor( &mut update_coords, from_s_x, from_r_x, shift_x, loader_chunk_y, loader_chunk_z, render_distance, &|a, b, c| (a, b, c) );
    axis_processor( &mut update_coords, from_s_y, from_r_y, shift_y, loader_chunk_x, loader_chunk_z, render_distance, &|a, b, c| (b, a, c) );
    axis_processor( &mut update_coords, from_s_z, from_r_z, shift_z, loader_chunk_x, loader_chunk_y, render_distance, &|a, b, c| (b, c, a) );

    let _ = tx.send( ChunkRes::ChunksStateUpdate( update_coords.0, update_coords.1 ) );
    create_missing_chunks( &tx, &chunks_dataset, move_to_chunk, render_distance );
}

fn axis_processor(
    update_coords:&mut (Vec<GridPosition>, Vec<GridPosition>),
    from_sim:i64,
    from_rend:i64,
    shift:i64,
    second_dim:i64,
    third_dim:i64,
    render_distance:i64,
    get_coords:&dyn Fn(i64, i64, i64) -> (i64, i64, i64)
) {
    let ranges = if shift < 0 {
        ((from_sim + shift + 1)..(from_sim + 1), (from_rend + shift + 1)..(from_rend + 1) )
    } else {
        (from_sim..(from_sim + shift), from_rend..(from_rend + shift) )
    };

    // Logic
    for a in ranges.0 {
        for b in (second_dim - render_distance - 1)..=(second_dim + render_distance + 1) {
            for c in (third_dim - render_distance - 1)..=(third_dim + render_distance + 1) {
                update_coords.0.push( get_coords( a, b, c ) );
            }
        }
    }

    // Render
    for a in ranges.1 {
        for b in (second_dim - render_distance)..=(second_dim + render_distance) {
            for c in (third_dim - render_distance)..=(third_dim + render_distance) {
                update_coords.1.push( get_coords( a, b, c ) );
            }
        }
    }
}

fn create_missing_chunks( tx:&mpsc::Sender<ChunkRes>, chunks_dataset:&Arc<ChunksDataset>, center_chunk_position:GridPosition, render_distance:i64 ) {
    let enlarged_render_dist = render_distance + 1;
    let mut new_chunks = vec![];
    let chunks = chunks_dataset.chunks.read().unwrap();

    for y in -enlarged_render_dist..=enlarged_render_dist {
        for x in -enlarged_render_dist..=enlarged_render_dist {
            for z in -enlarged_render_dist..=enlarged_render_dist {
                let chunk_pos = (center_chunk_position.0 + x, center_chunk_position.1 + y, center_chunk_position.2 + z);

                if !chunks.contains_key( &chunk_pos ) {
                    new_chunks.push( (chunk_pos, RwLock::new( WorldChunk::new() )) );
                }
            }
        }
    }

    let _ = tx.send( ChunkRes::NewChunks( new_chunks, center_chunk_position, render_distance as u8 ) );
}

fn generate_chunk_filling( chunks_dataset:&Arc<ChunksDataset>, center_chunk_position:GridPosition, render_distance:i64 ) {
    let enlarged_render_dist = render_distance + 1;
    let mut dataset = VoxelDataset::new();
    let mut chunks_pos_to_generate = vec![];

    // Collecting chunks to generate
    let chunks = chunks_dataset.chunks.read().unwrap();
    for y in -enlarged_render_dist..=enlarged_render_dist {
        for x in -enlarged_render_dist..=enlarged_render_dist {
            for z in -enlarged_render_dist..=enlarged_render_dist {
                let chunk_pos = (center_chunk_position.0 + x, center_chunk_position.1 + y, center_chunk_position.2 + z);

                if let Some( chunk_lock ) = chunks.get( &chunk_pos ) {
                    if matches!( chunk_lock.read().unwrap().state, WorldChunkState::Empty ) {
                        chunks_pos_to_generate.push( chunk_pos );
                    }
                }
            }
        }
    }
    drop( chunks );

    // Generating the chunks
    for pos in chunks_pos_to_generate {
        let chunk_data = chunks_dataset.default_generator.generate_chunk( &mut dataset, pos, CHUNK_SIZE as u8 );
        let chunks = chunks_dataset.chunks.read().unwrap();
        let Some( chunk ) = chunks.get( &pos ) else { continue };
        let mut chunk = chunk.write().unwrap();

        if matches!( chunk.state, WorldChunkState::Empty ) {
            chunk.set_data( chunk_data );
        }
    }

    // Chunks meshing
    let chunks = chunks_dataset.chunks.read().unwrap();
    for y in -render_distance..=render_distance {
        for x in -render_distance..=render_distance {
            'chunks: for z in -render_distance..=render_distance {
                let chunk_pos = (center_chunk_position.0 + x, center_chunk_position.1 + y, center_chunk_position.2 + z);
                let Some( chunk ) = chunks.get( &chunk_pos ) else { continue };
                let mut chunk = chunk.write().unwrap();
                if matches!( chunk.state, WorldChunkState::Meshed | WorldChunkState::Stashing ) { continue }

                let mut neighbours = vec![];

                for dy in -1..=1 {
                    for dz in -1..=1 {
                        for dx in -1..=1 {
                            if dx != 0 || dy != 0 || dz != 0 {
                                let Some( chunk ) = chunks.get( &(chunk_pos.0 + dx, chunk_pos.1 + dy, chunk_pos.2 + dz) ) else { continue 'chunks };
                                neighbours.push( chunk.read().unwrap() );
                            }
                        }
                    }
                }

                chunk.remesh( chunk_pos, neighbours );
            }
        }
    }
}
