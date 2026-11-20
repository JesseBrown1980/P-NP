//! Original Bitcoin SHA256d CNF -> standard GGUF v3 -> Bobby Fischer play.
//! Frozen source and SAT assignment trits are real I8 GGUF tensors. All paths
//! use exact integers and the existing original Fischer modular addresses.
#![forbid(unsafe_code)]

use pnp_native::bitcoin_gguf::{freeze, play_variable, thaw, BitcoinFrozen};
use pnp_native::sha256::hash_bytes;
use pnp_native::ternary::{Cnf, Trit};
use std::{env, fs, path::Path, time::Instant};

const NONCE32: &str =
    "6fe3e686c0b4c5dbd18a550c0b28c10ec6cabd41b02367a1fb6747eac49f88d3";
const MERKLE288: &str =
    "601412f3d918c401b00df22a1b2799e50811fdbf2f1e87602211d7fb1fa6bb4b";
type Result<T> = std::result::Result<T, String>;

fn authentic(case: &str, bytes: &[u8]) -> Result<Cnf> {
    let (sha, variables, clauses) = match case {
        "nonce32" => (NONCE32, 239_112, 877_879),
        "merkle_nonce288" => (MERKLE288, 361_188, 1_325_357),
        _ => return Err("unknown frozen original Bitcoin source".to_owned()),
    };
    if hash_bytes(bytes) != sha {
        return Err("original Bitcoin SHA256d circuit fingerprint changed".to_owned());
    }
    let source = std::str::from_utf8(bytes).map_err(|_| "non-ASCII source CNF")?;
    let cnf = Cnf::parse(source)?;
    if cnf.variables != variables || cnf.clauses.len() != clauses
        || cnf.clauses.iter().any(Vec::is_empty)
    {
        return Err("original Bitcoin complete CNF dimensions differ".to_owned());
    }
    Ok(cnf)
}
fn load(
    case: &str, expected_source: &[u8], archive: &Path,
) -> Result<(BitcoinFrozen, Cnf, Vec<u8>)> {
    let gguf = fs::read(archive).map_err(|e| format!("GGUF read: {e}"))?;
    let frozen = thaw(&gguf)?;
    if frozen.source != expected_source {
        return Err("GGUF CNF source bytes differ from separate original reference".to_owned());
    }
    let cnf = authentic(case, &frozen.source)?;
    if frozen.states.len() != cnf.variables {
        return Err("GGUF state tensor cannot index every original CNF variable".to_owned());
    }
    Ok((frozen, cnf, gguf))
}
fn count_states(states: &[Trit]) -> (usize, usize, usize) {
    states.iter().fold((0, 0, 0), |(minus, zero, plus), state| {
        match state {
            Trit::Minus => (minus + 1, zero, plus),
            Trit::Zero => (minus, zero + 1, plus),
            Trit::Plus => (minus, zero, plus + 1),
        }
    })
}
fn step_positions(total: usize) -> Vec<usize> {
    let mut indices = std::collections::BTreeSet::new();
    for point in [1, 2, 3, 27, 28, 81, total / 2, total - 1, total] {
        if point > 0 && point <= total { indices.insert(point); }
    }
    for index in 0..27_usize {
        indices.insert(1 + ((index * 65_537) % total));
    }
    indices.into_iter().collect()
}
fn playback(case: &str, frozen: &BitcoinFrozen, gguf: &[u8]) -> Result<String> {
    let now = Instant::now();
    let mut trace = Vec::new();
    let positions = step_positions(frozen.states.len());
    for &variable in &positions {
        let (address, state) = play_variable(frozen, variable)?;
        if state != frozen.states[variable - 1] {
            return Err("Bobby Fischer playback differs from frozen trit".to_owned());
        }
        trace.extend_from_slice(
            format!("FISCHER|variable={variable}|address={address}|trit={}\n",
                    state.symbol()).as_bytes()
        );
    }
    let (minus, unknown, plus) = count_states(&frozen.states);
    let trace_root = hash_bytes(&trace);
    let root = hash_bytes(gguf);
    Ok(format!(
        "BTC_GGUF_FISCHER|schema=1|case={case}|gguf_v=3|tensor_count=2|\
         cnf_sha256={}|state_sha256={}|gguf_sha256={root}|\
         variables={}|known_false={minus}|unknown={unknown}|known_true={plus}|\
         fischer_reads={}|fischer_trace_sha256={trace_root}|\
         source_bytes={}|gguf_bytes={}|play_wall_ns={}|\
         original_cnf=EXACT|fischer_forward_reverse=PASS|\
         known_nonce_injected=NO|solver_search=NOT_RUN|status=PASS|json=0\n",
        frozen.source_sha256, frozen.state_sha256,
        frozen.states.len(), positions.len(), frozen.source.len(), gguf.len(),
        now.elapsed().as_nanos()
    ))
}
fn run() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 5 {
        return Err("usage: bitcoin_gguf_fischer <freeze|play|branch> <case> <original.cnf> <archive.gguf> [receipt.hbp | next.gguf <variable> <+|->]".to_owned());
    }
    let action = args[1].as_str();
    let case = &args[2];
    let source = fs::read(&args[3]).map_err(|e| format!("source CNF read: {e}"))?;
    let cnf = authentic(case, &source)?;
    let archive = Path::new(&args[4]);
    match action {
        "freeze" => {
            if args.len() != 5 {
                return Err("freeze accepts exactly one GGUF destination".to_owned());
            }
            let started = Instant::now();
            // Preserve the legitimate unknown initial assignment. The
            // published known genesis nonce is excluded from SAT state.
            let states = vec![Trit::Zero; cnf.variables];
            let bytes = freeze(&source, &states)?;
            let parsed = thaw(&bytes)?;
            if parsed.source != source || parsed.states != states {
                return Err("initial Bitcoin GGUF did not survive exact freeze/play".to_owned());
            }
            fs::write(archive, &bytes).map_err(|e| format!("GGUF save: {e}"))?;
            println!(
                "BTC_GGUF_FREEZE|schema=1|case={case}|gguf_sha256={}|\
                 original_cnf_sha256={}|variables={}|clauses={}|\
                 source_bytes={}|gguf_bytes={}|freeze_wall_ns={}|\
                 origin_trits=ALL_UNKNOWN|known_nonce_injected=NO|status=PASS|json=0",
                hash_bytes(&bytes), parsed.source_sha256,
                cnf.variables, cnf.clauses.len(),
                source.len(), bytes.len(), started.elapsed().as_nanos()
            );
        }
        "play" => {
            if args.len() != 6 {
                return Err("play requires a receipt destination".to_owned());
            }
            let (frozen, _, bytes) = load(case, &source, archive)?;
            let evidence = playback(case, &frozen, &bytes)?;
            fs::write(&args[5], &evidence).map_err(|e| format!("receipt write: {e}"))?;
            print!("{evidence}");
        }
        "branch" => {
            if args.len() != 8 {
                return Err("branch requires next.gguf, variable and polarity".to_owned());
            }
            let (frozen, _, _) = load(case, &source, archive)?;
            let variable: usize = args[6].parse()
                .map_err(|_| "noninteger SAT variable")?;
            if variable == 0 || variable > cnf.variables {
                return Err("variable outside original CNF".to_owned());
            }
            let polarity = match args[7].as_str() {
                "+" => Trit::Plus,
                "-" => Trit::Minus,
                _ => return Err("branch polarity must be + or -".to_owned()),
            };
            let (_, before) = play_variable(&frozen, variable)?;
            if before != Trit::Zero {
                return Err("cannot branch on already assigned frozen variable".to_owned());
            }
            let mut updated = frozen.states;
            updated[variable - 1] = polarity;
            let next = freeze(&source, &updated)?;
            let result = thaw(&next)?;
            let (_, after) = play_variable(&result, variable)?;
            if after != polarity || result.source_sha256 != frozen.source_sha256 {
                return Err("Fischer branch failed original source-preserving replay".to_owned());
            }
            let target = Path::new(&args[5]);
            fs::write(target, &next).map_err(|e| format!("branch GGUF save: {e}"))?;
            println!(
                "BTC_GGUF_BRANCH|case={case}|source_sha256={}|\
                 from_state_sha256={}|to_state_sha256={}|variable={variable}|\
                 before={}|after={}|next_gguf_sha256={}|\
                 candidate_branch=YES|SAT_result=NOT_CLAIMED|\
                 fischer_play=PASS|status=PASS|json=0",
                frozen.source_sha256, frozen.state_sha256, result.state_sha256,
                before.symbol(), after.symbol(), hash_bytes(&next)
            );
        }
        _ => return Err("expected freeze, play or branch".to_owned()),
    }
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("BTC_GGUF_FAIL|reason={error}|status=FAIL|json=0");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn representative_addresses_include_multiple_fischer_rooms() {
        let positions = step_positions(239_112);
        assert!(positions.contains(&1) && positions.contains(&27)
                && positions.contains(&28) && positions.contains(&239_112));
        assert!(positions.len() >= 27);
        let mut states = vec![Trit::Zero; 82];
        states[81] = Trit::Plus;
        let source = b"p cnf 82 1\n82 0\n";
        let frozen = thaw(&freeze(source, &states).expect("freeze")).expect("thaw");
        assert_eq!(play_variable(&frozen, 82).expect("address").1, Trit::Plus);
        states[81] = Trit::Minus;
        assert_ne!(thaw(&freeze(source, &states).expect("changed"))
                   .expect("thaw changed").state_sha256, frozen.state_sha256);
    }
}
