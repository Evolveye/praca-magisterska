use std::{hint::black_box, sync::Arc};
use criterion::{BatchSize, Criterion};

use praca_magisterska::{
    chunks_generators::utilities::{create_voxel, generate_unique},
    structure_tests::{ octree::Octree, voxel_hasher::VoxelHashMap, voxel_list::VoxelList, voxel_map::VoxelMap },
    world::world_holder::{ Color, Material, Voxel, VoxelDataset, WorldHolding }
};

const SIZE:u32 = 100;

pub fn measure_structs_get_reference( c:&mut Criterion ) {
    let mut group = c.benchmark_group( "Get from the referencing arrays" );

    group.bench_function( format!( "Array (size 25)" ), |b| b.iter_batched(
        || [0; 25 * 25 * 25],
        |arr| {
            for i in 7800..7900 {
                black_box( arr[ black_box( i ) ]);
            }
        },
        BatchSize::SmallInput,
    ) );

    group.bench_function( "Vector (size 25)", |b| b.iter_batched(
        || vec![0; 25 * 25 * 25],
        |arr| {
            for i in 7800..7900 {
                black_box( arr[ black_box( i ) ]);
            }
        },
        BatchSize::SmallInput,
    ) );

    group.bench_function( format!( "Vector big (size {SIZE})" ), |b| b.iter_batched(
        || {
            let size = 100 * 100 * 100;
            let mut vec = Vec::with_capacity( size );

            for _ in 0..size {
                vec.push( 0 );
            }

            vec
        },
        |arr| {
            for i in 7800..7900 {
                black_box( arr[ black_box( i ) ]);
            }
        },
        BatchSize::SmallInput,
    ) );

    group.bench_function( format!( "Vector big with calc (size {SIZE})" ), |b| b.iter_batched(
        || {
            let size = 100 * 100 * 100;
            let mut vec = Vec::with_capacity( size );

            for _ in 0..size {
                vec.push( 0 );
            }

            vec
        },
        |arr| {
            for i in 0..100 {
                black_box( arr[
                    black_box( 55u32 ) as usize * black_box( 100 ) * black_box( 100 ) +
                    black_box( 55u32 ) as usize * black_box( 100 ) +
                    black_box( i ) as usize
                ]);
            }
        },
        BatchSize::SmallInput,
    ) );

    group.bench_function( format!( "Vector big with calc and Arc (size {SIZE})" ), |b| b.iter_batched(
        || {
            let size = 100 * 100 * 100;
            let mut vec = Vec::with_capacity( size );

            for _ in 0..size {
                vec.push( Arc::new( 0 ) );
            }

            vec
        },
        |arr| {
            for i in 0..100 {
                black_box( arr[
                    black_box( 55u32 ) as usize * black_box( 100 ) * black_box( 100 ) +
                    black_box( 55u32 ) as usize * black_box( 100 ) +
                    black_box( i ) as usize
                ].clone() );
            }
        },
        BatchSize::SmallInput,
    ) );

    let voxel= create_voxel(
        &mut VoxelDataset::new(),
        (String::from( "material" ), Material { _density:100 }),
        (String::from( "color" ), Color { red:250, green:10, blue:10 }),
    );

    group.bench_function( format!( "VoxelMap (size {SIZE})" ), |b| b.iter_batched(
        || {
            let mut wh = VoxelMap::<Voxel>::from_max_size( SIZE );
            wh.fill_voxels( (0, 0, 0), (SIZE - 1, SIZE - 1, SIZE - 1), black_box( Some( voxel.clone() ) ));
            wh
        },
        |wh| {
            for i in 0..100 {
                black_box( wh.get_voxel( black_box( 50 ), black_box( 50 ), black_box( i ) ) );
            }
        },
        BatchSize::SmallInput,
    ) );
}

pub fn measure_structs_get( c:&mut Criterion ) {
    let mut group = c.benchmark_group( "Get from the structs" );

    let voxel= create_voxel(
        &mut VoxelDataset::new(),
        (String::from( "material" ), Material { _density:100 }),
        (String::from( "color" ), Color { red:250, green:10, blue:10 }),
    );

    group.bench_function( "VoxelMap (size 100)", |b| b.iter_batched(
        || {
            let mut wh = VoxelMap::<Voxel>::from_max_size( 100 );
            wh.fill_voxels( (25, 25, 25), (74, 74, 74), black_box( Some( voxel.clone() ) ));
            wh
        },
        |wh| {
            for i in 0..100 {
                black_box( wh.get_voxel( black_box( 50 ), black_box( 50 ), black_box( i ) ) );
            }
        },
        BatchSize::SmallInput,
    ) );

    group.bench_function( "Octree (size 100)", |b| b.iter_batched(
        || {
            let mut wh = Octree::<Voxel>::from_max_size( 100 );
            wh.fill_voxels( (25, 25, 25), (74, 74, 74), black_box( Some( voxel.clone() ) ));
            wh
        },
        |wh| {
            for i in 0..100 {
                black_box( wh.get_voxel( black_box( 50 ), black_box( 50 ), black_box( i ) ) );
            }
        },
        BatchSize::SmallInput,
    ) );

    group.bench_function( "VoxelList", |b| b.iter_batched(
        || {
            let mut wh = VoxelList::<Voxel>::new();
            wh.fill_voxels( (25, 25, 25), (74, 74, 74), black_box( Some( voxel.clone() ) ));
            wh
        },
        |wh| black_box( wh.get_voxel( black_box( 55 ), black_box( 55 ), black_box( 55 ) ) ),
        BatchSize::SmallInput,
    ) );

    group.bench_function( "VoxelHashMap", |b| b.iter_batched(
        || {
            let mut wh = VoxelHashMap::<Voxel>::new();
            wh.fill_voxels( (25, 25, 25), (74, 74, 74), black_box( Some( voxel.clone() ) ));
            wh
        },
        |wh| black_box( wh.get_voxel( black_box( 55 ), black_box( 55 ), black_box( 55 ) ) ),
        BatchSize::SmallInput,
    ) );
}

pub fn measure_structs_get_random( c:&mut Criterion ) {
    let size_cube = SIZE * SIZE * SIZE;
    let randoms = generate_unique( 0, 1000 ).iter().map( |i| {
        let i = i % size_cube;

        (
            i % SIZE,
            (i / SIZE) % SIZE,
            i / (SIZE * SIZE),
        )
    } ).collect::<Vec<(u32, u32, u32)>>();

    let mut group = c.benchmark_group( "Random get from the structs" );
    let voxel= create_voxel(
        &mut VoxelDataset::new(),
        (String::from( "material" ), Material { _density:100 }),
        (String::from( "color" ), Color { red:250, green:10, blue:10 }),
    );

    group.bench_function( format!( "VoxelMap (size {SIZE})" ), |b| b.iter_batched(
        || {
            let mut wh = VoxelMap::<Voxel>::from_max_size( SIZE );
            wh.fill_voxels( (25, 25, 25), (74, 74, 74), black_box( Some( voxel.clone() ) ));
            wh
        },
        |wh| {
            for (x, y, z) in &randoms {
                black_box( wh.get_voxel( black_box( *x ), black_box( *y ), black_box( *z ) ) );
            }
        },
        BatchSize::SmallInput,
    ) );

    group.bench_function( format!( "Octree (size {SIZE})" ), |b| b.iter_batched(
        || {
            let mut wh = Octree::<Voxel>::from_max_size( SIZE );
            wh.fill_voxels( (5, 5, 5), (94, 94, 94), black_box( Some( voxel.clone() ) ));
            wh
        },
        |wh| {
            for (x, y, z) in &randoms {
                black_box( wh.get_voxel( black_box( *x ), black_box( *y ), black_box( *z ) ) );
            }
        },
        BatchSize::SmallInput,
    ) );

    group.bench_function( "VoxelList", |b| b.iter_batched(
        || {
            let mut wh = VoxelList::<Voxel>::new();
            wh.fill_voxels( (5, 5, 5), (94, 94, 94), black_box( Some( voxel.clone() ) ));
            wh
        },
        |wh| {
            for (x, y, z) in &randoms {
                black_box( wh.get_voxel( black_box( *x ), black_box( *y ), black_box( *z ) ) );
            }
        },
        BatchSize::SmallInput,
    ) );

    group.bench_function( "VoxelHashMap", |b| b.iter_batched(
        || {
            let mut wh = VoxelHashMap::<Voxel>::new();
            wh.fill_voxels( (5, 5, 5), (94, 94, 94), black_box( Some( voxel.clone() ) ));
            wh
        },
        |wh| {
            for (x, y, z) in &randoms {
                black_box( wh.get_voxel( black_box( *x ), black_box( *y ), black_box( *z ) ) );
            }
        },
        BatchSize::SmallInput,
    ) );
}
