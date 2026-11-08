//! A six-hole transverse flute in D, the Irish or simple-system flute, with
//! a regular octagonal bore, as a transmission-matrix model: how long the
//! bore runs past the embouchure, and where the six finger holes go. Rust
//! since 2026-11-07 (flute_model.py until then; the move changed no byte of
//! any file).
//!
//!     target/release/flute_model               # the design, and how it plays
//!     target/release/flute_model --mouth=1.2   # scale the embouchure's correction
//!     target/release/flute_model --cork=17     # another cork distance, mm
//!     target/release/flute_model --bores      # the design, for bore-tmm
//!     target/release/flute_model --heard=285 --length=537
//!                                              # the --mouth a cut flute implies
//!
//! All closed it sounds D4; opening the holes from the foot up gives E4,
//! F#4, G4, A4, B4 and C#5, and overblown, the octave above. C natural is a
//! cross fingering, as on any six-hole flute.
//!
//! THE BORE is a regular octagon FLATS across inside, 19mm: 299mm^2, the
//! area of a 19.5mm round bore, near the 19mm of an Irish flute's. Its loss
//! is taken on the hydraulic radius 2A/P, which for any regular polygon is
//! half the flats.
//!
//! The model, from the foot up:
//!
//!   * the bore is a lossy tube, Benade's wall loss on the hydraulic radius;
//!   * the foot radiates as an unflanged open end of the same area;
//!   * each finger hole is Keefe's (1990) tone hole, open or closed, its
//!     chimney the 3mm wall, as on the square flute;
//!   * at the top, the embouchure hole is a short neck, CHIMNEY deep (the
//!     wall and the lip plate), into open air, with the cork's closed stub
//!     beside it. The jet drives the hole, so the flute sounds where the
//!     reactance of the neck in series with the bore and the stub in
//!     parallel passes through zero, rising.
//!
//! The arithmetic is bore-tmm's jet kernel (src/models.rs), the one the
//! square flute's model uses too.
//!
//! THE EMBOUCHURE IS THE UNCALIBRATED PART. The player's lip covers part of
//! the hole, by an amount that changes with how they play, so its area is
//! taken as COVER of the cut hole and its end correction is physics-based,
//! not measured. --mouth= scales the correction. The foot is cut FOOT_SPARE
//! long, to be trimmed to pitch: a model that guessed short would leave a
//! flute too short to fix, so the guess is made long.
//!
//! Moving the correction moves the holes almost as far as the foot: at
//! --mouth=0.75 the foot is 8.9mm longer and the holes 7.8 to 8.6mm lower.
//! So trimming the foot does not tune the holes, as on the square flute;
//! each is tuned by filing it longer along the bore, which raises its note,
//! and every hole is cut with room for that (FILE).
//!
//! THE CORK sets how well the second octave lines up with the first: nearer
//! the embouchure, the octaves widen and the top comes easier; further, the
//! low octave is stronger. The cork here is press-fit discs, so it can be
//! moved; CORK is where it starts.
use bore_tmm::models::{scan, Jet};
use lasermade_tools::py;
use std::f64::consts::PI;
use std::sync::atomic::{AtomicU64, Ordering};

// --- air, warm from the breath -------------------------------------------------
pub const T_C: f64 = 25.0;
/// 346.1 m/s
pub fn c() -> f64 {
    331.3 * (1.0 + T_C / 273.15).sqrt()
}
pub const RHO: f64 = 1.184;

// --- the tune ------------------------------------------------------------------
/// 293.66Hz
pub fn d4() -> f64 {
    440.0 * py::powf(2.0, -7.0 / 12.0)
}
pub const NOTES: [&str; 7] = ["D4", "E4", "F#4", "G4", "A4", "B4", "C#5"];
pub const SEMIS: [i32; 7] = [0, 2, 4, 5, 7, 9, 11];
pub fn targets() -> Vec<f64> {
    SEMIS.iter().map(|&s| d4() * py::powf(2.0, s as f64 / 12.0)).collect()
}
pub const HOLES: [&str; 6] = ["R3", "R2", "R1", "L3", "L2", "L1"]; // foot-most first

// --- the bore and the head (mm) --------------------------------------------------
pub const T: f64 = 3.0; // the ply
pub const FLATS: f64 = 19.0; // the octagon across its flats, inside
pub fn area() -> f64 {
    2.0 * (2f64.sqrt() - 1.0) * py::powf(FLATS, 2.0)
}
/// one wall's inside width
pub fn face_in() -> f64 {
    FLATS * py::mtan(PI / 8.0)
}
// embouchure: across x along the bore, a stadium: the wall's inside is 7.87
// wide
pub const EMB_W: f64 = 7.0;
pub const EMB_L: f64 = 12.0;
pub const CHIMNEY: f64 = 2.0 * T; // the wall and the lip plate
pub const COVER: f64 = 0.75; // of the hole the lip leaves open
pub const FOOT_SPARE: f64 = 25.0; // cut this much long, trim to pitch
// A finger hole is round up to HOLE_W, the most the wall's inside takes with
// a little to spare, and past that a stadium HOLE_W across and longer along
// the bore, up to HOLE_LONG: a finger pad covers 7 x 12mm easily. The model
// reads a stadium as the round hole of its area.
pub const HOLE_W: f64 = 7.0;
pub const HOLE_LONG: f64 = 12.0;
pub const SPACING: f64 = 34.0; // centre to centre within a hand
pub const ANCHORS: [usize; 2] = [2, 6]; // R2 and L1, by the note each opens
pub const D_MIN: f64 = 4.0;
// the anchors are cut FILE short of HOLE_LONG along the bore, so even the
// biggest hole can be filed longer, and sharper, once the flute is played
pub const FILE: f64 = 1.0;

static MOUTH: AtomicU64 = AtomicU64::new(0x3ff0000000000000); // 1.0: scales the embouchure's end correction
static CORK: AtomicU64 = AtomicU64::new(0x4034000000000000); // 20.0: embouchure centre to the cork's face

pub fn mouth() -> f64 {
    f64::from_bits(MOUTH.load(Ordering::Relaxed))
}
pub fn set_mouth(m: f64) {
    MOUTH.store(m.to_bits(), Ordering::Relaxed);
}
pub fn cork() -> f64 {
    f64::from_bits(CORK.load(Ordering::Relaxed))
}
pub fn set_cork(m: f64) {
    CORK.store(m.to_bits(), Ordering::Relaxed);
}

pub fn stadium_area(w: f64, l: f64) -> f64 {
    w * (l - w) + PI * py::powf(w / 2.0, 2.0)
}

/// The hole a model diameter d is cut as: (across, along).
pub fn hole_shape(d: f64) -> (f64, f64) {
    if d <= HOLE_W {
        return (d, d);
    }
    let r = HOLE_W / 2.0;
    (HOLE_W, HOLE_W + (PI * d * d / 4.0 - PI * r * r) / HOLE_W)
}

pub fn d_max() -> f64 {
    (4.0 * stadium_area(HOLE_W, HOLE_LONG) / PI).sqrt()
}
pub fn d_anchor() -> f64 {
    (4.0 * stadium_area(HOLE_W, HOLE_LONG - FILE) / PI).sqrt()
}

pub fn emb_area() -> f64 {
    stadium_area(EMB_W, EMB_L)
}

/// The octagon: its area, its equivalent radius and its loss radius (m).
pub struct Bore {
    pub area: f64,
    pub rh: f64,
    pub a: f64,
}

impl Bore {
    pub fn new(flats: f64) -> Bore {
        let area = 2.0 * (2f64.sqrt() - 1.0) * py::powf(flats / 1000.0, 2.0);
        Bore { area, rh: flats / 2000.0, a: (area / PI).sqrt() }
    }
}

impl Default for Bore {
    fn default() -> Bore {
        Bore::new(FLATS)
    }
}

/// The embouchure as a neck: its open area (m^2), equivalent radius and
/// effective length (m).
pub fn emb_neck() -> (f64, f64, f64) {
    let ae_ = COVER * emb_area() / 1e6;
    let ae = (ae_ / PI).sqrt();
    (ae_, ae, CHIMNEY / 1000.0 + mouth() * 2.0 * 0.82 * ae)
}

/// The playing frequency nearest `near`: a zero crossing of the reactance
/// the jet sees, rising through zero, found by scanning then bisecting. L mm
/// of bore from the embouchure centre to the foot, holes at their distances
/// from the embouchure centre, foot-most first, the lowest n_open of them
/// open.
pub fn sounding(b: &Bore, l: f64, holes: &[f64], n_open: usize, near: f64, diams: &[f64], span: f64) -> Result<f64, String> {
    let mut chain: Vec<(u8, f64, f64, u8)> = vec![];
    let mut pos = l;
    for (i, &x) in holes.iter().enumerate() {
        chain.push((0, (pos - x) / 1000.0, 0.0, 0));
        chain.push((1, 0.0, diams[i] / 2000.0, (i < n_open) as u8));
        pos = x;
    }
    chain.push((0, pos / 1000.0, 0.0, 0));
    chain.reverse();
    let kind: Vec<u8> = chain.iter().map(|c| c.0).collect();
    let len: Vec<f64> = chain.iter().map(|c| c.1).collect();
    let r: Vec<f64> = chain.iter().map(|c| c.2).collect();
    let op: Vec<u8> = chain.iter().map(|c| c.3).collect();
    let (neck_area, neck_a, leff) = emb_neck();
    let jet = Jet {
        c: c(), rho: RHO, rh: b.rh, area: b.area, a: b.a,
        kind: &kind, len: &len, r: &r, open: &op, t: T,
        stub: cork() / 1000.0, neck_area, neck_a, leff,
    };
    let root = scan(near, span, 80, 40, |x| jet.y(x));
    if root.is_nan() {
        return Err(format!("no resonance within 40% of {near:.1}Hz"));
    }
    Ok(root)
}

/// x in [lo, hi] with fn(x) == target. fn falls as x grows, unless rising.
/// An error when the target is out of reach.
pub fn bisect(func: impl Fn(f64) -> Result<f64, String>, target: f64, mut lo: f64, mut hi: f64, rising: bool) -> Result<f64, String> {
    let g = |x: f64| -> Result<f64, String> { func(x).map(|v| if rising { -v } else { v }) };
    let t = if rising { -target } else { target };
    let (gh, gl) = (g(hi)?, g(lo)?);
    if !(gh <= t && t <= gl) {
        return Err(format!("out of reach between {lo:.1} and {hi:.1}"));
    }
    for _ in 0..40 {
        let mid = (lo + hi) / 2.0;
        if g(mid)? > t {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    Ok((lo + hi) / 2.0)
}

/// The bore's length from the embouchure centre to the foot, the six holes'
/// positions from the embouchure centre, foot-most first, and their
/// diameters, iterated until each closed hole's effect on the others
/// settles.
///
/// Laid out for the hands: one hole in each hand, the ANCHORS, is the
/// largest the wall takes less FILE, D_ANCHOR, and its place is solved; the
/// hand's other holes stand SPACING apart from it and their SIZES are
/// solved. The anchors are R2, the lower hand's middle, and L1, the upper
/// hand's top: each is the hole that, at full size, still leaves its
/// neighbours room to be smaller. Anchored on L2, L1 would have had to stand
/// lower than a full-size hole can for C#5. A smaller hole has to sit nearer
/// the embouchure for its note, which is what lets them close up: all 7mm
/// round, the six fall 56, 17, 55, 42 and 37mm apart, the lower hand far too
/// wide for a hand.
pub fn design(b: &Bore) -> Result<(f64, Vec<f64>, Vec<f64>), String> {
    let (d4, tg) = (d4(), targets());
    let mut l = 540.0;
    let frac = [0.82, 0.76, 0.69, 0.61, 0.54, 0.47];
    let mut holes: Vec<f64> = frac.iter().map(|r| l * r).collect();
    let mut diams = vec![d_anchor(); 6];
    for it in 0..6 {
        l = bisect(|x| sounding(b, x, &holes, 0, d4, &diams, 1.4), d4, 400.0, 700.0, false)?;
        if it == 0 {
            holes = frac.iter().map(|r| l * r).collect();
            l = bisect(|x| sounding(b, x, &holes, 0, d4, &diams, 1.4), d4, 400.0, 700.0, false)?;
        }
        // the anchors first, so the holes either side are sized where they
        // will stand
        for n in [2usize, 6, 1, 3, 4, 5] {
            let i = n - 1;
            if ANCHORS.contains(&n) {
                let lo = if n < 6 { holes[i + 1] + 0.5 } else { 60.0 };
                let hi = holes[i - 1] - 0.5;
                let f_pos = |x: f64| {
                    let mut hs = holes.clone();
                    hs[i] = x;
                    sounding(b, l, &hs, n, tg[n], &diams, 1.4)
                };
                holes[i] = bisect(f_pos, tg[n], lo, hi, false)?;
                let hand = if n <= 3 { 0..3 } else { 3..6 };
                for j in hand {
                    holes[j] = holes[i] + (i as i64 - j as i64) as f64 * SPACING;
                }
            } else {
                let f_dia = |d: f64| {
                    let mut ds = diams.clone();
                    ds[i] = d;
                    sounding(b, l, &holes, n, tg[n], &ds, 1.4)
                };
                diams[i] = bisect(f_dia, tg[n], D_MIN, d_anchor(), true)?;
            }
        }
    }
    Ok((l, holes, diams))
}

pub fn cents(f: f64, rf: f64) -> f64 {
    1200.0 * py::log2(f / rf)
}

/// Each note's error, and the octave each fingering overblows to: (name,
/// f1, cents off, the octave's cents off 2:1).
pub fn report(b: &Bore, l: f64, holes: &[f64], diams: &[f64]) -> Result<Vec<(&'static str, f64, f64, f64)>, String> {
    let tg = targets();
    let mut rows = vec![];
    for n in 0..7 {
        let f1 = sounding(b, l, holes, n, tg[n], diams, 1.4)?;
        let f2 = sounding(b, l, holes, n, 2.0 * tg[n], diams, 1.2)?;
        rows.push((NOTES[n], f1, cents(f1, tg[n]), cents(f2, 2.0 * f1)));
    }
    Ok(rows)
}

/// How much longer than the bore the flute plays, all closed: the
/// embouchure's and the foot's corrections and the cork's stub together.
pub fn end_correction(l: f64) -> f64 {
    1000.0 * c() / (2.0 * d4()) - l
}

/// The MOUTH that makes a flute whose embouchure centre is `length` mm from
/// its foot sound `heard` Hz with every hole closed. Closed holes barely
/// move the note, so they are left where the design puts them.
pub fn mouth_for(heard: f64, length: f64) -> Result<f64, String> {
    let b = Bore::default();
    let (_, holes, diams) = design(&b)?;
    let saved = mouth();
    let f = |m: f64| {
        set_mouth(m);
        sounding(&b, length, &holes, 0, heard, &diams, 1.4)
    };
    let r = bisect(f, heard, 0.05, 4.0, false);
    set_mouth(saved);
    r
}

fn die(e: String) -> ! {
    lasermade_tools::die(format!("ValueError: {e}"))
}

/// The flute as bore-tmm reads it, one JSON object on stdout: "models",
/// the design with what this model says every note plays (bore-tmm's
/// tools/export_models.py writes it to bores/models/), and "reader", what
/// tools/read_cut_files.py needs beside the cut file it measures: the air,
/// the lip's cover, the chimneys, the correction, the section, the holes'
/// names and the notes.
pub fn bores() -> String {
    use lasermade_tools::turn3d::{Num::F, J};
    let n = |x: f64| J::N(F(x));
    let s = |x: &str| J::S(x.to_string());
    let b = Bore::default();
    let (l, holes, diams) = design(&b).unwrap_or_else(|e| die(e));
    let tg = targets();
    let names = |k: usize| J::A(HOLES[..k].iter().map(|x| s(x)).collect());
    let model = J::obj(vec![
        ("name", s("octagonal flute in D, flute_model.rs design")),
        ("source", s("octagonal-flute/flute_model.rs design()")),
        ("temp_c", n(T_C)),
        (
            "drive",
            J::obj(vec![
                ("kind", s("jet")),
                ("area_mm2", n(COVER * emb_area())),
                ("chimney_mm", n(CHIMNEY)),
                ("mouth", n(mouth())),
                ("stub_mm", n(cork())),
            ]),
        ),
        ("bore", J::A(vec![J::obj(vec![("len_mm", n(l)), ("d_mm", n(2000.0 * b.a)), ("loss_ratio", n(b.rh / b.a))])])),
        ("end", s("unflanged")),
        ("holes", J::A((0..6).map(|i| J::obj(vec![("name", s(HOLES[i])), ("at_mm", n(holes[i])), ("d_mm", n(diams[i])), ("chimney_mm", n(T))])).collect())),
        (
            "notes",
            J::A(
                (0..7)
                    .map(|k| {
                        let hz = sounding(&b, l, &holes, k, tg[k], &diams, 1.4).unwrap_or_else(|e| die(e));
                        J::obj(vec![("name", s(NOTES[k])), ("hz", n(tg[k])), ("open", names(k)), ("span", n(1.4)), ("model_hz", n(hz))])
                    })
                    .collect(),
            ),
        ),
    ]);
    let reader = J::obj(vec![
        ("temp_c", n(T_C)),
        ("cover", n(COVER)),
        ("chimney_mm", n(CHIMNEY)),
        ("mouth", n(mouth())),
        ("wall_mm", n(T)),
        ("d_mm", n(2000.0 * b.a)),
        ("loss_ratio", n(b.rh / b.a)),
        ("holes", names(6)),
        ("notes", J::A((0..7).map(|k| J::A(vec![s(NOTES[k]), n(tg[k])])).collect())),
    ]);
    let mut out = String::new();
    J::O(vec![("models".into(), J::obj(vec![("octagonal-flute", model)])), ("reader".into(), reader)]).dump(&mut out);
    out
}

pub fn main() -> i32 {
    if std::env::args().nth(1).as_deref() == Some("--bores") {
        println!("{}", bores());
        return 0;
    }
    let (mut heard, mut length): (Option<f64>, Option<f64>) = (None, None);
    let num = |x: &str| {
        let t = x.split_once('=').map(|p| p.1).unwrap_or("");
        py::float(t).unwrap_or_else(|| lasermade_tools::die(format!("ValueError: could not convert string to float: {}", py::str_repr(t))))
    };
    for x in std::env::args().skip(1) {
        if x.starts_with("--mouth=") {
            set_mouth(num(&x));
        } else if x.starts_with("--cork=") {
            set_cork(num(&x));
        } else if x.starts_with("--heard=") {
            heard = Some(num(&x));
        } else if x.starts_with("--length=") {
            length = Some(num(&x));
        } else {
            lasermade_tools::die(format!("error: {x} is not a flag this model reads. It takes --mouth=, --cork=, --heard= and --length=."));
        }
    }
    if heard.is_none() != length.is_none() {
        lasermade_tools::die("error: --heard= and --length= go together: the closed note in Hz, and the embouchure centre to the foot in mm, as cut.");
    }
    if let (Some(h), Some(len)) = (heard, length) {
        let m = mouth_for(h, len).unwrap_or_else(|_| {
            lasermade_tools::die(format!(
                "error: no --mouth between 0.05 and 4 makes {}mm sound {}Hz. Check the length is from the embouchure's centre to the foot.",
                py::g(len),
                py::g(h)
            ))
        });
        println!("{}Hz from {}mm of bore: --mouth={m:.3}. Redraw with\n  target/release/octagonal_flute --mouth={m:.3} --out=mine", py::g(h), py::g(len));
        return 0;
    }
    let b = Bore::default();
    let (l, holes, diams) = design(&b).unwrap_or_else(|e| die(e));
    println!(
        "D4 = {:.2}Hz, air at {}C ({:.1} m/s); octagon {}mm across its flats, {:.0}mm^2 (a {:.1}mm round bore's area); embouchure {} x {}mm, {}mm deep, {} uncovered, mouth x{}; cork {}mm",
        d4(), py::g(T_C), c(), py::g(FLATS), area(), 2000.0 * b.a, py::g(EMB_W), py::g(EMB_L), py::g(CHIMNEY), py::g(COVER), py::g(mouth()), py::g(cork())
    );
    println!("embouchure centre to foot {l:.1}mm (cut {:.1}, trimmed to pitch); the corrections add {:.1}mm", l + FOOT_SPARE, end_correction(l));
    println!("holes from the embouchure centre, foot-most first:");
    let mut prev = l;
    for ((h, &x), &d) in HOLES.iter().zip(&holes).zip(&diams) {
        let (w, ln) = hole_shape(d);
        let cut = if w == ln { format!("ø{d:.2}") } else { format!("{} x {ln:.2} stadium (ø{d:.2})", py::g(w)) };
        println!("  {h}  {x:6.1}mm  {cut:28}  {:5.1}mm above the one below", prev - x);
        prev = x;
    }
    println!("note   Hz      cents  octave above, cents off 2:1");
    for (nm, f1, c1, c2) in report(&b, l, &holes, &diams).unwrap_or_else(|e| die(e)) {
        println!("  {nm:4} {f1:7.2} {c1:+6.1}  {c2:+6.1}");
    }
    0
}
