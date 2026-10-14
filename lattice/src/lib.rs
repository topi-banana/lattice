#![no_std]
#![forbid(unsafe_code)]
#![warn(missing_docs)]

//! # lattice
//!
//! `lattice` is a `no_std` library for programmable voxel simulations in the
//! spirit of Minecraft, without a fixed block table.
//!
//! A block is a *Rust type* that implements [`BlockDef`] plus any number of
//! facet traits such as [`Sticky`](trait@Sticky), [`Movable`](trait@Movable)
//! or [`InstantMinable`](trait@InstantMinable). Each
//! facet is a capability, and capabilities exist in two interchangeable
//! forms:
//!
//! * compile-time bounds — `T: Sticky + Movable`,
//! * composable values — `Sticky + Movable` in the palette DSL,
//!
//! because traits live in the type namespace and constants live in the value
//! namespace. Definitions are types; cells only store a palette index and a
//! raw `u16` of per-cell state.
//!
//! ## Storage layout
//!
//! ```text
//! World<H, P>   BTreeMap<ChunkPos, Chunk<H>>   chunks allocated on first write
//!   Chunk<H>    [SubChunk; H] + BTreeMap<Pos, Box<dyn BlockEntity>>
//!     SubChunk  [[[Cell; 16]; 16]; 16] + [u64; 64] schedule bitset
//! ```
//!
//! Horizontally the world is unbounded (negative coordinates included);
//! vertically it is `H` sub-chunks tall.
//!
//! ## Quick start
//!
//! ```
//! use lattice::{
//!     BlockPalette, Breakable, Empty, InstantMinable, Movable, Pos, Sticky, UnMovable, World,
//! };
//!
//! // Definitions live in a palette. `+` composes capabilities.
//! let palette = BlockPalette::from([
//!     Sticky + Movable + InstantMinable,
//!     Breakable + Movable,
//!     UnMovable,
//!     Empty,
//! ]);
//!
//! // H = 2 sub-chunks (32 blocks tall), palette capacity P = 4.
//! let mut world: World<'_, 2, 4> = World::new(&palette);
//!
//! world.set_block(&palette[0], (1, 1, 1));
//! world.set_block(&palette[2], (2, 1, 1));
//! world.schedule((1, 1, 1));
//!
//! assert_eq!(world.tick(), 1);
//!
//! let block = world.get(Pos::new(1, 1, 1)).unwrap();
//! assert_eq!(block, palette[0]); // the same definition is `eq`
//!
//! let boundary_sticky = block.sticky().unwrap();
//! let boundary_movable = block.movable().unwrap();
//! assert_ne!(boundary_sticky, boundary_movable); // same block, different boundary
//! ```
//!
//! ## Defining blocks
//!
//! ```
//! use lattice::{block, palette, BlockDef, CapSet, Empty, Movable, Sticky, TickCtx};
//!
//! block! {
//!     /// A sticky, movable block.
//!     pub struct Slime;
//!     facets: Sticky, Movable;
//! }
//!
//! // Manual definitions work too; keep `CAPS` in sync with the facets.
//! struct Crumbly;
//!
//! impl BlockDef for Crumbly {
//!     const CAPS: CapSet = CapSet::BREAKABLE;
//!     type State = u8;
//!
//!     fn on_scheduled_tick(ctx: &mut TickCtx<'_>) {
//!         let timer = ctx.state::<Self>();
//!         if timer == 0 {
//!             ctx.remove_self();
//!         } else {
//!             ctx.set_state::<Self>(timer - 1);
//!             ctx.schedule_self();
//!         }
//!     }
//! }
//!
//! impl lattice::Breakable for Crumbly {}
//!
//! let palette = palette![Slime::source(), Crumbly::source(), Empty];
//! assert!(palette.find_type::<Slime>().is_some());
//! ```
//!
//! ## Block entities
//!
//! A cell stores at most 16 bits of state. Definitions that need more attach
//! a [`BlockEntity`] to the position: a heap value stored in a per-chunk map
//! that is *separate* from the cells. Block states and block entities can be
//! queried independently, so a position may hold a block with no entity, a
//! block and an entity, or only an entity ("orphan").
//!
//! `entity: E;` in the [`block!`] macro declares that placing the definition
//! creates `E::default()`; [`World::set_block_entity`] attaches an entity
//! explicitly. [`World::tick`] runs the block hook first and the entity hook
//! second for every scheduled position.
//!
//! ```
//! use lattice::{BlockDef, BlockEntity, Empty, EntityTickCtx, Pos, UnMovable, World, block, palette};
//!
//! /// A counter that would not fit into the 16-bit cell state.
//! #[derive(Default)]
//! pub struct Counter {
//!     /// Number of scheduled ticks seen.
//!     pub ticks: u32,
//! }
//!
//! impl BlockEntity for Counter {
//!     fn on_scheduled_tick(&mut self, ctx: &mut EntityTickCtx<'_>) {
//!         self.ticks += 1;
//!         ctx.schedule_self();
//!     }
//! }
//!
//! block! {
//!     /// Has no room for `u32` in its cell state, so it uses an entity.
//!     pub struct Ticker;
//!     facets: UnMovable;
//!     entity: Counter;
//! }
//!
//! let palette = palette![Ticker::source(), Empty];
//! let mut world: World<'_, 1, 2> = World::new(&palette);
//!
//! world.set_block(palette.find_type::<Ticker>().unwrap(), (0, 0, 0));
//! world.schedule((0, 0, 0));
//! world.tick();
//! assert_eq!(world.entity::<Counter>((0, 0, 0)).unwrap().ticks, 1);
//!
//! // Entity-only positions are fine: the stores are independent.
//! world.set_block_entity((5, 0, 0), Box::new(Counter::default()));
//! assert!(world.get(Pos::new(5, 0, 0)).unwrap().empty().is_some());
//! assert!(world.entity::<Counter>((5, 0, 0)).is_some());
//! ```
//!
//! ## Mechanics
//!
//! The [`piston`] module ships a worked mechanism: [`PistonBlock`] observes
//! the [`Power`](const@Power) capability on its neighbours, waits
//! [`PISTON_EXTEND_TICKS`] scheduled ticks and then moves the line of movable
//! blocks in front of it itself. [`PowerBlock`] is the matching power source.
//!
//! ```
//! use lattice::{BlockDef, Facing, Movable, PistonBlock, PistonState, Pos, PowerBlock, StateCodec, World, palette};
//!
//! let palette = palette![PistonBlock::source(), PowerBlock::source(), Movable, lattice::Empty];
//! let mut world: World<'_, 1, 4> = World::new(&palette);
//!
//! let piston = palette.find_type::<PistonBlock>().unwrap();
//! let power = palette.find_type::<PowerBlock>().unwrap();
//! let movable = palette[2];
//!
//! world.set_block_with_state(piston, (0, 0, 0), PistonState::retracted(Facing::PosX).encode());
//! world.set_block(movable, (1, 0, 0));
//!
//! // Placing the power block notifies the piston.
//! world.set_block(power, (-1, 0, 0));
//! for _ in 0..6 {
//!     world.tick();
//! }
//!
//! assert_eq!(world.get(Pos::new(2, 0, 0)).unwrap(), movable);
//! ```
//!
//! # no_std
//!
//! The crate is `#![no_std]` and depends on `core` plus `alloc` (for the
//! chunk map, the per-chunk block entity map and the tick scratch buffer).
//! There is no OS dependency and no `unsafe` code; block and palette
//! definitions themselves need no allocation, and only entities that are
//! actually attached allocate.

extern crate alloc;

pub mod caps;
pub mod chunk;
pub mod def;
pub mod entity;
pub mod palette;
pub mod piston;
pub mod pos;
pub mod rng;
pub mod world;

pub use caps::{
    Breakable, CapSet, Empty, Flammable, InstantMinable, Movable, Power, Sticky, UnMovable,
};
pub use chunk::{Chunk, ChunkPos, SUBCHUNK_SIZE, SUBCHUNK_VOLUME, SubChunk};
pub use def::{BlockDef, Identity, StateCodec, default_break_ticks};
pub use entity::{AsAny, BlockEntity, EntityTickCtx};
pub use palette::{Block, BlockPalette, BlockSource, Facet, PaletteError};
pub use piston::{
    Facing, PISTON_EXTEND_TICKS, PISTON_PUSH_LIMIT, PistonBlock, PistonPhase, PistonState,
    PowerBlock,
};
pub use pos::{Dir, Pos};
pub use rng::Rng32;
pub use world::{BreakOutcome, Positions, TickCtx, World};

/// Declares a block definition type together with its facet implementations.
///
/// The macro computes [`BlockDef::CAPS`] as the union of the facets'
/// [`CAP`](Sticky::CAP) constants, so the compile-time and runtime views of a
/// definition cannot drift apart. Facet names resolve in the calling scope,
/// which means user-defined facets work exactly like the built-in ones.
///
/// Optional `state:` sets [`BlockDef::State`] (default `()`). Optional
/// `entity: E;` declares a block entity type: the world creates
/// `E::default()` via [`BlockDef::create_block_entity`] whenever it places
/// the definition. An optional `behavior { ... }` block may override any
/// [`BlockDef`] method (do not combine it with `entity:` to override
/// `create_block_entity` itself).
///
/// # Examples
///
/// ```
/// use lattice::{block, BlockDef, BlockEntity, Empty, Movable, Sticky, TickCtx};
///
/// block! {
///     /// Sticky and movable.
///     pub struct Slime;
///     facets: Sticky, Movable;
/// }
///
/// #[derive(Default)]
/// pub struct FuseEntity {
///     /// Previous tick, for illustration.
///     pub last: u8,
/// }
///
/// impl BlockEntity for FuseEntity {}
///
/// block! {
///     /// Counts down and removes itself.
///     pub struct Fuse;
///     facets: lattice::Breakable;
///     state: u8;
///     entity: FuseEntity;
///     behavior {
///         fn on_scheduled_tick(ctx: &mut TickCtx<'_>) {
///             let timer = ctx.state::<Self>();
///             if timer == 0 {
///                 ctx.remove_self();
///             } else {
///                 ctx.set_state::<Self>(timer - 1);
///                 ctx.schedule_self();
///             }
///         }
///     }
/// }
///
/// let _palette = lattice::palette![Slime::source(), Fuse::source(), Empty];
/// ```
#[macro_export]
macro_rules! block {
    (
        $(#[$meta:meta])*
        $vis:vis struct $name:ident;
        facets: $($facet:path),+ $(,)? ;
        $(state: $state:ty;)?
        $(entity: $entity:ty;)?
        $(behavior { $($body:tt)* })?
    ) => {
        $(#[$meta])*
        #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
        $vis struct $name;

        impl $crate::BlockDef for $name {
            const CAPS: $crate::CapSet =
                $crate::CapSet::NONE$(.or(<$name as $facet>::CAP))*;
            type State = $crate::block!(@state $($state)?);

            $(
                fn create_block_entity() -> ::core::option::Option<
                    $crate::__private::Box<dyn $crate::BlockEntity>,
                > {
                    ::core::option::Option::Some($crate::__private::Box::new(
                        <$entity as ::core::default::Default>::default(),
                    ))
                }
            )?

            $($($body)*)?
        }

        $(
            impl $facet for $name {}
        )+
    };

    (@state) => { () };
    (@state $state:ty) => { $state };
}

/// Implementation details used by the exported macros. Not a public API.
#[doc(hidden)]
pub mod __private {
    pub use alloc::boxed::Box;
}

/// Builds a [`BlockPalette`] from mixed definition sources.
///
/// `BlockPalette::from` takes an array, so all its elements must share one
/// type. This macro converts each element to a [`BlockSource`] first, which
/// makes it possible to mix capability expressions with typed definitions.
///
/// ```
/// use lattice::{block, palette, BlockDef, Empty, Movable, Sticky};
///
/// block! {
///     /// A sticky, movable block.
///     pub struct Slime;
///     facets: Sticky, Movable;
/// }
///
/// let palette = palette![Slime::source(), Sticky + Movable, Empty];
/// assert_eq!(palette.len(), 3);
/// ```
#[macro_export]
macro_rules! palette {
    ($($source:expr),* $(,)?) => {
        $crate::BlockPalette::from([$($crate::BlockSource::from($source)),*])
    };
}

/// The most commonly used items.
///
/// ```
/// use lattice::prelude::*;
///
/// let palette = BlockPalette::from([Sticky + Movable, Empty]);
/// assert_eq!(palette.len(), 2);
/// ```
pub mod prelude {
    pub use crate::{
        Block, BlockDef, BlockEntity, BlockPalette, BlockSource, BreakOutcome, CapSet, Chunk,
        ChunkPos, Dir, EntityTickCtx, Facet, Facing, Identity, PISTON_EXTEND_TICKS,
        PISTON_PUSH_LIMIT, PaletteError, PistonBlock, PistonPhase, PistonState, Pos, PowerBlock,
        Rng32, SUBCHUNK_SIZE, SUBCHUNK_VOLUME, StateCodec, SubChunk, TickCtx, World, block,
        palette,
    };
    pub use crate::{
        Breakable, Empty, Flammable, InstantMinable, Movable, Power, Sticky, UnMovable,
    };
}
