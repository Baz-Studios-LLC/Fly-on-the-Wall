//! The people who live here.
//!
//! Blocky, in the voxel language the game was described in from the start —
//! but jointed. A Minecraft body is six boxes and its arms swing from the
//! shoulder in one piece, which is why it reads as a puppet: nothing between
//! shoulder and hand ever changes shape. This one has **elbows, knees, wrists
//! and ankles**, so an arm is upper arm, forearm and hand hung off each other,
//! and a pose is a set of joint angles rather than a set of positions.
//!
//! Three things are doing most of the work of looking better than the
//! reference, and none of them cost the blocky read:
//!
//! 1. **Taper.** Every limb segment is three boxes, each a shade narrower than
//!    the last. A thigh is thick at the hip and thin at the knee, which is
//!    almost all of what makes a leg look like a leg.
//! 2. **Joints that exist.** A small box at each elbow and knee, slightly
//!    proud, so the corner has something in it when the limb bends instead of
//!    two prisms passing through one another.
//! 3. **Proportion.** Seven and a half heads, not four. The reference is a
//!    caricature; this is a man in a room built to centimetres, and standing a
//!    caricature in it would make the room look wrong rather than him.
//!
//! He is scenery for now: entities and transforms, with no collision and no
//! behaviour. The family simulation is a long way off and this is the body it
//! will eventually be given.

use bevy::prelude::*;

/// Overall height, in centimetres. A tall-ish man, so he reads against
/// nine-foot ceilings without looking like a child.
const TALL: f32 = 178.0;

/// A joint that can be posed. The name is for later — an animation system will
/// want to ask for "the right elbow" rather than count children.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Joint {
    Waist,
    Neck,
    Shoulder(Side),
    Elbow(Side),
    Hip(Side),
    Knee(Side),
    Ankle(Side),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Left,
    Right,
}

impl Side {
    fn sign(self) -> f32 {
        match self {
            Side::Left => -1.0,
            Side::Right => 1.0,
        }
    }
}

/// A person in the house.
#[derive(Component)]
pub struct Person;

pub struct FolkPlugin;

impl Plugin for FolkPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostStartup, raise_the_father);
    }
}

// Skin, hair, and the clothes of a man at home on a weekday afternoon.
const SKIN: Color = Color::srgb(0.78, 0.60, 0.47);
const SKIN_DARK: Color = Color::srgb(0.70, 0.53, 0.41);
const HAIR: Color = Color::srgb(0.24, 0.18, 0.14);
const SHIRT: Color = Color::srgb(0.42, 0.50, 0.58);
const SHIRT_DARK: Color = Color::srgb(0.36, 0.44, 0.52);
const JEANS: Color = Color::srgb(0.28, 0.32, 0.42);
const JEANS_DARK: Color = Color::srgb(0.24, 0.28, 0.37);
const BELT: Color = Color::srgb(0.22, 0.17, 0.14);
const SHOE: Color = Color::srgb(0.20, 0.18, 0.17);
const EYE_WHITE: Color = Color::srgb(0.90, 0.89, 0.86);
const EYE: Color = Color::srgb(0.20, 0.26, 0.30);
const MOUTH: Color = Color::srgb(0.54, 0.36, 0.32);

/// One box, hung off a parent at a local offset.
fn block(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    parent: Entity,
    at: Vec3,
    size: Vec3,
    paint: &Handle<StandardMaterial>,
) -> Entity {
    let mesh = meshes.add(Cuboid::new(size.x, size.y, size.z));
    commands
        .spawn((
            ChildOf(parent),
            Mesh3d(mesh),
            MeshMaterial3d(paint.clone()),
            Transform::from_translation(at),
        ))
        .id()
}

/// A limb segment: three boxes down its length, tapering.
///
/// Hung downward from the joint at the origin, because every limb on a standing
/// body points down and writing them all that way means a pose is one rotation
/// per joint with no offsets to keep straight.
fn limb(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    parent: Entity,
    length: f32,
    top: Vec2,
    bottom: Vec2,
    paint: &Handle<StandardMaterial>,
) {
    // Five segments, not three, and each one overlapping the next.
    //
    // Three left a visible ledge at every junction, and three ledges down a
    // forearm read as three separate blocks rather than one tapering limb —
    // stairs instead of an arm. Finer steps with an overlap put the change
    // where the eye reads it as shape.
    const STEPS: usize = 5;
    for k in 0..STEPS {
        let mid = (k as f32 + 0.5) / STEPS as f32;
        let wide = top.lerp(bottom, mid);
        block(
            commands,
            meshes,
            parent,
            Vec3::new(0.0, -length * mid, 0.0),
            Vec3::new(wide.x, length / STEPS as f32 + 1.6, wide.y),
            paint,
        );
    }
}

/// A joint block: something for the corner to be made of when the limb bends.
fn knuckle(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    parent: Entity,
    across: f32,
    paint: &Handle<StandardMaterial>,
) {
    block(
        commands,
        meshes,
        parent,
        Vec3::ZERO,
        Vec3::splat(across),
        paint,
    );
}

fn raise_the_father(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let dull = |base: Color, rough: f32| StandardMaterial {
        base_color: base,
        perceptual_roughness: rough,
        ..default()
    };
    let skin = materials.add(dull(SKIN, 0.72));
    let skin_dark = materials.add(dull(SKIN_DARK, 0.72));
    let hair = materials.add(dull(HAIR, 0.88));
    let shirt = materials.add(dull(SHIRT, 0.92));
    let shirt_dark = materials.add(dull(SHIRT_DARK, 0.92));
    let jeans = materials.add(dull(JEANS, 0.95));
    let jeans_dark = materials.add(dull(JEANS_DARK, 0.95));
    let belt = materials.add(dull(BELT, 0.55));
    let shoe = materials.add(dull(SHOE, 0.60));
    let eye_white = materials.add(dull(EYE_WHITE, 0.30));
    let eye = materials.add(dull(EYE, 0.20));
    let mouth = materials.add(dull(MOUTH, 0.70));
    let meshes = &mut *meshes;

    // Standing in the great room, off to one side, facing across it.
    let room = crate::house::room("great room");
    let stand = Vec3::new(
        room.min.x + room.wide() * 0.26,
        0.0,
        room.min.y + room.deep() * 0.30,
    );

    // The root is the pelvis. Everything above it hangs off the waist and
    // everything below off the hips, which is what lets a single rotation at
    // the waist lean the whole upper body.
    let hip_height = TALL * 0.53;
    let root = commands
        .spawn((
            Person,
            Name::new("Father"),
            Transform::from_translation(stand + Vec3::new(0.0, hip_height, 0.0))
                .with_rotation(Quat::from_rotation_y(-0.7)),
            Visibility::default(),
        ))
        .id();

    // -- Below the waist ----------------------------------------------------
    block(
        &mut commands,
        meshes,
        root,
        Vec3::new(0.0, 2.0, 0.0),
        Vec3::new(33.0, 20.0, 20.0),
        &jeans,
    );
    block(
        &mut commands,
        meshes,
        root,
        Vec3::new(0.0, 12.5, 0.0),
        Vec3::new(34.0, 5.0, 21.0),
        &belt,
    );

    for side in [Side::Left, Side::Right] {
        let s = side.sign();
        // A little contrapposto: one leg takes the weight and the other is
        // slack. A body with both legs identical reads as a mannequin however
        // well it is built.
        let slack = if side == Side::Right { 1.0 } else { 0.0 };

        let hip = commands
            .spawn((
                ChildOf(root),
                Joint::Hip(side),
                Transform::from_translation(Vec3::new(s * 9.0, -6.0, 0.0))
                    .with_rotation(Quat::from_rotation_x(slack * 0.10)),
                Visibility::default(),
            ))
            .id();
        limb(
            &mut commands,
            meshes,
            hip,
            42.0,
            Vec2::new(16.5, 17.5),
            Vec2::new(13.0, 14.0),
            &jeans,
        );

        let knee = commands
            .spawn((
                ChildOf(hip),
                Joint::Knee(side),
                Transform::from_translation(Vec3::new(0.0, -42.0, 0.0))
                    .with_rotation(Quat::from_rotation_x(-slack * 0.22)),
                Visibility::default(),
            ))
            .id();
        knuckle(&mut commands, meshes, knee, 13.5, &jeans_dark);
        limb(
            &mut commands,
            meshes,
            knee,
            42.0,
            Vec2::new(12.5, 13.5),
            Vec2::new(9.5, 10.5),
            &jeans,
        );

        let ankle = commands
            .spawn((
                ChildOf(knee),
                Joint::Ankle(side),
                Transform::from_translation(Vec3::new(0.0, -42.0, 0.0)),
                Visibility::default(),
            ))
            .id();
        // A shoe: sole and upper, with the toe toward the front of the body.
        //
        // This whole body faces −z — nose, brow and fringe are all on that
        // side — and the first version of the shoe put its long end on +z, so
        // he stood in the great room with his feet on backwards. It is the
        // kind of mistake that is invisible in the numbers and unmissable the
        // moment somebody looks at him.
        block(
            &mut commands,
            meshes,
            ankle,
            Vec3::new(0.0, -2.5, -3.0),
            Vec3::new(11.0, 5.0, 26.0),
            &shoe,
        );
        block(
            &mut commands,
            meshes,
            ankle,
            Vec3::new(0.0, 1.5, 1.0),
            Vec3::new(10.0, 6.0, 15.0),
            &shoe,
        );
    }

    // -- Above the waist ----------------------------------------------------
    let waist = commands
        .spawn((
            ChildOf(root),
            Joint::Waist,
            Transform::from_translation(Vec3::new(0.0, 12.0, 0.0))
                .with_rotation(Quat::from_rotation_y(0.06)),
            Visibility::default(),
        ))
        .id();

    // Abdomen and chest, the chest wider and deeper: the taper from waist to
    // shoulder is what separates a man from a fridge.
    block(
        &mut commands,
        meshes,
        waist,
        Vec3::new(0.0, 9.0, 0.0),
        Vec3::new(31.0, 20.0, 19.0),
        &shirt,
    );
    block(
        &mut commands,
        meshes,
        waist,
        Vec3::new(0.0, 27.0, 0.0),
        Vec3::new(36.0, 18.0, 21.0),
        &shirt,
    );
    block(
        &mut commands,
        meshes,
        waist,
        Vec3::new(0.0, 38.0, 0.0),
        Vec3::new(39.0, 8.0, 21.5),
        &shirt,
    );
    // Collar.
    block(
        &mut commands,
        meshes,
        waist,
        Vec3::new(0.0, 43.0, 0.5),
        Vec3::new(24.0, 4.0, 18.0),
        &shirt_dark,
    );

    // -- Head ---------------------------------------------------------------
    let neck = commands
        .spawn((
            ChildOf(waist),
            Joint::Neck,
            Transform::from_translation(Vec3::new(0.0, 44.0, 0.0))
                .with_rotation(Quat::from_rotation_y(-0.12)),
            Visibility::default(),
        ))
        .id();
    block(
        &mut commands,
        meshes,
        neck,
        Vec3::new(0.0, 3.0, 0.0),
        Vec3::new(11.0, 8.0, 11.0),
        &skin_dark,
    );
    // Skull, jaw, brow, nose, ears: six boxes and it stops being a cube.
    block(
        &mut commands,
        meshes,
        neck,
        Vec3::new(0.0, 16.0, 0.0),
        Vec3::new(16.0, 15.0, 18.0),
        &skin,
    );
    block(
        &mut commands,
        meshes,
        neck,
        Vec3::new(0.0, 8.5, 1.0),
        Vec3::new(14.5, 8.0, 16.0),
        &skin,
    );
    block(
        &mut commands,
        meshes,
        neck,
        Vec3::new(0.0, 17.5, -8.5),
        Vec3::new(14.0, 3.5, 2.5),
        &skin_dark,
    );
    block(
        &mut commands,
        meshes,
        neck,
        Vec3::new(0.0, 13.5, -9.5),
        Vec3::new(4.0, 5.0, 3.5),
        &skin,
    );
    for side in [-1.0f32, 1.0] {
        block(
            &mut commands,
            meshes,
            neck,
            Vec3::new(side * 8.5, 15.0, 1.0),
            Vec3::new(2.0, 7.0, 5.0),
            &skin_dark,
        );
    }
    // Eyes, and the whole face turns on them.
    //
    // A head with a brow and a nose and no eyes is a mannequin, and it is the
    // one place on this body where a two-centimetre box earns more than
    // anything else on it. White set into the socket under the brow, pupil
    // proud of that, and both a shade inside the cheekbone so the face has
    // depth rather than a decal.
    for side in [-1.0f32, 1.0] {
        block(
            &mut commands,
            meshes,
            neck,
            Vec3::new(side * 3.6, 15.6, -8.9),
            Vec3::new(4.2, 2.6, 1.0),
            &eye_white,
        );
        block(
            &mut commands,
            meshes,
            neck,
            Vec3::new(side * 4.2, 15.6, -9.3),
            Vec3::new(1.8, 2.2, 0.8),
            &eye,
        );
    }
    // A mouth, closed and unremarkable, which is what a face at rest has.
    block(
        &mut commands,
        meshes,
        neck,
        Vec3::new(0.0, 9.8, -7.6),
        Vec3::new(5.6, 1.0, 0.9),
        &mouth,
    );

    // Hair: a cap with a fringe over the brow and a shorter back and sides.
    block(
        &mut commands,
        meshes,
        neck,
        Vec3::new(0.0, 22.8, 0.5),
        Vec3::new(16.6, 5.6, 18.6),
        &hair,
    );
    block(
        &mut commands,
        meshes,
        neck,
        // Overlapping the cap, or a band of scalp shows between fringe and
        // crown — which on a head this size is a bald stripe.
        Vec3::new(0.0, 20.4, -8.4),
        Vec3::new(16.4, 5.6, 2.6),
        &hair,
    );
    for side in [-1.0f32, 1.0] {
        block(
            &mut commands,
            meshes,
            neck,
            Vec3::new(side * 8.3, 19.0, 1.0),
            Vec3::new(1.6, 7.0, 17.0),
            &hair,
        );
    }
    block(
        &mut commands,
        meshes,
        neck,
        Vec3::new(0.0, 18.5, 9.2),
        Vec3::new(16.4, 8.0, 1.8),
        &hair,
    );

    // -- Arms ---------------------------------------------------------------
    for side in [Side::Left, Side::Right] {
        let s = side.sign();
        // Arms hang with a little swing and a real bend at the elbow. Straight
        // arms at the sides is the pose of a doll in a box.
        let swing = if side == Side::Right { 0.16 } else { -0.24 };
        // A resting arm is barely bent. Half a radian is a man holding
        // something.
        let bend = if side == Side::Right { 0.16 } else { 0.30 };

        let shoulder = commands
            .spawn((
                ChildOf(waist),
                Joint::Shoulder(side),
                Transform::from_translation(Vec3::new(s * 19.5, 38.0, 0.0))
                    .with_rotation(Quat::from_rotation_z(-s * 0.10) * Quat::from_rotation_x(swing)),
                Visibility::default(),
            ))
            .id();
        // The shoulder cap, so the sleeve has a shoulder in it.
        block(
            &mut commands,
            meshes,
            shoulder,
            Vec3::new(-s * 1.5, 0.0, 0.0),
            Vec3::new(10.5, 8.0, 12.0),
            &shirt,
        );
        limb(
            &mut commands,
            meshes,
            shoulder,
            28.0,
            Vec2::new(11.0, 12.0),
            Vec2::new(9.0, 10.0),
            &shirt,
        );
        // The cuff of a rolled sleeve, where shirt gives way to arm.
        block(
            &mut commands,
            meshes,
            shoulder,
            Vec3::new(0.0, -27.0, 0.0),
            Vec3::new(9.6, 4.0, 10.6),
            &shirt_dark,
        );

        let elbow = commands
            .spawn((
                ChildOf(shoulder),
                Joint::Elbow(side),
                Transform::from_translation(Vec3::new(0.0, -28.0, 0.0))
                    .with_rotation(Quat::from_rotation_x(bend)),
                Visibility::default(),
            ))
            .id();
        knuckle(&mut commands, meshes, elbow, 9.5, &skin_dark);
        limb(
            &mut commands,
            meshes,
            elbow,
            26.0,
            Vec2::new(9.0, 9.5),
            Vec2::new(7.0, 7.5),
            &skin,
        );
        // A hand: palm, and a thumb off the inside edge.
        block(
            &mut commands,
            meshes,
            elbow,
            Vec3::new(0.0, -31.0, 0.5),
            Vec3::new(7.5, 11.0, 4.5),
            &skin,
        );
        block(
            &mut commands,
            meshes,
            elbow,
            Vec3::new(-s * 4.0, -28.5, 0.5),
            Vec3::new(3.0, 5.5, 4.0),
            &skin,
        );
    }
}
