mod world;
mod noise;
mod structure_tests;
mod chunks_generators;
mod rendering;
mod app;
pub mod flags;

use crate::app::app::App;

// use crate::measurements::measure;
// mod measurements;
// #[allow(unused_imports)]
// use crate::{ app::app::App, structure_tests::octree::Octree, world::chunk_region_iterator::ChunkRegionIterator };

#[cfg(feature = "dhat-heap")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

fn main() {
    // measure()

    pretty_env_logger::init();

    let mut app = App::new().unwrap();
    app.run_loop();

    // test_cube_with_side( 1, ChunkRegionIterator::with_range( 20..27 ) );
    // test_cube_with_side( 1, ChunkRegionIterator::with_range( 1..27 ) );
    // test_cube_with_side( 1, ChunkRegionIterator::with_range( 0..2 ) );
    // test_cube_with_side( 2, ChunkRegionIterator::with_range( 27..(27 + 98) ) );
    // test_cube_with_side( 2, ChunkRegionIterator::with_range( 105..(27 + 98) ) );
    // test_cube_indices_with_side( 1, 0, 26 );
    // test_cube_indices_with_side( 2, 0, 98 );
    // test_cube_indices_with_side( 2, 78, 20 );
    // test_cube_indices_with_side( 2, 80, 20 );
}
