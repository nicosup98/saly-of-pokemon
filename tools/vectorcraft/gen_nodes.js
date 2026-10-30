const path = require('path');
const { Doc, C, Ctx } = require('./ui_lib');

const OUT = path.resolve(__dirname, '../../assets/ui/nodes');

const NODES = [
  {
    name: 'node_battle',
    accent: C.red,
    tag: 'COMBATE',
    title: 'ENTRENADOR',
    lines: ['Pokémon salvaje', 'Recompensa: ₽ + EXP'],
    action: 'LUCHAR',
    icon: 'swords',
  },
  {
    name: 'node_elite',
    accent: C.orange,
    tag: 'MINI JEFE',
    title: 'POKÉMON ÉLITE',
    lines: ['Stats aumentados', 'Reliquia garantizada'],
    action: 'DESAFIAR',
    icon: 'flame',
  },
  {
    name: 'node_shop',
    accent: C.blue,
    tag: 'TIENDA',
    title: 'POKÉ MART',
    lines: ['Cartas, reliquias', 'y mejoras de mazo'],
    action: 'COMPRAR',
    icon: 'bag',
  },
  {
    name: 'node_center',
    accent: C.green,
    tag: 'DESCANSO',
    title: 'CENTRO POKÉMON',
    lines: ['Cura 35% de la vida', 'Elige una mejora'],
    action: 'DESCANSAR',
    icon: 'cross',
  },
  {
    name: 'node_treasure',
    accent: C.gold,
    tag: 'TESORO',
    title: 'CÁPSULA',
    lines: ['Reliquia aleatoria', 'Botín del acto'],
    action: 'ABRIR',
    icon: 'chest',
  },
  {
    name: 'node_gym',
    accent: C.purple,
    tag: 'GIMNASIO',
    title: 'LÍDER',
    lines: ['Combate de jefe', 'Medalla del acto'],
    action: 'RETAR',
    icon: 'crown',
  },
];

function drawIcon(c, cx, cy, accent, kind) {
  if (kind === 'swords') {
    c.sword(cx, cy, 66, { rotate: 44, strokeW: 2.5 });
    c.sword(cx, cy, 66, { rotate: -44, strokeW: 2.5 });
    c.circle(cx, cy, 8, { fill: accent, stroke: '#1a0709', strokeW: 2 });
    c.circle(cx, cy, 3.2, { fill: '#ffe3e6', stroke: null });
  } else if (kind === 'flame') {
    c.flame(cx, cy + 2, 50, 66, {
      fillGrad: [
        { offset: 0, color: '#ffd76b' },
        { offset: 0.5, color: '#ff8a3d' },
        { offset: 1, color: '#d92f24' },
      ],
      innerColor: '#fff1bd',
    });
    for (const [sx, sy, sr] of [
      [cx - 35, cy - 25, 2.6],
      [cx + 35, cy - 17, 3.6],
      [cx + 29, cy + 31, 2.2],
      [cx - 30, cy + 26, 1.8],
    ]) {
      c.circle(sx, sy, sr, { fill: C.goldLight, stroke: null, glow: { opacity: 35, blur: 5, color: C.orange } });
    }
  } else if (kind === 'bag') {
    c.bag(cx, cy + 3, 64, 58);
    c.circle(cx - 31, cy - 28, 3.2, { fill: '#cfeaff', stroke: null });
    c.circle(cx + 34, cy - 18, 2.2, { fill: '#cfeaff', stroke: null });
  } else if (kind === 'cross') {
    c.cross(cx, cy, 64, 21, { fill: '#f4fff8', stroke: '#0b2a18', radius: 5 });
    c.circle(cx - 35, cy - 35, 2.4, { fill: accent, stroke: null });
    c.circle(cx + 35, cy - 35, 2.4, { fill: accent, stroke: null });
    c.circle(cx - 35, cy + 35, 2.4, { fill: accent, stroke: null });
    c.circle(cx + 35, cy + 35, 2.4, { fill: accent, stroke: null });
  } else if (kind === 'chest') {
    c.chest(cx, cy + 2, 74, 58, { shadow: { opacity: 40, y: 6, blur: 10 } });
    for (const [sx, sy, r1, r2] of [
      [cx - 39, cy - 31, 8, 2.2],
      [cx + 39, cy - 27, 5.5, 1.6],
      [cx + 33, cy + 34, 4, 1.2],
    ]) {
      c.star(sx, sy, r1, r2, 4, { fill: '#ffe9a8', stroke: null, rotation: 45 });
    }
  } else if (kind === 'crown') {
    c.crown(cx, cy - 30, 72, 56, {
      fillGrad: [
        { offset: 0, color: '#f3d2ff' },
        { offset: 0.45, color: '#b06bff' },
        { offset: 1, color: '#5c1f96' },
      ],
      stroke: '#2a0b45',
      jewel: '#ffd76b',
      jewel2: '#ff8ac2',
      tip: '#f4e4ff',
    });
    c.circle(cx - 37, cy - 33, 3, { fill: '#f4e4ff', stroke: null });
    c.circle(cx + 37, cy - 33, 3, { fill: '#f4e4ff', stroke: null });
  }
}

function drawCard(c, n, x, y, w, h) {
  const accent = n.accent;
  // atmosphere: large faint rings in the card corners
  c.circle(x + 206, y + 52, 66, { fill: null, stroke: accent, strokeW: 1.2, opacity: 9 });
  c.circle(x + 206, y + 52, 48, { fill: null, stroke: accent, strokeW: 1, opacity: 7 });
  c.circle(x + 34, y + 258, 54, { fill: null, stroke: accent, strokeW: 1, opacity: 6 });

  c.panel(x, y, w, h, 24, { accent, borderW: 2.5, shadow: { opacity: 62, y: 10, blur: 20 } });

  // accent strip on the left edge
  c.rect(x + 2, y + 22, 3, h - 48, 1.5, { fill: accent, opacity: 38 });
  // glowing top edge
  c.rect(x + 14, y + 1.2, w - 28, 2, 1, {
    fillGrad: [
      { offset: 0, color: accent, opacity: 0 },
      { offset: 0.5, color: accent, opacity: 0.9 },
      { offset: 1, color: accent, opacity: 0 },
    ],
    stroke: null,
  });

  // type pill
  c.rect(x + 20, y + 16, w - 40, 25, 12.5, {
    fillGrad: [
      { offset: 0, color: '#111a28' },
      { offset: 1, color: '#080d15' },
    ],
    stroke: accent,
    strokeW: 1.4,
  });
  c.circle(x + 33, y + 28.5, 3.1, { fill: accent, stroke: null });
  c.text(x + 44, y + 17, w - 70, 23, n.tag, {
    size: 9.5,
    tracking: 145,
    color: accent,
    justify: 'center',
  });

  // medallion
  const cx = x + w / 2;
  const cy = y + 116;
  c.circle(cx, cy, 62, { fill: accent, opacity: 10, glow: { opacity: 22, blur: 18, color: accent } });
  c.circle(cx, cy, 55, {
    fillGrad: [
      { offset: 0, color: '#26354c' },
      { offset: 1, color: '#080d16' },
    ],
    stroke: accent,
    strokeW: 2.5,
    shadow: { opacity: 60, y: 6, blur: 12 },
  });
  c.circle(cx, cy, 47, {
    fillGrad: [
      { offset: 0, color: accent, opacity: 0.2 },
      { offset: 0.6, color: '#0b1220', opacity: 0.1 },
      { offset: 1, color: '#070b12', opacity: 0 },
    ],
    stroke: accent,
    strokeW: 1,
  });
  c.circle(cx, cy, 42, { fill: null, stroke: accent, strokeW: 0.8, opacity: 22 });
  // rivets
  for (const a of [45, 135, 225, 315]) {
    const rad = (a * Math.PI) / 180;
    c.circle(cx + Math.cos(rad) * 55, cy + Math.sin(rad) * 55, 2.4, {
      fill: accent,
      stroke: '#05080d',
      strokeW: 1,
    });
  }
  drawIcon(c, cx, cy, accent, n.icon);

  // title
  c.text(x + 16, y + 180, w - 32, 30, n.title, {
    size: 18,
    tracking: 35,
    color: C.text,
    justify: 'center',
  });
  // subtitle, two lines
  n.lines.forEach((line, i) => {
    c.text(x + 16, y + 212 + i * 16, w - 32, 15, line, {
      size: 10.5,
      style: 'Medium',
      tracking: 10,
      color: i === 1 && n.icon === 'chest' ? C.gold : C.muted,
      justify: 'center',
    });
  });

  // divider
  c.rect(x + 26, y + 252, w - 52, 1, 0, {
    fillGrad: [
      { offset: 0, color: accent, opacity: 0 },
      { offset: 0.5, color: accent, opacity: 0.45 },
      { offset: 1, color: accent, opacity: 0 },
    ],
    stroke: null,
  });

  // action pill
  c.rect(x + 20, y + 268, w - 40, 32, 16, {
    fillGrad: [
      { offset: 0, color: '#0e1724' },
      { offset: 1, color: '#060a11' },
    ],
    stroke: accent,
    strokeW: 1.6,
    shadow: { opacity: 45, y: 3, blur: 6 },
  });
  c.rect(x + 22, y + 269.5, w - 44, 10, 5, {
    fillGrad: [
      { offset: 0, color: '#ffffff', opacity: 0.09 },
      { offset: 1, color: '#ffffff', opacity: 0 },
    ],
    stroke: null,
  });
  c.text(x + 24, y + 269, w - 48, 30, n.action, {
    size: 11,
    tracking: 155,
    color: accent,
    justify: 'center',
  });
}

const doc = new Doc('nodes');
doc.board('node_battle', 1660, 400, (c) => {
  NODES.forEach((n, i) => drawCard(c, n, 20 + i * 272, 40, 240, 320));
});
// rename the first artboard: the builder uses the first board name, so all cards
// live on one artboard for a contact-sheet export; individual pieces are cut
// from the same master below as separate boards.
doc.boards = [];
NODES.forEach((n) => {
  doc.board(n.name, 280, 360, (c) => {
    drawCard(c, n, 20, 20, 240, 320);
  });
});
doc.board('map_path', 360, 120, (c) => {
  const d = 'M 22 92 C 90 92 110 28 180 30 C 250 32 270 88 338 54';
  // soft under-glow
  c.path(d, { closed: false, fill: null, stroke: '#4aa8ff', strokeW: 11, opacity: 14 });
  // dark rail
  c.path(d, { closed: false, fill: null, stroke: '#0a111c', strokeW: 7 });
  // dashed energy line
  c.path(d, {
    closed: false,
    fill: null,
    stroke: '#4aa8ff',
    strokeW: 3.2,
    strokeOpts: { dash: [1.5, 10], cap: 'round' },
    glow: { opacity: 55, blur: 8, color: '#4aa8ff' },
  });
  // start node (cleared)
  c.circle(22, 92, 13, { fill: '#0b111c', stroke: '#4aa8ff', strokeW: 2.5 });
  c.circle(22, 92, 5.5, { fill: '#4aa8ff', stroke: null });
  // end node (available)
  c.circle(338, 54, 17, { fill: '#0b111c', stroke: '#4aa8ff', strokeW: 3, glow: { opacity: 45, blur: 10, color: '#4aa8ff' } });
  c.circle(338, 54, 7, { fill: '#cfeaff', stroke: null });
});

doc.build(OUT);
console.log('built', OUT);
