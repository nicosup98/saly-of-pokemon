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
  c.circle(38, 49, 17, { fill: '#6d1d06', stroke: '#ffd7a6', strokeW: 1.4 });
  hourglass(c, 38, 49, 14, '#ffe9c9');
  c.path('M 268 43 L 275 49 L 268 55', {
    closed: false,
    fill: null,
    stroke: '#ffd7a6',
    strokeW: 2.2,
    strokeOpts: { cap: 'round', join: 'round' },
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
  c.sword(48, 96, 62, {
    rotate: 45,
    strokeW: 2.0,
    fillGrad: [
      { offset: 0, color: '#ffe3e6' },
      { offset: 0.5, color: '#d9a4ab' },
      { offset: 1, color: '#8c4a54' },
    ],
    stroke: '#3a1118',
    opacity: 28,
  });
  c.rect(30, 144, 180, 1, 0, {
    fillGrad: [
      { offset: 0, color: '#7a2633', opacity: 0 },
      { offset: 0.5, color: '#7a2633', opacity: 0.8 },
      { offset: 1, color: '#7a2633', opacity: 0 },
    ],
    stroke: null,
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
