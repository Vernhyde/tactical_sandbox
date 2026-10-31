use bevy::prelude::*;
use std::f32::consts::FRAC_PI_4;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup_scene)
        .add_systems(Update, update_camera_pivot)
        .run();
}

//Pivot for the camera
#[derive(Component)]
pub struct CameraPivot {
    pub move_speed: f32,  //Speed in M/s
    pub orbit_speed: f32, //Rotation is in Radians per Second
}

//Camera Arm to determine Pitch and such
#[derive(Component)]
pub struct CameraGimbal {
    pub distance: f32, //Distance in M
    pub pitch: f32,    //Pitch is in Radians
}

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    //Spawn a simple ground plane for testing, physics not enabled
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(50.0, 50.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.3, 0.5, 0.3))),
    ));

    //Directional Light for Shadows
    commands.spawn((
        DirectionalLight {
            shadow_maps_enabled: true,
            illuminance: 10_000.0,
            ..default()
        },
        Transform::from_rotation(Quat::from_rotation_x(-FRAC_PI_4)),
    ));

    //Defining Gimbal Variables
    let pitch = FRAC_PI_4; //45 Degree viewing angle
    let distance = 18.0; //18 meters back

    //This spawns the Pivot as a Parent with a Camera Child
    commands
        .spawn((
            CameraPivot {
                move_speed: 15.0, // 15 Meters per Second
                orbit_speed: 2.0, // 2 Radians per Second
            },
            Transform::from_xyz(0.0, 0.0, 0.0), //Starting at World Origin
            Visibility::default(),
        ))
        .with_children(|parent| {
            parent.spawn((
                Camera3d::default(),
                CameraGimbal { distance, pitch },
                //This Offsets the camera backwards and up, looking at the pivot
                Transform::from_xyz(0.0, distance * pitch.sin(), distance * pitch.cos())
                    .looking_at(Vec3::ZERO, Vec3::Y),
            ));
        });
}

fn update_camera_pivot(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    //This is makes sure to Query the Pivot and NOT the Camera
    mut pivot_query: Query<(&CameraPivot, &mut Transform)>,
) {
    let dt = time.delta_secs();

    for (pivot, mut transform) in &mut pivot_query {
        //Yaw Rotation Keys
        let mut rotation_input = 0.0;
        if keys.pressed(KeyCode::KeyQ) {
            rotation_input += 1.0;
        }
        if keys.pressed(KeyCode::KeyE) {
            rotation_input -= 1.0;
        }

        if rotation_input != 0.0 {
            //Rotation around the global Y Axis
            transform.rotate_y(rotation_input * pivot.orbit_speed * dt);
        }

        //Z and X movement
        let mut movement_input = Vec3::ZERO;
        if keys.pressed(KeyCode::KeyW) {
            movement_input.z -= 1.0;
        }
        if keys.pressed(KeyCode::KeyS) {
            movement_input.z += 1.0;
        }
        if keys.pressed(KeyCode::KeyA) {
            movement_input.x -= 1.0;
        }
        if keys.pressed(KeyCode::KeyD) {
            movement_input.x += 1.0;
        }

        if movement_input != Vec3::ZERO {
            //Normalizes diagnol movement so it is not faster
            let movement_input = movement_input.normalize();

            //Calculates Pivot's forward and right vectors on the X Z Plane
            let forward = *transform.local_z();
            let right = *transform.local_x();

            let flat_forward = Vec3::new(forward.x, 0.0, forward.z).normalize();
            let flat_right = Vec3::new(right.x, 0.0, right.z).normalize();

            //Calculates movement direction based on pivot facing
            let move_dir =
                (flat_forward * movement_input.z + flat_right * movement_input.x).normalize();

            //Apply translation
            transform.translation += move_dir * pivot.move_speed * dt;
        }
    }
}
