//! Cut files for a six-hole transverse flute in D, its bore a long regular
//! octagon of 3mm ply, finger-jointed down every edge.
//!
//!     target/release/octagonal_flute --no-write       # the checks, write nothing
//!     target/release/octagonal_flute --out=DIR        # the sheet into DIR
//!     target/release/octagonal_flute --mouth=1.2 ...  # the embouchure's correction, as the model
//!     target/release/octagonal_flute --kerf=0.18 ...  # for a laser with that kerf
//!
//! Rust since 2026-11-07 (octagonal_flute.py until then; the move changed no
//! byte of any cut file or page). The acoustics are flute_model.rs's: how
//! long the bore runs past the embouchure, and where the six holes go and
//! how big each is. This file only builds what that asks for.
//!
//! THE TUBE is eight walls, each a strip as long as the flute, joined edge
//! to edge at 135 degrees by polyhedron.rs's joint from the four-hole
//! ocarinas: in its own cells a wall runs out to the edge line, in its
//! neighbour's it stops clear(135) = 2.12mm short, so every joint closes on
//! a line. Both ends are open; the walls run square to them. The wall on
//! top, the one the holes and the embouchure go through, takes every cell of
//! both its edges wherever a hole is, so each hole has the wall's whole
//! width round it.
//!
//! FROM THE HEAD END:
//!
//!   head     HEAD_PAST of bare tube, past the cork
//!   cork     CORK_DISCS octagonal discs, press-fit into the bore, their
//!            face flute_model's CORK from the embouchure's centre. A blue
//!            line on the top marks where that face goes; push the cork
//!            further in or out to move the octaves (see flute_model.rs)
//!   emb.     the embouchure, a stadium EMB_W across and EMB_L along,
//!            through the top wall and the lip plate glued over it, so the
//!            chimney is the two plies, 6mm, a wooden flute's depth
//!   holes    the six, on the top wall's centre line, round where they are
//!            small and stadia, longer along the flute, where they are not
//!   foot     open, cut FOOT_SPARE long; a blue line marks where the model
//!            puts the end, and the foot is trimmed toward it to pitch
//!
//! THE LIP PLATE is wider than the top wall, which is only 10.4mm across
//! outside, so the lower lip has something to rest on. Before it is glued,
//! file the embouchure's far edge, the one the breath strikes, sharp, and
//! undercut both plies' walls a little, wider inside than out, as on any
//! wooden flute.
//!
//! The kerf, the sheet packing and the inks are the square flute's, from
//! square_flute.rs, and the joint and the mitred offset are the four-hole
//! ocarinas', from polyhedron.rs and four_hole_ocarinas.rs: all three
//! repositories are expected side by side, with swept-curve for the path
//! writer.
pub mod flute_model;
pub mod page_assets;

use flute_model as m;
use four_hole_ocarinas::polyhedron::{self as p3, Edge, Face, Frame, V3};
use four_hole_ocarinas::{offset, OCut, OPart, P2};
use lasermade_tools::py;
use ribbon_bore::path;
use square_flute::{kerf, pack, Sheet, BED, BLACK, BLUE, MARGIN, MIN_WEB, ORANGE, T};
use std::f64::consts::PI;

/// the octagon across its flats, outside
pub fn flats_out() -> f64 {
    m::FLATS + 2.0 * T
}
/// one wall's outside width
pub fn face_out() -> f64 {
    flats_out() * py::mtan(PI / 8.0)
}
pub const HEAD_PAST: f64 = 6.0; // bare tube past the cork's back
pub const CORK_DISCS: usize = 3;
pub const INTERF: f64 = square_flute::INTERF_ACROSS; // the cork's press-fit, across its flats
pub const HOLE_CLEAR: f64 = 3.0; // the top keeps its joint this far past a hole
pub const HOLE_GAP: f64 = 3.0; // ply between neighbouring holes
// the lip plate, across x along, corner radius
pub const LIP_W: f64 = 26.0;
pub const LIP_L: f64 = 40.0;
pub const LIP_R: f64 = 9.0;
pub const STEPS: usize = 16; // points on a quarter circle

/// Where everything stands along the walls, from the head end (mm).
#[derive(Clone, Debug)]
pub struct Pos {
    pub cork_back: f64,
    pub cork_face: f64,
    pub emb: f64,
    pub holes: Vec<f64>,
    pub foot: f64,
    pub length: f64,
}

pub fn positions(l: f64, holes: &[f64]) -> Pos {
    let cork_back = HEAD_PAST;
    let cork_face = cork_back + CORK_DISCS as f64 * T;
    let emb = cork_face + m::cork();
    Pos { cork_back, cork_face, emb, holes: holes.iter().map(|h| emb + h).collect(), foot: emb + l, length: emb + l + m::FOOT_SPARE }
}

/// A stadium centred (cx, cy), `along` long in x and `across` wide in y; a
/// circle when the two are equal.
pub fn stadium(cx: f64, cy: f64, along: f64, across: f64) -> Vec<P2> {
    let r = across / 2.0;
    let s = (along - across) / 2.0;
    let mut pts = vec![];
    for k in 0..=2 * STEPS {
        let a = -PI / 2.0 + PI * k as f64 / (2 * STEPS) as f64;
        pts.push((cx + s + r * py::mcos(a), cy + r * py::msin(a)));
    }
    for k in 0..=2 * STEPS {
        let a = PI / 2.0 + PI * k as f64 / (2 * STEPS) as f64;
        pts.push((cx - s + r * py::mcos(a), cy + r * py::msin(a)));
    }
    let mut out: Vec<P2> = vec![];
    for p in pts {
        if out.last().map_or(true, |q: &P2| py::hypot(p.0 - q.0, p.1 - q.1) > 1e-9) {
            out.push(p);
        }
    }
    let (f, l) = (out[0], out[out.len() - 1]);
    if py::hypot(f.0 - l.0, f.1 - l.1) < 1e-9 {
        out.pop();
    }
    out
}

/// A rectangle w along x and h across y, centred, corners rounded r.
pub fn rounded_rect(cx: f64, cy: f64, w: f64, h: f64, r: f64) -> Vec<P2> {
    let mut pts = vec![];
    for (qx, qy, a0) in [
        (cx + w / 2.0 - r, cy + h / 2.0 - r, 0.0),
        (cx - w / 2.0 + r, cy + h / 2.0 - r, PI / 2.0),
        (cx - w / 2.0 + r, cy - h / 2.0 + r, PI),
        (cx + w / 2.0 - r, cy - h / 2.0 + r, 3.0 * PI / 2.0),
    ] {
        for k in 0..=STEPS {
            let a = a0 + PI / 2.0 * k as f64 / STEPS as f64;
            pts.push((qx + r * py::mcos(a), qy + r * py::msin(a)));
        }
    }
    pts
}

/// A regular octagon `flats` across, its flats square to u and v.
pub fn octagon(flats: f64) -> Vec<P2> {
    let r = flats / 2.0 / py::mcos(PI / 8.0);
    (0..8)
        .map(|k| {
            let a = PI / 8.0 + k as f64 * PI / 4.0;
            (r * py::mcos(a), r * py::msin(a))
        })
        .collect()
}

/// The eight walls and the two open ends as faces: the axis x, from the
/// head end at 0 to the foot at `length`; wall k faces out at k x 45
/// degrees from +z, toward +y, so wall 0 is the top.
pub fn tube(length: f64) -> Vec<Face> {
    let r = flats_out() / 2.0 / py::mcos(PI / 8.0);
    let mut verts = vec![];
    for k in 0..8 {
        let a = PI / 8.0 + k as f64 * PI / 4.0;
        let (y, z) = (r * py::msin(a), r * py::mcos(a));
        for x in [0.0, length] {
            verts.push([x, y, z]);
        }
    }
    let mut planes = vec![("head end".to_string(), [-1.0, 0.0, 0.0], 0.0), ("foot end".to_string(), [1.0, 0.0, 0.0], length)];
    for k in 0..8 {
        let a = k as f64 * PI / 4.0;
        planes.push((if k == 0 { "top".to_string() } else { format!("wall {k}") }, [0.0, py::msin(a), py::mcos(a)], flats_out() / 2.0));
    }
    p3::solid(&planes, &verts)
}

/// What build() says beside its parts.
pub struct Info {
    pub faces: Vec<Face>,
    pub ft: usize,
    pub ends: Vec<usize>,
    pub edges: Vec<Edge>,
    pub cellmap: Vec<Vec<(f64, f64, usize)>>,
    pub frames: Vec<(usize, Frame)>,
    pub pos: Pos,
    pub spans: Vec<(f64, f64)>,
    pub shapes: Vec<(f64, f64)>,
    pub inner: Vec<(usize, Vec<P2>)>,
    pub end_poly: Vec<V3>,
    pub v: f64,
}

impl Info {
    pub fn inner_of(&self, i: usize) -> &[P2] {
        &self.inner.iter().find(|e| e.0 == i).unwrap().1
    }
    pub fn frame_of(&self, i: usize) -> &Frame {
        &self.frames.iter().find(|e| e.0 == i).unwrap().1
    }
}

/// An edge's cell, s0 to s1 along it, as x from the head end, low end first.
fn xspan(e: &Edge, s0: f64, s1: f64) -> (f64, f64) {
    let a = e.p[0] + s0 * (e.q[0] - e.p[0]) / e.l;
    let b = e.p[0] + s1 * (e.q[0] - e.p[0]) / e.l;
    if a <= b { (a, b) } else { (b, a) }
}

/// Every part. Each carries its frame as p.m, from its own (u, v, inward) to
/// the world, for the 3D view.
pub fn build(l: f64, holes: &[f64], diams: &[f64]) -> (Vec<OPart>, Info) {
    let pos = positions(l, holes);
    let faces = tube(pos.length);
    let ends: Vec<usize> = (0..faces.len()).filter(|&i| faces[i].name.ends_with(" end")).collect();
    let ft = faces.iter().position(|f| f.name == "top").unwrap();
    let shapes: Vec<(f64, f64)> = diams.iter().map(|&d| m::hole_shape(d)).collect(); // (across, along)
    let mut spans = vec![(pos.emb - m::EMB_L / 2.0, pos.emb + m::EMB_L / 2.0)];
    spans.extend(pos.holes.iter().zip(&shapes).map(|(x, (_, along))| (x - along / 2.0, x + along / 2.0)));

    let owner_at = |v: V3| *ends.iter().find(|&&i| (p3::dot(v, faces[i].n) - faces[i].off).abs() < 1e-6).unwrap();
    // The ends are open, so a wall owns all of its end; and the top owns
    // both its edges over every hole and the embouchure.
    let force = |e: &Edge, s0: f64, s1: f64, o: usize| -> usize {
        if ends.contains(&e.a) || ends.contains(&e.b) {
            return if ends.contains(&e.a) { e.b } else { e.a };
        }
        if ft != e.a && ft != e.b {
            return o;
        }
        let (x0, x1) = xspan(e, s0, s1);
        if spans.iter().any(|&(a, b)| x1 > a - HOLE_CLEAR && x0 < b + HOLE_CLEAR) {
            return ft;
        }
        o
    };
    let es = p3::edges(&faces);
    let cellmap: Vec<Vec<(f64, f64, usize)>> = es.iter().map(|e| p3::cells(e, owner_at, Some(&force))).collect();
    let (mut parts, mut frames): (Vec<OPart>, Vec<(usize, Frame)>) = (vec![], vec![]);
    let mut num = 0;
    for (i, f) in faces.iter().enumerate() {
        if ends.contains(&i) {
            continue;
        }
        let o = p3::mul(f.n, f.off);
        let fr = p3::frame(f, Some([1.0, 0.0, 0.0]), Some(o));
        frames.push((i, fr));
        let ol = p3::outline(i, f, &fr, &es, &cellmap);
        num += 1;
        let mut p = OPart::new(
            if i != ft { &f.name } else { "top wall" },
            num,
            ol,
            if i != ft { "fingers down both edges" } else { "the embouchure and the six holes; the lip plate glues over the embouchure" },
        );
        p.m = Some(p3::matrix(&fr));
        p.face = Some(i);
        parts.push(p);
    }
    let top = parts.iter_mut().find(|p| p.face == Some(ft)).unwrap();
    // on the top, u is x and v is -y; every opening is on the centre line
    top.cuts.push(OCut::Rect(stadium(pos.emb, 0.0, m::EMB_L, m::EMB_W)));
    for (x, (across, along)) in pos.holes.iter().zip(&shapes) {
        top.cuts.push(OCut::Rect(stadium(*x, 0.0, *along, *across)));
    }
    let tick = m::EMB_W / 2.0;
    top.strokes = vec![vec![(pos.cork_face, -tick), (pos.cork_face, tick)], vec![(pos.foot, -tick), (pos.foot, tick)]];
    top.note += "; blue lines mark the cork's face and the foot the model asks for, to trim toward";
    let Frame { o, u, v, w } = frames.iter().find(|e| e.0 == ft).unwrap().1;
    num += 1;
    let mut lip = OPart::new(
        "lip plate",
        num,
        rounded_rect(pos.emb, 0.0, LIP_L, LIP_W, LIP_R),
        "glued on the top wall over the embouchure; file the far edge sharp and undercut both plies first",
    );
    lip.cuts.push(OCut::Rect(stadium(pos.emb, 0.0, m::EMB_L, m::EMB_W)));
    lip.m = Some(p3::matrix(&Frame { o: p3::sub(o, p3::mul(w, T)), u, v, w }));
    parts.push(lip);
    for k in 0..CORK_DISCS {
        num += 1;
        let mut d = OPart::new("cork disc", num, octagon(m::FLATS + INTERF), "press-fit into the bore from the head end, up to the blue line");
        let x = pos.cork_back + k as f64 * T;
        d.m = Some(p3::matrix(&Frame { o: [x, 0.0, 0.0], u: [0.0, 1.0, 0.0], v: [0.0, 0.0, 1.0], w: [1.0, 0.0, 0.0] }));
        parts.push(d);
    }
    let (vol, polys) = p3::inner(&faces);
    let inner = frames.iter().map(|(i, fr)| (*i, polys[*i].iter().map(|&q| p3::to2d(fr, q)).collect())).collect();
    let end_poly = polys[ends[0]].clone();
    (parts, Info { faces, ft, ends, edges: es, cellmap, frames, pos, spans, shapes, inner, end_poly, v: vol })
}

pub fn poly_area(pts: &[P2]) -> f64 {
    let n = pts.len();
    let mut s = 0.0;
    for i in 0..n {
        let ((x0, y0), (x1, y1)) = (pts[i], pts[(i + 1) % n]);
        s += x0 * y1 - x1 * y0;
    }
    s.abs() / 2.0
}

pub type Check = (bool, String, String);

fn fmax(v: impl Iterator<Item = f64>) -> f64 {
    v.fold(f64::NEG_INFINITY, py::max)
}
fn fmin(v: impl Iterator<Item = f64>) -> f64 {
    v.fold(f64::INFINITY, py::min)
}

pub fn checks(parts: &[OPart], info: &Info, holes: &[f64], rows: &[(&str, f64, f64, f64)], sheets: &[Sheet]) -> Vec<Check> {
    let mut out: Vec<Check> = vec![];
    let mut note = |ok: bool, what: &str, detail: String| out.push((ok, what.to_string(), detail));
    let (pos, ft) = (&info.pos, info.ft);
    let worst = fmax(rows.iter().map(|r| r.2.abs()));
    note(worst < 1.0, "the model holds the scale", format!("worst {worst:.2} cents of seven notes"));
    // the bore's section, from the inside the walls make
    let mut sec: Vec<(f64, P2)> = info.end_poly.iter().map(|p| (py::matan2(p[2], p[1]), (p[1], p[2]))).collect();
    sec.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    let a = poly_area(&sec.iter().map(|x| x.1).collect::<Vec<_>>());
    note(
        (a / m::area() - 1.0).abs() < 1e-6,
        "the bore is the octagon the model plays",
        format!("{a:.1}mm^2 inside, {}mm across the flats", py::g(m::FLATS)),
    );
    let it = info.inner_of(ft);
    let inner_w = py::min(fmax(it.iter().map(|q| q.1)) - fmin(it.iter().map(|q| q.1)), m::face_in() + 1e-9);
    let widest = fmax(std::iter::once(m::EMB_W).chain(info.shapes.iter().map(|s| s.0)));
    note(
        widest / 2.0 + 0.3 <= inner_w / 2.0,
        "every hole and the embouchure open into the bore",
        format!("widest {widest:.2}mm across, the top's inside {inner_w:.2}mm"),
    );
    let mut bad = 0;
    for (k, e) in info.edges.iter().enumerate() {
        if (ft != e.a && ft != e.b) || info.ends.contains(&e.a) || info.ends.contains(&e.b) {
            continue;
        }
        for &(s0, s1, o) in &info.cellmap[k] {
            let (x0, x1) = xspan(e, s0, s1);
            if o != ft && info.spans.iter().any(|&(a_, b_)| x1 > a_ && x0 < b_) {
                bad += 1;
            }
        }
    }
    note(bad == 0, "the top keeps both its edges beside every opening", format!("{bad} cells of a neighbour beside a hole"));
    let mut sp = info.spans.clone();
    sp.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let gap = fmin(sp.windows(2).map(|w| w[1].0 - w[0].1));
    note(gap >= HOLE_GAP, "ply left between neighbouring openings", format!("{gap:.1}mm at the least"));
    let hands = [holes[0] - holes[1], holes[1] - holes[2], holes[3] - holes[4], holes[4] - holes[5]];
    let hm = fmax(hands.iter().copied());
    note(hm <= 36.0, "each hand's holes are in reach", format!("{hm:.1}mm apart within a hand, {:.1}mm between the hands", holes[2] - holes[3]));
    let web = face_out() - 2.0 * p3::clear(135.0);
    note(web >= 2.0 * MIN_WEB, "a wall keeps ply between its two joints", format!("{web:.2}mm of {:.2}mm where both neighbours own the edge", face_out()));
    note(
        pos.cork_face + m::cork() == pos.emb,
        "the cork stands where the model puts it",
        format!("its face {}mm from the embouchure's centre", py::g(m::cork())),
    );
    note(
        pos.foot + m::FOOT_SPARE == pos.length && m::FOOT_SPARE > 0.0,
        "the foot is cut long, to trim to pitch",
        format!("{}mm past the model's foot", py::g(m::FOOT_SPARE)),
    );
    note(
        LIP_W > face_out() + 2.0 * 5.0 && LIP_L > m::EMB_L + 2.0 * 10.0,
        "the lip plate leaves the lip somewhere to rest",
        format!("{} x {}mm on a {:.1}mm wall", py::g(LIP_W), py::g(LIP_L), face_out()),
    );
    let longest = fmax(parts.iter().map(|p| py::max(p.w, p.h)));
    note(
        longest + kerf() + 2.0 * MARGIN <= BED.0,
        "the longest part fits the bed",
        format!("{:.1}mm with the kerf, against {}", longest + kerf(), py::g(BED.0 - 2.0 * MARGIN)),
    );
    note(
        sheets.iter().all(|s| s.w <= BED.0 && s.h <= BED.1),
        "every sheet fits the bed",
        format!("{} sheet(s), largest {:.0} x {:.0}mm", sheets.len(), fmax(sheets.iter().map(|s| s.w)), fmax(sheets.iter().map(|s| s.h))),
    );
    let placed: usize = sheets.iter().map(|s| s.parts.len()).sum();
    note(placed == parts.len(), "every part is on a sheet", format!("{placed} of {}", parts.len()));
    out
}

pub fn svg_sheet(k: usize, n: usize, s: &Sheet, parts: &[OPart], what: &str) -> String {
    let kf = kerf();
    let (mut orange, mut black, mut blue) = (vec![], vec![], vec![]);
    for &(i, sx, sy) in &s.parts {
        let p = &parts[i];
        let (dx, dy) = (sx - p.x0 + kf / 2.0, sy - p.y0 + kf / 2.0);
        let mv = |pts: &[P2]| -> Vec<P2> { pts.iter().map(|&(x, y)| (x + dx, y + dy)).collect() };
        black.push(path(&mv(&offset(&p.outline, kf / 2.0)), true));
        for c in &p.cuts {
            if let OCut::Rect(q) = c {
                orange.push(path(&mv(&offset(q, -kf / 2.0)), true));
            }
        }
        for st in &p.strokes {
            blue.push(path(&mv(st), false));
        }
    }
    let desc = format!(
        "1 user unit = 1mm. Six-hole transverse flute in D, octagonal bore, sheet {k} of {n}: {what}. 3mm ply, drawn for a {}mm kerf. Blue {BLUE} engraves two lines on the top wall, which lies outside face up: where the cork's face goes, and where the model puts the foot, to trim toward. Then orange {ORANGE} cuts the embouchure and the holes, and black {BLACK} frees the parts.",
        py::g(kf)
    );
    let g = |col: &str, items: &[String]| {
        let mut o = format!("  <g fill=\"none\" stroke=\"{col}\" stroke-width=\"0.2\">\n");
        for i in items {
            o += &format!("    <path d=\"{i}\"/>\n");
        }
        o + "  </g>\n"
    };
    format!(
        "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{w:.2}mm\" height=\"{h:.2}mm\" viewBox=\"0 0 {w:.2} {h:.2}\">\n<title>Octagonal flute in D, sheet {k} of {n}</title>\n<desc>{desc}</desc>\n{}{}{}</svg>\n",
        g(BLUE, &blue),
        g(ORANGE, &orange),
        g(BLACK, &black),
        w = s.w,
        h = s.h
    )
}

pub fn main() -> i32 {
    let (mut out, mut write): (Option<String>, bool) = (None, true);
    for x in std::env::args().skip(1) {
        if x == "--no-write" {
            write = false;
        } else if let Some(v) = x.strip_prefix("--out=") {
            out = Some(v.to_string());
        } else if let Some(v) = x.strip_prefix("--mouth=") {
            m::set_mouth(py::float(v).unwrap_or_else(|| lasermade_tools::die(format!("ValueError: could not convert string to float: {}", py::str_repr(v)))));
        } else if x.starts_with("--kerf=") {
            square_flute::set_kerf(&x);
        } else {
            lasermade_tools::die(format!("error: {x} is not a flag this generator reads. It takes --out=, --mouth=, --kerf= and --no-write."));
        }
    }
    if write && out.is_none() {
        lasermade_tools::die("error: --out=DIR, or --no-write to run the checks only.");
    }
    let b = m::Bore::default();
    let (l, holes, diams) = m::design(&b).unwrap_or_else(|e| lasermade_tools::die(format!("ValueError: {e}")));
    let rows = m::report(&b, l, &holes, &diams).unwrap_or_else(|e| lasermade_tools::die(format!("ValueError: {e}")));
    let (parts, info) = build(l, &holes, &diams);
    let sheets = pack(&parts.iter().map(|p| (p.w, p.h)).collect::<Vec<_>>());
    let res = checks(&parts, &info, &holes, &rows, &sheets);
    let pos = &info.pos;
    println!(
        "octagonal flute in D: {}mm across the flats inside, walls {:.1}mm long; embouchure {:.1}mm from the head end",
        py::g(m::FLATS),
        pos.length,
        pos.emb
    );
    for (((h, x), d), (across, along)) in m::HOLES.iter().zip(&pos.holes).zip(&diams).zip(&info.shapes) {
        let cut = if across == along { format!("ø{d:.2}") } else { format!("{} x {along:.2}", py::g(*across)) };
        println!("  {h}  {x:6.1}mm from the head end  {cut}");
    }
    let mut bad = 0;
    for (ok, what, detail) in &res {
        println!("  {}  {what:52} {detail}", if *ok { "pass" } else { "FAIL" });
        bad += !ok as usize;
    }
    if bad > 0 {
        println!("\n  {bad} check(s) failed. Nothing written.");
        return 1;
    }
    if write {
        let o = out.unwrap();
        std::fs::create_dir_all(&o).unwrap_or_else(|e| lasermade_tools::die(format!("{o}: {e}")));
        for (k, s) in sheets.iter().enumerate() {
            let mut ns: Vec<&str> = s.parts.iter().map(|&(i, _, _)| parts[i].name.as_str()).collect();
            ns.sort();
            ns.dedup();
            let fname = format!("octagonal-flute-D4-sheet{}-cut-files.svg", k + 1);
            let f = format!("{}/{fname}", o.trim_end_matches('/'));
            std::fs::write(&f, svg_sheet(k + 1, sheets.len(), s, &parts, &ns.join(", "))).unwrap_or_else(|e| lasermade_tools::die(format!("{f}: {e}")));
            println!("  wrote {fname}  {:.0} x {:.0}mm, {} parts", s.w, s.h, s.parts.len());
        }
    }
    0
}
