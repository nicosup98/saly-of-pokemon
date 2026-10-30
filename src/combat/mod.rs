pub mod events;
pub mod ai;

use bevy::prelude::*;
use rand::{seq::SliceRandom, Rng};
use crate::card::{Card, InDiscardPile, InDrawPile, InHand, Exhausted};
use crate::map::generator::NodeType;
use crate::map::CurrentMap;
use crate::pokemon::{create_pokemon_from_species, check_evolution};
use crate::run::{AppState, CombatPhase, CurrentRunState, MetaProfile};
use crate::stats::PokemonStats;
use crate::types::{MoveCategory, StatusType};
use self::events::*;
use self::ai::{EnemyAI, EnemyIntentType};

#[derive(Component)]
pub struct PlayerPokemon;

#[derive(Component)]
pub struct EnemyPokemon;

#[derive(Component)]
pub struct CombatOutcomeResolved;

#[derive(Resource, Debug, Clone, Default)]
pub struct CombatVictoryData {
    pub enemy_name: String,
    pub enemy_trainer_name: String,
    pub enemy_trainer_id: u32,
    pub is_boss: bool,
    pub is_mini_boss: bool,
    pub player_name: String,
    pub player_species_id: u32,
    pub player_level: u32,
    pub exp_gained: u32,
    pub player_exp: u32,
    pub exp_to_next: u32,
    pub levels_gained: u32,
    pub money_gained: u32,
    pub tokens_gained: u32,
    pub fragment_reward: Option<String>,
    pub badge_reward: Option<String>,
    pub evolution_msg: Option<String>,
    pub reward_cards: Vec<crate::card::Card>,
    pub card_chosen: bool,
}

/// Calcula la cantidad de experiencia (XP) otorgada al derrotar a un enemigo:
/// - Los entrenadores comunes dan experiencia base moderada.
/// - Los minijefes otorgan sustancialmente más experiencia.
/// - El líder del gimnasio (jefe de nivel) otorga una gran bonificación de experiencia.
pub fn calculate_exp_reward(is_boss: bool, is_mini_boss: bool, enemy_level: u32) -> u32 {
    if is_boss {
        250 + enemy_level * 20
    } else if is_mini_boss {
        100 + enemy_level * 10
    } else {
        40 + enemy_level * 4
    }
}

pub fn roll_victory_cards(player_element: Option<crate::types::ElementType>) -> Vec<crate::card::Card> {
    use rand::seq::SliceRandom;
    let catalog = crate::card::create_master_card_catalog();
    let mut rng = rand::thread_rng();

    let mut result = Vec::new();
    if let Some(elem) = player_element {
        let affinity: Vec<crate::card::Card> = catalog.iter().filter(|c| c.element == elem).cloned().collect();
        if !affinity.is_empty() {
            for c in affinity.choose_multiple(&mut rng, 2) {
                result.push(c.clone());
            }
        }
    }

    let mut shuffled = catalog;
    shuffled.shuffle(&mut rng);
    for c in shuffled {
        if !result.iter().any(|r| r.id == c.id) {
            result.push(c);
            if result.len() >= 3 {
                break;
            }
        }
    }

    result
}

pub fn roll_enemy_encounter(
    node_type: NodeType,
    act: u8,
    floor: u8,
    player_level: u32,
) -> (u32, u32, crate::pokemon::TrainerSprite) {
    use rand::Rng;
    let mut rng = rand::thread_rng();

    match node_type {
        NodeType::GymLeader => {
            match act {
                1 => (95, 18, crate::pokemon::TrainerSprite::new(6, false, "Líder Brock")),
                2 => (121, 35, crate::pokemon::TrainerSprite::new(7, false, "Líder Misty")),
                _ => (150, 55, crate::pokemon::TrainerSprite::new(5, false, "Campeón de la Liga")),
            }
        }
        NodeType::MiniBoss => {
            // Pool variada y escalonada de Mini Jefes (Pokémon Élite y Salvajes Dominantes)
            let pool: Vec<(u32, u32, crate::pokemon::TrainerSprite)> = match act {
                1 => vec![
                    (123, player_level + 2, crate::pokemon::TrainerSprite::new(1, false, "Scyther Alfa")),
                    (143, player_level + 2, crate::pokemon::TrainerSprite::new(8, false, "Snorlax Gigante")),
                    (26, player_level + 2, crate::pokemon::TrainerSprite::new(11, false, "Raichu Dominante")),
                    (74, player_level + 3, crate::pokemon::TrainerSprite::new(8, false, "Geodude Titán")),
                    (5, player_level + 2, crate::pokemon::TrainerSprite::new(3, false, "Charmeleon Furia")),
                    (8, player_level + 2, crate::pokemon::TrainerSprite::new(0, false, "Wartortle Escudo")),
                    (2, player_level + 2, crate::pokemon::TrainerSprite::new(4, false, "Ivysaur Salvaje")),
                ],
                2 => vec![
                    (123, player_level + 3, crate::pokemon::TrainerSprite::new(1, false, "Scyther Depredador")),
                    (143, player_level + 3, crate::pokemon::TrainerSprite::new(8, false, "Snorlax Colosal")),
                    (26, player_level + 3, crate::pokemon::TrainerSprite::new(11, false, "Raichu Tormenta")),
                    (6, player_level + 2, crate::pokemon::TrainerSprite::new(3, false, "Charizard Salvaje")),
                    (9, player_level + 2, crate::pokemon::TrainerSprite::new(0, false, "Blastoise Acorazado")),
                    (3, player_level + 2, crate::pokemon::TrainerSprite::new(4, false, "Venusaur Titánico")),
                ],
                _ => vec![
                    (6, player_level + 4, crate::pokemon::TrainerSprite::new(3, false, "Charizard Volcánico")),
                    (9, player_level + 4, crate::pokemon::TrainerSprite::new(0, false, "Blastoise Fortaleza")),
                    (3, player_level + 4, crate::pokemon::TrainerSprite::new(4, false, "Venusaur Primigenio")),
                    (143, player_level + 4, crate::pokemon::TrainerSprite::new(8, false, "Snorlax Inmortal")),
                    (123, player_level + 4, crate::pokemon::TrainerSprite::new(1, false, "Scyther Segador")),
                    (26, player_level + 4, crate::pokemon::TrainerSprite::new(11, false, "Raichu Ancestral")),
                ],
            };
            if floor == 1 {
                pool[0].clone()
            } else {
                let index = rng.gen_range(0..pool.len());
                pool[index].clone()
            }
        }
        NodeType::Enemy | _ => {
            // Pool variada de Entrenadores Comunes y sus Pokémon según el Acto
            let pool: Vec<(u32, u32, crate::pokemon::TrainerSprite)> = match act {
                1 => vec![
                    (19, player_level, crate::pokemon::TrainerSprite::new(12, false, "Joven Joey")),     // Rattata
                    (10, player_level, crate::pokemon::TrainerSprite::new(1, false, "Cazabichos Sam")),    // Caterpie
                    (74, player_level, crate::pokemon::TrainerSprite::new(8, false, "Montañero Marcos")),  // Geodude
                    (25, player_level, crate::pokemon::TrainerSprite::new(2, false, "Entrenadora Sara")),  // Pikachu
                    (1, player_level, crate::pokemon::TrainerSprite::new(4, false, "Campista Leo")),       // Bulbasaur
                    (7, player_level, crate::pokemon::TrainerSprite::new(0, false, "Marinero Bruno")),     // Squirtle
                    (4, player_level, crate::pokemon::TrainerSprite::new(3, false, "Furtivo Ignis")),      // Charmander
                ],
                2 => vec![
                    (5, player_level, crate::pokemon::TrainerSprite::new(3, false, "Entrenador Guay Eric")),  // Charmeleon
                    (8, player_level, crate::pokemon::TrainerSprite::new(0, false, "Nadadora Marina")),       // Wartortle
                    (2, player_level, crate::pokemon::TrainerSprite::new(4, false, "Guardabosques Silva")),   // Ivysaur
                    (26, player_level, crate::pokemon::TrainerSprite::new(11, false, "Científico Edison")),   // Raichu
                    (74, player_level + 1, crate::pokemon::TrainerSprite::new(8, false, "Geólogo Rocky")),    // Geodude
                    (19, player_level + 1, crate::pokemon::TrainerSprite::new(12, false, "Experto Joey")),    // Rattata
                    (25, player_level + 1, crate::pokemon::TrainerSprite::new(2, false, "Voltio Sam")),       // Pikachu
                ],
                _ => vec![
                    (6, player_level, crate::pokemon::TrainerSprite::new(3, false, "As de la Liga Terry")),    // Charizard
                    (9, player_level, crate::pokemon::TrainerSprite::new(0, false, "As de la Liga Marina")),   // Blastoise
                    (3, player_level, crate::pokemon::TrainerSprite::new(4, false, "As de la Liga Flora")),    // Venusaur
                    (26, player_level + 1, crate::pokemon::TrainerSprite::new(11, false, "Médium Sabrina")),  // Raichu
                    (143, player_level, crate::pokemon::TrainerSprite::new(8, false, "Guardián Colosal")),     // Snorlax
                ],
            };

            let index = rng.gen_range(0..pool.len());
            pool[index].clone()
        }
    }
}

pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CombatVictoryData>()
            .add_message::<PlayCardEvent>()
            .add_message::<DealDamageEvent>()
            .add_message::<GainBlockEvent>()
            .add_message::<ApplyStatusEvent>()
            .add_message::<DrawCardEvent>()
            .add_message::<HealEvent>()
            .add_message::<EndPlayerTurnEvent>()
            .add_message::<UseConsumableEvent>()
            .add_message::<CombatAnimEvent>()
            .add_message::<DeckShuffleAnimEvent>()
            .add_systems(OnEnter(AppState::Combat), setup_combat)
            .add_systems(
                Update,
                (
                    handle_play_card,
                    handle_end_turn,
                    handle_use_consumable,
                    check_combat_outcome,
                    process_damage_events,
                    process_block_events,
                    process_heal_events,
                )
                    .chain()
                    .run_if(in_state(AppState::Combat)),
            );
    }
}

pub fn setup_combat(
    mut commands: Commands,
    mut run_state: ResMut<CurrentRunState>,
    map: Res<CurrentMap>,
    mut next_phase: ResMut<NextState<CombatPhase>>,
    existing_entities: Query<Entity, Or<(With<PlayerPokemon>, With<EnemyPokemon>, With<Card>)>>,
    mut ev_shuffle: Option<MessageWriter<DeckShuffleAnimEvent>>,
) {
    // Limpia entidades de combates previos
    for e in existing_entities.iter() {
        commands.entity(e).despawn();
    }

    run_state.battle_logs.clear();
    run_state.energy = run_state.max_energy;

    // Crea el Pokémon del jugador según su ID y nivel persistente
    let player_level = run_state.player_level();
    let mut player_stats = create_pokemon_from_species(run_state.selected_starter_id, player_level)
        .unwrap_or_else(|| create_pokemon_from_species(4, 5).unwrap());
    player_stats.exp = run_state.player_exp();

    // El HP se conserva entre combates (solo se recupera en el Centro Pokémon o con objetos)
    if let Some(hp) = run_state.player_hp {
        player_stats.hp = hp.clamp(1, player_stats.max_hp);
    }

    run_state.add_log(format!("[COMBATE] ¡Entras en combate con {} (Nv. {})!", player_stats.name, player_stats.level));

    // El tipo de rival depende del nodo que el jugador eligió en el mapa
    let node_type = map.node_type_at(run_state.current_floor, map.current_path_index);
    let is_boss = node_type == NodeType::GymLeader;
    let is_mini_boss = node_type == NodeType::MiniBoss;

    let (enemy_id, enemy_level, enemy_trainer) = roll_enemy_encounter(
        node_type,
        run_state.current_act,
        run_state.current_floor,
        player_level,
    );

    let mut enemy_stats = create_pokemon_from_species(enemy_id, enemy_level)
        .unwrap_or_else(|| create_pokemon_from_species(19, 5).unwrap());

    run_state.add_log(format!(
        "[COMBATE] ¡{} te desafía con {} (Nv. {})!",
        enemy_trainer.name, enemy_stats.name, enemy_stats.level
    ));

    let mut ai = EnemyAI::new(is_boss, is_mini_boss);
    let first_intent = ai.decide_next_intent(&enemy_stats);

    match &first_intent {
        EnemyIntentType::Attack { raw_damage, .. } => {
            let dmg = (raw_damage * PokemonStats::calc_mitigation(player_stats.def)).round() as i32;
            run_state.add_log(format!("[INTENCIÓN] {} atacará por {} de daño.", enemy_stats.name, dmg));
        }
        EnemyIntentType::Defend { block_amount } => {
            run_state.add_log(format!("[INTENCIÓN] {} ganará {} de Bloqueo.", enemy_stats.name, block_amount));
        }
        _ => {
            run_state.add_log(format!("[INTENCIÓN] {} usará habilidad de estado.", enemy_stats.name));
        }
    }

    // Aplica reliquias iniciales
    if run_state.has_relic("miracle_seed") {
        enemy_stats.apply_status(StatusType::LeechSeed, 3);
        run_state.add_log("[RELIQUIA] Semilla Milagro: Aplica 3 de Drenadoras al enemigo.");
    }

    let player_sprite = crate::pokemon::PokemonSprite::new(player_stats.species_id, true);
    let enemy_sprite = crate::pokemon::PokemonSprite::new(enemy_stats.species_id, false);
    let player_trainer = crate::pokemon::TrainerSprite::player_default();

    commands.spawn((player_stats, PlayerPokemon, player_sprite, player_trainer));
    commands.spawn((enemy_stats, EnemyPokemon, ai, enemy_sprite, enemy_trainer));

    // Si el mazo del jugador está vacío, genera el mazo inicial según la especie seleccionada
    if run_state.player_deck.is_empty() {
        let (elem, is_special) = if let Some(sp) = crate::pokemon::get_species_data(run_state.selected_starter_id) {
            (sp.primary_types.first().copied().unwrap_or(crate::types::ElementType::Normal), sp.base_sp_atk >= sp.base_atk)
        } else {
            (crate::types::ElementType::Fire, true)
        };
        run_state.player_deck = crate::card::create_starter_deck(elem, is_special);
    }

    // Spawnea las cartas en la pila de robo
    let mut deck = run_state.player_deck.clone();
    deck.shuffle(&mut rand::thread_rng());

    if let Some(ref mut ev_shuffle) = ev_shuffle {
        ev_shuffle.write(DeckShuffleAnimEvent { card_count: deck.len() });
    }

    for (i, card) in deck.into_iter().enumerate() {
        if i < 5 {
            // Roba 5 cartas para la mano inicial
            commands.spawn((card, InHand));
        } else {
            commands.spawn((card, InDrawPile));
        }
    }

    next_phase.set(CombatPhase::PlayerAction);
}

pub fn handle_play_card(
    mut ev_play: MessageReader<PlayCardEvent>,
    mut ev_anim: MessageWriter<CombatAnimEvent>,
    mut run_state: ResMut<CurrentRunState>,
    mut commands: Commands,
    mut q_player: Query<&mut PokemonStats, (With<PlayerPokemon>, Without<EnemyPokemon>)>,
    mut q_enemy: Query<&mut PokemonStats, (With<EnemyPokemon>, Without<PlayerPokemon>)>,
    mut q_cards: Query<(Entity, &mut Card, &InHand)>,
) {
    let Ok(mut player) = q_player.single_mut() else { return };
    let Ok(mut enemy) = q_enemy.single_mut() else { return };

    if enemy.is_fainted() || player.is_fainted() {
        return;
    }

    let mut hand_cards: Vec<(Entity, Mut<Card>)> = q_cards.iter_mut().map(|(e, c, _)| (e, c)).collect();

    for ev in ev_play.read() {
        if enemy.is_fainted() || player.is_fainted() {
            break;
        }

        if ev.card_index >= hand_cards.len() {
            continue;
        }

        let (card_entity, ref mut card) = hand_cards[ev.card_index];

        if run_state.energy < card.energy_cost {
            run_state.add_log("[AVISO] ¡No tienes suficiente Energía para jugar esta carta!");
            continue;
        }

        run_state.energy -= card.energy_cost;

        // Comprueba si aplica STAB (+20% si coincide con un tipo primario del jugador)
        let has_stab = player.primary_types.contains(&card.element);
        let stab_mult = if has_stab { 1.2 } else { 1.0 };

        // Efectividad elemental contra los tipos del enemigo
        let mut effectiveness = 1.0f32;
        for def_type in &enemy.primary_types {
            effectiveness *= card.element.effectiveness_against(*def_type);
        }

        let stab_text = if has_stab { " (¡STAB +20%!)" } else { "" };
        let eff_text = if effectiveness > 1.2 {
            " [¡Súper eficaz!]"
        } else if effectiveness < 0.8 {
            " [No muy eficaz...]"
        } else {
            ""
        };

        // Procesamiento por categoría
        match card.category {
            MoveCategory::Physical | MoveCategory::Special => {
                let stat = if card.category == MoveCategory::Physical {
                    player.effective_atk()
                } else {
                    player.effective_sp_atk()
                };

                // Bonificación de reliquias ofensivas
                let mut relic_bonus = 1.0f32;
                if run_state.has_relic("life_orb") {
                    relic_bonus += 0.30;
                    player.hp = (player.hp - 2).max(1);
                    run_state.add_log("[RELIQUIA] Vidasfera: Pierdes 2 HP de retroceso.");
                }
                if run_state.has_relic("expert_belt") && (has_stab || effectiveness > 1.2) {
                    relic_bonus += 0.25;
                }

                let raw_damage = stat * card.multiplier * stab_mult * effectiveness * relic_bonus;
                let (blocked, hp_damage) = enemy.take_damage(raw_damage, card.category);

                ev_anim.write(CombatAnimEvent { side: CombatSide::Player, kind: CombatAnimKind::Strike });
                ev_anim.write(CombatAnimEvent { side: CombatSide::Enemy, kind: CombatAnimKind::Damage(blocked + hp_damage) });

                run_state.add_log(format!(
                    "[ATAQUE] Juegas {}{}{}: Inflige {} daño ({} Bloqueo, {} HP).",
                    card.display_name(), stab_text, eff_text, blocked + hp_damage, blocked, hp_damage
                ));

                // Reliquia Campana Concha: infligir >= 25 daño cura 3 HP
                if run_state.has_relic("shell_bell") && (blocked + hp_damage) >= 25 {
                    player.hp = (player.hp + 3).min(player.max_hp);
                    ev_anim.write(CombatAnimEvent { side: CombatSide::Player, kind: CombatAnimKind::Heal(3) });
                    run_state.add_log("[RELIQUIA] Campana Concha: Recuperas 3 HP por daño masivo.");
                }

                // Generación de bloqueo secundario en la carta (ej. Pistola Agua, Surf)
                if card.secondary_block_multiplier > 0.0 {
                    let block_gain = (player.effective_sp_def() * card.secondary_block_multiplier).round() as i32;
                    player.add_block(block_gain);
                    ev_anim.write(CombatAnimEvent { side: CombatSide::Player, kind: CombatAnimKind::Block(block_gain) });
                    run_state.add_log(format!("[BLOQUEO] Ganas {} de Bloqueo adicional.", block_gain));
                }
            }
            MoveCategory::Defense => {
                let block_gain = (player.effective_def() * card.multiplier * stab_mult).round() as i32;
                player.add_block(block_gain);
                ev_anim.write(CombatAnimEvent { side: CombatSide::Player, kind: CombatAnimKind::Block(block_gain) });
                run_state.add_log(format!("[DEFENSA] Juegas {}: Ganas {} de Bloqueo.", card.display_name(), block_gain));
            }
            MoveCategory::Power | MoveCategory::Skill => {
                ev_anim.write(CombatAnimEvent { side: CombatSide::Player, kind: CombatAnimKind::Buff });
                run_state.add_log(format!("[ESTADO] Juegas {}.", card.display_name()));
            }
        }

        // Aplica estado secundario si la carta lo tiene
        if let Some((status, stacks)) = card.secondary_status {
            match status {
                StatusType::SwordDance | StatusType::NastyPlot | StatusType::IronDefense | StatusType::TrickRoom | StatusType::Plating => {
                    player.apply_status(status, stacks);
                    ev_anim.write(CombatAnimEvent { side: CombatSide::Player, kind: CombatAnimKind::Buff });
                    run_state.add_log(format!("[ESTADO] Aumentas tus estadísticas con {}.", card.name));
                }
                _ => {
                    let hit_chance = card.get_status_chance();
                    let hit = rand::thread_rng().gen_bool(hit_chance as f64);
                    if hit {
                        enemy.apply_status(status, stacks);
                        ev_anim.write(CombatAnimEvent { side: CombatSide::Enemy, kind: CombatAnimKind::Buff });
                        run_state.add_log(format!(
                            "[ESTADO] ¡Acierto! Aplicas {} cargas de {} a {}.",
                            stacks,
                            status.display_name(),
                            enemy.name
                        ));
                    } else {
                        run_state.add_log(format!(
                            "[ESTADO] ¡{} resistió el efecto de {}! ({}% acierto)",
                            enemy.name,
                            status.display_name(),
                            (hit_chance * 100.0) as u32
                        ));
                    }
                }
            }
        }

        // Movimiento a descarte o agotada
        commands.entity(card_entity).remove::<InHand>();
        if card.exhausts {
            commands.entity(card_entity).insert(Exhausted);
            run_state.add_log(format!("[AGOTADA] {} se agota por el resto del combate.", card.display_name()));
        } else {
            commands.entity(card_entity).insert(InDiscardPile);
        }
    }
}

pub fn handle_end_turn(
    mut ev_end: MessageReader<EndPlayerTurnEvent>,
    mut ev_anim: MessageWriter<CombatAnimEvent>,
    mut run_state: ResMut<CurrentRunState>,
    mut commands: Commands,
    mut q_player: Query<&mut PokemonStats, (With<PlayerPokemon>, Without<EnemyPokemon>)>,
    mut q_enemy: Query<(&mut PokemonStats, &mut EnemyAI), (With<EnemyPokemon>, Without<PlayerPokemon>)>,
    q_hand: Query<Entity, With<InHand>>,
    q_draw: Query<(Entity, &Card), With<InDrawPile>>,
    q_discard: Query<(Entity, &Card), With<InDiscardPile>>,
    mut ev_shuffle: Option<MessageWriter<DeckShuffleAnimEvent>>,
) {
    if ev_end.read().next().is_none() {
        return;
    }

    let Ok(mut player) = q_player.single_mut() else { return };
    let Ok((mut enemy, mut ai)) = q_enemy.single_mut() else { return };

    if enemy.is_fainted() || player.is_fainted() {
        return;
    }

    run_state.add_log("[TURNO] Fin del turno del jugador.");

    // Descarte de cartas de la mano
    for e in q_hand.iter() {
        commands.entity(e).remove::<InHand>().insert(InDiscardPile);
    }

    // Turno del enemigo: ejecuta su intención (un rival ya derrotado no actúa)
    let enemy_intent = if enemy.is_fainted() { None } else { ai.current_intent.take() };
    if let Some(intent) = enemy_intent {
        match intent {
            EnemyIntentType::Attack { raw_damage, category } => {
                let (blocked, hp_damage) = player.take_damage(raw_damage, category);
                ev_anim.write(CombatAnimEvent { side: CombatSide::Enemy, kind: CombatAnimKind::Strike });
                ev_anim.write(CombatAnimEvent { side: CombatSide::Player, kind: CombatAnimKind::Damage(blocked + hp_damage) });
                run_state.add_log(format!(
                    "[ATAQUE] {} ataca: {} daño ({} absorbido por Bloqueo, {} al HP).",
                    enemy.name, blocked + hp_damage, blocked, hp_damage
                ));

                // Reliquia Casco Dentado
                if run_state.has_relic("rocky_helmet") && blocked > 0 {
                    let recoil = (player.def * 0.3).round() as i32;
                    enemy.hp = (enemy.hp - recoil).max(0);
                    ev_anim.write(CombatAnimEvent { side: CombatSide::Enemy, kind: CombatAnimKind::Damage(recoil) });
                    run_state.add_log(format!("[RELIQUIA] Casco Dentado: Devuelve {} daño a {}.", recoil, enemy.name));
                }
            }
            EnemyIntentType::Defend { block_amount } => {
                enemy.add_block(block_amount);
                ev_anim.write(CombatAnimEvent { side: CombatSide::Enemy, kind: CombatAnimKind::Block(block_amount) });
                run_state.add_log(format!("[DEFENSA] {} se defiende y gana {} de Bloqueo.", enemy.name, block_amount));
            }
            EnemyIntentType::Buff { status, stacks } => {
                enemy.apply_status(status, stacks);
                ev_anim.write(CombatAnimEvent { side: CombatSide::Enemy, kind: CombatAnimKind::Buff });
                run_state.add_log(format!("[POTENCIAR] {} se potencia con {}.", enemy.name, status.display_name()));
            }
            EnemyIntentType::Debuff { status, stacks } => {
                let hit_chance = 0.65;
                let hit = rand::thread_rng().gen_bool(hit_chance);
                if hit {
                    player.apply_status(status, stacks);
                    ev_anim.write(CombatAnimEvent { side: CombatSide::Player, kind: CombatAnimKind::Buff });
                    run_state.add_log(format!("[ESTADO] ¡Acierto rival! {} te aplica {} cargas de {}.", enemy.name, stacks, status.display_name()));
                } else {
                    run_state.add_log(format!("[ESTADO] ¡Resististe el efecto de {} de {}! (Fallo del rival)", status.display_name(), enemy.name));
                }
            }
        }
    }

    // Efectos de fin de turno (Quemadura, Veneno)
    for log in player.on_turn_end() {
        run_state.add_log(log);
    }
    for log in enemy.on_turn_end() {
        run_state.add_log(log);
    }

    // Reliquia Restos (Leftovers): si conservas bloqueo, cura 3 HP
    if run_state.has_relic("leftovers") && player.block > 0 {
        player.hp = (player.hp + 3).min(player.max_hp);
        ev_anim.write(CombatAnimEvent { side: CombatSide::Player, kind: CombatAnimKind::Heal(3) });
        run_state.add_log("[RELIQUIA] Restos: Recuperas 3 HP por conservar Bloqueo.");
    }

    // Si algún combatiente cayó debilitado, detenemos aquí el turno para que `check_combat_outcome`
    // procese la victoria o derrota de inmediato.
    if enemy.is_fainted() || player.is_fainted() {
        return;
    }

    // Inicia nuevo turno del jugador
    player.on_turn_start(0.0);
    enemy.on_turn_start(0.0);
    run_state.energy = run_state.max_energy;

    // Próxima intención del enemigo
    let next_intent = ai.decide_next_intent(&enemy);
    match &next_intent {
        EnemyIntentType::Attack { raw_damage, .. } => {
            let dmg = (raw_damage * PokemonStats::calc_mitigation(player.def)).round() as i32;
            run_state.add_log(format!("[INTENCIÓN] {} atacará por {} de daño.", enemy.name, dmg));
        }
        EnemyIntentType::Defend { block_amount } => {
            run_state.add_log(format!("[INTENCIÓN] {} ganará {} de Bloqueo.", enemy.name, block_amount));
        }
        _ => {}
    }

    // Roba 5 cartas nuevas
    let mut draw_pile: Vec<Entity> = q_draw.iter().map(|(e, _)| e).collect();
    if draw_pile.len() < 5 {
        let recycled_count = q_discard.iter().count();
        // Recicla el descarte en la pila de robo
        for (e, _) in q_discard.iter() {
            commands.entity(e).remove::<InDiscardPile>().insert(InDrawPile);
            draw_pile.push(e);
        }
        run_state.add_log("[BARAJAR] Barajando el mazo de descarte en el mazo de robo.");
        if let Some(ref mut ev_shuffle) = ev_shuffle {
            ev_shuffle.write(DeckShuffleAnimEvent { card_count: recycled_count });
        }
    }

    draw_pile.shuffle(&mut rand::thread_rng());
    for &e in draw_pile.iter().take(5) {
        commands.entity(e).remove::<InDrawPile>().insert(InHand);
    }
}

pub fn check_combat_outcome(
    mut commands: Commands,
    mut run_state: ResMut<CurrentRunState>,
    mut next_state: ResMut<NextState<AppState>>,
    mut meta: ResMut<MetaProfile>,
    mut ev_anim: MessageWriter<CombatAnimEvent>,
    mut q_player: Query<&mut PokemonStats, (With<PlayerPokemon>, Without<EnemyPokemon>)>,
    mut q_player_sprite: Query<&mut crate::pokemon::PokemonSprite, With<PlayerPokemon>>,
    q_enemy: Query<(Entity, &PokemonStats, &EnemyAI, Option<&crate::pokemon::TrainerSprite>), (With<EnemyPokemon>, Without<PlayerPokemon>)>,
    q_resolved: Query<(), With<CombatOutcomeResolved>>,
) {
    if !q_resolved.is_empty() {
        return;
    }
    let Ok(mut player) = q_player.single_mut() else { return };
    let Ok((enemy_entity, enemy, ai, enemy_trainer)) = q_enemy.single() else { return };

    let (enemy_trainer_name, enemy_trainer_id) = match enemy_trainer {
        Some(t) => (t.name.clone(), t.trainer_id),
        None => (
            if ai.is_boss { "Líder Brock".to_string() } else { "Entrenador Rival".to_string() },
            if ai.is_boss { 6 } else { 0 },
        ),
    };

    if enemy.is_fainted() {
        commands.entity(enemy_entity).insert(CombatOutcomeResolved);
        run_state.add_log(format!("[VICTORIA] ¡Has derrotado a {}!", enemy.name));

        // Recompensa de EXP escalonada según el tipo de rival (minijefes y líderes dan sustancialmente más)
        let exp_gain = calculate_exp_reward(ai.is_boss, ai.is_mini_boss, enemy.level);
        let (levels_gained, level_logs) = player.gain_exp(exp_gain);
        if levels_gained > 0 {
            crate::pokemon::update_pokemon_stats_for_level(&mut player);
        }
        for log in level_logs {
            run_state.add_log(log);
        }
        run_state.add_log(format!(
            "[EXP] Ganas +{} EXP (Nivel actual: Nv. {} [{}/{} XP]).",
            exp_gain, player.level, player.exp, player.exp_to_next_level()
        ));

        // Persiste nivel y experiencia acumulada en la partida
        run_state.player_level = Some(player.level);
        run_state.player_exp = Some(player.exp);

        // Comprobación de evolución canónica
        let evo_result = check_evolution(&mut player);
        if let Some(ref evo_msg) = evo_result {
            run_state.add_log(evo_msg);
            run_state.selected_starter_id = player.species_id;
            if let Ok(mut sprite) = q_player_sprite.single_mut() {
                sprite.species_id = player.species_id;
                run_state.add_log(format!("[EVOLUCIÓN] Sprite actualizado a: {}", sprite.relative_path()));
            }
        }

        // Recompensa en Poké-Dólares
        let mut money_gain = if ai.is_boss { 120 } else { 25 };
        if run_state.has_relic("amulet_coin") {
            money_gain = (money_gain as f32 * 1.5).round() as u32;
        }
        run_state.poke_dollars += money_gain;
        run_state.add_log(format!("[RECOMPENSA] Obtienes {} P$ de recompensa.", money_gain));

        // Recompensa de Fragmento en Mini Jefe
        let mut fragment_reward = None;
        if ai.is_mini_boss || enemy.name.contains("Scyther") || enemy.name.contains("Snorlax") {
            let entry = meta.pokemon_fragments.entry(enemy.species_id).or_insert(0);
            *entry += 1;
            fragment_reward = Some(format!("1 Fragmento permanente de {}", enemy.name));
            run_state.add_log(format!("[FRAGMENTO] ¡Obtienes 1 Fragmento permanente de {} para el Gachapón!", enemy.name));
        }

        // Recompensa de Medalla si es Jefe de Gimnasio
        let mut badge_reward = None;
        if ai.is_boss {
            run_state.relics.push("boulder_badge".to_string());
            badge_reward = Some("Medalla de Gimnasio (Reliquia de Jefe)".to_string());
            run_state.add_log("[MEDALLA] ¡Has conseguido la Medalla de Gimnasio (Reliquia de Jefe)!");
        }

        // Fichas de metaprogresión
        let tokens_gain = if ai.is_boss { 30 } else { 15 };
        meta.gachapon_tokens += tokens_gain;

        // Dispara animaciones de victoria (jugador) y debilitamiento (rival)
        ev_anim.write(CombatAnimEvent { side: CombatSide::Enemy, kind: CombatAnimKind::Faint });
        ev_anim.write(CombatAnimEvent { side: CombatSide::Player, kind: CombatAnimKind::Victory });

        // Prepara los datos para la Pantalla de Victoria
        let reward_cards = roll_victory_cards(player.primary_types.first().copied());
        commands.insert_resource(CombatVictoryData {
            enemy_name: enemy.name.clone(),
            enemy_trainer_name,
            enemy_trainer_id,
            is_boss: ai.is_boss,
            is_mini_boss: ai.is_mini_boss,
            player_name: player.name.clone(),
            player_species_id: player.species_id,
            player_level: player.level,
            exp_gained: exp_gain,
            player_exp: player.exp,
            exp_to_next: player.exp_to_next_level(),
            levels_gained,
            money_gained: money_gain,
            tokens_gained: tokens_gain,
            fragment_reward,
            badge_reward,
            evolution_msg: evo_result,
            reward_cards,
            card_chosen: false,
        });

        // Avanza el piso del mapa y persiste HP
        run_state.player_hp = Some(player.hp);
        run_state.current_floor += 1;

        // Transiciona a la Pantalla de Victoria
        next_state.set(AppState::Victory);
        return;
    }

    if player.is_fainted() {
        // Reliquia Banda Focus Maestra
        if run_state.has_relic("focus_sash") {
            player.hp = 1;
            run_state.relics.retain(|r| r != "focus_sash");
            run_state.add_log("[RELIQUIA] Banda Focus Maestra: ¡Sobrevives milagrosamente con 1 HP!");
        } else {
            commands.entity(enemy_entity).insert(CombatOutcomeResolved);
            run_state.add_log("[DERROTA] ¡Tu Pokémon ha caído debilitado! Fin de la partida.");
            next_state.set(AppState::RunSummary);
        }
    }
}

pub fn handle_use_consumable(
    mut ev_use: MessageReader<UseConsumableEvent>,
    mut ev_anim: MessageWriter<CombatAnimEvent>,
    mut run_state: ResMut<CurrentRunState>,
    mut q_player: Query<&mut PokemonStats, With<PlayerPokemon>>,
) {
    let Ok(mut player) = q_player.single_mut() else { return };

    for ev in ev_use.read() {
        if ev.slot_index >= 3 {
            continue;
        }

        if let Some(item_name) = run_state.consumables[ev.slot_index].take() {
            match item_name.as_str() {
                "potion" => {
                    let heal = (player.max_hp as f32 * 0.20).round() as i32;
                    player.hp = (player.hp + heal).min(player.max_hp);
                    ev_anim.write(CombatAnimEvent { side: CombatSide::Player, kind: CombatAnimKind::Heal(heal) });
                    run_state.add_log(format!("[OBJETO] Usas Poción: Recuperas {} HP.", heal));
                }
                "super_potion" => {
                    let heal = (player.max_hp as f32 * 0.35).round() as i32;
                    player.hp = (player.hp + heal).min(player.max_hp);
                    ev_anim.write(CombatAnimEvent { side: CombatSide::Player, kind: CombatAnimKind::Heal(heal) });
                    run_state.add_log(format!("[OBJETO] Usas Superpoción: Recuperas {} HP.", heal));
                }
                "sitrus_berry" => {
                    run_state.energy += 2;
                    ev_anim.write(CombatAnimEvent { side: CombatSide::Player, kind: CombatAnimKind::Buff });
                    run_state.add_log("[OBJETO] Usas Baya Zidra: Ganas +2 de Energía de inmediato.");
                }
                "ether" => {
                    run_state.energy += 1;
                    ev_anim.write(CombatAnimEvent { side: CombatSide::Player, kind: CombatAnimKind::Buff });
                    run_state.add_log("[OBJETO] Usas Éter: Ganas +1 de Energía.");
                }
                _ => {}
            }
        }
    }
}

pub fn process_damage_events(mut ev_dmg: MessageReader<DealDamageEvent>) {
    for _ in ev_dmg.read() {}
}

pub fn process_block_events(mut ev_block: MessageReader<GainBlockEvent>) {
    for _ in ev_block.read() {}
}

pub fn process_heal_events(mut ev_heal: MessageReader<HealEvent>) {
    for _ in ev_heal.read() {}
}
