#![allow(deprecated)]

pub mod types;
pub mod stats;
pub mod card;
pub mod relics;
pub mod pokemon;
pub mod run;
pub mod combat;
pub mod map;
pub mod shop;
pub mod pokemon_center;
pub mod gachapon;
pub mod save;
pub mod vfx;
pub mod ui;

pub use bevy_save::prelude as bevy_save_prelude;
pub use bevy_hanabi::prelude as hanabi;
pub use bevy_tweening as tweening;

include!(concat!(env!("OUT_DIR"), "/embedded_assets.rs"));

/// Construye un directorio virtual en memoria (`bevy::asset::io::memory::Dir`) con todos los assets
/// incrustados en tiempo de compilación dentro del binario para que el ejecutable sea 100% autocontenido.
pub fn build_embedded_asset_dir() -> bevy::asset::io::memory::Dir {
    let dir = bevy::asset::io::memory::Dir::default();
    for &(rel_path, bytes) in EMBEDDED_ASSETS {
        dir.insert_asset(std::path::Path::new(rel_path), bytes);
    }
    dir
}

/// Registra el `MemoryAssetReader` autocontenido como la fuente por defecto (`AssetSourceId::Default`)
/// de Bevy antes de inicializar `DefaultPlugins` / `AssetPlugin`.
pub fn register_embedded_assets(app: &mut bevy::prelude::App) {
    use bevy::asset::io::{
        memory::MemoryAssetReader, AssetSourceBuilder, AssetSourceId,
    };
    use bevy::asset::AssetApp;

    let dir = build_embedded_asset_dir();
    app.register_asset_source(
        AssetSourceId::Default,
        AssetSourceBuilder::new(move || {
            Box::new(MemoryAssetReader {
                root: dir.clone(),
            })
        }),
    );
}

/// Resuelve la ruta raíz del directorio `assets/` para asegurar que el motor Bevy
/// siempre localice las texturas, fuentes y fondos tanto en `cargo run` como al ejecutar
/// el binario compilado directamente desde `target/debug`, `target/release` o un empaquetado.
pub fn resolve_asset_root() -> std::path::PathBuf {
    // 1. Variable de entorno explícita de Bevy
    if let Ok(root) = std::env::var("BEVY_ASSET_ROOT") {
        let p = std::path::PathBuf::from(root);
        if p.join("assets").exists() {
            return p;
        }
    }

    // 2. Variable CARGO_MANIFEST_DIR (fijada por cargo run y cargo test)
    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        let p = std::path::PathBuf::from(manifest_dir);
        if p.join("assets").exists() {
            return p;
        }
    }

    // 3. Comprobar si 'assets/' existe en el directorio de trabajo actual
    if let Ok(cwd) = std::env::current_dir() {
        if cwd.join("assets").exists() {
            return cwd;
        }
    }

    // 4. Comprobar respecto a la ubicación del ejecutable (mismo directorio o subiendo carpetas padre)
    if let Ok(exe_path) = std::env::current_exe() {
        let mut dir = exe_path.parent();
        while let Some(parent) = dir {
            if parent.join("assets").exists() {
                return parent.to_path_buf();
            }
            dir = parent.parent();
        }
    }

    // 5. Fallback al directorio actual
    std::path::PathBuf::from(".")
}
