#!/usr/bin/env python3
"""The published page's moving parts, built from the generator's own build()
so the page cannot disagree with the cut file. The writeup is README.md,
built into index.html by md2html.py like every page here; this adds what
markdown cannot carry.

    python3 page_assets.py                     # viewer.js, turn.html and plans/*.svg
    python3 page_assets.py --artifact OUT.html # the whole page, one file

  viewer.js    the 3D view, loaded on the page through a page-only line in
               README.md
  turn.html    viewer.js on a page of its own, for the hub to frame
  plans/       the head of the top wall drawn to scale, and the fingering
               chart, as plain SVG, so they read on GitHub as well as on the
               page

The viewer's styling, its page wrapper, turn.html and the artifact folding
are the square flute's and the four-hole ocarinas', imported from their
page_assets.py; the parts are this instrument's own.
"""
import importlib.util
import json
import math
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import octagonal_flute as B                                           # noqa: E402
M = B.M


def _load(name, repo):
    # loaded by path: by name, `import page_assets` finds this file
    spec = importlib.util.spec_from_file_location(
        name, os.path.join(HERE, '..', repo, 'page_assets.py'))
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


SFP = _load('square_flute_page_assets', 'square-flute')
OCP = _load('four_hole_page_assets', 'four-hole-ocarinas')

T = B.T
FONT = 'font-family="Helvetica, Arial, sans-serif"'
BUILD = 'octagonal-flute-D4'


def solids(parts, info):
    """The parts for the view, each placed by p.m. The tube lies along x,
    head end at 0, top up; exploded, each wall comes away along its own
    outward normal, the lip plate further than the top under it, and the
    cork's discs out of the head end, the outermost furthest."""
    faces = info['faces']
    R = B.FLATS_OUT / 2 / math.cos(math.pi / 8)
    out = []
    corks = 0
    for p in parts:
        m = [list(r) for r in p.m]
        m[2][3] += R                                  # the tube resting on z = 0
        if p.name == 'lip plate':
            e, g = [0, 0, 2.4], 'dbl'
        elif p.name == 'cork disc':
            e, g = [-1.2 - 0.8 * (B.CORK_DISCS - 1 - corks), 0, 0], 'plug'
            corks += 1
        else:
            n = faces[p.face].n
            e = [round(1.3 * x, 4) for x in n]
            g = 'plate' if p.name == 'top wall' else 'wall'
        out.append({'o': [[round(x, 3), round(y, 3)] for x, y in p.outline],
                    'h': [{'p': [[round(x, 3), round(y, 3)] for x, y in c[1]]} for c in p.cuts],
                    'm': [[round(v, 4) for v in r] for r in m],
                    'e': e, 'g': g, 'name': p.name, 'f': [], 'fc': None,
                    'flip': False, 'zc': 0})
    return {'parts': out, 'len': round(info['pos']['length'], 1), 'top': round(2 * R, 2),
            'bore': f'{M.FLATS:g}mm across the flats'}


def plan_svg(parts, info, diams, title):
    """The top wall from outside, to scale, head on the left, as it lies on
    the sheet, cut short past the last hole: the lip plate dashed, the
    embouchure and the six holes named with their sizes, the blue cork
    line, and the bore's inside edges dotted under the wall."""
    tp = next(p for p in parts if p.name == 'top wall')
    pos = info['pos']
    end = pos['holes'][0] + 20
    us = [u for u, v in tp.outline if u <= end]
    vs = [v for u, v in tp.outline]
    x0, y0 = 6, 6 - min(min(vs), -B.LIP_W / 2) + 6
    W = end + 12
    H = max(max(vs), B.LIP_W / 2) - min(min(vs), -B.LIP_W / 2) + 24
    clip = f'<clipPath id="c"><rect x="0" y="0" width="{end + x0:.1f}" height="{H:.1f}"/></clipPath>'
    d = lambda pts, z=' Z': 'M ' + ' L '.join(f'{u + x0:.2f},{v + y0:.2f}' for u, v in pts) + z
    s = [f'<rect x="0" y="0" width="{W:.1f}" height="{H:.1f}" fill="#ffffff"/>', clip,
         f'<g clip-path="url(#c)"><path d="{d(tp.outline)}" fill="#e7dcc4" stroke="#1d232b" '
         f'stroke-width=".4"/></g>']
    w = M.FACE_IN / 2
    s.append(f'<path d="M {x0:.2f},{y0 - w:.2f} H {end + x0:.2f} M {x0:.2f},{y0 + w:.2f} '
             f'H {end + x0:.2f}" fill="none" stroke="#8a7350" stroke-width=".5" '
             f'stroke-dasharray="1 1"/>')
    lip = next(p for p in parts if p.name == 'lip plate')
    s.append(f'<path d="{d(lip.outline)}" fill="none" stroke="#1d232b" stroke-width=".5" '
             f'stroke-dasharray="2 1.5"/>')
    s += [f'<path d="{d(c[1])}" fill="#ffffff" stroke="#a85800" stroke-width=".6"/>'
          for c in tp.cuts]
    s += [f'<path d="{d(st, "")}" fill="none" stroke="{B.BLUE}" stroke-width=".6"/>'
          for st in tp.strokes if st[0][0] < end]
    t = f'{FONT} font-size="4" fill="#1d232b" text-anchor="middle"'
    s.append(f'<text x="{pos["emb"] + x0:.2f}" y="{y0 + B.LIP_W / 2 + 6:.2f}" {t}>'
             f'embouchure {M.EMB_W:g} × {M.EMB_L:g}</text>')
    s.append(f'<text x="{pos["cork_face"] + x0:.2f}" y="{y0 - B.LIP_W / 2 - 2:.2f}" {t}>cork</text>')
    for k, (h, x, (across, along)) in enumerate(zip(M.HOLES, pos['holes'], info['shapes'])):
        size = f'ø{across:.1f}' if across == along else f'{across:g} × {along:.1f}'
        y = y0 - w - 3 if k % 2 else y0 + w + 6.5
        s.append(f'<text x="{x + x0:.2f}" y="{y:.2f}" {t}>{h} {size}</text>')
    return (f'<?xml version="1.0" encoding="utf-8"?>\n'
            f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {W:.0f} {H:.0f}">\n'
            f'<title>{title}</title>\n'
            f'<desc>Display drawing, not a cut file. The top wall from outside, to scale, head '
            f'end on the left, cut off 20mm past the lowest hole: its edges cut into fingers, '
            f'left whole beside every opening; the bore\'s inside edges dotted beneath; the '
            f'lip plate\'s outline dashed round the embouchure; the six finger holes named '
            f'for their fingers with their sizes, round or longer along the flute; and the '
            f'blue line where the cork\'s face goes.</desc>\n' + '\n'.join(s) + '\n</svg>\n')


# the six-hole flute's fingerings: holes L1 L2 L3 R1 R2 R3, 1 covered
CHART = [('D4', '111111'), ('E4', '111110'), ('F♯4', '111100'), ('G4', '111000'),
         ('A4', '110000'), ('B4', '100000'), ('C♯5', '000000'), ('D5', '011111')]


def chart_svg(title):
    """The fingering chart: a column of six holes per note, the head up,
    the left hand's three above the right's, black where covered."""
    cw, ch = 22.0, 78.0
    s = [f'<rect x="0" y="0" width="{cw * len(CHART):.0f}" height="{ch:.0f}" fill="#ffffff"/>']
    t = f'{FONT} text-anchor="middle"'
    for i, (note, pat) in enumerate(CHART):
        cx = cw * i + cw / 2
        s.append(f'<rect x="{cx - 5:.1f}" y="3" width="10" height="62" rx="2" '
                 f'fill="#e7dcc4" stroke="#1d232b" stroke-width=".4"/>')
        for k, c in enumerate(pat):
            cy = 10 + 8 * k + (4 if k >= 3 else 0)
            fill = '#1d232b' if c == '1' else '#ffffff'
            s.append(f'<circle cx="{cx:.1f}" cy="{cy:.1f}" r="2.6" fill="{fill}" '
                     f'stroke="#1d232b" stroke-width=".4"/>')
        s.append(f'<text x="{cx:.1f}" y="{ch - 4:.1f}" font-size="6" fill="#1d232b" {t}>{note}</text>')
    return (f'<?xml version="1.0" encoding="utf-8"?>\n'
            f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {cw * len(CHART):.0f} {ch:.0f}">\n'
            f'<title>{title}</title>\n'
            f'<desc>Display drawing. The D major scale from D4 to D5, one column per note: the '
            f'six holes with the head end up, the left hand\'s three above the right\'s. Black is '
            f'covered, white is open. D5 is the second octave, overblown, with the top hole '
            f'open.</desc>\n' + '\n'.join(s) + '\n</svg>\n')


MARKUP = OCP.MARKUP.split('  <div class="vctl">')[0] + (
    '  <div class="vctl">\n    <label for="explode">Explode</label><input id="explode" type="range" '
    'min="0" max="1" step="0.01" value="0">\n    <button type="button" id="vreset">Reset view</button>\n'
    '    <label class="chk"><input type="checkbox" id="xray"> See through the walls</label>\n  </div>\n'
    '  <details class="note vhow"><summary>How to use this view</summary><p>Drag to turn, scroll or '
    'pinch to zoom. The parts are the generator\'s own outlines before the kerf, so this is the flute '
    'as built: eight walls finger-jointed down their length, the top one whole beside every hole; the '
    'embouchure through the top and the lip plate over it; and the cork, three discs pressed into the '
    'head end. Exploded, each wall comes away along its own face, and the cork comes out of the '
    'head.</p></details>\n</section>')


def viewer_js(three):
    js = SFP.JS.split('  // one button per construction')[0]
    a = "'mm · bore ' + d.W.toFixed(2) + ' × ' + d.D.toFixed(2) + 'mm'"
    assert a in js, 'the square flute\'s viewer script changed'
    js = js.replace(a, "'mm · octagonal bore ' + d.bore")
    js += ("  const bb = [...document.querySelectorAll('#turn .vbar button[data-v]')];\n"
           "  bb.forEach(b => b.addEventListener('click', () => {\n"
           "    bb.forEach(x => x.setAttribute('aria-pressed', x === b)); build(b.dataset.v);\n"
           "  }));\n"
           "  window.addEventListener('resize', size);\n"
           "  build(bb[0].dataset.v); size();\n"
           "})();\n")
    js = js.replace('__DATA__', json.dumps(three, separators=(',', ':')))
    buttons = ''.join(f'<button type="button" data-v="{n}" aria-pressed="{str(i == 0).lower()}">'
                      f'<code>{n}</code></button>' for i, n in enumerate(three))
    return (SFP.VIEWER_JS.replace('__CSS__', json.dumps(SFP.CSS))
            .replace('__MARKUP__', json.dumps(MARKUP.replace('__BUTTONS__', buttons)))
            .replace('__JS__', js))


PLANS = ('octagonal-flute-top-wall.svg', 'octagonal-flute-fingering.svg')


def main():
    a = sys.argv[1:]
    if a[:1] == ['--artifact'] and len(a) == 2:
        OCP.HERE = HERE
        return OCP.artifact(a[1])
    vt = [x.split('=', 1)[1] for x in a if x.startswith('--viewer-to=')]
    pt = [x.split('=', 1)[1] for x in a if x.startswith('--plans-to=')]
    tt = [x.split('=', 1)[1] for x in a if x.startswith('--turn-to=')]
    if len(vt) + len(pt) + len(tt) != len(a):
        raise SystemExit('usage: page_assets.py [--viewer-to=FILE] [--plans-to=DIR] '
                         '[--turn-to=FILE] | --artifact OUT.html')
    if tt or not a:
        page = SFP.TURN_PAGE.replace('<title>Square-bore flute in 3D</title>',
                                     '<title>Octagonal flute in 3D</title>')
        assert 'Octagonal flute' in page, 'the square flute\'s turn page changed'
        open(tt[0] if tt else os.path.join(HERE, 'turn.html'), 'w').write(page)
    L, holes, diams = M.design()
    parts, info = B.build(L, holes, diams)
    three = {BUILD: solids(parts, info)}
    if vt or not a:
        open(vt[0] if vt else os.path.join(HERE, 'viewer.js'), 'w').write(viewer_js(three))
    if pt or not a:
        d = pt[0] if pt else os.path.join(HERE, 'plans')
        os.makedirs(d, exist_ok=True)
        open(os.path.join(d, PLANS[0]), 'w').write(
            plan_svg(parts, info, diams, 'The top wall from outside, to scale'))
        open(os.path.join(d, PLANS[1]), 'w').write(chart_svg('Fingering chart, D4 to D5'))
    print(f'  wrote viewer.js; plans: {", ".join(PLANS)}')
    return 0


if __name__ == '__main__':
    sys.exit(main())
