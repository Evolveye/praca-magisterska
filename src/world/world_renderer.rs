use vulkanalia::prelude::v1_0::*;
use vulkanalia::vk;
use crate::rendering::{
    model_strip::ModelStrip, renderer::Renderer, vertex::{ Renderable, RendererModelDescriptions, Vec3 }
};
use super::{
    voxel_vertices::{ VoxelVertex, VOXEL_SIDE_VERTICES },
    world_holder::{ Color, Voxel, VoxelSide },
};


pub struct WorldRenderer {
    pub model: ModelStrip<VoxelVertex>,
    // pub model: Model<VoxelVertex>,
}

impl WorldRenderer {
    pub fn new( renderer:&Renderer ) -> Self {
        Self {
            model: unsafe {
                ModelStrip::<VoxelVertex>::new( renderer, VOXEL_SIDE_VERTICES.to_vec() ).unwrap()
                // Model::<VoxelVertex>::new( renderer, VOXEL_SIDE_VERTICES.to_vec(), VOXEL_SIDE_INDICES.to_vec() ).unwrap()
            },
        }
    }

    pub fn update_instances_buffer( &mut self, renderer:&Renderer, renderables:Vec<VoxelSide> ) {
        unsafe{ self.model.update_instances_buffer( renderer, renderables ).unwrap() };
    }
}

impl Renderable for WorldRenderer {
    unsafe fn render( &self, device:&Device, command_buffer:vk::CommandBuffer ) {
        self.model.render( device, command_buffer );
    }
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
            .stride( size_of::<VoxelSide>() as u32 )
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

        let direction = vk::VertexInputAttributeDescription::builder()
            .binding( 1 )
            .location( 4 )
            .format( vk::Format::R8_UINT )
            // .format( vk::Format::R32G32B32A32_SFLOAT )
            .offset( size_of::<Vec3>() as u32 + size_of::<Color>() as u32 )
            .build();

        vec![ pos, color, direction ]
    }
}
