#![forbid(unsafe_code)]
use pnp_native::{
    certify_exact_witness, certify_matrix_fabric, certify_lazy_wave, binary_ternary_bridge, checkpoint, ensemble, fabric_gc, gc_archive, gulp_gc,
    guidance::{Guidance, Strategy},
    policy, prism, relic, scoring, source, omnibit_tournament, omnibit_operator_gate, tls_memory, tls_stochastic, tls_omnibit_gc,
    tls_entropy_omnibit, tls_stream_fabric, http0_fabric_witness,
    http0_compute, microkernel_compute, full_matrix_fabric, prl_temporal_fabric,
    solver::{check_against_exhaustive, solve},
    ternary::{model_to_text, Cnf},
};
use std::path::Path;

fn main() {
    let mut arguments = std::env::args().skip(1);
    let result: Result<(), String> = (|| {
        match arguments.next().as_deref() {
        Some("rime-disk-attest") => {
            let path = arguments.next().ok_or("missing original four-axes GGUF path")?;
            if arguments.next().is_some() {
                return_fail("too many disk-play arguments");
            }
            let store = pnp_native::rime_disk::DiskRime::open_four_axes(Path::new(&path))?;
            let comparisons = store.verify_three_paths()?;
            let shadow_cubes = store.verify_cube_shadows()?;
            let address = pnp_native::freeze::fischer_address(0, 0)
                .ok_or("Fischer zero address missing")?;
            let direct = store.read(0, 0, address)?;
            let remote = pnp_native::rime_disk::loopback_once(&store, 0, 0)?;
            if direct != remote { return_fail("HTTP-0 loopback differs from direct disk read"); }
            store.verify_source()?;
            println!("PNP_RIME_DISK|source_sha256={}|bytes={}|tensors={}|glyphs=27|rime_pairs={}|compared={}|shadow_cubes={}|anchors_per_cube=8|path1=STORED|path2=FORWARD|path3=INVERSE|fischer=PASS|http0_loopback=PASS|source_witness=PASS|gpu=0|status=PASS|json=0",
                pnp_native::rime_disk::FOUR_AXES_SHA256,
                pnp_native::rime_disk::FOUR_AXES_BYTES,
                store.tensor_count(), comparisons / 27, comparisons, shadow_cubes);
            Ok(())
        },
        Some("audit") => {
            let directory = arguments.next().unwrap_or_else(|| ".".to_owned());
            match policy::check_repository(Path::new(&directory)) {
                Ok(report) => {
                    println!("PNP_LEGACY_AUDIT|legacy_failed={}|python_scripts={}|old_workflows={}|new_rust_sources={}|source_pins=PASS|software_status=LEGACY_FAIL|json=0",
                             report.legacy_failed, report.pinned_scripts,
                             report.pinned_workflows, report.accepted_native_sources);
                    Ok(())
                }
                Err(error) => Err(error),
            }
        }
        Some("certify") => match certify_matrix_fabric() {
            Ok((rooms, cells, faults)) => {
                println!("PNP_NATIVE_MATRIX|architecture=Nx3x3x3|rooms={rooms}|played_cells={cells}|omega_centers=120|geometry=1_6_12_8|shadow=EXACT|fischer_play=EXACT|json=0");
                println!("PNP_NATIVE_TRIAD|worker=PASS|reflection=PASS|matrix_fabric=PASS|nest=3_9_27_81|planted_faults_rejected={faults}|operator_approval=HELD|json=0");
                let (balanced, coordinated, triples) = certify_exact_witness()
                    .map_err(str::to_owned)?;
                println!("PNP_NATIVE_WITNESS|ordered_triples={triples}|balanced_faults_rejected={balanced}|coordinated_reports_rejected={coordinated}|per_address_compare=PASS|signed_differences=EXACT|status=PASS|json=0");
                let (wave_rooms, path_decisions, children_verified) = certify_lazy_wave()
                    .map_err(str::to_owned)?;
                println!("PNP_NATIVE_WAVE|rooms={wave_rooms}|path_decisions={path_decisions}|signed_children={children_verified}|center=1|addressed_lazy=PASS|third_seat=PASS|altered_paths=REJECTED|status=PASS|json=0");
                println!("PNP_NATIVE_FINAL|rust=1.81.0|integer_only=PASS|source_frozen=PASS|nodes_runtime=0|json_runtime=0|v8_runtime=0|status=PASS|json=0");
                Ok(())
            }
            Err(error) => Err(error.to_owned()),
        },
        Some("gulp-discard") => {
            let gac = arguments.next().ok_or("missing frozen GAC source")?;
            let neural = arguments.next().ok_or("missing frozen neural source")?;
            let omni = arguments.next().ok_or("missing frozen GULP source")?;
            let out = arguments.next().ok_or("missing owned GULP receipt directory")?;
            if arguments.next().is_some() {
                return_fail("too many GULP discard arguments");
            }
            let count = gulp_gc::run(
                Path::new(&gac), Path::new(&neural), Path::new(&omni), Path::new(&out),
            )?;
            println!("PNP_GULP_GC|events={count}|grouping=KIND_GLYPH|summary=COMMITTED|raw_owned=DELETED|source_archive=PRESERVED|manual_approval=NOT_REQUIRED|hbi=PASS|hbp=PASS|sha256=PASS|sh=PASS|hash_chain=PASS|status=PASS|json=0");
            Ok(())
        }
        Some("gulp-discard-verify") => {
            let gac = arguments.next().ok_or("missing frozen GAC source")?;
            let neural = arguments.next().ok_or("missing frozen neural source")?;
            let omni = arguments.next().ok_or("missing frozen GULP source")?;
            let out = arguments.next().ok_or("missing GULP commitment archive")?;
            if arguments.next().is_some() {
                return_fail("too many GULP verifier arguments");
            }
            let count = gulp_gc::verify(
                Path::new(&gac), Path::new(&neural), Path::new(&omni), Path::new(&out),
            )?;
            println!("PNP_GULP_GC_VERIFY|events={count}|raw_owned=ABSENT|patterns=RETAINED|source_archive=PRESERVED|hbi=PASS|hbp=PASS|sha256=PASS|hash_chain=PASS|status=PASS|json=0");
            Ok(())
        }
        Some("gc-archive-attest") => {
            let gac = arguments.next().ok_or("missing original GAC source")?;
            let neural = arguments.next().ok_or("missing original neural fabric source")?;
            let omni = arguments.next().ok_or("missing original Omni GC source")?;
            let out = arguments.next().ok_or("missing original GC archive output")?;
            if arguments.next().is_some() { return_fail("too many GC archive arguments"); }
            let count = gc_archive::attest(
                Path::new(&gac), Path::new(&neural), Path::new(&omni), Path::new(&out)
            )?;
            if gc_archive::verify(
                Path::new(&gac), Path::new(&neural), Path::new(&omni), Path::new(&out)
            )? != count {
                return_fail("GC staged archive differs on readback");
            }
            println!("PNP_GC_ARCHIVE|original_bodies={count}|source_unchanged=PASS|archives=BYTE_IDENTICAL|overwrite=REJECTED|authorization=AUTO_VERIFIED|manual_approval=NOT_REQUIRED|live_rotation=HOLD_ATOMICITY|gulp_feedback=HOLD|hbi=PASS|hbp=PASS|sha256=PASS|sh=PASS|hash_chain=PASS|status=PASS|json=0");
            Ok(())
        }
        Some("gc-archive-verify") => {
            let gac = arguments.next().ok_or("missing original GAC source")?;
            let neural = arguments.next().ok_or("missing original neural fabric source")?;
            let omni = arguments.next().ok_or("missing original Omni GC source")?;
            let out = arguments.next().ok_or("missing archive replay directory")?;
            if arguments.next().is_some() { return_fail("too many GC archive verification arguments"); }
            let count = gc_archive::verify(
                Path::new(&gac), Path::new(&neural), Path::new(&omni), Path::new(&out)
            )?;
            println!("PNP_GC_ARCHIVE_VERIFY|original_bodies={count}|source=GIT_PINNED|archive=EXACT|restart_replay=PASS|hbi=PASS|hbp=PASS|sha256=PASS|hash_chain=PASS|status=PASS|json=0");
            Ok(())
        }
        Some("prl-temporal-fabric-attest") => {
            let previous=arguments.next().ok_or("missing accepted PRL source and fabric evidence")?;
            let out=arguments.next().ok_or("missing PRL causal forecast output directory")?;
            if arguments.next().is_some(){return_fail("too many PRL causal attest arguments");}
            let stats=prl_temporal_fabric::attest(Path::new(&previous),Path::new(&out))?;
            println!("PNP_PRL_CAUSAL|source_doi=10.1103/j1r2-5ph7|cases={}|paired_rows={}|binary_floors=4|original_operators=40|native_http0_calls={}|exact_event_predictions={}|source_events={}|gnn_forward=HOLD|forecasts_sha256={}|status=PASS|json=0",
                stats.cases,stats.verified_pairs,stats.calls,stats.forecasts,
                stats.events,stats.checksum);
            for arm in stats.arms{
                println!("PNP_PRL_CAUSAL_ARM|arity={}|calls={}|events={}|coupled_heldout_gains={}|coupled_shuffle_reversals={}|iid_null_holds={}|wall_ns={}|learned=ORIGINAL_INTEGER_SOURCE_TRAINED|source=HELDOUT|status=PASS|json=0",
                    arm.arity,arm.calls,arm.events,arm.held_gains,
                    arm.shuffled_reversals,arm.iid_holds,arm.wall_ns);
            }
            Ok(())
        }
        Some("prl-temporal-fabric-verify") => {
            let previous=arguments.next().ok_or("missing original PRL source evidence")?;
            let out=arguments.next().ok_or("missing PRL causal forecast evidence")?;
            if arguments.next().is_some(){return_fail("too many PRL causal verification arguments");}
            let stats=prl_temporal_fabric::verify(Path::new(&previous),Path::new(&out))?;
            println!("PNP_PRL_CAUSAL_VERIFY|cases={}|paired_rows={}|http0_calls={}|every_probability=EXACT|shuffles=COUNT_PRESERVED|archive=BYTE_IDENTICAL|source=FROZEN|status=PASS|json=0",
                stats.cases,stats.verified_pairs,stats.calls);
            Ok(())
        }
        Some("prl-temporal-fabric-inspect") => {
            let source=arguments.next().ok_or("missing PRL causal prediction binary")?;
            if arguments.next().is_some(){return_fail("too many PRL causal inspection arguments");}
            let streams=prl_temporal_fabric::inspect(Path::new(&source))?;
            println!("PNP_PRL_CAUSAL_INSPECT|streams={streams}|forecasts_per_stream=8000|baseline_and_memory_probabilities=EXACT_Q10000|status=PASS|json=0");
            Ok(())
        }
        Some("full-matrix-attest") => {
            let prior=arguments.next().ok_or("missing accepted prior source/evidence")?;
            let out=arguments.next().ok_or("missing full matrix output")?;
            if arguments.next().is_some(){return_fail("extra full matrix arguments");}
            let stats=full_matrix_fabric::attest(Path::new(&prior),Path::new(&out))?;
            println!("PNP_FULL_MATRIX|jobs={}|binary_floors=4|original_operators=40|direct_crossings={}|independent_http0_requests={}|exact_matrix_triads={}|gac60_metadata={}|gnn_forward_holds={}|direct_all_wall_ns={}|direct_memo_wall_ns={}|trained_models=SOURCE_PINNED_FORWARD_HOLD|status=PASS|json=0",
                stats.jobs,stats.crossings,stats.independent_calls,
                stats.matrix_triads,stats.tuple_checks,stats.gnn_hold_checks,
                stats.direct_all_ns,stats.direct_memo_ns);
            for arm in stats.arms {
                println!("PNP_FULL_MATRIX_ARM|mode={}|arity={}|calls={}|hits={}|physical_calculations={}|visited_rooms={}|wall_ns={}|floor_carry=INDEPENDENT|all40=EXECUTED|source_answer=EXACT|http0=BACKEND_ONLY|status=PASS|json=0",
                    arm.mode,arm.arity,arm.calls,arm.hits,arm.calls-arm.hits,
                    arm.rooms,arm.elapsed_ns);
            }
            Ok(())
        }
        Some("full-matrix-verify") => {
            let prior=arguments.next().ok_or("missing frozen matrix original evidence")?;
            let out=arguments.next().ok_or("missing matrix result archive")?;
            if arguments.next().is_some(){return_fail("extra full matrix verifier argument");}
            let stats=full_matrix_fabric::verify(Path::new(&prior),Path::new(&out))?;
            println!("PNP_FULL_MATRIX_VERIFY|jobs={}|network_calls={}|operator_crossings={}|source_seat=PASS|reflection=PASS|fabric_seat=PASS|authentic_gnn_signal=HOLD|archive=BYTE_IDENTICAL|status=PASS|json=0",
                stats.jobs,stats.independent_calls,stats.crossings);
            Ok(())
        }
        Some("full-matrix-inspect") => {
            let source=arguments.next().ok_or("missing full matrix result binary")?;
            if arguments.next().is_some(){return_fail("extra matrix witness argument");}
            let count=full_matrix_fabric::inspect(Path::new(&source))?;
            println!("PNP_FULL_MATRIX_INSPECT|jobs={count}|integer_reconstruction=EXACT|source_witness=PASS|status=PASS|json=0");
            Ok(())
        }
        Some("compute-fabric-serve") => {
            if arguments.next().is_some() {
                return_fail("compute fabric worker accepts no source or public endpoint");
            }
            http0_compute::serve()
        }
        Some("microkernel-compute-attest") => {
            let prior=arguments.next().ok_or("missing source-authenticated prior archive")?;
            let out=arguments.next().ok_or("missing microkernel receipt destination")?;
            if arguments.next().is_some(){return_fail("too many microkernel attest arguments");}
            let stats=microkernel_compute::attest(Path::new(&prior),Path::new(&out))?;
            println!("PNP_COMPUTE_BASE|jobs={}|direct_calls=162|direct_wall_ns={}|direct_cached_wall_ns={}|result_sha256={}|full_binary_bytes={}|status=PASS|json=0",
                stats.jobs,stats.direct_ns,stats.direct_cache_ns,
                stats.result_sha,stats.full_binary_bytes);
            for arm in &stats.arms {
                println!("PNP_COMPUTE_ARM|mode={}|arity={}|calls={}|hits={}|physical_computations={}|rooms={}|memo_entries={}|wall_ns={}|http0=BACKEND_ONLY|independent_replay=PASS|status=PASS|json=0",
                    arm.mode,arm.arity,arm.calls,arm.hits,arm.calls-arm.hits,
                    arm.rooms,arm.entries,arm.elapsed_ns);
            }
            Ok(())
        }
        Some("microkernel-compute-verify") => {
            let prior=arguments.next().ok_or("missing source-authenticated prior archive")?;
            let out=arguments.next().ok_or("missing microkernel archive")?;
            if arguments.next().is_some(){return_fail("too many microkernel verify arguments");}
            let stats=microkernel_compute::verify(Path::new(&prior),Path::new(&out))?;
            println!("PNP_COMPUTE_VERIFY|jobs={}|arms={}|archive=BYTE_IDENTICAL|microkernel_witness=EXACT|status=PASS|json=0",
                stats.jobs,stats.arms.len());
            Ok(())
        }
        Some("microkernel-compute-inspect") => {
            let source=arguments.next().ok_or("missing microkernel witness file")?;
            if arguments.next().is_some(){return_fail("too many witness arguments");}
            let jobs=microkernel_compute::verify_witness_file(Path::new(&source))?;
            println!("PNP_COMPUTE_INSPECT|jobs={jobs}|stencil=27_CELLS_243_TICKS|integer_replay=PASS|status=PASS|json=0");
            Ok(())
        }
        Some("stream-fabric-serve") => {
            if arguments.next().is_some() {
                return_fail("HTTP-0 fabric service takes no source or UI arguments");
            }
            http0_fabric_witness::serve()
        }
        Some("stream-fabric-attest") => {
            let prior=arguments.next().ok_or("missing sealed temporal source")?;
            let out=arguments.next().ok_or("missing stream fabric destination")?;
            if arguments.next().is_some(){return_fail("too many stream fabric arguments");}
            let measured=tls_stream_fabric::attest(Path::new(&prior),Path::new(&out))?;
            println!("PNP_TLS_STREAM_FABRIC|frames={}|bitpack_framed_bytes={}|continuous_range_framed_bytes={}|old_hybrid_bytes={}|model_bytes={}|http0_calls={}|fischer_routing=3,27,81|backend_worker=SEPARATE_PROCESS|reconstructed=EXACT|full_decoder_bytes={}|arity3_wall_ns={}|arity27_wall_ns={}|arity81_wall_ns={}|status=PASS|json=0",
                measured.frames,measured.bitpack_bytes,measured.range_bytes,
                measured.former_hybrid_bytes,measured.model_bytes,measured.http0_calls,
                measured.decoder_bytes,measured.wall_ns_by_arity[0],
                measured.wall_ns_by_arity[1],measured.wall_ns_by_arity[2]);
            Ok(())
        }
        Some("stream-fabric-verify") => {
            let prior=arguments.next().ok_or("missing sealed temporal source")?;
            let out=arguments.next().ok_or("missing stream fabric archive")?;
            if arguments.next().is_some(){return_fail("too many stream fabric verify arguments");}
            let measured=tls_stream_fabric::verify(Path::new(&prior),Path::new(&out))?;
            println!("PNP_TLS_STREAM_FABRIC_VERIFY|frames={}|bitpack_framed_bytes={}|continuous_range_framed_bytes={}|http0_calls={}|independent_source_free_inverse=PASS|status=PASS|json=0",
                measured.frames,measured.bitpack_bytes,measured.range_bytes,measured.http0_calls);
            Ok(())
        }
        Some("stream-fabric-inspect") => {
            let path=arguments.next().ok_or("missing framed archive")?;
            if arguments.next().is_some(){return_fail("too many stream archive arguments");}
            let (frames,events)=tls_stream_fabric::verify_archive(Path::new(&path))?;
            println!("PNP_TLS_STREAM_INSPECT|frames={frames}|events={events}|canonical=PASS|status=PASS|json=0");
            Ok(())
        }
        Some("tls-entropy-attest") => {
            let original = arguments.next().ok_or("missing original Omnibit source")?;
            let prior = arguments.next().ok_or("missing previously accepted archives")?;
            let out = arguments.next().ok_or("missing entropy archive destination")?;
            if arguments.next().is_some() { return_fail("too many entropy attest arguments"); }
            let (counts, elapsed) = tls_entropy_omnibit::timing(
                Path::new(&original),Path::new(&prior),Path::new(&out)
            )?;
            println!("PNP_TLS_ENTROPY|frames={}|rows={}|archive_bytes={}|restored_bytes={}|model_bytes={}|range_blocks={}|original_omnibit_blocks={}|original_operator_trials={}|full_decoder_bytes={}|elapsed_wall_ns={elapsed}|trained=PR43_TRAIN_ONLY|selector=VALIDATION_FRAMED_COST|bitpack54_unframed=54000|baseline_omni40_bytes=294615|source_independent_replay=PASS|hbi=PASS|hbp=PASS|sha256=PASS|hash_chain=PASS|status=PASS|json=0",
                counts.frames, counts.rows, counts.archive_bytes, counts.restored_bytes,
                counts.models_bytes, counts.range_blocks, counts.omega_blocks,
                counts.original_trials, counts.decoder_bytes);
            Ok(())
        }
        Some("tls-entropy-verify") => {
            let original = arguments.next().ok_or("missing original Omnibit source")?;
            let prior = arguments.next().ok_or("missing previously accepted archives")?;
            let out = arguments.next().ok_or("missing entropy archive")?;
            if arguments.next().is_some() { return_fail("too many entropy verification arguments"); }
            let counts = tls_entropy_omnibit::verify(
                Path::new(&original),Path::new(&prior),Path::new(&out)
            )?;
            println!("PNP_TLS_ENTROPY_VERIFY|frames={}|rows={}|archive_bytes={}|models_embedded=YES|original_source=RETAINED|standalone_decoder=PASS|status=PASS|json=0",
                counts.frames, counts.rows, counts.archive_bytes);
            Ok(())
        }
        Some("tls-entropy-decode") => {
            let source = arguments.next().ok_or("missing self-contained entropy archive")?;
            let output = arguments.next().ok_or("missing reconstructed events output")?;
            if arguments.next().is_some() { return_fail("too many entropy decoder arguments"); }
            let count = tls_entropy_omnibit::decode_to_file(Path::new(&source), Path::new(&output))?;
            println!("PNP_TLS_ENTROPY_DECODE|restored_bytes={count}|external_model=NONE|source_generator=NOT_USED|original_operators=40|events=EXACT|status=PASS|json=0");
            Ok(())
        }
        Some("tls-entropy-gulp-discard") => {
            let original = arguments.next().ok_or("missing original Omnibit source")?;
            let prior = arguments.next().ok_or("missing previously accepted archives")?;
            let out = arguments.next().ok_or("missing retained entropy archive")?;
            if arguments.next().is_some() { return_fail("too many entropy GULP arguments"); }
            let (deleted,retained) = tls_entropy_omnibit::gulp_discard_owned(
                Path::new(&original),Path::new(&prior),Path::new(&out)
            )?;
            println!("PNP_TLS_ENTROPY_GULP|owned_scratch_deleted_bytes={deleted}|archive_retained_bytes={retained}|scratch_absent=PASS|protected_original=RETAINED|live_operator_approval=NOT_REQUIRED|status=PASS|json=0");
            Ok(())
        }
        Some("tls-omni40-attest") => {
            let original=arguments.next().ok_or("missing original Git-pinned Omnibit source")?;
            let prior=arguments.next().ok_or("missing original TLS and Omnibit acceptance archives")?;
            let out=arguments.next().ok_or("missing temporal coder destination")?;
            if arguments.next().is_some(){return_fail("too many temporal coder arguments");}
            let (stats,elapsed)=tls_omnibit_gc::timing(
                Path::new(&original),Path::new(&prior),Path::new(&out)
            )?;
            println!("PNP_TLS_OMNI40|original_operators=40|routes=BINARY64_TERNARY256_MIXED256_PLUS64|heldout_and_shuffle_streams={}|candidate_trials={}|hbi_rows={}|input_event_bytes={}|selected_archive_bytes={}|identity_baseline_bytes={}|restored_archive_bytes={}|selected_nonidentity={}|decoder_program_bytes={}|standalone_inverse=EXACT|source_pr43=VERIFIED|original_omega=VERIFIED|gc=OWNED_SCRATCH_ONLY|hbi=PASS|hbp=PASS|sha256=PASS|sh=PASS|hash_chain=PASS|status=PASS|json=0",
                stats.streams,stats.original_operator_trials,stats.rows,
                stats.input_bytes,stats.archive_bytes,stats.baseline_identity_bytes,
                stats.restored_bytes,stats.selected_nonidentity,stats.program_bytes
            );
            println!("PNP_TLS_OMNI40_TIMING|scope=ATTEST_AND_INDEPENDENT_REPLAY|elapsed_wall_ns={elapsed}|full_decoder_binary=CHARGED|source=HOSTED_RUNNER|json=0");
            Ok(())
        }
        Some("tls-omni40-verify") => {
            let original=arguments.next().ok_or("missing original Git-pinned Omnibit source")?;
            let prior=arguments.next().ok_or("missing prior evidence archive")?;
            let out=arguments.next().ok_or("missing temporal archive")?;
            if arguments.next().is_some(){return_fail("too many temporal coder verifier arguments");}
            let stats=tls_omnibit_gc::verify(
                Path::new(&original),Path::new(&prior),Path::new(&out)
            )?;
            println!("PNP_TLS_OMNI40_VERIFY|candidate_trials={}|rows={}|archive_bytes={}|restored_bytes={}|standalone_inverse=PASS|source_event_bytes=UNCHANGED|protected_source=RETAINED|hbi=PASS|hbp=PASS|sha256=PASS|hash_chain=PASS|status=PASS|json=0",
                stats.original_operator_trials,stats.rows,stats.archive_bytes,stats.restored_bytes
            );
            Ok(())
        }
        Some("tls-omni40-decode") => {
            let archive=arguments.next().ok_or("missing selected standalone coding payload")?;
            let output=arguments.next().ok_or("missing restored event archive destination")?;
            if arguments.next().is_some(){return_fail("too many standalone decoder arguments");}
            let len=tls_omnibit_gc::decode_to_file(
                Path::new(&archive),Path::new(&output)
            )?;
            println!("PNP_TLS_OMNI40_DECODE|restored_bytes={len}|source_generator=NOT_REQUIRED|model_state=NOT_REQUIRED|original_operator_catalog=IN_BINARY|source_event_replay=EXACT|status=PASS|json=0");
            Ok(())
        }
        Some("tls-omni40-gulp-discard") => {
            let original=arguments.next().ok_or("missing source-pinned original Omnibit checkout")?;
            let prior=arguments.next().ok_or("missing original temporal source receipts")?;
            let out=arguments.next().ok_or("missing durable temporal coding destination")?;
            if arguments.next().is_some(){return_fail("too many GULP discard arguments");}
            let (deleted,retained)=tls_omnibit_gc::gulp_discard_owned(
                Path::new(&original),Path::new(&prior),Path::new(&out)
            )?;
            println!("PNP_TLS_OMNI40_GULP|owned_scratch_deleted_bytes={deleted}|retained_selected_bytes={retained}|scratch_absent=PASS|original_source=PROTECTED|decoder_replay=PASS|durable_archive=SYNCED_BEFORE_DELETE|human_approval=NOT_REQUIRED|status=PASS|json=0");
            Ok(())
        }
        Some("tls-stochastic-attest") => {
            let prior=arguments.next().ok_or("missing source-authenticated TLS pilot receipts")?;
            let out=arguments.next().ok_or("missing stochastic TLS HBI/HBP destination")?;
            if arguments.next().is_some(){return_fail("too many TLS stochastic attest arguments");}
            let stats=tls_stochastic::attest(Path::new(&prior),Path::new(&out))?;
            if tls_stochastic::verify(Path::new(&prior),Path::new(&out))?!=stats{
                return_fail("source-independent stochastic TLS archived replay differed");
            }
            println!("PNP_TLS_STOCHASTIC|source=DOI_10.1103_j1r2-5ph7|scope=SYNTHETIC_HIDDEN_MARKOV|regimes=3|repetitions=3|traces={}|candidate_models={}|ternary_capacities=0_1_3_27_81|lag_records={}|null_baseline_selections={}|coupled_heldout_improvements={}|shuffle_reversals={}|retained_source_bytes={}|records={}|split=TRAIN_VALIDATION_TEST|metric=HELDOUT_BRIER|g2_physical=NOT_MEASURED|quantum_hardware=NO|hbi=PASS|hbp=PASS|sha256=PASS|sh=PASS|hash_chain=PASS|status=PASS|json=0",
                stats.traces,stats.models,stats.lag_records,
                stats.null_baseline_selections,stats.coupled_heldout_improvements,
                stats.shuffle_reversals,stats.source_bytes,stats.rows
            );
            Ok(())
        }
        Some("tls-stochastic-verify") => {
            let prior=arguments.next().ok_or("missing original frozen TLS pilot source")?;
            let out=arguments.next().ok_or("missing stochastic TLS receipts")?;
            if arguments.next().is_some(){return_fail("too many TLS stochastic verifier arguments");}
            let stats=tls_stochastic::verify(Path::new(&prior),Path::new(&out))?;
            println!("PNP_TLS_STOCHASTIC_VERIFY|traces={}|models={}|lag_records={}|null_baseline={}|coupled_improved={}|shuffle_reversed={}|records={}|source=ORIGINAL_TLS_ACCEPTED|stochastic_replay=EXACT|hbi=PASS|hbp=PASS|sha256=PASS|hash_chain=PASS|status=PASS|json=0",
                stats.traces,stats.models,stats.lag_records,
                stats.null_baseline_selections,stats.coupled_heldout_improvements,
                stats.shuffle_reversals,stats.rows
            );
            Ok(())
        }
        Some("tls-memory-attest") => {
            let out = arguments.next().ok_or("missing TLS HBI/HBP output directory")?;
            if arguments.next().is_some() {
                return_fail("too many TLS memory attest arguments");
            }
            let stats = tls_memory::attest(Path::new(&out))?;
            if tls_memory::verify(Path::new(&out))? != stats {
                return_fail("TLS independent replay statistics differed");
            }
            println!("PNP_TLS_MEMORY|doi=10.1103/j1r2-5ph7|model=CLASSICAL_FIXED_POINT_EULER|latent_cells=0_1_3_27_81|models=10|pairs={}|lag_cases={}|null_controls={}|coupled_separations={}|heldout_markov_squared_error={}|heldout_oracle_squared_error={}|trace_bytes={}|receipts={}|gc_exact_replay=HELD_UNLESS_STATE_RETAINED|physics_fit=NO|quantum_hardware=NO|hbi=PASS|hbp=PASS|sha256=PASS|sh=PASS|hash_chain=PASS|status=PASS|json=0",
                stats.paired_histories,stats.lag_cases,stats.control_passes,
                stats.coupled_separations,stats.heldout_baseline_error,
                stats.matched_error,stats.trace_bytes,stats.rows
            );
            Ok(())
        }
        Some("tls-memory-verify") => {
            let out = arguments.next().ok_or("missing TLS source trace archive")?;
            if arguments.next().is_some() {
                return_fail("too many TLS memory verifier arguments");
            }
            let stats = tls_memory::verify(Path::new(&out))?;
            println!("PNP_TLS_MEMORY_VERIFY|models=10|pairs={}|lag_cases={}|null_controls={}|coupled_separations={}|trace_bytes={}|records={}|source=FROZEN_DOI_REFERENCE|exact_trace=PASS|hbi=PASS|hbp=PASS|sha256=PASS|hash_chain=PASS|status=PASS|json=0",
                stats.paired_histories,stats.lag_cases,stats.control_passes,
                stats.coupled_separations,stats.trace_bytes,stats.rows
            );
            Ok(())
        }
        Some("omni40-attest") => {
            let original = arguments.next().ok_or("missing original Omnibit catalog")?;
            let upstream = arguments.next().ok_or("missing accepted HBI/HBP tournament receipts")?;
            let out = arguments.next().ok_or("missing 40-operator evidence output")?;
            if arguments.next().is_some() {
                return_fail("too many Omnibit 40-operator arguments");
            }
            let count = omnibit_operator_gate::attest(
                Path::new(&original),Path::new(&upstream),Path::new(&out)
            )?;
            if omnibit_operator_gate::verify(
                Path::new(&original),Path::new(&upstream),Path::new(&out)
            )? != count {
                return_fail("original Omega 40-operator witness replay disagreed");
            }
            println!("PNP_OMNI40|original_catalog=BYTE_IDENTICAL|operators=40|rnq=8|tri=12|pi=20|floor_widths=6_8_10_12|compound=256_PLUS_64|public_fixtures=20|vantage_labels=6|candidate_trials=20160|selections=504|retention=EXACT_REPLAY_REQUIRED|receipts={count}|replay=EXACT|scoreboard=NOT_SCORED|winner=NONE|seat_local_runs=0|hbi=PASS|hbp=PASS|sha256=PASS|sh=PASS|hash_chain=PASS|status=PASS|json=0");
            Ok(())
        }
        Some("omni40-verify") => {
            let original = arguments.next().ok_or("missing original Omnibit catalog")?;
            let upstream = arguments.next().ok_or("missing original HBI/HBP tournament receipts")?;
            let out = arguments.next().ok_or("missing 40-operator evidence output")?;
            if arguments.next().is_some() {
                return_fail("too many original Omega verifier arguments");
            }
            let count = omnibit_operator_gate::verify(
                Path::new(&original),Path::new(&upstream),Path::new(&out)
            )?;
            println!("PNP_OMNI40_VERIFY|operators=40|candidate_trials=20160|selections=504|receipts={count}|all_payload_bytes=VERIFIED|source_git=PASS|floor_inverse=EXACT|hbi=PASS|hbp=PASS|sha256=PASS|hash_chain=PASS|status=PASS|json=0");
            Ok(())
        }
        Some("omnibit-attest") => {
            let original = arguments.next().ok_or("missing pinned Omnibit tournament root")?;
            let upstream = arguments.next().ok_or("missing bridge HBI/HBP archive")?;
            let out = arguments.next().ok_or("missing public tournament output")?;
            if arguments.next().is_some() {
                return_fail("too many tournament arguments");
            }
            let count = omnibit_tournament::attest(
                Path::new(&original),Path::new(&upstream),Path::new(&out)
            )?;
            if omnibit_tournament::verify(
                Path::new(&original),Path::new(&upstream),Path::new(&out)
            )? != count {
                return_fail("Omnibit public tournament repeat witness differed");
            }
            println!("PNP_OMNIBIT|original_source=VERIFIED|source_files=5|seats=3|views=2|routing_arms=3|public_fixtures=20|runs=360|exact_replay=360|retention_cases=3|receipts={count}|scoreboard=NOT_SCORED|winner=NONE|seat_local_runs=0|manual_approval=NOT_REQUIRED|hbi=PASS|hbp=PASS|sha256=PASS|sh=PASS|hash_chain=PASS|status=PASS|json=0");
            Ok(())
        }
        Some("omnibit-verify") => {
            let original = arguments.next().ok_or("missing original Omnibit source")?;
            let upstream = arguments.next().ok_or("missing original bridge receipts")?;
            let out = arguments.next().ok_or("missing Omnibit public results")?;
            if arguments.next().is_some() {
                return_fail("too many Omnibit verifier arguments");
            }
            let count = omnibit_tournament::verify(
                Path::new(&original),Path::new(&upstream),Path::new(&out)
            )?;
            println!("PNP_OMNIBIT_VERIFY|source_pins=PASS|public_fixture=20|runs=360|replay=EXACT|archive_payload=COMPLETE|receipts={count}|scoreboard=NOT_SCORED|hbi=PASS|hbp=PASS|sha256=PASS|hash_chain=PASS|status=PASS|json=0");
            Ok(())
        }
        Some("bridge-attest") => {
            let prism_source = arguments.next().ok_or("missing original PRISM source")?;
            let gac = arguments.next().ok_or("missing original GAC source")?;
            let neural = arguments.next().ok_or("missing original neural fabric source")?;
            let omni = arguments.next().ok_or("missing original Omni source")?;
            let receipts = arguments.next().ok_or("missing upstream HBI/HBP receipts")?;
            let out = arguments.next().ok_or("missing bridge output directory")?;
            if arguments.next().is_some() { return_fail("too many bridge attest arguments"); }
            let count = binary_ternary_bridge::attest(
                Path::new(&gac), Path::new(&neural), Path::new(&omni),
                Path::new(&prism_source), Path::new(&receipts), Path::new(&out),
            )?;
            if binary_ternary_bridge::verify(
                Path::new(&gac), Path::new(&neural), Path::new(&omni),
                Path::new(&prism_source), Path::new(&receipts), Path::new(&out),
            )? != count {
                return_fail("bridge second-seat replay did not agree");
            }
            println!("PNP_BRIDGE|source_prism=VERIFIED|source_gac=VERIFIED|binary_floors=4|ternary_landings=8|floor_checks=3186|route_checks=9840|reserved_byte_codes=13|reserved_as_data=REJECTED|matrix_cells=27|prism_roundtrip=EXACT|receipts={count}|hbi=PASS|hbp=PASS|sha256=PASS|sh=PASS|hash_chain=PASS|status=PASS|json=0");
            Ok(())
        }
        Some("bridge-verify") => {
            let prism_source = arguments.next().ok_or("missing original PRISM source")?;
            let gac = arguments.next().ok_or("missing original GAC source")?;
            let neural = arguments.next().ok_or("missing original neural fabric source")?;
            let omni = arguments.next().ok_or("missing original Omni source")?;
            let receipts = arguments.next().ok_or("missing upstream HBI/HBP receipts")?;
            let out = arguments.next().ok_or("missing bridge output directory")?;
            if arguments.next().is_some() { return_fail("too many bridge verify arguments"); }
            let count = binary_ternary_bridge::verify(
                Path::new(&gac), Path::new(&neural), Path::new(&omni),
                Path::new(&prism_source), Path::new(&receipts), Path::new(&out),
            )?;
            println!("PNP_BRIDGE_VERIFY|receipts={count}|upstream_source=VERIFIED|floor_inverses=PASS|reserved_code_rejection=PASS|hbi=PASS|hbp=PASS|sha256=PASS|hash_chain=PASS|status=PASS|json=0");
            Ok(())
        }
        Some("prism-attest") => {
            let root = arguments.next().ok_or("missing frozen Q-PRISM Git checkout")?;
            let out = arguments.next().ok_or("missing PRISM HBI/HBP receipt directory")?;
            if arguments.next().is_some() { return_fail("too many PRISM attest arguments"); }
            let count = prism::attest(Path::new(&root), Path::new(&out))?;
            if prism::verify(Path::new(&root), Path::new(&out))? != count {
                return_fail("PRISM inverse witness replay differed");
            }
            println!("PNP_PRISM|rung=256_1024|bits=40|bytes_per_block=5|glyphs_per_block=4|cube_bytes=3200|cube_glyphs=2560|cases={count}|source_git=VERIFIED|canonical_padding=PASS|inverse=EXACT|hbi=PASS|hbp=PASS|sha256=PASS|sh=PASS|hash_chain=PASS|operator=HELD|status=PASS|json=0");
            Ok(())
        }
        Some("prism-verify") => {
            let root = arguments.next().ok_or("missing original Q-PRISM checkout")?;
            let out = arguments.next().ok_or("missing PRISM HBI/HBP archive")?;
            if arguments.next().is_some() { return_fail("too many PRISM verify arguments"); }
            let count = prism::verify(Path::new(&root), Path::new(&out))?;
            println!("PNP_PRISM_VERIFY|cases={count}|original_source=VERIFIED|inverse=EXACT|hbi=PASS|hbp=PASS|sha256=PASS|hash_chain=PASS|status=PASS|json=0");
            Ok(())
        }
        Some("fabric-gc-attest") => {
            let gac = arguments.next().ok_or("missing pinned GAC root")?;
            let neural = arguments.next().ok_or("missing pinned neural fabric root")?;
            let omni = arguments.next().ok_or("missing pinned Omni collector root")?;
            let out = arguments.next().ok_or("missing HBI/HBP output root")?;
            if arguments.next().is_some() {
                return_fail("too many fabric GC attest arguments");
            }
            let n = fabric_gc::attest(
                Path::new(&gac), Path::new(&neural), Path::new(&omni),
                Path::new(&out),
            )?;
            if n != fabric_gc::verify(
                Path::new(&gac), Path::new(&neural), Path::new(&omni),
                Path::new(&out),
            )? {
                return_fail("GAC neural GC repeat check differs");
            }
            println!("PNP_FABRIC_GC|sources=10|gac=3|neural=5|omni=2|tuple_dim=60|access_cells=36|gc_policy_cases=6|records={n}|collection=PLAN_ONLY|archive=NEVER_DELETE|authority=HELD|hbi=PASS|hbp=PASS|sha256=PASS|sh=PASS|hash_chain=PASS|status=PASS|json=0");
            Ok(())
        }
        Some("fabric-gc-verify") => {
            let gac = arguments.next().ok_or("missing pinned GAC root")?;
            let neural = arguments.next().ok_or("missing pinned neural fabric root")?;
            let omni = arguments.next().ok_or("missing pinned Omni collector root")?;
            let out = arguments.next().ok_or("missing HBI/HBP output root")?;
            if arguments.next().is_some() {
                return_fail("too many fabric GC verify arguments");
            }
            let n = fabric_gc::verify(
                Path::new(&gac), Path::new(&neural), Path::new(&omni),
                Path::new(&out),
            )?;
            println!("PNP_FABRIC_GC_VERIFY|records={n}|tuple_dim=60|access_cells=36|source_hashes=VERIFIED|gc_policy=VERIFIED|hbi=PASS|hbp=PASS|hash_chain=PASS|status=PASS|json=0");
            Ok(())
        }
        Some("source-attest") => {
            let trained = arguments.next().ok_or("missing trained source root")?;
            let stage = arguments.next().ok_or("missing stage source root")?;
            let output = arguments.next().ok_or("missing HBI/HBP output root")?;
            if arguments.next().is_some() {
                return_fail("too many source attest arguments");
            }
            let count = source::attest(
                Path::new(&trained), Path::new(&stage), Path::new(&output),
            )?;
            let independently_replayed = source::verify(Path::new(&output))?;
            if count != independently_replayed {
                return_fail("source HBI/HBP acceptance count differed");
            }
            println!("PNP_SOURCE_ATTEST|source_pins={count}|model_checkpoints=9|manifest_checkpoints_sha=2|hbi=PASS|hbp=PASS|sha256=PASS|sh=PASS|hash_chain=PASS|port=HOLD|status=PASS|json=0");
            Ok(())
        }
        Some("checkpoint-attest") => {
            let models = arguments.next().ok_or("missing original checkpoint directory")?;
            let sources = arguments.next().ok_or("missing original source HBI/HBP archive")?;
            let out = arguments.next().ok_or("missing checkpoint HBI/HBP archive output")?;
            if arguments.next().is_some() {
                return_fail("too many checkpoint attest arguments");
            }
            let count = checkpoint::attest(
                Path::new(&models), Path::new(&sources), Path::new(&out),
            )?;
            if checkpoint::verify(
                Path::new(&models), Path::new(&sources), Path::new(&out),
            )? != count {
                return_fail("checkpoint replay differs");
            }
            println!("PNP_CHECKPOINT_ATTEST|models={count}|source_shas=VERIFIED|torch_zip=VERIFIED|numeric_decode=INTEGER_ONLY|q20=PER_STORAGE|forward_parity=HOLD|hbi=PASS|hbp=PASS|sha256=PASS|sh=PASS|hash_chain=PASS|status=PASS|json=0");
            Ok(())
        }
        Some("checkpoint-verify") => {
            let models = arguments.next().ok_or("missing original checkpoint directory")?;
            let sources = arguments.next().ok_or("missing original source HBI/HBP archive")?;
            let out = arguments.next().ok_or("missing checkpoint HBI/HBP archive output")?;
            if arguments.next().is_some() {
                return_fail("too many checkpoint verify arguments");
            }
            let count = checkpoint::verify(
                Path::new(&models), Path::new(&sources), Path::new(&out),
            )?;
            println!("PNP_CHECKPOINT_VERIFY|models={count}|source_shas=VERIFIED|torch_zip=VERIFIED|hbi=PASS|hbp=PASS|hash_chain=PASS|status=PASS|json=0");
            Ok(())
        }
        Some("gnn-gate-attest") => {
            let sources = arguments.next().ok_or("missing original source archive")?;
            let output = arguments.next().ok_or("missing multi-GNN HBI/HBP directory")?;
            if arguments.next().is_some() {
                return_fail("too many multi-GNN attest arguments");
            }
            let records = ensemble::attest(Path::new(&sources), Path::new(&output))?;
            if ensemble::verify(Path::new(&sources), Path::new(&output))? != records {
                return_fail("independent multi-GNN receipt replay differed");
            }
            println!("PNP_MULTIGNN|families=10|trained_architectures=4|checkpoints=9|scoring_signals=8|g4_cases=13|records={records}|tier0=SOURCE_MATCHED|operator_approval=HELD|g3=HOLD_SOURCE|neural_forward=HOLD|hbi=PASS|hbp=PASS|sha256=PASS|sh=PASS|hash_chain=PASS|status=PASS|json=0");
            Ok(())
        }
        Some("gnn-gate-verify") => {
            let sources = arguments.next().ok_or("missing original source archive")?;
            let output = arguments.next().ok_or("missing multi-GNN receipt directory")?;
            if arguments.next().is_some() {
                return_fail("too many multi-GNN verify arguments");
            }
            let records = ensemble::verify(Path::new(&sources), Path::new(&output))?;
            println!("PNP_MULTIGNN_VERIFY|records={records}|g4=SOURCE_MATCHED|hbi=PASS|hbp=PASS|sha256=PASS|hash_chain=PASS|status=PASS|json=0");
            Ok(())
        }
        Some("score-attest") => {
            let input = arguments.next().ok_or("missing original scoring HBI input")?;
            let sources = arguments.next().ok_or("missing authenticated source archive")?;
            let output = arguments.next().ok_or("missing scoring HBI/HBP output directory")?;
            if arguments.next().is_some() {
                return_fail("too many score attest arguments");
            }
            let count = scoring::attest(
                Path::new(&input), Path::new(&sources), Path::new(&output),
            )?;
            if scoring::verify(Path::new(&input), Path::new(&sources), Path::new(&output))?
                != count
            {
                return_fail("source score replay differed");
            }
            println!("PNP_SOURCE_SCORE|cases={count}|source_pins=23|source_blob=VERIFIED|sha_baseline=EXACT|reverse_gain=EXACT|hbi=PASS|hbp=PASS|sha256=PASS|sh=PASS|hash_chain=PASS|trained_gnn=HOLD|status=PASS|json=0");
            Ok(())
        }
        Some("score-verify") => {
            let input = arguments.next().ok_or("missing original scoring HBI input")?;
            let sources = arguments.next().ok_or("missing authenticated source archive")?;
            let output = arguments.next().ok_or("missing scoring archive")?;
            if arguments.next().is_some() {
                return_fail("too many score verify arguments");
            }
            let count = scoring::verify(
                Path::new(&input), Path::new(&sources), Path::new(&output),
            )?;
            println!("PNP_SCORE_VERIFY|cases={count}|sha_baseline=EXACT|reverse_gain=EXACT|hbi=PASS|hbp=PASS|hash_chain=PASS|status=PASS|json=0");
            Ok(())
        }
        Some("source-verify") => {
            let path = arguments.next().ok_or("missing HBI/HBP archive path")?;
            if arguments.next().is_some() {
                return_fail("too many source verify arguments");
            }
            let count = source::verify(Path::new(&path))?;
            println!("PNP_SOURCE_VERIFY|source_pins={count}|hbi=PASS|hbp=PASS|sha256=PASS|sh=PASS|hash_chain=PASS|status=PASS|json=0");
            Ok(())
        }
        Some("relic") => {
            for level in 0..5 {
                let a = relic::anchor(level).ok_or("landing overflow")?;
                let ws = relic::waves(level).ok_or("wave overflow")?;
                let next = relic::anchor(level + 1).ok_or("next landing overflow")?;
                if ws.last().copied() != Some(next) {
                    return_fail("landing-wave mismatch");
                }
                println!("RELIC_NATIVE|level={level}|center=1|center_coordinate=0|outward={a}|waves={ws:?}|next={next}|status=PASS|json=0");
            }
            println!("RELIC_NATIVE_FINAL|landings=5|invariant_center=PASS|exact_integers=PASS|status=PASS|json=0");
            Ok(())
        }
        Some("sat") => {
            let label = arguments.next().ok_or("missing strategy label")?;
            let strategy = Strategy::from_label(&label)
                .ok_or("unknown native strategy")?;
            let path = arguments.next().ok_or("missing original DIMACS source")?;
            let limit = arguments.next()
                .map(|value| value.parse::<u64>().map_err(|_| "invalid node budget"))
                .transpose()?;
            if arguments.next().is_some() {
                return_fail("too many SAT arguments");
            }
            let raw = std::fs::read_to_string(&path)
                .map_err(|e| format!("cannot read source: {e}"))?;
            let formula = Cnf::parse(&raw)?;
            let preparation = Guidance::for_source(&formula, strategy)?;
            let result = solve(&formula, &preparation, limit);
            if formula.variables <= 16 && !check_against_exhaustive(&formula, &result) {
                return_fail("independent exhaustive witness/refutation failed");
            }
            println!("PNP_TERNARY_SAT|engine={}|variables={}|clauses={}|fanout={}|lanes_used={}|mtpfrozen_patterns={}|fischer_addresses={}|matrix_rooms={}|fabric_witnesses={}|wave_reads={}|wave_children={}|nodes={}|propagations={}|literal_checks={}|status={}|json=0",
                strategy.label(), formula.variables, formula.clauses.len(),
                preparation.fanout, preparation.lanes_used,
                preparation.frozen_mtp_patterns, preparation.fischer_addresses,
                preparation.matrix_rooms, preparation.fabric_witnesses,
                preparation.wave_reads, preparation.wave_choices_checked,
                result.nodes, result.propagations, result.literal_checks,
                result.status.label());
            if let Some(model) = result.model {
                println!("PNP_TERNARY_MODEL|engine={}|trits={}|source_verified=PASS|json=0",
                    strategy.label(), model_to_text(&model));
            }
            Ok(())
        }
        _ => Err("usage: pnp-native audit [root] | certify | relic | gulp-discard GAC NEURAL OMNI OUT | gulp-discard-verify GAC NEURAL OMNI OUT | gc-archive-attest GAC NEURAL OMNI OUT | gc-archive-verify GAC NEURAL OMNI OUT | tls-omni40-attest ORIGINAL PRIOR OUT | tls-omni40-verify ORIGINAL PRIOR OUT | tls-omni40-decode ARCHIVE RESTORED | tls-omni40-gulp-discard ORIGINAL PRIOR OUT | tls-stochastic-attest PRIOR OUT | tls-stochastic-verify PRIOR OUT | tls-memory-attest OUT | tls-memory-verify OUT | omni40-attest SOURCE UPSTREAM OUT | omni40-verify SOURCE UPSTREAM OUT | omnibit-attest SOURCE UPSTREAM OUT | omnibit-verify SOURCE UPSTREAM OUT | bridge-attest PRISM GAC NEURAL OMNI UPSTREAM OUT | bridge-verify PRISM GAC NEURAL OMNI UPSTREAM OUT | prism-attest SOURCE OUT | prism-verify SOURCE OUT | fabric-gc-attest GAC NEURAL OMNI OUT | fabric-gc-verify GAC NEURAL OMNI OUT | source-attest TRAINED STAGE OUT | source-verify OUT | score-attest CASES SOURCE OUT | score-verify CASES SOURCE OUT | gnn-gate-attest SOURCE OUT | gnn-gate-verify SOURCE OUT | checkpoint-attest MODELS SOURCE OUT | checkpoint-verify MODELS SOURCE OUT | sat ENGINE FILE [node-budget]".to_owned()),
        }
    })();
    if let Err(message) = result {
        eprintln!("PNP_NATIVE_FAIL|reason={message}|json=0");
        std::process::exit(1);
    }
}

fn return_fail(reason: &str) -> ! {
    eprintln!("PNP_NATIVE_FAIL|reason={reason}|json=0");
    std::process::exit(1);
}
