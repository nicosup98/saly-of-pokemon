// Shared helpers for building VectorCraft UI documents.
const fs = require('fs');

const C = {
  primary: '#ff7a3d',
  primaryLight: '#ffb347',
  primaryDeep: '#e23b2b',
  gold: '#f2c14e',
  goldLight: '#ffe08a',
  goldDeep: '#b7821f',
  ink950: '#070a10',
  ink900: '#0b1019',
  ink850: '#0f1622',
  ink800: '#141d2c',
  ink700: '#1d293d',
  ink600: '#2a3a55',
  line: '#3b4d6b',
  text: '#eef3ff',
  muted: '#93a4c0',
  blue: '#4aa8ff',
  cyan: '#4de0e6',
  green: '#45d47c',
  red: '#f0525c',
  purple: '#b06bff',
  orange: '#ff8a3d',
};

class Doc {
  constructor(name) {
    this.name = name;
    this.boards = [];
  }

  board(name, w, h, draw) {
    this.boards.push({ name, w, h, draw });
    return this;
  }

  build(outDir) {
    const cmds = [];
    const push = (command, params = {}) => cmds.push({ command, params });
    const gap = 80;
    let x = 0;
    let maxH = 0;
    for (const b of this.boards) {
      b.x = x;
      b.y = 0;
      x += b.w + gap;
      maxH = Math.max(maxH, b.h);
    }
    const first = this.boards[0];
    push('file.new', {
      name: this.name,
      width: first.w,
      height: first.h,
      units: 'Pixels',
      backgroundContents: 'transparent',
    });
    push('artboard.setProps', { index: 0, name: first.name });
    for (let i = 1; i < this.boards.length; i++) {
      const b = this.boards[i];
      push('artboard.new', { x: b.x, y: 0, width: b.w, height: b.h, name: b.name });
    }
    for (const b of this.boards) {
      const ctx = new Ctx(push, b.x, b.y, b.w, b.h);
      b.draw(ctx);
    }
    // save the editable master document
    push('file.saveAs', { path: `${outDir}/${this.name}.vectorcraft` });
    // exports: one PNG (2x) and one SVG (text outlined) per artboard
    for (let i = 0; i < this.boards.length; i++) {
      const b = this.boards[i];
      push('document.export', {
        path: `${outDir}/${b.name}.png`,
        format: 'png',
        artboard: i,
        scale: 2,
        background: 'transparent',
      });
      push('document.export', {
        path: `${outDir}/${b.name}.svg`,
        format: 'svg',
        artboard: i,
        svg: { outlineText: true, decimals: 3 },
      });
    }
    fs.writeFileSync(`${outDir}/${this.name}.commands.json`, JSON.stringify(cmds, null, 1));
    return cmds;
  }
}

class Ctx {
  constructor(push, ox, oy, w, h) {
    this.push = push;
    this.ox = ox;
    this.oy = oy;
    this.w = w;
    this.h = h;
  }

  cmd(command, params = {}) {
    this.push(command, params);
  }

  // ---- paints ---------------------------------------------------------
  fill(color) {
    this.cmd('paint.setFill', color ? { color } : { none: true });
  }

  fillGrad(stops, angle = -90, opts = {}) {
    this.cmd('paint.setFill', { gradient: { kind: opts.kind || 'linear', stops, angle, ...opts } });
  }

  stroke(color, weight = 2, opts = {}) {
    if (!color) {
      this.cmd('paint.setStroke', { none: true });
      return;
    }
    this.cmd('paint.setStroke', { color });
    this.cmd('stroke.set', { weight, ...opts });
  }

  strokeGrad(stops, weight = 2, angle = -90, opts = {}) {
    this.cmd('paint.setStroke', { gradient: { kind: 'linear', stops, angle } });
    this.cmd('stroke.set', { weight, ...opts });
  }

  shadow(opts = {}) {
    this.cmd('effect.apply', {
      effect: 'stylize.dropShadow',
      params: {
        mode: 'multiply',
        opacity: opts.opacity ?? 55,
        x: opts.x ?? 0,
        y: opts.y ?? 8,
        blur: opts.blur ?? 14,
        color: opts.color ?? '#000000',
      },
    });
  }

  glow(opts = {}) {
    this.cmd('effect.apply', {
      effect: 'stylize.outerGlow',
      params: {
        mode: opts.mode ?? 'screen',
        opacity: opts.opacity ?? 45,
        blur: opts.blur ?? 12,
        color: opts.color ?? C.primary,
      },
    });
  }

  opacity(pct) {
    this.cmd('object.setProps', { opacity: pct });
  }

  // ---- shapes ---------------------------------------------------------
  rect(x, y, w, h, r = 0, opts = {}) {
    const p = { x: this.ox + x, y: this.oy + y, width: w, height: h };
    if (r) p.radius = r;
    if (opts.rotation) p.rotation = opts.rotation;
    this.cmd('shape.rectangle', p);
    if (opts.fillGrad) this.fillGrad(opts.fillGrad, opts.angle ?? -90, opts.gradOpts || {});
    else if (opts.fill === null) this.fill(null);
    else this.fill(opts.fill || C.ink800);
    if (opts.stroke) this.stroke(opts.stroke, opts.strokeW ?? 2, opts.strokeOpts || {});
    else this.stroke(null);
    if (opts.shadow) this.shadow(opts.shadow);
    if (opts.glow) this.glow(opts.glow);
    if (opts.opacity !== undefined) this.opacity(opts.opacity);
    return this;
  }

  ellipse(x, y, w, h, opts = {}) {
    this.cmd('shape.ellipse', { x: this.ox + x, y: this.oy + y, width: w, height: h });
    if (opts.fillGrad) this.fillGrad(opts.fillGrad, opts.angle ?? -90, opts.gradOpts || {});
    else if (opts.fill === null) this.fill(null);
    else this.fill(opts.fill || C.ink800);
    if (opts.stroke) this.stroke(opts.stroke, opts.strokeW ?? 2);
    else this.stroke(null);
    if (opts.shadow) this.shadow(opts.shadow);
    if (opts.glow) this.glow(opts.glow);
    if (opts.opacity !== undefined) this.opacity(opts.opacity);
    return this;
  }

  circle(cx, cy, r, opts = {}) {
    return this.ellipse(cx - r, cy - r, r * 2, r * 2, opts);
  }

  polygon(cx, cy, r, sides = 6, opts = {}) {
    this.cmd('shape.polygon', {
      cx: this.ox + cx,
      cy: this.oy + cy,
      radius: r,
      sides,
      ...(opts.rotation !== undefined ? { rotation: opts.rotation } : {}),
    });
    if (opts.fillGrad) this.fillGrad(opts.fillGrad, opts.angle ?? -90);
    else this.fill(opts.fill || C.ink800);
    if (opts.stroke) this.stroke(opts.stroke, opts.strokeW ?? 2);
    else this.stroke(null);
    return this;
  }

  star(cx, cy, r1, r2, points = 5, opts = {}) {
    this.cmd('shape.star', {
      cx: this.ox + cx,
      cy: this.oy + cy,
      radius1: r1,
      radius2: r2,
      points,
      ...(opts.rotation !== undefined ? { rotation: opts.rotation } : {}),
    });
    if (opts.fillGrad) this.fillGrad(opts.fillGrad, opts.angle ?? -90);
    else this.fill(opts.fill || C.gold);
    if (opts.stroke) this.stroke(opts.stroke, opts.strokeW ?? 2);
    else this.stroke(null);
    if (opts.shadow) this.shadow(opts.shadow);
    return this;
  }

  path(d, opts = {}) {
    this.cmd('path.create', { d, closed: opts.closed !== false });
    if (this.ox || this.oy) this.cmd('object.move', { dx: this.ox, dy: this.oy });
    if (opts.fillGrad) this.fillGrad(opts.fillGrad, opts.angle ?? -90, opts.gradOpts || {});
    else if (opts.fill === null) this.fill(null);
    else this.fill(opts.fill || C.ink800);
    if (opts.stroke) this.stroke(opts.stroke, opts.strokeW ?? 2, opts.strokeOpts || {});
    else this.stroke(null);
    if (opts.shadow) this.shadow(opts.shadow);
    if (opts.glow) this.glow(opts.glow);
    if (opts.opacity !== undefined) this.opacity(opts.opacity);
    if (opts.rotate) {
      this.cmd('object.rotate', {
        angle: opts.rotate,
        origin: opts.rotateOrigin ? [this.ox + opts.rotateOrigin[0], this.oy + opts.rotateOrigin[1]] : undefined,
      });
    }
    return this;
  }

  line(x1, y1, x2, y2, color, weight = 2, opts = {}) {
    this.cmd('shape.line', { x1: this.ox + x1, y1: this.oy + y1, x2: this.ox + x2, y2: this.oy + y2 });
    this.fill(null);
    this.stroke(color, weight, opts);
    return this;
  }

  image(src, x, y, w, h, opts = {}) {
    this.cmd('file.place', { path: src, rect: [this.ox + x, this.oy + y, w, h] });
    if (opts.shadow) this.shadow(opts.shadow);
    if (opts.glow) this.glow(opts.glow);
    if (opts.opacity !== undefined) this.opacity(opts.opacity);
    return this;
  }

  // ---- type -----------------------------------------------------------
  text(x, y, w, h, str, opts = {}) {
    this.cmd('text.create', {
      x: this.ox + x,
      y: this.oy + y,
      text: str,
      size: opts.size ?? 16,
      font: opts.font ?? 'Inter',
      style: opts.style ?? 'SemiBold',
      color: opts.color ?? C.text,
      justify: opts.justify ?? 'left',
      area: { width: w, height: h, fit: opts.fit ?? 'shrinkText' },
    });
    this.cmd('text.areaOptions', {
      width: w,
      height: h,
      inset: opts.inset ?? 0,
      verticalAlign: opts.valign ?? 'center',
      fit: opts.fit ?? 'shrinkText',
      fitMinPercent: opts.fitMinPercent ?? 40,
    });
    if (opts.tracking !== undefined || opts.leading !== undefined || opts.size) {
      const p = {};
      if (opts.tracking !== undefined) p.tracking = opts.tracking;
      if (opts.leading !== undefined) p.leading = opts.leading;
      if (opts.size) p.size = opts.size;
      this.cmd('text.setStyle', p);
    }
    return this;
  }

  label(x, y, w, h, str, opts = {}) {
    return this.text(x, y, w, h, str, {
      size: 15,
      style: 'SemiBold',
      tracking: 80,
      justify: 'left',
      color: C.text,
      ...opts,
    });
  }

  // ---- composite parts ------------------------------------------------
  panel(x, y, w, h, r, opts = {}) {
    const accent = opts.accent || C.line;
    // outer dark frame
    this.rect(x - 5, y - 5, w + 10, h + 10, r + 5, {
      fillGrad: [
        { offset: 0, color: opts.frameTop || '#26354d' },
        { offset: 1, color: opts.frameBottom || '#0a0f18' },
      ],
      stroke: opts.frameStroke || '#05080d',
      strokeW: 2,
      shadow: opts.shadow === false ? null : { opacity: 60, y: 12, blur: 22 },
    });
    // body
    this.rect(x, y, w, h, r, {
      fillGrad: opts.bodyGrad || [
        { offset: 0, color: '#1a2436' },
        { offset: 0.55, color: '#121a29' },
        { offset: 1, color: '#0b111c' },
      ],
      stroke: accent,
      strokeW: opts.borderW ?? 2.5,
    });
    // inner top sheen
    this.rect(x + 8, y + 7, w - 16, Math.min(34, h * 0.22), Math.max(2, r - 4), {
      fillGrad: [
        { offset: 0, color: '#ffffff', opacity: 0.16 },
        { offset: 1, color: '#ffffff', opacity: 0 },
      ],
      stroke: null,
    });
    return this;
  }

  pokeball(cx, cy, r, top = '#f0525c', bottom = '#f7f9ff', band = '#111827', opts = {}) {
    // body
    this.circle(cx, cy, r, { fill: bottom, stroke: opts.stroke || '#0a0f18', strokeW: opts.strokeW ?? 3 });
    // top half
    this.path(
      `M ${cx - r} ${cy} A ${r} ${r} 0 0 1 ${cx + r} ${cy} Z`,
      { fill: top, stroke: null }
    );
    // band
    this.rect(cx - r, cy - r * 0.12, r * 2, r * 0.24, 0, {
      fill: band,
      stroke: null,
    });
    // center button
    this.circle(cx, cy, r * 0.3, { fill: band, stroke: opts.ring || '#f7f9ff', strokeW: r * 0.08 });
    this.circle(cx, cy, r * 0.16, { fill: opts.core || '#f7f9ff', stroke: null });
    return this;
  }

  gear(cx, cy, r, color = C.text, hole = C.ink800) {
    const teeth = 8;
    for (let i = 0; i < teeth; i++) {
      const a = (i * 360) / teeth;
      this.rect(cx - r * 0.13, cy - r * 1.12, r * 0.26, r * 0.4, r * 0.07, {
        fill: color,
        stroke: null,
      });
      this.cmd('object.rotate', { angle: a, origin: [this.ox + cx, this.oy + cy] });
    }
    this.circle(cx, cy, r * 0.78, { fill: color, stroke: null });
    this.circle(cx, cy, r * 0.34, { fill: hole, stroke: null });
    return this;
  }

  crown(cx, y, w, h, opts = {}) {
    const x = cx - w / 2;
    const baseY = y + h;
    const pts = [
      [x, y + h * 0.12],
      [x + w * 0.12, y + h * 0.12],
      [x + w * 0.24, y + h * 0.54],
      [x + w * 0.38, y + h * 0.2],
      [x + w * 0.5, y + h * 0.66],
      [x + w * 0.62, y + h * 0.2],
      [x + w * 0.76, y + h * 0.54],
      [x + w * 0.88, y + h * 0.12],
      [x + w, y + h * 0.12],
      [x + w * 0.9, baseY],
      [x + w * 0.1, baseY],
    ];
    const d = 'M ' + pts.map((p) => `${p[0]} ${p[1]}`).join(' L ') + ' Z';
    this.path(d, {
      fillGrad: opts.fillGrad || [
        { offset: 0, color: '#ffe08a' },
        { offset: 0.55, color: '#f2b33d' },
        { offset: 1, color: '#b7821f' },
      ],
      stroke: opts.stroke || '#6b4306',
      strokeW: opts.strokeW ?? 3,
      shadow: opts.shadow,
    });
    // jewels
    const jy = y + h * 0.74;
    this.circle(cx, jy, w * 0.075, { fill: opts.jewel || '#e8455d', stroke: '#6b1220', strokeW: 2 });
    this.circle(x + w * 0.28, jy, w * 0.05, { fill: opts.jewel2 || '#4aa8ff', stroke: '#123c66', strokeW: 1.5 });
    this.circle(x + w * 0.72, jy, w * 0.05, { fill: opts.jewel2 || '#4aa8ff', stroke: '#123c66', strokeW: 1.5 });
    // tips
    for (const [tx, ty] of [
      [x + w * 0.12, y + h * 0.12],
      [x + w * 0.88, y + h * 0.12],
    ]) {
      this.circle(tx, ty, w * 0.055, { fill: opts.tip || '#fff3c4', stroke: '#6b4306', strokeW: 1.5 });
    }
    return this;
  }

  flame(cx, cy, w, h, opts = {}) {
    const d = `M ${cx} ${cy - h / 2}
      C ${cx + w * 0.34} ${cy - h * 0.18} ${cx + w * 0.48} ${cy + h * 0.06} ${cx + w * 0.3} ${cy + h * 0.3}
      C ${cx + w * 0.14} ${cy + h * 0.5} ${cx - w * 0.2} ${cy + h * 0.48} ${cx - w * 0.34} ${cy + h * 0.26}
      C ${cx - w * 0.46} ${cy + h * 0.06} ${cx - w * 0.28} ${cy - h * 0.12} ${cx - w * 0.16} ${cy - h * 0.24}
      C ${cx - w * 0.08} ${cy - h * 0.34} ${cx - w * 0.04} ${cy - h * 0.42} ${cx} ${cy - h / 2} Z`;
    this.path(d, {
      fillGrad: opts.fillGrad || [
        { offset: 0, color: '#ffd76b' },
        { offset: 0.55, color: '#ff8a3d' },
        { offset: 1, color: '#e23b2b' },
      ],
      stroke: opts.stroke || '#7a1f12',
      strokeW: opts.strokeW ?? 3,
      shadow: opts.shadow,
    });
    if (opts.inner !== false) {
      const d2 = `M ${cx} ${cy - h * 0.24}
        C ${cx + w * 0.18} ${cy - h * 0.04} ${cx + w * 0.22} ${cy + h * 0.12} ${cx + w * 0.1} ${cy + h * 0.26}
        C ${cx + w * 0.02} ${cy + h * 0.36} ${cx - w * 0.12} ${cy + h * 0.32} ${cx - w * 0.16} ${cy + h * 0.18}
        C ${cx - w * 0.2} ${cy + h * 0.04} ${cx - w * 0.1} ${cy - h * 0.08} ${cx} ${cy - h * 0.24} Z`;
      this.path(d2, {
        fill: opts.innerColor || '#ffe9a8',
        stroke: null,
      });
    }
    return this;
  }

  cross(cx, cy, size, thickness, opts = {}) {
    const r = opts.radius ?? thickness * 0.25;
    this.rect(cx - thickness / 2, cy - size / 2, thickness, size, r, { fill: opts.fill || '#f7f9ff', stroke: opts.stroke || null, strokeW: 2 });
    this.rect(cx - size / 2, cy - thickness / 2, size, thickness, r, { fill: opts.fill || '#f7f9ff', stroke: opts.stroke || null, strokeW: 2 });
    return this;
  }

  bag(cx, cy, w, h, opts = {}) {
    const x = cx - w / 2;
    const y = cy - h / 2;
    // handle
    this.path(
      `M ${x + w * 0.28} ${y + h * 0.24} C ${x + w * 0.28} ${y - h * 0.1} ${x + w * 0.72} ${y - h * 0.1} ${x + w * 0.72} ${y + h * 0.24}`,
      { closed: false, fill: null, stroke: opts.handle || '#d8e6ff', strokeW: Math.max(5, w * 0.07), strokeOpts: { cap: 'round' } }
    );
    // body
    this.rect(x, y + h * 0.16, w, h * 0.84, w * 0.12, {
      fillGrad: opts.fillGrad || [
        { offset: 0, color: '#7cc4ff' },
        { offset: 0.55, color: '#2f7fd6' },
        { offset: 1, color: '#184a86' },
      ],
      stroke: opts.stroke || '#0a2440',
      strokeW: 3,
      shadow: opts.shadow,
    });
    // flap
    this.path(
      `M ${x + w * 0.06} ${y + h * 0.38} L ${x + w * 0.94} ${y + h * 0.38} L ${x + w * 0.86} ${y + h * 0.56} L ${x + w * 0.14} ${y + h * 0.56} Z`,
      { fillGrad: [{ offset: 0, color: '#b9e2ff', opacity: 0.7 }, { offset: 1, color: '#b9e2ff', opacity: 0.08 }], stroke: null }
    );
    // emblem
    this.circle(cx, y + h * 0.66, w * 0.18, { fill: '#0d2b4d', stroke: '#8fd0ff', strokeW: 2 });
    this.text(cx - w * 0.14, y + h * 0.58, w * 0.28, h * 0.18, '₽', {
      size: w * 0.2,
      style: 'SemiBold',
      color: '#cfeaff',
      justify: 'center',
      valign: 'center',
      tracking: 0,
    });
    return this;
  }

  chest(cx, cy, w, h, opts = {}) {
    const x = cx - w / 2;
    const y = cy - h / 2;
    // base
    this.rect(x, y + h * 0.38, w, h * 0.62, w * 0.08, {
      fillGrad: [
        { offset: 0, color: '#b8752a' },
        { offset: 0.5, color: '#7d4317' },
        { offset: 1, color: '#4d260c' },
      ],
      stroke: '#2b1406',
      strokeW: 3,
      shadow: opts.shadow,
    });
    // lid
    this.path(
      `M ${x} ${y + h * 0.42} L ${x} ${y + h * 0.26} C ${x} ${y + h * 0.02} ${x + w} ${y + h * 0.02} ${x + w} ${y + h * 0.26} L ${x + w} ${y + h * 0.42} Z`,
      {
        fillGrad: [
          { offset: 0, color: '#e0a04a' },
          { offset: 0.6, color: '#a05a1d' },
          { offset: 1, color: '#6b3510' },
        ],
        stroke: '#2b1406',
        strokeW: 3,
      }
    );
    // metal bands
    for (const bx of [x + w * 0.18, x + w * 0.82]) {
      this.rect(bx - w * 0.035, y + h * 0.12, w * 0.07, h * 0.8, 2, {
        fillGrad: [
          { offset: 0, color: '#ffe08a' },
          { offset: 0.5, color: '#d6a02c' },
          { offset: 1, color: '#8a5a0a' },
        ],
        stroke: '#3d2604',
        strokeW: 1.5,
      });
    }
    // lock
    this.rect(cx - w * 0.1, y + h * 0.4, w * 0.2, h * 0.22, w * 0.035, {
      fillGrad: [
        { offset: 0, color: '#fff0b0' },
        { offset: 1, color: '#c88c16' },
      ],
      stroke: '#3d2604',
      strokeW: 2,
    });
    this.circle(cx, y + h * 0.51, w * 0.035, { fill: '#3a2406', stroke: null });
    return this;
  }

  sword(cx, cy, len, opts = {}) {
    const w = len * 0.13;
    const d = `M ${cx} ${cy - len * 0.52}
      L ${cx + w * 0.55} ${cy - len * 0.3}
      L ${cx + w * 0.55} ${cy + len * 0.12}
      L ${cx + w * 1.15} ${cy + len * 0.12}
      L ${cx + w * 1.15} ${cy + len * 0.22}
      L ${cx + w * 0.42} ${cy + len * 0.22}
      L ${cx + w * 0.42} ${cy + len * 0.42}
      L ${cx - w * 0.42} ${cy + len * 0.42}
      L ${cx - w * 0.42} ${cy + len * 0.22}
      L ${cx - w * 1.15} ${cy + len * 0.22}
      L ${cx - w * 1.15} ${cy + len * 0.12}
      L ${cx - w * 0.55} ${cy + len * 0.12}
      L ${cx - w * 0.55} ${cy - len * 0.3} Z`;
    this.path(d, {
      fillGrad: opts.fillGrad || [
        { offset: 0, color: '#f4f8ff' },
        { offset: 0.45, color: '#b9c8dd' },
        { offset: 1, color: '#6e8099' },
      ],
      stroke: opts.stroke || '#17202e',
      strokeW: opts.strokeW ?? 3,
      rotate: opts.rotate,
      rotateOrigin: opts.rotateOrigin || [cx, cy],
      shadow: opts.shadow,
    });
    return this;
  }
}

module.exports = { Doc, Ctx, C };
