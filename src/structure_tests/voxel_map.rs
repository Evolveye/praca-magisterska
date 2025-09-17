use std::{ mem::size_of, sync::Arc };

use crate::world::{ world_chunk::ChunkBitmask, world_holder::{ Voxel, VoxelSide, WorldHolding } };

use super::tester::*;

pub struct VoxelMap<T> {
    pub data: Vec<Option<Arc<T>>>,
    pub size_x: usize,
    pub size_y: usize,
    #[allow(dead_code)] pub size_z: usize,
    pub filled_cells: u32,
}

impl<T> VoxelMap<T> {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self::from_max_sizes( WORLD_X, WORLD_Y, WORLD_Z )
    }

    #[allow(dead_code)]
    pub fn from_max_size( size:u32 ) -> Self {
        Self::from_max_sizes( size, size, size )
    }

    #[allow(dead_code)]
    pub fn from_max_sizes( size_x:u32, size_y:u32, size_z:u32 ) -> Self {
        let total = size_x * size_y * size_z;

        Self {
            data: vec![None; total as usize],
            size_x: size_x as usize,
            size_y: size_y as usize,
            size_z: size_z as usize,
            filled_cells: 0,
        }
    }

    #[inline(always)]
    fn index(&self, x: usize, y: usize, z: usize) -> usize {
        z * self.size_y * self.size_x + y * self.size_x + x
    }

    fn set_data( &mut self, x:usize, y:usize, z:usize, voxel:Option<Arc<T>> ) {
        let index = self.index( x, y, z );

        if self.data[ index ].is_none() {
            if !voxel.is_none() {
                self.filled_cells += 1;
            }
        } else {
            if voxel.is_none() {
                self.filled_cells -= 1;
            }
        }

        self.data[ index ] = voxel;
    }
}

impl WorldHolding for VoxelMap<Voxel> {
    fn get_voxel( &self, x:u32, y:u32, z:u32 ) -> Option<Arc<Voxel>> {
        self.data[ self.index( x as usize, y as usize, z as usize ) ].clone()
    }

    fn get_all_voxels( &self ) -> Vec<(u32, u32, u32, Arc<Voxel>)> {
        todo!()
    }

    fn get_all_visible_voxels_from( &self, _from:(u32, u32, u32) ) -> Vec<VoxelSide> {
        todo!()
    }

    fn to_bitmask( &self ) -> ChunkBitmask {
        todo!()
    }

    fn set_voxel( &mut self, x:u32, y:u32, z:u32, voxel:Option<Arc<Voxel>> ) {
        self.set_data( x as usize, y as usize, z as usize, voxel )
    }

    fn fill_voxels( &mut self, from:(u32, u32, u32), to:(u32, u32, u32), voxel:Option<Arc<Voxel>> ) {
        let (x_min, x_max) = (from.0.min(to.0) as usize, from.0.max(to.0) as usize);
        let (y_min, y_max) = (from.1.min(to.1) as usize, from.1.max(to.1) as usize);
        let (z_min, z_max) = (from.2.min(to.2) as usize, from.2.max(to.2) as usize);

        for z in z_min..=z_max {
            for y in y_min..=y_max {
                for x in x_min..=x_max {
                    self.set_data( x, y, z, voxel.clone() );
                }
            }
        }
    }

    fn get_size( &self ) {
        println!( "VoxelMap sizes (in bytes by default):" );

        let its_size = size_of::<Self>();
        println!( " - its size = {}", its_size );

        let root_vec_size = size_of::<Vec<Vec<Vec<Option<Arc<Voxel>>>>>>();
        println!( " - root vector size = {}", root_vec_size );

        let depth_vec_size = size_of::<Vec<Vec<Option<Arc<Voxel>>>>>();
        println!( " - depth vectors size = {} * {}", self.size_z, depth_vec_size );

        let row_vec_size = size_of::<Vec<Option<Arc<Voxel>>>>();
        println!( " - row vectors size = {} * {} * {}", self.size_z, self.size_y, row_vec_size );

        let cell_size = size_of::<Option<Arc<Voxel>>>();
        println!( " - column vectors size = {} * {} * {} * {}", self.size_z, self.size_y, self.size_x, cell_size, );

        let depths_size = self.size_z * depth_vec_size;
        let rows_size = self.size_z * self.size_y * depth_vec_size;
        let cells_size = self.size_z * self.size_y * self.size_x * cell_size;
        let full_size = its_size + root_vec_size + depths_size + rows_size + cells_size;
        println!(
            " - full size = {} [its size] + {} [root] + {} [depths] + {} [rows] + {} [columns] = {}",
            its_size, root_vec_size, depths_size, rows_size, cells_size, self.get_bytes_with_prefixes( full_size )
        );
    }
}
