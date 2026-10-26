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
