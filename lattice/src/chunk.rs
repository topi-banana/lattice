//! Sub-chunks (16³ cells) and vertical chunks (`H` stacked sub-chunks).
//!
//! Storage layout:
//!
//! ```text
//! World<H, P>   BTreeMap<ChunkPos, Chunk<H>>
//!   Chunk<H>    [SubChunk; H] + BTreeMap<Pos, Box<dyn BlockEntity>>
//!     SubChunk  [[[Cell; 16]; 16]; 16] + [u64; 64] schedule bitset
//!       Cell    block index + u16 state
//! ```
//!
//! Block entities are stored in a chunk-level map keyed by [`Pos`], separate
//! from the cells, so a position can hold an entity independently of the
//! block state (see the [`entity`](crate::entity) module). They follow the
//! chunk: unloading a chunk yields its entities with it.
//!
//! A [`ChunkPos`] addresses a horizontal column; the vertical position is the
//! sub-chunk index inside [`Chunk<H>`]. Block coordinates map with floor
//! division, so negative `x`/`z` values are fine; `y` must be in
//! `0..H * SUBCHUNK_SIZE`.

use core::fmt;

use alloc::boxed::Box;
use alloc::collections::BTreeMap;

use crate::entity::BlockEntity;
use crate::pos::Pos;

/// Edge length of a sub-chunk along every axis.
pub const SUBCHUNK_SIZE: usize = 16;

/// Number of cells in a sub-chunk (`SUBCHUNK_SIZE^3`).
pub const SUBCHUNK_VOLUME: usize = SUBCHUNK_SIZE * SUBCHUNK_SIZE * SUBCHUNK_SIZE;

/// Number of `u64` words needed for one sub-chunk's schedule bitset.
pub(crate) const SCHEDULE_WORDS: usize = SUBCHUNK_VOLUME / 64;

const SIDE: i32 = SUBCHUNK_SIZE as i32;

/// The horizontal coordinates of a chunk column.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ChunkPos {
    /// Chunk coordinate along `x`; a block at `x` lives in chunk
    /// `x.div_euclid(16)`.
    pub x: i32,
    /// Chunk coordinate along `z`; a block at `z` lives in chunk
    /// `z.div_euclid(16)`.
    pub z: i32,
}

impl ChunkPos {
    /// Creates a chunk position.
    pub const fn new(x: i32, z: i32) -> Self {
        Self { x, z }
    }

    /// The chunk containing `pos`.
    ///
    /// Uses floor division, so negative block coordinates map to negative
    /// chunk coordinates.
    pub const fn of(pos: Pos) -> Self {
        Self {
            x: pos.x.div_euclid(SIDE),
            z: pos.z.div_euclid(SIDE),
        }
    }

    /// The lowest block coordinate of this chunk (with `y = 0`).
    pub const fn corner(self) -> Pos {
        Pos::new(self.x * SIDE, 0, self.z * SIDE)
    }
}

impl From<Pos> for ChunkPos {
    fn from(pos: Pos) -> Self {
        Self::of(pos)
    }
}

impl From<ChunkPos> for (i32, i32) {
    fn from(pos: ChunkPos) -> Self {
        (pos.x, pos.z)
    }
}

/// One stored cell: a palette index plus a raw `u16` state.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct Cell {
    pub(crate) block: u16,
    pub(crate) state: u16,
}

/// Linear index of a local position inside a sub-chunk.
///
/// The layout is `x`-major: `index = (x * 16 + y) * 16 + z`.
pub(crate) const fn local_index(x: usize, y: usize, z: usize) -> usize {
    (x * SUBCHUNK_SIZE + y) * SUBCHUNK_SIZE + z
}

/// Inverse of [`local_index`].
pub(crate) const fn local_coords(index: usize) -> (usize, usize, usize) {
    let x = index / (SUBCHUNK_SIZE * SUBCHUNK_SIZE);
    let rest = index % (SUBCHUNK_SIZE * SUBCHUNK_SIZE);
    (x, rest / SUBCHUNK_SIZE, rest % SUBCHUNK_SIZE)
}

/// A 16×16×16 section of cells and its schedule bitset.
pub struct SubChunk {
    cells: [[[Cell; SUBCHUNK_SIZE]; SUBCHUNK_SIZE]; SUBCHUNK_SIZE],
    scheduled: [u64; SCHEDULE_WORDS],
    scheduled_count: u32,
}

impl SubChunk {
    /// Edge length of a sub-chunk.
    pub const SIZE: usize = SUBCHUNK_SIZE;

    /// Number of cells in a sub-chunk.
    pub const VOLUME: usize = SUBCHUNK_VOLUME;

    pub(crate) fn filled(cell: Cell) -> Self {
        Self {
            cells: [[[cell; SUBCHUNK_SIZE]; SUBCHUNK_SIZE]; SUBCHUNK_SIZE],
            scheduled: [0; SCHEDULE_WORDS],
            scheduled_count: 0,
        }
    }

    /// Returns `true` when the local position `(x, y, z)` is scheduled.
    ///
    /// # Panics
    ///
    /// Panics when a coordinate is `>= SUBCHUNK_SIZE`.
    pub fn is_scheduled(&self, x: usize, y: usize, z: usize) -> bool {
        let index = local_index(x, y, z);
        self.scheduled[index / 64] & (1u64 << (index % 64)) != 0
    }

    /// Number of scheduled positions in this sub-chunk.
    pub const fn scheduled_count(&self) -> u32 {
        self.scheduled_count
    }

    pub(crate) fn cell(&self, x: usize, y: usize, z: usize) -> Cell {
        self.cells[x][y][z]
    }

    pub(crate) fn set_cell(&mut self, x: usize, y: usize, z: usize, cell: Cell) {
        self.cells[x][y][z] = cell;
    }

    pub(crate) fn cell_linear(&self, index: usize) -> Cell {
        let (x, y, z) = local_coords(index);
        self.cells[x][y][z]
    }

    /// Schedules a local index; returns `true` when it was not scheduled yet.
    pub(crate) fn schedule(&mut self, index: usize) -> bool {
        let word = index / 64;
        let mask = 1u64 << (index % 64);
        if self.scheduled[word] & mask == 0 {
            self.scheduled[word] |= mask;
            self.scheduled_count += 1;
            true
        } else {
            false
        }
    }

    /// Cancels a local index; returns `true` when it was scheduled.
    pub(crate) fn unschedule(&mut self, index: usize) -> bool {
        let word = index / 64;
        let mask = 1u64 << (index % 64);
        if self.scheduled[word] & mask != 0 {
            self.scheduled[word] &= !mask;
            self.scheduled_count -= 1;
            true
        } else {
            false
        }
    }

    pub(crate) fn schedule_word(&self, word: usize) -> u64 {
        self.scheduled[word]
    }

    pub(crate) fn clear_schedule_bit(&mut self, word: usize, bit: usize) {
        let mask = 1u64 << bit;
        if self.scheduled[word] & mask != 0 {
            self.scheduled[word] &= !mask;
            self.scheduled_count -= 1;
        }
    }
}

impl fmt::Debug for SubChunk {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SubChunk")
            .field("size", &Self::SIZE)
            .field("scheduled", &self.scheduled_count)
            .finish_non_exhaustive()
    }
}

/// A vertical stack of `H` sub-chunks plus their block entities.
///
/// Chunks are the unit of loading: a [`World`](crate::World) allocates one
/// when a block is first written into it. Block entities live in their own
/// map keyed by [`Pos`], isolated from the cells (see the
/// [`entity`](crate::entity) module), and travel with the chunk.
pub struct Chunk<const H: usize> {
    subchunks: [SubChunk; H],
    entities: BTreeMap<Pos, Box<dyn BlockEntity>>,
}

impl<const H: usize> Chunk<H> {
    /// Number of sub-chunks in a chunk.
    pub const SECTIONS: usize = H;

    pub(crate) fn filled(cell: Cell) -> Self {
        Self {
            subchunks: core::array::from_fn(|_| SubChunk::filled(cell)),
            entities: BTreeMap::new(),
        }
    }

    /// Number of sub-chunks in this chunk.
    pub const fn section_count(&self) -> usize {
        H
    }

    /// Height of this chunk in blocks (`H * SUBCHUNK_SIZE`).
    pub const fn height(&self) -> usize {
        H * SUBCHUNK_SIZE
    }

    /// All sub-chunks.
    pub fn sections(&self) -> &[SubChunk; H] {
        &self.subchunks
    }

    /// Read-only access to one sub-chunk.
    pub fn section(&self, index: usize) -> Option<&SubChunk> {
        self.subchunks.get(index)
    }

    /// Total number of scheduled positions across all sub-chunks.
    pub fn scheduled_count(&self) -> usize {
        self.subchunks
            .iter()
            .map(|section| section.scheduled_count as usize)
            .sum()
    }

    /// Number of block entities stored in this chunk.
    pub fn block_entity_count(&self) -> usize {
        self.entities.len()
    }

    /// Iterates over the `(position, entity)` pairs of this chunk in
    /// coordinate order.
    pub fn block_entities(&self) -> impl Iterator<Item = (Pos, &dyn BlockEntity)> + '_ {
        self.entities.iter().map(|(&pos, entity)| (pos, &**entity))
    }

    /// Read-only access to the block entity at `pos`.
    ///
    /// This is independent from the cell at the same position: the entity may
    /// exist while the cell is air, and vice versa.
    pub fn block_entity(&self, pos: Pos) -> Option<&dyn BlockEntity> {
        self.entities.get(&pos).map(|entity| &**entity)
    }

    /// Mutable access to the block entity at `pos`.
    pub fn block_entity_mut(&mut self, pos: Pos) -> Option<&mut dyn BlockEntity> {
        self.entities
            .get_mut(&pos)
            .map(|entity| &mut **entity as &mut dyn BlockEntity)
    }

    /// Inserts an entity, returning the previous one.
    pub(crate) fn insert_block_entity(
        &mut self,
        pos: Pos,
        entity: Box<dyn BlockEntity>,
    ) -> Option<Box<dyn BlockEntity>> {
        self.entities.insert(pos, entity)
    }

    /// Removes and returns the entity at `pos`.
    pub(crate) fn take_block_entity(&mut self, pos: &Pos) -> Option<Box<dyn BlockEntity>> {
        self.entities.remove(pos)
    }

    pub(crate) fn subchunk(&self, index: usize) -> &SubChunk {
        &self.subchunks[index]
    }

    pub(crate) fn subchunk_mut(&mut self, index: usize) -> &mut SubChunk {
        &mut self.subchunks[index]
    }

    /// Reads a cell with `y` in `0..H * SUBCHUNK_SIZE`.
    pub(crate) fn cell(&self, x: usize, y: usize, z: usize) -> Cell {
        self.subchunks[y / SUBCHUNK_SIZE].cell(x, y % SUBCHUNK_SIZE, z)
    }

    /// Writes a cell with `y` in `0..H * SUBCHUNK_SIZE`.
    pub(crate) fn set_cell(&mut self, x: usize, y: usize, z: usize, cell: Cell) {
        self.subchunks[y / SUBCHUNK_SIZE].set_cell(x, y % SUBCHUNK_SIZE, z, cell);
    }

    /// Reads a cell by chunk-wide linear index (`0..H * SUBCHUNK_VOLUME`).
    pub(crate) fn cell_linear(&self, index: usize) -> Cell {
        self.subchunks[index / SUBCHUNK_VOLUME].cell_linear(index % SUBCHUNK_VOLUME)
    }

    /// Inverse of the chunk-wide linear index.
    pub(crate) const fn linear_pos(index: usize) -> (usize, usize, usize) {
        let section = index / SUBCHUNK_VOLUME;
        let (x, local_y, z) = local_coords(index % SUBCHUNK_VOLUME);
        (x, section * SUBCHUNK_SIZE + local_y, z)
    }
}

impl<const H: usize> fmt::Debug for Chunk<H> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Chunk")
            .field("sections", &H)
            .field("scheduled", &self.scheduled_count())
            .field("block_entities", &self.entities.len())
            .finish_non_exhaustive()
    }
}
