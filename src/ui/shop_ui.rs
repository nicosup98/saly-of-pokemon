use bevy::prelude::*;
use crate::run::{AppState, CurrentRunState};
use crate::shop::ShopInventory;
use crate::ui::{element_color, CleanupUi, Button, BorderColor, CardHoverAnimation};

pub struct ShopUiPlugin;

impl Plugin for ShopUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(AppState::Shop),
            setup_shop_ui.after(crate::shop::setup_shop),
        )
        .add_systems(
            Update,
            (
                handle_buy_card,
                handle_buy_relic,
                handle_buy_consumable,
                handle_remove_card,
                handle_exit_shop,
                update_shop_displays,
            )
                .run_if(in_state(AppState::Shop)),
        )
        .add_systems(OnExit(AppState::Shop), cleanup_shop_ui);
    }
}

#[derive(Component)]
pub struct ShopExitButton;

#[derive(Component)]
pub struct ShopBuyCardButton(pub usize);

#[derive(Component)]
pub struct ShopCardPriceBadge(pub usize);

#[derive(Component)]
pub struct ShopBuyRelicButton(pub usize);

#[derive(Component)]
pub struct ShopBuyConsumableButton(pub usize);

#[derive(Component)]
pub struct ShopRemoveCardButton;

#[derive(Component)]
pub struct ShopBalanceText;

#[derive(Component)]
pub struct ShopRemovalCostText;

#[derive(Component)]
pub struct ShopStatusBanner;

#[derive(Component)]
pub struct ShopCardItemContainer(pub usize);

#[derive(Component)]
pub struct ShopRelicItemContainer(pub usize);

#[derive(Component)]
pub struct ShopConsumableItemContainer(pub usize);

fn setup_shop_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    preloaded: Option<Res<crate::ui::PreloadedAssets>>,
    run_state: Res<CurrentRunState>,
    inventory: Option<Res<ShopInventory>>,
) {
    let inv = match inventory {
        Some(i) => i.clone(),
        None => return,
    };
    let pre = preloaded.as_deref();
    let srv = Some(&*asset_server);
    let bg_tex = pre
        .and_then(|p| p.get_bg("fondos/tienda_pokemon.jpeg"))
        .unwrap_or_else(|| asset_server.load("fondos/tienda_pokemon.jpeg"));
    let top_bar_tex = crate::ui::load_ui_image(pre, srv, "ui/hud/hud_top_bar.png");
    let btn_gold_tex = crate::ui::load_ui_image(pre, srv, "ui/buttons/btn_gold.png");
    let btn_secondary_tex = crate::ui::load_ui_image(pre, srv, "ui/buttons/btn_secondary.png");
    let btn_danger_tex = crate::ui::load_ui_image(pre, srv, "ui/buttons/btn_danger.png");

    commands
        .spawn((
            CleanupUi,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Relative,
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(16.0)),
                overflow: Overflow::clip(),
                ..default()
            },
        ))
        .with_children(|root| {
            // Fondo temático de la Tienda Poké Mart
            root.spawn((
                ImageNode::new(bg_tex),
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.0),
                    right: Val::Px(0.0),
                    top: Val::Px(0.0),
                    bottom: Val::Px(0.0),
                    ..default()
                },
            ));

            // Tinte de contraste para preservar la nitidez del inventario VectorCraft
            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.0),
                    right: Val::Px(0.0),
                    top: Val::Px(0.0),
                    bottom: Val::Px(0.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.03, 0.05, 0.09, 0.45)),
            ));

            // === BARRA SUPERIOR (`ui/hud/hud_top_bar.png`) ===
            root.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(58.0),
                    position_type: PositionType::Relative,
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    padding: UiRect::horizontal(Val::Px(18.0)),
                    border: UiRect::all(Val::Px(2.0)),
                    border_radius: BorderRadius::all(Val::Px(14.0)),
                    overflow: Overflow::clip(),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.06, 0.09, 0.14)),
                BorderColor(Color::srgb(0.22, 0.71, 1.0)),
            ))
            .with_children(|top_bar| {
                if let Some(ref tex) = top_bar_tex {
                    top_bar.spawn((
                        ImageNode::new(tex.clone()),
                        Node {
                            position_type: PositionType::Absolute,
                            left: Val::Px(0.0),
                            right: Val::Px(0.0),
                            top: Val::Px(0.0),
                            bottom: Val::Px(0.0),
                            ..default()
                        },
                    ));
                }

                // Título y Piso con dependiente del Poké Mart
                top_bar
                    .spawn(Node {
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Px(12.0),
                        align_items: AlignItems::Center,
                        ..default()
                    })
                    .with_children(|left| {
                        left.spawn((
                            ImageNode::new(asset_server.load("sprites/trainers/clerk_front.png")),
                            Node {
                                width: Val::Px(40.0),
                                height: Val::Px(40.0),
                                ..default()
                            },
                        ));
                        left.spawn((
                            Text::new("[TIENDA] POKÉ MART (SILPH S.A.)"),
                            TextFont::from_font_size(20.0),
                            TextColor(Color::srgb(0.35, 0.82, 1.0)),
                        ));
                        left.spawn((
                            Text::new(format!(
                                "| Acto {} - Piso {}",
                                run_state.current_act, run_state.current_floor
                            )),
                            TextFont::from_font_size(14.0),
                            TextColor(Color::srgb(0.75, 0.82, 0.92)),
                        ));
                    });

                // Centro: Saldo en cápsula dorada
                top_bar
                    .spawn((
                        Node {
                            padding: UiRect::axes(Val::Px(14.0), Val::Px(5.0)),
                            border: UiRect::all(Val::Px(1.5)),
                            border_radius: BorderRadius::all(Val::Px(10.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.08, 0.12, 0.19, 0.95)),
                        BorderColor(Color::srgb(0.95, 0.76, 0.30)),
                    ))
                    .with_child((
                        ShopBalanceText,
                        Text::new(format!("Saldo: {} P$", run_state.poke_dollars)),
                        TextFont::from_font_size(18.0),
                        TextColor(Color::srgb(0.95, 0.76, 0.30)),
                    ));

                // Derecha: Botón Volver al Mapa (`ui/buttons/btn_gold.png`)
                top_bar
                    .spawn((
                        Button,
                        ShopExitButton,
                        Node {
                            min_width: Val::Px(175.0),
                            height: Val::Px(40.0),
                            position_type: PositionType::Relative,
                            padding: UiRect::horizontal(Val::Px(16.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border: UiRect::all(Val::Px(2.0)),
                            border_radius: BorderRadius::all(Val::Px(10.0)),
                            overflow: Overflow::clip(),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.28, 0.22, 0.08)),
                        BorderColor(Color::srgb(0.95, 0.76, 0.30)),
                    ))
                    .with_children(|btn| {
                        if let Some(ref tex) = btn_gold_tex {
                            btn.spawn((
                                ImageNode::new(tex.clone()),
                                Node {
                                    position_type: PositionType::Absolute,
                                    left: Val::Px(0.0),
                                    right: Val::Px(0.0),
                                    top: Val::Px(0.0),
                                    bottom: Val::Px(0.0),
                                    ..default()
                                },
                            ));
                        }
                        btn.spawn((
                            Text::new("VOLVER AL MAPA ->"),
                            TextFont::from_font_size(13.5),
                            TextColor(Color::srgb(0.15, 0.09, 0.02)),
                        ));
                    });
            });

            // === CONTENIDO CENTRAL (Scrollable / Secciones) ===
            root.spawn(Node {
                width: Val::Percent(100.0),
                height: Val::Percent(82.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(14.0),
                padding: UiRect::all(Val::Px(12.0)),
                ..default()
            })
            .with_children(|main_content| {
                // SECCIÓN 1: CARTAS EN VENTA (6 Cartas con descuentos aleatorios)
                main_content
                    .spawn(Node {
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(6.0),
                        ..default()
                    })
                    .with_children(|section_cards| {
                        section_cards.spawn((
                            Text::new("[CARTAS] CARTAS EN VENTA (Minitienda Táctica)"),
                            TextFont::from_font_size(16.0),
                            TextColor(Color::srgb(0.95, 0.76, 0.30)),
                        ));

                        // Fila de 6 cartas
                        section_cards
                            .spawn(Node {
                                width: Val::Percent(100.0),
                                flex_direction: FlexDirection::Row,
                                column_gap: Val::Px(10.0),
                                justify_content: JustifyContent::SpaceBetween,
                                ..default()
                            })
                            .with_children(|cards_row| {
                                for (i, item) in inv.cards.iter().enumerate() {
                                    let elem_col = element_color(item.card.element);
                                    let card_tex = pre
                                        .and_then(|p| p.get_card(item.card.asset_path()))
                                        .unwrap_or_else(|| asset_server.load(item.card.asset_path()));

                                    cards_row
                                        .spawn((
                                            Button,
                                            ShopBuyCardButton(i),
                                            ShopCardItemContainer(i),
                                            Node {
                                                width: Val::Percent(15.8),
                                                height: Val::Px(196.0),
                                                flex_direction: FlexDirection::Column,
                                                justify_content: JustifyContent::SpaceBetween,
                                                align_items: AlignItems::Center,
                                                border: UiRect::all(Val::Px(2.0)),
                                                border_radius: BorderRadius::all(Val::Px(8.0)),
                                                overflow: Overflow::clip(),
                                                ..default()
                                            },
                                            UiTransform::default(),
                                            GlobalZIndex(10),
                                            CardHoverAnimation::new(elem_col)
                                                .with_lift(-14.0)
                                                .with_scale(1.06),
                                            BackgroundColor(Color::srgba(0.06, 0.08, 0.12, 0.2)),
                                            BorderColor(elem_col),
                                        ))
                                        .with_children(|card_box| {
                                            // Fondo ilustrado según tipo de carta
                                            card_box.spawn((
                                                ImageNode::new(card_tex),
                                                Node {
                                                    position_type: PositionType::Absolute,
                                                    left: Val::Px(0.0),
                                                    right: Val::Px(0.0),
                                                    top: Val::Px(0.0),
                                                    bottom: Val::Px(0.0),
                                                    ..default()
                                                },
                                            ));

                                            // Encabezado: Nombre y Coste (en el banner superior)
                                            card_box
                                                .spawn(Node {
                                                    width: Val::Percent(100.0),
                                                    height: Val::Px(30.0),
                                                    margin: UiRect::top(Val::Px(10.0)),
                                                    padding: UiRect::axes(Val::Px(10.0), Val::Px(2.0)),
                                                    flex_direction: FlexDirection::Row,
                                                    justify_content: JustifyContent::SpaceBetween,
                                                    align_items: AlignItems::Center,
                                                    ..default()
                                                })
                                                .with_children(|ch| {
                                                    ch.spawn((
                                                        Text::new(item.card.display_name()),
                                                        TextFont::from_font_size(11.0),
                                                        TextColor(Color::WHITE),
                                                    ));
                                                    ch.spawn((
                                                        Text::new(format!("{} EN", item.card.energy_cost)),
                                                        TextFont::from_font_size(11.0),
                                                        TextColor(Color::srgb(1.0, 0.9, 0.2)),
                                                    ));
                                                });

                                            // Espacio central transparente para apreciar la ilustración elemental
                                            card_box.spawn(Node {
                                                flex_grow: 1.0,
                                                ..default()
                                            });

                                            // Descripción / Multiplicador en caja de descripción
                                            card_box
                                                .spawn(Node {
                                                    width: Val::Percent(86.0),
                                                    min_height: Val::Px(32.0),
                                                    padding: UiRect::new(
                                                        Val::Px(4.0),
                                                        Val::Px(20.0),
                                                        Val::Px(2.0),
                                                        Val::Px(2.0),
                                                    ),
                                                    justify_content: JustifyContent::Center,
                                                    align_items: AlignItems::FlexStart,
                                                    ..default()
                                                })
                                                .with_child((
                                                    Text::new(format!(
                                                        "{:.1}x ({:?})",
                                                        item.card.multiplier, item.card.category
                                                    )),
                                                    TextFont::from_font_size(9.5),
                                                    TextColor(Color::srgb(0.95, 0.95, 0.95)),
                                                ));

                                            // Etiqueta de descuento si aplica
                                            if item.discount_percent > 0 {
                                                card_box
                                                    .spawn((
                                                        Node {
                                                            padding: UiRect::axes(Val::Px(6.0), Val::Px(2.0)),
                                                            border_radius: BorderRadius::all(Val::Px(4.0)),
                                                            ..default()
                                                        },
                                                        BackgroundColor(Color::srgb(0.85, 0.2, 0.2)),
                                                    ))
                                                    .with_child((
                                                        Text::new(format!("-{}% ¡OFERTA!", item.discount_percent)),
                                                        TextFont::from_font_size(9.0),
                                                        TextColor(Color::WHITE),
                                                    ));
                                            }

                                            // Precio y Etiqueta
                                            card_box
                                                .spawn((
                                                    ShopCardPriceBadge(i),
                                                    Node {
                                                        width: Val::Percent(100.0),
                                                        height: Val::Px(26.0),
                                                        justify_content: JustifyContent::Center,
                                                        align_items: AlignItems::Center,
                                                        ..default()
                                                    },
                                                    BackgroundColor(if item.purchased {
                                                        Color::srgb(0.22, 0.25, 0.30)
                                                    } else {
                                                        Color::srgb(0.16, 0.56, 0.32)
                                                    }),
                                                ))
                                                .with_child((
                                                    ShopCardPriceBadge(i),
                                                    Text::new(if item.purchased {
                                                        "VENDIDO".to_string()
                                                    } else {
                                                        format!("{} P$", item.final_price)
                                                    }),
                                                    TextFont::from_font_size(12.0),
                                                    TextColor(Color::WHITE),
                                                ));
                                        });
                                }
                            });
                    });

                // SECCIÓN 2 & 3 & 4 EN COLUMNAS INFERIORES: RELIQUIAS, CONSUMIBLES Y OLVIDAR MOVIMIENTO
                main_content
                    .spawn(Node {
                        width: Val::Percent(100.0),
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Px(16.0),
                        justify_content: JustifyContent::SpaceBetween,
                        ..default()
                    })
                    .with_children(|bottom_row| {
                        // Columna 1: 3 Reliquias
                        bottom_row
                            .spawn((
                                Node {
                                    width: Val::Percent(35.0),
                                    flex_direction: FlexDirection::Column,
                                    row_gap: Val::Px(8.0),
                                    padding: UiRect::all(Val::Px(12.0)),
                                    border: UiRect::all(Val::Px(2.0)),
                                    border_radius: BorderRadius::all(Val::Px(14.0)),
                                    ..default()
                                },
                                BackgroundColor(Color::srgba(0.06, 0.09, 0.14, 0.94)),
                                BorderColor(Color::srgb(0.22, 0.71, 1.0)),
                            ))
                            .with_children(|col_relics| {
                                col_relics.spawn((
                                    Text::new("[RELIQUIAS] RELIQUIAS EQUIPABLES"),
                                    TextFont::from_font_size(15.0),
                                    TextColor(Color::srgb(0.35, 0.82, 1.0)),
                                ));

                                for (idx, (_r_id, r_name, r_price, purchased)) in inv.relics.iter().enumerate() {
                                    col_relics
                                        .spawn((
                                            ShopRelicItemContainer(idx),
                                            Node {
                                                width: Val::Percent(100.0),
                                                height: Val::Px(40.0),
                                                flex_direction: FlexDirection::Row,
                                                justify_content: JustifyContent::SpaceBetween,
                                                align_items: AlignItems::Center,
                                                padding: UiRect::horizontal(Val::Px(10.0)),
                                                border: UiRect::all(Val::Px(1.5)),
                                                border_radius: BorderRadius::all(Val::Px(8.0)),
                                                ..default()
                                            },
                                            BackgroundColor(Color::srgb(0.09, 0.13, 0.20)),
                                            BorderColor(Color::srgb(0.18, 0.27, 0.40)),
                                        ))
                                        .with_children(|r_box| {
                                            r_box.spawn((
                                                Text::new(r_name.clone()),
                                                TextFont::from_font_size(13.0),
                                                TextColor(Color::WHITE),
                                            ));

                                            r_box
                                                .spawn((
                                                    Button,
                                                    ShopBuyRelicButton(idx),
                                                    Node {
                                                        width: Val::Px(88.0),
                                                        height: Val::Px(28.0),
                                                        position_type: PositionType::Relative,
                                                        justify_content: JustifyContent::Center,
                                                        align_items: AlignItems::Center,
                                                        border_radius: BorderRadius::all(Val::Px(6.0)),
                                                        overflow: Overflow::clip(),
                                                        ..default()
                                                    },
                                                    BackgroundColor(if *purchased {
                                                        Color::srgb(0.3, 0.3, 0.3)
                                                    } else {
                                                        Color::srgb(0.2, 0.5, 0.8)
                                                    }),
                                                ))
                                                .with_children(|btn| {
                                                    if !*purchased {
                                                        if let Some(ref tex) = btn_secondary_tex {
                                                            btn.spawn((
                                                                ImageNode::new(tex.clone()),
                                                                Node {
                                                                    position_type: PositionType::Absolute,
                                                                    left: Val::Px(0.0),
                                                                    right: Val::Px(0.0),
                                                                    top: Val::Px(0.0),
                                                                    bottom: Val::Px(0.0),
                                                                    ..default()
                                                                },
                                                            ));
                                                        }
                                                    }
                                                    btn.spawn((
                                                        Text::new(if *purchased {
                                                            "COMPRADO".to_string()
                                                        } else {
                                                            format!("{} P$", r_price)
                                                        }),
                                                        TextFont::from_font_size(11.5),
                                                        TextColor(Color::WHITE),
                                                    ));
                                                });
                                        });
                                }
                            });

                        // Columna 2: 3 Consumibles
                        bottom_row
                            .spawn((
                                Node {
                                    width: Val::Percent(35.0),
                                    flex_direction: FlexDirection::Column,
                                    row_gap: Val::Px(8.0),
                                    padding: UiRect::all(Val::Px(12.0)),
                                    border: UiRect::all(Val::Px(2.0)),
                                    border_radius: BorderRadius::all(Val::Px(14.0)),
                                    ..default()
                                },
                                BackgroundColor(Color::srgba(0.06, 0.09, 0.14, 0.94)),
                                BorderColor(Color::srgb(0.20, 0.85, 0.48)),
                            ))
                            .with_children(|col_consumables| {
                                col_consumables.spawn((
                                    Text::new("CONSUMIBLES (Cinturón 3 Ranuras)"),
                                    TextFont::from_font_size(15.0),
                                    TextColor(Color::srgb(0.32, 0.96, 0.62)),
                                ));

                                for (idx, (_c_id, c_name, c_price, purchased)) in
                                    inv.consumables.iter().enumerate()
                                {
                                    col_consumables
                                        .spawn((
                                            ShopConsumableItemContainer(idx),
                                            Node {
                                                width: Val::Percent(100.0),
                                                height: Val::Px(40.0),
                                                flex_direction: FlexDirection::Row,
                                                justify_content: JustifyContent::SpaceBetween,
                                                align_items: AlignItems::Center,
                                                padding: UiRect::horizontal(Val::Px(10.0)),
                                                border: UiRect::all(Val::Px(1.5)),
                                                border_radius: BorderRadius::all(Val::Px(8.0)),
                                                ..default()
                                            },
                                            BackgroundColor(Color::srgb(0.09, 0.13, 0.20)),
                                            BorderColor(Color::srgb(0.18, 0.27, 0.40)),
                                        ))
                                        .with_children(|c_box| {
                                            c_box.spawn((
                                                Text::new(c_name.clone()),
                                                TextFont::from_font_size(13.0),
                                                TextColor(Color::WHITE),
                                            ));

                                            c_box
                                                .spawn((
                                                    Button,
                                                    ShopBuyConsumableButton(idx),
                                                    Node {
                                                        width: Val::Px(88.0),
                                                        height: Val::Px(28.0),
                                                        position_type: PositionType::Relative,
                                                        justify_content: JustifyContent::Center,
                                                        align_items: AlignItems::Center,
                                                        border_radius: BorderRadius::all(Val::Px(6.0)),
                                                        overflow: Overflow::clip(),
                                                        ..default()
                                                    },
                                                    BackgroundColor(if *purchased {
                                                        Color::srgb(0.3, 0.3, 0.3)
                                                    } else {
                                                        Color::srgb(0.2, 0.6, 0.3)
                                                    }),
                                                ))
                                                .with_children(|btn| {
                                                    if !*purchased {
                                                        if let Some(ref tex) = btn_secondary_tex {
                                                            btn.spawn((
                                                                ImageNode::new(tex.clone()),
                                                                Node {
                                                                    position_type: PositionType::Absolute,
                                                                    left: Val::Px(0.0),
                                                                    right: Val::Px(0.0),
                                                                    top: Val::Px(0.0),
                                                                    bottom: Val::Px(0.0),
                                                                    ..default()
                                                                },
                                                            ));
                                                        }
                                                    }
                                                    btn.spawn((
                                                        Text::new(if *purchased {
                                                            "COMPRADO".to_string()
                                                        } else {
                                                            format!("{} P$", c_price)
                                                        }),
                                                        TextFont::from_font_size(11.5),
                                                        TextColor(Color::WHITE),
                                                    ));
                                                });
                                        });
                                }
                            });

                        // Columna 3: Servicio de Olvidar Movimiento (`ui/buttons/btn_danger.png`)
                        bottom_row
                            .spawn((
                                Node {
                                    width: Val::Percent(27.0),
                                    flex_direction: FlexDirection::Column,
                                    justify_content: JustifyContent::SpaceBetween,
                                    padding: UiRect::all(Val::Px(12.0)),
                                    border: UiRect::all(Val::Px(2.0)),
                                    border_radius: BorderRadius::all(Val::Px(14.0)),
                                    ..default()
                                },
                                BackgroundColor(Color::srgba(0.07, 0.08, 0.14, 0.94)),
                                BorderColor(Color::srgb(1.0, 0.36, 0.36)),
                            ))
                            .with_children(|col_service| {
                                col_service.spawn((
                                    Text::new("OLVIDAR MOVIMIENTO"),
                                    TextFont::from_font_size(15.0),
                                    TextColor(Color::srgb(1.0, 0.45, 0.45)),
                                ));

                                col_service.spawn((
                                    Text::new("Elimina permanentemente una carta básica (Placaje o Refugio) de tu mazo para optimizar tu ciclado."),
                                    TextFont::from_font_size(11.5),
                                    TextColor(Color::srgb(0.78, 0.84, 0.92)),
                                ));

                                col_service.spawn((
                                    ShopRemovalCostText,
                                    Text::new(format!("Coste actual: {} P$", run_state.card_removal_cost)),
                                    TextFont::from_font_size(13.0),
                                    TextColor(Color::srgb(0.95, 0.76, 0.30)),
                                ));

                                col_service
                                    .spawn((
                                        Button,
                                        ShopRemoveCardButton,
                                        Node {
                                            width: Val::Percent(100.0),
                                            height: Val::Px(38.0),
                                            position_type: PositionType::Relative,
                                            justify_content: JustifyContent::Center,
                                            align_items: AlignItems::Center,
                                            border_radius: BorderRadius::all(Val::Px(8.0)),
                                            overflow: Overflow::clip(),
                                            ..default()
                                        },
                                        BackgroundColor(Color::srgb(0.65, 0.20, 0.25)),
                                    ))
                                    .with_children(|btn| {
                                        if let Some(ref tex) = btn_danger_tex {
                                            btn.spawn((
                                                ImageNode::new(tex.clone()),
                                                Node {
                                                    position_type: PositionType::Absolute,
                                                    left: Val::Px(0.0),
                                                    right: Val::Px(0.0),
                                                    top: Val::Px(0.0),
                                                    bottom: Val::Px(0.0),
                                                    ..default()
                                                },
                                            ));
                                        }
                                        btn.spawn((
                                            Text::new(format!(
                                                "ELIMINAR CARTA ({} P$)",
                                                run_state.card_removal_cost
                                            )),
                                            TextFont::from_font_size(12.0),
                                            TextColor(Color::WHITE),
                                        ));
                                    });
                            });
                    });
            });

            // === BARRA DE ESTADO / MENSAJES ===
            root.spawn((
                ShopStatusBanner,
                Text::new("[INFO] Pasa el ratón o pulsa sobre un artículo para adquirirlo con tus Poké-Dólares."),
                TextFont::from_font_size(13.0),
                TextColor(Color::srgb(0.55, 0.85, 1.0)),
            ));
        });
}

fn handle_buy_card(
    mut q_buttons: Query<(&Interaction, &ShopBuyCardButton, &mut BackgroundColor), Changed<Interaction>>,
    mut q_badges: Query<(&ShopCardPriceBadge, &mut BackgroundColor), Without<ShopBuyCardButton>>,
    mut q_texts: Query<(&ShopCardPriceBadge, &mut Text)>,
    mut run_state: ResMut<CurrentRunState>,
    mut inventory: ResMut<ShopInventory>,
    mut q_status: Query<&mut Text, (With<ShopStatusBanner>, Without<ShopCardPriceBadge>)>,
) {
    for (interaction, btn, _bg_color) in q_buttons.iter_mut() {
        if *interaction == Interaction::Pressed {
            let idx = btn.0;
            if idx < inventory.cards.len() {
                let item = &mut inventory.cards[idx];
                if item.purchased {
                    if let Ok(mut text) = q_status.single_mut() {
                        **text = "[!] Este movimiento ya ha sido vendido.".to_string();
                    }
                    continue;
                }

                if run_state.poke_dollars >= item.final_price {
                    run_state.poke_dollars -= item.final_price;
                    let card = item.card.clone();
                    let name = card.display_name();
                    let price = item.final_price;
                    run_state.player_deck.push(card);
                    item.purchased = true;

                    for (badge, mut badge_bg) in q_badges.iter_mut() {
                        if badge.0 == idx {
                            *badge_bg = BackgroundColor(Color::srgb(0.3, 0.3, 0.3));
                        }
                    }
                    for (badge, mut badge_txt) in q_texts.iter_mut() {
                        if badge.0 == idx {
                            **badge_txt = "VENDIDO".to_string();
                        }
                    }

                    let msg = format!("[TIENDA] ¡Has adquirido {} por {} P$! Añadida a tu mazo.", name, price);
                    run_state.add_log(msg.clone());
                    if let Ok(mut text) = q_status.single_mut() {
                        **text = msg;
                    }
                } else {
                    if let Ok(mut text) = q_status.single_mut() {
                        **text = "[AVISO] ¡No tienes suficientes Poké-Dólares para esta carta!".to_string();
                    }
                }
            }
        }
    }
}

fn handle_buy_relic(
    mut q_buttons: Query<(&Interaction, &ShopBuyRelicButton, &mut BackgroundColor), Changed<Interaction>>,
    mut run_state: ResMut<CurrentRunState>,
    mut inventory: ResMut<ShopInventory>,
    mut q_status: Query<&mut Text, With<ShopStatusBanner>>,
) {
    for (interaction, btn, mut bg_color) in q_buttons.iter_mut() {
        if *interaction == Interaction::Pressed {
            let idx = btn.0;
            if idx < inventory.relics.len() {
                let (r_id, r_name, r_price, purchased) = &mut inventory.relics[idx];
                if *purchased {
                    continue;
                }
                if run_state.poke_dollars >= *r_price {
                    run_state.poke_dollars -= *r_price;
                    run_state.relics.push(r_id.clone());
                    *purchased = true;
                    *bg_color = BackgroundColor(Color::srgb(0.3, 0.3, 0.3));

                    let msg = format!("[RELIQUIA] ¡Has equipado la reliquia {} por {} P$!", r_name, r_price);
                    run_state.add_log(msg.clone());
                    if let Ok(mut text) = q_status.single_mut() {
                        **text = msg;
                    }
                } else {
                    if let Ok(mut text) = q_status.single_mut() {
                        **text = "[AVISO] ¡Poké-Dólares insuficientes para adquirir esta Reliquia!".to_string();
                    }
                }
            }
        }
    }
}

fn handle_buy_consumable(
    mut q_buttons: Query<(&Interaction, &ShopBuyConsumableButton, &mut BackgroundColor), Changed<Interaction>>,
    mut run_state: ResMut<CurrentRunState>,
    mut inventory: ResMut<ShopInventory>,
    mut q_status: Query<&mut Text, With<ShopStatusBanner>>,
) {
    for (interaction, btn, mut bg_color) in q_buttons.iter_mut() {
        if *interaction == Interaction::Pressed {
            let idx = btn.0;
            if idx < inventory.consumables.len() {
                let (c_id, c_name, c_price, purchased) = &mut inventory.consumables[idx];
                if *purchased {
                    continue;
                }

                // Busca ranura libre en el cinturón de consumibles
                let empty_slot = run_state.consumables.iter().position(|s| s.is_none());
                match empty_slot {
                    Some(slot_idx) => {
                        if run_state.poke_dollars >= *c_price {
                            run_state.poke_dollars -= *c_price;
                            run_state.consumables[slot_idx] = Some(c_id.clone());
                            *purchased = true;
                            *bg_color = BackgroundColor(Color::srgb(0.3, 0.3, 0.3));

                            let msg = format!("[OBJETO] ¡Has comprado {} por {} P$! Guardado en ranura {}.", c_name, c_price, slot_idx + 1);
                            run_state.add_log(msg.clone());
                            if let Ok(mut text) = q_status.single_mut() {
                                **text = msg;
                            }
                        } else {
                            if let Ok(mut text) = q_status.single_mut() {
                                **text = "[AVISO] ¡Poké-Dólares insuficientes para comprar este consumible!".to_string();
                            }
                        }
                    }
                    None => {
                        if let Ok(mut text) = q_status.single_mut() {
                            **text = "[AVISO] Tu cinturón de consumibles está lleno (3/3). Usa uno en combate primero.".to_string();
                        }
                    }
                }
            }
        }
    }
}

fn handle_remove_card(
    mut q_buttons: Query<&Interaction, (Changed<Interaction>, With<ShopRemoveCardButton>)>,
    mut run_state: ResMut<CurrentRunState>,
    mut q_status: Query<&mut Text, With<ShopStatusBanner>>,
) {
    for interaction in q_buttons.iter_mut() {
        if *interaction == Interaction::Pressed {
            let cost = run_state.card_removal_cost;
            if run_state.poke_dollars >= cost {
                // Elimina la primera carta básica disponible (Placaje o Refugio)
                let basic_pos = run_state.player_deck.iter().position(|c| {
                    c.id == "tackle" || c.id == "shelter" || c.id == "scratch" || c.id == "growl"
                });

                if let Some(pos) = basic_pos {
                    run_state.poke_dollars -= cost;
                    let removed = run_state.player_deck.remove(pos);
                    run_state.card_removal_cost += 25;
                    let new_cost = run_state.card_removal_cost;

                    let msg = format!("[OLVIDO] Movimiento '{}' olvidado permanentemente. Nuevo coste: {} P$.", removed.display_name(), new_cost);
                    run_state.add_log(msg.clone());
                    if let Ok(mut text) = q_status.single_mut() {
                        **text = msg;
                    }
                } else if !run_state.player_deck.is_empty() {
                    // Si no hay básicas, elimina la última carta
                    run_state.poke_dollars -= cost;
                    let removed = run_state.player_deck.pop().unwrap();
                    run_state.card_removal_cost += 25;
                    let msg = format!("[OLVIDO] Movimiento '{}' olvidado. Nuevo coste: {} P$.", removed.display_name(), run_state.card_removal_cost);
                    run_state.add_log(msg.clone());
                    if let Ok(mut text) = q_status.single_mut() {
                        **text = msg;
                    }
                } else {
                    if let Ok(mut text) = q_status.single_mut() {
                        **text = "[AVISO] No te quedan cartas en el mazo para olvidar.".to_string();
                    }
                }
            } else {
                if let Ok(mut text) = q_status.single_mut() {
                    **text = format!("[AVISO] Se requieren {} P$ para el servicio de olvido.", cost);
                }
            }
        }
    }
}

use crate::ui::transition_ui::TransitionToEvent;

fn handle_exit_shop(
    mut q_buttons: Query<&Interaction, (Changed<Interaction>, With<ShopExitButton>)>,
    mut run_state: ResMut<CurrentRunState>,
    mut ev_trans: MessageWriter<TransitionToEvent>,
) {
    for interaction in q_buttons.iter_mut() {
        if *interaction == Interaction::Pressed {
            // Salir de la tienda completa el piso y avanza al siguiente
            run_state.current_floor += 1;
            ev_trans.write(TransitionToEvent {
                target: AppState::MapNavigation,
            });
            break;
        }
    }
}

fn update_shop_displays(
    run_state: Res<CurrentRunState>,
    mut q_balance: Query<&mut Text, With<ShopBalanceText>>,
    mut q_removal_cost: Query<&mut Text, (With<ShopRemovalCostText>, Without<ShopBalanceText>)>,
) {
    if run_state.is_changed() {
        for mut text in q_balance.iter_mut() {
            **text = format!("Saldo: {} P$", run_state.poke_dollars);
        }
        for mut text in q_removal_cost.iter_mut() {
            **text = format!("Coste actual: {} P$", run_state.card_removal_cost);
        }
    }
}

fn cleanup_shop_ui(mut commands: Commands, q_cleanup: Query<Entity, With<CleanupUi>>) {
    for entity in q_cleanup.iter() {
        commands.entity(entity).despawn();
    }
}
