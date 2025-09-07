use std::{ hint::black_box, u32 };
use criterion::Criterion;

use praca_magisterska::{
    structure_tests::{ octree::Octree, voxel_hasher::VoxelHashMap, voxel_list::VoxelList, voxel_map::VoxelMap },
    world::world_holder::Voxel
};

pub fn measure_structs_initialization( c:&mut Criterion ) {
    let mut group = c.benchmark_group( "Structs initialization" );

    group.bench_function( "VoxelMap (size 2)", |b| b.iter( ||
        black_box( VoxelMap::<Voxel>::from_max_size( black_box( 2 ) ) )
    ) );

    group.bench_function( "Octree (size 1000)", |b| b.iter( ||
        black_box( Octree::<Voxel>::from_max_size( black_box( 1000 ) ) )
    ) );

    group.bench_function( "VoxelList", |b| b.iter( ||
        black_box( VoxelList::<Voxel>::new() )
    ) );

    group.bench_function( "VoxelHashMap", |b| b.iter( ||
        black_box( VoxelHashMap::<Voxel>::new() )
    ) );


    drop( group );
    let mut group = c.benchmark_group( "Structs initialization (without VoxelMap)" );

    group.bench_function( "Octree (size 1000)", |b| b.iter( ||
        black_box( Octree::<Voxel>::from_max_size( black_box( 1000 ) ) )
    ) );

    group.bench_function( "Octree (size 1_000_000)", |b| b.iter( ||
        black_box( Octree::<Voxel>::from_max_size( black_box( 1_000_000 ) ) )
    ) );

    group.bench_function( "Octree (size u32::MAX)", |b| b.iter( ||
        black_box( Octree::<Voxel>::from_max_size( black_box( u32::MAX ) ) )
    ) );

    group.bench_function( "VoxelList", |b| b.iter( ||
        black_box( VoxelList::<Voxel>::new() )
    ) );

    group.bench_function( "VoxelHashMap", |b| b.iter( ||
        black_box( VoxelHashMap::<Voxel>::new() )
    ) );



    drop( group );
    let mut group = c.benchmark_group( "VoxelMap initialization" );

    group.bench_function( "VoxelMap (size 2)", |b| b.iter( ||
        black_box( VoxelMap::<Voxel>::from_max_size( black_box( 2 ) ) )
    ) );

    group.bench_function( "VoxelMap (size 10)", |b| b.iter( ||
        black_box( VoxelMap::<Voxel>::from_max_size( black_box( 10 ) ) )
    ) );

    group.bench_function( "VoxelMap (size 20)", |b| b.iter( ||
        black_box( VoxelMap::<Voxel>::from_max_size( black_box( 20 ) ) )
    ) );

    group.bench_function( "VoxelMap (size 30)", |b| b.iter( ||
        black_box( VoxelMap::<Voxel>::from_max_size( black_box( 30 ) ) )
    ) );
}
