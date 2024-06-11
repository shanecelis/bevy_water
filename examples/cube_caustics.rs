#[cfg(feature = "depth_prepass")]
use bevy::core_pipeline::prepass::DepthPrepass;

use bevy::pbr::wireframe::{Wireframe, WireframePlugin};
use bevy::pbr::NotShadowCaster;
use bevy::render::{
  render_resource::{Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages},
  view::RenderLayers,
};
use bevy::{input::common_conditions, prelude::*};

use bevy::pbr::{ExtendedMaterial, MaterialExtension};
#[cfg(feature = "atmosphere")]
use bevy_spectator::*;

use bevy_water::caustics::*;
use bevy_water::material::{StandardWaterMaterial, WaterMaterial};
use bevy_water::*;

const CUBE_SIZE: f32 = 10.0;

#[derive(Component)]
struct CausticsPass;

fn main() {
  let mut app = App::new();

  app
    .add_plugins(DefaultPlugins)
    .insert_resource(WaterSettings {
      spawn_tiles: None,
      ..default()
    })
    .add_plugins(WaterPlugin)
    .add_plugins(CausticsPlugin)
    // Wireframe
    .add_plugins(WireframePlugin)
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
  mut images: ResMut<Assets<Image>>,
) {
  let size = Extent3d {
    width: 512,
    height: 512,
    ..default()
  };
  // This is the texture that will be rendered to.
  let mut image = Image {
    texture_descriptor: TextureDescriptor {
      label: "caustics".into(),
      size,
      dimension: TextureDimension::D2,
      format: TextureFormat::Bgra8UnormSrgb,
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

  let water_material = WaterMaterial {
    amplitude: settings.amplitude,
    coord_scale: Vec2::new(256.0, 256.0),
    ..default()
  };

  let mesh: Handle<Mesh> = meshes.add(shape::Cube { size: CUBE_SIZE });
  let caustics_pass_layer = RenderLayers::layer(1);
  commands.spawn((
    Name::new("Water world".to_string()),
    MaterialMeshBundle {
      mesh,
      material: caustics_materials.add(ExtendedMaterial {
        base: CausticsMaterial {
          plane: Vec4::Y,
          light: Vec4::new(4.0, CUBE_SIZE + 8.0, 4.0, 0.0),
        },
        extension: WaterBindMaterial(water_material),
      }),
      transform: Transform::from_xyz(0.0, 0.0, 0.0).with_rotation(Quat::from_rotation_x(0.2)),
      ..default()
    },
    CausticsPass,
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
      // transform: Transform::from_translation(Vec3::new(0.0, 0.0, 15.0))
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
) {
  // Mesh for water.
  let mesh: Handle<Mesh> = meshes.add(shape::Cube { size: CUBE_SIZE });
  let water_material = WaterMaterial {
    amplitude: settings.amplitude,
    coord_scale: Vec2::new(256.0, 256.0),
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
      transform: Transform::from_xyz(0.0, 0.0, 0.0).with_rotation(Quat::from_rotation_x(0.2)),
      ..default()
    },
    // caustics_materials.add(CausticsMaterial {
    //     plane: Vec4::Y,
    //     light: -Vec4::Y,
    // }),
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
