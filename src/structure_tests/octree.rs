use std::rc::Rc;
use crate::world::world_holder::{Voxel, WorldHolder};

#[derive(Debug)]
pub enum OctreeNode<T> {
    Leaf( Option<Rc<T>> ),
    Branch( Box<OctreeBranch<T>> ),
}

impl<T> OctreeNode<T> {
    fn insert(&mut self, reversed_depth:u8, x:u32, y:u32, z:u32, value:Rc<T> ) {
        match self {
            OctreeNode::Leaf( leaf ) => {
                if reversed_depth == 0 {
                    *self = OctreeNode::Leaf( Some( value ) );
                    return;
                }

                let mut branch = OctreeBranch::new_filled_by( leaf.clone() );
                branch.insert( reversed_depth, x, y, z, value );
                *self = OctreeNode::Branch( Box::new( branch ) );
            }

            OctreeNode::Branch( branch ) => {
                branch.insert( reversed_depth, x, y, z, value );
                self.try_compress();
            }
        }
    }

    fn fill_at(
        &mut self,
        depth: u8,
        origin: (u32, u32, u32),
        size: u32,
        fill_from: (u32, u32, u32),
        fill_to: (u32, u32, u32),
        value: Option<Rc<T>>,
    ) {
        let (from_x, from_y, from_z) = fill_from;
        let (to_x, to_y, to_z) = fill_to;

        let (origin_x, origin_y, origin_z) = origin;
        let max_x = origin_x + size - 1;
        let max_y = origin_y + size - 1;
        let max_z = origin_z + size - 1;

        // No crossing; branch outside filling area
        if max_x < from_x || origin_x > to_x ||
           max_y < from_y || origin_y > to_y ||
           max_z < from_z || origin_z > to_z {
            return;
        }

        // Full contained branch inside filling area
        if origin_x >= from_x && max_x <= to_x &&
           origin_y >= from_y && max_y <= to_y &&
           origin_z >= from_z && max_z <= to_z {
            *self = OctreeNode::Leaf( value );
            return;
        }

        // Deepest node reached
        if depth == 0 {
            *self = OctreeNode::Leaf( value );
            return;
        }

        // Ensure "branch" on the node
        if let OctreeNode::Leaf( existing ) = self {
            let filled_value = existing.clone();
            let new_branch = OctreeBranch::new_filled_by( filled_value );
            *self = OctreeNode::Branch( Box::new( new_branch ) );
        }

        // Pass filling into the branch
        if let OctreeNode::Branch( branch ) = self {
            let child_size = size / 2;

            for child_index in 0..8 {
                let cx = origin_x + if (child_index >> 2) & 1 == 1 { child_size } else { 0 };
                let cy = origin_y + if (child_index >> 1) & 1 == 1 { child_size } else { 0 };
                let cz = origin_z + if child_index & 1 == 1 { child_size } else { 0 };

                branch.children[ child_index ].fill_at( depth - 1, (cx, cy, cz), child_size, fill_from, fill_to, value.clone() );
            }

            self.try_compress();
        }
    }

    fn get( &self, reversed_depth:u8, x:u32, y:u32, z:u32 ) -> Option<Rc<T>> {
        match self {
            OctreeNode::Leaf( value ) => value.clone(),
            OctreeNode::Branch( branch ) => {
                let child_index = OctreeBranch::<T>::get_child_index( reversed_depth, x, y, z );
                branch.children[ child_index ].get( reversed_depth - 1, x, y, z )
            }
        }
    }

    // fn get_reachable( &self, offset:(u32, u32, u32), reversed_depth:u8, searchable_points:&mut VecDeque<(u32, u32, u32)> ) {
    //     match self {
    //         OctreeNode::Leaf( None ) => {
    //             let cells_per_side = 1 << reversed_depth;

    //             for x in 0..cells_per_side {
    //                 for y in 0..cells_per_side {
    //                     searchable_points.insert( (offset.0 + x, offset.1 + y, offset.2 - 1) );
    //                 }
    //             }

    //             for x in 0..cells_per_side {
    //                 for y in 0..cells_per_side {
    //                     searchable_points.insert( (offset.0 + x, offset.1 + y, offset.2 + 1) );
    //                 }
    //             }

    //             for y in 0..cells_per_side {
    //                 for z in 0..cells_per_side {
    //                     searchable_points.insert( (offset.0 - 1, offset.1 + y, offset.2 + z) );
    //                 }
    //             }

    //             for y in 0..cells_per_side {
    //                 for z in 0..cells_per_side {
    //                     searchable_points.insert( (offset.0 + 1, offset.1 + y, offset.2 + z) );
    //                 }
    //             }

    //             for x in 0..cells_per_side {
    //                 for z in 0..cells_per_side {
    //                     searchable_points.insert( (offset.0 + x, offset.1 - 1, offset.2 + z) );
    //                 }
    //             }

    //             for x in 0..cells_per_side {
    //                 for z in 0..cells_per_side {
    //                     searchable_points.insert( (offset.0 + x, offset.1 + 1, offset.2 + z) );
    //                 }
    //             }
    //         },
    //         OctreeNode::Leaf( Some( _ ) ) => {},
    //         OctreeNode::Branch( branch ) => {
    //             let size = 1 << reversed_depth;

    //             for point in searchable_points.clone() {
    //                 if offset.0 > point.0 || point.0 > offset.0 + size { continue }
    //                 if offset.1 > point.1 || point.1 > offset.1 + size { continue }
    //                 if offset.2 > point.2 || point.2 > offset.2 + size { continue }

    //                 let child_index = OctreeBranch::<T>::get_child_index( reversed_depth, point.0, point.1, point.2 );
    //                 let child_offset = OctreeNode::<T>::get_child_offset( offset, child_index as u8, 1 << reversed_depth );

    //                 branch.children[ child_index ].get_reachable( child_offset, reversed_depth - 1, searchable_points );

    //                 if let OctreeNode::Leaf( _ ) = branch.children[ child_index ] {
    //                     searchable_points.remove( &point );
    //                 }
    //             }
    //         }
    //     }
    // }


    fn collect_voxels( &self, offset:(u32, u32, u32), depth:u8, out:&mut Vec<(u32, u32, u32, Rc<T>)> ) {
        match self {
            OctreeNode::Leaf(Some(voxel)) => {
                let size = 1 << depth;
                for x in 0..size {
                    for y in 0..size {
                        for z in 0..size {
                            out.push((
                                offset.0 + x,
                                offset.1 + y,
                                offset.2 + z,
                                Rc::clone(voxel),
                            ));
                        }
                    }
                }
            }
            OctreeNode::Leaf(None) => {
                // Pusty liść - nie dodajemy nic
            }
            OctreeNode::Branch(branch) => {
                let child_size = 1 << (depth - 1);
                for (i, child) in branch.children.iter().enumerate() {
                    let dx = ((i >> 2) & 1) as u32 * child_size;
                    let dy = ((i >> 1) & 1) as u32 * child_size;
                    let dz = (i & 1) as u32 * child_size;
                    child.collect_voxels( (offset.0 + dx, offset.1 + dy, offset.2 + dz), depth - 1, out );
                }
            }
        }
    }

    fn get_child_offset( parent_offset:(u32, u32, u32), child_index:u8, child_size:u32 ) -> (u32, u32, u32) {
        let dx = ((child_index >> 2) & 1) as u32 * child_size;
        let dy = ((child_index >> 1) & 1) as u32 * child_size;
        let dz = (child_index & 1) as u32 * child_size;

        (parent_offset.0 + dx, parent_offset.1 + dy, parent_offset.2 + dz)
    }

    fn remove( &mut self, depth:u8, x:u32, y:u32, z:u32 ) -> Option<Rc<T>> {
        match self {
            OctreeNode::Leaf( value ) => {
                value.take()
            }

            OctreeNode::Branch(branch) => {
                let child_index = OctreeBranch::<T>::get_child_index( depth, x, y, z );
                let result = branch.children[child_index].remove( depth - 1, x, y, z );

                branch.children[child_index].try_compress();
                self.try_compress();

                result
            }
        }
    }

    fn count_leaves(&self) -> usize {
        match self {
            OctreeNode::Leaf(_) => 1,
            OctreeNode::Branch( branch ) => branch.children.iter().map( |c| c.count_leaves() ).sum(),
        }
    }

    fn try_compress(&mut self) {
        let branch = match self {
            OctreeNode::Branch( branch ) => branch,
            _ => return
        };

        let first_value = match branch.children.iter_mut().next().unwrap() {
            OctreeNode::Leaf( v ) => v.clone(),
            _ => return
        };

        let are_all_the_same = match first_value.clone() {
            Some( value ) => branch.children.iter().skip( 1 ).all( |child| match child {
                OctreeNode::Leaf( val_opt ) => match val_opt {
                    Some( val ) => Rc::ptr_eq( val, &value ),
                    _ => false
                }
                _ => false,
            } ),

            None => branch.children.iter().skip( 1 ).all( |child| match child {
                OctreeNode::Leaf( val_opt ) => match val_opt {
                    None => true,
                    _ => false,
                }
                _ => false,
            } )
        };

        if !are_all_the_same {
            return
        }

        *self = OctreeNode::Leaf( first_value );
    }
}

#[derive(Debug)]
pub struct OctreeBranch<T> {
    children: [OctreeNode<T>; 8],
}

impl<T> OctreeBranch<T> {
    fn new_filled_by( value:Option<Rc<T>> ) -> Self {
        Self {
            children: std::array::from_fn( |_| OctreeNode::Leaf( value.clone() ) ),
        }
    }

    fn insert( &mut self, depth:u8, x:u32, y:u32, z:u32, value:Rc<T> ) {
        let child_index = Self::get_child_index( depth, x, y, z );

        if depth == 1 {
            self.children[ child_index ] = OctreeNode::Leaf( Some( value ) );
            return
        }

        self.children[ child_index ].insert( depth - 1, x, y, z, value );
        self.children[ child_index ].try_compress();
    }

    fn get_child_index( reversed_depth:u8, x:u32, y:u32, z:u32 ) -> usize {
        let shift = reversed_depth - 1;
        let xi = ((x >> shift) & 1) as usize;
        let yi = ((y >> shift) & 1) as usize;
        let zi = ((z >> shift) & 1) as usize;

        (xi << 2) | (yi << 1) | zi
    }
}

pub struct Octree<T> {
    root: OctreeNode<T>,
    max_depth: u8,
}

impl<T> Octree<T> {
    #[allow(dead_code)]
    pub fn new( max_depth:u8 ) -> Self {
        Self {
            max_depth,
            root: OctreeNode::Leaf( None )
        }
    }

    pub fn insert( &mut self, x:u32, y:u32, z:u32, value:Rc<T> ) {
        self.root.insert( self.max_depth, x, y, z, value )
    }

    pub fn get( &self, x: u32, y: u32, z: u32) -> Option<Rc<T>> {
        self.root.get( self.max_depth, x, y, z )
    }

    pub fn get_voxels(&self) -> Vec<(u32, u32, u32, Rc<T>)> {
        let mut result = Vec::new();
        self.root.collect_voxels( (0, 0, 0), self.max_depth, &mut result );
        result
    }

    pub fn remove( &mut self, x:u32, y:u32, z:u32 ) -> Option<Rc<T>>{
        self.root.remove( self.max_depth, x, y, z )
    }

    pub fn count_leaves(&self) -> usize {
        self.root.count_leaves()
    }
}

impl WorldHolder for Octree<Voxel> {
    fn get_voxel( &self, x:u32, y:u32, z:u32 ) -> Option<Rc<Voxel>> {
        self.get( x, y, z )
    }

    fn get_all_voxels( &self ) -> Vec<(u32, u32, u32, Rc<Voxel>)> {
        self.get_voxels()
    }

    fn get_all_visible_voxels( &self ) -> Vec<(u32, u32, u32, Rc<Voxel>)> {
        todo!()
    }

    fn set_voxel( &mut self, x:u32, y:u32, z:u32, voxel:Option<Rc<Voxel>> ) {
        if let Some( voxel ) = voxel {
            self.insert( x, y, z, voxel );
        } else {
            self.remove( x, y, z );
        }
    }

    fn fill_voxels( &mut self, from:(u32, u32, u32), to:(u32, u32, u32), voxel:Option<Rc<Voxel>> ) {
        let size = 1u32 << self.max_depth;
        self.root.fill_at( self.max_depth, (0, 0, 0), size, from, to, voxel );
    }

    fn get_size( &self ) {
        println!( "Leaves count = {}", self.count_leaves() )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::rc::Rc;

    #[derive(Debug, PartialEq)]
    struct TestVoxel(i32);

    #[test]
    fn test_insert_and_get() {
        let mut octree = Octree::new( 4 );
        let voxel = Rc::new( TestVoxel( 42 ) );
        octree.insert( 3, 2, 1, voxel.clone() );

        assert_eq!( octree.get( 3, 2, 1 ), Some( voxel ) );
    }

    #[test]
    fn test_compression() {
        let mut octree = Octree::new( 2 );
        let voxel = Rc::new( TestVoxel( 7 ) );

        for x in 0..(1 << 2) {
            for y in 0..(1 << 2) {
                for z in 0..(1 << 2) {
                    octree.insert( x, y, z, voxel.clone() );
                }
            }
        }

        assert_eq!( octree.get( 0, 0, 0 ), Some( voxel ) );
    }

    #[test]
    fn test_insert_get_remove() {
        let mut octree = Octree::new( 3 );
        let voxel = Rc::new( TestVoxel( 42 ) );

        octree.insert( 3, 3, 3, voxel.clone() );

        assert_eq!( octree.get( 3, 3, 3 ), Some( voxel.clone() ) );

        octree.remove( 3, 3, 3 );

        assert_eq!( octree.get( 3, 3, 3 ), None );
    }

    #[test]
    fn test_count_leaves_after_inserts() {
        let mut octree = Octree::new( 2 );
        assert_eq!( octree.count_leaves(), 1 );

        let voxel1 = Rc::new( TestVoxel( 1 ) );
        let voxel2 = Rc::new( TestVoxel( 2 ) );

        octree.insert( 0, 0, 0, voxel1.clone() );
        octree.insert( 3, 3, 3, voxel2.clone() );

        let leaves = octree.count_leaves();
        assert!( leaves == 22, "Tree should be branched, leaves count is {}", leaves );

        octree.insert( 3, 3, 2, voxel2.clone() );
        let leaves = octree.count_leaves();
        assert!( leaves == 22, "[1] Tree should be branched into 22, leaves count is {}", leaves );

        octree.insert( 0, 0, 1, voxel2.clone() );
        octree.insert( 0, 1, 1, voxel2.clone() );
        octree.insert( 1, 1, 1, voxel2.clone() );
        let leaves = octree.count_leaves();
        assert!( leaves == 22, "[2] Tree should be branched into 22, leaves count is {}", leaves );

        octree.insert( 0, 0, 3, voxel2.clone() );
        let leaves = octree.count_leaves();
        assert!( leaves == 29, "[3] Tree should be branched into 29 leaves, leaves count is {}", leaves );

        for x in 0..(1 << 2) {
            for y in 0..(1 << 2) {
                for z in 0..(1 << 2) {
                    octree.insert(x, y, z, voxel1.clone());
                }
            }
        }

        assert_eq!( octree.count_leaves(), 1 );
    }
}
