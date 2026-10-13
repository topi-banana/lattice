//! Positions and directions in a voxel grid.

/// An integer position in the world grid.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Pos {
    /// The `x` coordinate (east/west).
    pub x: i32,
    /// The `y` coordinate (up/down).
    pub y: i32,
    /// The `z` coordinate (north/south).
    pub z: i32,
}

impl Pos {
    /// The origin.
    pub const ZERO: Self = Self { x: 0, y: 0, z: 0 };

    /// Creates a position from its coordinates.
    pub const fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    /// Returns this position shifted by one step in `dir`.
    pub const fn offset(self, dir: Dir) -> Self {
        let (dx, dy, dz) = dir.delta();
        Self::new(self.x + dx, self.y + dy, self.z + dz)
    }

    /// Returns the coordinates as a tuple.
    pub const fn to_tuple(self) -> (i32, i32, i32) {
        (self.x, self.y, self.z)
    }
}

impl From<(i32, i32, i32)> for Pos {
    fn from(value: (i32, i32, i32)) -> Self {
        Self::new(value.0, value.1, value.2)
    }
}

impl From<[i32; 3]> for Pos {
    fn from(value: [i32; 3]) -> Self {
        Self::new(value[0], value[1], value[2])
    }
}

impl From<Pos> for (i32, i32, i32) {
    fn from(value: Pos) -> Self {
        value.to_tuple()
    }
}

/// One of the six axis-aligned directions.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Dir {
    /// Positive `x`.
    PosX,
    /// Negative `x`.
    NegX,
    /// Positive `y`.
    PosY,
    /// Negative `y`.
    NegY,
    /// Positive `z`.
    PosZ,
    /// Negative `z`.
    NegZ,
}

impl Dir {
    /// All six directions.
    pub const ALL: [Self; 6] = [
        Self::PosX,
        Self::NegX,
        Self::PosY,
        Self::NegY,
        Self::PosZ,
        Self::NegZ,
    ];

    /// The `(dx, dy, dz)` step of this direction.
    pub const fn delta(self) -> (i32, i32, i32) {
        match self {
            Self::PosX => (1, 0, 0),
            Self::NegX => (-1, 0, 0),
            Self::PosY => (0, 1, 0),
            Self::NegY => (0, -1, 0),
            Self::PosZ => (0, 0, 1),
            Self::NegZ => (0, 0, -1),
        }
    }

    /// The direction pointing the other way.
    pub const fn opposite(self) -> Self {
        match self {
            Self::PosX => Self::NegX,
            Self::NegX => Self::PosX,
            Self::PosY => Self::NegY,
            Self::NegY => Self::PosY,
            Self::PosZ => Self::NegZ,
            Self::NegZ => Self::PosZ,
        }
    }
}
