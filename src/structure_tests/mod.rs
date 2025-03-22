pub mod tester;
pub mod voxel_map;
pub mod voxel_list;
pub mod octree;
pub mod voxel_hasher;

use octree::Octree;
use tester::WorldHolder;

use voxel_hasher::VoxelHashMap;
#[allow(unused_imports)]
use voxel_list::VoxelList;

#[allow(unused_imports)]
use voxel_map::VoxelMap;

pub fn run_test() {
    println!( "" );
    println!( "Starting tester" );

    let tester = tester::Tester {};
    // let mut world_struct = VoxelMap::new();
    // let mut world_struct = VoxelList::new();
    let mut world_struct = Octree::new( 18 );
    // let mut world_struct = VoxelHashMap::new();
    let dataset = tester.fill_100pc( &mut world_struct );

    println!( "" );
    println!( "{:?}", world_struct.get_voxel( 0, 0, 0 ) );
    println!( "{:?}", world_struct.get_voxel( 100, 100, 100 ) );

    println!( "" );
    dataset.get_size();
    println!( "" );
    world_struct.get_size();
    println!( "" );
}
