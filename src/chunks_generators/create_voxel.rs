use std::sync::Arc;

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
