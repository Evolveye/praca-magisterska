use crate::world::block::Block;

pub struct Chunk {
  pub blocks: [[[Block; 16]; 16]; 384],
}

impl Chunk {
  pub fn new( x:i16, z:i16 ) -> Chunk {
    let mut blocks = [[[Block::new( 0, 0, 0 ); 16]; 16]; 384];

    for cy in 0..384i16 {
      for cx in 0..16i16 {
        for cz in 0..16i16 {
          blocks[ cy as usize ][ cx as usize ][ cz as usize ] = Block::new( x * 16 + cx, cy, z * 16 + cz );
        }
      }
    }

    Chunk { blocks }
  }

  pub fn print_dimensions( &self ) {
    println!( "Chunk dimensions {{ x={1} y={0} z={2} }} ", self.blocks.len(), self.blocks[ 0 ].len(), self.blocks[ 0 ][ 0 ].len() )
  }
}
