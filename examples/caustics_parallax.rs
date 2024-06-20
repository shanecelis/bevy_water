#[cfg(feature = "depth_prepass")]
use bevy::core_pipeline::prepass::DepthPrepass;

use bevy::pbr::wireframe::{Wireframe, WireframePlugin};
use bevy::pbr::{NotShadowCaster, NotShadowReceiver};
use bevy::render::{
    render_asset::RenderAssetUsages,
  render_resource::{Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages},
  view::RenderLayers,
};
use bevy::{input::common_conditions, prelude::*};

use bevy::pbr::ExtendedMaterial;
#[cfg(feature = "atmosphere")]
use bevy_spectator::*;

use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use bevy_water::caustics::CausticsPlugin;
use bevy_water::caustics_parallax::*;
use bevy_water::material::{StandardWaterMaterial, WaterMaterial};
use bevy_water::underwater::*;
use bevy_water::*;
use std::f32::consts::TAU;

const PLANE_SIZE: f32 = 2.0;
const PLANE_SUBDIVISIONS: u32 = 200;
const COORD_SCALE: Vec2 = Vec2::new(1.0, 1.0);
const WATER_PLANE: Vec4 = Vec4::new(0., 1., 0., 0.44);
const LIGHT: Vec4 = Vec4::new(0.66, 0.69, 0.3, 0.0);

fn main() {
  let mut app = App::new();

  app
    .add_plugins(DefaultPlugins)
    .insert_resource(WaterSettings {
      // amplitude: 0.5,
      amplitude: 0.1,
      // amplitude: 10.0,
      spawn_tiles: None,
      ..default()
    })
    .add_plugins(WaterPlugin)
    .add_plugins(CausticsPlugin)
    // Wireframe
    .add_plugins(WireframePlugin)
    .add_plugins(PanOrbitCameraPlugin)
    .add_systems(Startup, (setup, setup_caustics))
    .add_systems(
      Update,
      (toggle_wireframe.run_if(common_conditions::input_just_pressed(KeyCode::KeyR)),
        toggle_debug_visibility),
    );

  #[cfg(feature = "atmosphere")]
  app.add_plugins(SpectatorPlugin); // Simple movement for this example

  app.run();
}

fn setup_caustics(
  mut commands: Commands,
  mut meshes: ResMut<Assets<Mesh>>,
  settings: Res<WaterSettings>,
  mut caustics_materials: ResMut<Assets<CausticsParallaxMaterial>>,
  mut materials: ResMut<Assets<StandardMaterial>>,
  mut images: ResMut<Assets<Image>>,
  asset_server: Res<AssetServer>,
) {

  let water_material = WaterMaterial {
    amplitude: settings.amplitude,

    coord_scale: COORD_SCALE,
    ..default()
  };

  let size = PLANE_SIZE;
  let half_size = size / 2.0;

  let underwater_material = materials.add(StandardMaterial {
      // base_color: Color::hex("f6dcbd").unwrap(),
      base_color_texture: Some(asset_server.load("textures/tiles.jpg")),
      // emissive: Color::WHITE,
      // emissive_texture: Some(images.add(white_dot)),
      ..default()
    });
    let plane_half_size = PLANE_SIZE / 2.0;
    let mut ground_height = -plane_half_size;
  commands.spawn((
    Name::new("Ground"),
    MaterialMeshBundle {
      mesh: meshes.add(Mesh::from(shape::Plane {
        size: PLANE_SIZE,
        ..default()
      })),
      material: underwater_material.clone(),
      transform: Transform::from_xyz(0.0, ground_height, 0.0),
      ..default()
    },
    NotShadowCaster,
  ));


    ground_height = 0.0;
  commands.spawn((
    Name::new("Wall 1"),
    MaterialMeshBundle {
      mesh: meshes.add(Mesh::from(shape::Plane {
        size: PLANE_SIZE,
        ..default()
      })),
      material: underwater_material.clone(),
      transform: Transform::from_xyz(-plane_half_size, ground_height, 0.0)
        .with_rotation(Quat::from_rotation_z(-TAU / 4.0)),
      ..default()
    },
    NotShadowCaster,
  ));

  commands.spawn((
    Name::new("Wall 2"),
    MaterialMeshBundle {
      mesh: meshes.add(Mesh::from(shape::Plane {
        size: PLANE_SIZE,
        ..default()
      })),
      material: underwater_material.clone(),
      transform: Transform::from_xyz(plane_half_size, ground_height, 0.0).with_rotation(Quat::from_rotation_z(TAU / 4.0)),
      ..default()
    },
    NotShadowCaster,
  ));

  commands.spawn((
    Name::new("Wall 3"),
    MaterialMeshBundle {
      mesh: meshes.add(Mesh::from(shape::Plane {
        size: PLANE_SIZE,
        ..default()
      })),
      material: underwater_material.clone(),
      transform: Transform::from_xyz(0.0, ground_height, plane_half_size)
        .with_rotation(Quat::from_rotation_x(-TAU / 4.0)),
      ..default()
    },
    NotShadowCaster,
  ));

  commands.spawn((
    Name::new("Wall 4"),
    MaterialMeshBundle {
      mesh: meshes.add(Mesh::from(shape::Plane {
        size: PLANE_SIZE,
        ..default()
      })),
      material: underwater_material.clone(),
      transform: Transform::from_xyz(0.0, ground_height, -plane_half_size)
        .with_rotation(Quat::from_rotation_x(TAU / 4.0)),
      ..default()
    },
    NotShadowCaster,
  ));

  let mesh: Handle<Mesh> = meshes.add(Mesh::from(shape::Plane {
    size: PLANE_SIZE,
    subdivisions: PLANE_SUBDIVISIONS * 5,
    ..default()
  })
        // .with_generated_tangents()
  );
  // let mesh: Handle<Mesh> = meshes.add(shape::Cube { size: PLANE_SIZE });
  // let caustics_pass_layer = RenderLayers::layer(1);
  commands.spawn((
    Name::new("Caustics Plane".to_string()),
    MaterialMeshBundle {
      mesh,
      material: caustics_materials.add(CausticsParallaxMaterial {
        base: Color::WHITE.into(),
          extension:
          CausticsParallaxExtension {

              water_world_to_uv: Mat4::from_translation(Vec3::new(0.5, 0.0, 0.5))
                  * Mat4::from_scale(Vec3::new(1.0 / size, 1.0, 1.0 / size)),
          water_plane: Vec4::new(0.0, 1.0, 0.0, -1.0), // XXX: Why do this when you could just add max_depth = -10?
          light_dir: LIGHT,
        },
      }),
      // transform: Transform::from_xyz(0.0, 0.0, 0.0).with_rotation(Quat::from_rotation_x(0.2)),
      ..default()
    },

    NotShadowReceiver,
    NotShadowCaster,
    // caustics_pass_layer,
  ));

}

#[derive(Component)]
struct Debug;

fn toggle_debug_visibility(mut query: Query<&mut Visibility, With<Debug>>,

    keyboard: Res<ButtonInput<KeyCode>>,
                           ) {

    if keyboard.just_pressed(KeyCode::Space) {
        info!("toggling");
        for mut visibility in query.iter_mut() {
            *visibility = match *visibility {
                Visibility::Visible => Visibility::Hidden,
                Visibility::Hidden => Visibility::Visible,
                Visibility::Inherited => Visibility::Visible,
            }
        }
    }
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
  let mesh: Handle<Mesh> = meshes.add(Mesh::from(shape::Plane {
    size: PLANE_SIZE,
    subdivisions: PLANE_SUBDIVISIONS,
    ..default()
  }));
  let water_material = WaterMaterial {
    amplitude: settings.amplitude,
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
      transform: Transform::from_xyz(0.0, WATER_PLANE.w, 0.0),
      ..default()
    },
    NotShadowCaster,
  ));

  // light
  commands.spawn(PointLightBundle {
    transform: Transform::from_translation(LIGHT.xyz()),
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
      DepthPrepass,
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
