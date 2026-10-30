const path = require('path');
const { Doc, C } = require('./ui_lib');

const OUT = path.resolve(__dirname, '../../assets/ui/menus');
const SPRITE_PIKA = path.resolve(__dirname, '../../assets/sprites/pokemon/front/25.png');
const SPRITE_CHAR = path.resolve(__dirname, '../../assets/sprites/pokemon/front/6.png');

function chevron(c, x, y, size, color, weight = 2.2) {
  c.path(`M ${x} ${y - size} L ${x + size} ${y} L ${x} ${y + size}`, {
    closed: false,
    fill: null,
    stroke: color,
    strokeW: weight,
    strokeOpts: { cap: 'round', join: 'round' },
  });
}

function playIcon(c, cx, cy, size, color) {
  c.path(`M ${cx - size * 0.45} ${cy - size * 0.55} L ${cx + size * 0.55} ${cy} L ${cx - size * 0.45} ${cy + size * 0.55} Z`, {
    fill: color,
    stroke: null,
    // slight rounding is not available on a simple triangle; a soft shadow reads as polish
    shadow: { opacity: 30, y: 1, blur: 2, color: '#000000' },
  });
}

function saveIcon(c, cx, cy, size, color) {
  c.rect(cx - size / 2, cy - size / 2, size, size, 2, { fill: null, stroke: color, strokeW: 2 });
  c.rect(cx - size * 0.28, cy - size * 0.46, size * 0.56, size * 0.3, 1, { fill: color, stroke: null });
  c.rect(cx - size * 0.3, cy + size * 0.06, size * 0.6, size * 0.42, 1, { fill: null, stroke: color, strokeW: 1.6 });
}

function exitIcon(c, cx, cy, size, color) {
  c.path(
    `M ${cx - size * 0.4} ${cy - size * 0.55} L ${cx - size * 0.82} ${cy - size * 0.55} L ${cx - size * 0.82} ${cy + size * 0.55} L ${cx - size * 0.4} ${cy + size * 0.55}`,
    { closed: false, fill: null, stroke: color, strokeW: 2.2, strokeOpts: { cap: 'round', join: 'round' } }
  );
  c.line(cx - size * 0.4, cy, cx + size * 0.72, cy, color, 2.2, { cap: 'round' });
  c.path(`M ${cx + size * 0.36} ${cy - size * 0.28} L ${cx + size * 0.72} ${cy} L ${cx + size * 0.36} ${cy + size * 0.28}`, {
    closed: false,
    fill: null,
    stroke: color,
    strokeW: 2.2,
    strokeOpts: { cap: 'round', join: 'round' },
  });
}

function pauseIcon(c, cx, cy, size, color) {
  c.rect(cx - size * 0.36, cy - size * 0.4, size * 0.24, size * 0.8, 2, { fill: color, stroke: null });
  c.rect(cx + size * 0.12, cy - size * 0.4, size * 0.24, size * 0.8, 2, { fill: color, stroke: null });
}

function closeIcon(c, cx, cy, size, color) {
  c.line(cx - size, cy - size, cx + size, cy + size, color, 2.2, { cap: 'round' });
  c.line(cx + size, cy - size, cx - size, cy + size, color, 2.2, { cap: 'round' });
}

function gearIcon(c, cx, cy, size, color, hole) {
  const teeth = 8;
  for (let i = 0; i < teeth; i++) {
    const a = (i * 360) / teeth;
    c.rect(cx - size * 0.13, cy - size * 1.08, size * 0.26, size * 0.36, size * 0.05, { fill: color, stroke: null });
    c.cmd('object.rotate', { angle: a, origin: [c.ox + cx, c.oy + cy] });
  }
  c.circle(cx, cy, size * 0.72, { fill: color, stroke: null });
  c.circle(cx, cy, size * 0.32, { fill: hole, stroke: null });
}

function menuButton(c, x, y, w, h, opts) {
  const tone = opts.tone || 'dark';
  const label = opts.label;
  const accent = opts.accent || C.gold;
  const radius = h / 2;
  const iconCx = x + 34;
  const cy = y + h / 2;
  const bodyGrad =
    tone === 'gold'
      ? [
          { offset: 0, color: '#ffe9a6' },
          { offset: 0.38, color: '#f2c14e' },
          { offset: 0.72, color: '#d38f1e' },
          { offset: 1, color: '#a96210' },
        ]
      : tone === 'danger'
        ? [
            { offset: 0, color: '#341620' },
            { offset: 0.65, color: '#1d0b12' },
            { offset: 1, color: '#10060a' },
          ]
        : [
            { offset: 0, color: '#1b2638' },
            { offset: 0.62, color: '#111a29' },
            { offset: 1, color: '#090f19' },
          ];
  const border = tone === 'gold' ? '#6b4306' : tone === 'danger' ? '#8c2c3a' : '#3b5374';
  const txt = tone === 'gold' ? '#241503' : tone === 'danger' ? '#ff9da7' : '#e8effc';
  const iconBg = tone === 'gold' ? '#2a1a06' : tone === 'danger' ? '#2a0d14' : '#0b1320';

  c.rect(x, y + 4, w, h, radius, {
    fill: '#000000',
    stroke: null,
    shadow: tone === 'gold' ? { opacity: 45, y: 7, blur: 14, color: '#000000' } : { opacity: 42, y: 5, blur: 10 },
  });
  c.rect(x, y, w, h, radius, {
    fillGrad: bodyGrad,
    stroke: border,
    strokeW: tone === 'gold' ? 2 : 1.8,
    glow: tone === 'gold' ? { opacity: 20, blur: 16, color: C.gold } : null,
  });
  c.rect(x + 5, y + 3, w - 10, h * 0.36, radius - 4, {
    fillGrad: [
      { offset: 0, color: '#ffffff', opacity: tone === 'gold' ? 0.4 : 0.14 },
      { offset: 1, color: '#ffffff', opacity: 0 },
    ],
    stroke: null,
  });

  // icon medallion
  c.circle(iconCx, cy, 17, { fill: iconBg, stroke: tone === 'gold' ? '#8b5a08' : border, strokeW: 1.4 });
  if (opts.icon === 'pokeball') {
    c.pokeball(iconCx, cy, 9.5, '#f0525c', '#f7f9ff', '#111827', { strokeW: 1.8 });
  } else if (opts.icon === 'play') {
    playIcon(c, iconCx, cy, 12, tone === 'gold' ? '#ffe4a3' : C.gold);
  } else if (opts.icon === 'gear') {
    gearIcon(c, iconCx, cy, 7.5, tone === 'gold' ? '#ffe4a3' : C.muted, iconBg);
  } else if (opts.icon === 'exit') {
    exitIcon(c, iconCx, cy, 9, tone === 'gold' ? '#ffe4a3' : '#ff8a94');
  }

  c.text(x + 64, y, w - 118, h, label, {
    size: 15,
    tracking: 135,
    color: txt,
    justify: 'left',
  });
  if (opts.note) {
    c.text(x + w - 134, y, 92, h, opts.note, {
      size: 8.5,
      style: 'Medium',
      tracking: 110,
      color: tone === 'gold' ? '#5c3a08' : tone === 'danger' ? '#b46772' : C.muted,
      justify: 'right',
    });
  }
  chevron(c, x + w - 30, cy, 6, tone === 'gold' ? '#5c3a08' : tone === 'danger' ? '#b46772' : '#8fa3c0', 2);
}

function drawMainMenu(c) {
  // base
  c.rect(0, 0, 1280, 720, 0, {
    fillGrad: [
      { offset: 0, color: '#0f1928' },
      { offset: 0.52, color: '#0a111c' },
      { offset: 1, color: '#05080d' },
    ],
    stroke: null,
  });
  // atmospheric washes
  c.ellipse(640, -260, 900, 620, {
    fillGrad: [
      { offset: 0, color: '#4aa8ff', opacity: 0.1 },
      { offset: 1, color: '#4aa8ff', opacity: 0 },
    ],
    stroke: null,
  });
  c.ellipse(-240, 260, 620, 540, {
    fillGrad: [
      { offset: 0, color: '#ff7a3d', opacity: 0.09 },
      { offset: 1, color: '#ff7a3d', opacity: 0 },
    ],
    stroke: null,
  });
  // diagonal light beams
  for (const [x1, x2, op] of [
    [690, 1060, 2.2],
    [760, 1130, 1.6],
    [830, 1200, 1.1],
  ]) {
    c.path(`M ${x1} 0 L ${x1 + 120} 0 L ${x2 + 120} 720 L ${x2} 720 Z`, {
      fill: '#ffffff',
      stroke: null,
      opacity: op,
    });
  }
  // giant faint pokeball, top right
  c.circle(1110, 70, 240, { fill: null, stroke: '#4aa8ff', strokeW: 2, opacity: 5 });
  c.circle(1110, 70, 166, { fill: null, stroke: '#4aa8ff', strokeW: 1.4, opacity: 4 });
  // tiny grid dots
  for (let gx = 0; gx < 6; gx++) {
    for (let gy = 0; gy < 4; gy++) {
      c.circle(760 + gx * 86, 566 + gy * 38, 1.4, { fill: '#8fa3c0', stroke: null, opacity: 9 });
    }
  }

  // ---- left column ----------------------------------------------------
  c.rect(82, 155, 42, 2, 1, { fill: C.gold, stroke: null });
  c.text(134, 142, 500, 28, 'ROGUELIKE DE CARTAS · POKÉMON', {
    size: 11.5,
    tracking: 200,
    color: C.gold,
    justify: 'left',
  });

  c.text(78, 170, 620, 142, 'SLAY', {
    size: 128,
    font: 'Noto Sans Black',
    style: 'Regular',
    tracking: -18,
    color: '#f8f3e6',
    justify: 'left',
  });
  c.text(82, 280, 620, 90, 'OF POKÉMON', {
    size: 64,
    font: 'Noto Sans Black',
    style: 'Regular',
    tracking: 8,
    color: '#ffb347',
    justify: 'left',
  });
  // warm gradient over the second line
  c.fillGrad([
    { offset: 0, color: '#ffd76b' },
    { offset: 0.5, color: '#ff8a3d' },
    { offset: 1, color: '#e23b2b' },
  ]);

  c.text(84, 362, 560, 26, 'FORJA TU MAZO · DOMINA LOS GIMNASIOS · SOBREVIVE AL ACTO', {
    size: 11,
    style: 'Medium',
    tracking: 170,
    color: C.muted,
    justify: 'left',
  });

  menuButton(c, 82, 400, 430, 58, { tone: 'gold', label: 'INICIAR RUN', icon: 'pokeball', note: 'NUEVO' });
  menuButton(c, 82, 470, 430, 58, { tone: 'dark', label: 'CONTINUAR', icon: 'play', note: 'PISO 04' });
  menuButton(c, 82, 540, 430, 58, { tone: 'dark', label: 'OPCIONES', icon: 'gear', note: 'AUDIO · VÍDEO' });
  menuButton(c, 82, 610, 430, 58, { tone: 'danger', label: 'SALIR', icon: 'exit', note: 'CERRAR JUEGO' });

  // ---- team panel -----------------------------------------------------
  c.panel(780, 104, 438, 520, 28, { accent: '#3b5374', borderW: 2.2, shadow: { opacity: 65, y: 14, blur: 26 } });
  c.text(812, 124, 300, 24, 'EQUIPO ACTIVO', {
    size: 11,
    tracking: 210,
    color: C.muted,
    justify: 'left',
  });
  c.text(1100, 122, 86, 26, '3 / 6', {
    size: 12,
    tracking: 120,
    color: C.gold,
    justify: 'right',
  });

  // stage
  c.rect(812, 158, 374, 250, 20, {
    fillGrad: [
      { offset: 0, color: '#0c1522' },
      { offset: 1, color: '#060a11' },
    ],
    stroke: '#2a3a55',
    strokeW: 1.5,
  });
  // inner arena glow
  c.ellipse(902, 196, 194, 194, {
    fillGrad: [
      { offset: 0, color: '#4aa8ff', opacity: 0.1 },
      { offset: 1, color: '#4aa8ff', opacity: 0 },
    ],
    stroke: null,
  });
  c.circle(999, 286, 88, { fill: null, stroke: '#4aa8ff', strokeW: 1, opacity: 10 });
  c.circle(999, 286, 118, { fill: null, stroke: '#4aa8ff', strokeW: 1, opacity: 7 });
  // floor
  c.ellipse(849, 340, 300, 40, {
    fillGrad: [
      { offset: 0, color: '#131e2e' },
      { offset: 1, color: '#080d15' },
    ],
    stroke: '#293b56',
    strokeW: 1.2,
  });
  c.ellipse(889, 350, 220, 22, { fill: '#000000', opacity: 35, stroke: null });
  // sprite
  c.image(SPRITE_PIKA, 919, 200, 160, 160);
  // nameplate / stats
  c.text(824, 426, 220, 28, 'PIKACHU', {
    size: 18,
    tracking: 40,
    color: C.text,
    justify: 'left',
  });
  c.text(1004, 428, 168, 24, 'Nv. 18 · ELÉCTRICO', {
    size: 9.5,
    style: 'Medium',
    tracking: 60,
    color: C.muted,
    justify: 'right',
  });
  c.rect(824, 458, 348, 9, 4.5, { fill: '#070b12', stroke: '#2a3a55', strokeW: 1 });
  c.rect(826, 460, 244, 5, 2.5, {
    fillGrad: [
      { offset: 0, color: '#7dffb0' },
      { offset: 1, color: '#2fae61' },
    ],
    stroke: null,
  });
  c.text(824, 472, 348, 18, 'HP 84 / 120', {
    size: 8.5,
    style: 'Medium',
    tracking: 80,
    color: C.muted,
    justify: 'right',
  });

  // divider
  c.rect(824, 500, 348, 1, 0, {
    fillGrad: [
      { offset: 0, color: '#3b5374', opacity: 0 },
      { offset: 0.5, color: '#3b5374', opacity: 0.6 },
      { offset: 1, color: '#3b5374', opacity: 0 },
    ],
    stroke: null,
  });

  // Charizard mini row
  c.circle(848, 538, 26, { fill: '#0a111c', stroke: '#3b5374', strokeW: 1.5 });
  c.image(SPRITE_CHAR, 822, 512, 52, 52);
  c.text(886, 514, 190, 22, 'CHARIZARD', {
    size: 12,
    tracking: 60,
    color: C.text,
    justify: 'left',
  });
  c.text(886, 535, 120, 18, 'Nv. 16 · FUEGO', {
    size: 8,
    style: 'Medium',
    tracking: 50,
    color: C.muted,
    justify: 'left',
  });
  c.rect(886, 554, 190, 6, 3, { fill: '#070b12', stroke: null });
  c.rect(887, 555, 150, 4, 2, {
    fillGrad: [
      { offset: 0, color: '#ffb08a' },
      { offset: 1, color: '#d9472f' },
    ],
    stroke: null,
  });
  c.text(1084, 550, 88, 18, '42 / 52', {
    size: 8,
    style: 'Medium',
    tracking: 60,
    color: C.muted,
    justify: 'right',
  });

  // add slot
  c.circle(848, 590, 14, { fill: null, stroke: '#3b5374', strokeW: 1.2, opacity: 60 });
  c.text(838, 580, 20, 20, '+', { size: 16, color: '#8fa3c0', justify: 'center' });
  c.text(872, 578, 230, 24, 'AÑADIR AL EQUIPO', {
    size: 9,
    tracking: 120,
    color: C.muted,
    justify: 'left',
  });
  c.text(1080, 578, 92, 24, '3 ESPACIOS', {
    size: 7.5,
    style: 'Medium',
    tracking: 80,
    color: '#5f7392',
    justify: 'right',
  });

  // footer
  c.rect(82, 686, 1136, 1, 0, { fill: '#3b5374', opacity: 18, stroke: null });
  c.text(82, 694, 500, 18, 'v0.1.0 · BUILD 2026.10 · PROTOTIPO', {
    size: 8,
    style: 'Medium',
    tracking: 100,
    color: '#5f7392',
    justify: 'left',
  });
  c.text(720, 694, 498, 18, 'PERFIL: ASH · FICHAS 120 · ACTO 1', {
    size: 8,
    style: 'Medium',
    tracking: 100,
    color: '#5f7392',
    justify: 'right',
  });
}

function drawPauseMenu(c) {
  // modal panel only (transparent backdrop for overlay use)
  c.panel(30, 30, 700, 560, 30, { accent: '#3b5374', borderW: 2.4, shadow: { opacity: 68, y: 16, blur: 30 } });
  // top accent line
  c.rect(76, 30.8, 608, 2, 1, {
    fillGrad: [
      { offset: 0, color: C.gold, opacity: 0 },
      { offset: 0.5, color: C.gold, opacity: 0.8 },
      { offset: 1, color: C.gold, opacity: 0 },
    ],
    stroke: null,
  });

  // header
  c.circle(78, 76, 20, { fill: '#0b1320', stroke: '#3b5374', strokeW: 1.5 });
  pauseIcon(c, 78, 76, 12, C.gold);
  c.text(112, 56, 380, 18, 'PAUSA', { size: 9.5, tracking: 220, color: C.gold, justify: 'left' });
  c.text(110, 74, 480, 42, 'MENÚ DE JUEGO', {
    size: 26,
    font: 'Noto Sans Black',
    style: 'Regular',
    tracking: 0,
    color: C.text,
    justify: 'left',
  });

  // close button
  c.rect(654, 52, 44, 44, 12, {
    fillGrad: [
      { offset: 0, color: '#1a2436' },
      { offset: 1, color: '#0a111c' },
    ],
    stroke: '#3b5374',
    strokeW: 1.4,
  });
  closeIcon(c, 676, 74, 7, '#c4d2e6');

  c.rect(60, 126, 640, 1, 0, {
    fillGrad: [
      { offset: 0, color: '#3b5374', opacity: 0 },
      { offset: 0.5, color: '#3b5374', opacity: 0.65 },
      { offset: 1, color: '#3b5374', opacity: 0 },
    ],
    stroke: null,
  });

  const rows = [
    { y: 146, label: 'CONTINUAR', sub: 'Volver a la partida', icon: 'play', tone: 'gold', note: 'ESC' },
    { y: 214, label: 'GUARDAR PARTIDA', sub: 'Última guarda: hace 2 min', icon: 'save', tone: 'dark' },
    { y: 282, label: 'OPCIONES', sub: 'Audio, vídeo y controles', icon: 'gear', tone: 'dark' },
    { y: 350, label: 'SALIR AL MENÚ', sub: 'Se perderá el progreso no guardado', icon: 'exit', tone: 'danger' },
  ];
  for (const r of rows) {
    const x = 60;
    const w = 640;
    const h = 58;
    const accent = r.tone === 'gold' ? C.gold : r.tone === 'danger' ? '#8c2c3a' : '#3b5374';
    const labelColor = r.tone === 'gold' ? '#ffe6a8' : r.tone === 'danger' ? '#ff9da7' : C.text;
    c.rect(x, r.y, w, h, 16, {
      fillGrad:
        r.tone === 'gold'
          ? [
              { offset: 0, color: '#3a2a0e' },
              { offset: 0.6, color: '#211706' },
              { offset: 1, color: '#140e04' },
            ]
          : r.tone === 'danger'
            ? [
                { offset: 0, color: '#2e1119' },
                { offset: 1, color: '#14080c' },
              ]
            : [
                { offset: 0, color: '#161f2f' },
                { offset: 1, color: '#0a101a' },
              ],
      stroke: accent,
      strokeW: r.tone === 'dark' ? 1.2 : 1.8,
    });
    c.rect(x + 4, r.y + 2, w - 8, 20, 12, {
      fillGrad: [
        { offset: 0, color: '#ffffff', opacity: r.tone === 'gold' ? 0.1 : 0.06 },
        { offset: 1, color: '#ffffff', opacity: 0 },
      ],
      stroke: null,
    });
    c.circle(x + 36, r.y + h / 2, 17, { fill: '#080d15', stroke: accent, strokeW: 1.2 });
    if (r.icon === 'play') playIcon(c, x + 36, r.y + h / 2, 12, C.gold);
    if (r.icon === 'save') saveIcon(c, x + 36, r.y + h / 2, 15, '#a9bdd8');
    if (r.icon === 'gear') gearIcon(c, x + 36, r.y + h / 2, 7.5, '#a9bdd8', '#080d15');
    if (r.icon === 'exit') exitIcon(c, x + 36, r.y + h / 2, 9, '#ff8a94');
    c.text(x + 68, r.y + 8, 360, 24, r.label, {
      size: 13.5,
      tracking: 130,
      color: labelColor,
      justify: 'left',
    });
    c.text(x + 68, r.y + 29, 430, 20, r.sub, {
      size: 9.5,
      style: 'Medium',
      tracking: 30,
      color: r.tone === 'danger' ? '#b46772' : C.muted,
      justify: 'left',
    });
    if (r.note) {
      c.rect(x + w - 66, r.y + 20, 32, 18, 6, { fill: '#080d15', stroke: '#6b4306', strokeW: 1 });
      c.text(x + w - 66, r.y + 20, 32, 18, r.note, {
        size: 8,
        tracking: 80,
        color: '#d9b45a',
        justify: 'center',
      });
    } else {
      chevron(c, x + w - 30, r.y + h / 2, 5.5, r.tone === 'danger' ? '#b46772' : '#7f93b0', 2);
    }
  }

  // footer status card
  c.rect(60, 434, 640, 62, 14, {
    fill: '#090f19',
    stroke: '#22314a',
    strokeW: 1,
  });
  c.circle(82, 465, 5, { fill: C.green, stroke: null, glow: { opacity: 30, blur: 5, color: C.green } });
  c.text(96, 448, 300, 18, 'PARTIDA EN CURSO', { size: 8, tracking: 180, color: C.green, justify: 'left' });
  c.text(96, 464, 330, 22, 'ACTO 1 · PISO 04 · MAZO 12 CARTAS', {
    size: 9.5,
    style: 'Medium',
    tracking: 50,
    color: '#b7c7dd',
    justify: 'left',
  });
  c.text(560, 452, 120, 18, '12:34', {
    size: 12,
    style: 'Medium',
    tracking: 40,
    color: '#dbe6f5',
    justify: 'right',
  });
  c.text(560, 470, 120, 14, 'TIEMPO DE RUN', {
    size: 7,
    style: 'Medium',
    tracking: 110,
    color: '#5f7392',
    justify: 'right',
  });

  c.text(60, 540, 420, 20, 'PULSA ESC PARA VOLVER AL COMBATE', {
    size: 8,
    style: 'Medium',
    tracking: 150,
    color: '#5f7392',
    justify: 'left',
  });
  c.text(560, 540, 140, 20, 'v0.1.0', {
    size: 8,
    style: 'Medium',
    tracking: 100,
    color: '#5f7392',
    justify: 'right',
  });
}

function slider(c, x, y, w, pct, color, label, value) {
  c.text(x, y - 22, 180, 18, label, {
    size: 10,
    tracking: 150,
    color: '#b7c7dd',
    justify: 'left',
  });
  c.text(x + w - 60, y - 22, 60, 18, value, {
    size: 10,
    tracking: 80,
    color,
    justify: 'right',
  });
  c.rect(x, y, w, 7, 3.5, { fill: '#070b12', stroke: '#2a3a55', strokeW: 1 });
  const fillW = Math.max(6, (w - 2) * pct);
  c.rect(x + 1, y + 1, fillW, 5, 2.5, {
    fillGrad: [
      { offset: 0, color: '#ffffff', opacity: 0.35 },
      { offset: 0.4, color, opacity: 1 },
      { offset: 1, color, opacity: 0.65 },
    ],
    stroke: null,
  });
  const kx = x + 1 + fillW;
  c.circle(kx, y + 3.5, 9, {
    fillGrad: [
      { offset: 0, color: '#ffffff' },
      { offset: 1, color: '#cfdaea' },
    ],
    stroke: color,
    strokeW: 2,
    shadow: { opacity: 50, y: 2, blur: 4 },
  });
}

function toggle(c, x, y, on, color) {
  c.rect(x, y, 54, 28, 14, {
    fillGrad: on
      ? [
          { offset: 0, color: '#5a3c08' },
          { offset: 1, color: '#2a1b04' },
        ]
      : [
          { offset: 0, color: '#1b2638' },
          { offset: 1, color: '#0a111c' },
        ],
    stroke: on ? color : '#3b5374',
    strokeW: 1.6,
  });
  const kx = on ? x + 40 : x + 14;
  c.circle(kx, y + 14, 9, {
    fillGrad: on
      ? [
          { offset: 0, color: '#fff3cd' },
          { offset: 1, color: color },
        ]
      : [
          { offset: 0, color: '#b7c7dd' },
          { offset: 1, color: '#6c7f9c' },
        ],
    stroke: '#05080d',
    strokeW: 1.4,
    shadow: { opacity: 45, y: 2, blur: 4 },
  });
}

function drawSettingsMenu(c) {
  c.panel(30, 30, 700, 560, 30, { accent: '#3b5374', borderW: 2.4, shadow: { opacity: 68, y: 16, blur: 30 } });
  c.rect(76, 30.8, 608, 2, 1, {
    fillGrad: [
      { offset: 0, color: C.gold, opacity: 0 },
      { offset: 0.5, color: C.gold, opacity: 0.8 },
      { offset: 1, color: C.gold, opacity: 0 },
    ],
    stroke: null,
  });

  c.circle(78, 76, 20, { fill: '#0b1320', stroke: '#3b5374', strokeW: 1.5 });
  gearIcon(c, 78, 76, 8.5, C.gold, '#0b1320');
  c.text(112, 56, 380, 18, 'AJUSTES', { size: 9.5, tracking: 220, color: C.gold, justify: 'left' });
  c.text(110, 74, 480, 42, 'OPCIONES', {
    size: 26,
    font: 'Noto Sans Black',
    style: 'Regular',
    tracking: 0,
    color: C.text,
    justify: 'left',
  });
  c.rect(654, 52, 44, 44, 12, {
    fillGrad: [
      { offset: 0, color: '#1a2436' },
      { offset: 1, color: '#0a111c' },
    ],
    stroke: '#3b5374',
    strokeW: 1.4,
  });
  closeIcon(c, 676, 74, 7, '#c4d2e6');

  // tabs
  const tabs = ['AUDIO', 'VÍDEO', 'CONTROLES'];
  tabs.forEach((t, i) => {
    const x = 60 + i * 202;
    const active = i === 0;
    c.rect(x, 128, 190, 34, 10, {
      fillGrad: active
        ? [
            { offset: 0, color: '#3a2a0e' },
            { offset: 1, color: '#1a1204' },
          ]
        : [
            { offset: 0, color: '#141d2c' },
            { offset: 1, color: '#090f19' },
          ],
      stroke: active ? C.gold : '#2a3a55',
      strokeW: active ? 1.6 : 1,
    });
    c.text(x, 128, 190, 34, t, {
      size: 9.5,
      tracking: 170,
      color: active ? '#ffe6a8' : C.muted,
      justify: 'center',
    });
  });

  c.text(60, 182, 220, 18, 'AUDIO', { size: 8, tracking: 220, color: '#5f7392', justify: 'left' });
  slider(c, 60, 222, 640, 0.7, C.gold, 'MÚSICA', '70%');
  slider(c, 60, 272, 640, 0.85, C.blue, 'EFECTOS', '85%');
  slider(c, 60, 322, 640, 0.4, C.green, 'VOZ', '40%');

  c.rect(60, 356, 640, 1, 0, {
    fillGrad: [
      { offset: 0, color: '#3b5374', opacity: 0 },
      { offset: 0.5, color: '#3b5374', opacity: 0.55 },
      { offset: 1, color: '#3b5374', opacity: 0 },
    ],
    stroke: null,
  });

  c.text(60, 372, 220, 18, 'INTERFAZ', { size: 8, tracking: 220, color: '#5f7392', justify: 'left' });

  const toggles = [
    { y: 388, label: 'PANTALLA COMPLETA', sub: 'Usar toda la pantalla', on: true, color: C.gold },
    { y: 426, label: 'ANIMACIONES RÁPIDAS', sub: 'Combate a x1.5 de velocidad', on: false, color: C.blue },
    { y: 464, label: 'TEXTO GRANDE', sub: 'Mejor legibilidad en cartas', on: false, color: C.green },
  ];
  // panel grows a little: the toggles end above the footer buttons
  for (const t of toggles) {
    c.text(60, t.y, 420, 20, t.label, {
      size: 10,
      tracking: 130,
      color: '#dbe6f5',
      justify: 'left',
    });
    c.text(60, t.y + 17, 420, 17, t.sub, {
      size: 8.5,
      style: 'Medium',
      tracking: 20,
      color: '#7286a3',
      justify: 'left',
    });
    toggle(c, 646, t.y, t.on, t.color);
    c.text(626, t.y, 14, 28, t.on ? 'ON' : 'OFF', {
      size: 6.5,
      tracking: 60,
      color: t.on ? t.color : '#5f7392',
      justify: 'right',
    });
  }

  // footer buttons
  c.rect(60, 512, 280, 46, 23, {
    fillGrad: [
      { offset: 0, color: '#1a2436' },
      { offset: 1, color: '#090f19' },
    ],
    stroke: '#3b5374',
    strokeW: 1.4,
  });
  c.text(60, 512, 280, 46, 'RESTABLECER', {
    size: 10,
    tracking: 150,
    color: '#a9bdd8',
    justify: 'center',
  });
  c.rect(360, 512, 280, 46, 23, {
    fillGrad: [
      { offset: 0, color: '#ffe9a6' },
      { offset: 0.4, color: '#f2c14e' },
      { offset: 1, color: '#b7821f' },
    ],
    stroke: '#6b4306',
    strokeW: 1.8,
    glow: { opacity: 18, blur: 12, color: C.gold },
  });
  c.text(360, 512, 280, 46, 'GUARDAR CAMBIOS', {
    size: 10,
    tracking: 140,
    color: '#241503',
    justify: 'center',
  });

  c.text(60, 572, 420, 16, 'LOS CAMBIOS SE APLICAN AL INSTANTE', {
    size: 7,
    style: 'Medium',
    tracking: 130,
    color: '#5f7392',
    justify: 'left',
  });
  c.text(560, 572, 140, 16, 'v0.1.0', {
    size: 7,
    style: 'Medium',
    tracking: 100,
    color: '#5f7392',
    justify: 'right',
  });
}

const doc = new Doc('menus');
doc.board('menu_main', 1280, 720, drawMainMenu);
doc.board('menu_pause', 760, 620, drawPauseMenu);
doc.board('menu_settings', 760, 620, drawSettingsMenu);
doc.build(OUT);
console.log('built', OUT);
