const path = require('path');
const { Doc, C } = require('./ui_lib');

const doc = new Doc('buttons');

const W = 440, H = 128;
const X = 20, Y = 20, BW = 400, BH = 88;

const schemes = {
  primary: {
    body: [
      { offset: 0, color: '#ffc46b' },
      { offset: 0.42, color: '#ff8a3d' },
      { offset: 1, color: '#e8452b' },
    ],
    border: '#ffe3a6',
    outer: ['#3a2a22', '#120d0a'],
    text: '#fff8ed',
    glow: C.primary,
    icon: 'chevron',
  },
  secondary: {
    body: [
      { offset: 0, color: '#3b5374' },
      { offset: 0.48, color: '#22334c' },
      { offset: 1, color: '#111b2b' },
    ],
    border: '#708db8',
    outer: ['#1d2a3d', '#080d15'],
    text: '#eaf2ff',
    glow: C.blue,
    icon: null,
  },
  danger: {
    body: [
      { offset: 0, color: '#ff8a8a' },
      { offset: 0.45, color: '#e13a4a' },
      { offset: 1, color: '#9e1130' },
    ],
    border: '#ffc1c1',
    outer: ['#3a1a20', '#12070a'],
    text: '#fff5f5',
    glow: C.red,
    icon: null,
  },
  gold: {
    body: [
      { offset: 0, color: '#ffeaa6' },
      { offset: 0.42, color: '#f6c04a' },
      { offset: 1, color: '#c98a17' },
    ],
    border: '#fff6cf',
    outer: ['#3b2c13', '#120d05'],
    text: '#2b1804',
    glow: C.gold,
    icon: 'pokeball',
  },
  disabled: {
    body: [
      { offset: 0, color: '#3a4352' },
      { offset: 0.55, color: '#252c38' },
      { offset: 1, color: '#171c25' },
    ],
    border: '#4a5464',
    outer: ['#1b2029', '#080a0e'],
    text: '#7f8a9c',
    glow: null,
    icon: null,
  },
};

function button(ctx, kind, label, state = 'normal') {
  const s = schemes[kind];
  const pressed = state === 'pressed';
  const hover = state === 'hover';
  const oy = pressed ? 4 : 0;
  const alpha = kind === 'disabled' ? 0.6 : 1;

  // dark plate behind the button
  ctx.rect(X - 5, Y - 5, BW + 10, BH + 10, 27, {
    fillGrad: [
      { offset: 0, color: s.outer[0] },
      { offset: 1, color: s.outer[1] },
    ],
    stroke: '#05080d',
    strokeW: 2,
    shadow: pressed ? { opacity: 30, y: 4, blur: 10 } : { opacity: 62, y: 12, blur: 20 },
  });

  // base body
  ctx.rect(X, Y + oy, BW, BH, 22, {
    fillGrad: s.body,
    stroke: s.border,
    strokeW: hover ? 3.5 : 2.5,
  });

  if (hover && s.glow) {
    ctx.glow({ opacity: 50, blur: 18, color: s.glow });
  }

  // glass highlight across the top
  ctx.rect(X + 7, Y + oy + 6, BW - 14, BH * 0.46, 17, {
    fillGrad: [
      { offset: 0, color: '#ffffff', opacity: pressed ? 0.08 : hover ? 0.42 : 0.3 },
      { offset: 0.65, color: '#ffffff', opacity: 0.06 },
      { offset: 1, color: '#ffffff', opacity: 0 },
    ],
    stroke: null,
  });

  // bottom inner shade
  ctx.rect(X + 7, Y + oy + BH * 0.62, BW - 14, BH * 0.3, 13, {
    fillGrad: [
      { offset: 0, color: '#000000', opacity: 0 },
      { offset: 1, color: '#000000', opacity: pressed ? 0.3 : 0.18 },
    ],
    stroke: null,
  });

  // inner hairline
  ctx.rect(X + 6, Y + oy + 5, BW - 12, BH - 10, 18, {
    fill: null,
    stroke: '#ffffff',
    strokeW: 1,
  });
  ctx.opacity(kind === 'disabled' ? 8 : 20);

  // rivets
  for (const rx of [X + 15, X + BW - 15]) {
    ctx.circle(rx, Y + oy + BH / 2, 3.2, {
      fill: s.border,
      stroke: '#05080d',
      strokeW: 1,
    });
  }

  if (s.icon === 'pokeball') {
    ctx.pokeball(X + 46, Y + oy + BH / 2, 19, '#e8452b', '#f7f9ff', '#111827');
  }

  if (label) {
    let textX = s.icon === 'pokeball' ? X + 82 : X + 34;
    let textW = s.icon === 'pokeball' ? BW - 112 : BW - 68;
    ctx.text(textX, Y + oy + 18, textW, BH - 36, label, {
      size: 25,
      style: 'SemiBold',
      color: s.text,
      tracking: 70,
      justify: 'center',
      valign: 'center',
      fit: 'shrinkText',
    });
  }

  if (s.icon === 'chevron') {
    // small forward chevron at the right
    const cx = X + BW - 38;
    const cy = Y + oy + BH / 2;
    ctx.path(`M ${cx - 4} ${cy - 10} L ${cx + 5} ${cy} L ${cx - 4} ${cy + 10}`, {
      closed: false,
      fill: null,
      stroke: '#fff8ed',
      strokeW: 4,
      strokeOpts: { cap: 'round', join: 'round' },
    });
  }
}

doc
  .board('btn_primary', W, H, (c) => button(c, 'primary', ''))
  .board('btn_primary_hover', W, H, (c) => button(c, 'primary', '', 'hover'))
  .board('btn_primary_pressed', W, H, (c) => button(c, 'primary', '', 'pressed'))
  .board('btn_secondary', W, H, (c) => button(c, 'secondary', ''))
  .board('btn_danger', W, H, (c) => button(c, 'danger', ''))
  .board('btn_gold', W, H, (c) => button(c, 'gold', ''))
  .board('btn_disabled', W, H, (c) => button(c, 'disabled', ''))
  .board('btn_icon', 128, 128, (c) => {
    c.rect(14, 14, 100, 100, 26, {
      fillGrad: [
        { offset: 0, color: '#33445f' },
        { offset: 0.5, color: '#1a2537' },
        { offset: 1, color: '#0d1420' },
      ],
      stroke: '#708db8',
      strokeW: 2.5,
      shadow: { opacity: 55, y: 8, blur: 16 },
    });
    c.rect(22, 20, 84, 38, 18, {
      fillGrad: [
        { offset: 0, color: '#ffffff', opacity: 0.26 },
        { offset: 1, color: '#ffffff', opacity: 0 },
      ],
      stroke: null,
    });
    c.gear(64, 64, 26, '#dbe8ff', '#1a2537');
    c.circle(64, 64, 7, { fill: '#4aa8ff', stroke: null });
  });

doc.build(path.resolve(__dirname, '../../assets/ui/buttons'));
