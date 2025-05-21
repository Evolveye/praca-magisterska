use std::{collections::{HashMap, VecDeque}, ops::Deref, panic, rc::Rc};
use crate::world::world_holder::{Voxel, WorldHolder};

struct Direction;

impl Direction {
    pub const UNSPECIFIED: u8 = 0;
    pub const LEFT:        u8 = 1;
    pub const RIGHT:       u8 = 2;
    pub const TOP:         u8 = 3;
    pub const BOTTOM:      u8 = 4;
    pub const FRONT:       u8 = 5;
    pub const BACK:        u8 = 6;

    pub const OCTREE_NODE_INDICES: [[usize; 4]; 6] = [
        /* opposite to LEFT   */  [1, 3, 5, 7],
        /* opposite to RIGHT  */  [0, 2, 4, 6],
        /* opposite to TOP    */  [0, 1, 4, 5],
        /* opposite to BOTTOM */  [2, 3, 6, 7],
        /* opposite to FRONT  */  [0, 1, 2, 3],
        /* opposite to BACK   */  [4, 5, 6, 7],
    ];

    fn split_branch_point_into_chilren_by_direction( direction:u8, point:(u32, u32, u32), size:u32 ) -> [(u32, u32, u32); 4] {
        match direction {
            1 | 2 => [
                point,
                (point.0,           point.1 + size,     point.2),
                (point.0,           point.1,            point.2 + size),
                (point.0,           point.1 + size,     point.2 + size),
            ],
            3 | 4 => [
                point,
                (point.0 + size,    point.1,            point.2),
                (point.0,           point.1,            point.2 + size),
                (point.0 + size,    point.1,            point.2 + size),
            ],
            5 | 6 => [
                point,
                (point.0 + size,    point.1,            point.2),
                (point.0,           point.1 + size,     point.2),
                (point.0 + size,    point.1 + size,     point.2),
            ],
            _ => unreachable!( "Unknown direction value" )
        }
    }

    fn get_sibling_leaf( direction:u8, offset:(u32, u32, u32), size:u32 ) -> (u32, u32, u32) {
        match direction {
            1 => (offset.0 - 1,     offset.1, offset.2),
            2 => (offset.0 + size,  offset.1, offset.2),
            3 => (offset.0,         offset.1, offset.2),
            4 => (offset.0,         offset.1, offset.2),
            5 => (offset.0,         offset.1, offset.2),
            6 => (offset.0,         offset.1, offset.2),
            _ => unreachable!( "Unknown direction value" )
        }
    }

    fn get_dir_name( direction:u8 ) -> String {
        match direction {
            0 => String::from( "UNSPECIFIED" ),
            1 => String::from( "LEFT" ),
            2 => String::from( "RIGHT" ),
            3 => String::from( "TOP" ),
            4 => String::from( "BOTTOM" ),
            5 => String::from( "FRONT" ),
            6 => String::from( "BACK" ),
            _ => unreachable!( "Unknown direction value" )
        }
    }
}

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
                let child_index = OctreeBranch::<T>::get_child_index( reversed_depth, &(x, y, z) );
                branch.children[ child_index ].get( reversed_depth - 1, x, y, z )
            }
        }
    }

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
                    let dx = (i & 1) as u32 * child_size;
                    let dy = ((i >> 1) & 1) as u32 * child_size;
                    let dz = ((i >> 2) & 1) as u32 * child_size;
                    child.collect_voxels( (offset.0 + dx, offset.1 + dy, offset.2 + dz), depth - 1, out );
                }
            }
        }
    }

    fn remove( &mut self, depth:u8, x:u32, y:u32, z:u32 ) -> Option<Rc<T>> {
        match self {
            OctreeNode::Leaf( value ) => {
                value.take()
            }

            OctreeNode::Branch(branch) => {
                let child_index = OctreeBranch::<T>::get_child_index( depth, &(x, y, z) );
                let result = branch.children[child_index].remove( depth - 1, x, y, z );

                branch.children[child_index].try_compress();
                self.try_compress();

                result
            }
        }
    }

    fn count_leaves( &self ) -> usize {
        match self {
            OctreeNode::Leaf(_) => 1,
            OctreeNode::Branch( branch ) => branch.children.iter().map( |c| c.count_leaves() ).sum(),
        }
    }

    fn try_compress( &mut self ) {
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

    fn get_child_offset( parent_offset:(u32, u32, u32), parent_size:u32, child_index:usize ) -> (u32, u32, u32) {
        let child_size = parent_size >> 1;
        let child_index = child_index as u8;
        let dx = ( child_index       & 1) as u32 * child_size;
        let dy = ((child_index >> 1) & 1) as u32 * child_size;
        let dz = ((child_index >> 2) & 1) as u32 * child_size;

        (parent_offset.0 + dx, parent_offset.1 + dy, parent_offset.2 + dz)
    }

    fn contains_point( offset:&(u32, u32, u32), size:u32, point:&(u32, u32, u32) ) -> bool {
        point.0 >= offset.0 && point.0 < offset.0 + size &&
        point.1 >= offset.1 && point.1 < offset.1 + size &&
        point.2 >= offset.2 && point.2 < offset.2 + size
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
        let child_index = Self::get_child_index( depth, &(x, y, z) );

        if depth == 1 {
            self.children[ child_index ] = OctreeNode::Leaf( Some( value ) );
            return
        }

        self.children[ child_index ].insert( depth - 1, x, y, z, value );
        self.children[ child_index ].try_compress();
    }

    fn get_child_index( reversed_depth:u8, point:&(u32, u32, u32) ) -> usize {
        let shift = reversed_depth - 1;
        let xi = ((point.0 >> shift) & 1) as usize;
        let yi = ((point.1 >> shift) & 1) as usize;
        let zi = ((point.2 >> shift) & 1) as usize;

        xi | (yi << 1) | (zi << 2)
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

impl Octree<Voxel> {
    pub fn get_visible_with_flood( &self, initial_point:(u32,u32,u32) ) -> Vec<(u32, u32, u32, Rc<Voxel>)> {
        struct Point {
            coords: (u32,u32,u32),
            depth: u8,
            check_dir: u8
        }

        let mut result = HashMap::new();
        let mut points_memory = HashMap::<(u32, u32, u32), u8>::new();
        let mut points = VecDeque::from([ Point { coords:initial_point, depth:self.max_depth, check_dir:Direction::UNSPECIFIED } ]);
        let mut path = vec![(&self.root, (0, 0, 0))];
        // let debug_coord = (0, 3, 5);
        let debug_coord = (0, 3, 4);
        // let debug_coord = (26, 2, 22);
        // let debug_coord = (27, 28, 12);
        // let debug_coord = (28, 28, 12);
        // let debug_coord = (1000, 1000, 1000);

        // Collecting leaf, n.offset=(0, 3
        //

        'points: while let Some( point ) = points.pop_front() {
            // println!( " 0. - - - p.coords={:?}", point.coords );

            // Going up in path vector
            while {
                let (_, offset) = path.last().unwrap();
                let depth = path.len() as u8 - 1;
                let size = 1 << (self.max_depth - depth);
                // println!(
                //     "max_depth={}, path.len={}, shift={}, size={}, offset={:?}, coords={:?}",
                //     self.max_depth, path.len(), self.max_depth - depth, size, offset, point.coords
                // );
                !OctreeNode::<Voxel>::contains_point( &offset, size, &point.coords )
            } {
                if path.len() == 1 {
                    unreachable!(
                        "Cannot pop last path element! path.len={}; size={}; offset={:?}; point={:?}",
                        path.len(), 1 << (self.max_depth - path.len() as u8), path.last().unwrap().1, point.coords )
                }

                path.pop().unwrap();
            }


            // Going down in path vector
            // println!( "1. Outer: p.coords={:?} p.depth={:?} p.dir={:?}", point.coords, point.depth, Direction::get_dir_name( point.check_dir ) );

            let (mut node_ptr, mut node_offset) = path.last().unwrap();

            // if point.coords.0 == 30 && point.coords.1 == 28 && point.coords.2 == 12 {
            //     println!( "{:?}", point.coords )
            // }

            while let OctreeNode::Branch( branch ) = node_ptr {
                let depth = path.len() as u8 - 1;
                let reversed_depth = self.max_depth - depth;


                if depth == point.depth {
                    // println!(
                    //     "Dividing | parent={:?}; p.coords={:?}; p.check_dir={:?}; p.depth={}, n.depth={} parent_size={} | split={:?}",
                    //     node_offset, point.coords, Direction::get_dir_name( point.check_dir ), point.depth, depth, 1 << reversed_depth,
                    //     Direction::split_branch_point_into_chilren_by_direction( point.check_dir, point.coords, 1 << (reversed_depth - 1) )
                    // );
                    // if node_offset.0 == debug_coord.0 && node_offset.1 == debug_coord.1 && node_offset.2 == debug_coord.2 {
                    // }

                    for coords in Direction::split_branch_point_into_chilren_by_direction( point.check_dir, point.coords, 1 << (reversed_depth - 1) ) { // Every node on the side}
                        // if coords.0 == debug_coord.0 && coords.1 == debug_coord.1 && coords.2 == debug_coord.2 {
                        //     println!(
                        //         "insertion from division | parent={:?}; next={:?}; p.coords={:?}; p.check_dir={:?}; p.depth={}, parent_size={}",
                        //         node_offset, coords, point.coords, Direction::get_dir_name( point.check_dir ), point.depth, 1 << reversed_depth
                        //     )
                        // }

                        points.push_front( Point {
                            coords,
                            check_dir: point.check_dir,
                            depth: point.depth + 1,
                        } );
                    }

                    // println!( "Division" );

                    continue 'points;
                }

                let child_index = OctreeBranch::<Voxel>::get_child_index( reversed_depth, &point.coords );
                let child_offset = OctreeNode::<Voxel>::get_child_offset( node_offset, 1 << reversed_depth, child_index );

                node_ptr = &branch.children[ child_index as usize ];
                node_offset = child_offset;
                path.push( (node_ptr, child_offset) );
                // println!( "  - node_offset={:?} node_r_depth={:?}, size={:?}", offset, reversed_depth, 1 << reversed_depth );
            }



            // Continue if node offset is processed
            if result.contains_key( &point.coords ) {
                // println!( "Skip result n.offset={:?} p.coords={:?}, {:?}, {}, size={}", node_offset, point.coords, point.check_dir, point.depth, 1 << (self.max_depth - (path.len() as u8 - 1)) );
                continue
            }

            if let Some( count ) = points_memory.get_mut( &point.coords ) {
                // if point.coords.0 == debug_coord.0 && point.coords.1 == debug_coord.1 && point.coords.2 == debug_coord.2 {
                //     println!( "count of debug {:?} (with depth = {}) before decrement = {}", point.coords, point.depth, count )
                // }

                // if *count == 0 {
                //     unreachable!( "Count cannnot be zero here! p.coords={:?}; count={}", point.coords, count )
                // }

                *count -= 1;
                continue;
            }


            // Handling the leaf
            let depth = path.len() as u8 - 1;
            let reversed_depth = self.max_depth - depth;
            let leaf_size = 1 << reversed_depth;
            // println!( "2. Found: leaf_offset={:?} leaf_r_depth={:?}, leaf_size={:?} value={:?}", node_offset, reversed_depth, leaf_size, node_ptr );

            let leaf_value = match node_ptr {
                OctreeNode::Leaf( opt ) => opt,
                _ => unreachable!()
            };


            // Fill results if have value
            if let Some( data ) = leaf_value {
                let size = 1 << reversed_depth;
                let decremented_size = size - 1;

                for x in 0..size {
                    for y in 0..size {
                        for z in 0..size {
                            if x == 0 || y == 0 || z == 0 || x == decremented_size || y == decremented_size || z == decremented_size {
                                let coord = (node_offset.0 + x, node_offset.1 + y, node_offset.2 + z);
                                result.entry( coord ).or_insert_with( || data.clone() );
                            }
                        }
                    }
                }

                // println!( "Collecting leaf, n.offset={:?}, p.coords={:?}", node_offset, point.coords );

                continue
            }


            // Generate siblings for empty cell
            let root_size = 1 << self.max_depth;
            let mut next_points = Vec::with_capacity( 6 );

            // if node_offset.0 == debug_coord.0 && node_offset.1 == debug_coord.1 && node_offset.2 == debug_coord.2 {
            //     println!( "debug point" )
            // }

            if point.check_dir != Direction::RIGHT && node_offset.0 > 0 {
                next_points.push( Point { depth, coords:(node_offset.0 - 1,         node_offset.1, node_offset.2), check_dir:Direction::LEFT } )
            }
            if point.check_dir != Direction::LEFT && node_offset.0 < root_size - leaf_size {
                next_points.push( Point { depth, coords:(node_offset.0 + leaf_size, node_offset.1, node_offset.2), check_dir:Direction::RIGHT } )
            }

            if point.check_dir != Direction::TOP && node_offset.1 > 0 {
                next_points.push( Point { depth, coords:(node_offset.0, node_offset.1 - 1,         node_offset.2), check_dir:Direction::BOTTOM } )
            }
            if point.check_dir != Direction::BOTTOM && node_offset.1 < root_size - leaf_size {
                next_points.push( Point { depth, coords:(node_offset.0, node_offset.1 + leaf_size,  node_offset.2), check_dir:Direction::TOP } )
            }

            if point.check_dir != Direction::FRONT && node_offset.2 > 0 {
                next_points.push( Point { depth, coords:(node_offset.0, node_offset.1, node_offset.2 - 1        ), check_dir:Direction::BACK } )
            }
            if point.check_dir != Direction::BACK && node_offset.2 < root_size - leaf_size {
                next_points.push( Point { depth, coords:(node_offset.0, node_offset.1, node_offset.2 + leaf_size), check_dir:Direction::FRONT } )
            }


            // Memoize outer children
            if leaf_size == 1 {
                points_memory.insert( node_offset, if point.check_dir == Direction::UNSPECIFIED { 6 } else { 5 } );

                // if node_offset.0 == debug_coord.0 && node_offset.1 == debug_coord.1 && node_offset.2 == debug_coord.2 {
                // // if points_memory.get( &debug_coord ).is_some() {
                //     println!(
                //         "insertion into memory {:?} = {:?}; node.offset = {:?}; point.coord = {:?}, point.dir = {} [x1]",
                //         debug_coord, points_memory.get( &debug_coord ), node_offset, point.coords, point.check_dir
                //     );
                // }
            } else if leaf_size == 2 {
                points_memory.reserve( 8 );

                let next_memory = HashMap::from([
                    (node_offset, 3),
                    ((node_offset.0 + 1,   node_offset.1,      node_offset.2),     3),
                    ((node_offset.0,       node_offset.1 + 1,  node_offset.2),     3),
                    ((node_offset.0 + 1,   node_offset.1 + 1,  node_offset.2),     3),
                    ((node_offset.0,       node_offset.1,      node_offset.2 + 1), 3),
                    ((node_offset.0 + 1,   node_offset.1,      node_offset.2 + 1), 3),
                    ((node_offset.0,       node_offset.1 + 1,  node_offset.2 + 1), 3),
                    ((node_offset.0 + 1,   node_offset.1 + 1,  node_offset.2 + 1), 3),
                ]);

                // if next_memory.get( &debug_coord ).is_some() {
                //     println!( "{:?}", next_memory );
                //     println!(
                //         "insertion into memory {:?} = {:?}; node.offset = {:?}; point.coord = {:?}, point.dir = {} size={}",
                //         debug_coord, next_memory.get( &debug_coord ), node_offset, point.coords, Direction::get_dir_name( point.check_dir ), leaf_size
                //     );
                // }

                points_memory.extend( next_memory );
            } else {
                let boundary_range = [0, leaf_size - 1];
                let internal_size = leaf_size - 2;

                points_memory.reserve( (leaf_size * leaf_size * leaf_size) as usize - (internal_size * internal_size * internal_size) as usize );
                let mut next_memory = HashMap::new();

                for x in boundary_range {
                    for y in 1..leaf_size-1 {
                        for z in 1..leaf_size-1 {
                            next_memory.insert((node_offset.0 + x, node_offset.1 + y, node_offset.2 + z ), 1 );
                        }
                    }
                }

                for x in 1..leaf_size-1 {
                    for y in boundary_range {
                        for z in 1..leaf_size-1 {
                            next_memory.insert((node_offset.0 + x, node_offset.1 + y, node_offset.2 + z ), 1 );
                        }
                    }
                }

                for x in 1..leaf_size-1 {
                    for y in 1..leaf_size-1 {
                        for z in boundary_range {
                            next_memory.insert((node_offset.0 + x, node_offset.1 + y, node_offset.2 + z ), 1 );
                        }
                    }
                }

                for x in boundary_range {
                    for y in boundary_range {
                        for z in 1..leaf_size-1 {
                            next_memory.insert((node_offset.0 + x, node_offset.1 + y, node_offset.2 + z ), 2 );
                        }
                    }
                }

                for x in boundary_range {
                    for y in 1..leaf_size-1 {
                        for z in boundary_range {
                            next_memory.insert((node_offset.0 + x, node_offset.1 + y, node_offset.2 + z ), 2 );
                        }
                    }
                }

                for x in 1..leaf_size-1 {
                    for y in boundary_range {
                        for z in boundary_range {
                            next_memory.insert((node_offset.0 + x, node_offset.1 + y, node_offset.2 + z ), 2 );
                        }
                    }
                }

                for x in boundary_range {
                    for y in boundary_range {
                        for z in boundary_range {
                            next_memory.insert((node_offset.0 + x, node_offset.1 + y, node_offset.2 + z ), 3 );
                        }
                    }
                }

                if next_memory.get( &debug_coord ).is_some() {
                    println!( "{:?}", next_memory );
                    println!(
                        "insertion into memory {:?} = {:?}; node.offset = {:?}; point.coord = {:?}, point.dir = {} [xN]",
                        debug_coord, next_memory.get( &debug_coord ), node_offset, point.coords, point.check_dir
                    );
                }

                points_memory.extend( next_memory );
            }


            // Process siblings
            points.reserve( next_points.len() );

            for next_point in next_points {
                if result.contains_key( &next_point.coords ) {
                    continue;
                }

                if let Some( count ) = points_memory.get_mut( &next_point.coords ) {
                    // if next_point.coords.0 == debug_coord.0 && next_point.coords.1 == debug_coord.1 && next_point.coords.2 == debug_coord.2 {
                    //     println!( "count for {:?} = {:?}, np.dir={:?}, leaf.offset={:?}", next_point.coords, *count, Direction::get_dir_name( next_point.check_dir ), node_offset );
                    // }
                    // if *count == 0 {
                    //     println!( "count==0, p={:?}", next_point.coords )
                    // }
                    *count -= 1;
                } else {
                    // if next_point.coords.0 == debug_coord.0 && next_point.coords.1 == debug_coord.1 && next_point.coords.2 == debug_coord.2 {
                    //     println!( "creating point of {:?}", debug_coord )
                    // }
                    points.push_back( next_point );
                }
            }
        }

        result.drain().map( |pair| (pair.0.0, pair.0.1, pair.0.2, pair.1) ).collect()
    }
}

impl WorldHolder for Octree<Voxel> {
    fn get_voxel( &self, x:u32, y:u32, z:u32 ) -> Option<Rc<Voxel>> {
        self.get( x, y, z )
    }

    fn get_all_voxels( &self ) -> Vec<(u32, u32, u32, Rc<Voxel>)> {
        self.get_voxels()
    }

    fn get_all_visible_voxels_from( &self, from:(u32, u32, u32) ) -> Vec<(u32, u32, u32, Rc<Voxel>)> {
        self.get_visible_with_flood( from )
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
