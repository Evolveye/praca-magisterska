use std::{
    cell::RefCell,
    collections::{HashMap, VecDeque},
    sync::{ self, mpsc, Arc, Condvar, Mutex, RwLock }, time::Instant,
};

use crate::{app::camera::{Camera, Frustum, FrustumCheck}, flags::{FLAG_PROFILING_WORLD_GENERATION, FLAG_PROFILING_WORLD_RENDERING}, world::{
    world_chunk::{ WorldChunk, WorldChunkState }, world_chunk_worker::{ start_chunk_worker, ChunkCmd, ChunkRes, ChunksDataset, GroupId }, world_generator::WorldGenerative, world_holder::{ VoxelDataset, VoxelSide }
}};

pub type ChunkLoaderId = u16;
pub type GridPosition = (i64, i64, i64);
pub type Position = (f32, f32, f32);

pub static CHUNK_SIZE:usize = 64; // should be <= 64, because it is bit capacity of u64
pub static CHUNK_SIZE_X2:usize = CHUNK_SIZE * CHUNK_SIZE;
pub static CHUNK_SIZE_X3:usize = CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE;

const CHUNK_SIZE_F32:f32 = CHUNK_SIZE as f32;


pub struct ChunkLoader {
    id: ChunkLoaderId,
    render_distance: u8,
    position: Position,
}

pub type ChunkLoaderhandle = Arc<RefCell<ChunkLoader>>;

enum BlockingTask {
    ChunksToRemove( Vec<(i64, i64, i64)> ),
    ChunksEnsured( Vec<((i64, i64, i64), RwLock<WorldChunk>)>, GroupId, GridPosition, u32, u32 ),
}



#[allow(dead_code)]
pub struct World {
    chunks_generation_group_size: u32,
    tasks_receiver_single_tick_size: u8,
    chunks_dataset: Arc<ChunksDataset>,
    pub max_radius: Option<u8>,
    chunk_loaders: HashMap<ChunkLoaderId, sync::Weak<RefCell<ChunkLoader>>>,
    dataset: VoxelDataset,
    // chunks_tx: mpsc::Sender<ChunkCmd>,
    chunks_rx: mpsc::Receiver<ChunkRes>,
    worker_tasks: Arc<(Mutex<VecDeque<ChunkCmd>>,Condvar)>,
    blocking_tasks_queue: VecDeque<BlockingTask>,
    tasks_groups: HashMap<GroupId,(Option<ChunkLoaderId>, u32, Instant)>,
    pub debug_meshes: Vec<VoxelSide>,
}

impl World {
    pub fn new( default_generator:Box<dyn WorldGenerative>, max_radius:Option<u8> ) -> Self {
        debug_assert!( CHUNK_SIZE <= 64, "CHUNK_SIZE should be <= 64, because it is bit capacity of u64" );

        // let (cmd_tx, cmd_rx) = mpsc::channel();
        let (res_tx, res_rx) = mpsc::channel();
        let chunks_dataset = Arc::new( ChunksDataset::new( default_generator ) );
        let worker_tasks = Arc::new( (Mutex::new( VecDeque::<ChunkCmd>::new() ), Condvar::new()) );

        for i in 0..12 {
            start_chunk_worker( i, &chunks_dataset, &worker_tasks, res_tx.clone() );
        }

        Self {
            chunks_generation_group_size: 40,
            tasks_receiver_single_tick_size: 10,
            chunks_dataset,
            max_radius,
            dataset: VoxelDataset::new(),
            chunk_loaders: HashMap::new(),
            // chunks_tx: cmd_tx,
            chunks_rx: res_rx,
            worker_tasks,
            blocking_tasks_queue: VecDeque::new(),
            tasks_groups: HashMap::new(),
            debug_meshes: vec![],
        }
    }

    pub fn create_chunk_loader( &mut self, position:Position, render_distance:u8 ) -> ChunkLoaderhandle {
        let id = self.chunk_loaders.len() as ChunkLoaderId;
        let chunk_loader = Arc::new( RefCell::new( ChunkLoader { id, position, render_distance } ) );
        // let chunk_loader = Rc::new( RefCell::new( ChunkLoader { id, position, render_distance } ) );

        // println!( "create_chunk_loader | {:?}", position );
        self.chunk_loaders.insert( id, Arc::downgrade( &chunk_loader ) );

        self.load_chunks( WorldChunk::get_chunk_position_from_world_position( position ), render_distance, Some( id ) );

        chunk_loader
    }

    pub fn get_renderables( &mut self, camera:&Camera ) -> Vec<VoxelSide> {
        // println!( "Getting renderables" );

        // let renderables = self.chunks_dataset.chunks.read().unwrap().iter().flat_map( |(_coords, chunk_lock)| {
        //     if let Ok( chunk ) = chunk_lock.try_read() {
        //         if matches!( chunk.state, WorldChunkState::Meshed ) {
        //             chunk.renderables.clone()
        //         } else {
        //             vec![]
        //         }
        //     } else {
        //         vec![]
        //     }
        // } ).collect::<Vec<VoxelSide>>();

        // return renderables;


        // TODO make it working properly

        let mut meshes = Vec::new();
        for loader in self.chunk_loaders.values() {
            // println!( "Throught chunk loaders" );

            if let Some( loader ) = loader.upgrade() {
                // println!( "Upgrade" );

                if let Ok( loader ) = loader.try_borrow() {
                    let render_distance = loader.render_distance as f32;
                    let loader_pos = WorldChunk::get_chunk_position_from_world_position( loader.position );

                    let min = (
                        loader_pos.0 as f32 - render_distance,
                        loader_pos.1 as f32 - render_distance,
                        loader_pos.2 as f32 - render_distance,
                    );

                    // let max = (
                    //     min.0 + CHUNK_SIZE_F32,
                    //     min.1 + CHUNK_SIZE_F32 * 2.0,
                    //     min.2 + CHUNK_SIZE_F32,
                    // );
                    let max = (
                        loader_pos.0 as f32 + render_distance + 1.0,
                        loader_pos.1 as f32 + render_distance + 1.0,
                        loader_pos.2 as f32 + render_distance + 1.0,
                    );

                    self.collect_visible_chunks( &mut meshes, &camera.frustum, min, max, CHUNK_SIZE_F32 );
                }
            }
        }

        self.debug_meshes = meshes.clone();
        meshes
    }

    fn collect_visible_chunks( &self, result:&mut Vec<VoxelSide>, frustum:&Frustum, min:Position, max:Position, step:f32 ) {
        let world_min = (min.0 * step, min.1 * step, min.2 * step);
        let world_max = (max.0 * step, max.1 * step, max.2 * step);

        match frustum.intersects_aabb( world_min, world_max ) {
            FrustumCheck::Outside => {
                // Nothing to see
            }
            FrustumCheck::Inside => {
                let chunks = self.chunks_dataset.chunks.read().unwrap();
                let max = (max.0 as i64, max.1 as i64, max.2 as i64);
                let mut x = min.0 as i64;
                let mut y = min.1 as i64;
                let mut z = min.2 as i64;

                while x < max.0 {
                    while y < max.1 {
                        while z < max.2 {
                            if let Some( chunk ) = chunks.get( &(x, y, z) ) {
                                if let Ok( chunk ) = chunk.try_read() {
                                    result.extend( chunk.renderables.clone() );
                                }
                            }

                            z += 1;
                        }

                        y += 1;
                    }

                    x += 1;
                }
            }
            FrustumCheck::Intersect => {
                let grid_min = (min.0 as i64, min.1 as i64, min.2 as i64);
                let size_x = (max.0 - min.0) as i64;
                let size_y = (max.1 - min.1) as i64;
                let size_z = (max.2 - min.2) as i64;

                if size_x <= 1 && size_y <= 1 && size_z <= 1 {
                    if let Some( chunk ) = self.chunks_dataset.chunks.read().unwrap().get( &grid_min ) {
                        if let Ok( chunk ) = chunk.try_read() {
                            result.extend( chunk.renderables.clone() );
                        }
                    }
                } else {
                    let ranges_x = if size_x >= 2 {
                        let mid = min.0 + (size_x / 2) as f32;
                        vec![(min.0, mid), (mid, max.0)]
                    } else {
                        vec![(min.0, max.0)]
                    };

                    let ranges_y = if size_y >= 2 {
                        let mid = min.1 + (size_y / 2) as f32;
                        vec![(min.1, mid), (mid, max.1)]
                    } else {
                        vec![(min.1, max.1)]
                    };

                    let ranges_z = if size_z >= 2 {
                        let mid = min.2 + (size_z / 2) as f32;
                        vec![(min.2, mid), (mid, max.2)]
                    } else {
                        vec![(min.2, max.2)]
                    };

                    for x_range in &ranges_x {
                        for y_range in &ranges_y {
                            for z_range in &ranges_z {
                                self.collect_visible_chunks(
                                    result,
                                    frustum,
                                    (x_range.0, y_range.0, z_range.0),
                                    (x_range.1, y_range.1, z_range.1),
                                    step,
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn move_chunk_loader_to( &mut self, chunk_loader:&ChunkLoaderhandle, move_to:Position, freezed:bool ) {
        let mut loader = chunk_loader.borrow_mut();
        if !self.chunk_loaders.contains_key( &loader.id ) { return };

        let position = loader.position;
        let chunk_size = CHUNK_SIZE as i64;
        let loader_data = (loader.id, loader.render_distance);


        // println!( "move_chunk_loader_to | {:?}, {:?}", loader.position, move_to );
        loader.position = move_to;
        drop( loader );

        let loader_chunk_x = (position.0 as i64).div_euclid( chunk_size );
        let loader_chunk_y = (position.1 as i64).div_euclid( chunk_size );
        let loader_chunk_z = (position.2 as i64).div_euclid( chunk_size );

        let move_to_chunk_x = (move_to.0 as i64).div_euclid( chunk_size );
        let move_to_chunk_y = (move_to.1 as i64).div_euclid( chunk_size );
        let move_to_chunk_z = (move_to.2 as i64).div_euclid( chunk_size );

        let shift_x = move_to_chunk_x - loader_chunk_x;
        let shift_y = move_to_chunk_y - loader_chunk_y;
        let shift_z = move_to_chunk_z - loader_chunk_z;

        if shift_x | shift_y | shift_z != 0 {
            let new_loader_chunk_pos = (move_to_chunk_x, move_to_chunk_y, move_to_chunk_z);
            println!( "New loader pos = {new_loader_chunk_pos:?}" );

            if freezed {
                return
            }

            let mut tasks = self.worker_tasks.0.lock().unwrap();

            tasks.push_back( ChunkCmd::UpdateChunkLoaderChunks(
                loader_data.0,
                loader_data.1,
                // (move_to_chunk_x, move_to_chunk_y, move_to_chunk_z),
                (loader_chunk_x, loader_chunk_y, loader_chunk_z),
                (shift_x, shift_y, shift_z),
            ) );

            self.worker_tasks.1.notify_one();

            // self.load_chunks( (move_to_chunk_x, move_to_chunk_y, move_to_chunk_z), loader_data.1, Some( loader_data.0 ) );
        }

    }

    pub fn update( &mut self ) {
        // println!( "World update" );

        for _ in 0..self.tasks_receiver_single_tick_size {
            let res = match self.chunks_rx.try_recv() {
                Ok( res ) => res,
                _ => break,
            };

            match res {
                ChunkRes::ChunksStateUpdate( loader_id, chunks_to_remove, chunks_to_calculable ) => {
                    // println!( "main: ChunksStateUpdate" );
                    // println!( "main:  - chunks_to_remove={chunks_to_remove:?}" );
                    // println!( "main:  - chunks_to_calculable={chunks_to_calculable:?}" );

                    let chunks = self.chunks_dataset.chunks.read().unwrap();
                    // println!( "main: {:?}", chunks.iter()
                    //     .filter_map( |(k, c)| if matches!( c.read().unwrap().state, WorldChunkState::Meshed ) {
                    //         Some( k )
                    //         // Some( format!( "{:?}", c.read().unwrap().state ) )
                    //     } else {
                    //         None
                    //     } )
                    //     .collect::<Vec<_>>()
                    // );

                    for pos in chunks_to_calculable {
                        if let Some( chunk ) = chunks.get( &pos ) {
                            chunk.write().unwrap().state = WorldChunkState::Calculable;
                        }
                    }
                    for pos in &chunks_to_remove {
                        if let Some( chunk ) = chunks.get( &pos ) {
                            chunk.write().unwrap().state = WorldChunkState::Stashing;
                        }
                    }
                    drop( chunks );

                    let Some( chunk_loader ) = self.chunk_loaders.get( &loader_id ) else { break };
                    let Some( chunk_loader ) = chunk_loader.upgrade() else { break };
                    let chunk_loader = chunk_loader.borrow();
                    let chunk_pos = WorldChunk::get_chunk_position_from_world_position( chunk_loader.position );

                    self.load_chunks( chunk_pos, chunk_loader.render_distance, Some( chunk_loader.id ) );

                    self.blocking_tasks_queue.push_back( BlockingTask::ChunksToRemove( chunks_to_remove ) );
                },

                ChunkRes::ChunksEnsured( new_chunks, id, position, index_from, index_to ) => {
                    self.blocking_tasks_queue.push_back( BlockingTask::ChunksEnsured( new_chunks, id, position, index_from, index_to ) );
                }

                ChunkRes::ChunksGenerated( group_id ) => {
                    let group_tasks = self.tasks_groups.get_mut( &group_id ).unwrap();
                    group_tasks.1 -= 1;

                    // println!( "ChunkRes::ChunksGenerated | queue = {}", group_tasks.1 );

                    if group_tasks.1 == 0 {
                        let Some( loader_id ) = group_tasks.0 else { break };
                        let Some( chunk_loader ) = self.chunk_loaders.get( &loader_id ) else { break };
                        let Some( chunk_loader ) = chunk_loader.upgrade() else { break };
                        let chunk_loader = chunk_loader.borrow();
                        let chunk_pos = WorldChunk::get_chunk_position_from_world_position( chunk_loader.position );
                        let render_distance = chunk_loader.render_distance;
                        let loader_id = chunk_loader.id;
                        drop( chunk_loader );

                        let meshing_id = GroupId::new();

                        // let diameter = render_distance as u32 * 2 + 1;
                        // let cube_size = diameter * diameter * diameter;
                        // let mut tasks = self.worker_tasks.0.lock().unwrap();
                        // let mut i = 0;

                        // println!( "{:?}", self.chunks_dataset.chunks.read().unwrap().keys() );

                        // loop {
                        //     let count = if cube_size - i >= 5 { 5 } else { cube_size - i };
                        //     tasks.push_back( ChunkCmd::MultithreadedRemeshChunks( chunk_pos, i, count ) );

                        //     // println!( "ChunkRes::ChunksGenerated (remeshing request) | {count}, {i} < {cube_size}, render_distance={render_distance}, tasks.len={}", tasks.len() );

                        //     i += 5;
                        //     if i >= cube_size { break }
                        // }

                        // drop( tasks );
                        // self.worker_tasks.1.notify_all();

                        // println!( "Remesh queued" );

                        if FLAG_PROFILING_WORLD_GENERATION {
                            println!( "Chunks generation time: {:?}", group_tasks.2.elapsed() );
                        }

                        self.tasks_groups.insert( meshing_id.clone(), (Some( loader_id ), 1, Instant::now()) );
                        self.worker_tasks.0.lock().unwrap().push_back( ChunkCmd::RemeshChunks( meshing_id, chunk_pos, render_distance ) );
                        self.worker_tasks.1.notify_one();
                    }
                },

                ChunkRes::ChunksMeshed( group_id ) => {
                    let group_tasks = self.tasks_groups.get_mut( &group_id ).unwrap();
                    group_tasks.1 -= 1;

                    if FLAG_PROFILING_WORLD_RENDERING && group_tasks.1 == 0 {
                        println!( "Chunks meshing time: {:?}", group_tasks.2.elapsed() );
                    }
                }

                // ChunkRes::NewChunks( chunks, position, render_distance ) => {
                //     // println!( "main: NewChunks" );

                //     self.chunks_dataset.chunks.write().unwrap().extend( chunks );

                //     let tx = self.chunks_tx.clone();
                //     let _ = tx.send( ChunkCmd::RemeshChunks( position, render_distance ) );
                // },
            }
        }

        if self.blocking_tasks_queue.len() > 0 {
            let Ok( mut chunks ) = self.chunks_dataset.chunks.try_write() else { return };

            for _ in 0..self.tasks_receiver_single_tick_size {
                let Some( task ) = self.blocking_tasks_queue.pop_front() else { break };

                match task {
                    BlockingTask::ChunksToRemove( chunks_to_remove ) => {
                        for pos in chunks_to_remove {
                            chunks.remove( &pos );
                        }
                    }

                    BlockingTask::ChunksEnsured( new_chunks, id, position, index_from, index_to ) => {
                        chunks.extend( new_chunks );

                        self.worker_tasks.0.lock().unwrap().push_back( ChunkCmd::GenerateChunks( id, position, index_from, index_to ) );
                        self.worker_tasks.1.notify_one();
                    }
                }
            }
        }
    }

    fn load_chunks( &mut self, center_chunk_position:GridPosition, render_distance:u8, loader_id:Option<ChunkLoaderId> ) {
        let diameter = (render_distance + 1) as u32 * 2 + 1;
        let cube_size = diameter * diameter * diameter;
        let generation_id = GroupId::new();
        let mut tasks = vec![];
        let mut i = 0;
        let group_size = self.chunks_generation_group_size;

        loop {
            let count = if cube_size - i >= group_size { group_size } else { cube_size - i };
            tasks.push( ChunkCmd::EnsureChunks( generation_id.clone(), center_chunk_position, self.max_radius, i, count ) );

            i += group_size;
            if i >= cube_size { break }
        }

        self.tasks_groups.insert( generation_id, (loader_id, i / group_size, Instant::now()) );
        self.worker_tasks.0.lock().unwrap().extend( tasks );
        self.worker_tasks.1.notify_all();
    }
}
