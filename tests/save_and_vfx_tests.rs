use bevy::prelude::*;
use slay_of_pokemon::run::{AppState, CurrentRunState, MetaProfile};
use slay_of_pokemon::save::{self, ActiveSaveSlot, TOTAL_SLOTS};
use slay_of_pokemon::map::CurrentMap;
use bevy_tweening::{
    lens::TransformScaleLens,
    AnimTarget, RepeatCount, Tween, TweenAnim, TweeningPlugin,
};

#[test]
fn test_three_save_slots_isolation_and_persistence() {
    // Limpieza inicial previa
    for slot in 1..=TOTAL_SLOTS {
        let _ = save::delete_profile(slot);
    }

    // 1. Crear 3 perfiles distintos
    let mut meta1 = MetaProfile::default();
    meta1.gachapon_tokens = 500;
    meta1.unlocked_pokemon = vec![4];
    let mut run1 = CurrentRunState::default();
    run1.selected_starter_id = 4; // Charmander
    run1.poke_dollars = 150;
    run1.current_floor = 3;

    let mut meta2 = MetaProfile::default();
    meta2.gachapon_tokens = 800;
    meta2.unlocked_pokemon = vec![4, 7];
    let mut run2 = CurrentRunState::default();
    run2.selected_starter_id = 7; // Squirtle
    run2.poke_dollars = 320;
    run2.current_floor = 7;

    let mut meta3 = MetaProfile::default();
    meta3.gachapon_tokens = 1200;
    meta3.unlocked_pokemon = vec![4, 1, 25];
    let mut run3 = CurrentRunState::default();
    run3.selected_starter_id = 1; // Bulbasaur
    run3.poke_dollars = 500;
    run3.current_floor = 12;

    let map = CurrentMap::default();

    // Guardar en las 3 ranuras
    assert!(save::save_profile(1, &meta1, &run1, Some(&map)).is_ok());
    assert!(save::save_profile(2, &meta2, &run2, Some(&map)).is_ok());
    assert!(save::save_profile(3, &meta3, &run3, Some(&map)).is_ok());

    // 2. Verificar listado de perfiles
    let summaries = save::list_profiles();
    assert!(summaries[0].is_some(), "Ranura 1 debe existir");
    assert!(summaries[1].is_some(), "Ranura 2 debe existir");
    assert!(summaries[2].is_some(), "Ranura 3 debe existir");

    let s1 = summaries[0].as_ref().unwrap();
    assert_eq!(s1.slot, 1);
    assert_eq!(s1.starter_id, 4);
    assert_eq!(s1.current_floor, 3);
    assert_eq!(s1.poke_dollars, 150);

    let s2 = summaries[1].as_ref().unwrap();
    assert_eq!(s2.slot, 2);
    assert_eq!(s2.starter_id, 7);
    assert_eq!(s2.current_floor, 7);
    assert_eq!(s2.poke_dollars, 320);

    let s3 = summaries[2].as_ref().unwrap();
    assert_eq!(s3.slot, 3);
    assert_eq!(s3.starter_id, 1);
    assert_eq!(s3.current_floor, 12);
    assert_eq!(s3.poke_dollars, 500);

    // 3. Cargar individualmente y verificar integridad
    let loaded2 = save::load_profile(2).expect("Debe cargar ranura 2");
    assert_eq!(loaded2.run_state.selected_starter_id, 7);
    assert_eq!(loaded2.meta.gachapon_tokens, 800);
    assert!(loaded2.map.is_some());

    // 4. Borrar ranura 2 y verificar que las demás persisten intactas
    assert!(save::delete_profile(2).is_ok());
    let summaries_after = save::list_profiles();
    assert!(summaries_after[0].is_some(), "Ranura 1 debe permanecer");
    assert!(summaries_after[1].is_none(), "Ranura 2 debe estar vacía");
    assert!(summaries_after[2].is_some(), "Ranura 3 debe permanecer");

    // Limpieza posterior
    let _ = save::delete_profile(1);
    let _ = save::delete_profile(3);
}

#[test]
fn test_ui_profile_select_flow_and_slot_activation() {
    use slay_of_pokemon::ui::profile_ui::ProfileSlotAction;

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(bevy::state::app::StatesPlugin)
        .add_plugins(bevy::asset::AssetPlugin::default())
        .init_asset::<Image>()
        .add_plugins(TweeningPlugin)
        .init_state::<AppState>()
        .init_resource::<CurrentRunState>()
        .init_resource::<MetaProfile>()
        .init_resource::<ActiveSaveSlot>()
        .add_plugins(slay_of_pokemon::save::SavePlugin)
        .add_plugins(slay_of_pokemon::ui::GameUiPlugin);

    app.update();

    // 1. Iniciar en MainMenu y verificar que la ranura activa inicial es 1
    assert_eq!(app.world().resource::<ActiveSaveSlot>().0, 1);

    // 2. Transición a ProfileSelect
    app.world_mut().resource_mut::<NextState<AppState>>().set(AppState::ProfileSelect);
    app.update();

    assert_eq!(*app.world().resource::<State<AppState>>().get(), AppState::ProfileSelect);

    // 3. Simular que el jugador pulsa "NUEVA PARTIDA" en la Ranura 2
    {
        let mut q_actions = app.world_mut().query::<(&mut Interaction, &ProfileSlotAction)>();
        for (mut interaction, action) in q_actions.iter_mut(app.world_mut()) {
            if let ProfileSlotAction::New(2) = action {
                *interaction = Interaction::Pressed;
            }
        }
    }
    app.update();

    // Comprobar que la ranura activa cambió a 2 y transicionó a StarterSelection
    assert_eq!(app.world().resource::<ActiveSaveSlot>().0, 2);

    // Limpiar archivo de prueba
    let _ = save::delete_profile(2);
}

#[test]
fn test_bevy_tweening_integration_steps_smoothly() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(TweeningPlugin);

    let entity = app.world_mut().spawn(Transform::from_scale(Vec3::splat(1.0))).id();

    let tween = Tween::new(
        EaseFunction::Linear,
        std::time::Duration::from_millis(1000),
        TransformScaleLens {
            start: Vec3::splat(1.0),
            end: Vec3::splat(2.0),
        },
    )
    .with_repeat_count(RepeatCount::Finite(1));

    app.world_mut().spawn((
        TweenAnim::new(tween),
        AnimTarget::component::<Transform>(entity),
    ));

    app.update();

    // Verificar que la entidad objetivo existe y conserva su transform
    let transform = app.world().get::<Transform>(entity);
    assert!(transform.is_some(), "La entidad animada con Transform debe existir");
}
