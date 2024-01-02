use super::chunk::Chunk;

pub struct VisibleFragment {
  pub chunks: Vec<Vec<Chunk>>
}

impl VisibleFragment {
  pub fn new() -> Self {
    let radius:i16 = 32;
    let edge_len = radius * 2;
    let mut chunks:Vec<Vec<Chunk>> = Vec::with_capacity( edge_len as usize );

    for cx in 0..edge_len {
      chunks.push( Vec::with_capacity( edge_len as usize ) );

      for cz in 0..edge_len {
        chunks[ cx as usize ].push( Chunk::new( cx, cz ) );
      }
    }

    VisibleFragment { chunks }
  }

  pub fn print_dimensions( &self ) {
    println!( "World fragment dimensions {{ x={0} z={1} }} ", self.chunks.len(), self.chunks[ 0 ].len() )
  }
}