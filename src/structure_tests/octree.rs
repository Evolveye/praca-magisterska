use std::rc::Rc;

use super::tester::WorldHolder;

#[derive(Debug)]
pub enum OctreeNode<T> {
    Leaf( Option<Rc<T>> ),
    Branch( Box<OctreeBranch<T>> ),
}

impl<T> OctreeNode<T> {
    fn insert(&mut self, depth:u8, x:u32, y:u32, z:u32, value:Rc<T> ) {
        match self {
            OctreeNode::Leaf( leaf ) => {
                if depth == 0 {
                    *self = OctreeNode::Leaf( Some( value ) );
                    return;
                }

                let mut branch = OctreeBranch::new_filled_by( leaf.clone() );
                branch.insert( depth, x, y, z, value );
                *self = OctreeNode::Branch( Box::new( branch ) );
            }

            OctreeNode::Branch( branch ) => {
                branch.insert( depth, x, y, z, value );
                self.try_compress();
            }
        }
    }

    fn get( &self, depth:u8, x:u32, y:u32, z:u32 ) -> Option<Rc<T>> {
        match self {
            OctreeNode::Leaf( value ) => value.clone(),
            OctreeNode::Branch( branch ) => {
                let child_index = OctreeBranch::<T>::get_child_index( depth, x, y, z );
                branch.children[ child_index ].get( depth - 1, x, y, z )
            }
        }
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

    fn get_child_index( depth:u8, x:u32, y:u32, z:u32 ) -> usize {
        let shift = depth - 1;
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

    pub fn remove( &mut self, x:u32, y:u32, z:u32 ) -> Option<Rc<T>>{
        self.root.remove( self.max_depth, x, y, z )
    }

    pub fn count_leaves(&self) -> usize {
        self.root.count_leaves()
    }
}

impl WorldHolder for Octree<super::tester::Voxel> {
    fn get_voxel( &self, x:u32, y:u32, z:u32 ) -> Option<Rc<super::tester::Voxel>> {
        self.get( x, y, z )
    }

    fn set_voxel( &mut self, x:u32, y:u32, z:u32, voxel:Rc<super::tester::Voxel> ) {
        self.insert( x, y, z, voxel );
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
