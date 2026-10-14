//! Block entities: per-position data kept *beside*, but separate from, block
//! states.
//!
//! A cell only stores a palette index and a raw `u16`
//! ([`BlockDef::State`]). Definitions that need more per-position state — an
//! inventory, a 32-bit counter, a waypoint — attach a [`BlockEntity`] to the
//! position instead. Entities are heap values held in a per-chunk map keyed
//! by [`Pos`], so the two stores can be queried independently:
//!
//! ```text
//! Chunk<H>
//!   subchunks: [[[Cell; 16]; 16]; 16]   block = palette index + u16 state
//!   entities:  BTreeMap<Pos, Box<dyn BlockEntity>>   isolated storage
//! ```
//!
//! Because the stores are isolated, every combination is representable:
//!
//! * a cell can hold a block with no entity,
//! * a cell can hold a block and an entity,
//! * a position can hold an entity while the block is air (an *orphan*),
//! * a position can hold neither.
//!
//! The automatic lifecycle keeps the common cases in sync:
//!
//! | operation | effect on the block entity |
//! | --- | --- |
//! | [`World::set_block`](crate::World::set_block) with a different definition | the old entity is dropped, then the new definition's [`create_block_entity`](crate::BlockDef::create_block_entity) factory (if any) runs |
//! | `set_block` / `set_block_with_state` with the same definition | untouched (state-only update) |
//! | [`World::remove_block`](crate::World::remove_block) | removed with the block |
//! | [`World::break_block`](crate::World::break_block) | removed after [`on_break`](crate::BlockDef::on_break) ran, so the hook still sees it |
//! | piston moves (internal `move_cell`) | travels with its cell; the stroke fails when the destination already holds one |
//!
//! Everything else goes through the explicit API
//! ([`World::set_block_entity`](crate::World::set_block_entity),
//! [`World::remove_block_entity`](crate::World::remove_block_entity)), which
//! never touches the block and can create, keep or repair any mismatch.
//!
//! # Execution
//!
//! Entities run from the position schedule, exactly like blocks: [`World::tick`](crate::World::tick)
//! runs the block hook first and the entity hook second for every scheduled
//! position, and [`World::random_tick`](crate::World::random_tick) does the
//! same for random visits. A position with only an orphan entity still ticks
//! its entity.
//!
//! Entity hooks receive an [`EntityTickCtx`], which borrows the world while
//! the entity itself is owned by the hook. The entity is detached from the
//! world for the duration of the hook and re-attached when it returns; see
//! [`EntityTickCtx::remove_self`] for the exact rules.

use core::any::{Any, TypeId};
use core::fmt;

use alloc::boxed::Box;

use crate::caps::CapSet;
use crate::def::{BlockDef, StateCodec};
use crate::palette::Block;
use crate::pos::Pos;
use crate::world::{BreakOutcome, WorldAccess};

/// Support trait that makes [`BlockEntity`] trait objects downcastable.
///
/// It is blanket-implemented for every `'static` type, so entity types never
/// implement it by hand. It exists because trait upcasting
/// (`&dyn BlockEntity as &dyn Any`) is not available on every supported Rust
/// version; instead, `dyn BlockEntity` exposes the inherent methods
/// `is`, `downcast_ref` and `downcast_mut` built on this trait.
pub trait AsAny: 'static {
    /// Upcasts this value to `dyn Any`.
    fn as_any(&self) -> &dyn Any;

    /// Upcasts this value to `dyn Any`, mutably.
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl<T: Any> AsAny for T {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

/// Per-position data attached to a cell, stored beside the block state.
///
/// Implementing the trait needs no methods; state lives in the type itself
/// and the hook receives `&mut self`:
///
/// ```
/// use lattice::{BlockEntity, EntityTickCtx};
///
/// #[derive(Default)]
/// pub struct Counter {
///     /// Number of ticks seen so far.
///     pub ticks: u32,
/// }
///
/// impl BlockEntity for Counter {
///     fn on_scheduled_tick(&mut self, ctx: &mut EntityTickCtx<'_>) {
///         self.ticks += 1;
///         // Keep ticking while the cell stays scheduled.
///         ctx.schedule_self();
///     }
/// }
/// ```
///
/// The entity is attached to a position either by declaring `entity: E;` in
/// the [`block!`](crate::block) macro (the world creates `E::default()` when
/// the block is placed) or explicitly with
/// [`World::set_block_entity`](crate::World::set_block_entity).
pub trait BlockEntity: AsAny {
    /// Runs once for every scheduled position the entity occupies, after the
    /// block hook for that position.
    fn on_scheduled_tick(&mut self, _ctx: &mut EntityTickCtx<'_>) {}

    /// Runs when [`World::random_tick`](crate::World::random_tick) visits the
    /// position, after the block's random tick hook.
    fn on_random_tick(&mut self, _ctx: &mut EntityTickCtx<'_>) {}
}

impl dyn BlockEntity {
    /// Returns `true` when this entity is of type `T`.
    pub fn is<T: BlockEntity>(&self) -> bool {
        self.as_any().type_id() == TypeId::of::<T>()
    }

    /// Downcasts this entity to `T`.
    pub fn downcast_ref<T: BlockEntity>(&self) -> Option<&T> {
        self.as_any().downcast_ref::<T>()
    }

    /// Downcasts this entity to `T`, mutably.
    pub fn downcast_mut<T: BlockEntity>(&mut self) -> Option<&mut T> {
        self.as_any_mut().downcast_mut::<T>()
    }
}

/// The context passed to [`BlockEntity`] hooks.
///
/// It exposes the position, the block at it, and the same capability-scoped
/// world operations as [`TickCtx`](crate::TickCtx): read and write blocks,
/// schedule positions, read or write the cell state, and access block
/// entities.
///
/// # Detachment
///
/// The hook owns `&mut self`, so the world detaches the entity while the hook
/// runs: reading the entity back through this context returns `None`, even at
/// [`EntityTickCtx::pos`]. When the hook returns the entity is re-attached,
/// unless:
///
/// * [`EntityTickCtx::remove_self`] was called,
/// * the hook installed a different entity at the position, or
/// * the definition of the block at the position changed (placing a block
///   replaces its entity; removing it drops the entity with it).
pub struct EntityTickCtx<'w> {
    world: &'w mut dyn WorldAccess,
    pos: Pos,
    removed: bool,
}

impl<'w> EntityTickCtx<'w> {
    pub(crate) fn new(world: &'w mut dyn WorldAccess, pos: Pos) -> Self {
        Self {
            world,
            pos,
            removed: false,
        }
    }

    /// The position this hook runs for.
    pub fn pos(&self) -> Pos {
        self.pos
    }

    /// The block currently at [`pos`](EntityTickCtx::pos).
    ///
    /// Returns `None` when the chunk is not loaded. A block entity can exist
    /// where the block is air, so an [`Empty`](trait@crate::Empty) block is
    /// not a contradiction.
    pub fn block(&self) -> Option<Block> {
        self.world.get_block(self.pos)
    }

    /// The capability set of the block at [`pos`](EntityTickCtx::pos), or the
    /// empty set when the chunk is not loaded.
    pub fn caps(&self) -> CapSet {
        self.block()
            .map(|block| block.caps())
            .unwrap_or(CapSet::NONE)
    }

    /// Reads the block at `pos`.
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
    pub fn set(&mut self, block: impl Into<Block>, pos: impl Into<Pos>) -> Option<Block> {
        let block = block.into();
        self.world.set_block(&block, pos.into())
    }

    /// Removes the block at `pos`, replacing it with air.
    pub fn remove(&mut self, pos: impl Into<Pos>) -> Option<Block> {
        self.world.remove_block(pos.into())
    }

    /// Mines the block at `pos` according to its capabilities, exactly like
    /// [`World::break_block`](crate::World::break_block).
    pub fn break_block(&mut self, pos: impl Into<Pos>) -> BreakOutcome {
        self.world.break_block(pos.into())
    }

    /// Schedules `pos` for the next [`World::tick`](crate::World::tick).
    pub fn schedule(&mut self, pos: impl Into<Pos>) -> bool {
        self.world.schedule(pos.into())
    }

    /// Schedules [`pos`](EntityTickCtx::pos) for the next tick.
    pub fn schedule_self(&mut self) -> bool {
        self.world.schedule(self.pos)
    }

    /// The raw state of the cell at [`pos`](EntityTickCtx::pos).
    pub fn state_raw(&self) -> u16 {
        self.world.state_raw(self.pos)
    }

    /// Overwrites the raw state of the cell at [`pos`](EntityTickCtx::pos).
    pub fn set_state_raw(&mut self, state: u16) -> bool {
        self.world.set_state_raw(self.pos, state)
    }

    /// Decodes the state of the cell at [`pos`](EntityTickCtx::pos) as `T`'s
    /// state.
    pub fn state<T: BlockDef>(&self) -> T::State {
        <T::State as StateCodec>::decode(self.state_raw())
    }

    /// Encodes `state` into the cell at [`pos`](EntityTickCtx::pos).
    pub fn set_state<T: BlockDef>(&mut self, state: T::State) {
        self.set_state_raw(state.encode());
    }

    /// Reads the block entity at `pos`.
    ///
    /// The entity this hook runs for is detached; see the [type
    /// documentation](EntityTickCtx#detachment).
    pub fn block_entity(&self, pos: impl Into<Pos>) -> Option<&dyn BlockEntity> {
        self.world.block_entity(pos.into())
    }

    /// Mutable access to the block entity at `pos`.
    pub fn block_entity_mut(&mut self, pos: impl Into<Pos>) -> Option<&mut dyn BlockEntity> {
        self.world.block_entity_mut(pos.into())
    }

    /// Reads the block entity at `pos`, downcast to `T`.
    pub fn entity<T: BlockEntity>(&self, pos: impl Into<Pos>) -> Option<&T> {
        self.block_entity(pos)?.downcast_ref::<T>()
    }

    /// Mutable downcast of the block entity at `pos`.
    pub fn entity_mut<T: BlockEntity>(&mut self, pos: impl Into<Pos>) -> Option<&mut T> {
        self.block_entity_mut(pos)?.downcast_mut::<T>()
    }

    /// Stores (`Some`) or removes (`None`) the block entity at `pos`.
    ///
    /// Storing loads the chunk on demand. Returns `false` when `pos` is
    /// outside the vertical range, or when `None` was requested for an
    /// unloaded chunk.
    pub fn set_block_entity(
        &mut self,
        pos: impl Into<Pos>,
        entity: Option<Box<dyn BlockEntity>>,
    ) -> bool {
        self.world.set_block_entity(pos.into(), entity)
    }

    /// Removes and returns the block entity at `pos`.
    ///
    /// This also works for [`pos`](EntityTickCtx::pos) itself: the detached
    /// entity is not stored in the world, so `None` is returned there. Call
    /// [`EntityTickCtx::remove_self`] to drop the running entity.
    pub fn remove_block_entity(&mut self, pos: impl Into<Pos>) -> Option<Box<dyn BlockEntity>> {
        self.world.remove_block_entity(pos.into())
    }

    /// Requests removal of this block entity.
    ///
    /// The entity is dropped when the hook returns; the block at the position
    /// is not touched. Placing or removing the block from the same hook
    /// drops the entity anyway, because replacing the block replaces its
    /// entity.
    ///
    /// ```
    /// use lattice::{BlockEntity, EntityTickCtx};
    ///
    /// struct Disposable;
    ///
    /// impl BlockEntity for Disposable {
    ///     fn on_scheduled_tick(&mut self, ctx: &mut EntityTickCtx<'_>) {
    ///         ctx.remove_self();
    ///     }
    /// }
    /// ```
    pub fn remove_self(&mut self) {
        self.removed = true;
    }

    pub(crate) fn removal_requested(&self) -> bool {
        self.removed
    }
}

impl fmt::Debug for EntityTickCtx<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EntityTickCtx")
            .field("pos", &self.pos)
            .field("removed", &self.removed)
            .finish_non_exhaustive()
    }
}
