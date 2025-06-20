use std::collections::HashMap;
use std::hash::{ Hash, Hasher };
use std::mem::{ size_of, size_of_val };
use std::rc::Rc;

use crate::world::world_holder::{Voxel, VoxelSide, WorldHolding};


#[derive( Debug, PartialEq, Eq )]
struct Position {
    x: u32,
    y: u32,
    z: u32,
}

impl Hash for Position {
    fn hash<H:Hasher>( &self, state:&mut H ) {
        state.write_u32( self.x );
        state.write_u32( self.y );
        state.write_u32( self.z );
    }
}


#[derive(Debug)]
pub struct VoxelHashMap<T> {
    voxels: HashMap<Position, Rc<T>>,
}

#[allow(dead_code)]
impl<T> VoxelHashMap<T> {
    pub fn new() -> Self {
        Self {
            voxels: HashMap::new(),
        }
    }

    pub fn new_with_capacity( capacity:usize ) -> Self {
        Self {
            voxels: HashMap::with_capacity( capacity ),
        }
    }
}

impl WorldHolding for VoxelHashMap<Voxel> {
    fn get_voxel(&self, x:u32, y:u32, z:u32) -> Option<Rc<Voxel>> {
        let pos = Position { x, y, z };

        if let Some( data ) = self.voxels.get( &pos ) {
            Some( data.clone() )
        } else {
            None
        }
    }

    fn get_all_voxels( &self ) -> Vec<(u32, u32, u32, Rc<Voxel>)> {
        todo!()
    }

    fn get_all_visible_voxels_from( &self, _from:(u32, u32, u32) ) -> Vec<VoxelSide> {
        todo!()
    }

    fn set_voxel( &mut self, x:u32, y:u32, z:u32, voxel:Option<Rc<Voxel>> ) {
        let pos = Position { x, y, z };

        if let Some( voxel ) = voxel {
            self.voxels.insert( pos, voxel );
        } else {
            self.voxels.remove( &pos );
        }
    }

    fn fill_voxels( &mut self, from:(u32, u32, u32), to:(u32, u32, u32), voxel:Option<Rc<Voxel>> ) {
        let (from_x, from_y, from_z) = from;
        let (to_x, to_y, to_z) = to;

        let (min_x, max_x) = (from_x.min( to_x ), from_x.max( to_x ));
        let (min_y, max_y) = (from_y.min( to_y ), from_y.max( to_y ));
        let (min_z, max_z) = (from_z.min( to_z ), from_z.max( to_z ));

        if let Some( voxel ) = voxel {
            let total_voxels = ((max_x - min_x + 1) * (max_y - min_y + 1) * (max_z - min_z + 1)) as usize;
            self.voxels.reserve( total_voxels );

            let new_voxels = (min_x..=max_x).flat_map(
                |x| (min_y..=max_y).flat_map({
                    let voxel = voxel.clone();

                    move |y| (min_z..=max_z).map({
                        let voxel = voxel.clone();
                        move |z| (Position { x, y, z }, voxel.clone())
                    })
                })
            );

            self.voxels.extend( new_voxels );
        } else {
            self.voxels.retain( |k, _| !(
                k.x >= min_x && k.x <= max_x &&
                k.y >= min_y && k.y <= max_y &&
                k.z >= min_z && k.z <= max_z
            ) );
        }
    }

    fn get_size( &self ) {
        let its_size = size_of::<Self>();
        println!( " - its size = {}", its_size );

        let voxel_size = size_of::<Rc<Voxel>>();
        println!( " - voxel size = {}", voxel_size );

        let stored_voxels = self.voxels.iter().len();
        println!( " - stored voxels count = {}", stored_voxels );

        let voxels_hashmap_size = size_of_val( &self.voxels );
        println!( " - voxels hashmap size = {}", voxels_hashmap_size );

        let full_size = its_size + stored_voxels * voxel_size;
        println!(
            " - full size = {} [its size] + {} [voxels] * {} [voxel size] = {}",
            its_size, stored_voxels, voxel_size, self.get_bytes_with_prefixes( full_size ),
        );
    }
}
