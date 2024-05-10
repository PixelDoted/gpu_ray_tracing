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
            samples: 1,
        },
        BloomSettings::default(),
        FreeCam::default(),
    ));

    let light = materials.add(StandardMaterial {
        base_color: Color::BLACK,
        emissive: Color::rgb(2.0, 2.0, 2.0),
        ..default()
    });
    let red = materials.add(StandardMaterial {
        base_color: Color::rgb(1.0, 0.0, 0.0),
        ..default()
    });
    let green = materials.add(StandardMaterial {
        base_color: Color::rgb(0.0, 1.0, 0.0),
        ..default()
    });
    let blue = materials.add(StandardMaterial {
        base_color: Color::rgb(0.0, 0.0, 1.0),
        ..default()
    });
    let white = materials.add(StandardMaterial {
        base_color: Color::rgb(1.0, 1.0, 1.0),
        ..default()
    });
    let ball = materials.add(StandardMaterial {
        base_color: Color::rgb(1.0, 1.0, 1.0),
        perceptual_roughness: 0.0,
        metallic: 1.0,
        specular_transmission: 0.0,
        ..default()
    });

    commands.spawn((
        RTSphere { radius: 0.25 },
        ball.clone(),
        TransformBundle {
            local: Transform::from_xyz(0.0, 0.0, 0.0),
            ..default()
        },
    ));

    let scale = Vec3::ONE * 2.0;
    commands.spawn_batch([
        (
            RTQuad,
            white.clone(),
            TransformBundle {
                local: Transform::from_xyz(0.0, -1.0, 0.0).with_scale(scale),
                ..default()
            },
        ),
        (
            RTQuad,
            white.clone(),
            TransformBundle {
                local: Transform::from_xyz(0.0, 1.0, 0.0)
                    .with_rotation(Quat::from_euler(
                        EulerRot::XYZ,
                        180f32.to_radians(),
                        0.0,
                        0.0,
                    ))
                    .with_scale(scale),
                ..default()
            },
        ),
        (
            RTQuad,
            blue.clone(),
            TransformBundle {
                local: Transform::from_xyz(0.0, 0.0, -1.0)
                    .with_rotation(Quat::from_euler(
                        EulerRot::XYZ,
                        90f32.to_radians(),
                        0.0,
                        0.0,
                    ))
                    .with_scale(scale),
                ..default()
            },
        ),
        (
            RTQuad,
            red.clone(),
            TransformBundle {
                local: Transform::from_xyz(1.0, 0.0, 0.0)
                    .with_rotation(Quat::from_euler(
                        EulerRot::XYZ,
                        0.0,
                        0.0,
                        90f32.to_radians(),
                    ))
                    .with_scale(scale),
                ..default()
            },
        ),
        (
            RTQuad,
            green.clone(),
            TransformBundle {
                local: Transform::from_xyz(-1.0, 0.0, 0.0)
                    .with_rotation(Quat::from_euler(
                        EulerRot::XYZ,
                        0.0,
                        0.0,
                        -90f32.to_radians(),
                    ))
                    .with_scale(scale),
                ..default()
            },
        ),
        (
            RTQuad,
            white.clone(),
            TransformBundle {
                local: Transform::from_xyz(0.0, 0.0, 1.0)
                    .with_rotation(Quat::from_euler(
                        EulerRot::XYZ,
                        -90f32.to_radians(),
                        0.0,
                        0.0,
                    ))
                    .with_scale(scale),
                ..default()
            },
        ),
    ]);

    commands.spawn((
        RTQuad,
        light.clone(),
        TransformBundle {
            local: Transform::from_xyz(0.0, 0.99, 0.0)
                .with_rotation(Quat::from_euler(
                    EulerRot::XYZ,
                    180f32.to_radians(),
                    0.0,
                    0.0,
                ))
                .with_scale(Vec3::ONE * 0.5),
            ..default()
        },
    ));
}
