const path = require('path');
const { Doc, C } = require('./ui_lib');

const OUT = path.resolve(__dirname, '../../assets/ui/hud');

function gearIcon(c, cx, cy, size, color, hole) {
  for (let i = 0; i < 8; i++) {
    c.rect(cx - size * 0.13, cy - size * 1.08, size * 0.26, size * 0.36, size * 0.05, { fill: color, stroke: null });
    c.cmd('object.rotate', { angle: (i * 360) / 8, origin: [c.ox + cx, c.oy + cy] });
  }
  c.circle(cx, cy, size * 0.72, { fill: color, stroke: null });
  c.circle(cx, cy, size * 0.32, { fill: hole, stroke: null });
}

function pauseIcon(c, cx, cy, size, color) {
  c.rect(cx - size * 0.38, cy - size * 0.42, size * 0.26, size * 0.84, 2, { fill: color, stroke: null });
  c.rect(cx + size * 0.12, cy - size * 0.42, size * 0.26, size * 0.84, 2, { fill: color, stroke: null });
}

function coin(c, cx, cy, r, value = '₽') {
  c.circle(cx, cy, r, {
    fillGrad: [
      { offset: 0, color: '#ffe9a6' },
      { offset: 0.55, color: '#f2c14e' },
      { offset: 1, color: '#b7821f' },
    ],
    stroke: '#6b4306',
    strokeW: 1.6,
    shadow: { opacity: 45, y: 2, blur: 4 },
  });
  c.circle(cx, cy, r * 0.7, { fill: null, stroke: '#7a4c08', strokeW: 1 });
  c.text(cx - r, cy - r * 0.72, r * 2, r * 1.44, value, {
    size: r * 1.15,
    style: 'SemiBold',
    color: '#2a1a04',
    justify: 'center',
  });
}

function cardIcon(c, cx, cy, s, color) {
  c.rect(cx - s * 0.42, cy - s * 0.55, s * 0.66, s * 0.94, s * 0.08, {
    fill: '#0a111c',
    stroke: color,
    strokeW: 1.8,
    rotation: -10,
  });
  c.rect(cx - s * 0.22, cy - s * 0.5, s * 0.66, s * 0.94, s * 0.08, {
    fill: '#0a111c',
    stroke: color,
    strokeW: 1.8,
    rotation: 6,
  });
  c.circle(cx + s * 0.1, cy, s * 0.1, { fill: color, stroke: null });
}

function hourglass(c, cx, cy, s, color) {
  c.path(`M ${cx - s * 0.5} ${cy - s * 0.6} L ${cx + s * 0.5} ${cy - s * 0.6} L ${cx + s * 0.06} ${cy} L ${cx + s * 0.5} ${cy + s * 0.6} L ${cx - s * 0.5} ${cy + s * 0.6} L ${cx - s * 0.06} ${cy} Z`, {
    fill: null,
    stroke: color,
    strokeW: 2.2,
    strokeOpts: { join: 'round' },
  });
  c.line(cx - s * 0.5, cy - s * 0.6, cx + s * 0.5, cy - s * 0.6, color, 2.2, { cap: 'round' });
  c.line(cx - s * 0.5, cy + s * 0.6, cx + s * 0.5, cy + s * 0.6, color, 2.2, { cap: 'round' });
}

function drawTopBar(c) {
  c.rect(20, 8, 1560, 80, 18, {
    fill: '#000000',
    stroke: null,
    shadow: { opacity: 55, y: 8, blur: 18 },
  });
  c.rect(20, 8, 1560, 80, 18, {
    fillGrad: [
      { offset: 0, color: '#162235' },
      { offset: 0.55, color: '#0e1725' },
      { offset: 1, color: '#080e18' },
    ],
    stroke: '#2a3a55',
    strokeW: 1.6,
  });
  c.rect(26, 12, 1548, 24, 12, {
    fillGrad: [
      { offset: 0, color: '#ffffff', opacity: 0.1 },
      { offset: 1, color: '#ffffff', opacity: 0 },
    ],
    stroke: null,
  });
  c.rect(46, 84, 1508, 1, 0, {
    fillGrad: [
      { offset: 0, color: C.gold, opacity: 0 },
      { offset: 0.14, color: C.gold, opacity: 0.45 },
      { offset: 0.5, color: C.blue, opacity: 0.4 },
      { offset: 0.86, color: C.gold, opacity: 0.45 },
      { offset: 1, color: C.gold, opacity: 0 },
    ],
    stroke: null,
  });

  // act badge
  c.rect(40, 22, 110, 52, 12, {
    fillGrad: [
      { offset: 0, color: '#35260b' },
      { offset: 1, color: '#171003' },
    ],
    stroke: C.gold,
    strokeW: 1.4,
  });
  c.text(40, 26, 110, 13, 'ACTO', { size: 6.5, tracking: 220, color: '#d9b45a', justify: 'center' });
  c.text(40, 38, 110, 30, '1', {
    size: 21,
    font: 'Noto Sans Black',
    style: 'Regular',
    color: '#ffe6a8',
    justify: 'center',
  });

  // floor icon
  c.circle(196, 48, 22, { fill: '#0a111c', stroke: '#2a3a55', strokeW: 1.2 });
  c.line(188, 55, 196, 43, '#4aa8ff', 1.6, { cap: 'round' });
  c.line(196, 43, 205, 52, '#4aa8ff', 1.6, { cap: 'round' });
  c.circle(188, 55, 2.6, { fill: '#4aa8ff', stroke: null });
  c.circle(196, 43, 2.6, { fill: '#cfeaff', stroke: null });
  c.circle(205, 52, 2.6, { fill: '#4aa8ff', stroke: null });
  c.text(228, 25, 120, 16, 'PISO', { size: 7, tracking: 200, color: '#7f93b0', justify: 'left' });
  c.text(228, 39, 140, 24, '04 / 15', {
    size: 16,
    style: 'SemiBold',
    tracking: 40,
    color: C.text,
    justify: 'left',
  });

  c.rect(352, 24, 1, 48, 0, { fill: '#2a3a55', opacity: 45, stroke: null });

  // route title
  c.text(390, 24, 380, 16, 'RUTA DEL ACTO', { size: 7, tracking: 230, color: '#7f93b0', justify: 'left' });
  c.text(390, 38, 420, 26, 'BOSQUE VERDE', {
    size: 15,
    style: 'SemiBold',
    tracking: 90,
    color: C.text,
    justify: 'left',
  });
  // route breadcrumbs
  for (let i = 0; i < 4; i++) {
    const bx = 640 + i * 22;
    c.circle(bx, 52, i === 2 ? 4.2 : 2.8, {
      fill: i <= 2 ? C.gold : '#3b4d6b',
      stroke: null,
      glow: i === 2 ? { opacity: 30, blur: 5, color: C.gold } : null,
    });
    if (i < 3) c.line(bx + 5, 52, bx + 17, 52, i < 2 ? C.gold : '#3b4d6b', 1.4, {});
  }

  // currency
  coin(c, 1118, 48, 18);
  c.text(1146, 27, 100, 26, '248', {
    size: 17,
    style: 'SemiBold',
    tracking: 20,
    color: '#ffe6a8',
    justify: 'left',
  });
  c.text(1146, 52, 100, 14, 'FICHAS', { size: 6.5, tracking: 180, color: '#7f93b0', justify: 'left' });

  // deck
  cardIcon(c, 1262, 48, 24, '#4aa8ff');
  c.text(1288, 27, 90, 26, '12', {
    size: 17,
    style: 'SemiBold',
    tracking: 20,
    color: C.text,
    justify: 'left',
  });
  c.text(1288, 52, 90, 14, 'MAZO', { size: 6.5, tracking: 180, color: '#7f93b0', justify: 'left' });

  // buttons
  for (const [bx, icon] of [
    [1442, 'gear'],
    [1496, 'pause'],
  ]) {
    c.rect(bx, 25, 46, 46, 12, {
      fillGrad: [
        { offset: 0, color: '#1a2436' },
        { offset: 1, color: '#090f19' },
      ],
      stroke: '#3b5374',
      strokeW: 1.3,
    });
    if (icon === 'gear') gearIcon(c, bx + 23, 48, 8, '#c4d2e6', '#0a111c');
    else pauseIcon(c, bx + 23, 48, 13, '#c4d2e6');
  }
}

function drawEndTurn(c) {
  c.rect(10, 10, 280, 78, 20, {
    fill: '#5a1a0a',
    stroke: null,
    shadow: { opacity: 55, y: 8, blur: 16 },
  });
  c.rect(10, 10, 280, 78, 20, {
    fillGrad: [
      { offset: 0, color: '#ffb347' },
      { offset: 0.42, color: '#ff7a3d' },
      { offset: 0.78, color: '#e2491f' },
      { offset: 1, color: '#a9270f' },
    ],
    stroke: '#7a2208',
    strokeW: 1.8,
    glow: { opacity: 24, blur: 14, color: C.primary },
  });
  c.rect(16, 14, 268, 28, 14, {
    fillGrad: [
      { offset: 0, color: '#ffffff', opacity: 0.35 },
      { offset: 1, color: '#ffffff', opacity: 0 },
    ],
    stroke: null,
  });
  c.circle(44, 49, 18, { fill: '#6d1d06', stroke: '#ffd7a6', strokeW: 1.4 });
  hourglass(c, 44, 49, 15, '#ffe9c9');
  c.text(72, 10, 196, 78, 'TERMINAR TURNO', {
    size: 12.5,
    tracking: 100,
    color: '#fff4e4',
    justify: 'left',
  });
  c.path('M 268 43 L 275 49 L 268 55', {
    closed: false,
    fill: null,
    stroke: '#ffd7a6',
    strokeW: 2.2,
    strokeOpts: { cap: 'round', join: 'round' },
  });

  // key hint
  c.rect(120, 92, 60, 17, 5, { fill: '#070b12', stroke: '#2a3a55', strokeW: 1 });
  c.text(120, 92, 60, 17, 'ESPACIO', {
    size: 6.5,
    tracking: 80,
    color: '#7f93b0',
    justify: 'center',
  });
}

function cardBack(c, x, y, w, h, opts = {}) {
  c.rect(x, y, w, h, 10, {
    fillGrad: opts.dim
      ? [
          { offset: 0, color: '#161c28' },
          { offset: 1, color: '#0a0e16' },
        ]
      : [
          { offset: 0, color: '#1a3a5f' },
          { offset: 0.55, color: '#102742' },
          { offset: 1, color: '#08152a' },
        ],
    stroke: opts.stroke || '#4aa8ff',
    strokeW: 2,
    shadow: { opacity: 55, y: 7, blur: 12 },
  });
  c.rect(x + 6, y + 6, w - 12, h - 12, 6, {
    fill: null,
    stroke: opts.stroke || '#4aa8ff',
    strokeW: 0.8,
    opacity: 35,
  });
  if (opts.discard) {
    c.line(x + w / 2, y + h * 0.34, x + w / 2, y + h * 0.62, '#ffb3ba', 4, { cap: 'round' });
    c.path(`M ${x + w / 2 - 11} ${y + h * 0.52} L ${x + w / 2} ${y + h * 0.63} L ${x + w / 2 + 11} ${y + h * 0.52}`, {
      closed: false,
      fill: null,
      stroke: '#ffb3ba',
      strokeW: 4,
      strokeOpts: { cap: 'round', join: 'round' },
    });
    c.line(x + w * 0.28, y + h * 0.7, x + w * 0.72, y + h * 0.7, '#ffb3ba', 3, { cap: 'round' });
  } else {
    c.star(x + w / 2, y + h / 2, 26, 9, 4, {
      fillGrad: [
        { offset: 0, color: '#ffe9a6' },
        { offset: 1, color: '#c88716' },
      ],
      stroke: '#6b4306',
      strokeW: 1.5,
      rotation: 45,
    });
    c.circle(x + w / 2, y + h / 2, 5, { fill: '#0b1a2c', stroke: '#ffe9a6', strokeW: 1.5 });
  }
  // corner diamonds
  for (const [dx, dy] of [
    [x + 15, y + 15],
    [x + w - 15, y + 15],
    [x + 15, y + h - 15],
    [x + w - 15, y + h - 15],
  ]) {
    c.rect(dx - 2.2, dy - 2.2, 4.4, 4.4, 0.8, {
      fill: opts.stroke || '#4aa8ff',
      stroke: null,
      rotation: 45,
      opacity: 80,
    });
  }
}

function drawDeck(c, discard) {
  const accent = discard ? '#f0525c' : '#4aa8ff';
  // stack
  c.rect(42, 32, 112, 148, 10, {
    fill: '#0a0f18',
    stroke: '#26354c',
    strokeW: 1.2,
    rotation: discard ? 9 : -9,
  });
  c.rect(44, 26, 112, 148, 10, {
    fill: '#0d1522',
    stroke: '#2a3a55',
    strokeW: 1.2,
    rotation: discard ? 4 : -4,
  });
  cardBack(c, 44, 20, 112, 148, { stroke: accent, dim: discard, discard });
  c.circle(154, 158, 24, {
    fillGrad: discard
      ? [
          { offset: 0, color: '#5a1820' },
          { offset: 1, color: '#2a0b10' },
        ]
      : [
          { offset: 0, color: '#ffe9a6' },
          { offset: 1, color: '#c88716' },
        ],
    stroke: discard ? '#f0525c' : '#6b4306',
    strokeW: 2,
    shadow: { opacity: 50, y: 3, blur: 6 },
  });
  c.text(130, 134, 48, 48, discard ? '4' : '12', {
    size: 19,
    font: 'Noto Sans Black',
    style: 'Regular',
    color: discard ? '#ff9da7' : '#2a1a04',
    justify: 'center',
  });
  c.text(20, 190, 160, 20, discard ? 'DESCARTE' : 'MAZO', {
    size: 9,
    tracking: 220,
    color: discard ? '#b46772' : C.muted,
    justify: 'center',
  });
  c.text(20, 204, 160, 14, discard ? 'PILA VACÍA' : 'ROBAR CARTA', {
    size: 6.5,
    tracking: 140,
    color: discard ? '#7a4650' : '#5f7392',
    justify: 'center',
  });
}

function drawIntent(c) {
  c.rect(10, 12, 220, 176, 20, {
    fill: '#000000',
    stroke: null,
    shadow: { opacity: 55, y: 8, blur: 16 },
  });
  c.rect(10, 12, 220, 176, 20, {
    fillGrad: [
      { offset: 0, color: '#2a1219' },
      { offset: 0.6, color: '#160a0f' },
      { offset: 1, color: '#0b060a' },
    ],
    stroke: '#7a2633',
    strokeW: 2,
  });
  c.rect(16, 16, 208, 30, 14, {
    fillGrad: [
      { offset: 0, color: '#ffffff', opacity: 0.08 },
      { offset: 1, color: '#ffffff', opacity: 0 },
    ],
    stroke: null,
  });
  c.text(10, 24, 220, 16, 'INTENCIÓN', {
    size: 7.5,
    tracking: 240,
    color: '#ff8a94',
    justify: 'center',
  });
  c.sword(68, 100, 86, {
    rotate: 45,
    strokeW: 2.4,
    fillGrad: [
      { offset: 0, color: '#ffe3e6' },
      { offset: 0.5, color: '#d9a4ab' },
      { offset: 1, color: '#8c4a54' },
    ],
    stroke: '#3a1118',
  });
  // impact spark
  c.star(112, 66, 12, 3.5, 4, {
    fill: '#ffd7a6',
    stroke: null,
    rotation: 45,
    glow: { opacity: 35, blur: 6, color: C.orange },
  });
  c.text(112, 54, 106, 62, '12', {
    size: 52,
    font: 'Noto Sans Black',
    style: 'Regular',
    tracking: -20,
    color: '#ffe9ec',
    justify: 'right',
  });
  c.text(112, 112, 106, 16, 'DAÑO', {
    size: 7.5,
    tracking: 220,
    color: '#ff8a94',
    justify: 'right',
  });
  c.rect(30, 144, 180, 1, 0, {
    fillGrad: [
      { offset: 0, color: '#7a2633', opacity: 0 },
      { offset: 0.5, color: '#7a2633', opacity: 0.8 },
      { offset: 1, color: '#7a2633', opacity: 0 },
    ],
    stroke: null,
  });
  c.text(10, 152, 220, 18, 'ATACARÁ ESTE TURNO', {
    size: 7,
    tracking: 160,
    color: '#b46772',
    justify: 'center',
  });
}

function drawTurnBanner(c) {
  // ribbon tails
  c.path('M 80 36 L 44 36 L 58 60 L 44 84 L 80 84 Z', {
    fillGrad: [
      { offset: 0, color: '#3a2a0e' },
      { offset: 1, color: '#1a1204' },
    ],
    stroke: '#6b4306',
    strokeW: 1.4,
  });
  c.path('M 480 36 L 516 36 L 502 60 L 516 84 L 480 84 Z', {
    fillGrad: [
      { offset: 0, color: '#3a2a0e' },
      { offset: 1, color: '#1a1204' },
    ],
    stroke: '#6b4306',
    strokeW: 1.4,
  });
  c.rect(80, 24, 400, 72, 12, {
    fillGrad: [
      { offset: 0, color: '#1c2a40' },
      { offset: 0.5, color: '#0e1725' },
      { offset: 1, color: '#080e18' },
    ],
    stroke: C.gold,
    strokeW: 2,
    shadow: { opacity: 60, y: 7, blur: 14 },
  });
  c.rect(88, 28, 384, 26, 10, {
    fillGrad: [
      { offset: 0, color: '#ffffff', opacity: 0.12 },
      { offset: 1, color: '#ffffff', opacity: 0 },
    ],
    stroke: null,
  });
  c.text(80, 30, 400, 15, 'COMIENZA EL COMBATE', {
    size: 6.5,
    tracking: 230,
    color: '#d9b45a',
    justify: 'center',
  });
  c.text(80, 44, 400, 38, 'TURNO 3', {
    size: 28,
    font: 'Noto Sans Black',
    style: 'Regular',
    tracking: 40,
    color: '#ffe6a8',
    justify: 'center',
  });
  c.star(62, 60, 5, 2, 4, { fill: C.gold, stroke: null, rotation: 45 });
  c.star(498, 60, 5, 2, 4, { fill: C.gold, stroke: null, rotation: 45 });
}

const doc = new Doc('hud');
doc.board('hud_top_bar', 1600, 96, drawTopBar);
doc.board('hud_end_turn', 300, 112, drawEndTurn);
doc.board('hud_deck_draw', 200, 220, (c) => drawDeck(c, false));
doc.board('hud_deck_discard', 200, 220, (c) => drawDeck(c, true));
doc.board('hud_intent', 240, 200, drawIntent);
doc.board('hud_turn_banner', 560, 120, drawTurnBanner);
doc.build(OUT);
console.log('built', OUT);
