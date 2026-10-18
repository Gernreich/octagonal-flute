#!/usr/bin/env python3
"""A six-hole transverse flute in D, the Irish or simple-system flute, with
a regular octagonal bore, as a transmission-matrix model: how long the bore
runs past the embouchure, and where the six finger holes go.

    python3 flute_model.py               # the design, and how it plays
    python3 flute_model.py --mouth=1.2   # scale the embouchure's correction
    python3 flute_model.py --cork=17     # another cork distance, mm
    python3 flute_model.py --heard=285 --length=537
                                         # the --mouth a cut flute implies

All closed it sounds D4; opening the holes from the foot up gives E4, F#4,
G4, A4, B4 and C#5, and overblown, the octave above. C natural is a cross
fingering, as on any six-hole flute.

THE BORE is a regular octagon FLATS across inside, 19mm: 299mm^2, the area of
a 19.5mm round bore, near the 19mm of an Irish flute's. Its loss is taken on
the hydraulic radius 2A/P, which for any regular polygon is half the flats.

The model, from the foot up:

  * the bore is a lossy tube, Benade's wall loss on the hydraulic radius;
  * the foot radiates as an unflanged open end of the same area;
  * each finger hole is Keefe's (1990) tone hole, open or closed, its chimney
    the 3mm wall, as on the square flute;
  * at the top, the embouchure hole is a short neck, CHIMNEY deep (the wall
    and the lip plate), into open air, with the cork's closed stub beside
    it. The jet drives the hole, so the flute sounds where the reactance of
    the neck in series with the bore and the stub in parallel passes through
    zero, rising.

THE EMBOUCHURE IS THE UNCALIBRATED PART. The player's lip covers part of the
hole, by an amount that changes with how they play, so its area is taken as
COVER of the cut hole and its end correction is physics-based, not measured.
--mouth= scales the correction. The foot is cut FOOT_SPARE long, to be
trimmed to pitch: a model that guessed short would leave a flute too short
to fix, so the guess is made long.

Moving the correction moves the holes almost as far as the foot: at
--mouth=0.75 the foot is 8.9mm longer and the holes 7.8 to 8.6mm lower. So
trimming the foot does not tune the holes, as on the square flute; each is
tuned by filing it longer along the bore, which raises its note, and every
hole is cut with room for that (FILE).

THE CORK sets how well the second octave lines up with the first: nearer the
embouchure, the octaves widen and the top comes easier; further, the low
octave is stronger. The cork here is press-fit discs, so it can be moved;
CORK is where it starts.
"""
import cmath
import math
import sys

# --- air, warm from the breath ----------------------------------------------
T_C = 25.0
C = 331.3 * math.sqrt(1 + T_C / 273.15)          # 346.1 m/s
RHO = 1.184

# --- the tune ----------------------------------------------------------------
D4 = 440.0 * 2 ** (-7 / 12)                       # 293.66Hz
NOTES = ['D4', 'E4', 'F#4', 'G4', 'A4', 'B4', 'C#5']
SEMIS = [0, 2, 4, 5, 7, 9, 11]
TARGETS = [D4 * 2 ** (s / 12) for s in SEMIS]
HOLES = ['R3', 'R2', 'R1', 'L3', 'L2', 'L1']      # foot-most first

# --- the bore and the head (mm) ---------------------------------------------
T = 3.0                            # the ply
FLATS = 19.0                       # the octagon across its flats, inside
AREA = 2 * (math.sqrt(2) - 1) * FLATS ** 2
FACE_IN = FLATS * math.tan(math.pi / 8)          # one wall's inside width
EMB_W, EMB_L = 7.0, 12.0           # embouchure: across x along the bore,
                                   # a stadium: the wall's inside is 7.87 wide
CHIMNEY = 2 * T                    # the wall and the lip plate
COVER = 0.75                       # of the hole the lip leaves open
MOUTH = 1.0                        # scales the embouchure's end correction
CORK = 20.0                        # embouchure centre to the cork's face
FOOT_SPARE = 25.0                  # cut this much long, trim to pitch
# A finger hole is round up to HOLE_W, the most the wall's inside takes
# with a little to spare, and past that a stadium HOLE_W across and longer
# along the bore, up to HOLE_LONG: a finger pad covers 7 x 12mm easily. The
# model reads a stadium as the round hole of its area.
HOLE_W, HOLE_LONG = 7.0, 12.0
SPACING = 34.0                     # centre to centre within a hand
ANCHORS = (2, 6)                   # R2 and L1, by the note each opens
D_MIN = 4.0


def stadium_area(w, l):
    return w * (l - w) + math.pi * (w / 2) ** 2


def hole_shape(d):
    """The hole a model diameter d is cut as: (across, along)."""
    if d <= HOLE_W:
        return d, d
    r = HOLE_W / 2
    return HOLE_W, HOLE_W + (math.pi * d * d / 4 - math.pi * r * r) / HOLE_W


D_MAX = math.sqrt(4 * stadium_area(HOLE_W, HOLE_LONG) / math.pi)
# the anchors are cut FILE short of HOLE_LONG along the bore, so even the
# biggest hole can be filed longer, and sharper, once the flute is played
FILE = 1.0
D_ANCHOR = math.sqrt(4 * stadium_area(HOLE_W, HOLE_LONG - FILE) / math.pi)


def emb_area():
    return stadium_area(EMB_W, EMB_L)


class Bore:
    """The octagon: its area, its equivalent radius and its loss radius (m)."""

    def __init__(self, flats=FLATS):
        self.A = 2 * (math.sqrt(2) - 1) * (flats / 1000) ** 2
        self.rh = flats / 2000
        self.a = math.sqrt(self.A / math.pi)

    def k(self, f):
        return 2 * math.pi * f / C - 1j * 3.0e-5 * math.sqrt(f) / self.rh

    def z0(self):
        return RHO * C / self.A


def tube(b, f, L):
    k, z = b.k(f), b.z0()
    c, s = cmath.cos(k * L), cmath.sin(k * L)
    return ((c, 1j * z * s), (1j * s / z, c))


def mul(m, n):
    return ((m[0][0] * n[0][0] + m[0][1] * n[1][0], m[0][0] * n[0][1] + m[0][1] * n[1][1]),
            (m[1][0] * n[0][0] + m[1][1] * n[1][0], m[1][0] * n[0][1] + m[1][1] * n[1][1]))


def hole(b, f, d, t, is_open):
    """Keefe's tone hole: series half, shunt, series half."""
    k = 2 * math.pi * f / C
    r, a = d / 2000, b.a
    t = t / 1000
    dl = r / a
    zh = RHO * C / (math.pi * r * r)
    if is_open:
        te = ((1 / k) * math.tan(k * t) + r * (1.40 - 0.58 * dl ** 2)) / \
             (1 - 0.61 * k * r * math.tan(k * t))
        zs = zh * (1j * k * te + 0.25 * (k * r) ** 2)
        ta = 0.47 * r * dl ** 4 / (math.tanh(1.84 * t / r) + 0.62 * dl ** 2 + 0.64 * dl)
    else:
        tc = t + r * dl * (1 + 0.207 * dl ** 3) / 8
        zs = -1j * zh / math.tan(k * tc)
        ta = 0.47 * r * dl ** 4 / (1 / math.tanh(1.84 * t / r) + 0.62 * dl ** 2 + 0.64 * dl)
    za = -1j * zh * k * ta
    half = ((1, za / 2), (0, 1))
    return mul(mul(half, ((1, 0), (1 / zs, 1))), half)


def emb_z(f):
    """The embouchure as a neck CHIMNEY deep, of the area the lip leaves
    open, flanged both ends (0.82 a each: the lip plate outside, the bore
    wall in), the correction scaled by MOUTH."""
    w = 2 * math.pi * f
    Ae = COVER * emb_area() / 1e6
    ae = math.sqrt(Ae / math.pi)
    leff = CHIMNEY / 1000 + MOUTH * 2 * 0.82 * ae
    k = w / C
    return RHO * C / Ae * (0.5 * (k * ae) ** 2) + 1j * w * RHO * leff / Ae


def reactance(b, f, L, holes, n_open, diams):
    """Im(Z) the jet sees, for L mm of bore from the embouchure centre to the
    foot with holes at their distances from the embouchure centre, foot-most
    first, the lowest n_open of them open."""
    k, z = b.k(f), b.z0()
    ka = 2 * math.pi * f / C * b.a
    zr = z * (0.25 * ka ** 2 + 1j * 0.6133 * ka)
    chain, pos = [], L
    for i, x in enumerate(holes):
        chain.append(('tube', pos - x))
        chain.append(('hole', (i < n_open, diams[i])))
        pos = x
    chain.append(('tube', pos))
    m = ((1, 0), (0, 1))
    for kind, v in reversed(chain):
        m = mul(m, tube(b, f, v / 1000) if kind == 'tube' else hole(b, f, v[1], T, v[0]))
    zin = (m[0][0] * zr + m[0][1]) / (m[1][0] * zr + m[1][1])
    stub = -1j * z / cmath.tan(k * (CORK / 1000))
    return (emb_z(f) + 1 / (1 / zin + 1 / stub)).imag


def sounding(b, L, holes, n_open, near, diams, span=1.4):
    """The playing frequency nearest `near`: a zero crossing of the
    reactance, rising through zero, found by scanning then bisecting."""
    lo, hi = near / span, near * span
    X = lambda f: reactance(b, f, L, holes, n_open, diams)
    prev_f, prev_x = lo, X(lo)
    best = None
    for i in range(1, 81):
        f = lo + (hi - lo) * i / 80
        x = X(f)
        if prev_x < 0 <= x:
            a, c = prev_f, f
            for _ in range(40):
                mid = (a + c) / 2
                if X(mid) < 0:
                    a = mid
                else:
                    c = mid
            root = (a + c) / 2
            if best is None or abs(root - near) < abs(best - near):
                best = root
        prev_f, prev_x = f, x
    if best is None:
        raise ValueError(f'no resonance within 40% of {near:.1f}Hz')
    return best


def bisect(fn, target, lo, hi, rising=False, n=40):
    """x in [lo, hi] with fn(x) == target. fn falls as x grows, unless
    rising. Raises when the target is out of reach."""
    g = (lambda x: -fn(x)) if rising else fn
    t = -target if rising else target
    if not (g(hi) <= t <= g(lo)):
        raise ValueError(f'out of reach between {lo:.1f} and {hi:.1f}')
    for _ in range(n):
        mid = (lo + hi) / 2
        if g(mid) > t:
            lo = mid
        else:
            hi = mid
    return (lo + hi) / 2


def design(b=None):
    """The bore's length from the embouchure centre to the foot, the six
    holes' positions from the embouchure centre, foot-most first, and their
    diameters, iterated until each closed hole's effect on the others
    settles.

    Laid out for the hands: one hole in each hand, the ANCHORS, is the
    largest the wall takes less FILE, D_ANCHOR, and its place is solved; the hand's
    other holes stand SPACING apart from it and their SIZES are solved.
    The anchors are R2, the lower hand's middle, and L1, the upper hand's
    top: each is the hole that, at full size, still leaves its neighbours
    room to be smaller. Anchored on L2, L1 would have had to stand lower
    than a full-size hole can for C#5. A smaller
    hole has to sit nearer the embouchure for its note, which is what lets
    them close up: all 7mm round, the six fall 56, 17, 55, 42 and 37mm apart,
    the lower hand far too wide for a hand."""
    b = b or Bore()
    L = 540.0
    FRAC = (0.82, 0.76, 0.69, 0.61, 0.54, 0.47)
    holes = [L * r for r in FRAC]
    diams = [D_ANCHOR] * 6
    for it in range(6):
        L = bisect(lambda x: sounding(b, x, holes, 0, D4, diams), D4, 400, 700)
        if it == 0:
            holes = [L * r for r in FRAC]
            L = bisect(lambda x: sounding(b, x, holes, 0, D4, diams), D4, 400, 700)
        # the anchors first, so the holes either side are sized where they
        # will stand
        for n in (2, 6, 1, 3, 4, 5):
            i = n - 1
            def f_pos(x, i=i, n=n):
                hs = holes[:]
                hs[i] = x
                return sounding(b, L, hs, n, TARGETS[n], diams)
            def f_dia(d, i=i, n=n):
                ds = diams[:]
                ds[i] = d
                return sounding(b, L, holes, n, TARGETS[n], ds)
            if n in ANCHORS:
                lo = holes[i + 1] + 0.5 if n < 6 else 60.0
                hi = holes[i - 1] - 0.5
                holes[i] = bisect(f_pos, TARGETS[n], lo, hi)
                hand = range(0, 3) if n <= 3 else range(3, 6)
                for j in hand:
                    holes[j] = holes[i] + (i - j) * SPACING
            else:
                diams[i] = bisect(f_dia, TARGETS[n], D_MIN, D_ANCHOR, rising=True)
    return L, holes, diams


def cents(f, ref):
    return 1200 * math.log2(f / ref)


def report(b, L, holes, diams):
    """Each note's error, and the octave each fingering overblows to."""
    rows = []
    for n in range(7):
        f1 = sounding(b, L, holes, n, TARGETS[n], diams)
        f2 = sounding(b, L, holes, n, 2 * TARGETS[n], diams, span=1.2)
        rows.append((NOTES[n], f1, cents(f1, TARGETS[n]), cents(f2, 2 * f1)))
    return rows


def end_correction(b, L):
    """How much longer than the bore the flute plays, all closed: the
    embouchure's and the foot's corrections and the cork's stub together."""
    return 1000 * C / (2 * D4) - L


def mouth_for(heard, length):
    """The MOUTH that makes a flute whose embouchure centre is `length` mm
    from its foot sound `heard` Hz with every hole closed. Closed holes
    barely move the note, so they are left where the design puts them."""
    global MOUTH
    b = Bore()
    L, holes, diams = design(b)
    saved = MOUTH

    def f(m):
        global MOUTH
        MOUTH = m
        return sounding(b, length, holes, 0, heard, diams)
    try:
        return bisect(f, heard, 0.05, 4.0)
    finally:
        MOUTH = saved


def main():
    global MOUTH, CORK
    heard = length = None
    for x in sys.argv[1:]:
        if x.startswith('--mouth='):
            MOUTH = float(x.split('=', 1)[1])
        elif x.startswith('--cork='):
            CORK = float(x.split('=', 1)[1])
        elif x.startswith('--heard='):
            heard = float(x.split('=', 1)[1])
        elif x.startswith('--length='):
            length = float(x.split('=', 1)[1])
        else:
            raise SystemExit(f'error: {x} is not a flag this model reads. It takes '
                             f'--mouth=, --cork=, --heard= and --length=.')
    if (heard is None) != (length is None):
        raise SystemExit('error: --heard= and --length= go together: the closed note in Hz, '
                         'and the embouchure centre to the foot in mm, as cut.')
    if heard is not None:
        try:
            m = mouth_for(heard, length)
        except ValueError:
            raise SystemExit(f'error: no --mouth between 0.05 and 4 makes {length:g}mm '
                             f'sound {heard:g}Hz. Check the length is from the '
                             f'embouchure\'s centre to the foot.')
        print(f'{heard:g}Hz from {length:g}mm of bore: --mouth={m:.3f}. Redraw with\n'
              f'  python3 octagonal_flute.py --mouth={m:.3f} --out=mine')
        return 0
    b = Bore()
    L, holes, diams = design(b)
    print(f'D4 = {D4:.2f}Hz, air at {T_C:g}C ({C:.1f} m/s); octagon {FLATS:g}mm across '
          f'its flats, {AREA:.0f}mm^2 (a {2000 * b.a:.1f}mm round bore\'s area); '
          f'embouchure {EMB_W:g} x {EMB_L:g}mm, {CHIMNEY:g}mm deep, {COVER:g} uncovered, '
          f'mouth x{MOUTH:g}; cork {CORK:g}mm')
    print(f'embouchure centre to foot {L:.1f}mm (cut {L + FOOT_SPARE:.1f}, trimmed to pitch); '
          f'the corrections add {end_correction(b, L):.1f}mm')
    print('holes from the embouchure centre, foot-most first:')
    prev = L
    for h, x, d in zip(HOLES, holes, diams):
        w, l = hole_shape(d)
        cut = f'ø{d:.2f}' if w == l else f'{w:g} x {l:.2f} stadium (ø{d:.2f})'
        print(f'  {h}  {x:6.1f}mm  {cut:28}  {prev - x:5.1f}mm above the one below')
        prev = x
    print('note   Hz      cents  octave above, cents off 2:1')
    for nm, f1, c1, c2 in report(b, L, holes, diams):
        print(f'  {nm:4} {f1:7.2f} {c1:+6.1f}  {c2:+6.1f}')
    return 0


if __name__ == '__main__':
    sys.exit(main())
