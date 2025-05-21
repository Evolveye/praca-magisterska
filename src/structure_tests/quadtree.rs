type NoiseValue = f64;
type TreeSize = u32;

pub enum QuadtreeNode {
    Leaf( NoiseValue ),
    Branch( Box<QuadtreeBranch> ),
}

impl QuadtreeNode {
    fn get_min( &self ) -> NoiseValue {
        match self {
            QuadtreeNode::Leaf( value ) => *value,
            QuadtreeNode::Branch( branch ) => branch.min,
        }
    }

    fn get( &self, reversed_depth:u8, x:TreeSize, y:TreeSize ) -> NoiseValue {
        match self {
            QuadtreeNode::Leaf( value ) => value.clone(),
            QuadtreeNode::Branch( branch ) => {
                let child_index = QuadtreeBranch::get_child_index( reversed_depth, (x, y) );
                branch.children[ child_index ].get( reversed_depth - 1, x, y )
            }
        }
    }

    fn fill( offset:(TreeSize, TreeSize), depth:u8, generate_value:&impl Fn(TreeSize, TreeSize) -> NoiseValue ) -> Self {
        if depth == 0 {
            return QuadtreeNode::Leaf( generate_value( offset.0, offset.1 ) );
        }

        let child_size = 1 << (depth - 1);
        let children = [
            Self::fill( (offset.0,              offset.1),              depth - 1, generate_value ),
            Self::fill( (offset.0 + child_size, offset.1),              depth - 1, generate_value ),
            Self::fill( (offset.0,              offset.1 + child_size), depth - 1, generate_value ),
            Self::fill( (offset.0 + child_size, offset.1 + child_size), depth - 1, generate_value ),
        ];

        QuadtreeNode::Branch( Box::new( QuadtreeBranch::from_children( children ) ) )
    }

    fn get_child_offset( parent_offset:(TreeSize, TreeSize), parent_size:TreeSize, child_index:usize ) -> (TreeSize, TreeSize) {
        let child_size = parent_size >> 1;
        let child_index = child_index as u8;
        let dx = ( child_index       & 1) as u32 * child_size;
        let dy = ((child_index >> 1) & 1) as u32 * child_size;

        (parent_offset.0 + dx, parent_offset.1 + dy)
    }

    fn proces_entire_tree( &self, offset:(TreeSize, TreeSize, TreeSize), reversed_depth:u8, processor:&mut impl FnMut((TreeSize, TreeSize, TreeSize), TreeSize, NoiseValue) -> TreeSize ) {
        let size = 1 << reversed_depth;

        match self {
            QuadtreeNode::Leaf( value ) => {
                processor( offset, size, *value );
            },

            QuadtreeNode::Branch( branch ) => {
                let current_min = processor( offset, size, branch.min );

                for i in 0..4 {
                    let child_offset = QuadtreeNode::get_child_offset( (offset.0, offset.2), size, i );
                    branch.children[ i ].proces_entire_tree( (child_offset.0, current_min, child_offset.1), reversed_depth - 1, processor );
                }
            },
        };
    }
}

pub struct QuadtreeBranch {
    pub children: [QuadtreeNode; 4],
    pub min: NoiseValue
}

impl QuadtreeBranch {
    fn from_children( children:[QuadtreeNode; 4] ) -> Self {
        let min = children[ 0 ].get_min()
            .min( children[ 1 ].get_min() )
            .min( children[ 2 ].get_min() )
            .min( children[ 3 ].get_min() ).to_owned();

        QuadtreeBranch { children, min }
    }

    fn get_child_index( reversed_depth:u8, point:(TreeSize, TreeSize) ) -> usize {
        let shift = reversed_depth - 1;
        let xi = ((point.0 >> shift) & 1) as usize;
        let yi = ((point.1 >> shift) & 1) as usize;

        xi | (yi << 1)
    }
}

pub struct Quadtree {
    pub root: QuadtreeNode,
    pub max_depth: u8,
}

impl Quadtree {
    pub fn new( max_depth:u8, initial_value:NoiseValue ) -> Self {
        Self {
            max_depth,
            root: QuadtreeNode::Leaf( initial_value ),
        }
    }

    pub fn proces_entire_tree( &self, processor:&mut impl FnMut((TreeSize, TreeSize, TreeSize), TreeSize, NoiseValue) -> TreeSize ) {
        self.root.proces_entire_tree( (0, 0, 0), self.max_depth, processor )
    }

    pub fn from_terrain_generation( max_depth:u8, generate_value:&impl Fn(TreeSize, TreeSize) -> NoiseValue ) -> Self {
        let mut root_depth = 0;
        let mut root_node = QuadtreeNode::Leaf( generate_value( 0, 0 ) );

        loop {
            if root_depth == max_depth {
                break Self {
                    max_depth,
                    root: root_node,
                }
            }

            let size = 1 << root_depth;
            let children = [
                root_node,
                QuadtreeNode::fill( (size, 0),    root_depth, generate_value ),
                QuadtreeNode::fill( (0,    size), root_depth, generate_value ),
                QuadtreeNode::fill( (size, size), root_depth, generate_value ),
            ];

            root_node = QuadtreeNode::Branch( Box::new( QuadtreeBranch::from_children( children ) ) );
            root_depth += 1;
        }
    }

    pub fn get( &self, x:TreeSize, y:TreeSize ) -> NoiseValue {
        debug_assert!( x < 2u32.pow( self.max_depth as u32 ) as TreeSize, "Passed 'x' value ({}) is greater than quadtree size ({})", x, 2u32.pow( self.max_depth as u32 ) as TreeSize - 1 );
        debug_assert!( y < 2u32.pow( self.max_depth as u32 ) as TreeSize, "Passed 'y' value ({}) is greater than quadtree size ({})", y, 2u32.pow( self.max_depth as u32 ) as TreeSize - 1 );

        self.root.get( self.max_depth, x, y )
    }

    pub fn get_max_depth_for( n:u32 ) -> u8 {
        (32 - (n - 1).leading_zeros()) as u8
    }
}
