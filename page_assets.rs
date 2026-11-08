//! The published page's moving parts, built from the generator's own build()
//! so the page cannot disagree with the cut file. The writeup is README.md,
//! built into index.html by md2html like every page here; this adds what
//! markdown cannot carry.
//!
//!     target/release/page_assets                      # viewer.js, turn.html and plans/*.svg
//!     target/release/page_assets --artifact OUT.html  # the whole page, one file
//!
//!   viewer.js    the 3D view, loaded on the page through a page-only line
//!                in README.md
//!   turn.html    viewer.js on a page of its own, for the hub to frame
//!   plans/       the head of the top wall drawn to scale, and the fingering
//!                chart, as plain SVG, so they read on GitHub as well as on
//!                the page
//!
//! The viewer's styling, its page wrapper, turn.html and the artifact
//! folding are the square flute's and the four-hole ocarinas', from their
//! page_assets.rs; the parts are this instrument's own. Python
//! (page_assets.py) until 2026-11-07.
use crate::flute_model as m;
use crate::*;
use four_hole_ocarinas::page_assets as ocp;
use lasermade_tools::turn3d::{dump_str, Num, J};
use square_flute::page_assets::text as sfp;

const FONT: &str = "font-family=\"Helvetica, Arial, sans-serif\"";
const BUILD: &str = "octagonal-flute-D4";

fn here() -> &'static str {
    env!("CARGO_MANIFEST_DIR")
}

fn fl(x: f64) -> J {
    J::N(Num::F(x))
}
fn f3(x: f64) -> J {
    fl(py::round_dp(x, 3))
}
fn pts3(o: &[P2]) -> J {
    J::A(o.iter().map(|q| J::A(vec![f3(q.0), f3(q.1)])).collect())
}
const ZERO: J = J::N(Num::I(0));

/// The parts for the view, each placed by p.m. The tube lies along x, head
/// end at 0, top up; exploded, each wall comes away along its own outward
/// normal, the lip plate further than the top under it, and the cork's
/// discs out of the head end, the outermost furthest.
pub fn solids(parts: &[OPart], info: &Info) -> J {
    let r = flats_out() / 2.0 / py::mcos(PI / 8.0);
    let mut out = vec![];
    let mut corks = 0;
    for p in parts {
        let mut mm = p.m.unwrap();
        mm[2][3] += r; // the tube resting on z = 0
        let (e, g) = if p.name == "lip plate" {
            (J::A(vec![ZERO, ZERO, fl(2.4)]), "dbl")
        } else if p.name == "cork disc" {
            let e = J::A(vec![fl(-1.2 - 0.8 * (CORK_DISCS - 1 - corks) as f64), ZERO, ZERO]);
            corks += 1;
            (e, "plug")
        } else {
            let n = info.faces[p.face.unwrap()].n;
            (J::A(n.iter().map(|x| fl(py::round_dp(1.3 * x, 4))).collect()), if p.name == "top wall" { "plate" } else { "wall" })
        };
        let holes: Vec<J> = p
            .cuts
            .iter()
            .map(|c| match c {
                OCut::Rect(q) => J::obj(vec![("p", pts3(q))]),
                OCut::Circle(..) => unreachable!("the flute's openings are all polygons"),
            })
            .collect();
        out.push(J::obj(vec![
            ("o", pts3(&p.outline)),
            ("h", J::A(holes)),
            ("m", J::A(mm.iter().map(|row| J::A(row.iter().map(|v| fl(py::round_dp(*v, 4))).collect())).collect())),
            ("e", e),
            ("g", J::S(g.into())),
            ("name", J::S(p.name.clone())),
            ("f", J::A(vec![])),
            ("fc", J::Null),
            ("flip", J::B(false)),
            ("zc", ZERO),
        ]));
    }
    J::obj(vec![
        ("parts", J::A(out)),
        ("len", fl(py::round_dp(info.pos.length, 1))),
        ("top", fl(py::round_dp(2.0 * r, 2))),
        ("bore", J::S(format!("{}mm across the flats", py::g(m::FLATS)))),
    ])
}

/// The top wall from outside, to scale, head on the left, as it lies on the
/// sheet, cut short past the last hole: the lip plate dashed, the embouchure
/// and the six holes named with their sizes, the blue cork line, and the
/// bore's inside edges dotted under the wall.
pub fn plan_svg(parts: &[OPart], info: &Info, title: &str) -> String {
    let tp = parts.iter().find(|p| p.name == "top wall").unwrap();
    let pos = &info.pos;
    let end = pos.holes[0] + 20.0;
    let vs: Vec<f64> = tp.outline.iter().map(|q| q.1).collect();
    let vmin = py::min(square_flute::fmin(&vs), -LIP_W / 2.0);
    let (x0, y0) = (6.0, 6.0 - vmin + 6.0);
    let w_ = end + 12.0;
    let h_ = py::max(square_flute::fmax(&vs), LIP_W / 2.0) - vmin + 24.0;
    let clip = format!("<clipPath id=\"c\"><rect x=\"0\" y=\"0\" width=\"{:.1}\" height=\"{h_:.1}\"/></clipPath>", end + x0);
    let d = |pts: &[P2], z: &str| {
        format!("M {}{z}", pts.iter().map(|&(u, v)| format!("{:.2},{:.2}", u + x0, v + y0)).collect::<Vec<_>>().join(" L "))
    };
    let mut s = vec![
        format!("<rect x=\"0\" y=\"0\" width=\"{w_:.1}\" height=\"{h_:.1}\" fill=\"#ffffff\"/>"),
        clip,
        format!("<g clip-path=\"url(#c)\"><path d=\"{}\" fill=\"#e7dcc4\" stroke=\"#1d232b\" stroke-width=\".4\"/></g>", d(&tp.outline, " Z")),
    ];
    let w = m::face_in() / 2.0;
    s.push(format!(
        "<path d=\"M {x0:.2},{:.2} H {:.2} M {x0:.2},{:.2} H {:.2}\" fill=\"none\" stroke=\"#8a7350\" stroke-width=\".5\" stroke-dasharray=\"1 1\"/>",
        y0 - w,
        end + x0,
        y0 + w,
        end + x0
    ));
    let lip = parts.iter().find(|p| p.name == "lip plate").unwrap();
    s.push(format!("<path d=\"{}\" fill=\"none\" stroke=\"#1d232b\" stroke-width=\".5\" stroke-dasharray=\"2 1.5\"/>", d(&lip.outline, " Z")));
    for c in &tp.cuts {
        if let OCut::Rect(q) = c {
            s.push(format!("<path d=\"{}\" fill=\"#ffffff\" stroke=\"#a85800\" stroke-width=\".6\"/>", d(q, " Z")));
        }
    }
    for st in &tp.strokes {
        if st[0].0 < end {
            s.push(format!("<path d=\"{}\" fill=\"none\" stroke=\"{BLUE}\" stroke-width=\".6\"/>", d(st, "")));
        }
    }
    let t = format!("{FONT} font-size=\"4\" fill=\"#1d232b\" text-anchor=\"middle\"");
    s.push(format!(
        "<text x=\"{:.2}\" y=\"{:.2}\" {t}>embouchure {} × {}</text>",
        pos.emb + x0,
        y0 + LIP_W / 2.0 + 6.0,
        py::g(m::EMB_W),
        py::g(m::EMB_L)
    ));
    s.push(format!("<text x=\"{:.2}\" y=\"{:.2}\" {t}>cork</text>", pos.cork_face + x0, y0 - LIP_W / 2.0 - 2.0));
    for (k, ((h, x), (across, along))) in m::HOLES.iter().zip(&pos.holes).zip(&info.shapes).enumerate() {
        let size = if across == along { format!("ø{across:.1}") } else { format!("{} × {along:.1}", py::g(*across)) };
        let y = if k % 2 == 1 { y0 - w - 3.0 } else { y0 + w + 6.5 };
        s.push(format!("<text x=\"{:.2}\" y=\"{y:.2}\" {t}>{h} {size}</text>", x + x0));
    }
    format!(
        "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {w_:.0} {h_:.0}\">\n<title>{title}</title>\n<desc>Display drawing, not a cut file. The top wall from outside, to scale, head end on the left, cut off 20mm past the lowest hole: its edges cut into fingers, left whole beside every opening; the bore's inside edges dotted beneath; the lip plate's outline dashed round the embouchure; the six finger holes named for their fingers with their sizes, round or longer along the flute; and the blue line where the cork's face goes.</desc>\n{}\n</svg>\n",
        s.join("\n")
    )
}

/// the six-hole flute's fingerings: holes L1 L2 L3 R1 R2 R3, 1 covered
const CHART: [(&str, &str); 8] = [
    ("D4", "111111"),
    ("E4", "111110"),
    ("F♯4", "111100"),
    ("G4", "111000"),
    ("A4", "110000"),
    ("B4", "100000"),
    ("C♯5", "000000"),
    ("D5", "011111"),
];

/// The fingering chart: a column of six holes per note, the head up, the
/// left hand's three above the right's, black where covered.
pub fn chart_svg(title: &str) -> String {
    let (cw, ch) = (22.0, 78.0);
    let n = CHART.len() as f64;
    let mut s = vec![format!("<rect x=\"0\" y=\"0\" width=\"{:.0}\" height=\"{ch:.0}\" fill=\"#ffffff\"/>", cw * n)];
    let t = format!("{FONT} text-anchor=\"middle\"");
    for (i, (note, pat)) in CHART.iter().enumerate() {
        let cx = cw * i as f64 + cw / 2.0;
        s.push(format!(
            "<rect x=\"{:.1}\" y=\"3\" width=\"10\" height=\"62\" rx=\"2\" fill=\"#e7dcc4\" stroke=\"#1d232b\" stroke-width=\".4\"/>",
            cx - 5.0
        ));
        for (k, c) in pat.chars().enumerate() {
            let cy = 10 + 8 * k + if k >= 3 { 4 } else { 0 };
            let fill = if c == '1' { "#1d232b" } else { "#ffffff" };
            s.push(format!("<circle cx=\"{cx:.1}\" cy=\"{:.1}\" r=\"2.6\" fill=\"{fill}\" stroke=\"#1d232b\" stroke-width=\".4\"/>", cy as f64));
        }
        s.push(format!("<text x=\"{cx:.1}\" y=\"{:.1}\" font-size=\"6\" fill=\"#1d232b\" {t}>{note}</text>", ch - 4.0));
    }
    format!(
        "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {:.0} {ch:.0}\">\n<title>{title}</title>\n<desc>Display drawing. The D major scale from D4 to D5, one column per note: the six holes with the head end up, the left hand's three above the right's. Black is covered, white is open. D5 is the second octave, overblown, with the top hole open.</desc>\n{}\n</svg>\n",
        cw * n,
        s.join("\n")
    )
}

fn markup() -> String {
    ocp::MARKUP.split("  <div class=\"vctl\">").next().unwrap().to_string()
        + "  <div class=\"vctl\">\n    <label for=\"explode\">Explode</label><input id=\"explode\" type=\"range\" min=\"0\" max=\"1\" step=\"0.01\" value=\"0\">\n    <button type=\"button\" id=\"vreset\">Reset view</button>\n    <label class=\"chk\"><input type=\"checkbox\" id=\"xray\"> See through the walls</label>\n  </div>\n  <details class=\"note vhow\"><summary>How to use this view</summary><p>Drag to turn; right-drag, shift-drag or move two fingers to slide the view; scroll or pinch to zoom in on whatever is under the pointer; double-click a part to turn round it, and Reset view to go back. The parts are the generator's own outlines before the kerf, so this is the flute as built: eight walls finger-jointed down their length, the top one whole beside every hole; the embouchure through the top and the lip plate over it; and the cork, three discs pressed into the head end. Exploded, each wall comes away along its own face, and the cork comes out of the head.</p></details>\n</section>"
}

fn jstr(s: &str) -> String {
    let mut o = String::new();
    dump_str(s, &mut o);
    o
}

pub fn viewer_js(three: &[(String, J)]) -> String {
    let mut js = sfp::JS.split("  // one button per construction").next().unwrap().to_string();
    let a = "'mm · bore ' + d.W.toFixed(2) + ' × ' + d.D.toFixed(2) + 'mm'";
    if !js.contains(a) {
        lasermade_tools::die("AssertionError: the square flute's viewer script changed");
    }
    js = js.replace(a, "'mm · octagonal bore ' + d.bore");
    js += "  const bb = [...document.querySelectorAll('#turn .vbar button[data-v]')];\n  bb.forEach(b => b.addEventListener('click', () => {\n    bb.forEach(x => x.setAttribute('aria-pressed', x === b)); build(b.dataset.v);\n  }));\n  window.addEventListener('resize', size);\n  build(bb[0].dataset.v); size();\n})();\n";
    let mut data = String::new();
    J::O(three.to_vec()).dump(&mut data);
    js = js.replace("__DATA__", &data);
    let buttons: String = three
        .iter()
        .enumerate()
        .map(|(i, (n, _))| format!("<button type=\"button\" data-v=\"{n}\" aria-pressed=\"{}\"><code>{n}</code></button>", i == 0))
        .collect();
    sfp::VIEWER_JS
        .replace("__CSS__", &jstr(sfp::CSS))
        .replace("__MARKUP__", &jstr(&markup().replace("__BUTTONS__", &buttons)))
        .replace("__JS__", &js)
}

pub const PLANS: [&str; 2] = ["octagonal-flute-top-wall.svg", "octagonal-flute-fingering.svg"];

pub fn main() -> i32 {
    let a: Vec<String> = std::env::args().skip(1).collect();
    if a.len() == 2 && a[0] == "--artifact" {
        ocp::artifact(here(), &a[1]);
        return 0;
    }
    let pick = |k: &str| -> Vec<String> { a.iter().filter_map(|x| x.strip_prefix(k).map(String::from)).collect() };
    let (vt, pt, tt) = (pick("--viewer-to="), pick("--plans-to="), pick("--turn-to="));
    if vt.len() + pt.len() + tt.len() != a.len() {
        lasermade_tools::die("usage: page_assets [--viewer-to=FILE] [--plans-to=DIR] [--turn-to=FILE] | --artifact OUT.html");
    }
    let h = here();
    let write = |f: &str, s: &str| std::fs::write(f, s).unwrap_or_else(|e| lasermade_tools::die(format!("{f}: {e}")));
    if !tt.is_empty() || a.is_empty() {
        let page = sfp::TURN_PAGE.replace("<title>Square-bore flute in 3D</title>", "<title>Octagonal flute in 3D</title>");
        if !page.contains("Octagonal flute") {
            lasermade_tools::die("AssertionError: the square flute's turn page changed");
        }
        write(&tt.first().cloned().unwrap_or(format!("{h}/turn.html")), &page);
    }
    let (l, holes, diams) = m::design(&m::Bore::default()).unwrap_or_else(|e| lasermade_tools::die(format!("ValueError: {e}")));
    let (parts, info) = build(l, &holes, &diams);
    let three = vec![(BUILD.to_string(), solids(&parts, &info))];
    if !vt.is_empty() || a.is_empty() {
        write(&vt.first().cloned().unwrap_or(format!("{h}/viewer.js")), &viewer_js(&three));
    }
    if !pt.is_empty() || a.is_empty() {
        let d = pt.first().cloned().unwrap_or(format!("{h}/plans"));
        std::fs::create_dir_all(&d).unwrap_or_else(|e| lasermade_tools::die(format!("{d}: {e}")));
        let d = d.trim_end_matches('/');
        write(&format!("{d}/{}", PLANS[0]), &plan_svg(&parts, &info, "The top wall from outside, to scale"));
        write(&format!("{d}/{}", PLANS[1]), &chart_svg("Fingering chart, D4 to D5"));
    }
    println!("  wrote viewer.js; plans: {}", PLANS.join(", "));
    0
}
