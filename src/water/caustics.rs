use bevy::prelude::*;
use bevy::asset::embedded_asset;
use bevy::render::render_resource::{AsBindGroup, ShaderRef};

pub struct CausticsPlugin;

impl Plugin for CausticsPlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "water", "caustics.wgsl");
        app.add_plugins(MaterialPlugin::<CausticsMaterial>::default());
    }
}

#[derive(Clone, Debug, AsBindGroup, Asset, Reflect)]
pub struct CausticsMaterial {
    pub plane: Vec4,
    pub light: Vec4,
}

impl Material for CausticsMaterial {
  fn vertex_shader() -> ShaderRef {
      "embedded://bevy_water/caustics.wgsl".into()
  }

  fn fragment_shader() -> ShaderRef {
      "embedded://bevy_water/caustics.wgsl".into()
  }
}
