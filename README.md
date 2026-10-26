# 🔥 SpireMon: Roguelike Deckbuilder RPG (Rust + Bevy Engine 0.20)

Implementación fiel del **Game Design Document (GDD)** en **Rust (Edición 2024)** utilizando la arquitectura ECS de **Bevy Engine (v0.20)**.

---

## 🎮 Características Implementadas

### 1. Sistema RPG y Fórmulas Matemáticas Canónicas
- **HP Progresivo y Curva de Vida:** 
  $$\text{HP} = \text{round}\left(\text{Base HP} + 41.0 + \frac{47.0 \times \text{Lvl}}{341.0} + \frac{45.0 \times \text{Lvl}^2}{341.0}\right)$$
  - *Charmander (Base HP 39, Nv. 5)* = **84 HP**
  - *Charmeleon (Base HP 58, Nv. 16)* = **135 HP**
  - *Charizard (Base HP 78, Nv. 36)* = **295 HP**
- **Mitigación por Defensa:**
  $$\text{Factor de Mitigación} = \frac{100.0}{100.0 + \text{Def}}$$
- **Efectividades Elementales y STAB:**
  - Matriz de 18 tipos elementales (Súper Eficaz $\times 1.5$, Neutro $\times 1.0$, Resistente $\times 0.65$, Inmune $\times 0.25$).
  - Bono STAB de $+20\%$ en multiplicador si el movimiento coincide con la afinidad elemental primaria del Pokémon.

### 2. Catálogo Maestro de Cartas y Reliquias
- **32 Cartas Maestras:**
  - 12 Ataques Físicos, 10 Ataques Especiales, 6 Habilidades de Bloqueo/Buff, 4 Poderes Legendarios.
  - Soporte de versiones entrenadas (`+`).
- **Reliquias:**
  - Reliquias iniciales (*Carbón*, *Semilla Milagro*, *Agua Mística*, *Imán*, *Pañuelo Seda*).
  - Reliquias comunes, poco comunes y de tienda (*Tarjeta VIP*, *Moneda Amuleto*, *Restos*, *Cinta Elegida*, etc.).

### 3. Generación Procedural de Mapa (Acto 1 - 15 Pisos)
- Estructura procedural de 3 caminos interconectados.
- Máximo 3 Mini Jefes por Acto con al menos 1 piso de separación.
- **Centro Pokémon garantizado** en el piso inmediatamente posterior a cada Mini Jefe.
- Piso 15 convergente en el Líder de Gimnasio (*Brock / Onix*).

### 4. Sprites Oficiales (PokeAPI)
- 38 sprites oficiales en formato PNG:
  - `assets/sprites/pokemon/front/{id}.png` (vista frontal para rivales y showcase).
  - `assets/sprites/pokemon/back/{id}.png` (vista dorsal clásica para el jugador en combate).
  - Cubre todas las 19 especies clave (#1, #2, #3, #4, #5, #6, #7, #8, #9, #10, #19, #25, #26, #74, #95, #121, #123, #143, #150).

### 5. Interfaz Gráfica 2D Interactiva (UI/UX en Bevy)
- **`MainMenu` (`menu_ui.rs`):** Pantalla de inicio con sprite del compañero activo y accesos directos tácticos.
- **`Combat` (`combat_ui.rs`):** HUD con Top Bar, arena 2D con sprites dorsal/frontal, barras de HP reactivas, insignias de bloqueo, orbe de energía (`3/3 ⚡`), abanico de cartas interactivas y botón de terminar turno.
- **`MapNavigation` (`map_ui.rs`):** Visualizador de nodos interactivos (`⚔️`, `👾`, `🏪`, `🏥`, `📦`, `👑`).
- **`Shop` (`shop_ui.rs`):** Poké Mart con 6 cartas, etiqueta brillante de ofertas aleatorias ($-25\%$ a $-90\%$), reliquias, consumibles y servicio de "Olvidar Movimiento".
- **`PokemonCenter` (`center_ui.rs`):** Sala de recuperación con opciones de curar $+35\%$ HP o entrenar una carta a su versión `+`.
- **`GachaponShop` (`gachapon_ui.rs`):** Máquina de cápsulas con selector de hasta 3 fragmentos para *Rate-Up* focalizado y tiradas 1x / 10x.

---

## 🛠️ Cómo Compilar y Ejecutar

### Pruebas Unitarias e Integración (10/10 Tests)
```bash
cargo test
```

### Ejecución en Modo Demostración CLI / Headless
Ideal para terminales sin servidor X11/Wayland activo o para verificar simulación en pipelines:
```bash
cargo run -- --demo
```

### Ejecución en Modo Gráfico 2D Interactivo (Ventana 1280x720)
```bash
cargo run
```
- **Wayland Nativo:** Se conecta directamente a tu sesión Wayland de KDE Plasma / GNOME con escalado fraccional y aceleración gráfica.
- **Configuración de Backend:** Utiliza el backend primario nativo de Wayland (`Backends::PRIMARY` / Vulkan con `VK_KHR_wayland_surface`). Si tu hardware lo requiere, también puedes especificar `WGPU_BACKEND=gl cargo run`.
