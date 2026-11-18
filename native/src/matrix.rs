//! Uniform 27-cell integer matrix. Each room shares one addressing table.
//! The central cell is the designated Omega address (1,1,1).
//! Other cell roles are geometric, not claims about trained neural weights.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CellKind {
    Omega,
    Face,
    Edge,
    Corner,
}

pub const WIDTH: usize = 3;
pub const CELLS: usize = WIDTH * WIDTH * WIDTH;
pub const OMEGA: usize = 13;

pub fn address(x: usize, y: usize, z: usize) -> Option<usize> {
    if x < WIDTH && y < WIDTH && z < WIDTH {
        Some(x + WIDTH * y + WIDTH * WIDTH * z)
    } else {
        None
    }
}

pub fn coordinates(index: usize) -> Option<(usize, usize, usize)> {
    if index >= CELLS {
        None
    } else {
        Some((index % WIDTH, (index / WIDTH) % WIDTH, index / (WIDTH * WIDTH)))
    }
}

pub fn kind(index: usize) -> Option<CellKind> {
    let (x, y, z) = coordinates(index)?;
    let displaced = usize::from(x != 1) + usize::from(y != 1) + usize::from(z != 1);
    match displaced {
        0 => Some(CellKind::Omega),
        1 => Some(CellKind::Face),
        2 => Some(CellKind::Edge),
        3 => Some(CellKind::Corner),
        _ => None,
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Matrix {
    rooms: Vec<[i64; CELLS]>,
}

impl Matrix {
    pub fn from_rooms(rooms: Vec<[i64; CELLS]>) -> Result<Self, &'static str> {
        if rooms.is_empty() {
            return Err("room count must be positive");
        }
        Ok(Self { rooms })
    }

    pub fn generated(room_count: usize) -> Result<Self, &'static str> {
        if !matches!(room_count, 3 | 9 | 27 | 81) {
            return Err("room count must be 3, 9, 27 or 81");
        }
        let mut rooms = Vec::with_capacity(room_count);
        for room in 0..room_count {
            let mut cells = [0_i64; CELLS];
            for (index, value) in cells.iter_mut().enumerate() {
                let seed = i64::try_from(room).map_err(|_| "room overflow")?;
                let local = i64::try_from(index).map_err(|_| "cell overflow")?;
                *value = seed * 27 + local - 13;
            }
            rooms.push(cells);
        }
        Self::from_rooms(rooms)
    }

    pub fn room_count(&self) -> usize {
        self.rooms.len()
    }

    pub fn room(&self, index: usize) -> Option<&[i64; CELLS]> {
        self.rooms.get(index)
    }

    pub fn rooms(&self) -> &[[i64; CELLS]] {
        &self.rooms
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn universal_table_is_bijective_and_has_correct_shells() {
        let mut counts = [0_usize; 4];
        for index in 0..CELLS {
            let (x, y, z) = coordinates(index).expect("every cell addressed");
            assert_eq!(address(x, y, z), Some(index));
            match kind(index).expect("role exists") {
                CellKind::Omega => counts[0] += 1,
                CellKind::Face => counts[1] += 1,
                CellKind::Edge => counts[2] += 1,
                CellKind::Corner => counts[3] += 1,
            }
        }
        assert_eq!(counts, [1, 6, 12, 8]);
        assert_eq!(address(1, 1, 1), Some(OMEGA));
        assert_eq!(kind(OMEGA), Some(CellKind::Omega));
        assert_eq!(address(3, 1, 1), None);
        assert_eq!(coordinates(27), None);
    }

    #[test]
    fn exact_n_by_three_cube_allocation() {
        for n in [3, 9, 27, 81] {
            let matrix = Matrix::generated(n).expect("generated");
            assert_eq!(matrix.room_count() * CELLS, 27 * n);
            assert!(matrix.room(n).is_none());
        }
    }
}
