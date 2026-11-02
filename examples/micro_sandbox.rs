use avian3d::prelude::*;
use bevy::prelude::*;
use std::f32::consts::FRAC_PI_4;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, PhysicsPlugins::default()))
        .add_systems(Startup, setup_scene)
        .add_systems(Update, read_player_input) //Variable Rate Loop
        .add_systems(FixedUpdate, apply_locomotion) //64hz Loo
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

#[derive(Component, Default)]
pub struct MovementIntent {
    pub direction: Vec2, //X and Z Axis Movement
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
        RigidBody::Static,
        Collider::cuboid(50.0, 0.1, 50.0),
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

    //This spawns the Pivot as a Parent with a Camera Child (Now as a physical pawn)
    commands
        .spawn((
            CameraPivot {
                move_speed: 15.0, // 15 Meters per Second
                orbit_speed: 2.0, // 2 Radians per Second
            },
            MovementIntent::default(),
            //Visuals
            Mesh3d(meshes.add(Capsule3d::new(0.5, 1.0))),
            MeshMaterial3d(materials.add(Color::srgb(0.8, 0.2, 0.2))),
            Transform::from_xyz(0.0, 1.0, 0.0), //Starting at World Origin
            RigidBody::Kinematic,
            Collider::capsule(0.5, 1.0),
            LinearVelocity::default(),
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

fn handle_mouse_clicks(
    mouse_input: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform)>, //This is to tell where the camera is looking from
    spatial_query: SpatialQuery, //This is an Avian3d tool that raycast against colliders
) {
    if mouse_input.just_pressed(MouseButton::Right) {
        let window = windows.single();
        let (camera, camera_transform) = camera_query.single();

        if let Some(cursor_position) = window.cursor_position()
        //This gets the 2D pixel coords of the cursor in the window
        {
            if let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_position) {
                if let Some(hit) = spatial_query.cast_ray(
                    ray.origin,
                    ray.direction,
                    100.0,
                    true,
                    &SpatialQueryFilter::default(),
                ) {
                    let target_point = ray.origin + ray.direction * hit.time_of_impact;
                    println!("Commanded to move to: {}", target_point);
                }
            }
        }
    }
}

fn read_player_input(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    //This is makes sure to Query the Pivot Pawn and NOT the Camera
    mut pivot_query: Query<(&CameraPivot, &mut Transform, &mut MovementIntent)>,
) {
    let dt = time.delta_secs();

    for (pivot, mut transform, mut intent) in &mut pivot_query {
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
            //Normalizes diagonal movement so it is not faster
            let movement_input = movement_input.normalize();

            //Calculates Pivot's forward and right vectors on the X Z Plane
            let forward = *transform.local_z();
            let right = *transform.local_x();

            let flat_forward = Vec3::new(forward.x, 0.0, forward.z).normalize();
            let flat_right = Vec3::new(right.x, 0.0, right.z).normalize();

            //Calculates movement direction based on pivot facing
            let move_dir =
                (flat_forward * movement_input.z + flat_right * movement_input.x).normalize();

            intent.direction = Vec2::new(move_dir.x, move_dir.z);
        } else {
            intent.direction = Vec2::ZERO;
        }
    }
}

fn apply_locomotion(mut query: Query<(&CameraPivot, &MovementIntent, &mut LinearVelocity)>) {
    for (pivot, intent, mut velocity) in &mut query {
        //Applies the Intent Direction that is scaled by the pawn's move speed.
        // Do not use Velocity Y so that gravity adn slopes work naitively.
        velocity.x = intent.direction.x * pivot.move_speed;
        velocity.z = intent.direction.y * pivot.move_speed;
    }
}
