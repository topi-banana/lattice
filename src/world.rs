//! Dynamic worlds: [`World<H, P>`](World) maps [`ChunkPos`] to [`Chunk<H>`]
//! through a `BTreeMap`. Chunks are allocated on the first write, so the
//! horizontal extent grows on demand while the vertical extent is fixed to
//! `H` sub-chunks.
//!
//! Ticking stays allocation-conscious: each chunk keeps its schedule bitsets
//! inside its sub-chunks, and `tick` only reuses a small scratch buffer of
//! chunk coordinates.

use core::any::TypeId;
use core::fmt;

use alloc::collections::{BTreeMap, btree_map};
use alloc::vec::Vec;

use crate::caps::CapSet;
use crate::chunk::{
    Cell, Chunk, ChunkPos, SCHEDULE_WORDS, SUBCHUNK_SIZE, SUBCHUNK_VOLUME, local_index,
};
use crate::def::{BlockDef, Identity, StateCodec};
use crate::palette::{Block, BlockPalette, PaletteError};
use crate::pos::{Dir, Pos};
use crate::rng::Rng32;

/// What happened when a block was mined.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BreakOutcome {
    /// The block was [`InstantMinable`](trait@crate::InstantMinable) and is gone.
    Broken,
    /// The block is [`Breakable`](trait@crate::Breakable); this many ticks of
    /// work remain. The world itself does not track mining progress.
    NeedsTicks(u32),
    /// The block has neither the instant nor the timed capability.
    Unbreakable,
    /// The target cell already contains air.
    Empty,
    /// The position is outside the vertical range or in an unloaded chunk.
    OutOfBounds,
}

/// The subset of world operations visible to block hooks.
pub(crate) trait WorldAccess {
    fn get_block(&self, pos: Pos) -> Option<Block>;
    fn set_block(&mut self, block: &Block, pos: Pos) -> Option<Block>;
    fn schedule(&mut self, pos: Pos) -> bool;
    fn state_raw(&self, pos: Pos) -> u16;
    fn set_state_raw(&mut self, pos: Pos, state: u16) -> bool;
    fn break_block(&mut self, pos: Pos) -> BreakOutcome;
    fn remove_block(&mut self, pos: Pos) -> Option<Block>;
    fn move_cell(&mut self, from: Pos, to: Pos) -> bool;
    fn notify(&mut self, pos: Pos);
}

/// The context passed to every [`BlockDef`] hook.
///
/// It exposes the current position and definition, plus a capability-scoped
/// view of the world: read and write blocks, schedule positions and read or
/// write the current cell's state.
pub struct TickCtx<'w> {
    world: &'w mut dyn WorldAccess,
    pos: Pos,
    block: Block,
}

impl<'w> TickCtx<'w> {
    pub(crate) fn new(world: &'w mut dyn WorldAccess, pos: Pos, block: Block) -> Self {
        Self { world, pos, block }
    }

    /// The position this hook runs for.
    pub fn pos(&self) -> Pos {
        self.pos
    }

    /// The block that was current when this context was created.
    ///
    /// A hook may change the cell; the context keeps reporting the original
    /// block, which is what allows [`TickCtx::break_self`] to notify the
    /// broken definition.
    pub fn block(&self) -> Block {
        self.block
    }

    /// The capability set of the current block.
    pub fn caps(&self) -> CapSet {
        self.block.caps()
    }

    /// The identity of the current block.
    pub fn identity(&self) -> Identity {
        self.block.identity()
    }

    /// Returns `true` when the current cell holds the definition `T`.
    pub fn is<T: BlockDef>(&self) -> bool {
        matches!(self.block.type_id(), Some(type_id) if type_id == TypeId::of::<T>())
    }

    /// Reads the block at `pos`.
    ///
    /// Returns `None` when the chunk is not loaded or `pos` is outside the
    /// vertical range.
    pub fn get(&self, pos: impl Into<Pos>) -> Option<Block> {
        self.world.get_block(pos.into())
    }

    /// Returns `true` when `pos` is air, unloaded or outside the world.
    pub fn is_empty(&self, pos: impl Into<Pos>) -> bool {
        self.get(pos)
            .map(|block| block.caps().contains(CapSet::EMPTY))
            .unwrap_or(true)
    }

    /// Writes a block at `pos`, resetting its state and loading the chunk
    /// when necessary.
    ///
    /// Returns the previous block, or `None` when the world rejects the
    /// write (vertical range or unknown block).
    pub fn set<B: Into<Block>>(&mut self, block: B, pos: impl Into<Pos>) -> Option<Block> {
        let block = block.into();
        self.world.set_block(&block, pos.into())
    }

    /// The raw state of the current cell.
    pub fn state_raw(&self) -> u16 {
        self.world.state_raw(self.pos)
    }

    /// Overwrites the raw state of the current cell.
    pub fn set_state_raw(&mut self, state: u16) -> bool {
        self.world.set_state_raw(self.pos, state)
    }

    /// Decodes the current cell's state as `T`'s state.
    ///
    /// Debug builds assert that the current block is `T`.
    pub fn state<T: BlockDef>(&self) -> T::State {
        debug_assert!(self.is::<T>(), "state() called for a different block type");
        <T::State as StateCodec>::decode(self.state_raw())
    }

    /// Encodes `state` into the current cell.
    pub fn set_state<T: BlockDef>(&mut self, state: T::State) {
        debug_assert!(
            self.is::<T>(),
            "set_state() called for a different block type"
        );
        self.set_state_raw(state.encode());
    }

    /// Schedules `pos` for the next [`World::tick`].
    ///
    /// Returns `false` when `pos` is outside the vertical range or its chunk
    /// is not loaded.
    pub fn schedule(&mut self, pos: impl Into<Pos>) -> bool {
        self.world.schedule(pos.into())
    }

    /// Schedules the current position for the next tick.
    pub fn schedule_self(&mut self) -> bool {
        self.world.schedule(self.pos)
    }

    /// Removes the block at `pos`, replacing it with air.
    ///
    /// Unlike [`TickCtx::break_self`] this ignores capabilities and runs no
    /// break hook.
    pub fn remove(&mut self, pos: impl Into<Pos>) -> Option<Block> {
        self.world.remove_block(pos.into())
    }

    /// Mines the current block with [`World::break_block`].
    pub fn break_self(&mut self) -> BreakOutcome {
        self.world.break_block(self.pos)
    }

    /// Removes the current block unconditionally, without the
    /// [`BlockDef::on_break`] hook.
    pub fn remove_self(&mut self) -> Option<Block> {
        self.world.remove_block(self.pos)
    }

    /// Moves one cell to `to`, preserving its raw state and without running
    /// placement or neighbour hooks.
    ///
    /// Crate-internal building block for mechanisms such as
    /// [`PistonBlock`](crate::PistonBlock); callers are responsible for
    /// validating loaded chunks and for calling [`TickCtx::notify`] when the
    /// dust has settled.
    pub(crate) fn move_cell(&mut self, from: Pos, to: Pos) -> bool {
        self.world.move_cell(from, to)
    }

    /// Runs [`BlockDef::on_neighbor_changed`] around `pos`.
    pub(crate) fn notify(&mut self, pos: Pos) {
        self.world.notify(pos);
    }
}

impl fmt::Debug for TickCtx<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TickCtx")
            .field("pos", &self.pos)
            .field("block", &self.block)
            .finish_non_exhaustive()
    }
}

/// A dynamic voxel world over a [`BlockPalette`].
///
/// `H` is the number of 16³ sub-chunks stacked vertically (`H = 4` is 64
/// blocks high) and `P` is the palette capacity. Horizontally the world is
/// unbounded: chunks are created when a block is first written into them.
///
/// ```
/// use lattice::{BlockPalette, Empty, Movable, Pos, Sticky, World};
///
/// let palette = BlockPalette::from([Sticky + Movable, Empty]);
/// let mut world: World<'_, 1, 2> = World::new(&palette);
///
/// world.set_block(&palette[0], (0, 0, 0));
/// world.schedule((0, 0, 0));
/// assert_eq!(world.tick(), 1);
///
/// let block = world.get(Pos::new(0, 0, 0)).unwrap();
/// assert!(block.sticky().is_some());
/// ```
pub struct World<'a, const H: usize, const P: usize> {
    palette: &'a BlockPalette<P>,
    empty: Block,
    chunks: BTreeMap<ChunkPos, Chunk<H>>,
    scratch: Vec<ChunkPos>,
}

impl<'a, const H: usize, const P: usize> World<'a, H, P> {
    /// Creates an empty world. Chunks are allocated on demand.
    ///
    /// # Panics
    ///
    /// Panics when the palette has no [`Empty`](trait@crate::Empty) block;
    /// use [`World::try_new`] to handle that case.
    pub fn new(palette: &'a BlockPalette<P>) -> Self {
        match Self::try_new(palette) {
            Ok(world) => world,
            Err(error) => panic!("cannot create world: {error}"),
        }
    }

    /// Creates an empty world, reporting a palette without an `Empty` block.
    pub fn try_new(palette: &'a BlockPalette<P>) -> Result<Self, PaletteError> {
        let empty = palette.empty().ok_or(PaletteError::NoEmptyBlock)?;
        Ok(Self {
            palette,
            empty,
            chunks: BTreeMap::new(),
            scratch: Vec::new(),
        })
    }

    /// The palette this world was built from.
    pub fn palette(&self) -> &'a BlockPalette<P> {
        self.palette
    }

    /// The block used to represent air.
    pub fn empty_block(&self) -> Block {
        self.empty
    }

    /// Height of the world in blocks (`H * SUBCHUNK_SIZE`).
    pub const fn height(&self) -> usize {
        H * SUBCHUNK_SIZE
    }

    /// Returns `true` when `pos.y` is inside the vertical range.
    ///
    /// Horizontal coordinates are always inside the world; whether a chunk is
    /// loaded is a separate question (see [`World::is_loaded`]).
    pub const fn contains(&self, pos: Pos) -> bool {
        pos.y >= 0 && (pos.y as usize) < H * SUBCHUNK_SIZE
    }

    /// Returns `true` when the chunk at `pos` is currently allocated.
    pub fn is_loaded(&self, pos: ChunkPos) -> bool {
        self.chunks.contains_key(&pos)
    }

    /// Number of loaded chunks.
    pub fn chunk_count(&self) -> usize {
        self.chunks.len()
    }

    /// Iterates over the positions of all loaded chunks, in coordinate order.
    pub fn loaded_chunks(&self) -> impl Iterator<Item = ChunkPos> + '_ {
        self.chunks.keys().copied()
    }

    /// Read-only access to a loaded chunk.
    pub fn chunk(&self, pos: ChunkPos) -> Option<&Chunk<H>> {
        self.chunks.get(&pos)
    }

    /// Allocates the chunk at `pos`, filled with air.
    ///
    /// Returns `true` when the chunk was newly created. This is rarely needed
    /// directly: [`World::set_block`] loads chunks on demand.
    pub fn load_chunk(&mut self, pos: ChunkPos) -> bool {
        if self.chunks.contains_key(&pos) {
            return false;
        }
        let cell = Cell {
            block: self.empty.index,
            state: 0,
        };
        self.chunks.insert(pos, Chunk::filled(cell));
        true
    }

    /// Removes a chunk and returns its data.
    ///
    /// Scheduled positions inside it are dropped.
    pub fn unload_chunk(&mut self, pos: ChunkPos) -> Option<Chunk<H>> {
        self.chunks.remove(&pos)
    }

    /// Reads the block at `pos`.
    ///
    /// Returns `None` when the chunk is not loaded or `pos.y` is outside the
    /// vertical range.
    pub fn get(&self, pos: Pos) -> Option<Block> {
        let (chunk_pos, x, y, z) = self.split(pos)?;
        let chunk = self.chunks.get(&chunk_pos)?;
        Some(self.palette.block_at(chunk.cell(x, y, z).block as usize))
    }

    /// Writes a block at `pos`, loading its chunk when necessary and
    /// resetting the cell state to zero.
    ///
    /// Returns the previous block. `None` means that `pos.y` is outside the
    /// vertical range or that the block does not belong to this world's
    /// palette (the world is then left untouched).
    ///
    /// Setting the block that is already there only updates the state and does
    /// not notify neighbours.
    pub fn set_block<B: Into<Block>>(&mut self, block: B, pos: impl Into<Pos>) -> Option<Block> {
        self.set_block_impl(block.into(), pos.into(), 0)
    }

    /// Like [`World::set_block`], but also stores a raw state value.
    pub fn set_block_with_state<B: Into<Block>>(
        &mut self,
        block: B,
        pos: impl Into<Pos>,
        state: u16,
    ) -> Option<Block> {
        self.set_block_impl(block.into(), pos.into(), state)
    }

    fn set_block_impl(&mut self, block: Block, pos: Pos, state: u16) -> Option<Block> {
        let (chunk_pos, x, y, z) = self.split(pos)?;
        let block_index = self.resolve(&block)?;
        let empty_index = self.empty.index;
        let chunk = self.chunks.entry(chunk_pos).or_insert_with(|| {
            Chunk::filled(Cell {
                block: empty_index,
                state: 0,
            })
        });
        let previous = self.palette.block_at(chunk.cell(x, y, z).block as usize);
        chunk.set_cell(
            x,
            y,
            z,
            Cell {
                block: block_index,
                state,
            },
        );
        if previous.identity() != block.identity() {
            self.notify_neighbors(pos);
            let entry = *self.palette.entry(block_index as usize);
            let mut ctx = TickCtx::new(self, pos, block);
            (entry.on_placed)(&mut ctx);
        }
        Some(previous)
    }

    /// Removes the block at `pos`, replacing it with air.
    ///
    /// Unlike [`World::break_block`] this ignores capabilities and does not
    /// run [`BlockDef::on_break`]. Returns `None` when the chunk is not
    /// loaded or `pos` is outside the vertical range.
    pub fn remove_block(&mut self, pos: Pos) -> Option<Block> {
        let (chunk_pos, x, y, z) = self.split(pos)?;
        let chunk = self.chunks.get_mut(&chunk_pos)?;
        let previous = self.palette.block_at(chunk.cell(x, y, z).block as usize);
        let empty_index = self.empty.index;
        chunk.set_cell(
            x,
            y,
            z,
            Cell {
                block: empty_index,
                state: 0,
            },
        );
        if !previous.caps().contains(CapSet::EMPTY) {
            self.notify_neighbors(pos);
        }
        Some(previous)
    }

    /// Returns the raw state of the cell at `pos`.
    pub fn state_raw(&self, pos: impl Into<Pos>) -> Option<u16> {
        let (chunk_pos, x, y, z) = self.split(pos.into())?;
        let chunk = self.chunks.get(&chunk_pos)?;
        Some(chunk.cell(x, y, z).state)
    }

    /// Overwrites the raw state of the cell at `pos`.
    ///
    /// Returns `false` when the chunk is not loaded or `pos` is outside the
    /// vertical range.
    pub fn set_state_raw(&mut self, pos: impl Into<Pos>, state: u16) -> bool {
        let Some((chunk_pos, x, y, z)) = self.split(pos.into()) else {
            return false;
        };
        let Some(chunk) = self.chunks.get_mut(&chunk_pos) else {
            return false;
        };
        let block = chunk.cell(x, y, z).block;
        chunk.set_cell(x, y, z, Cell { block, state });
        true
    }

    /// Decodes the state of the cell at `pos` as `T`'s state.
    pub fn state<T: BlockDef>(&self, pos: impl Into<Pos>) -> Option<T::State> {
        self.state_raw(pos).map(<T::State as StateCodec>::decode)
    }

    /// Encodes `state` into the cell at `pos`.
    ///
    /// Returns `false` when the chunk is not loaded or `pos` is outside the
    /// vertical range.
    pub fn set_state<T: BlockDef>(&mut self, pos: impl Into<Pos>, state: T::State) -> bool {
        self.set_state_raw(pos, state.encode())
    }

    /// Schedules `pos` for the next [`World::tick`].
    ///
    /// The chunk must already be loaded (writing a block loads it), otherwise
    /// this returns `false`.
    pub fn schedule(&mut self, pos: impl Into<Pos>) -> bool {
        let Some((chunk_pos, x, y, z)) = self.split(pos.into()) else {
            return false;
        };
        let Some(chunk) = self.chunks.get_mut(&chunk_pos) else {
            return false;
        };
        let index = local_index(x, y % SUBCHUNK_SIZE, z);
        chunk.subchunk_mut(y / SUBCHUNK_SIZE).schedule(index);
        true
    }

    /// Cancels a previously scheduled position.
    ///
    /// Returns `true` when the position was scheduled and is now cancelled,
    /// `false` when it was not scheduled or its chunk is not loaded.
    pub fn unschedule(&mut self, pos: impl Into<Pos>) -> bool {
        let Some((chunk_pos, x, y, z)) = self.split(pos.into()) else {
            return false;
        };
        let Some(chunk) = self.chunks.get_mut(&chunk_pos) else {
            return false;
        };
        let index = local_index(x, y % SUBCHUNK_SIZE, z);
        chunk.subchunk_mut(y / SUBCHUNK_SIZE).unschedule(index)
    }

    /// Returns `true` when `pos` is scheduled for the next tick.
    pub fn is_scheduled(&self, pos: impl Into<Pos>) -> bool {
        let Some((chunk_pos, x, y, z)) = self.split(pos.into()) else {
            return false;
        };
        let Some(chunk) = self.chunks.get(&chunk_pos) else {
            return false;
        };
        chunk
            .subchunk(y / SUBCHUNK_SIZE)
            .is_scheduled(x, y % SUBCHUNK_SIZE, z)
    }

    /// Runs every scheduled tick.
    ///
    /// Chunks are visited in coordinate order and positions in sub-chunk
    /// linear order, so the order is deterministic. Positions scheduled by a
    /// hook run are executed on the *next* call to `tick`, never twice in the
    /// same pass. Chunks loaded while ticking are visited on the next pass.
    ///
    /// Returns the number of executed scheduled ticks.
    pub fn tick(&mut self) -> usize {
        let mut processed = 0usize;

        // Snapshot the loaded chunk list so hooks may load or unload chunks.
        let mut scratch = core::mem::take(&mut self.scratch);
        scratch.clear();
        scratch.extend(self.chunks.keys().copied());

        for chunk_pos in scratch.iter().copied() {
            for section in 0..H {
                let scheduled = match self.chunks.get(&chunk_pos) {
                    Some(chunk) => chunk.subchunk(section).scheduled_count(),
                    None => break,
                };
                if scheduled == 0 {
                    continue;
                }
                for word in 0..SCHEDULE_WORDS {
                    let mut mask = match self.chunks.get(&chunk_pos) {
                        Some(chunk) => chunk.subchunk(section).schedule_word(word),
                        None => break,
                    };
                    while mask != 0 {
                        let bit = mask.trailing_zeros() as usize;
                        mask &= mask - 1;

                        let Some(chunk) = self.chunks.get_mut(&chunk_pos) else {
                            break;
                        };
                        chunk.subchunk_mut(section).clear_schedule_bit(word, bit);

                        let index = section * SUBCHUNK_VOLUME + word * 64 + bit;
                        let (x, y, z) = Chunk::<H>::linear_pos(index);
                        let pos = Pos::new(
                            chunk_pos.x * SUBCHUNK_SIZE as i32 + x as i32,
                            y as i32,
                            chunk_pos.z * SUBCHUNK_SIZE as i32 + z as i32,
                        );

                        let Some(block) = self.get(pos) else {
                            continue;
                        };
                        let handler = self.palette.entry(block.index()).on_scheduled_tick;
                        let mut ctx = TickCtx::new(self, pos, block);
                        handler(&mut ctx);
                        processed += 1;
                    }
                }
            }
        }

        self.scratch = scratch;
        processed
    }

    /// Runs `count` random ticks and returns how many cells were visited.
    ///
    /// Each tick picks a loaded chunk (uniformly) and a cell inside it
    /// (uniformly), then calls [`BlockDef::on_random_tick`]. Unloaded chunks
    /// are never picked and an empty world visits nothing.
    pub fn random_tick(&mut self, rng: &mut Rng32, count: usize) -> usize {
        let total = self.chunks.len();
        if total == 0 {
            return 0;
        }
        let height = self.height() as u32;
        let mut processed = 0usize;
        for _ in 0..count {
            let pick = rng.next_below(total as u32) as usize;
            let Some((&chunk_pos, _)) = self.chunks.iter().nth(pick) else {
                continue;
            };
            let x = rng.next_below(SUBCHUNK_SIZE as u32) as i32;
            let z = rng.next_below(SUBCHUNK_SIZE as u32) as i32;
            let y = rng.next_below(height) as i32;
            let pos = Pos::new(
                chunk_pos.x * SUBCHUNK_SIZE as i32 + x,
                y,
                chunk_pos.z * SUBCHUNK_SIZE as i32 + z,
            );
            let Some(block) = self.get(pos) else {
                continue;
            };
            let handler = self.palette.entry(block.index()).on_random_tick;
            let mut ctx = TickCtx::new(self, pos, block);
            handler(&mut ctx);
            processed += 1;
        }
        processed
    }

    /// Mines the block at `pos` according to its capabilities.
    ///
    /// * [`InstantMinable`](trait@crate::InstantMinable) blocks are removed
    ///   and their [`on_break`](BlockDef::on_break) hook runs.
    /// * [`Breakable`](trait@crate::Breakable) blocks report
    ///   [`BreakOutcome::NeedsTicks`] with [`BlockDef::break_ticks`].
    /// * Anything else reports [`BreakOutcome::Unbreakable`].
    ///
    /// Unloaded chunks and positions outside the vertical range report
    /// [`BreakOutcome::OutOfBounds`].
    pub fn break_block(&mut self, pos: Pos) -> BreakOutcome {
        let Some(block) = self.get(pos) else {
            return BreakOutcome::OutOfBounds;
        };
        let caps = block.caps();
        if caps.contains(CapSet::EMPTY) {
            return BreakOutcome::Empty;
        }
        if caps.contains(CapSet::INSTANT_MINABLE) {
            let entry = *self.palette.entry(block.index());
            let empty_index = self.empty.index;
            self.write_cell(pos, empty_index, 0);
            self.notify_neighbors(pos);
            let mut ctx = TickCtx::new(self, pos, block);
            (entry.on_break)(&mut ctx);
            return BreakOutcome::Broken;
        }
        if caps.contains(CapSet::BREAKABLE) {
            let ticks = (self.palette.entry(block.index()).break_ticks)(caps);
            return BreakOutcome::NeedsTicks(ticks);
        }
        BreakOutcome::Unbreakable
    }

    /// Iterates over every position holding the definition `T`.
    pub fn positions_of<T: BlockDef>(&self) -> Positions<'_, 'a, H, P> {
        Positions::new(self, Identity::nominal::<T>())
    }

    /// Iterates over every position matching the structural definition
    /// `caps`.
    pub fn positions(&self, caps: CapSet) -> Positions<'_, 'a, H, P> {
        Positions::new(self, Identity::Structural(caps))
    }

    /// Iterates over every position matching an exact identity.
    pub fn positions_matching(&self, identity: Identity) -> Positions<'_, 'a, H, P> {
        Positions::new(self, identity)
    }

    /// Splits a position into `(chunk, local x, y in chunk, local z)`.
    fn split(&self, pos: Pos) -> Option<(ChunkPos, usize, usize, usize)> {
        if pos.y < 0 || (pos.y as usize) >= H * SUBCHUNK_SIZE {
            return None;
        }
        let chunk = ChunkPos::of(pos);
        let x = pos.x.rem_euclid(SUBCHUNK_SIZE as i32) as usize;
        let z = pos.z.rem_euclid(SUBCHUNK_SIZE as i32) as usize;
        Some((chunk, x, pos.y as usize, z))
    }

    fn resolve(&self, block: &Block) -> Option<u16> {
        let index = block.index();
        if index < self.palette.len() && self.palette.block_at(index).identity == block.identity {
            return Some(index as u16);
        }
        if let Some(found) = self.palette.find(block.identity) {
            return Some(found.index);
        }
        debug_assert!(false, "block does not belong to this world's palette");
        None
    }

    fn cell_at(&self, pos: Pos) -> Option<Cell> {
        let (chunk_pos, x, y, z) = self.split(pos)?;
        let chunk = self.chunks.get(&chunk_pos)?;
        Some(chunk.cell(x, y, z))
    }

    fn write_cell(&mut self, pos: Pos, block: u16, state: u16) {
        if let Some((chunk_pos, x, y, z)) = self.split(pos)
            && let Some(chunk) = self.chunks.get_mut(&chunk_pos)
        {
            chunk.set_cell(x, y, z, Cell { block, state });
        }
    }

    /// Moves one cell to `to`, preserving its raw state; returns `false`
    /// when either cell is missing.
    pub(crate) fn move_cell(&mut self, from: Pos, to: Pos) -> bool {
        let Some(cell) = self.cell_at(from) else {
            return false;
        };
        if self.split(to).is_none() || !self.chunks.contains_key(&ChunkPos::of(to)) {
            return false;
        }
        let empty_index = self.empty.index;
        self.write_cell(to, cell.block, cell.state);
        self.write_cell(from, empty_index, 0);
        true
    }

    fn notify_neighbors(&mut self, pos: Pos) {
        for dir in Dir::ALL {
            let neighbor = pos.offset(dir);
            let Some(block) = self.get(neighbor) else {
                continue;
            };
            let handler = self.palette.entry(block.index()).on_neighbor_changed;
            let mut ctx = TickCtx::new(self, neighbor, block);
            handler(&mut ctx, dir.opposite());
        }
    }
}

impl<'a, const H: usize, const P: usize> WorldAccess for World<'a, H, P> {
    fn get_block(&self, pos: Pos) -> Option<Block> {
        World::get(self, pos)
    }

    fn set_block(&mut self, block: &Block, pos: Pos) -> Option<Block> {
        World::set_block(self, block, pos)
    }

    fn schedule(&mut self, pos: Pos) -> bool {
        World::schedule(self, pos)
    }

    fn state_raw(&self, pos: Pos) -> u16 {
        World::state_raw(self, pos).unwrap_or(0)
    }

    fn set_state_raw(&mut self, pos: Pos, state: u16) -> bool {
        World::set_state_raw(self, pos, state)
    }

    fn break_block(&mut self, pos: Pos) -> BreakOutcome {
        World::break_block(self, pos)
    }

    fn remove_block(&mut self, pos: Pos) -> Option<Block> {
        World::remove_block(self, pos)
    }

    fn move_cell(&mut self, from: Pos, to: Pos) -> bool {
        World::move_cell(self, from, to)
    }

    fn notify(&mut self, pos: Pos) {
        World::notify_neighbors(self, pos)
    }
}

impl<const H: usize, const P: usize> fmt::Debug for World<'_, H, P> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("World")
            .field("sections", &H)
            .field("chunks", &self.chunks.len())
            .field("palette", &self.palette)
            .finish_non_exhaustive()
    }
}

/// Iterator over positions matching one block identity.
///
/// Created by [`World::positions_of`], [`World::positions`] and
/// [`World::positions_matching`]. Chunks are visited in coordinate order and
/// cells in linear order.
pub struct Positions<'w, 'a, const H: usize, const P: usize> {
    world: &'w World<'a, H, P>,
    chunks: btree_map::Iter<'w, ChunkPos, Chunk<H>>,
    current: Option<(ChunkPos, &'w Chunk<H>)>,
    matching: Identity,
    cursor: usize,
}

impl<'w, 'a, const H: usize, const P: usize> Positions<'w, 'a, H, P> {
    fn new(world: &'w World<'a, H, P>, matching: Identity) -> Self {
        Self {
            world,
            chunks: world.chunks.iter(),
            current: None,
            matching,
            cursor: 0,
        }
    }
}

impl<const H: usize, const P: usize> Iterator for Positions<'_, '_, H, P> {
    type Item = Pos;

    fn next(&mut self) -> Option<Pos> {
        let total = H * SUBCHUNK_VOLUME;
        loop {
            if let Some((chunk_pos, chunk)) = self.current {
                if self.cursor < total {
                    let index = self.cursor;
                    self.cursor += 1;
                    let cell = chunk.cell_linear(index);
                    let block = self.world.palette.block_at(cell.block as usize);
                    if block.identity() == self.matching {
                        let (x, y, z) = Chunk::<H>::linear_pos(index);
                        return Some(Pos::new(
                            chunk_pos.x * SUBCHUNK_SIZE as i32 + x as i32,
                            y as i32,
                            chunk_pos.z * SUBCHUNK_SIZE as i32 + z as i32,
                        ));
                    }
                    continue;
                }
            }
            match self.chunks.next() {
                Some((chunk_pos, chunk)) => {
                    self.current = Some((*chunk_pos, chunk));
                    self.cursor = 0;
                }
                None => {
                    self.current = None;
                    return None;
                }
            }
        }
    }
}
