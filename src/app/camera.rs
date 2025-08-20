use cgmath::{ vec3, Deg, InnerSpace };

type Vec2 = cgmath::Vector2<f32>;
type Vec3 = cgmath::Vector3<f32>;
type Vec4 = cgmath::Vector4<f32>;
type Point = cgmath::Point3<f32>;
type Mat4 = cgmath::Matrix4<f32>;

static CORRECTION:Mat4 = Mat4::new(
    1.0,  0.0,       0.0, 0.0,
    0.0, -1.0,       0.0, 0.0,
    0.0,  0.0, 1.0 / 2.0, 0.0,
    0.0,  0.0, 1.0 / 2.0, 1.0,
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
}

impl Camera {
    pub fn new( position:Point, rotation:Vec2, width:u32, height:u32 ) -> Self {
        let fov_y = Deg( 45.0 );
        let near = 0.1;
        let far = 1000.0;
        let proj_matrix = CORRECTION * cgmath::perspective(
            fov_y,
            width as f32 / height as f32,
            near,
            far,
        );

        let view_matrix = Self::get_view_mat( position, rotation );

        Self {
            view_matrix,
            proj_matrix,
            frustum: Frustum::from_view_proj( view_matrix, proj_matrix ),

            fov: fov_y,
            width,
            height,
            near,
            far,
        }
    }

    pub fn update_view( &mut self, position:Point, rotation:Vec2 ) {
        self.view_matrix = Self::get_view_mat( position, rotation );
        self.frustum = Frustum::from_view_proj( self.view_matrix, self.proj_matrix )
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
}



#[allow(dead_code)]
pub struct Frustum {
    planes: [Plane; 6],
}

impl Frustum {
    fn from_view_proj( view:Mat4, proj:Mat4 ) -> Self {
        let m = view * proj;

        let planes = [
            Plane::from_vec4( m.w + m.x ), // left
            Plane::from_vec4( m.w - m.x ), // right
            Plane::from_vec4( m.w + m.y ), // bottom
            Plane::from_vec4( m.w - m.y ), // top
            Plane::from_vec4( m.w + m.z ), // near
            Plane::from_vec4( m.w - m.z ), // far
        ];

        Frustum { planes }
    }

    fn aabb_visible( &self, min:Vec3, max:Vec3 ) -> bool {
        for plane in &self.planes {
            // Most "negative" point
            let p = vec3(
                if plane.normal.x >= 0.0 { min.x } else { max.x },
                if plane.normal.y >= 0.0 { min.y } else { max.y },
                if plane.normal.z >= 0.0 { min.z } else { max.z },
            );

            if plane.normal.dot( p ) + plane.d < 0.0 {
                return false;
            }
        }

        true
    }
}



struct Plane {
    normal: Vec3,
    d: f32,
}

impl Plane {
    fn from_vec4( v:Vec4 ) -> Self {
        let normal = Vec3::new( v.x, v.y, v.z );
        let length = normal.magnitude();

        Plane {
            normal: normal / length,
            d: v.w / length,
        }
    }

    fn normalize( mut self ) -> Self {
        let len = self.normal.magnitude();
        self.normal /= len;
        self.d /= len;
        self
    }
}
