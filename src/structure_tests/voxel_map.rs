use std::{mem::size_of, rc::Rc};

use super::tester::*;

pub struct VoxelMap {
    pub data: Vec<Vec<Vec<Option<Rc<Voxel>>>>>,
    pub set_count: u32,
}

impl VoxelMap {
    pub fn new() -> Self {
        Self {
            data: vec![ vec![ vec![ None; WORLD_Z as usize ]; WORLD_Y as usize ]; WORLD_X as usize ],
            set_count: 0,
        }
    }
}

impl WorldHolder for VoxelMap {
    fn get_voxel( &self, x:u32, y:u32, z:u32 ) -> Option<Rc<Voxel>> {
        self.data[ z as usize ][ y as usize ][ x as usize ].clone()
    }

    fn set_voxel( &mut self, x:u32, y:u32, z:u32, voxel_ref:Rc<Voxel> ) {
        self.set_count += 1;
        self.data[ z as usize ][ y as usize ][ x as usize ] = Some( voxel_ref );
    }

    fn get_size( &self ) {
        println!( "VoxelMap sizes (in bytes by default):" );

        let its_size = size_of::<Self>();
        println!( " - its size = {}", its_size );

        let rc_size = size_of::<Rc<Voxel>>();
        println!( " - Reference Counting size = {}", rc_size );

        let root_vec_size = size_of::<Vec<Vec<Vec<Option<Rc<Voxel>>>>>>();
        println!( " - root vector size = {}", root_vec_size );

        let depth_vec_size = size_of::<Vec<Vec<Option<Rc<Voxel>>>>>();
        println!( " - depth vectors size = {} * {}", self.data.len(), depth_vec_size );

        let row_vec_size = size_of::<Vec<Option<Rc<Voxel>>>>();
        println!( " - row vectors size = {} * {} * {}", self.data.len(), self.data[ 0 ].len(), row_vec_size );

        let cell_size = size_of::<Option<Rc<Voxel>>>();
        println!( " - column vectors size = {} * {} * {} * {}", self.data.len(), self.data[ 0 ].len(), self.data[ 0 ][ 0 ].len(), cell_size, );

        let depths_size = self.data.len() * depth_vec_size;
        let rows_size = self.data.len() * self.data[ 0 ].len() * depth_vec_size;
        let cells_size = self.data.len() * self.data[ 0 ].len() * self.data[ 0 ][ 0 ].len() * cell_size;
        let full_size = root_vec_size + depths_size + rows_size + cells_size;
        println!( " - full size = root + depths + rows + columns" );
        println!( " - full size = {} + {} + {} + {} = {} = {} KiB = {} MiB = {} GiB",
            root_vec_size, depths_size, rows_size, cells_size,
            full_size, full_size / 1024, full_size / 1024 / 1024, full_size / 1024 / 1024 / 1024,
        );
    }
}
