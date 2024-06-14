use bevy::asset::embedded_asset;
use bevy::prelude::*;

use crate::water::WaterMaterial;
use bevy::pbr::{ExtendedMaterial, MaterialExtension, MaterialExtensionPipeline, MaterialExtensionKey, MaterialPipeline, MaterialPipelineKey};
use bevy::render::{
    mesh::MeshVertexBufferLayout,
  render_asset::RenderAssets,
  render_resource::{
    AsBindGroup, AsBindGroupError, AsBindGroupShaderType, BindGroupLayout, BindGroupLayoutEntry,
    ShaderRef, ShaderType, UnpreparedBindGroup, RenderPipelineDescriptor,SpecializedMeshPipelineError
  },
  renderer::RenderDevice,
  texture::FallbackImage,
};
use crate::water::underwater::*;

/// We bind all the same stuff as WaterMaterial, but we don't draw it like
/// WaterMaterial.
#[derive(Clone, Asset, Reflect, Default)]
pub struct WaterBindMaterial(pub WaterMaterial);

impl AsBindGroup for WaterBindMaterial {
  type Data = <WaterMaterial as AsBindGroup>::Data;
  fn unprepared_bind_group(
    &self,
    layout: &BindGroupLayout,
    render_device: &RenderDevice,
    images: &RenderAssets<Image>,
    fallback_image: &FallbackImage,
  ) -> Result<UnpreparedBindGroup<Self::Data>, AsBindGroupError> {
    self
      .0
      .unprepared_bind_group(layout, render_device, images, fallback_image)
  }
  fn bind_group_layout_entries(render_device: &RenderDevice) -> Vec<BindGroupLayoutEntry>
  where
    Self: Sized,
  {
    WaterMaterial::bind_group_layout_entries(render_device)
  }
}

impl MaterialExtension for WaterBindMaterial {}

pub struct CausticsPlugin;

pub type CausticsWaterMaterial = ExtendedMaterial<CausticsMaterial, WaterBindMaterial>;

#[derive(Resource)]
struct CausticsFunctions(Handle<Shader>);

impl Plugin for CausticsPlugin {
  fn build(&self, app: &mut App) {
    // app
    //       .register_type::<CausticsMaterial>()
    //       .register_type::<CausticsWaterMaterial>()
    //       .register_type::<WaterBindMaterial>()
    //       .register_type::<UnderwaterExtension>()
    //       .register_type::<UnderwaterMaterial>();
    embedded_asset!(app, "water", "caustics_functions.wgsl");
    embedded_asset!(app, "water", "caustics.wgsl");
    app.add_plugins(MaterialPlugin::<CausticsWaterMaterial>::default());
    embedded_asset!(app, "water", "underwater.wgsl");
    app.add_plugins(MaterialPlugin::<UnderwaterMaterial>::default());

    let asset_server = app.world.resource::<AssetServer>();
    let caustics_functions = asset_server.load::<Shader>("embedded://bevy_water/caustics_functions.wgsl");
    assert!(caustics_functions.is_strong());
    app.insert_resource(CausticsFunctions(caustics_functions));
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

    fn specialize(
        // _pipeline: &MaterialExtensionPipeline,
        _pipeline: &MaterialPipeline<CausticsMaterial>,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayout,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        descriptor.primitive.cull_mode = None;
        Ok(())
    }

}
