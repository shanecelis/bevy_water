use bevy::prelude::*;

use crate::water::WaterMaterialUniform;
use bevy::pbr::{ExtendedMaterial, MaterialExtension, MaterialExtensionPipeline, MaterialExtensionKey};
use bevy::render::{
    mesh::MeshVertexBufferLayout,
  render_asset::RenderAssets,
  render_resource::{AsBindGroup, AsBindGroupShaderType, ShaderRef, ShaderType, RenderPipelineDescriptor, SpecializedMeshPipelineError, CompareFunction},
};

pub type CausticsParallaxMaterial = ExtendedMaterial<StandardMaterial, CausticsParallaxExtension>;

#[derive(Clone, AsBindGroup, Asset, Reflect)]
#[uniform(200, CausticsExtensionUniform)]
pub struct CausticsParallaxExtension {
  // #[uniform(203)]
  pub water_world_to_uv: Mat4,
  pub water_plane: Vec4,
  pub light_dir: Vec4,

  // #[uniform(100)]
  // pub water: WaterMaterialUniform,
}

#[derive(Clone, Default, ShaderType)]
struct CausticsExtensionUniform {
  water_world_to_uv: Mat4,
  water_plane: Vec4,
  light_dir: Vec4,
//   water: WaterMaterialUniform,
}

impl AsBindGroupShaderType<CausticsExtensionUniform> for CausticsParallaxExtension {
  fn as_bind_group_shader_type(&self, _images: &RenderAssets<Image>) -> CausticsExtensionUniform {
    CausticsExtensionUniform {
      water_world_to_uv: self.water_world_to_uv,
      water_plane: self.water_plane,
      // water_color: self.water_color.rgba_linear_to_vec4(),
      light_dir: self.light_dir,
      // water: self.water.clone(),
      // blah: 0.0,
    }
  }
}

impl MaterialExtension for CausticsParallaxExtension {
  fn vertex_shader() -> ShaderRef {
    "embedded://bevy_water/caustics_vertex.wgsl".into()
  }

  fn fragment_shader() -> ShaderRef {
    "embedded://bevy_water/caustics_fragment.wgsl".into()
  }

    fn specialize(
        _pipeline: &MaterialExtensionPipeline,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayout,
        _key: MaterialExtensionKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        // if let Some(label) = &mut descriptor.label {
        //     *label = format!("decal_{}", *label).into();
        // }
        if let Some(ref mut depth) = &mut descriptor.depth_stencil {
            depth.depth_compare = CompareFunction::Always;
        }

        Ok(())
    }
}
