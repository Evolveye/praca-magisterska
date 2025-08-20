use std::{
    cell::RefCell,
    collections::{HashMap, VecDeque},
    sync::{ self, mpsc, Arc, Condvar, Mutex, RwLock },
};

use crate::{app::camera::Camera, world::{
    world_chunk::{ WorldChunk, WorldChunkState }, world_chunk_worker::{ start_chunk_worker, ChunkCmd, ChunkRes, ChunksDataset, GroupId }, world_generator::WorldGenerative, world_holder::{ VoxelDataset, VoxelSide }
}};

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

pub type ChunkLoaderhandle = Arc<RefCell<ChunkLoader>>;

enum BlockingTask {
    ChunksToRemove( Vec<(i64, i64, i64)> ),
    ChunksEnsured( Vec<((i64, i64, i64), RwLock<WorldChunk>)>, GroupId, GridPosition, u32, u32 ),
}



#[allow(dead_code)]
pub struct World {
    chunks_dataset: Arc<ChunksDataset>,
    chunk_loaders: HashMap<ChunkLoaderId, sync::Weak<RefCell<ChunkLoader>>>,
    dataset: VoxelDataset,
    // chunks_tx: mpsc::Sender<ChunkCmd>,
    chunks_rx: mpsc::Receiver<ChunkRes>,
    worker_tasks: Arc<(Mutex<VecDeque<ChunkCmd>>,Condvar)>,
    blocking_tasks_queue: VecDeque<BlockingTask>,
    tasks_groups: HashMap<GroupId,(Option<ChunkLoaderId>, u32)>,
}

impl World {
    pub fn new( default_generator:Box<dyn WorldGenerative> ) -> Self {
        debug_assert!( CHUNK_SIZE <= 64, "CHUNK_SIZE should be <= 64, because it is bit capacity of u64" );

        // let (cmd_tx, cmd_rx) = mpsc::channel();
        let (res_tx, res_rx) = mpsc::channel();
        let chunks_dataset = Arc::new( ChunksDataset::new( default_generator ) );
        let worker_tasks = Arc::new( (Mutex::new( VecDeque::<ChunkCmd>::new() ), Condvar::new()) );

        for i in 0..3 {
            start_chunk_worker( i, &chunks_dataset, &worker_tasks, res_tx.clone() );
        }

        Self {
            chunks_dataset,
            dataset: VoxelDataset::new(),
            chunk_loaders: HashMap::new(),
            // chunks_tx: cmd_tx,
            chunks_rx: res_rx,
            worker_tasks,
            blocking_tasks_queue: VecDeque::new(),
            tasks_groups: HashMap::new(),
        }
    }

    pub fn create_chunk_loader( &mut self, position:Position, render_distance:u8 ) -> ChunkLoaderhandle {
        let id = self.chunk_loaders.len() as ChunkLoaderId;
        let chunk_loader = Arc::new( RefCell::new( ChunkLoader { id, position, render_distance } ) );
        // let chunk_loader = Rc::new( RefCell::new( ChunkLoader { id, position, render_distance } ) );

        self.chunk_loaders.insert( id, Arc::downgrade( &chunk_loader ) );

        self.load_chunks( WorldChunk::get_chunk_position_from_world_position( position ), render_distance, Some( id ) );

        chunk_loader
    }

    pub fn get_renderables( &self, _camera:&Camera ) -> Vec<VoxelSide> {
        // println!( "Getting renderables" );

        self.chunks_dataset.chunks.read().unwrap().iter().flat_map( |(_coords, chunk_lock)| {
            if let Ok( chunk ) = chunk_lock.try_read() {
                if matches!( chunk.state, WorldChunkState::Meshed ) {
                    chunk.renderables.clone()
                } else {
                    vec![]
                }
            } else {
                vec![]
            }
        } ).collect::<Vec<VoxelSide>>()
    }

    pub fn move_chunk_loader_to( &mut self, chunk_loader:&ChunkLoaderhandle, move_to:Position ) {
        let mut loader = chunk_loader.borrow_mut();
        if !self.chunk_loaders.contains_key( &loader.id ) { return };

        let position = loader.position;
        let chunk_size = CHUNK_SIZE as i64;
        let loader_data = (loader.id, loader.render_distance);

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

        while let Ok( res ) = self.chunks_rx.try_recv() {
            match res {
                ChunkRes::ChunksStateUpdate( loader_id, chunks_to_remove, chunks_to_calculable ) => {
                    // println!( "main: ChunksStateUpdate" );
                    // println!( "main:  - {chunks_to_remove:?}" );
                    // println!( "main:  - {chunks_to_calculable:?}" );

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
                        drop( chunk_loader );

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

                        self.worker_tasks.0.lock().unwrap().push_back( ChunkCmd::RemeshChunks( chunk_pos, render_distance ) );
                        self.worker_tasks.1.notify_one();
                    }
                },

                // ChunkRes::NewChunks( chunks, position, render_distance ) => {
                //     // println!( "main: NewChunks" );

                //     self.chunks_dataset.chunks.write().unwrap().extend( chunks );

                //     let tx = self.chunks_tx.clone();
                //     let _ = tx.send( ChunkCmd::RemeshChunks( position, render_distance ) );
                // },
            }
        }

        if self.blocking_tasks_queue.len() > 0 {
            if let Ok( mut chunks ) = self.chunks_dataset.chunks.try_write() {
                while let Some( task ) = self.blocking_tasks_queue.pop_front() {
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
    }

    fn load_chunks( &mut self, center_chunk_position:GridPosition, render_distance:u8, loader_id:Option<ChunkLoaderId> ) {
        // let render_distance = ChunkRegionIterator::get_layer_edge( render_distance as i32 + 1 ) as u32;
        let diameter = (render_distance + 1) as u32 * 2 + 1;
        let cube_size = diameter * diameter * diameter;
        let generation_id = GroupId::new();
        let mut tasks = self.worker_tasks.0.lock().unwrap();
        let mut i = 0;

        // println!( "load_chunks | render_distance={render_distance} cube_size={cube_size}" );
        loop {
            let count = if cube_size - i >= 5 { 5 } else { cube_size - i };
            tasks.push_back( ChunkCmd::EnsureChunks( generation_id.clone(), center_chunk_position, i, count ) );

            // println!( "load_chunks | {i} < {cube_size}, tasks.len={}", tasks.len() );

            i += 5;
            if i >= cube_size { break }
        }

        // println!( "load_chunks" );
        self.tasks_groups.insert( generation_id, (loader_id, i / 5) );
        self.worker_tasks.1.notify_all();
        // let _ = self.chunks_tx.send( ChunkCmd::EnsureChunks( center_chunk_position, render_distance ) );
    }
}
