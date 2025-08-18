

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
    pub iterations: Option<u32>,
    pub side: u8,
    layer: u8,
    layer_edge: i8,
    x: i8,
    y: i8,
    z: i8,
}

#[allow(dead_code)]
impl ChunkRegionIterator {
    pub fn new( layer:u8 ) -> Self {
        assert!( layer < 125, "Layer should be lower than 125, passed {}", layer );

        let layer = 1;

        Self {
            side: 0,
            layer,
            layer_edge: (layer as i8 * 2 - 1),
            iterations: None,
            x: 0,
            y: 0,
            z: 0,
        }
    }

    pub fn with_range( range:Range<u32> ) -> Self {
        let iterations = Some( range.end - range.start );

        if range.start == 0 {
            return Self {
                side: 0,
                layer: 0,
                layer_edge: 1,
                iterations,
                x: 0,
                y: -1,
                z: 0,
            }
        }

        let mut layer = 1;
        let mut layer_first_idx = 1;
        let start_index = range.start as i32;

        loop {
            let indices_in_layer = layer_first_idx + Self::get_layer_size( layer );
            if indices_in_layer > start_index { break }

            layer += 1;
            layer_first_idx = indices_in_layer;
        }

        let layer_edge = (layer + 1) * 2 - 1;
        let border = layer_edge - 1;
        let start_idx_on_layer = start_index - layer_first_idx;

        let side_size_small = (layer_edge - 2) * (layer_edge - 2);
        let side_size = layer_edge * (layer_edge - 1);
        let side = 1 + if start_idx_on_layer < side_size * 4 {
            start_idx_on_layer / side_size
        } else {
            4 + (start_idx_on_layer - side_size * 4) / side_size_small
        };

        // println!( "ChunkRegionIterator | side={side} layer={layer}, layer_edge={layer_edge}, layer_first_idx={layer_first_idx}, start_idx_on_layer={start_idx_on_layer}, side_row={}, start_index={start_index}", start_idx_on_layer / layer_edge );

        let (x, y, z) = match side {
            1 => (
                layer_edge / -2 + start_idx_on_layer / layer_edge,
                layer_edge / -border,
                layer_edge / -2 + start_idx_on_layer % layer_edge,
            ),

            2 => (
                layer_edge / border,
                layer_edge /  2 - (start_idx_on_layer - side_size) / layer_edge,
                layer_edge / -2 + start_idx_on_layer % layer_edge,
            ),

            3 => (
                layer_edge /  2 - (start_idx_on_layer - side_size * 2) / layer_edge,
                layer_edge / border,
                layer_edge / -2 + start_idx_on_layer % layer_edge,
            ),

            4 => (
                layer_edge / -border,
                layer_edge / -2 + (start_idx_on_layer - side_size * 3) / layer_edge,
                layer_edge / -2 + start_idx_on_layer % layer_edge,
            ),

            // 5 => (
            //     layer_size / -2 + start_idx_on_layer / layer_size,
            //     layer_size / -2 + start_idx_on_layer % layer_size,
            //     layer_size / border,
            // ),

            5 => (
                layer_edge / -2 + (start_idx_on_layer - side_size * 4) % (layer_edge - 2) + 1,
                layer_edge /  2 - (start_idx_on_layer - side_size * 4) / (layer_edge - 2) - 1,
                layer_edge / border,
            ),

            6 => (
                layer_edge / -2 + (start_idx_on_layer - side_size * 4) % (layer_edge - 2) + 1,
                layer_edge /  2 - (start_idx_on_layer - side_size * 4 - side_size_small) / (layer_edge - 2) - 1,
                layer_edge / -border,
            ),

            _ => unreachable!( "Side cannot be out of 0-5 scope (layer_first_idx={layer_first_idx}, layer={layer}, side_size={side_size}, side={side})" )
        };

        Self {
            side: side as u8,
            layer: layer as u8,
            layer_edge: layer_edge as i8,
            iterations,
            x: x as i8,
            y: y as i8,
            z: z as i8,
        }
    }

    pub fn get_pos_from_index( index:u32 ) -> (i8, i8, i8) {
        Self::with_range( index..(index+1) ).next().unwrap()
    }

    fn setup_next_layer( &mut self ) {
        self.layer += 1;

        let layer_size = (self.layer as i8 + 1) * 2 - 1;

        self.x = layer_size / -2;
        self.y = layer_size /  2;
        self.z = layer_size / -2;

        self.layer_edge = layer_size;
        self.side = 1;
    }

    fn update_iterations( &mut self, count:i8 ) {
        if let Some( iterations ) = self.iterations {
            if count < 0 {
                if iterations == 0 { return }
                self.iterations = Some( iterations - (-count) as u32 );
            } else {
                self.iterations = Some( iterations + count as u32 );
            };
        }
    }

    pub fn get_layer_edge( layer:i32 ) -> i32 {
        (layer + 1) * 2 - 1
    }

    pub fn get_layer_size( layer:i32 ) -> i32 {
        if layer == 0 {
            1
        } else {
            24 * (layer * layer) + 2
        }
    }
}

impl Iterator for ChunkRegionIterator {
    type Item = (i8, i8, i8);

    fn next( &mut self ) -> Option<Self::Item> {
        let border = (self.layer_edge / 2) as i8;

        // println!( "ChunkRegionIterator Iter | border={border}, side={}, layer={}, layer_size={}", self.side, self.layer, self.layer_size );

        if let Some( iterations ) = self.iterations {
            if iterations == 0 { return None }
            self.iterations = Some( iterations - 1 );
        }

        match self.side {
            1 => {
                // println!( "+Y x={} z={}", self.x, self.z );

                if self.z == border + 1 {
                    if self.x == border - 1 {
                        self.y = border;
                        self.z = -border;
                        self.side += 1;

                        self.update_iterations( 1 );
                        return self.next();
                    } else {
                        self.z = -border + 1;
                        self.x += 1;
                    }
                } else {
                    self.z += 1;

                    if self.x == self.layer_edge {
                        self.x = 0;
                    }
                }

                return Some( (self.x, self.layer_edge / 2, self.z - 1) );
            }

            2 => {
                // println!( "+X y={} z={}", self.y, self.z );

                if self.z == border + 1 {
                    if self.y == -border + 1 {
                        self.x = border;
                        self.z = -border;
                        self.side += 1;

                        self.update_iterations( 1 );
                        return self.next();
                    } else {
                        self.z = -border + 1;
                        self.y -= 1;
                    }
                } else {
                    self.z += 1;

                    if self.y == self.layer_edge {
                        self.y = 0;
                    }
                }

                return Some( (self.layer_edge / 2, self.y, self.z - 1) );
            }

            3 => {
                // println!( "-Y x={} z={}", self.x, self.z );

                if self.z == border + 1 {
                    if self.x == -border + 1 {
                        self.y = -border;
                        self.z = -border;
                        self.side += 1;

                        self.update_iterations( 1 );
                        return self.next();
                    } else {
                        self.z = -border + 1;
                        self.x -= 1;
                    }
                } else {
                    self.z += 1;

                    if self.x == self.layer_edge {
                        self.x = 0;
                    }
                }

                return Some( (self.x, self.layer_edge / -2, self.z - 1) );
            }

            4 => {
                // println!( "-X y={} z={}", self.y, self.z );

                if self.z == border + 1 {
                    if self.y == border - 1 {
                        self.y = border - 1;
                        self.x = -border + 1;
                        self.side += 1;

                        self.update_iterations( 1 );
                        return self.next();
                    } else {
                        self.z = -border + 1;
                        self.y += 1;
                    }
                } else {
                    self.z += 1;

                    if self.y == self.layer_edge {
                        self.y = 0;
                    }
                }

                return Some( (self.layer_edge / -2, self.y, self.z - 1) );
            }

            5 => {
                // println!( "-Z x={} y={}", self.x, self.y );

                if self.x == border {
                    if self.y == -border + 1 {
                        // println!( "-Z: {} {border} {}", self.layer_edge, -border + 1 );

                        self.y = border - 1;
                        self.x = -border + 1;
                        self.side += 1;

                        self.update_iterations( 1 );
                        return self.next();
                    } else {
                        self.x = -border + 2;
                        self.y -= 1;
                    }
                } else {
                    self.x += 1;

                    if self.y == self.layer_edge {
                        self.y = 0;
                    }
                }

                return Some( (self.x - 1, self.y, self.layer_edge / -2) );
            }

            6 => {
                // println!( "+Z x={} y={}", self.x, self.y );

                if self.x == border {
                    if self.y == -border + 1 {
                        self.setup_next_layer();
                        self.update_iterations( 1 );

                        return self.next();
                    } else {
                        self.x = -border + 2;
                        self.y -= 1;
                    }
                } else {
                    self.x += 1;

                    if self.y == self.layer_edge {
                        self.y = 0;
                    }
                }

                return Some( (self.x - 1, self.y, self.layer_edge / 2) );
            }

            0 => {
                self.setup_next_layer();
                return Some( (0, 0, 0) );
            }

            _ => unreachable!( "Side cannot be out of 0-5 scope" )
        }
    }
}
