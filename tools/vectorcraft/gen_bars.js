const path = require('path');
const { Doc, C } = require('./ui_lib');

const doc = new Doc('bars');

const BAR_W = 660;
const BAR_H = 44;

function bolt(ctx, cx, cy, s, fill, stroke = null) {
  const d = `M ${cx + s * 0.08} ${cy - s * 0.62}
    L ${cx - s * 0.36} ${cy + s * 0.06}
    L ${cx - s * 0.04} ${cy + s * 0.06}
    L ${cx - s * 0.14} ${cy + s * 0.62}
    L ${cx + s * 0.36} ${cy - s * 0.08}
    L ${cx + s * 0.04} ${cy - s * 0.08} Z`;
  ctx.path(d, { fill, stroke, strokeW: stroke ? 2 : 0 });
}

function drop(ctx, cx, cy, w, h, fill, stroke = null) {
  const d = `M ${cx} ${cy - h / 2}
    C ${cx + w * 0.55} ${cy - h * 0.08} ${cx + w * 0.5} ${cy + h * 0.34} ${cx} ${cy + h / 2}
    C ${cx - w * 0.5} ${cy + h * 0.34} ${cx - w * 0.55} ${cy - h * 0.08} ${cx} ${cy - h / 2} Z`;
  ctx.path(d, { fill, stroke, strokeW: stroke ? 2 : 0 });
}

function barShape(x, y, w, h, r = 20) {
  const cut = 12;
  return `M ${x + r} ${y}
    L ${x + w - cut - 10} ${y}
    L ${x + w} ${y + cut}
    L ${x + w} ${y + h - cut}
    L ${x + w - cut - 10} ${y + h}
    L ${x + r} ${y + h}
    Q ${x} ${y + h} ${x} ${y + h - r}
    L ${x} ${y + r}
    Q ${x} ${y} ${x + r} ${y} Z`;
}

function hpBar(ctx, opts) {
  const {
    name,
    level,
    hp,
    maxHp,
    pct,
    ghostPct,
    colors,
    accent,
    icon,
    subline,
    valueLine,
    boss = false,
  } = opts;
  const x = 20;
  const y = 112;
  const w = BAR_W;
  const h = BAR_H;

  // icon medallion
  const cx = 50;
  const cy = 60;
  ctx.circle(cx + 3, cy + 4, 27, { fill: '#000000', stroke: null, opacity: 0.4 });
  ctx.circle(cx, cy, 28, {
    fillGrad: [
      { offset: 0, color: colors[0] },
      { offset: 0.55, color: colors[1] },
      { offset: 1, color: colors[2] },
    ],
    stroke: accent,
    strokeW: 2.5,
    shadow: { opacity: 45, y: 4, blur: 8 },
  });
  if (icon === 'bolt') bolt(ctx, cx, cy, 34, '#fff9d8', '#7a4a00');
  else if (icon === 'flame') ctx.flame(cx, cy + 2, 34, 40, { stroke: '#5c130b', strokeW: 2, innerColor: '#ffe9a8' });
  else if (icon === 'crown') ctx.crown(cx, cy - 12, 38, 24, { shadow: null });

  // name and level
  ctx.text(92, 30, 360, 30, name, {
    size: 23,
    style: 'SemiBold',
    color: '#f4f8ff',
    tracking: 55,
    justify: 'left',
    valign: 'center',
  });
  ctx.text(92, 62, 360, 20, level, {
    size: 12,
    style: 'Medium',
    color: C.muted,
    tracking: 110,
    justify: 'left',
    valign: 'center',
  });

  // hp numbers
  ctx.text(400, 28, 260, 34, `${hp} / ${maxHp}`, {
    size: 23,
    style: 'SemiBold',
    color: '#ffffff',
    tracking: 30,
    justify: 'right',
    valign: 'center',
  });
  ctx.text(430, 60, 230, 20, valueLine || 'PUNTOS DE SALUD', {
    size: 11.5,
    style: 'Medium',
    color: C.muted,
    tracking: 120,
    justify: 'right',
    valign: 'center',
  });

  // bar frame
  ctx.path(barShape(x - 4, y - 4, w + 8, h + 8, 22), {
    fillGrad: [
      { offset: 0, color: '#2a3a55' },
      { offset: 0.5, color: '#111a28' },
      { offset: 1, color: '#070b12' },
    ],
    stroke: '#05080d',
    strokeW: 2,
    shadow: { opacity: 60, y: 10, blur: 18 },
  });
  ctx.path(barShape(x, y, w, h, 19), {
    fillGrad: [
      { offset: 0, color: '#070b12' },
      { offset: 1, color: '#111a28' },
    ],
    stroke: accent,
    strokeW: 2.5,
  });

  const innerX = x + 5;
  const innerY = y + 5;
  const innerW = w - 10;
  const innerH = h - 10;

  // current fill
  const fw = (innerW - 6) * pct;
  ctx.path(barShape(innerX, innerY, fw, innerH, 15), {
    fillGrad: [
      { offset: 0, color: colors[0] },
      { offset: 0.35, color: colors[1] },
      { offset: 1, color: colors[2] },
    ],
    stroke: null,
  });

  // glossy top half of the fill
  ctx.path(barShape(innerX + 3, innerY + 3, Math.max(12, fw - 6), innerH * 0.46, 12), {
    fillGrad: [
      { offset: 0, color: '#ffffff', opacity: 0.42 },
      { offset: 1, color: '#ffffff', opacity: 0 },
    ],
    stroke: null,
  });

  // tick marks
  for (let i = 1; i < 10; i++) {
    const tx = innerX + (innerW * i) / 10;
    ctx.rect(tx - 1, innerY + 1, 2, innerH - 2, 1, {
      fillGrad: [
        { offset: 0, color: '#05080d', opacity: 0.1 },
        { offset: 1, color: '#05080d', opacity: 0.5 },
      ],
      stroke: null,
    });
  }

  // bright leading cap
  if (pct > 0.02) {
    const capX = innerX + fw - 5;
    ctx.rect(capX, innerY + 2, 4, innerH - 4, 2, {
      fill: '#ffffff',
      stroke: null,
      opacity: 38,
    });
  }

  // bottom label
  ctx.text(x + 2, y + h + 8, 320, 20, subline, {
    size: 11,
    style: 'Medium',
    color: C.muted,
    tracking: 100,
    justify: 'left',
    valign: 'center',
  });
  ctx.text(x + 340, y + h + 8, 320, 20, boss ? 'OBJETIVO PRIORITARIO' : 'ESTADO: NORMAL', {
    size: 11,
    style: 'SemiBold',
    color: boss ? accent : C.muted,
    tracking: 100,
    justify: 'right',
    valign: 'center',
  });
}

function barBoard(title, opts) {
  return (c) => {
    c.rect(4, 4, 692, 192, 24, {
      fillGrad: [
        { offset: 0, color: '#1b2638' },
        { offset: 0.55, color: '#111927' },
        { offset: 1, color: '#080d15' },
      ],
      stroke: '#2a3a55',
      strokeW: 2,
      shadow: { opacity: 55, y: 10, blur: 18 },
    });
    c.rect(14, 10, 672, 34, 14, {
      fillGrad: [
        { offset: 0, color: '#ffffff', opacity: 0.08 },
        { offset: 1, color: '#ffffff', opacity: 0 },
      ],
      stroke: null,
    });
    c.text(20, 14, 660, 18, title, {
      size: 10,
      style: 'SemiBold',
      color: C.muted,
      tracking: 160,
      justify: 'left',
      valign: 'center',
    });
    hpBar(c, opts);
  };
}

doc
  .board(
    'hp_player',
    700,
    200,
    barBoard('BARRA DE VIDA · JUGADOR', {
      name: 'PIKACHU',
      level: 'Nv. 18  ·  TIPO ELÉCTRICO',
      hp: 84,
      maxHp: 120,
      pct: 0.7,
      ghostPct: 0.745,
      colors: ['#c8ffdc', '#45d47c', '#138a45'],
      accent: '#7dffb0',
      icon: 'bolt',
      subline: 'HP 70%  ·  SIN ESTADOS',
      valueLine: 'PUNTOS DE SALUD',
    })
  )
  .board(
    'hp_enemy',
    700,
    200,
    barBoard('BARRA DE VIDA · ENEMIGO', {
      name: 'CHARIZARD',
      level: 'Nv. 24  ·  TIPO FUEGO / VOLADOR',
      hp: 142,
      maxHp: 180,
      pct: 0.79,
      ghostPct: 0.825,
      colors: ['#ffd0b0', '#f0525c', '#9e1130'],
      accent: '#ff9c9c',
      icon: 'flame',
      subline: 'HP 79%  ·  INTENCIÓN: 18',
      valueLine: 'PUNTOS DE SALUD',
    })
  )
  .board(
    'hp_boss',
    700,
    200,
    barBoard('BARRA DE VIDA · LÍDER', {
      name: 'MISTY',
      level: 'Nv. 30  ·  LÍDER DE GIMNASIO',
      hp: 260,
      maxHp: 320,
      pct: 0.81,
      ghostPct: 0.845,
      colors: ['#ecd6ff', '#b06bff', '#5e1fa8'],
      accent: '#d9b3ff',
      icon: 'crown',
      subline: 'FASE 2 / 3  ·  ESCUDO 24',
      valueLine: 'PUNTOS DE SALUD',
      boss: true,
    })
  )
  .board('hp_bar_bg', 700, 80, (c) => {
    const x = 20, y = 18, w = BAR_W, h = BAR_H;
    c.path(barShape(x - 4, y - 4, w + 8, h + 8, 22), {
      fillGrad: [
        { offset: 0, color: '#2a3a55' },
        { offset: 0.5, color: '#111a28' },
        { offset: 1, color: '#070b12' },
      ],
      stroke: '#05080d',
      strokeW: 2,
      shadow: { opacity: 60, y: 8, blur: 16 },
    });
    c.path(barShape(x, y, w, h, 19), {
      fillGrad: [
        { offset: 0, color: '#070b12' },
        { offset: 1, color: '#111a28' },
      ],
      stroke: '#3b4d6b',
      strokeW: 2.5,
    });
    for (let i = 1; i < 10; i++) {
      const tx = x + 5 + ((w - 10) * i) / 10;
      c.rect(tx - 1, y + 6, 2, h - 12, 1, {
        fillGrad: [
          { offset: 0, color: '#05080d', opacity: 0.1 },
          { offset: 1, color: '#05080d', opacity: 0.5 },
        ],
        stroke: null,
      });
    }
  })
  .board('hp_fill_player', 700, 80, (c) => {
    const x = 20, y = 18, w = BAR_W, h = BAR_H;
    c.path(barShape(x, y, w, h, 19), {
      fillGrad: [
        { offset: 0, color: '#c8ffdc' },
        { offset: 0.35, color: '#45d47c' },
        { offset: 1, color: '#138a45' },
      ],
      stroke: '#7dffb0',
      strokeW: 2,
    });
    c.path(barShape(x + 5, y + 5, w - 10, h * 0.46, 12), {
      fillGrad: [
        { offset: 0, color: '#ffffff', opacity: 0.42 },
        { offset: 1, color: '#ffffff', opacity: 0 },
      ],
      stroke: null,
    });
  })
  .board('hp_fill_enemy', 700, 80, (c) => {
    const x = 20, y = 18, w = BAR_W, h = BAR_H;
    c.path(barShape(x, y, w, h, 19), {
      fillGrad: [
        { offset: 0, color: '#ffd0b0' },
        { offset: 0.35, color: '#f0525c' },
        { offset: 1, color: '#9e1130' },
      ],
      stroke: '#ff9c9c',
      strokeW: 2,
    });
    c.path(barShape(x + 5, y + 5, w - 10, h * 0.46, 12), {
      fillGrad: [
        { offset: 0, color: '#ffffff', opacity: 0.42 },
        { offset: 1, color: '#ffffff', opacity: 0 },
      ],
      stroke: null,
    });
  })
  .board('hp_fill_boss', 700, 80, (c) => {
    const x = 20, y = 18, w = BAR_W, h = BAR_H;
    c.path(barShape(x, y, w, h, 19), {
      fillGrad: [
        { offset: 0, color: '#ecd6ff' },
        { offset: 0.35, color: '#b06bff' },
        { offset: 1, color: '#5e1fa8' },
      ],
      stroke: '#d9b3ff',
      strokeW: 2,
    });
    c.path(barShape(x + 5, y + 5, w - 10, h * 0.46, 12), {
      fillGrad: [
        { offset: 0, color: '#ffffff', opacity: 0.42 },
        { offset: 1, color: '#ffffff', opacity: 0 },
      ],
      stroke: null,
    });
  })
  .board('energy_orb', 200, 210, (c) => {
    const cx = 100;
    const cy = 100;
    const r = 68;
    // outer metal ring
    c.circle(cx, cy + 5, r + 14, { fill: '#000000', stroke: null, opacity: 0.35 });
    c.circle(cx, cy, r + 13, {
      fillGrad: [
        { offset: 0, color: '#5c7ba6' },
        { offset: 0.5, color: '#1b2a41' },
        { offset: 1, color: '#070b12' },
      ],
      stroke: '#05080d',
      strokeW: 2,
      shadow: { opacity: 55, y: 8, blur: 16 },
    });
    // orb
    c.circle(cx, cy, r, {
      fillGrad: [
        { offset: 0, color: '#e6ffff' },
        { offset: 0.38, color: '#4de0e6' },
        { offset: 0.78, color: '#127f9e' },
        { offset: 1, color: '#073b53' },
      ],
      stroke: '#9bf6ff',
      strokeW: 2,
    });
    c.glow({ opacity: 40, blur: 22, color: '#4de0e6' });
    // lightning watermark
    bolt(c, cx, cy - 6, 104, '#ffffff', null);
    c.opacity(22);
    // glass shine
    c.path(`M ${cx - 50} ${cy - 30} C ${cx - 30} ${cy - 58} ${cx + 10} ${cy - 62} ${cx + 34} ${cy - 48} C ${cx + 8} ${cy - 42} ${cx - 20} ${cy - 20} ${cx - 50} ${cy - 30} Z`, {
      fillGrad: [
        { offset: 0, color: '#ffffff', opacity: 0.65 },
        { offset: 1, color: '#ffffff', opacity: 0 },
      ],
      stroke: null,
    });
    // bottom banner
    c.rect(cx - 44, cy + 28, 88, 38, 12, {
      fillGrad: [
        { offset: 0, color: '#0c2233', opacity: 0.96 },
        { offset: 1, color: '#050d16', opacity: 0.96 },
      ],
      stroke: '#9bf6ff',
      strokeW: 2,
    });
    // orbital dots
    for (let i = 0; i < 8; i++) {
      const a = (i * Math.PI * 2) / 8 - Math.PI / 2;
      const dx = cx + Math.cos(a) * (r + 20);
      const dy = cy + Math.sin(a) * (r + 20);
      c.circle(dx, dy, 4, {
        fill: i < 3 ? '#eaffff' : '#31506b',
        stroke: i < 3 ? '#4de0e6' : '#0a111c',
        strokeW: 1,
      });
    }
  })
  .board('block_badge', 160, 180, (c) => {
    const cx = 80;
    const cy = 90;
    const w = 104;
    const h = 122;
    const d = `M ${cx - w / 2} ${cy - h / 2 + 12}
      Q ${cx - w / 2} ${cy - h / 2} ${cx - w / 2 + 14} ${cy - h / 2}
      L ${cx + w / 2 - 14} ${cy - h / 2}
      Q ${cx + w / 2} ${cy - h / 2} ${cx + w / 2} ${cy - h / 2 + 12}
      L ${cx + w / 2} ${cy + h * 0.1}
      Q ${cx + w / 2} ${cy + h * 0.34} ${cx} ${cy + h / 2}
      Q ${cx - w / 2} ${cy + h * 0.34} ${cx - w / 2} ${cy + h * 0.1} Z`;
    c.path(d, {
      fillGrad: [
        { offset: 0, color: '#bcd9ff' },
        { offset: 0.45, color: '#4aa8ff' },
        { offset: 1, color: '#123c66' },
      ],
      stroke: '#bfe4ff',
      strokeW: 3,
      shadow: { opacity: 55, y: 6, blur: 12 },
    });
    c.path(`M ${cx - 34} ${cy - 18} L ${cx} ${cy - 38} L ${cx + 34} ${cy - 18} L ${cx + 26} ${cy + 22} L ${cx} ${cy + 42} L ${cx - 26} ${cy + 22} Z`, {
      fillGrad: [
        { offset: 0, color: '#ffffff', opacity: 0.4 },
        { offset: 1, color: '#ffffff', opacity: 0 },
      ],
      stroke: null,
    });
  });

doc.build(path.resolve(__dirname, '../../assets/ui/bars'));
