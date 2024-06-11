use bevy::prelude::*;
use bevy::asset::embedded_asset;

use bevy::pbr::{ExtendedMaterial, MaterialExtension};
use bevy::render::{
  renderer::RenderDevice,
  render_asset::RenderAssets,
  render_resource::{AsBindGroup, ShaderRef, ShaderType, AsBindGroupShaderType, BindGroupLayout, UnpreparedBindGroup, AsBindGroupError, BindGroupLayoutEntry},
  texture::FallbackImage,
};
use crate::water::WaterMaterial;

#[derive(Clone, Asset, Reflect, Default)]
pub struct WaterBindMaterial(pub WaterMaterial);

impl AsBindGroup for WaterBindMaterial {
  type Data = <WaterMaterial as AsBindGroup>::Data;
fn unprepared_bind_group(
        &self,
        layout: &BindGroupLayout,
        render_device: &RenderDevice,
        images: &RenderAssets<Image>,
        fallback_image: &FallbackImage
    ) -> Result<UnpreparedBindGroup<Self::Data>, AsBindGroupError> {
  self.0.unprepared_bind_group(layout, render_device, images, fallback_image)
}
fn bind_group_layout_entries(
        render_device: &RenderDevice
    ) -> Vec<BindGroupLayoutEntry>
       where Self: Sized {
  WaterMaterial::bind_group_layout_entries(render_device)

}
}

impl MaterialExtension for WaterBindMaterial {
}

pub struct CausticsPlugin;

pub type CausticsWaterMaterial = ExtendedMaterial<CausticsMaterial, WaterBindMaterial>;

impl Plugin for CausticsPlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "water", "caustics.wgsl");
        app.add_plugins(MaterialPlugin::<CausticsWaterMaterial>::default());
    }
}

#[derive(Clone, Debug, AsBindGroup, Asset, Reflect)]
#[uniform(0, CausticsMaterialUniform)]
pub struct CausticsMaterial {
    pub plane: Vec4,
    pub light: Vec4,
}

#[derive(Clone, Default, ShaderType)]
struct CausticsMaterialUniform {
    plane: Vec4,
    light: Vec4,
}

impl AsBindGroupShaderType<CausticsMaterialUniform> for CausticsMaterial {
  fn as_bind_group_shader_type(&self, _images: &RenderAssets<Image>) -> CausticsMaterialUniform {
    CausticsMaterialUniform {
      plane: self.plane,
      light: self.light,
    }
  }
}


impl Material for CausticsMaterial {
  fn vertex_shader() -> ShaderRef {
      "embedded://bevy_water/caustics.wgsl".into()
  }

  fn fragment_shader() -> ShaderRef {
      "embedded://bevy_water/caustics.wgsl".into()
  }
}
