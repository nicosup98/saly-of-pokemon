use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Relic {
    pub id: String,
    pub name: String,
    pub description: String,
    pub price: u32,
    pub tier: RelicTier,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum RelicTier {
    Starter,
    Common,
    Uncommon,
    Rare,
    BossBadge,
}

pub fn get_all_relics() -> Vec<Relic> {
    vec![
        // Iniciales
        Relic {
            id: "miracle_seed".to_string(),
            name: "Semilla Milagro".to_string(),
            description: "Al inicio del combate aplica 3 Drenadoras. Cura 4 HP al terminar combate.".to_string(),
            price: 0,
            tier: RelicTier::Starter,
        },
        Relic {
            id: "charcoal".to_string(),
            name: "Carbón".to_string(),
            description: "El primer ataque de Fuego aplica +3 Quemadura adicional y +15% daño.".to_string(),
            price: 0,
            tier: RelicTier::Starter,
        },
        Relic {
            id: "mystic_water".to_string(),
            name: "Agua Mística".to_string(),
            description: "Conserva hasta 1.0x Sp. Def de Bloqueo para el turno 2.".to_string(),
            price: 0,
            tier: RelicTier::Starter,
        },
        Relic {
            id: "magnet".to_string(),
            name: "Imán".to_string(),
            description: "Turno 1: Roba +1 carta y gana +1 Energía al jugar >= 3 cartas.".to_string(),
            price: 0,
            tier: RelicTier::Starter,
        },
        Relic {
            id: "silk_scarf".to_string(),
            name: "Pañuelo Seda".to_string(),
            description: "Cartas Normales ganan +20% de multiplicador.".to_string(),
            price: 0,
            tier: RelicTier::Starter,
        },
        // Comunes
        Relic {
            id: "quick_claw".to_string(),
            name: "Garra Rápida".to_string(),
            description: "+15% Spe. En turno 1 tu primera carta de coste 1 cuesta 0.".to_string(),
            price: 160,
            tier: RelicTier::Common,
        },
        Relic {
            id: "zoom_lens".to_string(),
            name: "Lente Zoom".to_string(),
            description: "Si tu Spe es menor que la del enemigo, tus cartas ganan +15% daño.".to_string(),
            price: 160,
            tier: RelicTier::Common,
        },
        Relic {
            id: "shell_bell".to_string(),
            name: "Campana Concha".to_string(),
            description: "Infligir 25 o más de daño con una carta cura 3 HP.".to_string(),
            price: 175,
            tier: RelicTier::Common,
        },
        Relic {
            id: "focus_band".to_string(),
            name: "Banda Focus".to_string(),
            description: "Si tu HP cae < 20%, ganas 1.5x Def de Bloqueo (1 vez por combate).".to_string(),
            price: 170,
            tier: RelicTier::Common,
        },
        // Poco Comunes
        Relic {
            id: "leftovers".to_string(),
            name: "Restos".to_string(),
            description: "Al final de turno, si conservas Bloqueo, recuperas 3 HP.".to_string(),
            price: 200,
            tier: RelicTier::Uncommon,
        },
        Relic {
            id: "lucky_egg".to_string(),
            name: "Huevo Suerte".to_string(),
            description: "Ganas +25% de EXP adicional en todos los combates.".to_string(),
            price: 210,
            tier: RelicTier::Uncommon,
        },
        Relic {
            id: "rocky_helmet".to_string(),
            name: "Casco Dentado".to_string(),
            description: "Recibir ataque físico con Bloqueo devuelve 30% de tu Def como daño.".to_string(),
            price: 195,
            tier: RelicTier::Uncommon,
        },
        // Raras
        Relic {
            id: "amulet_coin".to_string(),
            name: "Moneda Amuleto".to_string(),
            description: "Ganas +50% de Poké-Dólares (P$) en todos los combates.".to_string(),
            price: 275,
            tier: RelicTier::Rare,
        },
        Relic {
            id: "vip_card".to_string(),
            name: "Tarjeta VIP / Cupón de Silph".to_string(),
            description: "Aumenta drásticamente las ofertas altas en la Tienda.".to_string(),
            price: 280,
            tier: RelicTier::Rare,
        },
        Relic {
            id: "life_orb".to_string(),
            name: "Vidasfera".to_string(),
            description: "Cartas de ataque ganan +30% daño, pero sufres 2 HP de retroceso.".to_string(),
            price: 290,
            tier: RelicTier::Rare,
        },
        Relic {
            id: "focus_sash".to_string(),
            name: "Banda Focus Maestra".to_string(),
            description: "Sobrevives a un golpe mortal con 1 HP e invulnerabilidad por ese turno.".to_string(),
            price: 300,
            tier: RelicTier::Rare,
        },
        // Medallas de Gimnasio
        Relic {
            id: "boulder_badge".to_string(),
            name: "Medalla Roca".to_string(),
            description: "+1 de Energía por turno, pero las cartas defensivas dan -15% Bloqueo.".to_string(),
            price: 0,
            tier: RelicTier::BossBadge,
        },
        Relic {
            id: "cascade_badge".to_string(),
            name: "Medalla Cascada".to_string(),
            description: "+1 carta por turno y tu primer ataque Especial cuesta -1 Energía.".to_string(),
            price: 0,
            tier: RelicTier::BossBadge,
        },
        Relic {
            id: "volcano_badge".to_string(),
            name: "Medalla Volcán".to_string(),
            description: "+1 de Energía por turno, pero inicias con 2 de Quemadura.".to_string(),
            price: 0,
            tier: RelicTier::BossBadge,
        },
    ]
}
