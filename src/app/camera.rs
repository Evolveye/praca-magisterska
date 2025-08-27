use cgmath::{ vec3, vec4, Deg, InnerSpace, Matrix, SquareMatrix, Zero };

use crate::{ rendering::vertex::SimpleVertex, world::world::Position };

type Vec2 = cgmath::Vector2<f32>;
type Vec3 = cgmath::Vector3<f32>;
type Vec4 = cgmath::Vector4<f32>;
type Point = cgmath::Point3<f32>;
type Mat4 = cgmath::Matrix4<f32>;

pub type FrustumVertex = SimpleVertex;

static CORRECTION:Mat4 = Mat4::new(
    1.0,  0.0, 0.0, 0.0,
    0.0, -1.0, 0.0, 0.0,
    0.0,  0.0, 0.5, 0.0,
    0.0,  0.0, 0.5, 1.0,
);

#[allow(dead_code)]
pub struct Camera {
    pub view_matrix: Mat4,
    pub proj_matrix: Mat4,
    pub frustum: Frustum,

    fov: Deg<f32>,
    pub width: u32,
    pub height: u32,
    near: f32,
    far: f32,

    freezed_frustum_corners: Option<[FrustumVertex; 8]>,
}

impl Camera {
    pub fn new( position:Point, rotation:Vec2, width:u32, height:u32 ) -> Self {
        let fov_y = Deg( 45.0 );
        let near = 0.1;
        let far = 1000.0;

        let view_matrix = Self::get_view_mat( position, rotation );
        let proj_matrix = CORRECTION * cgmath::perspective(
            fov_y,
            width as f32 / height as f32,
            near,
            far,
        );

        Self {
            view_matrix,
            proj_matrix,
            frustum: Frustum::from_view_proj( view_matrix, proj_matrix ),

            fov: fov_y,
            width,
            height,
            near,
            far,

            freezed_frustum_corners: None,
        }
    }

    pub fn update_view( &mut self, position:Point, rotation:Vec2, freezed:bool ) {
        self.view_matrix = Self::get_view_mat( position, rotation );

        if freezed {
            self.freezed_frustum_corners = Some( self.get_frustum_corners() );
        } else {
            self.frustum = Frustum::from_view_proj( self.view_matrix, self.proj_matrix );
            self.freezed_frustum_corners = None;
        }

        // println!( "{position:?} {rotation:?}" );
    }

    fn get_view_mat( position:Point, rotation:Vec2 ) -> Mat4 {
        Mat4::look_at_rh(
            position,
            position + vec3(
                rotation.y.cos() * rotation.x.cos(),
                rotation.x.sin(),
                rotation.y.sin() * rotation.x.cos(),
            ),
            Vec3::unit_y(),
        )
    }

    pub fn get_frustum_corners( &self ) -> [FrustumVertex; 8] {
        // let tan_half_fov = (self.fov.0 * 0.5).tan();
        // let aspect = self.width as f32 / self.height as f32;

        // let h_near = 2.0 * tan_half_fov * self.near;
        // let w_near = h_near * aspect;

        // let h_far = 2.0 * tan_half_fov * self.far;
        // let w_far = h_far * aspect;

        // let inv = (self.proj_matrix * self.view_matrix).invert().unwrap();

        if let Some( corners ) = self.freezed_frustum_corners {
            return corners;
        }

        let inv = (self.proj_matrix * self.view_matrix).invert().unwrap();

        let ndc_corners = [
            vec4(-1.0, -1.0, 0.0, 1.0), // near bottom-left
            vec4( 1.0, -1.0, 0.0, 1.0), // near bottom-right
            vec4( 1.0,  1.0, 0.0, 1.0), // near top-right
            vec4(-1.0,  1.0, 0.0, 1.0), // near top-left

            vec4(-1.0, -1.0, 1.0, 1.0), // far bottom-left
            vec4( 1.0, -1.0, 1.0, 1.0), // far bottom-right
            vec4( 1.0,  1.0, 1.0, 1.0), // far top-right
            vec4(-1.0,  1.0, 1.0, 1.0), // far top-left
        ];

        let mut corners = [FrustumVertex { pos:vec3(0.0, 0.0, 0.0), color:vec3(0.0, 0.0, 0.0) }; 8];

        for (i, corner) in ndc_corners.iter().enumerate() {
            let v = inv * *corner;
            let pos = v.truncate() / v.w;

            corners[i] = FrustumVertex {
                pos,
                color: vec3( 1.0, 1.0, 1.0 ),
                // color: vec3( 1.0, 0.0, 0.0 ),
            }
        }

        Self::scale_frustum_corners( &mut corners, 0.99999 );

        corners
    }

    pub fn scale_frustum_corners( corners:&mut [FrustumVertex; 8], scale:f32 ) {
        let mut center = Vec3::zero();
        for c in corners.iter() {
            center += c.pos;
        }
        center /= 8.0;

        for c in corners {
            c.pos = center + (c.pos - center) * scale;
        }
    }
}



#[allow(dead_code)]
pub struct Frustum {
    planes: [Plane; 6],
}

pub enum FrustumCheck {
    Outside,
    Intersect,
    Inside,
}

impl Frustum {
    fn from_view_proj( view:Mat4, proj:Mat4 ) -> Self {
        let clip_space = (proj * view).transpose();

        let planes = [
            Plane::from_vec4( clip_space.w + clip_space.x ), // left
            Plane::from_vec4( clip_space.w - clip_space.x ), // right
            Plane::from_vec4( clip_space.w + clip_space.y ), // bottom
            Plane::from_vec4( clip_space.w - clip_space.y ), // top
            Plane::from_vec4( clip_space.w + clip_space.z ), // near
            Plane::from_vec4( clip_space.w - clip_space.z ), // far
        ];

        Self { planes }
    }

    pub fn intersects_aabb( &self, min:Position, max:Position ) -> FrustumCheck {
        let mut result = FrustumCheck::Inside;

        for plane in &self.planes {
            let positive = vec3(
                if plane.normal.x >= 0.0 { max.0 } else { min.0 },
                if plane.normal.y >= 0.0 { max.1 } else { min.1 },
                if plane.normal.z >= 0.0 { max.2 } else { min.2 },
            );

            let negative = vec3(
                if plane.normal.x >= 0.0 { min.0 } else { max.0 },
                if plane.normal.y >= 0.0 { min.1 } else { max.1 },
                if plane.normal.z >= 0.0 { min.2 } else { max.2 },
            );

            if plane.distance( positive ) < 0.0 {
                return FrustumCheck::Outside;
            }

            if plane.distance( negative ) < 0.0 {
                result = FrustumCheck::Intersect;
            }
        }

        result
    }
}



struct Plane {
    normal: Vec3,
    d: f32,
}

impl Plane {
    fn from_vec4( v:Vec4 ) -> Self {
        let normal = Vec3::new( v.x, v.y, v.z );
        let mut length = normal.magnitude();

        if length == 0.0 {
            println!( "Frustum zero division" );
            length = 1.0;
        }

        Self {
            normal: normal / length,
            d: v.w / length,
        }
    }

    pub fn distance( &self, pos:Vec3 ) -> f32 {
        self.normal.dot( pos ) + self.d
    }
}
