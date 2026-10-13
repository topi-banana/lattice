//! Palettes: the registry that turns [`BlockDef`] types and inline capability
//! sets into [`Block`] handles.

use core::any::TypeId;
use core::fmt;
use core::hash::{Hash, Hasher};
use core::ops::Index;

use crate::caps::CapSet;
use crate::def::{BlockDef, Identity, default_break_ticks};
use crate::pos::Dir;
use crate::world::TickCtx;

/// Behavior hook taking only a context.
pub(crate) type TickFn = fn(&mut TickCtx<'_>);
/// Behavior hook reacting to a neighbour.
pub(crate) type NeighborFn = fn(&mut TickCtx<'_>, Dir);
/// Mining duration query.
pub(crate) type BreakTicksFn = fn(CapSet) -> u32;

fn noop_tick(_ctx: &mut TickCtx<'_>) {}
fn noop_neighbor(_ctx: &mut TickCtx<'_>, _from: Dir) {}

fn nominal_break_ticks<T: BlockDef>(_caps: CapSet) -> u32 {
    T::break_ticks()
}

/// Erased behavior of one palette entry.
#[derive(Clone, Copy)]
pub(crate) struct Entry {
    pub(crate) identity: Identity,
    pub(crate) on_placed: TickFn,
    pub(crate) on_scheduled_tick: TickFn,
    pub(crate) on_random_tick: TickFn,
    pub(crate) on_neighbor_changed: NeighborFn,
    pub(crate) on_break: TickFn,
    pub(crate) break_ticks: BreakTicksFn,
}

impl Entry {
    const fn placeholder() -> Self {
        Self {
            identity: Identity::Structural(CapSet::NONE),
            on_placed: noop_tick,
            on_scheduled_tick: noop_tick,
            on_random_tick: noop_tick,
            on_neighbor_changed: noop_neighbor,
            on_break: noop_tick,
            break_ticks: default_break_ticks,
        }
    }
}

impl fmt::Debug for Entry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Entry")
            .field("identity", &self.identity)
            .finish_non_exhaustive()
    }
}

/// A definition ready to be put into a [`BlockPalette`].
///
/// Built from a capability set (`.into()`) or from a Rust type through
/// [`BlockSource::of`] / [`BlockDef::source`].
#[derive(Clone, Copy, Debug)]
pub struct BlockSource(pub(crate) Entry);

impl BlockSource {
    /// Erases the definition `T` into a palette source.
    pub fn of<T: BlockDef>() -> Self {
        Self(Entry {
            identity: Identity::Nominal(TypeId::of::<T>(), T::CAPS),
            on_placed: <T as BlockDef>::on_placed,
            on_scheduled_tick: <T as BlockDef>::on_scheduled_tick,
            on_random_tick: <T as BlockDef>::on_random_tick,
            on_neighbor_changed: <T as BlockDef>::on_neighbor_changed,
            on_break: <T as BlockDef>::on_break,
            break_ticks: nominal_break_ticks::<T>,
        })
    }

    /// The identity of this source.
    pub const fn identity(&self) -> Identity {
        self.0.identity
    }

    /// The capability set of this source.
    pub const fn caps(&self) -> CapSet {
        self.0.identity.caps()
    }
}

impl From<CapSet> for BlockSource {
    fn from(caps: CapSet) -> Self {
        Self(Entry {
            identity: Identity::Structural(caps),
            on_placed: noop_tick,
            on_scheduled_tick: noop_tick,
            on_random_tick: noop_tick,
            on_neighbor_changed: noop_neighbor,
            on_break: noop_tick,
            break_ticks: default_break_ticks,
        })
    }
}

/// A handle to one palette entry.
///
/// Blocks are tiny `Copy` values. Equality follows the definition:
/// `palette[0] == palette[0]`, and blocks from different palettes compare
/// equal when they carry the same [`Identity`]. To compare a specific
/// *boundary* (a facet), obtain a [`Facet`].
#[derive(Clone, Copy)]
pub struct Block {
    pub(crate) index: u16,
    pub(crate) identity: Identity,
}

impl Block {
    const PLACEHOLDER: Self = Self {
        index: 0,
        identity: Identity::Structural(CapSet::NONE),
    };

    /// The palette-relative index of this block.
    pub const fn index(&self) -> usize {
        self.index as usize
    }

    /// The full definition identity.
    pub const fn identity(&self) -> Identity {
        self.identity
    }

    /// The capability set of this block's definition.
    pub const fn caps(&self) -> CapSet {
        self.identity.caps()
    }

    /// The [`TypeId`] of this block, when its definition is a Rust type.
    pub const fn type_id(&self) -> Option<TypeId> {
        self.identity.type_id()
    }

    /// Views this block through a single capability boundary.
    ///
    /// Returns `None` when the block does not have `cap`.
    pub fn facet(&self, cap: CapSet) -> Option<Facet> {
        if self.caps().contains(cap) {
            Some(Facet { block: *self, cap })
        } else {
            None
        }
    }

    /// Views this block as a [`Sticky`](trait@crate::Sticky) block.
    pub fn sticky(&self) -> Option<Facet> {
        self.facet(CapSet::STICKY)
    }

    /// Views this block as a [`Movable`](trait@crate::Movable) block.
    pub fn movable(&self) -> Option<Facet> {
        self.facet(CapSet::MOVABLE)
    }

    /// Views this block as an [`InstantMinable`](trait@crate::InstantMinable) block.
    pub fn instant_minable(&self) -> Option<Facet> {
        self.facet(CapSet::INSTANT_MINABLE)
    }

    /// Views this block as a [`Breakable`](trait@crate::Breakable) block.
    pub fn breakable(&self) -> Option<Facet> {
        self.facet(CapSet::BREAKABLE)
    }

    /// Views this block as an [`UnMovable`](trait@crate::UnMovable) block.
    pub fn unmovable(&self) -> Option<Facet> {
        self.facet(CapSet::UNMOVABLE)
    }

    /// Views this block as an [`Empty`](trait@crate::Empty) block.
    pub fn empty(&self) -> Option<Facet> {
        self.facet(CapSet::EMPTY)
    }

    /// Views this block as a [`Flammable`](trait@crate::Flammable) block.
    pub fn flammable(&self) -> Option<Facet> {
        self.facet(CapSet::FLAMMABLE)
    }
}

impl From<&Block> for Block {
    fn from(block: &Block) -> Self {
        *block
    }
}

impl PartialEq for Block {
    fn eq(&self, other: &Self) -> bool {
        self.identity == other.identity
    }
}

impl Eq for Block {}

impl Hash for Block {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.identity.hash(state);
    }
}

impl fmt::Debug for Block {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Block")
            .field("index", &self.index)
            .field("identity", &self.identity)
            .finish()
    }
}

/// A block viewed through a single capability boundary.
///
/// Facets implement equality over `(definition, boundary)`:
///
/// * the same block under the same boundary is `eq`,
/// * the same block under different boundaries (`movable()` vs `sticky()`) is
///   `ne`,
/// * different definitions under the same boundary (`T: Movable` vs
///   `S: Movable`) are `ne`.
#[derive(Clone, Copy, Debug)]
pub struct Facet {
    block: Block,
    cap: CapSet,
}

impl Facet {
    /// The underlying block.
    pub const fn block(&self) -> Block {
        self.block
    }

    /// The capability this facet represents.
    pub const fn capability(&self) -> CapSet {
        self.cap
    }
}

impl PartialEq for Facet {
    fn eq(&self, other: &Self) -> bool {
        self.block == other.block && self.cap == other.cap
    }
}

impl Eq for Facet {}

impl Hash for Facet {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.block.hash(state);
        self.cap.hash(state);
    }
}

/// Error returned by [`World::try_new`](crate::World::try_new).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PaletteError {
    /// The palette does not contain a block with the
    /// [`Empty`](trait@crate::Empty) capability, so the world cannot be filled.
    NoEmptyBlock,
}

impl fmt::Display for PaletteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoEmptyBlock => f.write_str("the block palette does not contain an Empty block"),
        }
    }
}

/// A fixed-size registry of block definitions.
///
/// Construction deduplicates entries: two structural definitions with the
/// same capability set collapse into one entry, and registering the same type
/// twice does the same. This is what makes "the same `T` is `eq`" hold for
/// handles obtained from different palettes.
///
/// # Examples
///
/// ```
/// use lattice::{BlockPalette, Empty, Movable, Sticky};
///
/// let palette = BlockPalette::from([
///     Sticky + Movable,
///     Movable + Sticky, // duplicate, collapsed
///     Empty,
/// ]);
///
/// assert_eq!(palette.len(), 2);
/// assert_eq!(palette[0].caps(), Sticky + Movable);
/// ```
pub struct BlockPalette<const P: usize> {
    blocks: [Block; P],
    entries: [Entry; P],
    len: usize,
}

impl<const P: usize> BlockPalette<P> {
    /// The number of distinct definitions in this palette.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns `true` when the palette has no entries.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// All definitions, in palette order.
    pub fn blocks(&self) -> &[Block] {
        &self.blocks[..self.len]
    }

    /// All definitions, in palette order.
    pub fn iter(&self) -> core::slice::Iter<'_, Block> {
        self.blocks().iter()
    }

    /// Returns the entry at `index`, if any.
    pub fn get(&self, index: usize) -> Option<Block> {
        self.blocks.get(index).copied()
    }

    /// Finds a definition by full identity.
    pub fn find(&self, identity: Identity) -> Option<Block> {
        self.blocks()
            .iter()
            .find(|block| block.identity == identity)
            .copied()
    }

    /// Finds the definition declared by the type `T`.
    ///
    /// ```
    /// use lattice::{block, palette, BlockDef, Empty, Movable, Sticky};
    ///
    /// block! {
    ///     pub struct Slime;
    ///     facets: Sticky, Movable;
    /// }
    ///
    /// let palette = palette![Slime::source(), Empty];
    /// assert!(palette.find_type::<Slime>().is_some());
    /// ```
    pub fn find_type<T: BlockDef>(&self) -> Option<Block> {
        self.find(Identity::nominal::<T>())
    }

    /// The first block carrying the [`Empty`](trait@crate::Empty) capability.
    pub fn empty(&self) -> Option<Block> {
        self.blocks()
            .iter()
            .find(|block| block.caps().contains(CapSet::EMPTY))
            .copied()
    }

    pub(crate) fn block_at(&self, index: usize) -> Block {
        self.blocks[index]
    }

    pub(crate) fn entry(&self, index: usize) -> &Entry {
        &self.entries[index]
    }
}

impl<S: Into<BlockSource>, const P: usize> From<[S; P]> for BlockPalette<P> {
    fn from(sources: [S; P]) -> Self {
        let mut blocks = [Block::PLACEHOLDER; P];
        let mut entries = [Entry::placeholder(); P];
        let mut len = 0usize;

        for source in sources {
            let source = source.into();
            let identity = source.0.identity;
            if blocks[..len].iter().any(|block| block.identity == identity) {
                continue;
            }
            debug_assert!(len <= u16::MAX as usize);
            blocks[len] = Block {
                index: len as u16,
                identity,
            };
            entries[len] = source.0;
            len += 1;
        }

        Self {
            blocks,
            entries,
            len,
        }
    }
}

impl<const P: usize> Index<usize> for BlockPalette<P> {
    type Output = Block;

    /// Returns the `index`-th definition.
    ///
    /// # Panics
    ///
    /// Panics when `index` is out of range.
    fn index(&self, index: usize) -> &Self::Output {
        assert!(index < self.len, "block palette index out of range");
        &self.blocks[index]
    }
}

impl<const P: usize> fmt::Debug for BlockPalette<P> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BlockPalette")
            .field("len", &self.len)
            .field("capacity", &P)
            .field("blocks", &self.blocks())
            .finish()
    }
}
