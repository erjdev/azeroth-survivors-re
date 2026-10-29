//! The hero: its body, WASD movement over the terrain, the top-down camera, and its health.

mod camera_input;

use benilla_world::collision::WorldCollision;
use benilla_world::view::WorldCamera;
use bevy::input::mouse::{AccumulatedMouseMotion, MouseScrollUnit, MouseWheel};
use bevy::prelude::*;

use super::data::{self, ClassDef};
use super::units::{self, flat, wow_to_bevy, yaw_toward, Fx, Marks, UnitSpec};
use super::{Phase, Run, Selection};
use crate::creature_anim::{move_flags, MovementState};
use crate::net::{FieldChanged, Guid, ObjectStore};

#[derive(Component)]
pub(super) struct Hero {
    /// Vertical velocity in yards/second while airborne; 0 on the ground.
    pub(super) vy: f32,
    /// Height above the ground while airborne. Kept here, not in the transform: the creature
    /// ground clamp (`net::motion::ground_clamp_creatures`) re-seats every unit on the floor at
    /// the top of the frame, so the arc is re-applied over the ground each frame.
    pub(super) air: f32,
}

/// The body radius enemies stop at.
pub(crate) const HERO_RADIUS: f32 = 0.7;

/// The hop's initial upward speed and the gravity that brings it back down — the reference
/// `CMovement` jump seed and gravity (`player/state.rs`'s `JUMP_SPEED` / `GRAVITY`).
const JUMP_SPEED: f32 = 7.955_547;
const GRAVITY: f32 = 19.291_105;

/// Radians of orbit per raw mouse unit while right-dragging.
const ORBIT_RATE: f32 = 0.008;
/// Elevation clamp (radians): ~10°..~83° above the horizon, so the camera never dives underground.
const PITCH_MIN: f32 = 0.18;
const PITCH_MAX: f32 = 1.45;
/// The locked view's tilt.
const FIXED_PITCH: f32 = 61.0 * std::f32::consts::PI / 180.0;
/// Shared camera/cutaway focus above the hero's grounded origin.
pub(super) const CAMERA_FOCUS_HEIGHT: f32 = 1.4;
/// Wheel zoom range, yards from the hero's shoulders.
const ZOOM_MIN: f32 = 1.5;
const ZOOM_MAX: f32 = 70.0;
/// Radians per second the menu camera walks around the hero. A third of the first
/// pass, which turned the world behind them too quickly.
const MENU_ORBIT: f32 = 0.2 / 3.0;

/// Whether the camera is locked; `C` toggles it.
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub(super) enum CamMode {
    /// Locked top-down (58°), laptop-friendly — the original feel.
    #[default]
    Fixed,
    /// Unlocked: a mouse drag swings and tilts it around the hero.
    Orbit,
}

#[derive(Resource)]
pub(super) struct CamRig {
    pub dist: f32,
    pub orbit: f32,
    /// Orbit azimuth (radians) around the hero; right-drag swings it during play.
    pub yaw: f32,
    /// Elevation above the horizon (radians), clamped so the camera never dives underground.
    pub pitch: f32,
    pub mode: CamMode,
}

impl Default for CamRig {
    fn default() -> Self {
        Self {
            dist: 30.0,
            orbit: 0.6,
            yaw: 0.0,
            pitch: FIXED_PITCH,
            mode: CamMode::Fixed,
        }
    }
}

/// Where a battleground's runs start.
pub(super) fn arena(map: &data::MapDef) -> Vec3 {
    wow_to_bevy(map.center)
}

/// The world is the battleground being stood in ([`Selection::stage`]): entering one on another
/// continent worldports, which raises the loading screen until the new ground is in.
pub(super) fn follow_map(
    selection: Res<Selection>,
    current: Option<ResMut<benilla_world::world_map::CurrentMap>>,
) {
    let Some(mut current) = current else {
        return;
    };
    let want = selection.stage_def().map;
    if current.0 != want {
        current.0 = want;
    }
}

/// The hero in the class trainer's look, weapons drawn.
pub(super) fn spawn_hero(
    commands: &mut Commands,
    class: &'static ClassDef,
    display: u32,
    race: u8,
    sex: u8,
    serial: u64,
    pos: Vec3,
) -> Entity {
    let bytes0 = u32::from(race)
        | u32::from(class.id) << 8
        | u32::from(sex) << 16
        | u32::from(class.power) << 24;
    let e = units::spawn_unit(
        commands,
        &UnitSpec {
            serial,
            display,
            scale: 1.0,
            level: 60,
            faction: units::FACTION_FRIENDLY,
            bytes0,
            weapons: class.weapons,
            sheath: class.sheath,
        },
        pos,
        0.0,
    );
    commands
        .entity(e)
        .insert((Hero { vy: 0.0, air: 0.0 }, Marks::buffs()));
    e
}

/// Midday over the battleground, whatever the clock says.
pub(super) const MINUTE_OF_DAY: u32 = 780;

pub(super) fn daylight(debug: Option<ResMut<benilla_world::dev_state::DebugState>>) {
    let Some(mut debug) = debug else { return };
    if debug.lighting.follow_server_time || debug.lighting.manual_minute != MINUTE_OF_DAY {
        debug.lighting.follow_server_time = false;
        debug.lighting.manual_minute = MINUTE_OF_DAY;
    }
}

/// The ground under the mouse cursor, `None` with the cursor off the window. The locked camera's
/// hero faces it, and the aimed mobility spells (Blink, Charge) go toward it.
#[derive(Resource, Default)]
pub(super) struct AimPoint(pub(super) Option<Vec3>);

/// Cast the cursor onto the terrain: the camera ray meets the plane at the hero's height, then the
/// terrain height there, a few times over, which settles on hills within a frame.
pub(super) fn aim_cursor(
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
    cams: Query<(&Camera, &Transform), With<WorldCamera>>,
    hero: Query<&Transform, (With<Hero>, Without<WorldCamera>)>,
    world: super::Floor,
    mut aim: ResMut<AimPoint>,
) {
    let point = (|| {
        let cursor = windows.single().ok()?.cursor_position()?;
        let (cam, tf) = cams.single().ok()?;
        let ray = cam
            .viewport_to_world(&GlobalTransform::from(*tf), cursor)
            .ok()?;
        let mut y = hero.single().ok()?.translation.y;
        let mut at = None;
        for _ in 0..4 {
            if ray.direction.y.abs() < 1e-4 {
                break;
            }
            let t = (y - ray.origin.y) / ray.direction.y;
            if t <= 0.0 {
                break;
            }
            let p = ray.origin + *ray.direction * t;
            at = Some(p);
            match world.height(p) {
                Some(h) if (h - y).abs() > 0.05 => y = h,
                _ => break,
            }
        }
        at
    })();
    if aim.0 != point {
        aim.0 = point;
    }
}

/// WASD over the terrain within the battleground's leash, and the streaming focus on the hero.
#[allow(clippy::too_many_arguments)]
pub(super) fn drive_hero(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    phase: Res<State<Phase>>,
    run: Res<Run>,
    selection: Res<Selection>,
    rig: Res<CamRig>,
    aim: Res<AimPoint>,
    controller: Res<super::controller_input::ControllerInput>,
    loading: Res<crate::loading_screen::LoadingScreen>,
    world: super::Floor,
    auto: Option<Res<super::auto::Auto>>,
    mut player: ResMut<crate::player::Player>,
    mut hero: Query<(&mut Transform, &mut MovementState, &mut Hero)>,
) {
    let focus = match hero.single_mut() {
        Ok((mut tf, mut mv, mut hm)) => {
            let dt = time.delta_secs();
            let mut dir = Vec3::ZERO;
            let mut analog = false;
            if let Some(auto) = auto
                .as_deref()
                .filter(|_| *phase.get() == Phase::Playing && run.dead_t.is_none())
            {
                dir = super::auto::steer(auto);
            } else if *phase.get() == Phase::Playing && run.dead_t.is_none() && !loading.covering()
            {
                if keys.any_pressed([KeyCode::KeyW, KeyCode::ArrowUp]) {
                    dir.z -= 1.0;
                }
                if keys.any_pressed([KeyCode::KeyS, KeyCode::ArrowDown]) {
                    dir.z += 1.0;
                }
                if keys.any_pressed([KeyCode::KeyA, KeyCode::ArrowLeft]) {
                    dir.x -= 1.0;
                }
                if keys.any_pressed([KeyCode::KeyD, KeyCode::ArrowRight]) {
                    dir.x += 1.0;
                }
                if dir == Vec3::ZERO {
                    let [x, y] = controller.frame.movement;
                    dir = Vec3::new(x, 0.0, -y);
                    analog = dir != Vec3::ZERO;
                }
                // In orbit mode WASD is camera-relative; in fixed mode it stays world-relative.
                if rig.mode == CamMode::Orbit {
                    let (sy, cy) = rig.yaw.sin_cos();
                    dir = Vec3::new(dir.x * cy + dir.z * sy, 0.0, dir.z * cy - dir.x * sy);
                }
            }
            let mut pos = tf.translation;
            // Locked, the hero faces the cursor and WASD moves him under it, so he backpedals and
            // strafes as a player does; unlocked (or driven), he faces where he runs.
            let face = (rig.mode == CamMode::Fixed
                && auto.is_none()
                && !analog
                && *phase.get() == Phase::Playing
                && run.dead_t.is_none())
            .then(|| aim.0.map(|a| flat(a - pos)))
            .flatten()
            .filter(|d| d.length() > 0.5)
            .map(|d| d.normalize());
            if let Some(look) = face {
                let target = Quat::from_rotation_y(yaw_toward(look));
                tf.rotation = tf.rotation.slerp(target, (dt * 20.0).min(1.0));
            }
            if dir != Vec3::ZERO && dt > 0.0 {
                let strength = dir.length().min(1.0);
                let dir = dir.normalize();
                let speed = run.stats().speed * run.haste_mult() * strength;
                pos += dir * speed * dt;
                mv.speed = speed;
                mv.flags = match face {
                    Some(look) => gait(look, dir),
                    None => {
                        let target = Quat::from_rotation_y(yaw_toward(dir));
                        tf.rotation = tf.rotation.slerp(target, (dt * 20.0).min(1.0));
                        move_flags::FORWARD
                    }
                };
            } else if mv.speed != 0.0 || mv.flags != 0 {
                mv.speed = 0.0;
                mv.flags = 0;
            }
            let map = selection.stage_def();
            let center = arena(map);
            let off = flat(pos - center);
            if off.length() > map.leash * 2.0 {
                // Just entered another battleground: start at its centre.
                pos = center;
            } else {
                if off.length() > map.leash {
                    let held = center + off.normalize() * map.leash;
                    pos.x = held.x;
                    pos.z = held.z;
                }
                pos = world.slide(tf.translation, pos, HERO_RADIUS);
            }
            // Jump height is relative to the floor, not the input to the next floor probe.
            let feet = pos - Vec3::Y * hm.air;
            let ground = world.height(feet).unwrap_or(feet.y);
            let playing =
                *phase.get() == Phase::Playing && run.dead_t.is_none() && !loading.covering();
            // Space hops: lift off the terrain, then gravity brings the hero back down.
            if playing
                && (keys.just_pressed(KeyCode::Space) || controller.frame.jump)
                && hm.vy == 0.0
                && hm.air == 0.0
            {
                hm.vy = JUMP_SPEED;
            }
            if playing && (hm.vy != 0.0 || hm.air > 0.0) {
                hm.vy -= GRAVITY * dt;
                hm.air += hm.vy * dt;
                if hm.air <= 0.0 {
                    hm.air = 0.0;
                    hm.vy = 0.0;
                }
            } else {
                hm.air = 0.0;
                hm.vy = 0.0;
            }
            pos.y = ground + hm.air;
            // Feed the animation driver: `vertical_speed`'s sign and the `FALLING` flag are what
            // select the Jump (37/38) / Fall (40) animation (`creature_anim::select`).
            mv.vertical_speed = hm.vy;
            if hm.vy != 0.0 {
                mv.flags |= move_flags::FALLING;
            }
            if tf.translation != pos {
                tf.translation = pos;
            }
            pos
        }
        Err(_) => {
            let mut p = arena(selection.stage_def());
            if let Some(h) = world.height(p) {
                p.y = h;
            }
            p
        }
    };
    // The terrain streams and the loading screen waits on the ground under this.
    player.active = true;
    player.detached = false;
    player.settling = false;
    player.world_stale = false;
    player.pos = focus;
}

/// A slow orbit at the menu; in play, top-down until `C` unlocks it to swing and tilt.
pub(super) fn place_camera(
    time: Res<Time<Real>>,
    phase: Res<State<Phase>>,
    mut rig: ResMut<CamRig>,
    mut camera_settings: ResMut<super::camera::CameraSettings>,
    mut trace: Local<super::camera::CameraTrace>,
    controller: Res<super::controller_input::ControllerInput>,
    loading: Res<crate::loading_screen::LoadingScreen>,
    mut wheel: MessageReader<MouseWheel>,
    mouse: Res<ButtonInput<MouseButton>>,
    motion: Res<AccumulatedMouseMotion>,
    keys: Res<ButtonInput<KeyCode>>,
    player: Res<crate::player::Player>,
    world: WorldCollision,
    hero: Query<&Transform, (With<Hero>, Without<WorldCamera>)>,
    mut cams: Query<&mut Transform, With<WorldCamera>>,
) {
    camera_input::apply(
        &mut rig,
        &camera_settings,
        *phase.get(),
        loading.covering(),
        keys.just_pressed(KeyCode::KeyC),
        mouse
            .any_pressed([MouseButton::Left, MouseButton::Right])
            .then_some(motion.delta),
        wheel.read(),
    );
    camera_input::analog(
        &mut rig,
        &camera_settings,
        if *phase.get() == Phase::Playing && !loading.covering() {
            controller.frame
        } else {
            Default::default()
        },
        time.delta_secs(),
    );
    // With no hero yet, the focus `drive_hero` grounded.
    let focus = hero.single().map_or(player.pos, |t| t.translation);
    let (eye, look) = if *phase.get() == Phase::Menu {
        rig.orbit += time.delta_secs() * MENU_ORBIT;
        let d = 7.0;
        (
            focus + Vec3::new(rig.orbit.sin() * d, 2.4, rig.orbit.cos() * d),
            focus + Vec3::Y * 1.1,
        )
    } else {
        let (sy, cy) = rig.yaw.sin_cos();
        let (sp, cp) = rig.pitch.sin_cos();
        let look = focus + Vec3::Y * CAMERA_FOCUS_HEIGHT;
        let eye = look + Vec3::new(cp * sy, sp, cp * cy) * rig.dist;
        (eye, look)
    };
    let safe = if camera_settings.auto_push {
        super::floor::camera_eye(&world, look, eye)
    } else {
        eye
    };
    let desired = eye;
    let eye = camera_settings.resolve(look, desired, safe, time.delta_secs());
    trace.record(time.elapsed_secs_f64(), look, desired, safe, eye);
    let placed = Transform::from_translation(eye).looking_at(look, Vec3::Y);
    for mut cam in &mut cams {
        *cam = placed;
    }
}

/// Test entry point; gameplay contacts also retain their attacker's identity.
#[cfg(test)]
pub(super) fn hurt_hero(run: &mut Run, raw: f32) -> f32 {
    super::defense::hurt(run, raw).health_removed()
}

fn advance_buffs(run: &mut Run, dt: f32) {
    for t in [
        &mut run.shield_t,
        &mut run.immune_t,
        &mut run.thorns_t,
        &mut run.enrage_t,
        &mut run.ancestral_healing_t,
        &mut run.inspiration_t,
        &mut run.flurry_t,
        &mut run.vengeance_t,
    ] {
        *t = (*t - dt).max(0.0);
    }
}

/// Consume only the active part of the frame; healing cannot revive a lethal hit.
fn regenerate(run: &mut Run, stats: &super::Stats, dt: f32) {
    if run.hp <= 0.0 || run.dead_t.is_some() {
        return;
    }
    let mut heal = stats.regen * dt + run.blessed_recovery.advance(run.milestones, dt);
    heal += run.healing.advance(dt);
    if run.blood_craze_t > 0.0 {
        heal +=
            stats.max_hp * run.milestones.blood_craze_fraction() * dt.min(run.blood_craze_t) / 6.0;
        run.blood_craze_t = (run.blood_craze_t - dt).max(0.0);
    }
    if run.spirit_tap_t > 0.0 {
        heal += stats.max_hp * run.milestones.spirit_tap_regen() * dt.min(run.spirit_tap_t);
        run.spirit_tap_t = (run.spirit_tap_t - dt).max(0.0);
    }
    run.heal(heal, stats.max_hp);
}

/// The clock, regeneration, the timed buffs and their visuals, death and victory.
pub(super) fn hero_vitals(
    time: Res<Time>,
    selection: Res<Selection>,
    mut run: ResMut<Run>,
    mut next: ResMut<NextState<Phase>>,
    mut hero: Query<(Entity, &Guid, &mut ObjectStore, &mut Marks), With<Hero>>,
    mut fields: MessageWriter<FieldChanged>,
    mut fx: Fx,
) {
    let dt = time.delta_secs();
    let stats = run.stats();
    let Ok((entity, guid, mut store, mut marks)) = hero.single_mut() else {
        return;
    };
    if let Some(t) = run.dead_t.as_mut() {
        *t += dt;
        if *t > 3.0 {
            next.set(Phase::GameOver);
        }
        return;
    }
    run.elapsed += dt;
    advance_buffs(&mut run, dt);
    regenerate(&mut run, &stats, dt);

    // The buffs show as their own state kits: the shield bubble, the immunity glow, the orbs.
    let mut shown: Vec<u32> = Vec::new();
    if run.shield_t > 0.0 && run.shield > 0.0 {
        shown.push(17);
    }
    if run.immune_t > 0.0 {
        shown.push(642);
    }
    if run.thorns_t > 0.0 {
        shown.push(run.thorns_spell);
    }
    if run.ghostly_strike.remaining(run.elapsed) > 0.0 {
        shown.push(14278);
    }
    if run.healing.contains("rejuvenation") {
        shown.push(774);
    }
    if run.healing.contains("regrowth") {
        shown.push(8936);
    }
    for owned in &run.abilities {
        if owned.def.key == "lightning_shield" || owned.def.kind == super::data::Kind::Aspect {
            shown.push(owned.def.spell);
        }
    }
    if run.passives.iter().any(|(p, _)| p.key == "renew") {
        shown.push(139);
    }
    marks.set(&shown);
    marks.flush(entity, guid.0, &mut store, &mut fields);

    if run.hp <= 0.0 {
        run.hp = 0.0;
        run.dead_t = Some(0.0);
        units::kill_unit(&mut store);
        marks.clear();
        marks.flush(entity, guid.0, &mut store, &mut fields);
    } else if super::encounters::won(selection.map, run.elapsed, run.bosses_defeated) {
        run.won = true;
        // A last flourish before the results.
        fx.cast(entity, 642, &[entity], None);
        next.set(Phase::GameOver);
    }
}

/// The ground-plane distance from the hero, the measure every range in the mode uses.
/// The movement flags for running `dir` while facing `look`: forward, backpedal or strafe, by
/// which quarter the run falls in, so the walk, backpedal and strafe animations play.
fn gait(look: Vec3, dir: Vec3) -> u32 {
    let ahead = look.dot(dir);
    // Positive when `dir` is to the hero's left: WoW faces −Z with +X on the right.
    let left = look.cross(dir).y;
    let quarter = std::f32::consts::FRAC_1_SQRT_2;
    if ahead >= quarter {
        move_flags::FORWARD
    } else if ahead <= -quarter {
        move_flags::BACKWARD
    } else if left > 0.0 {
        move_flags::STRAFE_LEFT
    } else {
        move_flags::STRAFE_RIGHT
    }
}

pub(super) fn reach(hero: Vec3, other: Vec3) -> f32 {
    flat(other - hero).length()
}

#[cfg(test)]
mod reactive_talent_tests {
    use super::*;

    #[test]
    fn inspiration_refreshes_armor_without_stacking_and_expires() {
        let mut run = Run {
            class: &data::CLASSES[4],
            milestones: Milestones {
                class: 4,
                ..Default::default()
            }
            .with_ranks(&[(1, 361, 3)]),
            ..Default::default()
        };
        run.bonus.armor = 100.;
        let base = run.stats().armor;
        for (key, healed) in [
            ("renew", 10.),
            ("holy_shock", 10.),
            ("holy_nova", 0.),
            ("holy_nova", -1.),
            ("holy_nova", f32::NAN),
        ] {
            run.record_inspiration_healing(key, healed);
            assert_eq!(run.inspiration_t, 0.);
        }
        run.record_inspiration_healing("holy_nova", 1.);
        assert!((run.stats().armor - base * 1.24).abs() < 0.0001);
        advance_buffs(&mut run, 6.);
        assert_eq!(run.inspiration_t, 9.);
        run.record_inspiration_healing("holy_nova", 2.);
        assert_eq!(run.inspiration_t, 15.);
        assert!((run.stats().armor - base * 1.24).abs() < 0.0001);
        advance_buffs(&mut run, 20.);
        assert_eq!(run.stats().armor, base);
        run.record_inspiration_healing("holy_nova", 1.);
        run.milestones = run.milestones.with_ranks(&[]);
        assert_eq!(run.stats().armor, base);
        run.milestones = run.milestones.with_ranks(&[(1, 361, 3)]);
        run.hp = 0.;
        assert_eq!(run.stats().armor, base);
        run.inspiration_t = 0.;
        run.record_inspiration_healing("holy_nova", 1.);
        assert_eq!(run.inspiration_t, 0.);
    }

    #[test]
    fn blessed_recovery_tracks_only_nonlethal_health_damage_and_never_banks_overheal() {
        let mut run = Run {
            class: &super::super::data::CLASSES[4],
            milestones: Milestones {
                class: 4,
                ..Default::default()
            }
            .with_ranks(&[(1, 1636, 3)]),
            hp: 80.,
            ..Default::default()
        };
        let mut stats = run.stats();
        stats.regen = 0.;
        for shielded in [true, false] {
            run.shield = if shielded { 100. } else { 0. };
            run.shield_t = 10.;
            run.immune_t = if shielded { 0. } else { 1. };
            assert_eq!(hurt_hero(&mut run, 10.), 0.);
            assert_eq!(run.blessed_recovery.remaining(run.milestones), 0.);
        }
        run.immune_t = 0.;
        let damage = hurt_hero(&mut run, 20.);
        let after_hit = run.hp;
        assert_eq!(run.blessed_recovery.remaining(run.milestones), 6.);
        regenerate(&mut run, &stats, 0.);
        assert_eq!(run.hp, after_hit);
        regenerate(&mut run, &stats, 3.);
        assert!((run.hp - after_hit - damage * 0.105).abs() < 0.0001);
        let second_damage = hurt_hero(&mut run, 10.);
        let before_heal = run.hp;
        regenerate(&mut run, &stats, 30.);
        assert!((run.hp - before_heal - damage * 0.105 - second_damage * 0.21).abs() < 0.0001);
        assert_eq!(run.blessed_recovery.remaining(run.milestones), 0.);
        hurt_hero(&mut run, 10.);
        run.hp = stats.max_hp;
        regenerate(&mut run, &stats, 6.);
        assert_eq!(run.hp, stats.max_hp);
        assert_eq!(run.blessed_recovery.remaining(run.milestones), 0.);
        hurt_hero(&mut run, 10.);
        run.milestones = run.milestones.with_ranks(&[]);
        let hp = run.hp;
        regenerate(&mut run, &stats, 6.);
        assert_eq!(run.hp, hp);
        run.milestones = run.milestones.with_ranks(&[(1, 1636, 3)]);
        hurt_hero(&mut run, 10.);
        hurt_hero(&mut run, 10000.);
        let hp = run.hp;
        regenerate(&mut run, &stats, 6.);
        assert_eq!(run.hp, hp);
        assert!(!run.alive());
    }

    #[test]
    fn deterrence_reduces_hits_before_shields_only_during_trained_windows() {
        for class in [0, 2, 3] {
            for rank in [0, 1, 255] {
                for elapsed in [0., 9.99, 10., 180., 190., 905., 920.] {
                    let mut run = Run {
                        class: &super::super::data::CLASSES[class],
                        milestones: survivors_core::talents::Milestones {
                            class,
                            ..Default::default()
                        }
                        .with_ranks(&[(2, 1308, rank)]),
                        elapsed,
                        hp: 100.,
                        shield: 3.,
                        shield_t: 2.,
                        ..Default::default()
                    };
                    run.bonus.armor = 40.;
                    run.bonus.reduction = 0.1;
                    let stats = run.stats();
                    let active = class == 2 && rank > 0 && elapsed % 180. < 10.;
                    let mitigated = super::super::sheet::mitigate(
                        if active { 15. } else { 30. },
                        stats.armor,
                        stats.reduction,
                    );
                    let damage = hurt_hero(&mut run, 30.);
                    assert!((damage - (mitigated - 3.).max(0.)).abs() < 0.00001);
                    assert!((run.hp - (100. - damage)).abs() < 0.00001);
                    assert!((run.shield - (3. - mitigated).max(0.)).abs() < 0.00001);
                    assert_eq!(run.shield_t, 2.);
                    run.immune_t = 1.;
                    assert_eq!(hurt_hero(&mut run, 100.), 0.);
                    run.immune_t = 0.;
                    run.hp = 0.;
                    assert_eq!(hurt_hero(&mut run, 100.), 0.);
                }
            }
        }
    }

    #[test]
    fn sanctuary_prevents_each_hit_before_shields_without_healing_or_refreshing() {
        for class in [0, 1, 4] {
            for rank in [0, 1, 255] {
                for raw in [0., 1., 2., 5., 30.] {
                    let mut run = Run {
                        class: &super::super::data::CLASSES[class],
                        milestones: survivors_core::talents::Milestones {
                            class,
                            ..Default::default()
                        }
                        .with_ranks(&[(1, 1431, rank)]),
                        hp: 100.,
                        shield: 3.,
                        shield_t: 2.,
                        ..Default::default()
                    };
                    run.bonus.armor = 40.;
                    run.bonus.reduction = 0.1;
                    let stats = run.stats();
                    let expected = if raw <= 0. {
                        0.
                    } else {
                        (super::super::sheet::mitigate(raw, stats.armor, stats.reduction)
                            - if class == 1 && rank > 0 { 2. } else { 0. })
                        .max(0.)
                    };
                    assert!((hurt_hero(&mut run, raw) - (expected - 3.).max(0.)).abs() < 0.00001);
                    assert!((run.shield - (3. - expected).max(0.)).abs() < 0.00001);
                    assert_eq!(run.shield_t, 2.);
                    assert!(run.hp <= 100.);
                    run.shield = 0.;
                    assert!((hurt_hero(&mut run, raw) - expected).abs() < 0.00001);
                    run.immune_t = 1.;
                    let hp = run.hp;
                    assert_eq!(hurt_hero(&mut run, 100.), 0.);
                    assert_eq!(run.hp, hp);
                    run.immune_t = 0.;
                    run.hp = 0.;
                    assert_eq!(hurt_hero(&mut run, 100.), 0.);
                }
            }
        }
    }

    use survivors_core::talents::Milestones;

    #[test]
    fn shared_healing_caps_and_never_revives_a_lethal_hit() {
        let mut run = Run {
            hp: 90.,
            ..Default::default()
        };
        assert_eq!(run.heal(30., 100.), 10.);
        for invalid in [-1., f32::NAN, f32::INFINITY] {
            assert_eq!(run.heal(invalid, 100.), 0.);
        }
        assert_eq!(run.hp, 100.);
        run.hp = 0.1;
        hurt_hero(&mut run, 1.);
        let lethal = run.hp;
        assert!(!run.alive());
        assert_eq!(run.heal(1000., 100.), 0.);
        assert_eq!(hurt_hero(&mut run, 10.), 0.);
        assert_eq!(run.hp, lethal);
        run.hp = 10.;
        run.dead_t = Some(0.);
        assert_eq!(run.heal(100., 100.), 0.);
        assert_eq!(run.hp, 10.);
    }

    #[test]
    fn vengeance_refreshes_scoped_cast_damage_and_expires_without_stacking() {
        let mut run = Run {
            milestones: Milestones {
                class: 1,
                ..Default::default()
            }
            .with_ranks(&[(2, 1402, 5), (2, 1409, 1)]),
            ..Default::default()
        };
        let params = |key, run: &Run| {
            super::super::abilities::Params::new(
                &super::super::Owned {
                    def: data::ability(key),
                    level: 1,
                    cd: 0.,
                },
                &run.stats(),
                run.milestones,
            )
        };
        let before = params("judgement", &run);
        let other = params("smite", &run);
        run.record_vengeance_hit("judgement", 10., false);
        assert_eq!(run.vengeance_t, 0.);
        run.record_vengeance_hit("judgement", 10., true);
        assert_eq!(run.vengeance_t, 8.);
        let after = params("judgement", &run);
        assert!((after.dmg - before.dmg * 1.15).abs() < 0.0001);
        assert_eq!(params("smite", &run).dmg, other.dmg);
        assert_eq!(after.count, before.count);
        assert_eq!(after.cooldown, before.cooldown);
        let ground = params("consecration", &run);
        advance_buffs(&mut run, 3.);
        run.record_vengeance_hit("holy_shock", 10., true);
        assert_eq!(run.vengeance_t, 8.);
        assert_eq!(params("judgement", &run).dmg, after.dmg);
        advance_buffs(&mut run, 9.);
        assert_eq!(params("judgement", &run).dmg, before.dmg);
        assert!((ground.dmg - params("consecration", &run).dmg * 1.15).abs() < 0.0001);
        run.record_vengeance_hit("judgement", 10., true);
        run.milestones = run.milestones.with_ranks(&[(2, 1409, 1)]);
        assert_eq!(params("judgement", &run).dmg, before.dmg);
        run.hp = 0.;
        run.vengeance_t = 0.;
        run.record_vengeance_hit("judgement", 10., true);
        assert_eq!(run.vengeance_t, 0.);
    }

    #[test]
    fn ancestral_healing_requires_effective_totem_healing_and_expires() {
        let mut run = Run {
            hp: 50.,
            milestones: survivors_core::talents::Milestones {
                class: 5,
                ..Default::default()
            }
            .with_ranks(&[(2, 581, 3)]),
            ..Default::default()
        };
        let base = run.stats().armor;
        for (source, amount) in [
            ("renew", 10.),
            ("searing_totem", 0.),
            ("magma_totem", f32::NAN),
        ] {
            run.record_totem_healing(source, amount);
            assert_eq!(run.ancestral_healing_t, 0.);
        }
        run.record_totem_healing("searing_totem", 1.);
        assert_eq!(run.stats().armor, base + 36.);
        advance_buffs(&mut run, 6.);
        assert_eq!(run.ancestral_healing_t, 4.);
        run.record_totem_healing("magma_totem", 2.);
        assert_eq!(run.ancestral_healing_t, 10.);
        assert_eq!(run.stats().armor, base + 36.);
        advance_buffs(&mut run, 11.);
        assert_eq!(run.stats().armor, base);
        run.record_totem_healing("searing_totem", 1.);
        run.hp = 0.;
        assert_eq!(run.stats().armor, base);
        run.ancestral_healing_t = 0.;
        run.record_totem_healing("searing_totem", 1.);
        assert_eq!(run.ancestral_healing_t, 0.);
    }

    #[test]
    fn flurry_changes_cooldowns_refreshes_and_expires() {
        for (class, talent, spell, refresh) in [
            (0, 156, "cleave", "heroic_strike"),
            (5, 602, "lightning_bolt", "earth_shock"),
        ] {
            let mut run = Run {
                class: &data::CLASSES[class],
                milestones: Milestones {
                    class,
                    ..Default::default()
                }
                .with_ranks(&[(1, talent, 5)]),
                ..Default::default()
            };
            let owned = super::super::Owned {
                def: data::ability(spell),
                level: 1,
                cd: 0.0,
            };
            let before = run.stats();
            run.record_flurry_hit(spell, 10., false);
            assert_eq!(run.flurry_t, 0.);
            run.record_flurry_hit(spell, 10., true);
            assert_eq!(run.flurry_t, 3.);
            assert!(
                (owned.cooldown(&run.stats()) - owned.cooldown(&before) * 0.75).abs() < 0.00001
            );
            advance_buffs(&mut run, 2.);
            run.record_flurry_hit(refresh, 10., true);
            assert_eq!(run.flurry_t, 3.);
            assert_eq!(run.stats().haste, before.haste + 0.25);
            advance_buffs(&mut run, 4.);
            assert_eq!(run.stats().haste, before.haste);
            run.hp = 0.;
            run.record_flurry_hit(spell, 10., true);
            assert_eq!(run.flurry_t, 0.);
        }
    }

    #[test]
    fn spirit_tap_refreshes_without_stacking_and_heals_only_active_time() {
        for steps in [vec![20.0], vec![1.0; 20], vec![0.25; 80]] {
            let mut run = Run {
                hp: 20.0,
                milestones: Milestones {
                    class: 4,
                    ..Default::default()
                }
                .with_ranks(&[(2, 465, 5)]),
                ..Default::default()
            };
            let mut rng = super::super::Rng(42);
            let mut stats = run.stats();
            stats.regen = 0.0;
            run.record_spirit_tap_kill(&mut rng);
            assert_eq!(run.spirit_tap_t, 15.0);
            regenerate(&mut run, &stats, 2.0);
            run.record_spirit_tap_kill(&mut rng);
            assert_eq!(run.spirit_tap_t, 15.0);
            let before = run.hp;
            for dt in steps {
                regenerate(&mut run, &stats, dt);
            }
            assert!((run.hp - before - stats.max_hp * 0.15).abs() < 0.0001);
            assert_eq!(run.spirit_tap_t, 0.0);
            run.hp = stats.max_hp - 0.01;
            run.record_spirit_tap_kill(&mut rng);
            regenerate(&mut run, &stats, 1.0);
            assert_eq!(run.hp, stats.max_hp);
            run.hp = 0.0;
            let remaining = run.spirit_tap_t;
            run.record_spirit_tap_kill(&mut rng);
            assert_eq!(run.spirit_tap_t, remaining);
            regenerate(&mut run, &stats, 5.0);
            assert_eq!(run.hp, 0.0);
            run.hp = 20.0;
            run.milestones = run.milestones.with_ranks(&[]);
            regenerate(&mut run, &stats, 5.0);
            assert_eq!(run.hp, 20.0);
        }
    }

    #[test]
    fn spirit_tap_failed_roll_preserves_remaining_duration_and_untrained_rng() {
        let seed = (1..10000)
            .find(|seed| super::super::Rng(*seed).f32() > 0.2)
            .unwrap();
        let mut rng = super::super::Rng(seed);
        let mut run = Run::default();
        run.record_spirit_tap_kill(&mut rng);
        assert_eq!(rng.0, seed);
        run.milestones = Milestones {
            class: 4,
            ..Default::default()
        }
        .with_ranks(&[(2, 465, 1)]);
        run.spirit_tap_t = 7.0;
        run.record_spirit_tap_kill(&mut rng);
        assert_eq!(run.spirit_tap_t, 7.0);
    }

    #[test]
    fn blood_craze_refreshes_and_heals_only_its_active_time() {
        for steps in [vec![8.0], vec![1.0; 8], vec![0.25; 32]] {
            let mut run = Run {
                milestones: Milestones::default().with_ranks(&[(1, 661, 3)]),
                hp: 60.,
                immune_t: 1.,
                ..Default::default()
            };
            hurt_hero(&mut run, 10.);
            assert_eq!(run.blood_craze_t, 0.);
            run.immune_t = 0.;
            run.shield_t = 3.;
            run.shield = 10.;
            hurt_hero(&mut run, 10.);
            assert_eq!(run.blood_craze_t, 0.);
            hurt_hero(&mut run, 10.);
            assert_eq!(run.blood_craze_t, 6.);
            let mut stats = run.stats();
            stats.regen = 0.;
            regenerate(&mut run, &stats, 2.);
            hurt_hero(&mut run, 1.);
            assert_eq!(run.blood_craze_t, 6.);
            let before = run.hp;
            for dt in steps {
                regenerate(&mut run, &stats, dt);
            }
            assert!((run.hp - before - stats.max_hp * 0.03).abs() < 0.0001);
            assert_eq!(run.blood_craze_t, 0.);
            run.hp = stats.max_hp - 0.01;
            run.blood_craze_t = 6.;
            regenerate(&mut run, &stats, 6.);
            assert_eq!(run.hp, stats.max_hp);
            run.hp = 0.1;
            hurt_hero(&mut run, 1.);
            let lethal = run.hp;
            stats.regen = 100.;
            run.healing.refresh("rejuvenation", 500., 5.);
            run.blood_craze_t = 6.;
            regenerate(&mut run, &stats, 1.);
            assert_eq!(run.hp, lethal, "no regeneration path revives a lethal hit");
        }
    }

    #[test]
    fn enrage_requires_health_damage_refreshes_and_expires() {
        let mut run = Run {
            milestones: Milestones::default().with_ranks(&[(1, 155, 5)]),
            ..Default::default()
        };
        let base = run.stats().might;
        run.immune_t = 1.;
        assert_eq!(hurt_hero(&mut run, 10.), 0.);
        assert_eq!(run.enrage_t, 0.);
        run.immune_t = 0.;
        run.shield = 20.;
        run.shield_t = 3.;
        assert_eq!(hurt_hero(&mut run, 10.), 0.);
        assert_eq!(run.enrage_t, 0.);
        assert!(hurt_hero(&mut run, 20.) > 0.);
        assert_eq!(run.enrage_t, 4.);
        assert!((run.stats().might - base * 1.2).abs() < 0.00001);
        advance_buffs(&mut run, 2.);
        hurt_hero(&mut run, 1.);
        assert_eq!(run.enrage_t, 4.);
        assert!((run.stats().might - base * 1.2).abs() < 0.00001);
        advance_buffs(&mut run, 5.);
        assert_eq!(run.enrage_t, 0.);
        assert_eq!(run.stats().might, base);
        run.hp = 1.;
        hurt_hero(&mut run, 100.);
        assert_eq!(run.enrage_t, 0., "lethal damage cannot activate Enrage");
    }
}

#[cfg(test)]
mod periodic_healing_tests {
    use super::*;

    #[test]
    fn independent_heals_expire_during_overheal_and_do_not_revive() {
        let mut run = Run {
            hp: 10.,
            ..Default::default()
        };
        let mut stats = run.stats();
        stats.regen = 0.;
        stats.max_hp = 100.;
        run.healing.refresh("rejuvenation", 30., 10.);
        run.healing.refresh("other_heal", 12., 6.);
        regenerate(&mut run, &stats, 0.);
        assert_eq!(run.hp, 10.);
        regenerate(&mut run, &stats, 2.);
        assert_eq!(run.hp, 20.);
        run.hp = 99.;
        regenerate(&mut run, &stats, 8.);
        assert_eq!(run.hp, 100.);
        run.hp = 50.;
        regenerate(&mut run, &stats, 1.);
        assert_eq!(run.hp, 50., "overhealing is not stored for later damage");
        assert!(!run.healing.contains("rejuvenation"));
        assert!(!run.healing.contains("other_heal"));
        for (hp, dead) in [(0., None), (10., Some(0.))] {
            run.hp = hp;
            run.dead_t = dead;
            run.healing.refresh("rejuvenation", 30., 10.);
            regenerate(&mut run, &stats, 20.);
            assert_eq!(run.hp, hp);
        }
        assert_eq!(Run::default().healing, Default::default());
    }
}
