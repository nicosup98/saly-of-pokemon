use bevy::prelude::*;
use bevy_hanabi::prelude::*;
use crate::combat::events::{CombatAnimEvent, CombatAnimKind, CombatSide};

pub struct HanabiVfxPlugin;

impl Plugin for HanabiVfxPlugin {
    fn build(&self, app: &mut App) {
        if app.world().get_resource::<Assets<EffectAsset>>().is_some() {
            app.add_systems(Startup, setup_hanabi_effects)
                .add_systems(Update, handle_combat_particle_events);
        }
    }
}

#[derive(Resource, Clone)]
pub struct CombatParticleEffects {
    pub strike_effect: Handle<EffectAsset>,
    pub sparkle_effect: Handle<EffectAsset>,
}

fn setup_hanabi_effects(
    mut commands: Commands,
    mut effects: ResMut<Assets<EffectAsset>>,
) {
    // 1. Efecto de Impacto / Golpe (Strike)
    let writer = ExprWriter::new();
    let age = writer.lit(0.).expr();
    let init_age = SetAttributeModifier::new(Attribute::AGE, age);
    let lifetime = writer.lit(0.6).expr();
    let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, lifetime);

    let init_pos = SetPositionCircleModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        axis: writer.lit(Vec3::Z).expr(),
        radius: writer.lit(25.0).expr(),
        dimension: ShapeDimension::Surface,
    };

    let init_vel = SetVelocityCircleModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        axis: writer.lit(Vec3::Z).expr(),
        speed: writer.lit(120.0).expr(),
    };

    let module = writer.finish();
    let spawner = SpawnerSettings::once(25.0.into());

    let strike_effect = effects.add(
        EffectAsset::new(1024, spawner, module)
            .with_name("strike_particles")
            .init(init_pos)
            .init(init_vel)
            .init(init_age)
            .init(init_lifetime)
            .render(ColorOverLifetimeModifier::new({
                let mut g = bevy_hanabi::Gradient::new();
                g.add_key(0.0, Vec4::new(1.0, 0.4, 0.1, 1.0));
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

    let init_pos2 = SetPositionCircleModifier {
        center: writer2.lit(Vec3::ZERO).expr(),
        axis: writer2.lit(Vec3::Z).expr(),
        radius: writer2.lit(40.0).expr(),
        dimension: ShapeDimension::Volume,
    };

    let init_vel2 = SetVelocityCircleModifier {
        center: writer2.lit(Vec3::ZERO).expr(),
        axis: writer2.lit(Vec3::Z).expr(),
        speed: writer2.lit(60.0).expr(),
    };

    let module2 = writer2.finish();
    let spawner2 = SpawnerSettings::once(30.0.into());

    let sparkle_effect = effects.add(
        EffectAsset::new(1024, spawner2, module2)
            .with_name("sparkle_particles")
            .init(init_pos2)
            .init(init_vel2)
            .init(init_age2)
            .init(init_lifetime2)
            .render(ColorOverLifetimeModifier::new({
                let mut g = bevy_hanabi::Gradient::new();
                g.add_key(0.0, Vec4::new(0.2, 0.8, 1.0, 1.0));
                g.add_key(1.0, Vec4::new(0.9, 0.4, 1.0, 0.0));
                g
            })),
    );

    commands.insert_resource(CombatParticleEffects {
        strike_effect,
        sparkle_effect,
    });
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
                    ParticleEffect::new(particles.strike_effect.clone()),
                    Transform::from_translation(pos),
                    Visibility::Visible,
                ));
            }
            CombatAnimKind::Buff | CombatAnimKind::Heal(_) | CombatAnimKind::Block(_) => {
                commands.spawn((
                    ParticleEffect::new(particles.sparkle_effect.clone()),
                    Transform::from_translation(pos),
                    Visibility::Visible,
                ));
            }
            _ => {}
        }
    }
}
