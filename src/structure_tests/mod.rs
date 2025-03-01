pub mod tester;
pub mod voxel_map;
pub mod voxel_list;

use tester::WorldHolder;

#[allow(unused_imports)]
use voxel_list::VoxelList;

#[allow(unused_imports)]
use voxel_map::VoxelMap;

pub fn run_test() {
    println!( "" );
    println!( "Starting tester" );

    let tester = tester::Tester {};
    // let mut world_struct = VoxelMap::new();
    let mut world_struct = VoxelList::new();
    let dataset = tester.fill_1( &mut world_struct );

    println!( "" );
    dataset.get_size();
    println!( "" );
    world_struct.get_size();
    println!( "" );
}
