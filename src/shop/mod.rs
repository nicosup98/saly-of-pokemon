use bevy::prelude::*;
use rand::Rng;
use crate::card::{Card, create_master_card_catalog};
use crate::relics::{get_all_relics, RelicTier};
use crate::run::{AppState, CurrentRunState};

#[derive(Clone, Debug)]
pub struct ShopItemCard {
    pub card: Card,
    pub original_price: u32,
    pub final_price: u32,
    pub discount_percent: u32,
    pub purchased: bool,
}

#[derive(Resource, Debug, Clone)]
pub struct ShopInventory {
    pub cards: Vec<ShopItemCard>,
    pub relics: Vec<(String, String, u32, bool)>, // (id, name, price, purchased)
    pub consumables: Vec<(String, String, u32, bool)>, // (id, name, price, purchased)
}

pub struct ShopPlugin;

impl Plugin for ShopPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Shop), setup_shop);
    }
}

pub fn setup_shop(
    mut commands: Commands,
    mut run_state: ResMut<CurrentRunState>,
) {
    let mut rng = rand::thread_rng();
    let catalog = create_master_card_catalog();
    let has_vip = run_state.has_relic("vip_card");

    // Selecciona 6 cartas para la tienda
    let mut shop_cards = Vec::new();
    let discounted_index = rng.gen_range(0..6);

    // Sistema de descuentos aleatorios según el GDD (Sección 7.2)
    let roll_discount = rng.gen_range(0..100);
    let discount = if has_vip {
        if roll_discount < 10 { 25 }
        else if roll_discount < 25 { 30 }
        else if roll_discount < 65 { 50 }
        else if roll_discount < 90 { 75 }
        else { 90 }
    } else {
        if roll_discount < 40 { 25 }
        else if roll_discount < 75 { 30 }
        else if roll_discount < 90 { 50 }
        else if roll_discount < 99 { 75 }
        else { 90 }
    };

    for i in 0..6 {
        let card = catalog[rng.gen_range(0..catalog.len())].clone();
        let base_p = card.rarity.base_price();
        let (final_p, disc) = if i == discounted_index {
            (base_p * (100 - discount) / 100, discount)
        } else {
            (base_p, 0)
        };

        shop_cards.push(ShopItemCard {
            card,
            original_price: base_p,
            final_price: final_p,
            discount_percent: disc,
            purchased: false,
        });
    }

    // 3 Reliquias
    let all_relics = get_all_relics();
    let available_relics: Vec<_> = all_relics
        .iter()
        .filter(|r| r.tier != RelicTier::Starter && r.tier != RelicTier::BossBadge && !run_state.has_relic(&r.id))
        .collect();

    let mut shop_relics = Vec::new();
    for r in available_relics.iter().take(3) {
        shop_relics.push((r.id.clone(), r.name.clone(), r.price, false));
    }

    // 3 Consumibles
    let shop_consumables = vec![
        ("potion".to_string(), "Poción (+20% HP)".to_string(), 50, false),
        ("super_potion".to_string(), "Superpoción (+35% HP)".to_string(), 75, false),
        ("sitrus_berry".to_string(), "Baya Zidra (+2 Energía)".to_string(), 85, false),
    ];

    let balance = run_state.poke_dollars;
    run_state.add_log(format!(
        "[TIENDA] Bienvenido al Poké Mart. Tu saldo actual: {} P$.",
        balance
    ));

    commands.insert_resource(ShopInventory {
        cards: shop_cards,
        relics: shop_relics,
        consumables: shop_consumables,
    });
}
