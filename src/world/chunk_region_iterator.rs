

// Example for 2D space:
// d =  1 ->  1;  1
// d =  3 ->  8;  2,  3,  4,  5,  6,  7,  8,  9
// d =  5 -> 16; 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25
//           15 - 9 = 6
//           6 / d = 1 r 1 => skip 1 side, iterate to position "1"

// Example for 3D space:
// 1 ->  1:  1
// 3 -> 26:  2,  3,  4,  5,  6,  7,  8,  9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27
// 5 -> 98: 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 48, 49, 50, 51, 52, ...
//      id = 80
//      80 - 27 = 63
//      63 / 5^2 = 2 r 13 => skip 2 sides, iterate to position "13"

// Cube layers:
// 1
// 26  = 24 * 1^2 + 2 = 3*3*2 + 3*1*2 + 1*1*2
// 98  = 24 * 2^2 + 2 = 5*5*2 + 5*3*2 + 3*3*2
// 218 = 24 * 3^2 + 2 = 5^2 = 7*7*2 + 7*5*2 + 5*5*2

use std::ops::Range;

pub struct ChunkRegionIterator {
    pub side: i8,
    layer: i8,
    layer_size: i16,
    iterations: Option<i16>,
    x: i16,
    y: i16,
    z: i16,
}

impl ChunkRegionIterator {
    pub fn new( layer:u8 ) -> Self {
        assert!( layer < 125, "Layer should be lower than 125, passed {}", layer );

        let layer = 1;

        Self {
            side: 0,
            layer,
            layer_size: (layer as i16) * 2 - 1,
            iterations: None,
            x: 0,
            y: 0,
            z: 0,
        }
    }

    pub fn with_range( range:Range<i16> ) -> Self {
        let mut layer = 1;
        let mut layer_first_idx = 1;

        loop {
            let indices_in_layer = layer_first_idx + 24 * (layer * layer) + 2;
            if indices_in_layer >= range.start { break }

            layer += 1;
            layer_first_idx = indices_in_layer;
        }

        let layer_size = (layer as i16 + 1) * 2 - 1;
        let side_size = layer_size * layer_size;
        let border = layer_size - 1;
        let side = (layer_first_idx / side_size) as i8;
        let start_idx_on_side = layer_first_idx % layer;

        let (x, y, z) = match side {
            0 | 2 => (
                (layer_size + 1) / -2 + layer_first_idx % layer_size,
                layer_size / if side == 0 { 2 } else { -2 },
                (layer_size) / -2 + start_idx_on_side / layer_size,
            ),

            1 | 3 => (
                if side == 0 { 0 } else { border },
                layer_first_idx % layer_size,
                start_idx_on_side / layer_size,
            ),

            4 | 5 => (
                start_idx_on_side / layer_size,
                layer_first_idx % layer_size,
                if side == 0 { 0 } else { border },
            ),

            _ => unreachable!( "Side cannot be out of 0-5 scope (layer_first_idx={}, layer={}, side_size={}, side={})", layer_first_idx, layer, side_size, side )
        };

        println!( "side={side} layer={layer}, layer_size={layer_size}, layer_first_idx={layer_first_idx}, side_row={}, ({x}, {y}, {z})", start_idx_on_side / layer_size );

        Self {
            side,
            layer: layer as i8,
            layer_size,
            iterations: Some( range.end - range.start ),
            x,
            y,
            z,
        }
    }
}

impl Iterator for ChunkRegionIterator {
    type Item = (i16, i16, i16);

    fn next( &mut self ) -> Option<Self::Item> {
        let border = self.layer_size / 2;

        if let Some( iterations ) = self.iterations {
            if iterations == 0 { return None }
            self.iterations = Some( iterations - 1 );
        }

        match self.side {
            0 => {
                // println!( "+Y x={} z={}", self.x, self.z );

                if self.z == border + 1 {
                    if self.x == border {
                        self.y = border - 1;
                        self.z = -border;
                        self.side += 1;

                        return self.next();
                    } else {
                        self.z = -border + 1;
                        self.x += 1;
                    }
                } else {
                    self.z += 1;

                    if self.x == self.layer_size {
                        self.x = 0;
                    }
                }

                return Some( (self.x, self.layer_size / 2, self.z - 1) );
            }

            1 => {
                // println!( "+X y={} z={}", self.y, self.z );

                if self.z == border + 1 {
                    if self.y == -border + 1 {
                        self.x = border;
                        self.z = -border;
                        self.side += 1;

                        return self.next();
                    } else {
                        self.z = -border + 1;
                        self.y += 1;
                    }
                } else {
                    self.z += 1;

                    if self.y == self.layer_size {
                        self.y = 0;
                    }
                }

                return Some( (self.layer_size / 2, self.y, self.z - 1) );
            }

            2 => {
                // println!( "-Y x={} z={}", self.x, self.z );

                if self.z == border + 1 {
                    if self.x == -border {
                        self.y = -border + 1;
                        self.z = -border;
                        self.side += 1;

                        return self.next();
                    } else {
                        self.z = -border + 1;
                        self.x -= 1;
                    }
                } else {
                    self.z += 1;

                    if self.x == self.layer_size {
                        self.x = 0;
                    }
                }

                return Some( (self.x, self.layer_size / -2, self.z - 1) );
            }

            3 => {
                // println!( "-X y={} z={}", self.y, self.z );

                if self.z == border + 1 {
                    if self.y == border - 1 {
                        self.y = border - 1;
                        self.x = -border + 1;
                        self.side += 1;

                        return self.next();
                    } else {
                        self.z = -border + 2;
                        self.y += 1;
                    }
                } else {
                    self.z += 1;

                    if self.y == self.layer_size {
                        self.y = 0;
                    }
                }

                return Some( (self.layer_size / -2, self.y, self.z - 1) );
            }

            4 => {
                // println!( "-Z x={} y={}", self.x, self.y );

                if self.x == border {
                    if self.y == border - 1 {
                        self.y = border - 1;
                        self.x = -border + 1;
                        self.side += 1;

                        return self.next();
                    } else {
                        self.x = -border + 1;
                        self.y += 1;
                    }
                } else {
                    self.x += 1;

                    if self.y == self.layer_size {
                        self.y = 0;
                    }
                }

                return Some( (self.x - 1, self.y, self.layer_size / -2) );
            }

            5 => {
                // println!( "+Z y={} z={}", self.y, self.z );

                if self.x == border {
                    if self.y == border - 1 {
                        self.y = -border;
                        self.side += 1;

                        return self.next();
                    } else {
                        self.x = -border + 1;
                        self.y += 1;
                    }
                } else {
                    self.x += 1;

                    if self.y == self.layer_size {
                        self.y = 0;
                    }
                }

                return Some( (self.x - 1, self.y, self.layer_size / 2) );
            }

            _ => unreachable!( "Side cannot be out of 0-5 scope" )
        }
    }
}