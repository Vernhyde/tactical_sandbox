//! Tactical Sandbox - Syntax & ECS Verification Test
use bevy::prelude::*;
use bevy::render::camera::ScalingMode;

const ARENA_SIZE_M: f32 = 40.0; // 40m [131.2 ft] training pad
const OPERATOR_SPEED_MPS: f32 = 4.5; // 4.5 m/s [14.8 fps] tactical pace

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup_tactical_arena)
        .add_systems(
            Update,
            (
                cycle_squad_possession,
                move_possessed_operator,
                update_indicator_colors,
            )
                .chain(),
        )
        .run();
}

/// Marker component: Whoever holds this tag is directly controlled by WASD.
#[derive(Component, Debug)]
struct Possessed;

/// Core operator data shared across player and AI squadmates.
#[derive(Component, Debug, Clone)]
struct Operator {
    callsign: String,
    squad_index: usize,
    move_speed: f32,
}

fn setup_tactical_arena(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Ground Plane
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(ARENA_SIZE_M, ARENA_SIZE_M))),
        MeshMaterial3d(materials.add(Color::srgb(0.12, 0.14, 0.12))),
    ));

    // Directional Sun Light
    commands.spawn((
        DirectionalLight {
            illuminance: 10_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(15.0, 25.0, 15.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Angled 3D Orthographic Camera
    commands.spawn((
        Camera3d::default(),
        Projection::from(OrthographicProjection {
            scaling_mode: ScalingMode::FixedVertical {
                viewport_height: 20.0,
            },
            ..OrthographicProjection::default_3d()
        }),
        Transform::from_xyz(0.0, 20.0, 20.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    let operator_mesh = meshes.add(Capsule3d::new(0.4, 1.0));

    let squad_spawns = [
        (0, "Alpha-1", Vec3::new(0.0, 0.9, 0.0), true),
        (1, "Alpha-2", Vec3::new(-3.0, 0.9, -2.5), false),
        (2, "Alpha-3", Vec3::new(3.0, 0.9, -2.5), false),
    ];

    for (idx, callsign, position, is_leader) in squad_spawns {
        let mut entity_cmds = commands.spawn((
            Mesh3d(operator_mesh.clone()),
            MeshMaterial3d(materials.add(Color::srgb(0.3, 0.35, 0.3))),
            Transform::from_translation(position),
            Operator {
                callsign: callsign.to_string(),
                squad_index: idx,
                move_speed: OPERATOR_SPEED_MPS,
            },
        ));

        if is_leader {
            entity_cmds.insert(Possessed);
        }
    }
}

/// Press TAB to cycle direct control between Alpha-1, Alpha-2, and Alpha-3.
fn cycle_squad_possession(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    query: Query<(Entity, &Operator, Has<Possessed>)>,
) {
    if !keyboard.just_pressed(KeyCode::Tab) {
        return;
    }

    let mut current_idx = 0;
    let total_operators = query.iter().count();

    for (entity, op, is_possessed) in &query {
        if is_possessed {
            current_idx = op.squad_index;
            commands.entity(entity).remove::<Possessed>();
        }
    }

    let next_idx = (current_idx + 1) % total_operators.max(1);

    for (entity, op, _) in &query {
        if op.squad_index == next_idx {
            commands.entity(entity).insert(Possessed);
            info!("Possessed operator switched to: {}", op.callsign);
        }
    }
}

fn move_possessed_operator(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut query: Query<(&mut Transform, &Operator), With<Possessed>>,
) {
    let mut direction = Vec3::ZERO;

    if keyboard.pressed(KeyCode::KeyW) { direction.z -= 1.0; }
    if keyboard.pressed(KeyCode::KeyS) { direction.z += 1.0; }
    if keyboard.pressed(KeyCode::KeyA) { direction.x -= 1.0; }
    if keyboard.pressed(KeyCode::KeyD) { direction.x += 1.0; }

    if direction.length_squared() > 0.0 {
        direction = direction.normalize();
    }

    for (mut transform, operator) in &mut query {
        transform.translation += direction * operator.move_speed * time.delta_secs();
    }
}

fn update_indicator_colors(
    mut materials: ResMut<Assets<StandardMaterial>>,
    query: Query<(&MeshMaterial3d<StandardMaterial>, Has<Possessed>), With<Operator>>,
) {
    for (mat_handle, is_possessed) in &query {
        if let Some(material) = materials.get_mut(mat_handle) {
            material.base_color = if is_possessed {
                Color::srgb(0.898, 0.663, 0.239) // Amber (#e5a93d)
            } else {
                Color::srgb(0.3, 0.35, 0.3)      // Olive Drab
            };
        }
    }
}
