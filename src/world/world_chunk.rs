use crate::{
    structure_tests::octree::Octree,
    world::{
        world::{ CHUNK_SIZE, CHUNK_SIZE_X2, CHUNK_SIZE_X3 },
        world_holder::{ Voxel, VoxelSide, WorldHolding }
    }
};

#[allow(dead_code)]
pub struct WorldChunk {
    pub data: Octree<Voxel>,
    pub renderables: Vec<VoxelSide>,
    pub solids_mask: ChunkBitmask,
}

impl WorldChunk {
    pub fn new( data:Octree<Voxel> ) -> Self {
        let solids_mask = data.to_bitmask();

        Self {
            data,
            renderables: vec![],
            solids_mask,
        }
    }

    pub fn remesh( &mut self, mut offset:(i64, i64, i64), neighbours:[&WorldChunk; 26] ) {
        // self.renderables = self.data.get_visible_with_flood( (0, self.data.get_size() as u32 - 1, 0) )
        //     .into_iter()
        //     .filter_map( |mut s| {
        //         // if s.get_position().x != 0.0 { return None }
        //         s.move_by( (offset.0 as f32, offset.1 as f32, offset.2 as f32) );
        //         Some( s )
        //     } )
        //     .collect::<Vec<VoxelSide>>();
        // return;

        // println!( "Remeshing chunk {:?}", offset );

        offset.0 = offset.0 * CHUNK_SIZE as i64;
        offset.1 = offset.1 * CHUNK_SIZE as i64;
        offset.2 = offset.2 * CHUNK_SIZE as i64;

        let mut renderables = vec![];
        let mut col_face_masks = vec![ 0; CHUNK_SIZE_X3 * 2 ];
        let neighbour_shift = CHUNK_SIZE - 1;
        let axies_neighbours = [
            (neighbours[ 13 ], neighbours[ 12 ]),
            (neighbours[ 21 ], neighbours[  4 ]),
            (neighbours[ 15 ], neighbours[ 10 ]),
        ];

        for axis in 0..3 {
            for i in 0..CHUNK_SIZE_X2 {
                let index = CHUNK_SIZE_X2 * axis + i;
                let column = self.solids_mask.data[ index ];

                let neighbour_a_shift = (axies_neighbours[ axis ].0.solids_mask.data[ index ] & 1) << neighbour_shift;
                let neighbour_b_shift = (axies_neighbours[ axis ].1.solids_mask.data[ index ] >> neighbour_shift) & 1;

                col_face_masks[ CHUNK_SIZE_X2 * (axis * 2    ) + i ] = column & !(column << 1 | neighbour_b_shift);
                col_face_masks[ CHUNK_SIZE_X2 * (axis * 2 + 1) + i ] = column & !(column >> 1 | neighbour_a_shift);
            }
        }

        for axis_turn in 0..6 {
            for x in 0..CHUNK_SIZE {
                for y in 0..CHUNK_SIZE {
                    let column_index = x + y * CHUNK_SIZE + CHUNK_SIZE_X2 * axis_turn;
                    let mut num = col_face_masks[ column_index ];

                    while num != 0 {
                        let z = num.trailing_zeros();

                        let voxel_pos = match axis_turn {
                            0 | 1 => (z, x as u32, y as u32), // y,z=x 1,2 X
                            2 | 3 => (x as u32, z, y as u32), // x,z=y 3,4 Y
                            _     => (x as u32, y as u32, z), // x,y=z 5,6 Z
                        };

                        if let Some( voxel ) = self.data.get( voxel_pos.0, voxel_pos.1, voxel_pos.2 ) {
                            renderables.push( VoxelSide::from_voxel_rc(
                                offset.0 + voxel_pos.0 as i64,
                                offset.1 + voxel_pos.1 as i64,
                                offset.2 + voxel_pos.2 as i64,
                                axis_turn as u8 + 1,
                                &voxel
                            ) );
                        }

                        num &= num - 1;
                    }
                }
            }
        }

        self.renderables = renderables;
    }

    #[allow(unused)]
    pub fn print_bitmask_layer( &self, layer:usize ) {
        let size = self.data.get_size() as usize;
        let mut num = 0;

        print!( "   " );
        for _ in 0..=1 {
            for _ in 0..size {
                print!( "{num}" );
                if num < 9 { num += 1 } else { num = 0 }
            }

            num = 0;
            print!( " " );
        }

        print!( "\n  ,{}+{}", "-".repeat( size ), "-".repeat( size ) );

        for z in 0..size {
            print!( "\n{num} |" );

            for x in 0..size {
                let bit = (self.solids_mask.data[ z + (layer * CHUNK_SIZE) + CHUNK_SIZE_X2     ] >> x) & 1;
                print!("{}", if bit == 0 { " " } else { "#" } );
            }

            print!( "|" );

            for x in 0..size {
                let bit = (self.solids_mask.data[ x + (layer * CHUNK_SIZE) + CHUNK_SIZE_X2 * 2 ] >> z) & 1;
                print!("{}", if bit == 0 { " " } else { "#" } );
            }

            if num < 9 { num += 1 } else { num = 0 }
        }

        println!();
    }
}

pub struct ChunkBitmask {
    pub data: Vec<u64>,
}

impl ChunkBitmask {
    pub fn new( size:usize ) -> Self {
        Self {
            data: vec![ 0; size ]
        }
    }
}
