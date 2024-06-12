use bevy::asset::embedded_asset;
use bevy::prelude::*;

use crate::water::WaterMaterial;
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

#[derive(Clone, Debug, AsBindGroup, Asset, Reflect)]
#[uniform(100, UnderwaterExtensionUniform)]
pub struct UnderwaterExtension {
  pub water_plane: Vec4,
  pub water_color: Vec4,
  pub light_dir: Vec4,

    #[texture(101)]
    #[sampler(102)]
    pub caustics_texture: Handle<Image>,
  // pub light: Vec4,
}

#[derive(Clone, Default, ShaderType)]
struct UnderwaterExtensionUniform {
  water_plane: Vec4,
  water_color: Vec4,
  light_dir: Vec4,
  // light: Vec4,
}

impl AsBindGroupShaderType<UnderwaterExtensionUniform> for UnderwaterExtension {
  fn as_bind_group_shader_type(&self, _images: &RenderAssets<Image>) -> UnderwaterExtensionUniform {
    UnderwaterExtensionUniform {
      water_plane: self.water_plane,
      water_color: self.water_color,
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
