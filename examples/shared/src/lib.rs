use bevy::{input::mouse::MouseMotion, prelude::*, window::CursorGrabMode};

#[derive(Component, Default)]
pub struct FreeCam {
    look_vector: Vec2,
}

#[derive(Component)]
pub struct DebugText;

// ---- Plugin ----
pub struct SharedPlugin;

impl Plugin for SharedPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, init_camera)
            .add_systems(Update, (grab_cursor, move_camera, debug_text));
    }
}

fn grab_cursor(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    mut windows: Query<&mut Window>,
) {
    let mut window = windows.single_mut();

    if mouse_input.just_pressed(MouseButton::Left) {
        window.cursor.visible = false;
        window.cursor.grab_mode = CursorGrabMode::Locked;
    }

    if keyboard_input.just_pressed(KeyCode::Escape) {
        window.cursor.visible = true;
        window.cursor.grab_mode = CursorGrabMode::None;
    }
}

fn init_camera(mut query: Query<(&mut FreeCam, &Transform), Added<FreeCam>>) {
    for (mut freecam, transform) in query.iter_mut() {
        let (y, x, _) = transform.rotation.to_euler(EulerRot::YXZ);
        freecam.look_vector.x = x.to_radians();
        freecam.look_vector.y = y.to_radians();
    }
}

fn move_camera(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut mouse_motion: EventReader<MouseMotion>,
    mut query: Query<(&mut FreeCam, &mut Transform)>,
) {
    let mut look_delta = Vec2::ZERO;
    for motion in mouse_motion.read() {
        look_delta += motion.delta;
    }

    let move_vector = Vec3::new(
        (keyboard_input.pressed(KeyCode::KeyD) as i8 - keyboard_input.pressed(KeyCode::KeyA) as i8)
            as f32,
        (keyboard_input.pressed(KeyCode::KeyE) as i8 - keyboard_input.pressed(KeyCode::KeyQ) as i8)
            as f32,
        (keyboard_input.pressed(KeyCode::KeyS) as i8 - keyboard_input.pressed(KeyCode::KeyW) as i8)
            as f32,
    );

    let mouse_sensitivity = 0.001;
    let speed = 4.0;

    for (mut freecam, mut transform) in query.iter_mut() {
        freecam.look_vector -= look_delta * mouse_sensitivity;
        transform.rotation = Quat::from_euler(
            EulerRot::YXZ,
            freecam.look_vector.x,
            freecam.look_vector.y,
            0.0,
        );

        let movement = transform.rotation.mul_vec3(move_vector);
        transform.translation += movement * speed * time.delta_seconds();
    }
}

fn debug_text(time: Res<Time>, mut query: Query<&mut Text, With<DebugText>>) {
    for mut text in query.iter_mut() {
        text.sections[0].value = format!(
            "FPS: {:.1}\nMs: {:.4}",
            1.0 / time.delta_seconds(),
            time.delta_seconds()
        );
    }
}
