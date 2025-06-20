use std::num::Wrapping;

const GRAD3: [(i8, i8, i8); 12] = [
    (1,1,0), (-1,1,0), (1,-1,0), (-1,-1,0),
    (1,0,1), (-1,0,1), (1,0,-1), (-1,0,-1),
    (0,1,1), (0,-1,1), (0,1,-1), (0,-1,-1)
];

const F3: f64 = 1.0 / 3.0;
const G3: f64 = 1.0 / 6.0;

pub struct SimplexNoise {
    perm: [u8; 512],
}

#[allow(dead_code)]
impl SimplexNoise {
    pub fn new(seed: u32) -> Self {
        let mut perm = [0u8; 512];
        let mut p = [0u8; 256];

        for i in 0..256 {
            p[i] = i as u8;
        }

        let mut seed = Wrapping(seed);
        for i in (1..256).rev() {
            seed *= Wrapping(1664525u32);
            seed += Wrapping(1013904223u32);
            let j = (seed.0 + 31) as usize % (i + 1);
            p.swap(i, j);
        }

        for i in 0..512 {
            perm[i] = p[i & 255];
        }

        Self { perm }
    }

    pub fn noise3d( &self, x:f64, y:f64, z:f64 ) -> f64 {
        let s = (x + y + z) * F3;
        let i = (x + s).floor() as isize;
        let j = (y + s).floor() as isize;
        let k = (z + s).floor() as isize;
        let t = ((i + j + k) as f64) * G3;
        let x0 = x - (i as f64 - t);
        let y0 = y - (j as f64 - t);
        let z0 = z - (k as f64 - t);

        let (i1, j1, k1, i2, j2, k2) = if x0 >= y0 {
            if y0 >= z0 { (1, 0, 0, 1, 1, 0) }
            else if x0 >= z0 { (1, 0, 0, 1, 0, 1) }
            else { (0, 0, 1, 1, 0, 1) }
        } else {
            if y0 < z0 { (0, 0, 1, 0, 1, 1) }
            else if x0 < z0 { (0, 1, 0, 0, 1, 1) }
            else { (0, 1, 0, 1, 1, 0) }
        };

        let x1 = x0 - i1 as f64 + G3;
        let y1 = y0 - j1 as f64 + G3;
        let z1 = z0 - k1 as f64 + G3;
        let x2 = x0 - i2 as f64 + 2.0 * G3;
        let y2 = y0 - j2 as f64 + 2.0 * G3;
        let z2 = z0 - k2 as f64 + 2.0 * G3;
        let x3 = x0 - 1.0 + 3.0 * G3;
        let y3 = y0 - 1.0 + 3.0 * G3;
        let z3 = z0 - 1.0 + 3.0 * G3;

        let ii = (i & 255) as usize;
        let jj = (j & 255) as usize;
        let kk = (k & 255) as usize;

        let gi0 = self.perm[ ii      + self.perm[ jj      + self.perm[ kk     ] as usize] as usize] as usize % 12;
        let gi1 = self.perm[ ii + i1 + self.perm[ jj + j1 + self.perm[ kk + k1] as usize] as usize] as usize % 12;
        let gi2 = self.perm[ ii + i2 + self.perm[ jj + j2 + self.perm[ kk + k2] as usize] as usize] as usize % 12;
        let gi3 = self.perm[ ii + 1  + self.perm[ jj + 1  + self.perm[ kk + 1 ] as usize] as usize] as usize % 12;

        let mut t0 = 0.6 - x0*x0 - y0*y0 - z0*z0;
        let n0 = if t0 < 0.0 { 0.0 } else {
            t0 *= t0;
            t0 * t0 * SimplexNoise::dot( GRAD3[ gi0 ], x0, y0, z0 )
        };

        let mut t1 = 0.6 - x1*x1 - y1*y1 - z1*z1;
        let n1 = if t1 < 0.0 { 0.0 } else {
            t1 *= t1;
            t1 * t1 * SimplexNoise::dot( GRAD3[ gi1 ], x1, y1, z1 )
        };

        let mut t2 = 0.6 - x2*x2 - y2*y2 - z2*z2;
        let n2 = if t2 < 0.0 { 0.0 } else {
            t2 *= t2;
            t2 * t2 * SimplexNoise::dot( GRAD3[ gi2 ], x2, y2, z2 )
        };

        let mut t3 = 0.6 - x3*x3 - y3*y3 - z3*z3;
        let n3 = if t3 < 0.0 { 0.0 } else {
            t3 *= t3;
            t3 * t3 * SimplexNoise::dot( GRAD3[ gi3 ], x3, y3, z3 )
        };

        32.0 * (n0 + n1 + n2 + n3)
    }

    pub fn noise3d_with_gradient( &self, x:f64, y:f64, z:f64) -> (f64, (f64, f64, f64)) {
        let s = (x + y + z) * F3;
        let i = (x + s).floor() as isize;
        let j = (y + s).floor() as isize;
        let k = (z + s).floor() as isize;
        let t = ((i + j + k) as f64) * G3;
        let x0 = x - (i as f64 - t);
        let y0 = y - (j as f64 - t);
        let z0 = z - (k as f64 - t);

        let (i1, j1, k1, i2, j2, k2) = if x0 >= y0 {
            if y0 >= z0 { (1, 0, 0, 1, 1, 0) }
            else if x0 >= z0 { (1, 0, 0, 1, 0, 1) }
            else { (0, 0, 1, 1, 0, 1) }
        } else {
            if y0 < z0 { (0, 0, 1, 0, 1, 1) }
            else if x0 < z0 { (0, 1, 0, 0, 1, 1) }
            else { (0, 1, 0, 1, 1, 0) }
        };

        let x1 = x0 - i1 as f64 + G3;
        let y1 = y0 - j1 as f64 + G3;
        let z1 = z0 - k1 as f64 + G3;
        let x2 = x0 - i2 as f64 + 2.0 * G3;
        let y2 = y0 - j2 as f64 + 2.0 * G3;
        let z2 = z0 - k2 as f64 + 2.0 * G3;
        let x3 = x0 - 1.0 + 3.0 * G3;
        let y3 = y0 - 1.0 + 3.0 * G3;
        let z3 = z0 - 1.0 + 3.0 * G3;

        let ii = (i & 255) as usize;
        let jj = (j & 255) as usize;
        let kk = (k & 255) as usize;

        let gi0 = self.perm[ ii      + self.perm[ jj      + self.perm[ kk     ] as usize] as usize] as usize % 12;
        let gi1 = self.perm[ ii + i1 + self.perm[ jj + j1 + self.perm[ kk + k1] as usize] as usize] as usize % 12;
        let gi2 = self.perm[ ii + i2 + self.perm[ jj + j2 + self.perm[ kk + k2] as usize] as usize] as usize % 12;
        let gi3 = self.perm[ ii + 1  + self.perm[ jj + 1  + self.perm[ kk + 1 ] as usize] as usize] as usize % 12;

        let mut n0 = 0.0;

        let mut dx = 0.0;
        let mut dy = 0.0;
        let mut dz = 0.0;

        for (t, x, y, z, gi) in [
            (0.6 - x0*x0 - y0*y0 - z0*z0, x0, y0, z0, gi0),
            (0.6 - x1*x1 - y1*y1 - z1*z1, x1, y1, z1, gi1),
            (0.6 - x2*x2 - y2*y2 - z2*z2, x2, y2, z2, gi2),
            (0.6 - x3*x3 - y3*y3 - z3*z3, x3, y3, z3, gi3),
        ] {
            if t > 0.0 {
                let t2 = t * t;
                let g = GRAD3[gi];

                let dot = SimplexNoise::dot(g, x, y, z);
                let t4 = t2 * t2;

                n0 += t4 * dot;

                let grad_factor = 8.0 * t2 * dot;
                dx -= grad_factor * x;
                dy -= grad_factor * y;
                dz -= grad_factor * z;
            }
        }

        (32.0 * n0, (dx, dy, dz))
    }

    pub fn noise3d_with_laplacian( &self, x:f64, y:f64, z:f64 ) -> (f64, f64) {
        let s = (x + y + z) * F3;
        let i = (x + s).floor() as isize;
        let j = (y + s).floor() as isize;
        let k = (z + s).floor() as isize;
        let t = ((i + j + k) as f64) * G3;
        let x0 = x - (i as f64 - t);
        let y0 = y - (j as f64 - t);
        let z0 = z - (k as f64 - t);

        let (i1, j1, k1, i2, j2, k2) = if x0 >= y0 {
            if y0 >= z0 { (1, 0, 0, 1, 1, 0) }
            else if x0 >= z0 { (1, 0, 0, 1, 0, 1) }
            else { (0, 0, 1, 1, 0, 1) }
        } else {
            if y0 < z0 { (0, 0, 1, 0, 1, 1) }
            else if x0 < z0 { (0, 1, 0, 0, 1, 1) }
            else { (0, 1, 0, 1, 1, 0) }
        };

        let x1 = x0 - i1 as f64 + G3;
        let y1 = y0 - j1 as f64 + G3;
        let z1 = z0 - k1 as f64 + G3;
        let x2 = x0 - i2 as f64 + 2.0 * G3;
        let y2 = y0 - j2 as f64 + 2.0 * G3;
        let z2 = z0 - k2 as f64 + 2.0 * G3;
        let x3 = x0 - 1.0 + 3.0 * G3;
        let y3 = y0 - 1.0 + 3.0 * G3;
        let z3 = z0 - 1.0 + 3.0 * G3;

        let ii = (i & 255) as usize;
        let jj = (j & 255) as usize;
        let kk = (k & 255) as usize;

        let gi0 = self.perm[ ii      + self.perm[ jj      + self.perm[ kk     ] as usize] as usize] as usize % 12;
        let gi1 = self.perm[ ii + i1 + self.perm[ jj + j1 + self.perm[ kk + k1] as usize] as usize] as usize % 12;
        let gi2 = self.perm[ ii + i2 + self.perm[ jj + j2 + self.perm[ kk + k2] as usize] as usize] as usize % 12;
        let gi3 = self.perm[ ii + 1  + self.perm[ jj + 1  + self.perm[ kk + 1 ] as usize] as usize] as usize % 12;

        let mut n = 0.0;
        let mut laplacian = 0.0;

        for (t, x, y, z, gi) in [
            (0.6 - x0*x0 - y0*y0 - z0*z0, x0, y0, z0, gi0),
            (0.6 - x1*x1 - y1*y1 - z1*z1, x1, y1, z1, gi1),
            (0.6 - x2*x2 - y2*y2 - z2*z2, x2, y2, z2, gi2),
            (0.6 - x3*x3 - y3*y3 - z3*z3, x3, y3, z3, gi3),
        ] {
            if t > 0.0 {
                let g = GRAD3[ gi ];
                let dot = SimplexNoise::dot( g, x, y, z );
                let t2 = t * t;
                let t4 = t2 * t2;

                n += t4 * dot;

                let laplacian_term = -8.0 * t2 * (dot + g.0 as f64 * x + g.1 as f64 * y + g.2 as f64 * z);
                laplacian += laplacian_term;
            }
        }

        (32.0 * n, laplacian)
    }

    #[inline(always)]
    fn dot(g: (i8, i8, i8), x: f64, y: f64, z: f64) -> f64 {
        (g.0 as f64) * x + (g.1 as f64) * y + (g.2 as f64) * z
    }
}
