use bevy::{
    core_pipeline::bloom::BloomSettings,
    prelude::*,
    render::{
        settings::{RenderCreation, WgpuFeatures, WgpuSettings},
        RenderPlugin,
    },
};
use bevy_ray_tracing::{RTQuad, RTSphere, RayTracingPlugin, RayTracingSettings};
use shared::{DebugText, FreeCam, SharedPlugin};

fn main() {
    let mut app = App::new();
    app.add_plugins((
        DefaultPlugins.set(RenderPlugin {
            render_creation: RenderCreation::Automatic(WgpuSettings {
                features: WgpuFeatures::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES,
                // | WgpuFeatures::RAY_QUERY
                // | WgpuFeatures::RAY_TRACING_ACCELERATION_STRUCTURE,
                ..default()
            }),
            ..default()
        }),
        RayTracingPlugin,
        SharedPlugin,
    ));

    app.add_systems(Startup, setup);
    app.run();
}

fn setup(mut materials: ResMut<Assets<StandardMaterial>>, mut commands: Commands) {
    commands.spawn((
        TextBundle {
            style: Style::default(),
            text: Text {
                sections: vec!["FPS: 0.0\nMs: 0.0".into()],
                ..Default::default()
            },
            ..Default::default()
        },
        DebugText,
    ));

    commands.spawn((
        Camera3dBundle {
            camera: Camera {
                hdr: true,
                ..default()
            },
            transform: Transform::from_xyz(0.0, 0.0, 1.1),
            ..default()
        },
        RayTracingSettings {
            bounces: 10,
            samples: 2,
            sky: Vec3::splat(0.5),
        },
        BloomSettings::default(),
        FreeCam::default(),
    ));

    let white = materials.add(StandardMaterial {
        base_color: Color::rgb(1.0, 1.0, 1.0),
        ..default()
    });
    let red = materials.add(StandardMaterial {
        base_color: Color::BLACK,
        emissive: Color::rgb(10.0, 0.0, 0.0),
        ..default()
    });

    commands.spawn((
        RTSphere { radius: 0.25 },
        white.clone(),
        TransformBundle {
            local: Transform::from_xyz(0.0, 0.0, 0.0),
            ..default()
        },
    ));

    commands.spawn((
        RTSphere { radius: 0.25 },
        red.clone(),
        TransformBundle {
            local: Transform::from_xyz(1.0, 0.75, 0.0),
            ..default()
        },
    ));
}
