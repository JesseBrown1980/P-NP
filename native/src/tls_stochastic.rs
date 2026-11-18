//! Source-scoped stochastic TLS memory experiment. Every pseudo-random event
//! is synthetic; the referenced PRL measured separate real qubit hardware.
//! Evaluates frozen train/validation/test, trained ternary memory and shuffles.
#![forbid(unsafe_code)]

use std::fs;
use std::path::Path;

use crate::{sha256::hash_bytes, tls_memory};

const S: i64 = 10_000;
const TRAIN: usize = 8_000;
const VALID: usize = 4_000;
const TEST: usize = 8_000;
const REPS: usize = 3;
const SEED_BASE: u32 = 0x7251_ab19;
const SEED_STEP: u32 = 0x9e37_79b9;
const VALID_XOR: u32 = 0xabcd_ef33;
const TEST_XOR: u32 = 0xdead_beef;
const SHUFFLE_XOR: u32 = 0xabc1_3579;
const PENALTY: u128 = 500_000_000;
const CAP: [usize; 5] = [0, 1, 3, 27, 81];
const LAGS: [usize; 5] = [1, 2, 4, 8, 16];
const MODES: [&str; 3] = ["IID_NULL", "ONE_TLS", "THREE_TLS"];
const SPEC: &str = include_str!("../fixtures/TLS_STOCHASTIC_CORPUS.hbi");
const ROWS: usize = 1 + 9 + 45 + 45 + 9 + 3 + 4 + 1;
const HEADER: &str = "HBPHEADER|schema=pnp.tls.stochastic.heldout.v1|source=PRL_DOI_INSPIRED|results=SYNTHETIC|seal=HBI-HBP-SHA-SH-HASH|json=0\n";
const INDEX: &str = "HBIHEADER|schema=pnp.tls.stochastic.heldout.index.v1|for=PNP_TLS_STOCHASTIC.hbp|seal=HBI-HBP-SHA-SH-HASH|json=0\n";
const TRACE_HEADER: &[u8] = b"PNP-TLS-STOCHASTIC-v1\0";

#[derive(Clone, Copy, Debug)]
struct Rng(u32);
impl Rng {
    fn next(&mut self) -> u32 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.0 = x;
        x
    }
}
#[derive(Clone, Debug)]
struct ThreeTrace {
    training: Vec<u8>,
    validation: Vec<u8>,
    heldout: Vec<u8>,
    shuffled: Vec<u8>,
}
fn generate(mode: &str, n: usize, seed: u32) -> Result<Vec<u8>, String> {
    let mut rng = Rng(seed);
    let mut hidden = [0_u8; 3];
    let cells = match mode {
        "IID_NULL" => 0,
        "ONE_TLS" => 1,
        "THREE_TLS" => 3,
        _ => return Err("unrecognized synthetic TLS generator".to_owned()),
    };
    let mut result = Vec::with_capacity(n);
    for _ in 0..n {
        let p = match mode {
            "IID_NULL" => 4500,
            "ONE_TLS" => 800 + 7500 * u32::from(hidden[0]),
            "THREE_TLS" => {
                1000 + 2500 * hidden.iter().map(|v| u32::from(*v)).sum::<u32>()
            }
            _ => return Err("generator mode invalid".to_owned()),
        };
        result.push(u8::from(rng.next() % 10_000 < p));
        for (index, item) in hidden.iter_mut().enumerate().take(cells) {
            if rng.next() % 10_000 < [800, 200, 500][index] {
                *item ^= 1;
            }
        }
    }
    Ok(result)
}
fn shuffle_count_preserving(original: &[u8], seed: u32) -> Vec<u8> {
    let mut out = original.to_vec();
    let mut rng = Rng(seed);
    for i in (1..out.len()).rev() {
        let j = (rng.next() as usize) % (i + 1);
        out.swap(i, j);
    }
    out
}
fn traces(mode: &str, rep: usize) -> Result<ThreeTrace, String> {
    let seed = SEED_BASE.wrapping_add((rep as u32).wrapping_mul(SEED_STEP));
    let training = generate(mode, TRAIN, seed)?;
    let validation = generate(mode, VALID, seed ^ VALID_XOR)?;
    let heldout = generate(mode, TEST, seed ^ TEST_XOR)?;
    let shuffled = shuffle_count_preserving(&heldout, seed ^ SHUFFLE_XOR);
    if heldout.iter().map(|&v|usize::from(v)).sum::<usize>()
        != shuffled.iter().map(|&v|usize::from(v)).sum::<usize>()
    {
        return Err("count-preserving shuffle changed the number of events".to_owned());
    }
    Ok(ThreeTrace { training, validation, heldout, shuffled })
}
#[derive(Clone, Copy, Debug, Default)]
struct Bucket { count: u32, success: u32 }
#[derive(Clone, Debug)]
struct Fit {
    prior: i64,
    mean_x: i64,
    beta: i64,
    tables: [Vec<Bucket>; 3],
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Keys { trit3: usize, trit27: usize, trit81: usize }
#[derive(Clone, Copy, Debug)]
struct Context {
    history: [u8; 5],
    ema: i64,
}
impl Context {
    fn new() -> Self { Self { history: [0; 5], ema: 5_000 } }
    fn keys(&self) -> Keys {
        let a = usize::from(self.history[0]) + usize::from(self.history[1]);
        let b = usize::from(self.history[1]) + usize::from(self.history[2]);
        let c = usize::from(self.history[2]) + usize::from(self.history[3]);
        let d = usize::from(self.history[3]) + usize::from(self.history[4]);
        Keys { trit3: a, trit27: a + 3*b + 9*c, trit81: a + 3*b + 9*c + 27*d }
    }
    fn observe(&mut self, outcome: u8) {
        self.history.rotate_right(1);
        self.history[0] = outcome;
        self.ema = (15*self.ema + S*i64::from(outcome)) / 16;
    }
}
fn fit(data: &[u8]) -> Result<Fit, String> {
    if data.len() != TRAIN || data.iter().any(|&y| y > 1) {
        return Err("training data missing or nonbinary".to_owned());
    }
    let prior = (i64::try_from(data.iter().map(|&v| u64::from(v)).sum::<u64>())
        .map_err(|_|"large training event count")? * S + (data.len() as i64)/2)
        / data.len() as i64;
    let mut contexts = Context::new();
    let mut table = [vec![Bucket::default();3],
                     vec![Bucket::default();27],
                     vec![Bucket::default();81]];
    let mut x = Vec::with_capacity(data.len());
    for &event in data {
        let keys = contexts.keys();
        for (i,k) in [keys.trit3,keys.trit27,keys.trit81].iter().enumerate() {
            table[i][*k].count += 1;
            table[i][*k].success += u32::from(event);
        }
        x.push(contexts.ema);
        contexts.observe(event);
    }
    let mean_x = x.iter().sum::<i64>() / data.len() as i64;
    let mut covariance = 0_i128;
    let mut variance = 0_i128;
    for (&input, &y) in x.iter().zip(data) {
        let dx = i128::from(input - mean_x);
        covariance += dx * i128::from(i64::from(y)*S - prior);
        variance += dx*dx;
    }
    let beta = if variance == 0 { 0 } else {
        i64::try_from((covariance*i128::from(S)/variance).clamp(0,20_000))
            .map_err(|_|"EMA regression coefficient outside range")?
    };
    Ok(Fit { prior, mean_x, beta, tables: table })
}
fn predict(model:usize, fit:&Fit,context:&Context)->Result<i64,String>{
    let keys=context.keys();
    let p=match model {
        0 => fit.prior,
        1 => fit.prior + fit.beta*(context.ema-fit.mean_x)/S,
        2..=4 => {
            let i=model-2;
            let key=[keys.trit3,keys.trit27,keys.trit81][i];
            let b=fit.tables[i][key];
            let pseudocount=[12_i64,48,96][i];
            (i64::from(b.success)*S+pseudocount*fit.prior)
                /(i64::from(b.count)+pseudocount)
        }
        _ => return Err("unsupported original ternary context capacity".to_owned()),
    };
    Ok(p.clamp(1,9_999))
}
fn score(data:&[u8], fitted:&Fit)->Result<[u128;5],String>{
    let mut sums=[0_u128;5];
    let mut context=Context::new();
    for &y in data {
        if y > 1 {return Err("held-out event outside Bernoulli alphabet".to_owned());}
        for (m, sum) in sums.iter_mut().enumerate() {
            let p=predict(m,fitted,&context)?;
            let delta=S*i64::from(y)-p;
            *sum+=(delta*delta) as u128;
        }
        context.observe(y);
    }
    Ok(sums)
}
fn select(validation:&[u128;5])->usize{
    (0..5).min_by_key(|&i|(validation[i]+PENALTY*(i as u128),i)).unwrap_or(0)
}
fn lag(data:&[u8], gap:usize)->Result<(usize,usize,usize,usize,i64),String>{
    if data.len()<=gap{return Err("no future events for requested lag".to_owned());}
    let mut yes_total=0_usize;
    let mut no_total=0_usize;
    let mut yes_event=0_usize;
    let mut no_event=0_usize;
    for i in 0..data.len()-gap{
        if data[i]==1{
            yes_total+=1;
            yes_event+=usize::from(data[i+gap]);
        }else{
            no_total+=1;
            no_event+=usize::from(data[i+gap]);
        }
    }
    if yes_total==0 || no_total==0 {return Err("invalid conditional event denominator".to_owned());}
    let positive=i64::try_from(yes_event*S as usize/yes_total).map_err(|_|"lag positive overflow")?;
    let negative=i64::try_from(no_event*S as usize/no_total).map_err(|_|"lag negative overflow")?;
    Ok((yes_event,yes_total,no_event,no_total,positive-negative))
}
fn required_spec()->Result<String,String>{
    let expected="HBIHEADER|schema=pnp.tls.stochastic.corpus.v1|source_doi=10.1103/j1r2-5ph7|scope=SYNTHETIC_PRECOMMITTED|json=0\nTLS_RANDOM|prng=XORSHIFT32|seed_base=0x7251AB19|seed_step=0x9E3779B9|train_xor=0x00000000|validation_xor=0xABCDEF33|test_xor=0xDEADBEEF|shuffle_xor=0xABC13579|json=0\nTLS_SAMPLE|train=8000|validation=4000|heldout=8000|repetitions=3|scenarios=IID_NULL,ONE_TLS,THREE_TLS|json=0\nTLS_GENERATOR|mode=IID_NULL|probability_q10000=4500|hidden_cells=0|json=0\nTLS_GENERATOR|mode=ONE_TLS|probability_low=800|probability_high=8300|flip_q10000=800|hidden_cells=1|json=0\nTLS_GENERATOR|mode=THREE_TLS|probability_base=1000|probability_per_active_cell=2500|flip_rates_q10000=200,500,900|hidden_cells=3|json=0\nTLS_PREDICTOR|capacities=0,1,3,27,81|zero=CONSTANT_TRAINED_RATE|one=TRAINED_EMA_ONE_SCALAR|three=LAST_TWO_EVENTS_TERNARY|twentyseven=THREE_OVERLAPPING_PAIR_TRITS|eightyone=FOUR_OVERLAPPING_PAIR_TRITS|unseen_smoothing=TRAINED_GLOBAL_PRIOR|json=0\nTLS_SELECTION|criterion=MIN_VALIDATION_BRIER_PLUS_FIXED_COMPLEXITY_PENALTY|penalty_per_model_index=500000000|heldout_labels=NEVER_TRAINED|shuffled_heldout=FISHER_YATES_COUNT_PRESERVING|json=0\n";
    if SPEC!=expected{return Err("published stochastic source contract drift".to_owned());}
    Ok(hash_bytes(SPEC.as_bytes()))
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Stats{
    pub rows:usize,
    pub traces:usize,
    pub models:usize,
    pub lag_records:usize,
    pub coupled_heldout_improvements:usize,
    pub null_baseline_selections:usize,
    pub shuffle_reversals:usize,
    pub source_bytes:usize,
}
type Bundle=(String,String,String,Vec<u8>);
fn make_bundle(prior_hbp_sha:&str) -> Result<(Bundle,Stats),String>{
    let spec_sha=required_spec()?;
    if prior_hbp_sha.len()!=64{return Err("previous DOI TLS proof root not pinned".to_owned());}
    let mut hbp=HEADER.to_owned();
    let mut hbi=INDEX.to_owned();
    let mut trace=TRACE_HEADER.to_vec();
    let mut previous=hash_bytes(b"PNP-TLS-STOCHASTIC-HELDOUT-2026");
    let mut seq=0_usize;
    let mut push=|tag:&str,id:&str,data:&str|{
        seq+=1;
        let body=format!("{tag}|seq={seq}|id={id}|prior_tls_sha256={prior_hbp_sha}|fixture_sha256={spec_sha}|{data}|prev_hash={previous}");
        let sha=hash_bytes(body.as_bytes());
        let hash=hash_bytes(format!("{previous}|{sha}").as_bytes());
        let row=format!("{body}|receipt_sha={sha}|hash={hash}|json=0\n");
        hbi.push_str(&format!("HBI|seq={seq}|id={id}|offset={}|len={}|hbp_sha256={}|sh={}|hash={hash}|json=0\n",
            hbp.len(),row.len(),hash_bytes(row.as_bytes()),&sha[..16]));
        hbp.push_str(&row);
        previous=hash;
    };
    push("TLS_STOCH_SOURCE","prl_generator",
        "doi=10.1103/j1r2-5ph7|regimes=IID_NULL,ONE_TLS,THREE_TLS|seed=frozen|generator=SYNTHETIC_HIDDEN_MARKOV|nonstationary=NO|quantum_hardware=NO|source_paper_data=NOT_USED|g2_physical=NOT_MEASURED");
    let mut improved=0;
    let mut null_baseline=0;
    let mut reversed=0;
    let mut total_lags=0;
    let mut total_models=0;
    let mut results=[[0_u128;3];3];
    for (mode_index,mode) in MODES.iter().enumerate(){
        for rep in 0..REPS{
            let case_id=format!("{mode}_seed{rep}");
            let batch=traces(mode,rep)?;
            let trained=fit(&batch.training)?;
            let validation=score(&batch.validation,&trained)?;
            let test=score(&batch.heldout,&trained)?;
            let shuffled=score(&batch.shuffled,&trained)?;
            let selected=select(&validation);
            let label=if mode_index==0{"NEGATIVE_MARKOV"}else{"HIDDEN_RESERVOIR"};
            let offset=trace.len();
            trace.push(mode_index as u8);
            trace.push(rep as u8);
            for events in [&batch.training,&batch.validation,&batch.heldout,&batch.shuffled]{
                trace.extend_from_slice(&(events.len() as u32).to_le_bytes());
                trace.extend_from_slice(events);
            }
            let trace_len=trace.len()-offset;
            push("TLS_STOCH_TRACE",&case_id,&format!(
                "condition={label}|mode={mode}|rep={rep}|train=8000|validation=4000|test=8000|shuffle=8000|segment_offset={offset}|segment_len={trace_len}|segment_sha256={}|train_sha256={}|validation_sha256={}|test_sha256={}|shuffle_sha256={}|event_count_test={}|event_count_shuffle={}|source=FROZEN_SYNTHETIC|temporal_model=HIDDEN_MARKOV",
                hash_bytes(&trace[offset..]),hash_bytes(&batch.training),
                hash_bytes(&batch.validation),hash_bytes(&batch.heldout),hash_bytes(&batch.shuffled),
                batch.heldout.iter().map(|v|usize::from(*v)).sum::<usize>(),
                batch.shuffled.iter().map(|v|usize::from(*v)).sum::<usize>()
            ));
            for (i, capacity) in CAP.iter().enumerate(){
                push("TLS_STOCH_MODEL",&format!("{case_id}_n{capacity}"),&format!(
                    "mode={mode}|rep={rep}|capacity={capacity}|model_index={i}|parameters_learned=TRAIN_ONLY|validation_brier_sum={}|test_brier_sum={}|shuffled_brier_sum={}|events_scored_test={TEST}|qscale={S}|training_prior={}|trained_ema_beta={}|table_state_bytes={}|model_metadata_bytes=NOT_FULLY_CHARGED|source_training=INDEPENDENT_SEED",
                    validation[i],test[i],shuffled[i],trained.prior,trained.beta,
                    match i{0=>8,1=>32,2=>3*8,3=>27*8,_=>81*8}
                ));
                total_models+=1;
            }
            for lag_n in LAGS {
                let (yes_count,yes_total,no_count,no_total,difference)=lag(&batch.heldout,lag_n)?;
                let (_,_,_,_,scrambled)=lag(&batch.shuffled,lag_n)?;
                push("TLS_STOCH_LAG",&format!("{case_id}_lag{lag_n}"),&format!(
                    "mode={mode}|rep={rep}|lag={lag_n}|cond_yes_num={yes_count}|cond_yes_den={yes_total}|cond_no_num={no_count}|cond_no_den={no_total}|conditional_difference_q10000={difference}|shuffled_difference_q10000={scrambled}|g2_like=EVENT_CONDITIONAL_ONLY|g2_physical=NO"
                ));
                total_lags+=1;
                if mode_index==0 && difference.abs()>400 {
                    return Err("IID null produced a large apparent lag-correlation".to_owned());
                }
                if lag_n==1 && mode_index!=0 && difference<1000{
                    return Err("source-inspired persistent reservoir failed lag-1 control".to_owned());
                }
            }
            let baseline=test[0];
            let heldout_gain=i128::try_from(baseline).map_err(|_|"baseline Brier overflow")?
                -i128::try_from(test[selected]).map_err(|_|"selected Brier overflow")?;
            let shuffled_gain=i128::try_from(shuffled[0]).map_err(|_|"shuffle Brier overflow")?
                -i128::try_from(shuffled[selected]).map_err(|_|"selected shuffle Brier overflow")?;
            if mode_index==0{
                if selected!=0 {return Err("IID null generated false temporal-memory selection".to_owned());}
                null_baseline+=1;
            }else{
                if selected==0 || heldout_gain<=0 {
                    return Err("coupled heldout memory gain not reproduced".to_owned());
                }
                improved+=1;
                if shuffled_gain>=0 {return Err("shuffled hidden-history negative control did not reject signal".to_owned());}
                reversed+=1;
            }
            results[mode_index][0]+=baseline;
            results[mode_index][1]+=test[selected];
            results[mode_index][2]+=shuffled[selected];
            push("TLS_STOCH_SELECT",&case_id,&format!(
                "mode={mode}|rep={rep}|selector=VALIDATION_ONLY|chosen_capacity={}|chosen_model_index={selected}|validation_penalty_per_index={PENALTY}|test_brier_base={baseline}|test_brier_selected={}|heldout_gain_squared_q={heldout_gain}|shuffled_brier_base={}|shuffled_brier_selected={}|shuffle_gain_squared_q={shuffled_gain}|no_holdout_training=PASS|time_order_shuffle=COUNT_PRESERVING|performance_source=SYNTHETIC",
                CAP[selected],test[selected],shuffled[0],shuffled[selected]
            ));
        }
    }
    for (mode_index,mode) in MODES.iter().enumerate() {
        let b=results[mode_index][0];
        let s=results[mode_index][1];
        push("TLS_STOCH_AGGREGATE",mode,&format!(
            "mode={mode}|replicates=3|test_events={}|baseline_brier_q2={b}|selected_brier_q2={s}|selected_gain_brier_q2={}|selected_shuffled_brier_q2={}|status={}",
            REPS*TEST,
            i128::try_from(b).map_err(|_|"aggregate baseline overflow")?
                -i128::try_from(s).map_err(|_|"aggregate selected overflow")?,
            results[mode_index][2],
            if mode_index==0{"NO_FALSE_MEMORY"}else{"HELDOUT_HISTORY_GAIN"}
        ));
    }
    for (id,summary,full,replay,expected) in [
        ("summary_only",true,false,false,true),
        ("latent_missing",true,false,false,false),
        ("latent_replay",true,true,true,true),
        ("uncommitted",false,true,true,false)
    ] {
        let retention=if id=="summary_only" || id=="uncommitted" {
            tls_memory::Retention::SummaryOnly
        }else{
            tls_memory::Retention::ExactReplayRequired
        };
        if tls_memory::may_discard(retention,summary,full,replay)!=expected{
            return Err("TLS temporal GC state deletion admission violated".to_owned());
        }
        push("TLS_STOCH_GC",id,&format!(
            "summary_committed={}|complete_state={}|independent_replay={}|discard_eligible={}|deletion_executed=NO|manual_approval=NOT_REQUIRED|source_archive=PROTECTED",
            u8::from(summary),u8::from(full),u8::from(replay),u8::from(expected)
        ));
    }
    if (total_lags,total_models,null_baseline,improved,reversed)
        !=(45,45,3,6,6){
        return Err("stochastic TLS acceptance counts inconsistent".to_owned());
    }
    push("TLS_STOCH_SUMMARY","accepted",&format!(
        "regimes=3|replicates={REPS}|trace_groups=9|candidate_models=45|lag_records={total_lags}|null_baseline={null_baseline}|coupled_heldout_gain={improved}|coupled_shuffle_reversal={reversed}|train_events={TRAIN}|validation_events={VALID}|heldout_events={TEST}|model_selection=VALIDATION_ONLY|predictive_metric=HELDOUT_BRIER|uncertainty=REPLICATE_DISPERSION_ONLY|trace_bytes={}|trace_sha256={}|source_hardware=NO|source_jump_data=NO|true_g2=NOT_MEASURED|gc_exact_replay=RETAIN_UNTIL_VERIFIED",
        trace.len(),hash_bytes(&trace)
    ));
    // The last receipt append completes the closure's mutable capture.
    if seq!=ROWS{return Err("TLS source-bound row census changed".to_owned());}
    let root=format!(
        "TLS_STOCH_ANCHOR|rows={}|trace_groups=9|source_sha256={spec_sha}|prior_hbp_sha256={prior_hbp_sha}|trace_sha256={}|trace_bytes={}|hbp_sha256={}|hbi_sha256={}|terminal_hash={previous}|seal=HBI-HBP-SHA-SH-HASH",
        ROWS,hash_bytes(&trace),trace.len(),hash_bytes(hbp.as_bytes()),hash_bytes(hbi.as_bytes())
    );
    let anchor=format!("{root}|checksum={}|json=0\n",hash_bytes(root.as_bytes()));
    let source_bytes=trace.len();
    Ok(((hbp,hbi,anchor,trace),Stats{
        rows:seq,traces:9,models:total_models,lag_records:total_lags,
        coupled_heldout_improvements:improved,null_baseline_selections:null_baseline,
        shuffle_reversals:reversed,source_bytes
    }))
}
fn read_prior(input:&Path)->Result<String,String>{
    if tls_memory::verify(input)?.rows!=196{
        return Err("accepted exact fixed-point TLS pilot root changed".to_owned());
    }
    let original=fs::read(input.join("PNP_TLS.hbp"))
        .map_err(|e|format!("original TLS pilot receipt unavailable: {e}"))?;
    Ok(hash_bytes(&original))
}
pub fn attest(prior:&Path,out:&Path)->Result<Stats,String>{
    let root=read_prior(prior)?;
    let ((hbp,hbi,anchor,trace),stats)=make_bundle(&root)?;
    fs::create_dir_all(out).map_err(|e|e.to_string())?;
    for (name,bytes) in [
        ("PNP_TLS_STOCHASTIC.hbp",hbp.as_bytes()),
        ("PNP_TLS_STOCHASTIC.hbi",hbi.as_bytes()),
        ("PNP_TLS_STOCHASTIC.anchor",anchor.as_bytes()),
        ("PNP_TLS_STOCHASTIC_TRACE.bin",trace.as_slice())
    ]{
        fs::write(out.join(name),bytes)
            .map_err(|e|format!("write stochastic TLS evidence {name}: {e}"))?;
    }
    Ok(stats)
}
pub fn verify(prior:&Path,out:&Path)->Result<Stats,String>{
    let root=read_prior(prior)?;
    let ((hbp,hbi,anchor,trace),stats)=make_bundle(&root)?;
    for (name,expected) in [
        ("PNP_TLS_STOCHASTIC.hbp",hbp.as_bytes()),
        ("PNP_TLS_STOCHASTIC.hbi",hbi.as_bytes()),
        ("PNP_TLS_STOCHASTIC.anchor",anchor.as_bytes()),
        ("PNP_TLS_STOCHASTIC_TRACE.bin",trace.as_slice())
    ]{
        if fs::read(out.join(name))
            .map_err(|e|format!("read stochastic TLS evidence {name}: {e}"))?!=expected{
            return Err(format!("stochastic TLS evidence differs: {name}"));
        }
    }
    Ok(stats)
}
#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn real_independent_stochastic_heldout_passes_all_three_regimes(){
        let ((hbp,hbi,anchor,trace),stats)=make_bundle(&hash_bytes(b"accepted-prior")).unwrap();
        assert_eq!((stats.rows,stats.traces,stats.models,stats.lag_records),(117,9,45,45));
        assert_eq!(stats.null_baseline_selections,3);
        assert_eq!(stats.coupled_heldout_improvements,6);
        assert_eq!(stats.shuffle_reversals,6);
        assert_eq!(hbp.lines().filter(|line|line.starts_with("TLS_STOCH_MODEL|")).count(),45);
        assert_eq!(hbi.lines().filter(|line|line.starts_with("HBI|")).count(),117);
        assert!(anchor.contains("seal=HBI-HBP-SHA-SH-HASH"));
        assert!(trace.len()>200_000);
    }
    #[test]
    fn original_event_counts_and_past_only_prediction_are_sealed(){
        for mode in MODES{
            for rep in 0..REPS{
                let batch=traces(mode,rep).unwrap();
                assert_eq!(batch.training.len(),TRAIN);
                assert_eq!(batch.validation.len(),VALID);
                assert_eq!(batch.heldout.len(),TEST);
                assert_eq!(batch.heldout.iter().filter(|v|**v==1).count(),
                           batch.shuffled.iter().filter(|v|**v==1).count());
                let trained=fit(&batch.training).unwrap();
                let s=score(&batch.validation,&trained).unwrap();
                assert!(s.iter().all(|x|*x>0));
            }
        }
    }
    #[test]
    fn ternary_context_and_hidden_state_do_not_read_current_event(){
        let mut c=Context::new();
        assert_eq!(c.keys(),Keys{trit3:0,trit27:0,trit81:0});
        c.observe(1);
        assert_eq!(c.keys().trit3,1);
        c.observe(1);
        assert_eq!(c.keys().trit3,2);
        let model=fit(&generate("ONE_TLS",TRAIN,0x1234_5678).unwrap()).unwrap();
        for capacity in 0..CAP.len() {
            let p=predict(capacity,&model,&c).unwrap();
            assert!((1..S).contains(&p));
        }
        assert_eq!(select(&[20,15,10,5,0]),0); // Validation complexity guard
    }
}
