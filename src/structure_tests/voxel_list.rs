use std::{mem::size_of, rc::Rc};

use crate::world::world_holder::{Voxel, VoxelSide, WorldHolder};

pub struct VoxelInWorld {
    x: u32,
    y: u32,
    z: u32,
    voxel: Rc<Voxel>
}

pub struct VoxelList {
    pub data: Vec<VoxelInWorld>,
}

impl VoxelList {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            data: vec![],
        }
    }
}

impl WorldHolder for VoxelList {
    fn get_voxel( &self, x:u32, y:u32, z:u32 ) -> Option<std::rc::Rc<Voxel>> {
        match self.data.iter().find( |v| v.x == x && v.y == y && v.z == z ) {
            Some( voxel_in_world ) => Some( voxel_in_world.voxel.clone() ),
            None => None
        }
    }

    fn get_all_voxels( &self ) -> Vec<(u32, u32, u32, Rc<Voxel>)> {
        todo!()
    }

    fn get_all_visible_voxels_from( &self, _from:(u32, u32, u32) ) -> Vec<VoxelSide> {
        todo!()
    }

    fn set_voxel( &mut self, x:u32, y:u32, z:u32, voxel:Option<Rc<Voxel>> ) {
        if let Some( voxel ) = voxel {
            match self.data.iter_mut().find( |v| v.x == x && v.y == y && v.z == z ) {
                Some( voxel_in_world ) => voxel_in_world.voxel = voxel,
                None => self.data.push( VoxelInWorld { x, y, z, voxel } ),
            }
        } else {
            if let Some( index ) = self.data.iter().position( |v| v.x == x && v.y == y && v.z == z ) {
                self.data.remove( index );
            }
        }
    }

    fn fill_voxels( &mut self, from:(u32, u32, u32), to:(u32, u32, u32), voxel:Option<Rc<Voxel>> ) {
        let (x_min, x_max) = (from.0.min( to.0 ), from.0.max( to.0 ));
        let (y_min, y_max) = (from.1.min( to.1 ), from.1.max( to.1 ));
        let (z_min, z_max) = (from.2.min( to.2 ), from.2.max( to.2 ));

        self.data.retain( |v| !(
            v.x >= x_min && v.x <= x_max &&
            v.y >= y_min && v.y <= y_max &&
            v.z >= z_min && v.z <= z_max
        ) );

        if let Some( voxel ) = voxel {
            self.data.reserve(
                ((x_max - x_min + 1) as usize) *
                ((y_max - y_min + 1) as usize) *
                ((z_max - z_min + 1) as usize)
            );

            for x in x_min..=x_max {
                for y in y_min..=y_max {
                    for z in z_min..=z_max {
                        self.data.push( VoxelInWorld { x, y, z, voxel: voxel.clone() } );
                    }
                }
            }
        }
    }

    fn get_size( &self ) {
        println!( "VoxelList sizes (in bytes by default):" );

        let its_size = size_of::<Self>();
        println!( " - its size = {}", its_size );

        let root_vec_size = size_of::<Vec<VoxelInWorld>>();
        println!( " - root vector size = {}", root_vec_size );

        let voxel_in_world_size = size_of::<VoxelInWorld>();
        println!( " - voxel in world size = {}", voxel_in_world_size );

        let voxel_size = size_of::<Rc<Voxel>>();
        println!( " - voxel size = {}", voxel_size );

        let list_size = self.data.len() * voxel_in_world_size * voxel_size;
        println!( " - list size = {} [data length] * {} [voxel in world size] * {} [voxel size] = {}", self.data.len(), voxel_in_world_size, voxel_size, list_size );

        let full_size = its_size + list_size;
        print!(
            " - full size = {} [its size] + {} [list size] = {}",
            its_size, list_size, self.get_bytes_with_prefixes( full_size )
        );
    }
}
