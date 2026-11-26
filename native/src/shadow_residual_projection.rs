//! Bounded shadow-projection control using the existing native FrozenMatrix.
use crate::freeze::FrozenMatrix;
use crate::matrix::{Matrix,CELLS};
use crate::triad::{wave_cascade,wave_cascade_balanced,Fault,Seat};

fn truth(clauses: &[Vec<i32>], a: usize) -> bool {
    clauses.iter().all(|c| c.iter().any(|&v| {
        (a & (1 << (v.unsigned_abs()-1)) != 0) == (v>0)
    }))
}
fn hidden_xor(charge: bool) -> Vec<Vec<i32>> {
    // Boundary a,b,c = 1,2,3. Internal t=a OR b, u=a AND b,
    // z=t AND NOT u. c=z XOR charge.
    let mut clauses = vec![
        vec![-1,4],vec![-2,4],vec![1,2,-4],
        vec![1,-5],vec![2,-5],vec![-1,-2,5],
        vec![4,-6],vec![-5,-6],vec![-4,5,6],
    ];
    if charge {
        clauses.extend([vec![6,3],vec![-6,-3]]);
    } else {
        clauses.extend([vec![-6,3],vec![6,-3]]);
    }
    clauses
}
#[test]
fn compact_gate_projection_with_complementary_native_shadow() {
    // Three 27-cell rooms, indexed by boundary assignment (0..7)
    // and lane (z=0, z=1, reserved). Third room duplicates even charge
    // as an explicit control. No whole-instance enumeration is claimed.
    let clauses = [hidden_xor(false),hidden_xor(true),hidden_xor(false)];
    let mut rooms = Vec::new();
    let mut witnesses = Vec::new();
    for cs in &clauses {
        let mut cells = [0i64;CELLS];
        let mut choices = [None;8];
        for a in (0..64).rev() {
            if truth(cs,a) {
                let boundary = a & 7;
                let lane = (a >> 5) & 1;
                cells[boundary+9*lane] += 1;
                choices[boundary] = Some(a);
            }
        }
        rooms.push(cells);
        witnesses.push(choices);
    }
    let m = Matrix::from_rooms(rooms).expect("rooms");
    let frozen = FrozenMatrix::freeze(&m);
    assert!(frozen.verify());
    for room in 0..3 {
        for a in 0..8usize {
            let charge = room == 1;
            let expected = a.count_ones()%2 == u32::from(charge);
            let t = frozen.reconstruct_triple(room,a).expect("triple");
            assert_eq!(t.iter().sum::<i64>(),i64::from(expected));
            assert_eq!(frozen.projection(room).expect("shadow")[a],i128::from(expected));
            let p = frozen.play(room,a).expect("addressed play");
            assert_eq!(p.shadow_value,i128::from(expected));
            match witnesses[room][a] {
                Some(x) => {
                    assert!(expected && truth(&clauses[room],x));
                    assert_eq!(x&7,a);
                    assert_eq!(t[(x>>5)&1],1);
                }
                None => assert!(!expected),
            }
        }
    }
    assert!(wave_cascade(&frozen,None).expect("three seats").accepted);
    for seat in [Seat::Worker,Seat::Reflection,Seat::MatrixFabric] {
        assert!(!wave_cascade_balanced(&frozen,Fault{level:0,seat}).expect("fault trace").accepted);
    }
    // Independent pair projections cannot distinguish opposite parity.
    for (i,j) in [(0,1),(0,2),(1,2)] {
        let pair = |charge: u32| {
            (0..8usize).filter(|a| a.count_ones()%2==charge)
                .fold(0u32,|mask,a| mask | (1 << (((a>>i)&1)+2*((a>>j)&1))))
        };
        assert_eq!(pair(0),15);
        assert_eq!(pair(1),15);
    }
}
