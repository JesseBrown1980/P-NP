//! Exact 3x3x3 recovery from three orthogonal 2D sum shadows plus eight
//! independent residual coordinates. The projections have rank 19 over Q;
//! the eight 2x2x2 corner coordinates provide the missing complement.
#![forbid(unsafe_code)]

pub const EDGE: usize = 3;
pub const CELLS: usize = 27;
pub const ANCHORS: usize = 8;

fn at(x: usize, y: usize, z: usize) -> usize { x + EDGE * y + EDGE * EDGE * z }
fn anchor_at(x: usize, y: usize, z: usize) -> usize { x + 2 * y + 4 * z }

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ThreeShadows {
    pub xy: [i128; 9],
    pub yz: [i128; 9],
    pub zx: [i128; 9],
    pub residual_corner: [i128; ANCHORS],
}

impl ThreeShadows {
    pub fn from_cells(cells: &[i128; CELLS]) -> Self {
        let mut out = Self {
            xy: [0; 9], yz: [0; 9], zx: [0; 9],
            residual_corner: [0; ANCHORS],
        };
        for z in 0..EDGE {
            for y in 0..EDGE {
                for x in 0..EDGE {
                    let value = cells[at(x, y, z)];
                    out.xy[x + EDGE * y] += value;
                    out.yz[y + EDGE * z] += value;
                    out.zx[z + EDGE * x] += value;
                    if x < 2 && y < 2 && z < 2 {
                        out.residual_corner[anchor_at(x, y, z)] = value;
                    }
                }
            }
        }
        out
    }

    /// Construct the missing boundary planes in three stages. Each subtraction
    /// uses known coordinates. Check all 27 shadow equations on completion.
    pub fn reconstruct(&self) -> Result<[i128; CELLS], String> {
        let mut reconstructed = [None::<i128>; CELLS];
        for z in 0..2 {
            for y in 0..2 {
                for x in 0..2 {
                    reconstructed[at(x, y, z)] =
                        Some(self.residual_corner[anchor_at(x, y, z)]);
                }
            }
        }
        for z in 0..2 {
            for y in 0..2 {
                let a = reconstructed[at(0, y, z)].ok_or("missing x0 anchor")?;
                let b = reconstructed[at(1, y, z)].ok_or("missing x1 anchor")?;
                reconstructed[at(2, y, z)] = Some(self.yz[y + EDGE * z] - a - b);
            }
        }
        for z in 0..2 {
            for x in 0..EDGE {
                let a = reconstructed[at(x, 0, z)].ok_or("missing y0 value")?;
                let b = reconstructed[at(x, 1, z)].ok_or("missing y1 value")?;
                reconstructed[at(x, 2, z)] = Some(self.zx[z + EDGE * x] - a - b);
            }
        }
        for y in 0..EDGE {
            for x in 0..EDGE {
                let a = reconstructed[at(x, y, 0)].ok_or("missing z0 value")?;
                let b = reconstructed[at(x, y, 1)].ok_or("missing z1 value")?;
                reconstructed[at(x, y, 2)] = Some(self.xy[x + EDGE * y] - a - b);
            }
        }
        let cells = std::array::from_fn(|i| reconstructed[i].unwrap_or(0));
        if reconstructed.iter().any(Option::is_none) {
            return Err("three-shadow reconstruction left a coordinate open".into());
        }
        if Self::from_cells(&cells) != *self {
            return Err("three-shadow consistency or residual witness failure".into());
        }
        Ok(cells)
    }

    pub fn certify_u64(cells: &[u64; CELLS]) -> Result<(), String> {
        let original = std::array::from_fn(|i| i128::from(cells[i]));
        let shadows = Self::from_cells(&original);
        if shadows.reconstruct()? != original {
            return Err("three-shadow exact inverse changed source coordinates".into());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signed_exact_reconstruction_and_each_projection_tamper() {
        let source: [i128; CELLS] =
            std::array::from_fn(|i| i128::from(i as i64 - 13) * (i128::from(i as i64) + 3));
        let witness = ThreeShadows::from_cells(&source);
        assert_eq!(witness.reconstruct(), Ok(source));
        let mut changed = witness.clone();
        changed.xy[0] += 1;
        assert!(changed.reconstruct().is_err());
        changed = witness.clone();
        changed.yz[7] -= 1;
        assert!(changed.reconstruct().is_err());
        changed = witness.clone();
        changed.zx[2] += 1;
        assert!(changed.reconstruct().is_err());
        changed = witness.clone();
        changed.residual_corner[3] += 1;
        // The eight independent residuals select a different, still valid
        // cube with the SAME shadows. Source hash/witness binding detects
        // this substitution; linear equations alone cannot.
        assert_ne!(changed.reconstruct(), Ok(source));
    }

    #[test]
    fn eight_corner_coordinates_close_all_27() {
        for seed in [0_u64, 1, 27, 256, 1_000_081] {
            let cube = std::array::from_fn(|index| {
                (seed.wrapping_mul(17).wrapping_add(index as u64 * 29)) % 1_000_081
            });
            assert_eq!(ThreeShadows::certify_u64(&cube), Ok(()));
        }
    }

    #[test]
    fn a_single_shadow_hides_canceling_changes_but_complete_witness_detects_them() {
        let mut source = [0_i128; CELLS];
        source[at(0, 0, 0)] = 19;
        source[at(0, 0, 1)] = -19;
        let original = [0_i128; CELLS];
        let shadows = ThreeShadows::from_cells(&source);
        let zero = ThreeShadows::from_cells(&original);
        assert_eq!(shadows.xy, zero.xy);
        assert_ne!(shadows.yz, zero.yz);
        assert_ne!(shadows.zx, zero.zx);
        assert_ne!(shadows.residual_corner, zero.residual_corner);
        assert_eq!(shadows.reconstruct(), Ok(source));
    }
}
