use std::{
    collections::{HashMap, VecDeque}, sync::{ atomic::AtomicU64, mpsc, Arc, Condvar, Mutex, RwLock }, thread, time::Duration, vec
};

use crate::world::{chunk_region_iterator::ChunkRegionIterator, world::{ ChunkLoaderId, GridPosition, CHUNK_SIZE as CHUNK_SIZE_USIZE }, world_chunk::{WorldChunk, WorldChunkState}, world_generator::WorldGenerative, world_holder::VoxelDataset};

const CHUNK_SIZE:i64 = CHUNK_SIZE_USIZE as i64;

static NEXT_CMD_GROUP_ID:AtomicU64 = AtomicU64::new(1);

#[derive(Hash, PartialEq, Eq, Clone)]
pub struct GroupId(u64);

impl GroupId {
    pub fn new() -> Self {
        GroupId( NEXT_CMD_GROUP_ID.fetch_add( 1, std::sync::atomic::Ordering::Relaxed ) )
    }
}

type WorldChunkCoord = i64;

pub struct ChunksDataset {
    pub chunks: RwLock<HashMap<(WorldChunkCoord, WorldChunkCoord, WorldChunkCoord), RwLock<WorldChunk>>>,
    pub default_generator: Box<dyn WorldGenerative>,
}

impl ChunksDataset {
    pub fn new( default_generator:Box<dyn WorldGenerative> ) -> Self {
        Self {
            chunks: RwLock::new( HashMap::new() ),
            default_generator,
        }
    }
}

#[allow(dead_code)]
pub enum ChunkCmd {
    EnsureChunks( GroupId, GridPosition, u32, u32 ),
    GenerateChunks( GroupId, GridPosition, u32, u32 ),
    MultithreadedRemeshChunks( GridPosition, u32, u32 ),
    RemeshChunks( GridPosition, u8 ),
    UpdateChunkLoaderChunks( ChunkLoaderId, u8, GridPosition, GridPosition ),
}

#[allow(dead_code)]
pub enum ChunkRes {
    ChunksEnsured( Vec<((i64, i64, i64), RwLock<WorldChunk>)>, GroupId, GridPosition, u32, u32 ),
    ChunksStateUpdate( ChunkLoaderId, Vec<GridPosition>, Vec<GridPosition> ),
    ChunksGenerated( GroupId ),
}

pub fn start_chunk_worker( worker_id:u8, chunks_dataset:&Arc<ChunksDataset>, tasks_lock:&Arc<(Mutex<VecDeque<ChunkCmd>>,Condvar)>, tx:mpsc::Sender<ChunkRes> ) {
    let name = format!( "chunk-worker-{worker_id}" );

    thread::Builder::new()
        .name( name.clone() )
        .spawn( {
            let chunks_dataset = Arc::clone( chunks_dataset );
            let tasks_lock = Arc::clone( tasks_lock );

            move || {
                println!( "Worker \"{name}\" has been started" );
                let (lock, cvar) = &*tasks_lock;

                loop {
                    // print!( "Iteration start" );

                    let mut tasks = cvar
                        .wait_while( lock.lock().unwrap(), |t| t.is_empty() )
                        .unwrap();

                    // println!( " | tasks.len={}", tasks.len() );
                    let task = tasks.pop_front().unwrap();
                    drop( tasks );

                    // let task = cvar
                    //     .wait_while( lock.lock().unwrap(), |t| t.is_empty() )
                    //     .unwrap()
                    //     .pop_front()
                    //     .unwrap();

                    // main            gen          main
                    // EnsureChunks -> NewChunks -> FillChunks

                    // main                       gen
                    // UpdateChunkLoaderChunks -> ChunksStateUpdate


                    match task {
                        ChunkCmd::EnsureChunks( id, position, index_from, count ) => {
                            let index_to = index_from + count;
                            let new_chunks = get_nonexistant_chunks( &chunks_dataset, position, index_from, index_to );
                            let _ = tx.send( ChunkRes::ChunksEnsured( new_chunks, id, position, index_from, index_to ) );
                        },
                        ChunkCmd::GenerateChunks( id, position, index_from, index_to ) => {
                            generate_chunks( &chunks_dataset, position, index_from, index_to );
                            let _ = tx.send( ChunkRes::ChunksGenerated( id ) );
                        }
                        ChunkCmd::RemeshChunks( position, render_distance ) => {
                            remesh_chunks( &chunks_dataset, position, render_distance );
                        }
                        ChunkCmd::MultithreadedRemeshChunks( position, index_from, count ) => {
                            let index_to = index_from + count;
                            multithreaded_remesh_chunks( &chunks_dataset, position, index_from, index_to );
                        }
                        ChunkCmd::UpdateChunkLoaderChunks( loader_id, render_distance, new_pos, shift ) => {
                            update_chunk_loader_chunks( &tx, loader_id, render_distance, new_pos, shift );
                        }
                    }
                }
            }
        } )
        .expect( "Failed to spawn thread" );
}

fn generate_chunks( chunks_dataset:&Arc<ChunksDataset>, position:GridPosition, index_from:u32, index_to:u32 ) {
    // println!( "generate_chunks" );

    let mut cube_layer_iter = ChunkRegionIterator::with_range( index_from..index_to );
    let mut dataset = VoxelDataset::new();
    let mut chunks_pos_to_generate = vec![];


    // Collecting chunks to generate
    let chunks = chunks_dataset.chunks.read().unwrap();
    loop {
        let Some( relative_pos ) = cube_layer_iter.next() else { break };
        let chunk_pos = (
            position.0 + relative_pos.0 as i64,
            position.1 + relative_pos.1 as i64,
            position.2 + relative_pos.2 as i64
        );

        if let Some( chunk_lock ) = chunks.get( &chunk_pos ) {
            if matches!( chunk_lock.read().unwrap().state, WorldChunkState::Empty ) {
                // println!( "index_from={index_from} {: >2?} | side={}, {:?}", cube_layer_iter.iterations, cube_layer_iter.side, relative_pos );
                chunks_pos_to_generate.push( chunk_pos );
            }
        }
    }
    drop( chunks );

    // Generating the chunks
    // println!( "Generating the chunks" );
    for pos in chunks_pos_to_generate {
        let chunk_data = chunks_dataset.default_generator.generate_chunk( &mut dataset, pos, CHUNK_SIZE as u8 );
        let chunks = chunks_dataset.chunks.read().unwrap();
        let Some( chunk ) = chunks.get( &pos ) else { continue };
        let mut chunk = chunk.write().unwrap();

        if matches!( chunk.state, WorldChunkState::Empty ) {
            chunk.set_data( chunk_data );
        }
    }
}

fn update_chunk_loader_chunks( tx:&mpsc::Sender<ChunkRes>, loader_id:ChunkLoaderId, render_distance:u8, position:GridPosition, shift:GridPosition ) {
    let render_distance = render_distance as i64;

    let render_distance_addition_surrounding_x = shift.0.signum() * (render_distance + 1);
    let render_distance_addition_surrounding_y = shift.1.signum() * (render_distance + 1);
    let render_distance_addition_surrounding_z = shift.2.signum() * (render_distance + 1);

    let render_distance_addition_rendering_x = shift.0.signum() * render_distance;
    let render_distance_addition_rendering_y = shift.1.signum() * render_distance;
    let render_distance_addition_rendering_z = shift.2.signum() * render_distance;

    let from_s_x = position.0 - render_distance_addition_surrounding_x;
    let from_s_y = position.1 - render_distance_addition_surrounding_y;
    let from_s_z = position.2 - render_distance_addition_surrounding_z;

    let from_r_x = position.0 - render_distance_addition_rendering_x;
    let from_r_y = position.1 - render_distance_addition_rendering_y;
    let from_r_z = position.2 - render_distance_addition_rendering_z;

    let mut update_coords = (vec![], vec![]);

    // println!( "position={position:?}, shift={shift:?}" );

    axis_processor( &mut update_coords, from_s_x, from_r_x, shift.0, position.1, position.2, render_distance, &|a, b, c| (a, b, c) );
    axis_processor( &mut update_coords, from_s_y, from_r_y, shift.1, position.0, position.2, render_distance, &|a, b, c| (b, a, c) );
    axis_processor( &mut update_coords, from_s_z, from_r_z, shift.2, position.0, position.1, render_distance, &|a, b, c| (b, c, a) );

    let _ = tx.send( ChunkRes::ChunksStateUpdate( loader_id, update_coords.0, update_coords.1 ) );
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

fn get_nonexistant_chunks( chunks_dataset:&Arc<ChunksDataset>, position:GridPosition, index_from:u32, index_to:u32 ) -> Vec<(GridPosition, RwLock<WorldChunk>)> {
    // println!( "get_nonexistant_chunks" );

    let mut new_chunks = vec![];
    let chunks = chunks_dataset.chunks.read().unwrap();
    let mut cube_layer_iter = ChunkRegionIterator::with_range( index_from..index_to );

    loop {
        let Some( relative_pos ) = cube_layer_iter.next() else { break };
        let chunk_pos = (
            position.0 + relative_pos.0 as i64,
            position.1 + relative_pos.1 as i64,
            position.2 + relative_pos.2 as i64
        );

        if !chunks.contains_key( &chunk_pos ) {
            new_chunks.push( (chunk_pos, RwLock::new( WorldChunk::new() )) );
        }
    }

    new_chunks
}

fn multithreaded_remesh_chunks( chunks_dataset:&Arc<ChunksDataset>, center_chunk_position:GridPosition, index_from:u32, index_to:u32 ) {
    // println!( "remesh_chunks" );

    let mut cube_layer_iter = ChunkRegionIterator::with_range( index_from..index_to );
    let chunks = chunks_dataset.chunks.read().unwrap();

    'chunks: loop {
        let Some( relative_pos ) = cube_layer_iter.next() else { break };
        let chunk_pos = (
            center_chunk_position.0 + relative_pos.0 as i64,
            center_chunk_position.1 + relative_pos.1 as i64,
            center_chunk_position.2 + relative_pos.2 as i64
        );

        // println!( "index_from={index_from} {: >2?} | side={}, {:?}", cube_layer_iter.iterations, cube_layer_iter.side, relative_pos );

        let Some( chunk ) = chunks.get( &chunk_pos ) else {
            println!( "Chunk not exists ({chunk_pos:?})" );
            continue
        };

        let mut chunk = chunk.write().unwrap();
        if !matches!( chunk.state, WorldChunkState::Dirty ) { continue }

        let mut neighbours = vec![];

        for dy in -1..=1 {
            for dz in -1..=1 {
                for dx in -1..=1 {
                    if dx != 0 || dy != 0 || dz != 0 {
                        let Some( chunk ) = chunks.get( &(chunk_pos.0 + dx, chunk_pos.1 + dy, chunk_pos.2 + dz) ) else { continue 'chunks };
                        let chunk = chunk.read().unwrap();
                        if matches!( chunk.state, WorldChunkState::Empty ) { continue 'chunks }

                        neighbours.push( chunk );
                    }
                }
            }
        }

        // println!( "Remesihng {chunk_pos:?}" );
        chunk.remesh( chunk_pos, neighbours );
    }

    // println!( "{:?}", chunks.values().map( |c| format!( "{:?}", c.read().unwrap().state ) ).collect::<Vec<_>>() );
    // dbg!( chunks.values().map( |c| c.read().unwrap().state ).collect::<Vec<_>>() );
}


fn remesh_chunks( chunks_dataset:&Arc<ChunksDataset>, center_chunk_position:GridPosition, render_distance:u8 ) {
    // println!( "remesh_chunks start" );

    // Chunks meshing
    let render_distance = render_distance as i64;
    let chunks = chunks_dataset.chunks.read().unwrap();
    for y in -render_distance..=render_distance {
        for x in -render_distance..=render_distance {
            'chunks: for z in -render_distance..=render_distance {
                let chunk_pos = (center_chunk_position.0 + x, center_chunk_position.1 + y, center_chunk_position.2 + z);
                let Some( chunk ) = chunks.get( &chunk_pos ) else { continue };
                let mut chunk = chunk.write().unwrap();
                // if matches!( chunk.state, WorldChunkState::Meshed | WorldChunkState::Stashing ) { continue }
                if !matches!( chunk.state, WorldChunkState::Dirty ) { continue }

                let mut neighbours = vec![];

                for dy in -1..=1 {
                    for dz in -1..=1 {
                        for dx in -1..=1 {
                            if dx != 0 || dy != 0 || dz != 0 {
                                let Some( chunk ) = chunks.get( &(chunk_pos.0 + dx, chunk_pos.1 + dy, chunk_pos.2 + dz) ) else { continue 'chunks };
                                let chunk = chunk.read().unwrap();
                                if matches!( chunk.state, WorldChunkState::Empty ) { continue 'chunks }

                                neighbours.push( chunk );
                            }
                        }
                    }
                }

                chunk.remesh( chunk_pos, neighbours );
            }
        }
    }

    thread::sleep( Duration::from_secs( 5 ) );
    // println!( "remesh_chunks end" );

    // println!( "{:?}", chunks.values().map( |c| format!( "{:?}", c.read().unwrap().state ) ).collect::<Vec<_>>() );
    // dbg!( chunks.values().map( |c| c.read().unwrap().state ).collect::<Vec<_>>() );
}
