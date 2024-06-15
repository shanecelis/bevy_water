#[cfg(feature = "depth_prepass")]
use bevy::core_pipeline::prepass::DepthPrepass;

use bevy::pbr::wireframe::{Wireframe, WireframePlugin};
use bevy::pbr::NotShadowCaster;
use bevy::render::{
  render_resource::{Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages},
  view::RenderLayers,
};
use bevy::{input::common_conditions, prelude::*};

use bevy::pbr::ExtendedMaterial;
#[cfg(feature = "atmosphere")]
use bevy_spectator::*;

use bevy_inspector_egui::quick; //::AssetInspectorPlugin;
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use bevy_water::caustics::*;
use bevy_water::material::{StandardWaterMaterial, WaterMaterial};
use bevy_water::underwater::*;
use bevy_water::*;

const PLANE_SIZE: f32 = 1.0;
const PLANE_SUBDIVISIONS: u32 = 200;
const COORD_SCALE: Vec2 = Vec2::new(1.0, 1.0);

fn main() {
  let mut app = App::new();

  app
    .add_plugins(DefaultPlugins)
    .insert_resource(WaterSettings {
      amplitude: 0.1,
      // amplitude: 10.0,
      spawn_tiles: None,
      ..default()
    })
    .add_plugins(WaterPlugin)
    .add_plugins(CausticsPlugin)
    // .add_plugins(AssetInspectorPlugin::<Image>::default())
    .add_plugins(quick::WorldInspectorPlugin::new())
    // Wireframe
    .add_plugins(WireframePlugin)
    .add_plugins(PanOrbitCameraPlugin)
    .add_systems(Startup, (setup, setup_caustics))
    .add_systems(
      Update,
      toggle_wireframe.run_if(common_conditions::input_just_pressed(KeyCode::KeyR)),
    );

  #[cfg(feature = "atmosphere")]
  app.add_plugins(SpectatorPlugin); // Simple movement for this example

  app.run();
}

fn setup_caustics(
  mut commands: Commands,
  mut meshes: ResMut<Assets<Mesh>>,
  settings: Res<WaterSettings>,
  mut caustics_materials: ResMut<Assets<CausticsWaterMaterial>>,
  mut underwater_materials: ResMut<Assets<UnderwaterMaterial>>,
  mut images: ResMut<Assets<Image>>,
) {
  // let size = Extent3d {
  //   width: 512,
  //   height: 512,
  //   ..default()
  // };
  let size = Extent3d {
    width: 1024,
    height: 1024,
    ..default()
  };
  // This is the texture that will be rendered to.
  let mut image = Image {
    texture_descriptor: TextureDescriptor {
      label: "caustics".into(),
      size,
      dimension: TextureDimension::D2,
      // format: TextureFormat::Bgra8UnormSrgb,
      format: TextureFormat::Rgba32Float,
      mip_level_count: 1,
      sample_count: 1,
      usage: TextureUsages::TEXTURE_BINDING
        | TextureUsages::COPY_DST
        | TextureUsages::RENDER_ATTACHMENT,
      view_formats: &[],
    },
    ..default()
  };
  image.resize(size);
  let image_handle = images.add(image);
  info!("caustics image handle {:?}", image_handle.id());

  let water_material = WaterMaterial {
    amplitude: settings.amplitude,

    coord_scale: COORD_SCALE,
    // coord_scale: Vec2::new(256.0, 256.0),
    ..default()
  };

  let size = PLANE_SIZE;
  let half_size = size / 2.0;

  // let white_dot = Image::new_fill(Extent3d { width: 2,
  //                                            height: 2,
  //                                            ..default() },
  //                                 TextureDimension::D2,
  //                                 &[255, 255, 255, 255],
  //                                 TextureFormat::Rgba8UnormSrgb,
  //                                 RenderAssetUsages::RENDER_WORLD);
  commands.spawn((
    Name::new("Ground"),
    MaterialMeshBundle {
      mesh: meshes.add(Mesh::from(shape::Plane {
        size: PLANE_SIZE,
        ..default()
      })),
      // material: ground_materials.add(Color::WHITE),
      material: underwater_materials.add(UnderwaterMaterial {
        base: StandardMaterial {
          base_color: Color::hex("f6dcbd").unwrap(),
          // emissive: Color::WHITE,
          // emissive_texture: Some(images.add(white_dot)),
          ..default()
        },
        extension: UnderwaterExtension {
          water: water_material.clone().into(),
          water_world_to_uv: Mat4::from_translation(Vec3::new(0.5, 0.0, 0.5))
            * Mat4::from_scale(Vec3::new(1.0 / size, 1.0, 1.0 / size)),
          water_plane: Vec4::new(0.0, 1.0, 0.0, 0.0),
          water_color: Color::hex("74ccf4").unwrap(),
          light_dir: Vec4::new(0.65, 0.69, 0.3, 0.0),
          caustics_texture: image_handle.clone(),
        },
      }),
      transform: Transform::from_xyz(0.0, -1.0, 0.0), // .with_rotation(Quat::from_rotation_z(-1.0))
      // .with_rotation(Quat::from_euler(EulerRot::YZX, TAU / 4.0, -1.0, 0.0))
      ..default()
    },
    NotShadowCaster,
  ));

  let mesh: Handle<Mesh> = meshes.add(Mesh::from(shape::Plane {
    size: PLANE_SIZE,
    subdivisions: PLANE_SUBDIVISIONS * 5,
    ..default()
  }));
  // let mesh: Handle<Mesh> = meshes.add(shape::Cube { size: PLANE_SIZE });
  let caustics_pass_layer = RenderLayers::layer(1);
  commands.spawn((
    Name::new("Water world".to_string()),
    MaterialMeshBundle {
      mesh,
      material: caustics_materials.add(ExtendedMaterial {
        base: CausticsMaterial {
          plane: Vec4::new(0.0, 1.0, 0.0, -10.0), // XXX: Why do this when you could just add max_depth = -10?
          light: Vec4::new(4.0, PLANE_SIZE + 8.0, 4.0, 0.0),
        },
        extension: WaterBindMaterial(water_material),
      }),
      // transform: Transform::from_xyz(0.0, 0.0, 0.0).with_rotation(Quat::from_rotation_x(0.2)),
      ..default()
    },
    NotShadowCaster,
    caustics_pass_layer,
  ));

  commands.spawn((
    Camera3dBundle {
      camera: Camera {
        // render before the "main pass" camera
        order: -1,
        target: image_handle.clone().into(),
        // clear_color: Color::WHITE.into(),
        ..default()
      },
      // transform: Transform::from_translation(Vec3::new(0.0, 0.0, 15.0)),
      //     .looking_at(Vec3::ZERO, Vec3::Y),
      ..default()
    },
    caustics_pass_layer,
  ));
}

fn toggle_wireframe(
  mut show_wireframe: Local<bool>,
  query: Query<Entity, With<Handle<Mesh>>>,
  mut commands: Commands,
) {
  // Update flag.
  *show_wireframe = !*show_wireframe;

  for entity in query.iter() {
    let mut entity = commands.entity(entity);
    if *show_wireframe {
      entity.insert(Wireframe);
    } else {
      entity.remove::<Wireframe>();
    }
  }
}

/// Setup water.
fn setup(
  mut commands: Commands,
  settings: Res<WaterSettings>,
  mut meshes: ResMut<Assets<Mesh>>,
  mut materials: ResMut<Assets<StandardWaterMaterial>>,
  ground_materials: ResMut<Assets<StandardMaterial>>,
) {
  // Mesh for water.
  // let mesh: Handle<Mesh> = meshes.add(shape::Cube { size: PLANE_SIZE });

  let mesh: Handle<Mesh> = meshes.add(Mesh::from(shape::Plane {
    size: PLANE_SIZE,
    subdivisions: PLANE_SUBDIVISIONS,
    ..default()
  }));
  let water_material = WaterMaterial {
    // amplitude: settings.amplitude,
    coord_scale: COORD_SCALE,
    ..default()
  };
  // Water material.
  let material = materials.add(StandardWaterMaterial {
    base: default(),
    extension: water_material.clone(),
  });

  commands.spawn((
    Name::new("Water world".to_string()),
    MaterialMeshBundle {
      mesh: mesh.clone(),
      material,
      transform: Transform::from_xyz(0.0, 0.0, 0.0),
      ..default()
    },
    // caustics_materials.add(CausticsMaterial {
    //     plane: Vec4::Y,
    //     light: -Vec4::Y,
    // }),
    NotShadowCaster,
  ));

  // commands.spawn((
  //   Name::new("Ground"),
  //   MaterialMeshBundle {
  //     mesh: meshes.add(Mesh::from(shape::Plane {
  //       size: PLANE_SIZE,
  //       ..default()
  //     })),
  //     material: ground_materials.add(Color::WHITE),
  //     transform: Transform::from_xyz(0.0, -10.0, 0.0),
  //     ..default()
  //   },
  //   NotShadowCaster,
  // ));

  // light
  commands.spawn(PointLightBundle {
    transform: Transform::from_xyz(0.65, 0.69, 0.3),
    point_light: PointLight {
      intensity: 1600.0, // lumens - roughly a 100W non-halogen incandescent bulb
      shadows_enabled: true,
      ..default()
    },
    ..default()
  });

  // camera
  let mut cam = commands.spawn((
    Camera3dBundle {
      transform: Transform::from_xyz(0.0, -0.76, 4.0).looking_at(Vec3::new(0., 0., 0.), Vec3::Y),
      ..default()
    },
    PanOrbitCamera::default(),
  ));

  #[cfg(feature = "atmosphere")]
  cam.insert(Spectator);

  #[cfg(feature = "depth_prepass")]
  {
    // This will write the depth buffer to a texture that you can use in the main pass
    cam.insert(DepthPrepass);
  }
  // This is just to keep the compiler happy when not using `depth_prepass` feature.
  cam.insert(Name::new("Camera"));
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_world_to_uv() {
    let size = PLANE_SIZE;
    let half_size = size / 2.0;
    let max_point = Vec4::new(half_size, 0.0, half_size, 1.0);
    let max_point_x = Vec4::new(half_size, 0.0, -half_size, 1.0);
    let min_point = Vec4::new(-half_size, 0.0, -half_size, 1.0);
    let mat = Mat4::from_translation(Vec3::new(0.5, 0.0, 0.5))
      * Mat4::from_scale(Vec3::new(1.0 / size, 1.0, 1.0 / size));
    assert_eq!(mat * max_point, Vec4::new(1.0, 0.0, 1.0, 1.0));
    assert_eq!(mat * max_point_x, Vec4::new(1.0, 0.0, 0.0, 1.0));
    assert_eq!(mat * min_point, Vec4::new(0.0, 0.0, 0.0, 1.0));
  }

  #[test]
  fn test_world_to_uv_diffw() {
    let size = PLANE_SIZE;
    let half_size = size / 2.0;
    let max_point = Vec4::new(half_size, 0.0, half_size, 5.0);
    let max_point_x = Vec4::new(half_size, 0.0, -half_size, 4.0);
    let min_point = Vec4::new(-half_size, 0.0, -half_size, -1.0);
    let mat = Mat4::from_translation(Vec3::new(0.5, 0.0, 0.5))
      * Mat4::from_scale(Vec3::new(1.0 / size, 1.0, 1.0 / size));
    assert_eq!(mat * max_point, Vec4::new(1.0, 0.0, 1.0, 5.0));
    assert_eq!(mat * max_point_x, Vec4::new(1.0, 0.0, 0.0, 4.0));
    assert_eq!(mat * min_point, Vec4::new(0.0, 0.0, 0.0, -1.0));
  }

  #[test]
  fn test_world_to_uv2() {
    let size = PLANE_SIZE;
    let half_size = size / 2.0;
    let max_point = Vec4::new(half_size, 1.0, half_size, 1.0);
    let min_point = Vec4::new(-half_size, 2.0, -half_size, 1.0);
    let mat = Mat4::from_translation(Vec3::new(0.5, 0.0, 0.5))
      * Mat4::from_scale(Vec3::new(1.0 / size, 1.0, 1.0 / size));
    assert_eq!(mat * max_point, Vec4::new(1.0, 1.0, 1.0, 1.0));
    assert_eq!(mat * min_point, Vec4::new(0.0, 2.0, 0.0, 1.0));
  }
}
