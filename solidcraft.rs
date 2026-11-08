//! The flute as a parametric SolidCraft design (2026-11-08): the rules
//! build() follows, as expressions, so changing one number rebuilds the rest.
//! `octagonal_flute --solidcraft=DIR` writes the script;
//! `solidcraft-cli run SCRIPT --save X.sc` makes the design, `--out X.step`
//! the STEP file.
//!
//! The frame is the 3D view's and the STEP file's: X along the flute from the
//! head end, Y across, Z up, the bore's axis at Z = the octagon's outside
//! corner radius, the top wall facing +Z.
//!
//! What follows a parameter: the ply and the bore (every wall, the joints'
//! depth, the cork discs), the head (the bare tube past the cork, the cork's
//! distance from the embouchure, the embouchure), the lip plate, the foot's
//! spare, and the joints, which re-count along the walls; the top keeps both
//! its edges within `hole_clear` of every opening, wherever the openings move.
//! A hole wider than `hole_w` turns into a stadium of the same area, as
//! flute_model's hole_shape() makes it.
//!
//! What does not: the tuning. The chamber (embouchure centre to the foot)
//! and the six holes' places and diameters are what flute_model's solver
//! found, which no expression can redo; they are parameters so a flute tuned
//! by hand can be written back into its design. The number of cork discs is
//! fixed (each is a body).
//!
//! The blue lines (the cork's face, the model's foot) are not drawn.

use crate::flute_model as m;
use crate::*;
use four_hole_ocarinas::polyhedron::{self as ph, V3};
use lasermade_tools::solidcraft::{n, Script};
use std::f64::consts::PI;

/// A point as expressions.
#[derive(Clone)]
struct R {
    x: String,
    y: String,
    z: String,
}

/// sum of coef * (expr), the zero terms dropped.
fn lin(terms: &[(f64, &str)]) -> String {
    let t: Vec<String> = terms.iter().filter(|(c, _)| c.abs() > 1e-12).map(|(c, e)| format!("{}*({e})", n(*c))).collect();
    if t.is_empty() { "0 mm".into() } else { t.join(" + ") }
}

/// The point's component along the world direction d.
fn dotw(p: &R, d: [f64; 3]) -> String {
    lin(&[(d[0], &p.x), (d[1], &p.y), (d[2], &p.z)])
}

fn dot3(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// An opening on the top's centre line: a stadium `along` x `across` at x,
/// cut through `targets` from z0 up `height`. Two cylinders at the ends'
/// centres and a box between, lifted clear when the opening is round.
fn opening(s: &mut Script, name: &str, x: &str, along: &str, across: &str, z0: &str, height: &str, targets: &[&str]) {
    let half = format!("(({along}) - ({across}))/2");
    for (k, sign) in [(1, "-"), (2, "+")] {
        s.cylinder(&format!("{name}, end {k}"), [format!("{x} {sign} {half}"), "0 mm".into(), z0.to_string()], None, across, height, "cut", targets);
    }
    let on = format!("min(1, floor((({along}) - ({across}))/(0.001 mm)))");
    s.bx(
        &format!("{name}, middle"),
        [format!("{x} - {half} + (1 - {on})*1000 mm"), format!("-({across})/2"), z0.to_string()],
        [format!("max(2*{half}, 0.01 mm)"), across.to_string(), height.to_string()],
        "cut",
        targets,
    );
}

pub fn script(l: f64, holes: &[f64], diams: &[f64]) -> String {
    let mut s = Script::new();
    let t8 = (PI / 8.0).tan();
    let c8 = (PI / 8.0).cos();
    // the sheet and the bore
    s.len("ply", format!("{} mm", n(square_flute::T)), "sheet thickness: every wall, the lip plate, each cork disc, and the joints' depth");
    s.len("flats", format!("{} mm", n(m::FLATS)), "the bore, an octagon, across its flats inside");
    s.len("fo", "flats + 2*ply", "read only: the octagon across its flats outside");
    s.len("face_out", format!("fo*{}", n(t8)), "read only: one wall's outside width");
    s.len("lift", format!("fo/2/{}", n(c8)), "read only: the axis's height, the octagon's outside corner radius");
    s.len("finger", format!("{} mm", n(ph::FINGER)), "the shortest cell along an edge");
    // the head
    s.len("head_past", format!("{} mm", n(HEAD_PAST)), "bare tube at the head end, past the cork");
    s.num("cork_discs", n(CORK_DISCS as f64), "the cork's press-fit discs, each a body");
    s.len("interf", format!("{} mm", n(INTERF)), "the cork's press-fit, across its flats");
    s.len("cork", format!("{} mm", n(m::cork())), "the cork's face to the embouchure's centre: moves the octaves");
    s.len("emb_w", format!("{} mm", n(m::EMB_W)), "the embouchure across");
    s.len("emb_l", format!("{} mm", n(m::EMB_L)), "and along the flute");
    s.len("emb", "head_past + cork_discs*ply + cork", "read only: the embouchure's centre from the head end");
    // the tuning, the solver's
    s.len("chamber", format!("{} mm", n(l)), "the embouchure's centre to the foot: flute_model's D4 for this bore");
    s.len("foot_spare", format!("{} mm", n(m::FOOT_SPARE)), "cut this much long at the foot, trim to pitch");
    s.len("hole_w", format!("{} mm", n(m::HOLE_W)), "a hole is round up to this; past it, a stadium this wide and longer along the flute, of the same area");
    for k in 0..6 {
        let i = k + 1;
        s.len(&format!("h_{i}"), format!("{} mm", n(holes[k])), &format!("hole {i} ({}), from the embouchure's centre: the solver's", m::HOLES[k]));
        s.len(&format!("d_{i}"), format!("{} mm", n(diams[k])), &format!("hole {i}'s diameter, or the round hole of its area: the solver's"));
        s.len(&format!("across_{i}"), format!("min(d_{i}, hole_w)"), &format!("read only: hole {i} across"));
        s.len(&format!("along_{i}"), format!("across_{i} + max(0 mm, pi*(d_{i}*d_{i} - hole_w*hole_w)/4/hole_w)"), &format!("read only: hole {i} along the flute"));
    }
    s.len("length", "emb + chamber + foot_spare", "read only: the walls' length");
    s.num("ncell", "floor(length/finger + 1e-9) - mod(floor(length/finger + 1e-9), 2)", "read only: cells along each edge, an even count, both ends open");
    s.len("cell", "length/ncell", "read only: each cell's length");
    s.len("hole_clear", format!("{} mm", n(HOLE_CLEAR)), "the top keeps both its edges this far past every opening");
    // the lip plate
    s.len("lip_w", format!("{} mm", n(LIP_W)), "the lip plate across");
    s.len("lip_l", format!("{} mm", n(LIP_L)), "and along the flute");
    s.len("lip_r", format!("{} mm", n(LIP_R)), "its corners' radius");
    // the checks, as room left: below 0 fails
    s.len("bed_room", format!("{} mm - length", n(square_flute::BED.0 - 2.0 * square_flute::MARGIN - square_flute::kerf())), "read only: what the bed leaves past the walls");
    s.len(
        "open_room",
        format!("flats*{} - 0.6 mm - max(emb_w, max(max(max(across_1, across_2), max(across_3, across_4)), max(across_5, across_6)))", n(t8)),
        "read only: room left across the top's inside for the widest opening",
    );
    s.len(
        "gap_room",
        "min(min(min(h_6 - along_6/2 - emb_l/2, h_5 - h_6 - (along_5 + along_6)/2), min(h_4 - h_5 - (along_4 + along_5)/2, h_3 - h_4 - (along_3 + along_4)/2)), min(h_2 - h_3 - (along_2 + along_3)/2, h_1 - h_2 - (along_1 + along_2)/2)) - 3 mm",
        "read only: ply between the closest two openings, past 3 mm",
    );
    s.len("reach_room", "36 mm - max(max(h_1 - h_2, h_2 - h_3), max(h_4 - h_5, h_5 - h_6))", "read only: a hand's span left within a hand, of 36 mm");
    s.len("web_room", format!("face_out - 2*ply*{} - {} mm", n((135f64).to_radians().sin()), n(2.0 * square_flute::MIN_WEB)), "read only: ply a wall keeps between its two joints");
    // the forced runs: each opening's cells, as indices from the head end
    let spans: Vec<(String, String)> = std::iter::once(("emb - emb_l/2".to_string(), "emb + emb_l/2".to_string()))
        .chain((1..=6).rev().map(|i| (format!("emb + h_{i} - along_{i}/2"), format!("emb + h_{i} + along_{i}/2"))))
        .collect();
    let names: Vec<String> = std::iter::once("e".to_string()).chain((1..=6).rev().map(|i| i.to_string())).collect();
    for ((a, b), nm) in spans.iter().zip(&names) {
        let what = if nm == "e" { "the embouchure".to_string() } else { format!("hole {nm}") };
        s.num(&format!("jlo_{nm}"), format!("max(0, floor(({a} - hole_clear)/cell + 1e-9))"), &format!("read only: the first cell the top keeps for {what}"));
        s.num(&format!("jhi_{nm}"), format!("min(ncell - 1, ceil(({b} + hole_clear)/cell - 1e-9) - 1)"), &format!("read only: and the last"));
    }

    let pos = positions(l, holes);
    let faces = tube(pos.length);
    let ends: Vec<usize> = (0..faces.len()).filter(|&i| faces[i].name.ends_with(" end")).collect();
    let ft = faces.iter().position(|f| f.name == "top").unwrap();
    let fo0 = flats_out();
    let len0 = pos.length;
    let at = |v: V3| R {
        x: if v[0].abs() < 1e-6 { "0 mm".into() } else if (v[0] - len0).abs() < 1e-6 { "length".into() } else { panic!("a vertex off the ends") },
        y: lin(&[(v[1] / fo0, "fo")]),
        z: lin(&[(1.0, "lift"), (v[2] / fo0, "fo")]),
    };
    // the walls: each face's outline, extruded inward
    struct Fr {
        xd: [f64; 3],
        yd: [f64; 3],
        off: String,
        body: String,
    }
    let mut frs: Vec<Option<Fr>> = (0..faces.len()).map(|_| None).collect();
    let mut num = 0;
    for (fi, f) in faces.iter().enumerate() {
        if ends.contains(&fi) {
            continue;
        }
        let xd = [1.0, 0.0, 0.0];
        let mut yd = ph::cross(f.n, xd);
        let mut nrm = ph::cross(xd, yd);
        if dot3(nrm, f.n) > 0.0 {
            yd = [-yd[0], -yd[1], -yd[2]];
            nrm = ph::cross(xd, yd);
        }
        num += 1;
        let body = format!("{num} {}", if fi == ft { "top wall".to_string() } else { f.name.clone() });
        let off = dotw(&at(f.verts[0]), nrm);
        let sk = s.sketch_on([0.0, 0.0, 0.0], xd, yd, Some(&off), &format!("{body} sketch"));
        let pts: Vec<[String; 2]> = f.verts.iter().map(|&v| [dotw(&at(v), xd), dotw(&at(v), yd)]).collect();
        s.poly(&pts);
        s.finish();
        s.extrude(&sk, "ply", false, "new", &[], &body);
        frs[fi] = Some(Fr { xd, yd, off, body });
    }
    let top_body = frs[ft].as_ref().unwrap().body.clone();
    // the joints
    let mut nfeat = 0usize;
    for e in ph::edges(&faces) {
        if ends.contains(&e.a) || ends.contains(&e.b) {
            continue;
        }
        let dirv = ph::unit(ph::sub(e.q, e.p));
        let from_head = e.p[0].abs() < 1e-6;
        let start = at(e.p);
        let dc = format!("(ply*{})", n(e.theta.to_radians().sin()));
        let topped = e.a == ft || e.b == ft;
        // the forced runs as indices from p, in order along the edge
        let runs: Vec<(String, String)> = {
            let r: Vec<(String, String)> = names
                .iter()
                .map(|nm| {
                    if from_head {
                        (format!("jlo_{nm}"), format!("jhi_{nm}"))
                    } else {
                        (format!("(ncell - 1 - jhi_{nm})"), format!("(ncell - 1 - jlo_{nm})"))
                    }
                })
                .collect();
            if from_head { r } else { r.into_iter().rev().collect() }
        };
        for pl in [e.a, e.b] {
            let fr = frs[pl].as_ref().unwrap();
            let s2 = (dotw(&start, fr.xd), dotw(&start, fr.yd));
            let d2 = (dot3(dirv, fr.xd), dot3(dirv, fr.yd));
            let c2 = (dot3(ph::sub(faces[pl].centre, e.p), fr.xd), dot3(ph::sub(faces[pl].centre, e.p), fr.yd));
            let mut nin = (-d2.1, d2.0);
            if c2.0 * nin.0 + c2.1 * nin.1 < 0.0 {
                nin = (-nin.0, -nin.1);
            }
            let strip = |t0: &str, t1: &str, away: &str| -> Vec<[String; 2]> {
                let pt = |t: &str, dd: &str| {
                    let along = format!("({t})");
                    let across = format!("(({dd}) - ({away}))");
                    [
                        format!("{} + {}", s2.0, lin(&[(d2.0, &along), (nin.0, &across)])),
                        format!("{} + {}", s2.1, lin(&[(d2.1, &along), (nin.1, &across)])),
                    ]
                };
                vec![pt(t0, "-ply"), pt(t1, "-ply"), pt(t1, &dc), pt(t0, &dc)]
            };
            let (target, xd, yd, off) = (fr.body.clone(), fr.xd, fr.yd, fr.off.clone());
            let mut cutter = |s: &mut Script, pts: Vec<[String; 2]>, count: Option<(String, String)>| {
                nfeat += 1;
                let name = format!("cells {nfeat}");
                let sk = s.sketch_on([0.0, 0.0, 0.0], xd, yd, Some(&off), &format!("{name} sketch"));
                s.poly(&pts);
                s.finish();
                s.extrude(&sk, "ply", false, "cut", &[&target], &name);
                if let Some((cnt, sp)) = count {
                    s.pattern(&name, dirv, cnt, sp);
                }
            };
            // every other cell from `first`, `cnt` of them
            let mut prog = |s: &mut Script, first: &str, cnt: &str| {
                let away = format!("(1 - min({cnt}, 1))*1000 mm");
                cutter(s, strip(&format!("({first})*cell"), &format!("({first} + 1)*cell"), &away), Some((format!("max({cnt}, 1)"), "2*cell".into())));
            };
            // the cells this plate loses: its neighbour's, from k0
            let k0 = if pl == e.a { 1 } else { 0 };
            if !topped {
                prog(&mut s, &k0.to_string(), &format!("floor((ncell - {k0} + 1)/2)"));
            } else if pl == ft {
                // the neighbour's, only between the runs the top keeps
                let mut bounds: Vec<(String, String)> = vec![];
                let mut g0 = "0".to_string();
                for (lo, hi) in &runs {
                    bounds.push((g0.clone(), format!("({lo} - 1)")));
                    g0 = format!("({hi} + 1)");
                }
                bounds.push((g0, "(ncell - 1)".into()));
                for (g0, g1) in bounds {
                    let first = format!("({g0} + mod({g0} + {k0}, 2))");
                    prog(&mut s, &first, &format!("max(0, floor(({g1} - {first})/2) + 1)"));
                }
            } else {
                // the top's, and every cell of every run
                prog(&mut s, &k0.to_string(), &format!("floor((ncell - {k0} + 1)/2)"));
                for (lo, hi) in &runs {
                    let away = format!("(1 - min(1, max(0, {hi} - {lo} + 1)))*1000 mm");
                    cutter(&mut s, strip(&format!("({lo})*cell"), &format!("({hi} + 1)*cell"), &away), None);
                }
            }
        }
    }
    // the lip plate: two crossed boxes and the four corners' round
    let lip = format!("{} lip plate", num + 1);
    let top_out = "lift + fo/2";
    let lz = (top_out.to_string(), format!("{top_out} + ply"));
    s.bx(&lip, ["emb - lip_l/2".to_string(), "-lip_w/2 + lip_r".into(), lz.0.clone()], ["lip_l".to_string(), "lip_w - 2*lip_r".into(), "ply".into()], "new", &[]);
    s.bx(&format!("{lip}, ends"), ["emb - lip_l/2 + lip_r".to_string(), "-lip_w/2".into(), lz.0.clone()], ["lip_l - 2*lip_r".to_string(), "lip_w".into(), "ply".into()], "join", &[&lip]);
    for (k, (sx, sy)) in [("-", "-"), ("+", "-"), ("+", "+"), ("-", "+")].iter().enumerate() {
        s.cylinder(
            &format!("{lip}, corner {}", k + 1),
            [format!("emb {sx} (lip_l/2 - lip_r)"), format!("{sy}(lip_w/2 - lip_r)"), lz.0.clone()],
            None,
            "2*lip_r",
            "ply",
            "join",
            &[&lip],
        );
    }
    // the cork's discs
    for k in 0..CORK_DISCS {
        let body = format!("{} cork disc", num + 2 + k);
        let r = format!("(flats + interf)/2/{}", n(c8));
        let sk = s.sketch_on([0.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0], Some(&format!("head_past + {k}*ply")), &format!("{body} sketch"));
        let pts: Vec<[String; 2]> = (0..8)
            .map(|q| {
                let a = PI / 8.0 + q as f64 * PI / 4.0;
                [lin(&[(a.cos(), &r)]), lin(&[(1.0, "lift"), (a.sin(), &r)])]
            })
            .collect();
        s.poly(&pts);
        s.finish();
        s.extrude(&sk, "ply", false, "new", &[], &body);
    }
    // the openings: the embouchure through the top and the lip plate, the holes through the top
    let z0 = "lift + fo/2 - ply - 1 mm";
    opening(&mut s, "embouchure", "emb", "emb_l", "emb_w", z0, "2*ply + 2 mm", &[&top_body, &lip]);
    for i in 1..=6 {
        opening(&mut s, &format!("hole {i}"), &format!("emb + h_{i}"), &format!("along_{i}"), &format!("across_{i}"), z0, "ply + 2 mm", &[&top_body]);
    }
    s.measure();
    s.text()
}
