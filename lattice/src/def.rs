//! Block definitions: [`BlockDef`], identity, state codecs and break timing.

use core::any::TypeId;

use alloc::boxed::Box;

use crate::caps::CapSet;
use crate::entity::BlockEntity;
use crate::palette::BlockSource;
use crate::pos::Dir;
use crate::world::TickCtx;

/// Encodes a cell state value into (at most) 16 raw bits.
///
/// A world cell carries a raw `u16` alongside the block itself. Definitions
/// that need per-cell state (a timer, a rotation, a fill level, ...) pick a
/// concrete, zero-cost representation by implementing this trait.
///
/// Implementations exist for `()`, `bool`, `u8`, `u16` and pairs of codecs.
/// A pair occupies `A::BITS + B::BITS` bits and packs `A` into the low bits.
///
/// # Examples
///
/// ```
/// use lattice::StateCodec;
///
/// let value = (3u8, true);
/// let decoded = <(u8, bool) as StateCodec>::decode(value.encode());
/// assert_eq!(decoded, value);
/// assert_eq!(<(u8, bool) as StateCodec>::BITS, 9);
/// ```
pub trait StateCodec: Copy + Default + 'static {
    /// Number of raw bits used by this codec. Implementations must keep this
    /// at `16` or below.
    const BITS: u8;

    /// Packs the value into raw bits.
    fn encode(self) -> u16;

    /// Unpacks a value from raw bits.
    fn decode(raw: u16) -> Self;
}

impl StateCodec for () {
    const BITS: u8 = 0;

    fn encode(self) -> u16 {
        0
    }

    fn decode(_raw: u16) -> Self {}
}

impl StateCodec for bool {
    const BITS: u8 = 1;

    fn encode(self) -> u16 {
        self as u16
    }

    fn decode(raw: u16) -> Self {
        raw & 1 != 0
    }
}

impl StateCodec for u8 {
    const BITS: u8 = 8;

    fn encode(self) -> u16 {
        self as u16
    }

    fn decode(raw: u16) -> Self {
        raw as u8
    }
}

impl StateCodec for u16 {
    const BITS: u8 = 16;

    fn encode(self) -> u16 {
        self
    }

    fn decode(raw: u16) -> Self {
        raw
    }
}

impl<A: StateCodec, B: StateCodec> StateCodec for (A, B) {
    const BITS: u8 = {
        assert!(A::BITS as u16 + B::BITS as u16 <= 16);
        A::BITS + B::BITS
    };

    fn encode(self) -> u16 {
        self.0.encode() | (self.1.encode() << A::BITS)
    }

    fn decode(raw: u16) -> Self {
        (A::decode(raw), B::decode(raw >> A::BITS))
    }
}

/// The identity of a block definition.
///
/// Two blocks compare equal exactly when their identities are equal, which
/// implements the rule "the same `T` is `eq`, a different `T` is `ne` even
/// when the capability sets overlap".
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Identity {
    /// A definition declared inline with the capability DSL, e.g.
    /// `Sticky + Movable`. Structural definitions are equal when their
    /// capability sets are equal.
    Structural(CapSet),
    /// A definition declared by a Rust type, recorded by its
    /// [`TypeId`]. Nominal definitions are equal only when they are the same
    /// type; sharing a facet (`T: Movable`, `S: Movable`) is not enough.
    Nominal(TypeId, CapSet),
}

impl Identity {
    /// Builds a structural identity.
    pub const fn structural(caps: CapSet) -> Self {
        Self::Structural(caps)
    }

    /// Builds the nominal identity of `T`.
    pub fn nominal<T: BlockDef>() -> Self {
        Self::Nominal(TypeId::of::<T>(), T::CAPS)
    }

    /// The capability set of this definition.
    pub const fn caps(&self) -> CapSet {
        match *self {
            Self::Structural(caps) => caps,
            Self::Nominal(_, caps) => caps,
        }
    }

    /// The [`TypeId`] of a nominal definition.
    pub const fn type_id(&self) -> Option<TypeId> {
        match self {
            Self::Nominal(type_id, _) => Some(*type_id),
            Self::Structural(_) => None,
        }
    }

    /// Returns `true` for [`Identity::Structural`].
    pub const fn is_structural(&self) -> bool {
        matches!(self, Self::Structural(_))
    }
}

/// The default mining duration implied by a capability set.
///
/// [`CapSet::INSTANT_MINABLE`] wins over [`CapSet::BREAKABLE`].
pub const fn default_break_ticks(caps: CapSet) -> u32 {
    if caps.contains(CapSet::INSTANT_MINABLE) {
        0
    } else if caps.contains(CapSet::BREAKABLE) {
        20
    } else {
        u32::MAX
    }
}

/// A block definition: a Rust type describing one entry of a palette.
///
/// Definitions are types, not instances: they are stateless ZSTs, and all
/// hooks are associated functions. Everything that varies per cell lives in
/// the cell state ([`BlockDef::State`]).
///
/// Most definitions are written with the [`block!`](crate::block) macro,
/// which also implements the facet traits and derives `CAPS` from them:
///
/// ```
/// use lattice::{block, Movable, Sticky};
///
/// block! {
///     /// A sticky, movable block.
///     pub struct Slime;
///     facets: Sticky, Movable;
/// }
/// ```
///
/// Manual implementations must declare `CAPS` consistently with the facet
/// traits they implement; the macro exists to keep that invariant easy.
pub trait BlockDef: 'static {
    /// The full capability set of this definition. It must include the
    /// `CAP` constant of every facet trait implemented for `Self`.
    const CAPS: CapSet;

    /// The per-cell state representation.
    type State: StateCodec;

    /// Number of bits used by [`BlockDef::State`].
    const STATE_BITS: u8 = <Self::State as StateCodec>::BITS;

    /// Runs once for every position that was scheduled with
    /// [`World::schedule`](crate::World::schedule).
    fn on_scheduled_tick(_ctx: &mut TickCtx<'_>) {}

    /// Runs when the world performs a random tick on this block.
    fn on_random_tick(_ctx: &mut TickCtx<'_>) {}

    /// Runs after this block was written by
    /// [`World::set_block`](crate::World::set_block).
    ///
    /// Neighbours are notified first, then the newly placed block receives
    /// this hook. Overwriting a cell with the same definition (a state-only
    /// update) does not run it.
    fn on_placed(_ctx: &mut TickCtx<'_>) {}

    /// Runs after an adjacent cell changed.
    ///
    /// `from` points from this block towards the cell that changed.
    fn on_neighbor_changed(_ctx: &mut TickCtx<'_>, _from: Dir) {}

    /// Runs after this block was broken (removed) by
    /// [`World::break_block`](crate::World::break_block).
    fn on_break(_ctx: &mut TickCtx<'_>) {}

    /// Creates the block entity that belongs to this definition.
    ///
    /// The world calls this whenever it places the definition where a
    /// different definition (or air) was before, and stores the result at
    /// the position. Any previous entity is dropped first: replacing a block
    /// replaces its entity, while a state-only update leaves it untouched.
    ///
    /// The default implements the "no block entity" case. [`block!`](crate::block)
    /// generates the method from the optional `entity: E;` clause, using
    /// `E::default()`; definitions that need arguments can write the method
    /// by hand.
    ///
    /// ```
    /// use lattice::{BlockDef, BlockEntity, Empty, UnMovable, World, block, palette};
    ///
    /// #[derive(Default)]
    /// pub struct Counter {
    ///     /// Number of ticks seen so far.
    ///     pub ticks: u32,
    /// }
    ///
    /// impl BlockEntity for Counter {}
    ///
    /// block! {
    ///     /// A block that stores a `u32` in a block entity.
    ///     pub struct Ticker;
    ///     facets: UnMovable;
    ///     entity: Counter;
    /// }
    ///
    /// let palette = palette![Ticker::source(), Empty];
    /// let mut world: World<'_, 1, 2> = World::new(&palette);
    ///
    /// world.set_block(palette.find_type::<Ticker>().unwrap(), (0, 0, 0));
    /// assert_eq!(world.entity::<Counter>((0, 0, 0)).unwrap().ticks, 0);
    /// ```
    fn create_block_entity() -> Option<Box<dyn BlockEntity>> {
        None
    }

    /// How many ticks [`World::break_block`](crate::World::break_block)
    /// reports for a breakable block. Defaults to the capability-based
    /// [`default_break_ticks`].
    fn break_ticks() -> u32 {
        default_break_ticks(Self::CAPS)
    }

    /// Creates the palette source for this definition.
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
    fn source() -> BlockSource
    where
        Self: Sized,
    {
        BlockSource::of::<Self>()
    }
}
