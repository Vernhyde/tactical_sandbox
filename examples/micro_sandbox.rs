use avian3d::prelude::*;
use bevy::prelude::*;
use std::f32::consts::FRAC_PI_4;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, PhysicsPlugins::default()))
        .add_systems(Startup, setup_scene)
        .add_systems(Update, read_player_input) //Variable Rate Loop
        .add_systems(Update, handle_mouse_clicks)
        .add_systems(Update, steer_to_target)
        .add_systems(FixedUpdate, apply_locomotion) //64hz Loop
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

#[derive(Component, Default)]
pub struct TargetDestination {
    pub point: Option<Vec3>,
}

fn setup_scene(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>) {
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
            TargetDestination::default(),
        ))
        .with_children(|parent| {
            parent.spawn((
                Camera3d::default(),
                CameraGimbal { distance, pitch },
                //This Offsets the camera backwards and up, looking at the pivot
                Transform::from_xyz(0.0, distance * pitch.sin(), distance * pitch.cos()).looking_at(Vec3::ZERO, Vec3::Y),
            ));
        });
}

#[rustfmt::skip]
fn handle_mouse_clicks(
    mouse_input: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform)>, //This is to tell where the camera is looking from
    spatial_query: SpatialQuery, //This is an Avian3d tool that raycast against colliders
    mut pawn_query: Query<&mut TargetDestination, With<CameraPivot>>, //Query the Pawn's transform and intent
) {
    if !mouse_input.just_pressed(MouseButton::Right) { return; } //First time using Guard Clauses, This should stop the function if the user did not right click

    let Ok(window) = windows.single() else { return; }; //This extracts the window and camera from the results but if they do not exsist it returns
    let Ok((camera, camera_transform)) = camera_query.single() else { return; };
    let Some(cursor_position) = window.cursor_position() else { return; }; //Get 2D cursor position, if the mouse is off screen returns no data and stops
    let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_position) else { return; }; //This projects the ray in 3D space
    if let Some(hit) = spatial_query.cast_ray(ray.origin, ray.direction, 100.0, true, &SpatialQueryFilter::default(),) //This should be the only nested function, this fires the actual ray cast
   {
       let Ok(mut destination) = pawn_query.single_mut() else { return; };
       destination.point = Some(ray.origin + *ray.direction * hit.distance);
   }
}

#[rustfmt::skip]
fn read_player_input(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mut pivot_query: Query<(&CameraPivot, &mut Transform)>, //This is makes sure to Query the Pivot Pawn and NOT the Camera
) {
    let dt = time.delta_secs();

    for (pivot, mut transform) in &mut pivot_query {
        //Yaw Rotation Keys
        let mut rotation_input = 0.0;
        if keys.pressed(KeyCode::KeyQ) { rotation_input += 1.0; }
        if keys.pressed(KeyCode::KeyE) { rotation_input -= 1.0; }
        if rotation_input != 0.0 { transform.rotate_y(rotation_input * pivot.orbit_speed * dt); } //Rotation around the global Y Axis
    }
}

#[rustfmt::skip]
fn apply_locomotion(mut query: Query<(&CameraPivot, &MovementIntent, &mut LinearVelocity)>) {
    for (pivot, intent, mut velocity) in &mut query {
        velocity.x = intent.direction.x * pivot.move_speed;
        velocity.z = intent.direction.y * pivot.move_speed;
    } //Applies the Intent Direction that is scaled by the pawn's move speed. Do not use Velocity Y so that gravity and slopes work naitively.
}

#[rustfmt::skip]
fn steer_to_target(mut pawn_query: Query<(&Transform, &mut MovementIntent, &mut TargetDestination), With<CameraPivot>>)
{
    let Ok((transform, mut intent, mut destination)) = pawn_query.single_mut() else { return; };

    if let Some(target_point) = destination.point
    {
        let difference = target_point - transform.translation;
        let flat_direction = Vec2::new(difference.x, difference.z);

        if flat_direction.length_squared() > 0.0625 { intent.direction = flat_direction.normalize(); }
        else { intent.direction = Vec2::ZERO; destination.point = None; }
    }
}
