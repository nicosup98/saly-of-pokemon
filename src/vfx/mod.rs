use bevy::prelude::*;
use bevy_hanabi::prelude::*;
use crate::combat::events::{CombatAnimEvent, CombatAnimKind, CombatSide};
use crate::run::AppState;

pub struct HanabiVfxPlugin;

#[derive(Component)]
pub struct MenuHanabiEmitter;

/// Marcador con temporizador para limpiar automáticamente los emisores de partículas de un solo uso
/// en combate y evitar la acumulación de buffers GPU frame a frame.
#[derive(Component)]
pub struct TransientParticleEffect {
    pub timer: Timer,
}

impl Plugin for HanabiVfxPlugin {
    fn build(&self, app: &mut App) {
        if app.world().get_resource::<Assets<EffectAsset>>().is_some() {
            app.add_systems(Startup, setup_hanabi_effects)
                .add_systems(OnEnter(AppState::MainMenu), spawn_menu_hanabi_particles)
                .add_systems(OnExit(AppState::MainMenu), cleanup_menu_hanabi_particles)
                .add_systems(OnExit(AppState::Combat), cleanup_transient_combat_particles)
                .add_systems(
                    Update,
                    (
                        handle_combat_particle_events,
                        tick_and_cleanup_transient_particles,
                    ),
                );
        }
    }
}

#[derive(Resource, Clone)]
pub struct CombatParticleEffects {
    pub strike_effect: Handle<EffectAsset>,
    pub sparkle_effect: Handle<EffectAsset>,
    pub menu_ambient_effect: Handle<EffectAsset>,
}

fn setup_hanabi_effects(
    mut commands: Commands,
    mut effects: ResMut<Assets<EffectAsset>>,
) {
    // 1. Efecto de Impacto / Golpe (Strike)
    let writer = ExprWriter::new();
    let age = writer.lit(0.).expr();
    let init_age = SetAttributeModifier::new(Attribute::AGE, age);
    let lifetime = writer.lit(0.65).expr();
    let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, lifetime);
    let size = writer.lit(14.0).expr();
    let init_size = SetAttributeModifier::new(Attribute::SIZE, size);

    let init_pos = SetPositionCircleModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        axis: writer.lit(Vec3::Z).expr(),
        radius: writer.lit(25.0).expr(),
        dimension: ShapeDimension::Surface,
    };

    let init_vel = SetVelocityCircleModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        axis: writer.lit(Vec3::Z).expr(),
        speed: writer.lit(140.0).expr(),
    };

    let module = writer.finish();
    let spawner = SpawnerSettings::once(28.0.into());

    let strike_effect = effects.add(
        EffectAsset::new(128, spawner, module)
            .with_name("strike_particles")
            .init(init_pos)
            .init(init_vel)
            .init(init_age)
            .init(init_lifetime)
            .init(init_size)
            .render(ColorOverLifetimeModifier::new({
                let mut g = bevy_hanabi::Gradient::new();
                g.add_key(0.0, Vec4::new(1.0, 0.45, 0.1, 1.0));
                g.add_key(1.0, Vec4::new(1.0, 0.9, 0.2, 0.0));
                g
            })),
    );

    // 2. Efecto de Destellos / Buff / Celebración (Sparkle)
    let writer2 = ExprWriter::new();
    let age2 = writer2.lit(0.).expr();
    let init_age2 = SetAttributeModifier::new(Attribute::AGE, age2);
    let lifetime2 = writer2.lit(1.0).expr();
    let init_lifetime2 = SetAttributeModifier::new(Attribute::LIFETIME, lifetime2);
    let size2 = writer2.lit(12.0).expr();
    let init_size2 = SetAttributeModifier::new(Attribute::SIZE, size2);

    let init_pos2 = SetPositionCircleModifier {
        center: writer2.lit(Vec3::ZERO).expr(),
        axis: writer2.lit(Vec3::Z).expr(),
        radius: writer2.lit(40.0).expr(),
        dimension: ShapeDimension::Volume,
    };

    let init_vel2 = SetVelocityCircleModifier {
        center: writer2.lit(Vec3::ZERO).expr(),
        axis: writer2.lit(Vec3::Z).expr(),
        speed: writer2.lit(70.0).expr(),
    };

    let module2 = writer2.finish();
    let spawner2 = SpawnerSettings::once(32.0.into());

    let sparkle_effect = effects.add(
        EffectAsset::new(128, spawner2, module2)
            .with_name("sparkle_particles")
            .init(init_pos2)
            .init(init_vel2)
            .init(init_age2)
            .init(init_lifetime2)
            .init(init_size2)
            .render(ColorOverLifetimeModifier::new({
                let mut g = bevy_hanabi::Gradient::new();
                g.add_key(0.0, Vec4::new(0.2, 0.85, 1.0, 1.0));
                g.add_key(1.0, Vec4::new(0.9, 0.4, 1.0, 0.0));
                g
            })),
    );

    // 3. Efecto Ambiental Continuo del Menú (Menu Ambient Embers)
    let writer3 = ExprWriter::new();
    let age3 = writer3.lit(0.).expr();
    let init_age3 = SetAttributeModifier::new(Attribute::AGE, age3);
    let lifetime3 = writer3.lit(3.0).expr();
    let init_lifetime3 = SetAttributeModifier::new(Attribute::LIFETIME, lifetime3);
    let size3 = writer3.lit(10.0).expr();
    let init_size3 = SetAttributeModifier::new(Attribute::SIZE, size3);

    let init_pos3 = SetPositionCircleModifier {
        center: writer3.lit(Vec3::ZERO).expr(),
        axis: writer3.lit(Vec3::Z).expr(),
        radius: writer3.lit(420.0).expr(),
        dimension: ShapeDimension::Volume,
    };

    let init_vel3 = SetVelocityCircleModifier {
        center: writer3.lit(Vec3::ZERO).expr(),
        axis: writer3.lit(Vec3::Z).expr(),
        speed: writer3.lit(35.0).expr(),
    };

    let module3 = writer3.finish();
    let spawner3 = SpawnerSettings::rate(25.0.into());

    let menu_ambient_effect = effects.add(
        EffectAsset::new(512, spawner3, module3)
            .with_name("menu_ambient_particles")
            .init(init_pos3)
            .init(init_vel3)
            .init(init_age3)
            .init(init_lifetime3)
            .init(init_size3)
            .render(ColorOverLifetimeModifier::new({
                let mut g = bevy_hanabi::Gradient::new();
                g.add_key(0.0, Vec4::new(1.0, 0.55, 0.15, 0.9));
                g.add_key(0.5, Vec4::new(1.0, 0.85, 0.25, 0.7));
                g.add_key(1.0, Vec4::new(1.0, 0.3, 0.1, 0.0));
                g
            })),
    );

    commands.insert_resource(CombatParticleEffects {
        strike_effect,
        sparkle_effect,
        menu_ambient_effect,
    });
}

fn spawn_menu_hanabi_particles(
    mut commands: Commands,
    particles: Option<Res<CombatParticleEffects>>,
) {
    let Some(particles) = particles else { return };
    commands.spawn((
        MenuHanabiEmitter,
        ParticleEffect::new(particles.menu_ambient_effect.clone()),
        Transform::from_translation(Vec3::new(0.0, 0.0, 5.0)),
        Visibility::Visible,
    ));
}

fn cleanup_menu_hanabi_particles(
    mut commands: Commands,
    q_emitters: Query<Entity, With<MenuHanabiEmitter>>,
) {
    for entity in q_emitters.iter() {
        commands.entity(entity).despawn();
    }
}

fn handle_combat_particle_events(
    mut commands: Commands,
    mut ev_anim: MessageReader<CombatAnimEvent>,
    particles: Option<Res<CombatParticleEffects>>,
) {
    let Some(particles) = particles else { return };

    for ev in ev_anim.read() {
        let pos = match ev.side {
            CombatSide::Enemy => Vec3::new(260.0, 100.0, 10.0),
            CombatSide::Player => Vec3::new(-260.0, -80.0, 10.0),
        };

        match ev.kind {
            CombatAnimKind::Strike | CombatAnimKind::Damage(_) => {
                commands.spawn((
                    TransientParticleEffect {
                        timer: Timer::from_seconds(0.85, TimerMode::Once),
                    },
                    ParticleEffect::new(particles.strike_effect.clone()),
                    Transform::from_translation(pos),
                    Visibility::Visible,
                ));
            }
            CombatAnimKind::Buff | CombatAnimKind::Heal(_) | CombatAnimKind::Block(_) => {
                commands.spawn((
                    TransientParticleEffect {
                        timer: Timer::from_seconds(1.15, TimerMode::Once),
                    },
                    ParticleEffect::new(particles.sparkle_effect.clone()),
                    Transform::from_translation(pos),
                    Visibility::Visible,
                ));
            }
            _ => {}
        }
    }
}

fn tick_and_cleanup_transient_particles(
    mut commands: Commands,
    time: Res<Time>,
    mut q_transient: Query<(Entity, &mut TransientParticleEffect)>,
) {
    let dt = time.delta();
    for (entity, mut transient) in q_transient.iter_mut() {
        transient.timer.tick(dt);
        if transient.timer.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}

fn cleanup_transient_combat_particles(
    mut commands: Commands,
    q_transient: Query<Entity, With<TransientParticleEffect>>,
) {
    for entity in q_transient.iter() {
        commands.entity(entity).despawn();
    }
}
