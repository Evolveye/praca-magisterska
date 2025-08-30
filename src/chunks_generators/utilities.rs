use std::{collections::HashSet, sync::Arc};

use rand::{ rngs::StdRng, Rng, SeedableRng };

use crate::world::world_holder::{Color, CommonVoxelData, Material, Voxel, VoxelDataset};

pub fn create_voxel( dataset:&mut VoxelDataset, material:(String, Material), color:(String, Color) ) -> Arc<Voxel> {
    let composed_key = format!( "c:{}|m:{}", color.0, material.0 );

    if let Some( voxel ) = dataset.voxels.get( &composed_key ) {
        return Arc::clone( voxel );
    }

    dataset.materials.entry( material.0.clone() ).or_insert_with( || Arc::new( material.1 ) );
    dataset.colors.entry( color.0.clone() ).or_insert_with( || Arc::new( color.1 ));

    dataset.common_voxel_dataset.insert( composed_key.clone(), Arc::new( CommonVoxelData {
        material: Arc::clone( dataset.materials.get( &material.0 ).unwrap() ),
        color: Arc::clone( dataset.colors.get( &color.0 ).unwrap() ),
    } ) );

    dataset.voxels.insert( composed_key.clone(), Arc::new( Voxel {
        _individual_data: vec![],
        _common_data: Arc::clone( dataset.common_voxel_dataset.get( &composed_key ).unwrap() ),
    } ) );

    Arc::clone( dataset.voxels.get( &composed_key ).unwrap() )
}

pub fn generate_unique( seed:u64, n:usize ) -> Vec<u32> {
    let mut rng = StdRng::seed_from_u64( seed );
    let mut set = HashSet::with_capacity( n );

    while set.len() < n {
        let val:u32 = rng.random();
        set.insert( val );
    }

    set.into_iter().collect()
}

fn hsv_to_rgb( h:f32, s:f32, v:f32 ) -> (u8, u8, u8) {
    let c = v * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = v - c;

    let (r1, g1, b1) = if h < 60.0 {
        (c, x, 0.0)
    } else if h < 120.0 {
        (x, c, 0.0)
    } else if h < 180.0 {
        (0.0, c, x)
    } else if h < 240.0 {
        (0.0, x, c)
    } else if h < 300.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };

    (
        ((r1 + m) * 255.0) as u8,
        ((g1 + m) * 255.0) as u8,
        ((b1 + m) * 255.0) as u8,
    )
}

pub fn hash_u32( mut x:u32 ) -> u32 {
    x ^= x >> 16;
    x = x.wrapping_mul(0x7feb_352d);
    x ^= x >> 15;
    x = x.wrapping_mul(0x846c_a68b);
    x ^= x >> 16;
    x
}

pub fn get_pastel_color( index:u32, max_colors:u32 ) -> (u8, u8, u8) {
    let index = index as f32;
    let max_colors = max_colors as f32;
    let hue = (index % max_colors * (360.0 / max_colors)) % 360.0;

    let saturation = 0.8;
    let value = 0.75;

    hsv_to_rgb( hue, saturation, value )
}