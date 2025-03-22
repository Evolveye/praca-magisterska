use std::{mem::size_of, rc::Rc};

use super::tester::*;

pub struct VoxelMap {
    pub data: Vec<Vec<Vec<Option<Rc<Voxel>>>>>,
    pub filled_cells: u32,
}

impl VoxelMap {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            data: vec![ vec![ vec![ None; WORLD_Z as usize ]; WORLD_Y as usize ]; WORLD_X as usize ],
            filled_cells: 0,
        }
    }
}

impl WorldHolder for VoxelMap {
    fn get_voxel( &self, x:u32, y:u32, z:u32 ) -> Option<Rc<Voxel>> {
        self.data[ z as usize ][ y as usize ][ x as usize ].clone()
    }

    fn set_voxel( &mut self, x:u32, y:u32, z:u32, voxel:Option<Rc<Voxel>> ) {
        if self.data[z as usize][y as usize][x as usize].is_none() {
            if !voxel.is_none() {
                self.filled_cells += 1;
            }
        } else {
            if voxel.is_none() {
                self.filled_cells -= 1;
            }
        }

        self.data[ z as usize ][ y as usize ][ x as usize ] = voxel;
    }


    fn fill_voxels( &mut self, from:(u32, u32, u32), to:(u32, u32, u32), voxel:Option<Rc<Voxel>> ) {
        let (x_min, x_max) = (from.0.min(to.0), from.0.max(to.0));
        let (y_min, y_max) = (from.1.min(to.1), from.1.max(to.1));
        let (z_min, z_max) = (from.2.min(to.2), from.2.max(to.2));

        for z in z_min..=z_max {
            for y in y_min..=y_max {
                for x in x_min..=x_max {
                    self.set_voxel( x, y, z, voxel.clone() );
                }
            }
        }
    }

    fn get_size( &self ) {
        println!( "VoxelMap sizes (in bytes by default):" );

        let its_size = size_of::<Self>();
        println!( " - its size = {}", its_size );

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
        let full_size = its_size + root_vec_size + depths_size + rows_size + cells_size;
        println!(
            " - full size = {} [its size] + {} [root] + {} [depths] + {} [rows] + {} [columns] = {}",
            its_size, root_vec_size, depths_size, rows_size, cells_size, self.get_bytes_with_prefixes( full_size )
        );
    }
}
