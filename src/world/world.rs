use std::{
    cell::RefCell,
    collections::HashMap,
    sync::{ self, mpsc, Arc },
};

use crate::world::{
    world_chunk::{ WorldChunk, WorldChunkState },
    world_chunk_worker::{ start_chunk_worker, ChunkCmd, ChunkRes, ChunksDataset },
    world_generator::WorldGenerative,
    world_holder::{ VoxelDataset, VoxelSide },
};

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



#[allow(dead_code)]
pub struct World {
    chunks_dataset: Arc<ChunksDataset>,
    chunk_loaders: HashMap<ChunkLoaderId, sync::Weak<RefCell<ChunkLoader>>>,
    dataset: VoxelDataset,
    chunks_tx: mpsc::Sender<ChunkCmd>,
    chunks_rx: mpsc::Receiver<ChunkRes>,
}

impl World {
    pub fn new( default_generator:Box<dyn WorldGenerative> ) -> Self {
        debug_assert!( CHUNK_SIZE <= 64, "CHUNK_SIZE should be <= 64, because it is bit capacity of u64" );

        let (cmd_tx, cmd_rx) = mpsc::channel();
        let (res_tx, res_rx) = mpsc::channel();
        let chunks_dataset = Arc::new( ChunksDataset::new( default_generator ) );

        start_chunk_worker( &chunks_dataset, cmd_rx, res_tx );

        let _ = cmd_tx.send( ChunkCmd::GenerateChunks( (1,1,1), 1 ) );

        Self {
            chunks_dataset,
            dataset: VoxelDataset::new(),
            chunk_loaders: HashMap::new(),
            chunks_tx: cmd_tx,
            chunks_rx: res_rx,
        }
    }

    pub fn create_chunk_loader( &mut self, position:Position, render_distance:u8 ) -> ChunkLoaderhandle {
        let id = self.chunk_loaders.len() as ChunkLoaderId;
        let chunk_loader = Arc::new( RefCell::new( ChunkLoader { id, position, render_distance } ) );
        // let chunk_loader = Rc::new( RefCell::new( ChunkLoader { id, position, render_distance } ) );

        self.chunk_loaders.insert( id, Arc::downgrade( &chunk_loader ) );

        self.load_chunks( WorldChunk::get_chunk_position_from_world_position( position ), render_distance );

        chunk_loader
    }

    pub fn get_renderables( &self ) -> Vec<VoxelSide> {
        // println!( "Getting renderables" );

        self.chunks_dataset.chunks.read().unwrap().values().flat_map( |c| {
            if let Ok( chunk ) = c.try_read() {
                if matches!( chunk.state, WorldChunkState::Calculable ) {
                    vec![]
                } else {
                    chunk.renderables.clone()
                }
            } else {
                vec![]
            }
        } ).collect::<Vec<VoxelSide>>()
    }

    pub fn move_chunk_loader_to( &mut self, chunk_loader:&ChunkLoaderhandle, move_to:Position ) {
        let mut loader = chunk_loader.borrow_mut();
        if !self.chunk_loaders.contains_key( &loader.id ) { return };

        let _ = self.chunks_tx.send( ChunkCmd::UpdateChunkLoaderChunks( loader.position, move_to, loader.render_distance ) );

        loader.position = move_to;
    }

    pub fn update( &mut self ) {
        // println!( "World update" );

        while let Ok( res ) = self.chunks_rx.try_recv() {
            match res {
                ChunkRes::ChunksStateUpdate( chunks_to_remove, chunks_to_calculable ) => {
                    // println!( "main: ChunksStateUpdate" );

                    let chunks = self.chunks_dataset.chunks.read().unwrap();
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

                    let mut chunks = self.chunks_dataset.chunks.write().unwrap();
                    for pos in chunks_to_remove {
                        chunks.remove( &pos );
                    }
                    drop( chunks );
                },

                // ChunkRes::NewChunk( position, chunk ) => {
                //     let mut chunks = self.chunks_dataset.chunks.write().unwrap();
                //     chunks.insert( position, RwLock::new( chunk ) );
                // },

                ChunkRes::NewChunks( chunks, position, render_distance ) => {
                    // println!( "main: NewChunks" );

                    self.chunks_dataset.chunks.write().unwrap().extend( chunks );

                    let tx = self.chunks_tx.clone();
                    let _ = tx.send( ChunkCmd::FillChunks( position, render_distance ) );
                },
            }
        }
    }

    fn load_chunks( &self, center_chunk_position:GridPosition, render_distance:u8 ) {
        let _ = self.chunks_tx.send( ChunkCmd::EnsureChunks( center_chunk_position, render_distance ) );
    }
}
