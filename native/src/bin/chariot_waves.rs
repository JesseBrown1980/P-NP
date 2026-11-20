//! Chariot historical provenance and GitRAM two-wave twenty-leaf acceptance.
//! Rust 1.81, integer-only, no unsafe, no runtime JSON or network service.
//! The historical Chariot seal is immutable; wave roots are new commitments.
#![forbid(unsafe_code)]

use pnp_native::brown_ledger::{BrownDelta, BrownSlice, Direction};
use pnp_native::matrix::CELLS;
use pnp_native::sha256::hash_bytes;
use std::env;
use std::fs;
use std::path::Path;

const SEAL: &str = "2b1889ee7a3a18ddd54cd9ec4229dbb266bdafd6f03cb9d2cf5f979cbc23967e";
const SOURCES: [(&str, &[u8], &str); 3] = [
    (
        "CHARIOT-LYNN-2026-08-08.md",
        include_bytes!("../../fixtures/chariots/CHARIOT-LYNN-2026-08-08.md"),
        "ab139e987e8468ca6953fdba20db997b11bb6da26afacf181cdbe771fe6fdeb4",
    ),
    (
        "CHARIOT-REBECCA-2026-08-08.md",
        include_bytes!("../../fixtures/chariots/CHARIOT-REBECCA-2026-08-08.md"),
        "d8b197a9b06b7acbc4812cf7469a1ed3e306bb96d7c6615e116f9a4442012671",
    ),
    (
        "CHARIOT-EZEQUEL-2026-08-08.md",
        include_bytes!("../../fixtures/chariots/CHARIOT-EZEQUEL-2026-08-08.md"),
        "b15d03148c23a7fcc943f3e6d162dad21b9501ef4e9e1d170d343784f7fe5a30",
    ),
];
const HBP: &str = include_str!("../../fixtures/chariots/THE-THREE-CHARIOTS-SEALED-2026-08-08.hbp");
const N: usize = 20;
type State = [i64; CELLS];

fn require(pass: bool, message: &str) -> Result<(), String> {
    if pass { Ok(()) } else { Err(message.to_owned()) }
}

fn provenance() -> Result<(), String> {
    let mut digest_words = Vec::new();
    for (filename, bytes, expected) in SOURCES {
        let digest = hash_bytes(bytes);
        require(digest == expected, "original Chariot source bytes changed")?;
        let line = format!("CHARIOT|file={filename}|sha256={expected}|");
        require(
            HBP.lines().any(|row| row.starts_with(&line)),
            "Chariot HBP member digest mismatch",
        )?;
        digest_words.push(digest);
    }
    require(
        HBP.lines().filter(|row| row.starts_with("CHARIOT|")).count() == 3,
        "unexpected Chariot membership",
    )?;
    require(
        hash_bytes(digest_words.join("|").as_bytes()) == SEAL,
        "historic Chariot seal failed independent derivation",
    )?;
    require(
        HBP.lines().any(|row| row == "CENTER|set={HBI,HBP,SHA,SH,HASH}|the sign at the centre of the nullspace|unordered=1|json=0"),
        "historical unordered center changed",
    )?;
    require(
        HBP.lines().any(|row| row == "TRAVERSAL|order=HBI->HBP->SH->HASH->SHA|json=0"),
        "historical ordered traversal changed",
    )?;
    let seal_line = format!("SEAL_OF_THE_THREE|sha256={SEAL}|");
    require(
        HBP.lines().any(|row| row.starts_with(&seal_line)),
        "historical HBP seal absent",
    )?;
    Ok(())
}

fn valid_run(run: &str) -> Result<(), String> {
    require(
        !run.is_empty() && run.len() <= 128 &&
        run.bytes().all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b)),
        "invalid or empty run identity",
    )
}

fn state_sha(state: &State) -> String {
    let mut bytes = Vec::with_capacity(CELLS * 8);
    for value in state {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    hash_bytes(&bytes)
}

fn original(leaf: usize) -> State {
    let mut out = [0_i64; CELLS];
    for (cell, value) in out.iter_mut().enumerate() {
        *value = (leaf as i64 - 10) * 27 + cell as i64 * 7 - 93;
    }
    out
}

fn transformed(leaf: usize, source: &State) -> State {
    let mut target = *source;
    for (cell, value) in target.iter_mut().enumerate() {
        let direction = if (cell + leaf) % 2 == 0 { 1_i64 } else { -1_i64 };
        *value += direction * (leaf as i64 % 3 + 1) * (cell as i64 % 4 + 1);
    }
    target
}

fn to_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for &byte in bytes {
        out.push(char::from(HEX[usize::from(byte >> 4)]));
        out.push(char::from(HEX[usize::from(byte & 15)]));
    }
    out
}

fn from_hex(hex: &str) -> Result<Vec<u8>, String> {
    fn nibble(byte: u8) -> Option<u8> {
        match byte {
            b'0'..=b'9' => Some(byte - b'0'),
            b'a'..=b'f' => Some(byte - b'a' + 10),
            _ => None,
        }
    }
    let bytes = hex.as_bytes();
    require(bytes.len() % 2 == 0, "odd hexadecimal field")?;
    bytes.chunks_exact(2).map(|pair| {
        let hi = nibble(pair[0]).ok_or_else(|| "invalid hex digit".to_owned())?;
        let lo = nibble(pair[1]).ok_or_else(|| "invalid hex digit".to_owned())?;
        Ok((hi << 4) | lo)
    }).collect()
}

fn field<'a>(line: &'a str, name: &str) -> Result<&'a str, String> {
    let prefix = format!("{name}=");
    let mut matches = line.split('|').skip(1).filter_map(|pair| pair.strip_prefix(&prefix));
    let value = matches.next().ok_or_else(|| format!("missing field {name}"))?;
    require(matches.next().is_none(), "duplicate field")?;
    Ok(value)
}

fn leaf_path(directory: &Path, prefix: &str, leaf: usize) -> std::path::PathBuf {
    directory.join(format!("{prefix}-{leaf:02}.hbp"))
}

fn make_forward(run: &str, leaf: usize) -> Result<String, String> {
    valid_run(run)?;
    provenance()?;
    require(leaf < N, "forward leaf outside 0..19")?;
    let source = original(leaf);
    let target = transformed(leaf, &source);
    let slice = BrownSlice::freeze(&source);
    let delta = BrownDelta::freeze(&source, &target);
    let reloaded = BrownSlice::load(slice.serialized().to_vec()).map_err(str::to_owned)?;
    let replay = BrownDelta::load(delta.serialized().to_vec(), &source).map_err(str::to_owned)?;
    require(
        reloaded.play().map_err(str::to_owned)? == source
            && replay.play(&source, Direction::Forward).map_err(str::to_owned)? == target
            && replay.play(&target, Direction::Reverse).map_err(str::to_owned)? == source,
        "Brown Ledger forward, reverse, or source replay failed",
    )?;
    let slice_sha = hash_bytes(slice.serialized());
    let delta_sha = hash_bytes(delta.serialized());
    Ok(format!(
        "CH20_FORWARD|schema=1|run={run}|leaf={leaf}|to={}|lane=EZEQUEL|chariot={SEAL}|source={}|target={}|slice_sha={slice_sha}|delta_sha={delta_sha}|triad=SOURCE_REFLECTION_FABRIC|json=0\nSLICE|hex={}|json=0\nDELTA|hex={}|json=0\n",
        (leaf + 1) % N, state_sha(&source), state_sha(&target),
        to_hex(slice.serialized()), to_hex(delta.serialized()),
    ))
}

fn check_forward(run: &str, leaf: usize, content: &str) -> Result<(State, State), String> {
    require(content.ends_with('\n'), "forward record must end with LF")?;
    let parts: Vec<_> = content.lines().collect();
    require(parts.len() == 3, "forward checkpoint must contain three lines")?;
    require(parts[0].starts_with("CH20_FORWARD|schema=1|"), "wrong forward header")?;
    require(field(parts[0], "run")? == run, "cross-run forward receipt")?;
    require(field(parts[0], "leaf")? == leaf.to_string(), "forward leaf mismatch")?;
    require(field(parts[0], "to")? == ((leaf + 1) % N).to_string(), "forward routing changed")?;
    require(field(parts[0], "lane")? == "EZEQUEL", "forward lane mismatch")?;
    require(field(parts[0], "chariot")? == SEAL, "forward provenance mismatch")?;
    require(parts[1].starts_with("SLICE|hex=") && parts[2].starts_with("DELTA|hex="), "missing snapshot/delta")?;
    let slice_bytes = from_hex(field(parts[1], "hex")?)?;
    let delta_bytes = from_hex(field(parts[2], "hex")?)?;
    require(field(parts[0], "slice_sha")? == hash_bytes(&slice_bytes), "snapshot SHA mismatch")?;
    require(field(parts[0], "delta_sha")? == hash_bytes(&delta_bytes), "delta SHA mismatch")?;
    let snapshot = BrownSlice::load(slice_bytes).map_err(str::to_owned)?;
    let source = snapshot.play().map_err(str::to_owned)?;
    let delta = BrownDelta::load(delta_bytes, &source).map_err(str::to_owned)?;
    let target = delta.play(&source, Direction::Forward).map_err(str::to_owned)?;
    let restored = delta.play(&target, Direction::Reverse).map_err(str::to_owned)?;
    require(restored == source, "forward delta not reversible")?;
    require(field(parts[0], "source")? == state_sha(&source), "forward source checksum")?;
    require(field(parts[0], "target")? == state_sha(&target), "forward target checksum")?;
    require(source == original(leaf) && target == transformed(leaf, &source), "independent source/fabric replay failed")?;
    require(content == make_forward(run, leaf)?, "noncanonical forward receipt or payload")?;
    Ok((source, target))
}

fn forward_root(run: &str, directory: &Path) -> Result<String, String> {
    provenance()?;
    valid_run(run)?;
    require(fs::read_dir(directory).map_err(|e| e.to_string())?.count() == N, "forward fan-in requires exactly twenty files")?;
    let mut parts = vec![format!("CH20_FORWARD_ROOT_V1|run={run}|seal={SEAL}")];
    for leaf in 0..N {
        let content = fs::read_to_string(leaf_path(directory, "forward", leaf)).map_err(|e| e.to_string())?;
        check_forward(run, leaf, &content)?;
        parts.push(hash_bytes(content.as_bytes()));
    }
    Ok(hash_bytes(parts.join("|").as_bytes()))
}

fn root_line(run: &str, root: &str) -> String {
    format!("CH20_ROOT|schema=1|run={run}|phase=FORWARD|chariot={SEAL}|leaves=20|root={root}|json=0\n")
}

fn checked_root(run: &str, directory: &Path, root_path: &Path) -> Result<String, String> {
    let actual = forward_root(run, directory)?;
    require(
        fs::read_to_string(root_path).map_err(|e| e.to_string())? == root_line(run, &actual),
        "forward wave root changed, missing, or corrupt",
    )?;
    Ok(actual)
}

fn make_reverse(run: &str, leaf: usize, directory: &Path, root_path: &Path) -> Result<String, String> {
    require(leaf < N, "reverse leaf outside 0..19")?;
    let root = checked_root(run, directory, root_path)?;
    let from = (leaf + N - 1) % N;
    let received = fs::read_to_string(leaf_path(directory, "forward", from)).map_err(|e| e.to_string())?;
    let (source, target) = check_forward(run, from, &received)?;
    let forward_lines: Vec<_> = received.lines().collect();
    let delta_bytes = from_hex(field(forward_lines[2], "hex")?)?;
    let delta = BrownDelta::load(delta_bytes, &source).map_err(str::to_owned)?;
    let restored = delta.play(&target, Direction::Reverse).map_err(str::to_owned)?;
    require(restored == source && restored == original(from), "reverse leaf failed exact source reconstruction")?;
    Ok(format!(
        "CH20_REVERSE|schema=1|run={run}|leaf={leaf}|from={from}|lane=REBECCA|chariot={SEAL}|parent_root={root}|input_sha={}|restored_sha={}|forward_receipt_sha={}|cells=27|reconstruction=EXACT|triad=SOURCE_REFLECTION_FABRIC|json=0\n",
        state_sha(&target), state_sha(&restored), hash_bytes(received.as_bytes()),
    ))
}

fn final_record(run: &str, forward_dir: &Path, root_path: &Path, reverse_dir: &Path) -> Result<String, String> {
    let forward_hash = checked_root(run, forward_dir, root_path)?;
    require(fs::read_dir(reverse_dir).map_err(|e| e.to_string())?.count() == N, "reverse fan-in requires exactly twenty files")?;
    let mut parts = vec![format!("CH20_REVERSE_ROOT_V1|run={run}|seal={SEAL}|parent={forward_hash}")];
    for leaf in 0..N {
        let received = fs::read_to_string(leaf_path(reverse_dir, "reverse", leaf)).map_err(|e| e.to_string())?;
        let expected = make_reverse(run, leaf, forward_dir, root_path)?;
        require(received == expected, "independent reverse leaf verification failed")?;
        parts.push(hash_bytes(received.as_bytes()));
    }
    let reverse_hash = hash_bytes(parts.join("|").as_bytes());
    Ok(format!(
        "CH20_FINAL|schema=1|run={run}|historical_seal={SEAL}|forward_root={forward_hash}|reverse_root={reverse_hash}|forward_leaves=20|reverse_leaves=20|checkpoints=40|restored_cells=540|replay=EXACT|source_bytes=RETAINED_IN_PAID_SLICES|transfer=GITHUB_ACTIONS_ARTIFACTS|backend_http0=SEPARATE_GATE|status=PASS|json=0\n"
    ))
}

fn arg(parts: &mut impl Iterator<Item = String>, what: &str) -> Result<String, String> {
    parts.next().ok_or_else(|| format!("missing {what}"))
}

fn run() -> Result<(), String> {
    let mut args = env::args().skip(1);
    let command = arg(&mut args, "command")?;
    match command.as_str() {
        "provenance" => {
            provenance()?;
            println!("CH20_PROVENANCE|commit=ffcf56177654b4e0c600377d9cfd6a14796647b0|members=3|seal={SEAL}|sha256=PASS|center=UNORDERED|traversal=PINNED|status=PASS|json=0");
        }
        "forward" => {
            let run_id = arg(&mut args, "run id")?;
            let leaf: usize = arg(&mut args, "leaf id")?.parse().map_err(|_| "invalid leaf id")?;
            let output = arg(&mut args, "checkpoint path")?;
            fs::write(&output, make_forward(&run_id, leaf)?).map_err(|e| e.to_string())?;
            println!("CH20_FORWARD_PASS|leaf={leaf}|to={}|run={run_id}|status=PASS|json=0", (leaf + 1) % N);
        }
        "fanin" => {
            let run_id = arg(&mut args, "run id")?;
            let directory = arg(&mut args, "forward directory")?;
            let output = arg(&mut args, "root file")?;
            let root = forward_root(&run_id, Path::new(&directory))?;
            fs::write(&output, root_line(&run_id, &root)).map_err(|e| e.to_string())?;
            println!("CH20_FANIN|leaves=20|root={root}|status=PASS|json=0");
        }
        "reverse" => {
            let run_id = arg(&mut args, "run id")?;
            let leaf: usize = arg(&mut args, "leaf id")?.parse().map_err(|_| "invalid leaf id")?;
            let directory = arg(&mut args, "forward directory")?;
            let root_file = arg(&mut args, "forward root")?;
            let output = arg(&mut args, "reverse checkpoint")?;
            let body = make_reverse(&run_id, leaf, Path::new(&directory), Path::new(&root_file))?;
            fs::write(&output, body).map_err(|e| e.to_string())?;
            println!("CH20_REVERSE_PASS|leaf={leaf}|source_leaf={}|restored_cells=27|status=PASS|json=0", (leaf + N - 1) % N);
        }
        "final" => {
            let run_id = arg(&mut args, "run id")?;
            let forward_dir = arg(&mut args, "forward directory")?;
            let root_file = arg(&mut args, "forward root")?;
            let reverse_dir = arg(&mut args, "reverse directory")?;
            let output = arg(&mut args, "final receipt")?;
            let receipt = final_record(
                &run_id, Path::new(&forward_dir), Path::new(&root_file), Path::new(&reverse_dir),
            )?;
            fs::write(&output, &receipt).map_err(|e| e.to_string())?;
            print!("{receipt}");
        }
        _ => return Err("commands: provenance, forward, fanin, reverse, final".to_owned()),
    }
    require(args.next().is_none(), "unexpected extra arguments")
}

fn main() {
    if let Err(message) = run() {
        eprintln!("CH20_HOLD|reason={message}|status=HOLD|json=0");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_and_two_wave_exact_replay_with_negative_controls() {
        provenance().expect("historical source and seal");
        let run = "test-chariot-20";
        let root_dir = env::temp_dir().join(format!("chariot-wave-test-{}", std::process::id()));
        let forward_dir = root_dir.join("forward");
        let reverse_dir = root_dir.join("reverse");
        let root_file = root_dir.join("forward-root.hbp");
        let _ = fs::remove_dir_all(&root_dir);
        fs::create_dir_all(&forward_dir).expect("forward directory");
        fs::create_dir_all(&reverse_dir).expect("reverse directory");
        for leaf in 0..N {
            fs::write(leaf_path(&forward_dir, "forward", leaf), make_forward(run, leaf).expect("forward leaf")).expect("write forward");
        }
        let root = forward_root(run, &forward_dir).expect("forward fan-in");
        fs::write(&root_file, root_line(run, &root)).expect("write root");
        for leaf in 0..N {
            fs::write(
                leaf_path(&reverse_dir, "reverse", leaf),
                make_reverse(run, leaf, &forward_dir, &root_file).expect("reverse leaf"),
            ).expect("write reverse");
        }
        assert!(final_record(run, &forward_dir, &root_file, &reverse_dir).expect("final fan-in").contains("status=PASS"));
        let first_forward = leaf_path(&forward_dir, "forward", 0);
        let original = fs::read(&first_forward).expect("read checkpoint");
        let mut broken = original.clone();
        broken[0] ^= 1;
        fs::write(&first_forward, &broken).expect("tamper forward");
        assert!(forward_root(run, &forward_dir).is_err());
        fs::write(&first_forward, original).expect("restore test fixture");
        let first_reverse = leaf_path(&reverse_dir, "reverse", 0);
        let original = fs::read(&first_reverse).expect("read reverse");
        let mut broken = original.clone();
        broken[0] ^= 1;
        fs::write(&first_reverse, &broken).expect("tamper reverse");
        assert!(final_record(run, &forward_dir, &root_file, &reverse_dir).is_err());
        fs::write(&first_reverse, original).expect("restore reverse");
        fs::write(&root_file, "invalid root\n").expect("tamper root");
        assert!(checked_root(run, &forward_dir, &root_file).is_err());
        fs::remove_dir_all(&root_dir).expect("cleanup");
    }
}
