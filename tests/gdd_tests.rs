use std::path::Path;
use slay_of_pokemon::card::create_master_card_catalog;
use slay_of_pokemon::gachapon::pull_gachapon;
use slay_of_pokemon::map::generator::{generate_act_map, NodeType};
use slay_of_pokemon::pokemon::{create_pokemon_from_species, check_evolution};
use slay_of_pokemon::run::MetaProfile;
use slay_of_pokemon::stats::PokemonStats;
use slay_of_pokemon::types::ElementType;

#[test]
fn test_damage_and_mitigation_formulas() {
    // Charmander Nv 5 (Base Sp. Atk 60 -> Sp. Atk 8)
    let charmander = create_pokemon_from_species(4, 5).unwrap();
    assert_eq!(charmander.level, 5);
    assert_eq!(charmander.hp, 84);
    assert_eq!(charmander.sp_atk, 8.0);

    // Carta Ascuas (Tier Común, Coste 1, Mult 1.1)
    let raw_damage = charmander.sp_atk * 1.1 * 1.2; // 8.0 * 1.1 * 1.2 STAB = 10.56
    assert!((raw_damage - 10.56).abs() < 0.01);

    // Mitigación: 100 / (100 + Def)
    let target_def = 40.0;
    let mitigation = PokemonStats::calc_mitigation(target_def);
    assert!((mitigation - (100.0 / 140.0)).abs() < 0.001);

    // Daño final mitigado
    let mitigated = (raw_damage * mitigation).round() as i32;
    assert_eq!(mitigated, 8);
}

#[test]
fn test_type_effectiveness_gdd() {
    // Súper Eficaz (x1.5)
    assert_eq!(ElementType::Fire.effectiveness_against(ElementType::Grass), 1.5);
    assert_eq!(ElementType::Water.effectiveness_against(ElementType::Fire), 1.5);

    // Neutro (x1.0)
    assert_eq!(ElementType::Normal.effectiveness_against(ElementType::Normal), 1.0);

    // No muy eficaz (x0.65)
    assert_eq!(ElementType::Fire.effectiveness_against(ElementType::Water), 0.65);

    // Inmunidad clásica adaptada (x0.25)
    assert_eq!(ElementType::Normal.effectiveness_against(ElementType::Ghost), 0.25);
    assert_eq!(ElementType::Electric.effectiveness_against(ElementType::Ground), 0.25);
    assert_eq!(ElementType::Poison.effectiveness_against(ElementType::Steel), 0.25);
}

#[test]
fn test_canonical_evolution_charmander_line() {
    // Nivel 5: Charmander
    let mut pokemon = create_pokemon_from_species(4, 5).unwrap();
    assert_eq!(pokemon.name, "Charmander");
    assert_eq!(pokemon.species_id, 4);

    // Nivel 16: Charmeleon
    pokemon.level = 16;
    let evo1 = check_evolution(&mut pokemon);
    assert!(evo1.is_some());
    assert_eq!(pokemon.name, "Charmeleon");
    assert_eq!(pokemon.species_id, 5);
    assert_eq!(pokemon.max_hp, 135);
    assert_eq!(pokemon.sp_atk, 18.0);

    // Nivel 36: Charizard
    pokemon.level = 36;
    let evo2 = check_evolution(&mut pokemon);
    assert!(evo2.is_some());
    assert_eq!(pokemon.name, "Charizard");
    assert_eq!(pokemon.species_id, 6);
    assert_eq!(pokemon.max_hp, 295);
    assert_eq!(pokemon.sp_atk, 44.0);
    assert!(pokemon.primary_types.contains(&ElementType::Dragon));
}

#[test]
fn test_procedural_map_rules() {
    let map = generate_act_map(1);
    assert_eq!(map.floors.len(), 15);

    let mut total_mini_bosses = 0;
    for (i, floor) in map.floors.iter().enumerate() {
        if i == 14 {
            // Piso 15 converge en Líder de Gimnasio
            assert_eq!(floor.len(), 1);
            assert_eq!(floor[0].node_type, NodeType::GymLeader);
        } else {
            assert_eq!(floor.len(), 3);
            let has_mb = floor.iter().any(|n| n.node_type == NodeType::MiniBoss);
            if has_mb {
                total_mini_bosses += 1;
                // El piso siguiente debe tener Centro Pokémon garantizado en un único camino (no en los 3)
                let next_floor = &map.floors[i + 1];
                let center_count = next_floor.iter().filter(|n| n.node_type == NodeType::PokemonCenter).count();
                assert_eq!(center_count, 1, "el Centro Pokémon debe aparecer en un solo camino como opción, no en los 3");
            }
        }
    }
    // Máximo 3 Mini Jefes por Acto (GDD 6.3)
    assert!(total_mini_bosses <= 3);
}

#[test]
fn test_master_catalog_has_all_base_cards() {
    let catalog = create_master_card_catalog();
    // 8 Comunes + 7 Poco Comunes + 7 Raras + 5 Exóticas + 5 Legendarias = 32 cartas
    assert_eq!(catalog.len(), 32);
}

#[test]
fn test_gachapon_fragment_rate_up() {
    let mut meta = MetaProfile {
        gachapon_tokens: 1000,
        unlocked_pokemon: vec![4],
        pokemon_fragments: [(123, 3)].into_iter().collect(),
        starter_chosen: true,
        ascension_level: 0,
    };

    // Pull con 3 fragmentos de Scyther
    let result = pull_gachapon(&mut meta, Some(123), 3);
    assert!(result.is_some());
    assert_eq!(meta.gachapon_tokens, 900);
    // Los fragmentos usados se consumieron
    assert_eq!(*meta.pokemon_fragments.get(&123).unwrap(), 0);
}

#[test]
fn test_pokemon_sprites_exist() {
    let species_db = slay_of_pokemon::pokemon::get_species_database();
    for sp in species_db {
        let front_path = format!("assets/{}", sp.front_sprite_path());
        let back_path = format!("assets/{}", sp.back_sprite_path());

        assert!(
            std::path::Path::new(&front_path).exists(),
            "Falta el sprite frontal de {}: {}",
            sp.name,
            front_path
        );
        assert!(
            std::path::Path::new(&back_path).exists(),
            "Falta el sprite trasero de {}: {}",
            sp.name,
            back_path
        );
    }
}

#[test]
fn test_trainer_sprites_exist() {
    let sheet_path = "assets/sprites/Characters_V3_Colour.png";
    assert!(std::path::Path::new(sheet_path).exists(), "Falta la hoja maestra Characters_V3_Colour.png");

    let expected_trainers = [
        "player_front.png",
        "player_back.png",
        "enemy_front.png",
        "brock_front.png",
        "leader_front.png",
        "nurse_front.png",
        "clerk_front.png",
        "common_front.png",
    ];

    for file_name in expected_trainers {
        let path = format!("assets/sprites/trainers/{}", file_name);
        assert!(
            std::path::Path::new(&path).exists(),
            "Falta el sprite de entrenador generado: {}",
            path
        );
    }
}

#[test]
fn test_situation_backgrounds_exist_and_mapping() {
    use slay_of_pokemon::map::generator::NodeType;
    use slay_of_pokemon::ui::combat_ui::combat_background_path;

    let expected_backgrounds = [
        ("assets/fondos/fondo_pelea.jpeg", "Combate contra entrenadores comunes"),
        ("assets/fondos/gym_misty.jpeg", "Combate contra Mini Jefe / Élite"),
        ("assets/fondos/gym_blaine.jpeg", "Combate contra Líder de Gimnasio"),
        ("assets/fondos/centro_pokemon.jpeg", "Centro Pokémon"),
        ("assets/fondos/tienda_pokemon.jpeg", "Tienda Poké Mart"),
    ];

    for (path, desc) in expected_backgrounds {
        assert!(
            std::path::Path::new(path).exists(),
            "Falta el fondo para {}: {}",
            desc,
            path
        );
    }

    // Comprobar la selección contextual del fondo para cada situación
    assert_eq!(
        combat_background_path(Some(NodeType::Enemy), false, false),
        "fondos/fondo_pelea.jpeg"
    );
    assert_eq!(
        combat_background_path(Some(NodeType::MiniBoss), false, true),
        "fondos/gym_misty.jpeg"
    );
    assert_eq!(
        combat_background_path(Some(NodeType::GymLeader), true, false),
        "fondos/gym_blaine.jpeg"
    );

    // Fallbacks si no se especifica el tipo de nodo en el mapa
    assert_eq!(
        combat_background_path(None, true, false),
        "fondos/gym_blaine.jpeg"
    );
    assert_eq!(
        combat_background_path(None, false, true),
        "fondos/gym_misty.jpeg"
    );
    assert_eq!(
        combat_background_path(None, false, false),
        "fondos/fondo_pelea.jpeg"
    );
}

#[test]
fn test_card_type_assets_exist_and_mapping() {
    use slay_of_pokemon::card::{card_type_asset_path, Card};
    use slay_of_pokemon::types::{ElementType, MoveCategory, RarityTier};

    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let expected_files = [
        "cartas/acero.jpeg",
        "cartas/agua.jpeg",
        "cartas/electrico.jpeg",
        "cartas/fuego.jpeg",
        "cartas/insecto.jpeg",
        "cartas/normal.jpeg",
        "cartas/planta.jpeg",
        "cartas/tierra.jpeg",
    ];

    for file_rel in expected_files {
        let full_path = manifest_dir.join("assets").join(file_rel);
        assert!(
            full_path.exists(),
            "El archivo de carta {:?} debe existir en assets/",
            file_rel
        );
    }

    // Comprobar que los tipos elementales tienen sus asignaciones correctas
    assert_eq!(card_type_asset_path(ElementType::Fire), "cartas/fuego.jpeg");
    assert_eq!(card_type_asset_path(ElementType::Water), "cartas/agua.jpeg");
    assert_eq!(card_type_asset_path(ElementType::Grass), "cartas/planta.jpeg");
    assert_eq!(card_type_asset_path(ElementType::Bug), "cartas/insecto.jpeg");
    assert_eq!(card_type_asset_path(ElementType::Steel), "cartas/acero.jpeg");
    assert_eq!(card_type_asset_path(ElementType::Ground), "cartas/tierra.jpeg");
    assert_eq!(card_type_asset_path(ElementType::Electric), "cartas/electrico.jpeg");
    assert_eq!(card_type_asset_path(ElementType::Normal), "cartas/normal.jpeg");

    // Verificar que todas las variantes de ElementType apuntan a un asset existente
    let all_elements = [
        ElementType::Normal,
        ElementType::Fire,
        ElementType::Water,
        ElementType::Grass,
        ElementType::Electric,
        ElementType::Ice,
        ElementType::Fighting,
        ElementType::Poison,
        ElementType::Ground,
        ElementType::Flying,
        ElementType::Psychic,
        ElementType::Bug,
        ElementType::Rock,
        ElementType::Ghost,
        ElementType::Dragon,
        ElementType::Dark,
        ElementType::Steel,
        ElementType::Fairy,
    ];

    for elem in all_elements {
        let path = card_type_asset_path(elem);
        let full = manifest_dir.join("assets").join(path);
        assert!(
            full.exists(),
            "El asset asignado para {:?} ({}) no existe en assets/",
            elem,
            path
        );

        let card = Card::new("test", "Ataque Test", elem, MoveCategory::Physical, RarityTier::Common, 1, 1.0);
        assert_eq!(card.asset_path(), path);
    }
}

#[test]
fn test_pokemon_level_progression_and_xp_scaling() {
    use slay_of_pokemon::combat::calculate_exp_reward;
    use slay_of_pokemon::pokemon::{create_pokemon_from_species, update_pokemon_stats_for_level};
    use slay_of_pokemon::stats::PokemonStats;

    // 1. Minijefes dan más XP que entrenadores, y líderes de nivel dan aún más
    for lvl in [5, 10, 15, 20] {
        let trainer_xp = calculate_exp_reward(false, false, lvl);
        let miniboss_xp = calculate_exp_reward(false, true, lvl);
        let boss_xp = calculate_exp_reward(true, false, lvl);

        assert!(
            miniboss_xp > trainer_xp,
            "El minijefe en nivel {} ({}) debe dar más XP que el entrenador ({})",
            lvl, miniboss_xp, trainer_xp
        );
        assert!(
            boss_xp > miniboss_xp,
            "El líder de nivel en nivel {} ({}) debe dar aún más XP que el minijefe ({})",
            lvl, boss_xp, miniboss_xp
        );
    }

    // 2. Mientras más nivel tenga, más XP requiere para subir (curva cuadrática progresiva)
    let mut prev_req = 0;
    for lvl in 1..=40 {
        let req = PokemonStats::calc_exp_to_next_level(lvl);
        assert!(
            req > prev_req,
            "El nivel {} debe requerir estrictamente más XP ({}) que el nivel anterior ({})",
            lvl, req, prev_req
        );
        prev_req = req;
    }

    // Comprobar umbrales concretos
    let req_5 = PokemonStats::calc_exp_to_next_level(5);
    let req_15 = PokemonStats::calc_exp_to_next_level(15);
    let req_30 = PokemonStats::calc_exp_to_next_level(30);
    assert_eq!(req_5, 130);
    assert_eq!(req_15, 610);
    assert_eq!(req_30, 2080);
    assert!(req_15 > req_5 * 4, "Nivel 15 requiere más de 4 veces la XP de nivel 5");
    assert!(req_30 > req_15 * 3, "Nivel 30 requiere más de 3 veces la XP de nivel 15");

    // 3. Simulación de combates y subida de nivel con persistencia de exceso de XP
    let mut charmander = create_pokemon_from_species(4, 5).unwrap();
    assert_eq!(charmander.level, 5);
    assert_eq!(charmander.exp, 0);
    let initial_hp = charmander.max_hp;
    let initial_atk = charmander.atk;

    // Victoria contra entrenador común (60 XP) -> Sigue en nivel 5
    let trainer_xp = calculate_exp_reward(false, false, 5);
    let (levels, _) = charmander.gain_exp(trainer_xp);
    assert_eq!(levels, 0);
    assert_eq!(charmander.level, 5);
    assert_eq!(charmander.exp, 60);

    // Victoria contra minijefe (150 XP) -> 60 + 150 = 210 XP >= 130 XP -> Sube a nivel 6 con 80 XP sobrantes
    let miniboss_xp = calculate_exp_reward(false, true, 5);
    let (levels, logs) = charmander.gain_exp(miniboss_xp);
    assert_eq!(levels, 1);
    assert_eq!(charmander.level, 6);
    assert_eq!(charmander.exp, 80);
    assert!(!logs.is_empty());

    // Actualizar estadísticas al subir de nivel
    update_pokemon_stats_for_level(&mut charmander);
    assert!(charmander.max_hp > initial_hp, "Max HP debe aumentar al subir de nivel");
    assert!(charmander.atk >= initial_atk, "Ataque debe aumentar o mantenerse");
}
