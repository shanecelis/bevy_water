#[cfg(feature = "depth_prepass")]
use bevy::core_pipeline::prepass::DepthPrepass;

use bevy::pbr::wireframe::{Wireframe, WireframePlugin};
use bevy::pbr::NotShadowCaster;
use bevy::{input::common_conditions, prelude::*};

#[cfg(feature = "atmosphere")]
use bevy_spectator::*;

use bevy_water::material::{StandardWaterMaterial, WaterMaterial};
use bevy_water::*;

const CUBE_SIZE: f32 = 10.0;

fn main() {
  let mut app = App::new();

  app
    .add_plugins(DefaultPlugins)
    .insert_resource(WaterSettings {
        amplitude: 1.0,
      spawn_tiles: None,
      ..default()
    })
    .add_plugins(WaterPlugin)
    // Wireframe
    .add_plugins(WireframePlugin)
    .add_systems(Startup, setup)
    .add_systems(
      Update,
      toggle_wireframe.run_if(common_conditions::input_just_pressed(KeyCode::KeyR)),
    );

  #[cfg(feature = "atmosphere")]
  app.add_plugins(SpectatorPlugin); // Simple movement for this example

  app.run();
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
  mut ground_materials: ResMut<Assets<StandardMaterial>>,
) {
  // Mesh for water.
  // let mesh: Handle<Mesh> = meshes.add(shape::Cube { size: CUBE_SIZE });
  let mesh: Handle<Mesh> = meshes.add(Mesh::from(shape::Plane {
        size: CUBE_SIZE,
      subdivisions: 10,
        ..default()
      }));
  // Water material.
  let material = materials.add(StandardWaterMaterial {
    base: default(),
    extension: WaterMaterial {
      amplitude: settings.amplitude,
      coord_scale: Vec2::new(2.0, 2.0),
      ..default()
    },
  });

  commands.spawn((
    Name::new("Water world"),
    MaterialMeshBundle {
      mesh,
      material,
      transform: Transform::from_xyz(0.0, 0.0, 0.0),
      ..default()
    },
    NotShadowCaster,
  ));

  commands.spawn((
    Name::new("Ground"),
    MaterialMeshBundle {
      mesh: meshes.add(Mesh::from(shape::Plane {
        size: CUBE_SIZE,
        ..default()
      })),
      material: ground_materials.add(Color::WHITE),
      transform: Transform::from_xyz(0.0, -10.0, 0.0),
      ..default()
    },
    NotShadowCaster,
  ));

  // light
  commands.spawn(PointLightBundle {
    transform: Transform::from_xyz(4.0, CUBE_SIZE + 8.0, 4.0),
    point_light: PointLight {
      intensity: 1600.0, // lumens - roughly a 100W non-halogen incandescent bulb
      shadows_enabled: true,
      ..default()
    },
    ..default()
  });

  // camera
  let mut cam = commands.spawn((Camera3dBundle {
    transform: Transform::from_xyz(-40.0, CUBE_SIZE + 5.0, 0.0)
      .looking_at(Vec3::new(0., 0., 0.), Vec3::Y),
    ..default()
  },));

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
