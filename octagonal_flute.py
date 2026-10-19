#!/usr/bin/env python3
"""Cut files for a six-hole transverse flute in D, its bore a long regular
octagon of 3mm ply, finger-jointed down every edge.

    python3 octagonal_flute.py --no-write       # the checks, write nothing
    python3 octagonal_flute.py --out=DIR        # the sheet into DIR
    python3 octagonal_flute.py --mouth=1.2 ...  # the embouchure's correction, as the model
    python3 octagonal_flute.py --kerf=0.18 ...  # for a laser with that kerf

The acoustics are flute_model.py's: how long the bore runs past the
embouchure, and where the six holes go and how big each is. This file only
builds what that asks for.

THE TUBE is eight walls, each a strip as long as the flute, joined edge to
edge at 135 degrees by polyhedron.py's joint from the four-hole ocarinas:
in its own cells a wall runs out to the edge line, in its neighbour's it
stops clear(135) = 2.12mm short, so every joint closes on a line. Both ends
are open; the walls run square to them. The wall on top, the one the holes
and the embouchure go through, takes every cell of both its edges wherever
a hole is, so each hole has the wall's whole width round it.

FROM THE HEAD END:

  head     HEAD_PAST of bare tube, past the cork
  cork     CORK_DISCS octagonal discs, press-fit into the bore, their face
           flute_model.CORK from the embouchure's centre. A blue line on the
           top marks where that face goes; push the cork further in or out
           to move the octaves (see flute_model.py)
  emb.     the embouchure, a stadium EMB_W across and EMB_L along, through
           the top wall and the lip plate glued over it, so the chimney is
           the two plies, 6mm, a wooden flute's depth
  holes    the six, on the top wall's centre line, round where they are
           small and stadia, longer along the flute, where they are not
  foot     open, cut FOOT_SPARE long; a blue line marks where the model
           puts the end, and the foot is trimmed toward it to pitch

THE LIP PLATE is wider than the top wall, which is only 10.4mm across
outside, so the lower lip has something to rest on. Before it is glued, file
the embouchure's far edge, the one the breath strikes, sharp, and undercut
both plies' walls a little, wider inside than out, as on any wooden flute.

The kerf, the sheet packing and the inks are the square flute's, imported
from square_flute.py, and the joint and the mitred offset are the four-hole
ocarinas', from polyhedron.py and four_hole_ocarinas.py: all three
repositories are expected side by side, with swept-curve for the path
writer.
"""
import math
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
for sib in ('square-flute', 'swept-curve', 'four-hole-ocarinas'):
    sys.path.insert(0, os.path.join(HERE, '..', sib))
import flute_model as M                                        # noqa: E402
import square_flute as SF                                      # noqa: E402
import polyhedron as P                                         # noqa: E402
from four_hole_ocarinas import offset                          # noqa: E402
from square_flute import Part, pack                            # noqa: E402
from ribbon_bore import path                                   # noqa: E402

T, KERF = SF.T, SF.KERF
MIN_WEB = SF.MIN_WEB
BED, MARGIN = SF.BED, SF.MARGIN
BLUE, ORANGE, BLACK = SF.BLUE, SF.ORANGE, SF.BLACK

FLATS_OUT = M.FLATS + 2 * T        # the octagon across its flats, outside
FACE_OUT = FLATS_OUT * math.tan(math.pi / 8)     # one wall's outside width
HEAD_PAST = 6.0                    # bare tube past the cork's back
CORK_DISCS = 3
INTERF = SF.INTERF_ACROSS          # the cork's press-fit, across its flats
HOLE_CLEAR = 3.0                   # the top keeps its joint this far past a hole
HOLE_GAP = 3.0                     # ply between neighbouring holes
LIP_W, LIP_L, LIP_R = 26.0, 40.0, 9.0   # the lip plate, across x along, corner radius
STEPS = 16                         # points on a quarter circle


def positions(L, holes):
    """Where everything stands along the walls, from the head end (mm)."""
    cork_back = HEAD_PAST
    cork_face = cork_back + CORK_DISCS * T
    emb = cork_face + M.CORK
    return dict(cork_back=cork_back, cork_face=cork_face, emb=emb,
                holes=[emb + h for h in holes], foot=emb + L,
                length=emb + L + M.FOOT_SPARE)


def stadium(cx, cy, along, across):
    """A stadium centred (cx, cy), `along` long in x and `across` wide in y;
    a circle when the two are equal."""
    r = across / 2
    s = (along - across) / 2
    pts = []
    for k in range(2 * STEPS + 1):
        a = -math.pi / 2 + math.pi * k / (2 * STEPS)
        pts.append((cx + s + r * math.cos(a), cy + r * math.sin(a)))
    for k in range(2 * STEPS + 1):
        a = math.pi / 2 + math.pi * k / (2 * STEPS)
        pts.append((cx - s + r * math.cos(a), cy + r * math.sin(a)))
    out = []
    for p in pts:
        if not out or math.hypot(p[0] - out[-1][0], p[1] - out[-1][1]) > 1e-9:
            out.append(p)
    if math.hypot(out[0][0] - out[-1][0], out[0][1] - out[-1][1]) < 1e-9:
        out.pop()
    return out


def rounded_rect(cx, cy, w, h, r):
    """A rectangle w along x and h across y, centred, corners rounded r."""
    pts = []
    for (qx, qy, a0) in ((cx + w / 2 - r, cy + h / 2 - r, 0.0),
                         (cx - w / 2 + r, cy + h / 2 - r, math.pi / 2),
                         (cx - w / 2 + r, cy - h / 2 + r, math.pi),
                         (cx + w / 2 - r, cy - h / 2 + r, 3 * math.pi / 2)):
        for k in range(STEPS + 1):
            a = a0 + math.pi / 2 * k / STEPS
            pts.append((qx + r * math.cos(a), qy + r * math.sin(a)))
    return pts


def octagon(flats):
    """A regular octagon `flats` across, its flats square to u and v."""
    R = flats / 2 / math.cos(math.pi / 8)
    return [(R * math.cos(math.pi / 8 + k * math.pi / 4),
             R * math.sin(math.pi / 8 + k * math.pi / 4)) for k in range(8)]


def tube(length):
    """The eight walls and the two open ends as faces: the axis x, from the
    head end at 0 to the foot at `length`; wall k faces out at k x 45
    degrees from +z, toward +y, so wall 0 is the top."""
    R = FLATS_OUT / 2 / math.cos(math.pi / 8)
    ring = [(R * math.sin(math.pi / 8 + k * math.pi / 4),
             R * math.cos(math.pi / 8 + k * math.pi / 4)) for k in range(8)]
    verts = [(x, y, z) for y, z in ring for x in (0.0, length)]
    planes = [('head end', (-1, 0, 0), 0.0), ('foot end', (1, 0, 0), length)]
    for k in range(8):
        a = k * math.pi / 4
        planes.append(('top' if k == 0 else f'wall {k}',
                       (0, math.sin(a), math.cos(a)), FLATS_OUT / 2))
    return P.solid(planes, verts)


def build(L, holes, diams):
    """Every part. Each carries its frame as p.m, from its own (u, v,
    inward) to the world, for the 3D view."""
    pos = positions(L, holes)
    faces = tube(pos['length'])
    ends = [i for i, f in enumerate(faces) if f.name.endswith(' end')]
    ft = next(i for i, f in enumerate(faces) if f.name == 'top')
    shapes = [M.hole_shape(d) for d in diams]          # (across, along)
    spans = [(pos['emb'] - M.EMB_L / 2, pos['emb'] + M.EMB_L / 2)]
    spans += [(x - along / 2, x + along / 2) for x, (across, along) in zip(pos['holes'], shapes)]

    def owner_at(v):
        return next(i for i in ends if abs(P.dot(v, faces[i].n) - faces[i].off) < 1e-6)

    def force(E, s0, s1, o):
        """The ends are open, so a wall owns all of its end; and the top
        owns both its edges over every hole and the embouchure."""
        if E['a'] in ends or E['b'] in ends:
            return E['b'] if E['a'] in ends else E['a']
        if ft not in (E['a'], E['b']):
            return o
        x0, x1 = sorted((E['P'][0] + s0 * (E['Q'][0] - E['P'][0]) / E['L'],
                         E['P'][0] + s1 * (E['Q'][0] - E['P'][0]) / E['L']))
        if any(x1 > a - HOLE_CLEAR and x0 < b + HOLE_CLEAR for a, b in spans):
            return ft
        return o
    es = P.edges(faces)
    cellmap = {id(E): P.cells(E, owner_at, force) for E in es}
    parts, frames = [], {}
    num = iter(range(1, 100))
    for i, f in enumerate(faces):
        if i in ends:
            continue
        O = P.mul(f.n, f.off)
        fr = P.frame(f, U=(1.0, 0.0, 0.0), O=O)
        frames[i] = fr
        o = P.outline(i, f, fr, es, cellmap)
        p = Part(f.name if i != ft else 'top wall', next(num), o,
                 'fingers down both edges' if i != ft else
                 'the embouchure and the six holes; the lip plate glues over the embouchure')
        p.m = P.matrix(fr)
        p.face = i
        p.strokes = []
        parts.append(p)
    top = next(p for p in parts if p.face == ft)
    # on the top, u is x and v is -y; every opening is on the centre line
    top.cuts.append(('poly', stadium(pos['emb'], 0.0, M.EMB_L, M.EMB_W)))
    for x, (across, along) in zip(pos['holes'], shapes):
        top.cuts.append(('poly', stadium(x, 0.0, along, across)))
    tick = M.EMB_W / 2
    top.strokes = [[(pos['cork_face'], -tick), (pos['cork_face'], tick)],
                   [(pos['foot'], -tick), (pos['foot'], tick)]]
    top.note += ('; blue lines mark the cork\'s face and the foot the model '
                 'asks for, to trim toward')
    O, U, V, W = frames[ft]
    lip = Part('lip plate', next(num), rounded_rect(pos['emb'], 0.0, LIP_L, LIP_W, LIP_R),
               'glued on the top wall over the embouchure; file the far edge sharp and '
               'undercut both plies first')
    lip.cuts.append(('poly', stadium(pos['emb'], 0.0, M.EMB_L, M.EMB_W)))
    lip.m = P.matrix((P.sub(O, P.mul(W, T)), U, V, W))
    lip.strokes = []
    parts.append(lip)
    for k in range(CORK_DISCS):
        d = Part('cork disc', next(num), octagon(M.FLATS + INTERF),
                 'press-fit into the bore from the head end, up to the blue line')
        x = pos['cork_back'] + k * T
        d.m = P.matrix(((x, 0.0, 0.0), (0.0, 1.0, 0.0), (0.0, 0.0, 1.0), (1.0, 0.0, 0.0)))
        d.strokes = []
        parts.append(d)
    vol, polys = P.inner(faces)
    return parts, dict(faces=faces, ft=ft, ends=ends, edges=es, cellmap=cellmap,
                       frames=frames, pos=pos, spans=spans, shapes=shapes,
                       inner={i: [P.to2d(frames[i], q) for q in polys[i]] for i in frames},
                       end_poly=polys[ends[0]], V=vol)


def poly_area(pts):
    return abs(sum(x0 * y1 - x1 * y0 for (x0, y0), (x1, y1)
                   in zip(pts, pts[1:] + pts[:1]))) / 2


def checks(parts, info, L, holes, diams, rows, sheets):
    out = []

    def note(ok, what, detail):
        out.append((ok, what, detail))
    pos, ft = info['pos'], info['ft']
    worst = max(abs(c1) for nm, f1, c1, c2 in rows)
    note(worst < 1.0, 'the model holds the scale',
         f'worst {worst:.2f} cents of seven notes')
    # the bore's section, from the inside the walls make
    sec = [(p[1], p[2]) for p in info['end_poly']]
    a = poly_area(sorted(sec, key=lambda q: math.atan2(q[1], q[0])))
    note(abs(a / M.AREA - 1) < 1e-6, 'the bore is the octagon the model plays',
         f'{a:.1f}mm^2 inside, {M.FLATS:g}mm across the flats')
    inner_w = min(max(v for u, v in info['inner'][ft]) - min(v for u, v in info['inner'][ft]),
                  M.FACE_IN + 1e-9)
    widest = max([M.EMB_W] + [s[0] for s in info['shapes']])
    note(widest / 2 + 0.3 <= inner_w / 2, 'every hole and the embouchure open into the bore',
         f'widest {widest:.2f}mm across, the top\'s inside {inner_w:.2f}mm')
    bad = 0
    for E in info['edges']:
        if ft not in (E['a'], E['b']) or set((E['a'], E['b'])) & set(info['ends']):
            continue
        for s0, s1, o in info['cellmap'][id(E)]:
            x0, x1 = sorted((E['P'][0] + s0 * (E['Q'][0] - E['P'][0]) / E['L'],
                             E['P'][0] + s1 * (E['Q'][0] - E['P'][0]) / E['L']))
            if o != ft and any(x1 > a_ and x0 < b_ for a_, b_ in info['spans']):
                bad += 1
    note(bad == 0, 'the top keeps both its edges beside every opening',
         f'{bad} cells of a neighbour beside a hole')
    sp = sorted(info['spans'])
    gap = min(b[0] - a[1] for a, b in zip(sp, sp[1:]))
    note(gap >= HOLE_GAP, 'ply left between neighbouring openings', f'{gap:.1f}mm at the least')
    hands = [holes[0] - holes[1], holes[1] - holes[2], holes[3] - holes[4], holes[4] - holes[5]]
    note(max(hands) <= 36.0, 'each hand\'s holes are in reach',
         f'{max(hands):.1f}mm apart within a hand, {holes[2] - holes[3]:.1f}mm between the hands')
    web = FACE_OUT - 2 * P.clear(135.0)
    note(web >= 2 * MIN_WEB, 'a wall keeps ply between its two joints',
         f'{web:.2f}mm of {FACE_OUT:.2f}mm where both neighbours own the edge')
    note(pos['cork_face'] + M.CORK == pos['emb'], 'the cork stands where the model puts it',
         f'its face {M.CORK:g}mm from the embouchure\'s centre')
    note(pos['foot'] + M.FOOT_SPARE == pos['length'] and M.FOOT_SPARE > 0,
         'the foot is cut long, to trim to pitch', f'{M.FOOT_SPARE:g}mm past the model\'s foot')
    note(LIP_W > FACE_OUT + 2 * 5 and LIP_L > M.EMB_L + 2 * 10,
         'the lip plate leaves the lip somewhere to rest',
         f'{LIP_W:g} x {LIP_L:g}mm on a {FACE_OUT:.1f}mm wall')
    longest = max(max(p.w, p.h) for p in parts)
    note(longest + KERF + 2 * MARGIN <= BED[0], 'the longest part fits the bed',
         f'{longest + KERF:.1f}mm with the kerf, against {BED[0] - 2 * MARGIN:g}')
    note(all(s['w'] <= BED[0] and s['h'] <= BED[1] for s in sheets), 'every sheet fits the bed',
         f'{len(sheets)} sheet(s), largest {max(s["w"] for s in sheets):.0f} x '
         f'{max(s["h"] for s in sheets):.0f}mm')
    placed = sum(len(s['parts']) for s in sheets)
    note(placed == len(parts), 'every part is on a sheet', f'{placed} of {len(parts)}')
    return out


def svg_sheet(k, n, s, what):
    orange, black, blue = [], [], []
    for p, sx, sy in s['parts']:
        dx, dy = sx - p.x0 + KERF / 2, sy - p.y0 + KERF / 2
        mv = lambda pts: [(x + dx, y + dy) for x, y in pts]
        black.append(path(mv(offset(p.outline, KERF / 2))))
        for c in p.cuts:
            orange.append(path(mv(offset(c[1], -KERF / 2))))
        for st in p.strokes:
            blue.append(path(mv(st), close=False))
    desc = (f'1 user unit = 1mm. Six-hole transverse flute in D, octagonal bore, sheet {k} '
            f'of {n}: {what}. 3mm ply, drawn for a {KERF:g}mm kerf. Blue {BLUE} engraves '
            f'two lines on the top wall, which lies outside face up: where the cork\'s face '
            f'goes, and where the model puts the foot, to trim toward. Then orange {ORANGE} '
            f'cuts the embouchure and the holes, and black {BLACK} frees the parts.')
    g = lambda col, items: (
        f'  <g fill="none" stroke="{col}" stroke-width="0.2">\n'
        + ''.join(f'    <path d="{i}"/>\n' for i in items) + '  </g>\n')
    return (f'<?xml version="1.0" encoding="utf-8"?>\n'
            f'<svg xmlns="http://www.w3.org/2000/svg" width="{s["w"]:.2f}mm" '
            f'height="{s["h"]:.2f}mm" viewBox="0 0 {s["w"]:.2f} {s["h"]:.2f}">\n'
            f'<title>Octagonal flute in D, sheet {k} of {n}</title>\n'
            f'<desc>{desc}</desc>\n'
            + g(BLUE, blue) + g(ORANGE, orange) + g(BLACK, black) + '</svg>\n')


def main():
    global KERF
    out, write = None, True
    for x in sys.argv[1:]:
        if x == '--no-write':
            write = False
        elif x.startswith('--out='):
            out = x.split('=', 1)[1]
        elif x.startswith('--mouth='):
            M.MOUTH = float(x.split('=', 1)[1])
        elif x.startswith('--kerf='):
            KERF = SF.set_kerf(x)
        else:
            raise SystemExit(f'error: {x} is not a flag this generator reads. '
                             f'It takes --out=, --mouth=, --kerf= and --no-write.')
    if write and not out:
        raise SystemExit('error: --out=DIR, or --no-write to run the checks only.')
    b = M.Bore()
    L, holes, diams = M.design(b)
    rows = M.report(b, L, holes, diams)
    parts, info = build(L, holes, diams)
    sheets = pack(parts)
    res = checks(parts, info, L, holes, diams, rows, sheets)
    pos = info['pos']
    print(f'octagonal flute in D: {M.FLATS:g}mm across the flats inside, walls '
          f'{pos["length"]:.1f}mm long; embouchure {pos["emb"]:.1f}mm from the head end')
    for h, x, d, (across, along) in zip(M.HOLES, pos['holes'], diams, info['shapes']):
        cut = f'ø{d:.2f}' if across == along else f'{across:g} x {along:.2f}'
        print(f'  {h}  {x:6.1f}mm from the head end  {cut}')
    bad = 0
    for ok, what, detail in res:
        print(f'  {"pass" if ok else "FAIL"}  {what:52} {detail}')
        bad += not ok
    if bad:
        print(f'\n  {bad} check(s) failed. Nothing written.')
        return 1
    if write:
        os.makedirs(out, exist_ok=True)
        for k, s in enumerate(sheets, 1):
            what = ', '.join(sorted({p.name for p, _, _ in s['parts']}))
            f = os.path.join(out, f'octagonal-flute-D4-sheet{k}-cut-files.svg')
            open(f, 'w').write(svg_sheet(k, len(sheets), s, what))
            print(f'  wrote {os.path.basename(f)}  {s["w"]:.0f} x {s["h"]:.0f}mm, '
                  f'{len(s["parts"])} parts')
    return 0


if __name__ == '__main__':
    sys.exit(main())
