use cgmath::Vector3;
use vulkanalia::prelude::v1_0::*;
use vulkanalia::vk;
use crate::{
    structure_tests::tester::{ WORLD_X, WORLD_Y, WORLD_Z },
    rendering::{
        model::Model,
        renderer::Renderer,
        vertex::{ Renderable, RendererModelDescriptions, Vec3 }
    },
};
use super::{
    voxel_vertices::{ VoxelVertex, VOXEL_INDICES, VOXEL_VERTICES },
    world_holder::{ Color, Voxel, WorldHolder },
};


pub struct WorldRenderer {
    pub model: Model<VoxelVertex>
}

impl WorldRenderer {
    pub fn new( renderer:&Renderer ) -> Self {
        Self {
            model: unsafe {
                Model::<VoxelVertex>::new( renderer, VOXEL_VERTICES.to_vec(), VOXEL_INDICES.to_vec() ).unwrap()
            },
        }
    }

    pub fn update_instances_buffer( &mut self, renderer:&Renderer, world_holder:&impl WorldHolder ) {
        println!( "Getting voxels..." );

        // let mut instances = world_holder.get_all_voxels().iter().map( |(x, y, z, v)| {
        let mut instances = world_holder.get_all_visible_voxels_from( (0, WORLD_Y, 0) ).iter().map( |(x, y, z, v)| {
            VoxelInstance {
                translate: Vector3::new(
                    *x as f32,
                    // -((WORLD_Y / 2) as f32) + *y as f32,
                    *y as f32,
                    *z as f32,
                ),
                color: (*v._common_data._color).clone(),
            }
        } ).collect::<Vec<VoxelInstance>>();

        println!( "instances_to_render={}", instances.len() );

        instances.push( VoxelInstance {
            translate: Vector3::new( -1.0, 20.0, -1.0 ),
            color: Color { red:255, green:255, blue:50 },
        } );

        // instances.push( VoxelInstance {
        //     translate: Vector3::new( 0.0, 4.0, 5.0 ),
        //     color: Color { red:255, green:0, blue:50 },
        // } );

        println!( "{:?}", world_holder.get_voxel( 1, 3, 2 ) );

        unsafe{ self.model.update_instances_buffer( renderer, instances ).unwrap() };
    }
}

impl Renderable for WorldRenderer {
    unsafe fn render( &self, device:&Device, command_buffer:vk::CommandBuffer ) {
        self.model.render( device, command_buffer );
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct VoxelInstance {
    pub translate: Vec3,
    pub color: Color,
}

impl RendererModelDescriptions for Voxel {
    fn binding_description() -> vk::VertexInputBindingDescription {
        vk::VertexInputBindingDescription::builder()
            .binding( 0 )
            .stride( size_of::<VoxelVertex>() as u32 )
            .input_rate( vk::VertexInputRate::VERTEX )
            .build()
    }

    fn attribute_description() -> Vec<vk::VertexInputAttributeDescription> {
        let pos = vk::VertexInputAttributeDescription::builder()
            .binding( 0 )
            .location( 0 )
            .format( vk::Format::R32G32B32_SFLOAT )
            .offset( 0 )
            .build();

        let normal = vk::VertexInputAttributeDescription::builder()
            .binding( 0 )
            .location( 1 )
            .format( vk::Format::R32G32B32_SFLOAT )
            .offset( (size_of::<Vec3>() + size_of::<Vec3>()) as u32 )
            .build();

        vec![ pos, normal ]
    }

    fn instances_binding_description() -> vk::VertexInputBindingDescription {
        vk::VertexInputBindingDescription::builder()
            .binding( 1 )
            .stride( size_of::<VoxelInstance>() as u32 )
            .input_rate( vk::VertexInputRate::INSTANCE )
            .build()
    }

    fn instances_attribute_description() -> Vec<vk::VertexInputAttributeDescription> {
        let pos = vk::VertexInputAttributeDescription::builder()
            .binding( 1 )
            .location( 2 )
            .format( vk::Format::R32G32B32_SFLOAT )
            // .format( vk::Format::R32G32B32A32_SFLOAT )
            .offset( 0 )
            .build();

        let color = vk::VertexInputAttributeDescription::builder()
            .binding( 1 )
            .location( 3 )
            .format( vk::Format::R8G8B8_UNORM )
            .offset( size_of::<Vec3>() as u32 )
            .build();

        vec![ pos, color ]
    }
}
