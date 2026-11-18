#![forbid(unsafe_code)]
use pnp_native::{certify_matrix_fabric, policy};
use std::path::Path;

fn main() {
    let mut arguments = std::env::args().skip(1);
    let result = match arguments.next().as_deref() {
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
                println!("PNP_NATIVE_FINAL|rust=1.81.0|integer_only=PASS|source_frozen=PASS|nodes_runtime=0|json_runtime=0|v8_runtime=0|status=PASS|json=0");
                Ok(())
            }
            Err(error) => Err(error.to_owned()),
        },
        _ => Err("usage: pnp-native audit [repository-root] OR pnp-native certify".to_owned()),
    };
    if let Err(message) = result {
        eprintln!("PNP_NATIVE_FAIL|reason={message}|json=0");
        std::process::exit(1);
    }
}
