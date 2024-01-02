#[derive(Debug, Clone, Copy)]
pub struct Block {
  x: i16,
  y: i16,
  z: i16,
}

impl Block {
  pub fn new( x:i16, y:i16, z:i16 ) -> Self {
    Self { x, y, z }
  }

  pub fn print( &self ) {
    println!( "Block {{ x={0} y={1} z={2} }}", self.x, self.y, self.z )
  }
}