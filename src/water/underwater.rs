use bevy::asset::embedded_asset;
use bevy::prelude::*;

use crate::water::{WaterMaterial, WaterMaterialUniform};
use bevy::pbr::{ExtendedMaterial, MaterialExtension};
use bevy::render::{
  render_asset::RenderAssets,
  render_resource::{
    AsBindGroup, AsBindGroupError, AsBindGroupShaderType, BindGroupLayout, BindGroupLayoutEntry,
    ShaderRef, ShaderType, UnpreparedBindGroup,
  },
  renderer::RenderDevice,
  texture::FallbackImage,
};

pub type UnderwaterMaterial = ExtendedMaterial<StandardMaterial, UnderwaterExtension>;

#[derive(Clone, AsBindGroup, Asset, Reflect)]
#[uniform(200, UnderwaterExtensionUniform)]
pub struct UnderwaterExtension {
  #[uniform(100)]
  pub water: WaterMaterialUniform,
  pub water_plane: Vec4,
  pub water_color: Color,
  pub light_dir: Vec4,

    #[texture(201)]
    #[sampler(202)]
    pub caustics_texture: Handle<Image>,
  // pub light: Vec4,
}

#[derive(Clone, Default, ShaderType)]
struct UnderwaterExtensionUniform {
  water: WaterMaterialUniform,
  water_plane: Vec4,
  water_color: Vec4,
  light_dir: Vec4,
  // light: Vec4,
}

impl AsBindGroupShaderType<UnderwaterExtensionUniform> for UnderwaterExtension {
  fn as_bind_group_shader_type(&self, _images: &RenderAssets<Image>) -> UnderwaterExtensionUniform {
    UnderwaterExtensionUniform {
      water: self.water.clone(),
      water_plane: self.water_plane,
      water_color: self.water_color.rgba_linear_to_vec4(),
      light_dir: self.light_dir,
    }
  }
}

impl MaterialExtension for UnderwaterExtension {
  // fn vertex_shader() -> ShaderRef {
  //   "embedded://bevy_water/underwater.wgsl".into()
  // }

  fn fragment_shader() -> ShaderRef {
    "embedded://bevy_water/underwater.wgsl".into()
  }
}
