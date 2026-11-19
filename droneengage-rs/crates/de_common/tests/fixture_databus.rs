//! Conformance against traffic recorded from the C++ stack
//! (tests/fixtures/cpp_upstream_tau0, see tests/fixtures/README.md).

use std::collections::HashMap;
use std::io::{BufRead, BufReader};

use base64::Engine;
use de_common::chunk::{self, Reassembler};
use de_common::json::{dump, parse_c_string};
use de_common::module::{HARDWARE_TYPE_CPU, MODULE_CLASS_FCB};
use de_common::{Module, ModuleDefinition};
use serde_json::Value;

struct Datagram {
    src: String,
    dst: String,
    data: Vec<u8>,
}

fn load() -> Vec<Datagram> {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/fixtures/cpp_upstream_tau0/databus.jsonl.gz"
    );
    let f = std::fs::File::open(path).expect("fixture present");
    BufReader::new(flate2::read::GzDecoder::new(f))
        .lines()
        .map(|l| {
            let v: Value = serde_json::from_str(&l.unwrap()).unwrap();
            Datagram {
                src: v["src"].as_str().unwrap().to_string(),
                dst: v["dst"].as_str().unwrap().to_string(),
                data: base64::engine::general_purpose::STANDARD
                    .decode(v["data"].as_str().unwrap())
                    .unwrap(),
            }
        })
        .collect()
}

/// Datagrams of one message as they appeared on the wire.
type Datagrams = Vec<Vec<u8>>;

/// (source, message with trailing NUL, original datagrams)
fn reassemble(dg: &[Datagram]) -> Vec<(String, Vec<u8>, Datagrams)> {
    let mut by_dir: HashMap<(String, String), (Reassembler, Datagrams)> = HashMap::new();
    let mut out = Vec::new();
    for d in dg {
        let e = by_dir.entry((d.src.clone(), d.dst.clone())).or_default();
        e.1.push(d.data.clone());
        if let Some(msg) = e.0.push(&d.data) {
            out.push((d.src.clone(), msg, std::mem::take(&mut e.1)));
        }
    }
    out
}

fn from_cpp_module(src: &str) -> bool {
    // de_mavlink (C++) listens on 61013/61023/61033 in the rig
    src.ends_with(":61013") || src.ends_with(":61023") || src.ends_with(":61033")
}

#[test]
fn every_datagram_reassembles_and_rechunks_identically() {
    let dg = load();
    assert_eq!(dg.len(), 7492);
    let msgs = reassemble(&dg);
    assert_eq!(msgs.len(), 7492, "all fixture messages are single-chunk");
    for (_, msg, original) in &msgs {
        let body = &msg[..msg.len() - 1]; // drop the NUL we add
        assert_eq!(&chunk::split(body, 8192), original);
    }
}

/// Byte-identical except where a double sits exactly halfway between two
/// shortest decimal forms: nlohmann (Grisu2) and serde_json (ryu) may pick
/// different last digits there. Both texts must still parse to the same
/// value, and nothing else may differ. See docs/DIVERGENCES.md.
#[test]
fn json_from_cpp_modules_reserialises_identically() {
    let msgs = reassemble(&load());
    let (mut checked, mut float_ties) = (0, 0);
    for (src, msg, _) in msgs.iter().filter(|(s, _, _)| from_cpp_module(s)) {
        let end = msg.iter().position(|&b| b == 0).unwrap();
        let header = &msg[..end];
        let v = parse_c_string(msg).unwrap();
        let ours = dump(&v);
        checked += 1;
        if ours.as_bytes() == header {
            continue;
        }
        float_ties += 1;
        let reparsed: Value = serde_json::from_str(&ours).unwrap();
        assert_eq!(
            reparsed,
            v,
            "value changed, from {src}: {}",
            String::from_utf8_lossy(header)
        );
        // Same text once number spellings are removed: only float digits differ.
        let strip = |s: &[u8]| -> Vec<u8> {
            s.iter()
                .copied()
                .filter(|b| !b"0123456789.-+eE".contains(b))
                .collect()
        };
        assert_eq!(
            strip(ours.as_bytes()),
            strip(header),
            "non-numeric difference:\n ours {ours}\n  cpp {}",
            String::from_utf8_lossy(header)
        );
    }
    assert!(checked > 7000, "checked {checked}");
    eprintln!("checked {checked} messages, {float_ties} with float tie-break digits");
    assert!(
        float_ties * 100 < checked,
        "float ties should be rare: {float_ties}/{checked}"
    );
}

#[test]
fn rust_id_message_equals_cpp_id_message() {
    let msgs = reassemble(&load());
    let cpp = msgs
        .iter()
        .filter(|(s, _, _)| s.ends_with(":61013"))
        .map(|(_, m, _)| parse_c_string(m).unwrap())
        .find(|v| v["mt"] == 9100 && v["ms"]["z"] == false)
        .expect("FCB_1 ID");
    let ms = &cpp["ms"];
    let m = Module::new(ModuleDefinition {
        class: MODULE_CLASS_FCB.into(),
        id: ms["a"].as_str().unwrap().into(),
        key: ms["e"].as_str().unwrap().into(),
        version: ms["v"].as_str().unwrap().into(),
        message_filter: ms["c"].clone(),
    });
    for f in ms["d"].as_array().unwrap() {
        m.add_module_feature(f.as_str().unwrap());
    }
    m.set_hardware(ms["s"].as_str().unwrap(), HARDWARE_TYPE_CPU);
    let mut rust: Value = serde_json::from_str(&m.create_json_id(false)).unwrap();
    rust["ms"]["u"] = ms["u"].clone(); // instance timestamp differs per run
    assert_eq!(dump(&rust), dump(&cpp));
}

#[test]
fn rust_envelope_equals_cpp_for_recorded_messages() {
    // Rebuild each FCB_1 text message from (tg, ty, mt, ms) with Module::envelope.
    let msgs = reassemble(&load());
    let m = Module::new(ModuleDefinition {
        class: MODULE_CLASS_FCB.into(),
        id: "FCB_1".into(),
        key: "1795085454675320".into(),
        version: String::new(),
        message_filter: Value::Null,
    });
    let mut n = 0;
    for (_, msg, _) in msgs.iter().filter(|(s, _, _)| s.ends_with(":61013")) {
        let v = parse_c_string(msg).unwrap();
        if v.get("GU").is_none() || v.get("tg").is_none() {
            continue; // ID, system and module-remote-execute messages have their own layout
        }
        let internal = v["ty"] == "uv";
        let rebuilt = m.envelope(
            v["tg"].as_str().unwrap(),
            v["ms"].clone(),
            v["mt"].as_i64().unwrap() as i32,
            internal,
        );
        assert_eq!(rebuilt, v);
        n += 1;
    }
    assert!(n > 1000, "rebuilt {n}");
}
