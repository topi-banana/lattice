//! Block entities: lifecycle, isolation from block states, and execution.
//!
//! The first half checks the automatic lifecycle (`entity: E;` in `block!`,
//! placement / removal / piston moves); the second half checks the explicit
//! API that can represent every block/entity mismatch.

use std::sync::Mutex;

use lattice::{
    BlockDef, BlockEntity, BreakOutcome, CapSet, ChunkPos, Empty, EntityTickCtx, Facing,
    InstantMinable, Movable, PISTON_EXTEND_TICKS, PistonBlock, PistonPhase, PistonState, Pos,
    PowerBlock, Rng32, StateCodec, TickCtx, UnMovable, World, block, palette,
};

/// Heap state that does not fit into the 16-bit cell state.
#[derive(Default)]
pub struct Counter {
    /// Number of scheduled ticks seen so far.
    pub ticks: u32,
}

impl BlockEntity for Counter {
    fn on_scheduled_tick(&mut self, ctx: &mut EntityTickCtx<'_>) {
        self.ticks += 1;
        ctx.schedule_self();
    }
}

block! {
    /// A block whose counter lives in a [`Counter`] block entity.
    struct CounterBlock;
    facets: UnMovable;
    entity: Counter;
}

block! {
    /// A block that keeps both cell state and a block entity.
    struct Gauge;
    facets: UnMovable;
    state: u8;
    entity: Counter;
    behavior {
        fn on_scheduled_tick(ctx: &mut TickCtx<'_>) {
            let value = ctx.state::<Self>();
            ctx.set_state::<Self>(value.wrapping_add(1));
        }
    }
}

block! {
    /// A movable block with heap payload; used for piston moves.
    struct Crate;
    facets: Movable;
    state: u8;
    entity: Counter;
}

#[test]
fn placing_creates_the_entity_and_state_updates_keep_it() {
    let palette = palette![CounterBlock::source(), Empty];
    let mut world: World<'_, 1, 2> = World::new(&palette);
    let counter = palette.find_type::<CounterBlock>().unwrap();

    world.set_block(counter, (0, 0, 0));
    assert_eq!(world.block_entity_count(), 1);
    assert_eq!(world.entity::<Counter>(Pos::ZERO).unwrap().ticks, 0);

    // The same definition with new state is a state-only update: the entity
    // is not recreated.
    world.entity_mut::<Counter>(Pos::ZERO).unwrap().ticks = 5;
    world.set_block_with_state(counter, Pos::ZERO, 9);
    assert_eq!(world.state_raw(Pos::ZERO), Some(9));
    assert_eq!(world.entity::<Counter>(Pos::ZERO).unwrap().ticks, 5);

    // A different definition replaces the entity; `Empty` has no factory.
    world.set_block(palette.empty().unwrap(), Pos::ZERO);
    assert_eq!(world.block_entity_count(), 0);
    assert!(world.block_entity(Pos::ZERO).is_none());
}

#[test]
fn removing_the_block_removes_its_entity() {
    let palette = palette![CounterBlock::source(), Empty];
    let mut world: World<'_, 1, 2> = World::new(&palette);
    let counter = palette.find_type::<CounterBlock>().unwrap();

    world.set_block(counter, (0, 0, 0));
    world.remove_block(Pos::ZERO);
    assert!(world.block_entity(Pos::ZERO).is_none());
    assert!(world.get(Pos::ZERO).unwrap().empty().is_some());
}

#[test]
fn a_scheduled_position_ticks_its_block_and_its_entity() {
    let palette = palette![Gauge::source(), Empty];
    let mut world: World<'_, 1, 2> = World::new(&palette);
    let gauge = palette.find_type::<Gauge>().unwrap();

    world.set_block(gauge, (0, 0, 0));
    world.schedule((0, 0, 0));

    assert_eq!(world.tick(), 1);
    assert_eq!(world.state::<Gauge>(Pos::ZERO), Some(1));
    assert_eq!(world.entity::<Counter>(Pos::ZERO).unwrap().ticks, 1);
}

#[test]
fn block_states_and_entities_are_isolated() {
    let palette = palette![CounterBlock::source(), Empty];
    let mut world: World<'_, 1, 2> = World::new(&palette);
    let counter = palette.find_type::<CounterBlock>().unwrap();

    // An entity can exist where the block is air.
    assert!(
        world
            .set_block_entity(Pos::ZERO, Box::new(Counter::default()))
            .is_none()
    );
    assert!(world.get(Pos::ZERO).unwrap().empty().is_some());
    assert!(world.block_entity(Pos::ZERO).is_some());

    // Removing the (already empty) cell leaves the orphan entity in place...
    world.remove_block(Pos::ZERO);
    assert!(world.block_entity(Pos::ZERO).is_some());

    // ...and it ticks from its position schedule, with no block hook.
    world.schedule(Pos::ZERO);
    assert_eq!(world.tick(), 1);
    assert_eq!(world.entity::<Counter>(Pos::ZERO).unwrap().ticks, 1);

    // Placing a block replaces the orphan with the definition's fresh entity.
    world.set_block(counter, Pos::ZERO);
    assert_eq!(world.entity::<Counter>(Pos::ZERO).unwrap().ticks, 0);

    // Removing a real block removes its entity too.
    world.remove_block(Pos::ZERO);
    assert!(world.block_entity(Pos::ZERO).is_none());
}

#[test]
fn chunks_carry_their_entities() {
    let palette = palette![CounterBlock::source(), Empty];
    let mut world: World<'_, 1, 2> = World::new(&palette);

    world.set_block_entity(Pos::ZERO, Box::new(Counter::default()));
    let chunk = world.unload_chunk(ChunkPos::new(0, 0)).unwrap();

    assert_eq!(chunk.block_entity_count(), 1);
    assert!(chunk.block_entity(Pos::ZERO).is_some());
    assert!(world.block_entity_count() == 0);
    // Removing from an unloaded chunk is a no-op, not a load.
    assert!(world.remove_block_entity(Pos::ZERO).is_none());
}

#[test]
fn block_entities_iterates_loaded_chunks_in_order() {
    let palette = palette![CounterBlock::source(), Empty];
    let mut world: World<'_, 1, 2> = World::new(&palette);

    world.set_block_entity(Pos::new(16, 0, 0), Box::new(Counter::default()));
    world.set_block_entity(Pos::new(0, 0, 0), Box::new(Counter::default()));
    world.set_block_entity(Pos::new(-1, 0, 0), Box::new(Counter::default()));

    let positions: Vec<Pos> = world.block_entities().map(|(pos, _)| pos).collect();
    assert_eq!(
        positions,
        vec![Pos::new(-1, 0, 0), Pos::new(0, 0, 0), Pos::new(16, 0, 0)]
    );
    assert_eq!(world.block_entity_count(), 3);
    // x = 16 lives in chunk (1, 0), x = -1 in chunk (-1, 0).
    assert_eq!(
        world
            .chunk(ChunkPos::new(0, 0))
            .unwrap()
            .block_entity_count(),
        1
    );
}

/// A definition that creates its entity by hand in `on_placed`.
struct Socket;

impl BlockDef for Socket {
    const CAPS: CapSet = CapSet::NONE;
    type State = ();

    fn on_placed(ctx: &mut TickCtx<'_>) {
        ctx.set_block_entity(ctx.pos(), Some(Box::new(Counter { ticks: 100 })));
    }
}

#[test]
fn block_hooks_can_manage_entities_explicitly() {
    let palette = palette![Socket::source(), Empty];
    let mut world: World<'_, 1, 2> = World::new(&palette);
    let socket = palette.find_type::<Socket>().unwrap();

    world.set_block(socket, (0, 0, 0));
    assert_eq!(world.entity::<Counter>(Pos::ZERO).unwrap().ticks, 100);

    // Dropping the entity by hand leaves the block in place: the stores are
    // independent, so the mismatch is representable.
    let taken = world.remove_block_entity(Pos::ZERO).unwrap();
    assert!(taken.downcast_ref::<Counter>().is_some());
    assert_eq!(world.get(Pos::ZERO).unwrap(), socket);
    assert!(world.block_entity(Pos::ZERO).is_none());
}

block! {
    /// `on_placed` runs after the factory created the entity, so the block
    /// hook can already write to it.
    struct Valve;
    facets: UnMovable;
    entity: Counter;
    behavior {
        fn on_placed(ctx: &mut TickCtx<'_>) {
            if let Some(counter) = ctx.entity_mut::<Counter>(ctx.pos()) {
                counter.ticks = 100;
            }
        }
    }
}

#[test]
fn block_hooks_see_factory_entities() {
    let palette = palette![Valve::source(), Empty];
    let mut world: World<'_, 1, 2> = World::new(&palette);

    world.set_block(palette.find_type::<Valve>().unwrap(), (0, 0, 0));
    assert_eq!(world.entity::<Counter>(Pos::ZERO).unwrap().ticks, 100);
}

/// Records what the hook saw for its own position.
#[derive(Default)]
pub struct SelfAware {
    /// Whether the running entity was visible in the world.
    pub saw_self: bool,
}

impl BlockEntity for SelfAware {
    fn on_scheduled_tick(&mut self, ctx: &mut EntityTickCtx<'_>) {
        self.saw_self = ctx.block_entity(ctx.pos()).is_some();
    }
}

#[test]
fn the_running_entity_is_detached_then_reattached() {
    let palette = palette![CounterBlock::source(), Empty];
    let mut world: World<'_, 1, 2> = World::new(&palette);

    world.set_block_entity(Pos::ZERO, Box::new(SelfAware::default()));
    world.schedule(Pos::ZERO);
    world.tick();

    // Detached while the hook ran...
    assert!(!world.entity::<SelfAware>(Pos::ZERO).unwrap().saw_self);
    // ...and re-attached afterwards.
    assert!(world.block_entity(Pos::ZERO).is_some());
}

/// Replaces itself with a [`Counter`] from inside its own hook.
#[derive(Default)]
pub struct Replacer;

impl BlockEntity for Replacer {
    fn on_scheduled_tick(&mut self, ctx: &mut EntityTickCtx<'_>) {
        let pos = ctx.pos();
        ctx.set_block_entity(pos, Some(Box::new(Counter { ticks: 42 })));
    }
}

#[test]
fn an_entity_can_replace_itself() {
    let palette = palette![CounterBlock::source(), Empty];
    let mut world: World<'_, 1, 2> = World::new(&palette);

    world.set_block_entity(Pos::ZERO, Box::new(Replacer));
    world.schedule(Pos::ZERO);
    world.tick();

    assert_eq!(world.entity::<Counter>(Pos::ZERO).unwrap().ticks, 42);
    assert!(world.entity::<Replacer>(Pos::ZERO).is_none());
}

/// Removes itself without touching the block.
#[derive(Default)]
pub struct Vanisher;

impl BlockEntity for Vanisher {
    fn on_scheduled_tick(&mut self, ctx: &mut EntityTickCtx<'_>) {
        ctx.remove_self();
    }
}

#[test]
fn an_entity_can_remove_itself() {
    let palette = palette![CounterBlock::source(), Empty];
    let mut world: World<'_, 1, 2> = World::new(&palette);
    let block = palette.find_type::<CounterBlock>().unwrap();

    world.set_block(block, Pos::ZERO);
    world.set_block_entity(Pos::ZERO, Box::new(Vanisher));
    world.schedule(Pos::ZERO);
    world.tick();

    assert!(world.block_entity(Pos::ZERO).is_none());
    assert_eq!(world.get(Pos::ZERO).unwrap(), block);
}

/// Removes its host block from inside the entity hook.
#[derive(Default)]
pub struct Demolisher;

impl BlockEntity for Demolisher {
    fn on_scheduled_tick(&mut self, ctx: &mut EntityTickCtx<'_>) {
        ctx.remove(ctx.pos());
    }
}

block! {
    /// A block that demolishes itself through its entity.
    struct Hut;
    facets: UnMovable;
    entity: Demolisher;
}

#[test]
fn replacing_the_host_block_drops_the_entity() {
    let palette = palette![Hut::source(), Empty];
    let mut world: World<'_, 1, 2> = World::new(&palette);

    world.set_block(palette.find_type::<Hut>().unwrap(), Pos::ZERO);
    world.schedule(Pos::ZERO);
    world.tick();

    assert!(world.get(Pos::ZERO).unwrap().empty().is_some());
    assert!(world.block_entity(Pos::ZERO).is_none());
}

static BREAK_SAW_ENTITY: Mutex<Option<bool>> = Mutex::new(None);

block! {
    /// Instant-minable; `on_break` can still see its entity.
    struct Egg;
    facets: InstantMinable;
    entity: Counter;
    behavior {
        fn on_break(ctx: &mut TickCtx<'_>) {
            *BREAK_SAW_ENTITY.lock().unwrap() =
                Some(ctx.entity::<Counter>(ctx.pos()).is_some());
        }
    }
}

#[test]
fn break_hook_sees_the_entity_before_removal() {
    let palette = palette![Egg::source(), Empty];
    let mut world: World<'_, 1, 2> = World::new(&palette);
    let egg = palette.find_type::<Egg>().unwrap();

    world.set_block(egg, Pos::ZERO);
    world.entity_mut::<Counter>(Pos::ZERO).unwrap().ticks = 3;

    assert_eq!(world.break_block(Pos::ZERO), BreakOutcome::Broken);
    assert_eq!(*BREAK_SAW_ENTITY.lock().unwrap(), Some(true));
    assert!(world.block_entity(Pos::ZERO).is_none());
    assert!(world.get(Pos::ZERO).unwrap().empty().is_some());
}

#[test]
fn piston_carries_block_entities() {
    let palette = palette![
        PistonBlock::source(),
        PowerBlock::source(),
        Crate::source(),
        Empty
    ];
    let mut world: World<'_, 1, 4> = World::new(&palette);

    let piston = palette.find_type::<PistonBlock>().unwrap();
    let power = palette.find_type::<PowerBlock>().unwrap();
    let krate = palette.find_type::<Crate>().unwrap();

    world.set_block_with_state(
        piston,
        (0, 0, 0),
        PistonState::retracted(Facing::PosX).encode(),
    );
    world.set_block_with_state(krate, (1, 0, 0), 7);
    world.set_block_with_state(krate, (2, 0, 0), 8);
    world.entity_mut::<Counter>((1, 0, 0)).unwrap().ticks = 1;
    world.entity_mut::<Counter>((2, 0, 0)).unwrap().ticks = 2;
    world.set_block(power, (0, 1, 0));

    for _ in 0..PISTON_EXTEND_TICKS {
        world.tick();
    }

    // Each entity travelled with its own cell, cell states included.
    assert_eq!(world.state::<Crate>((2, 0, 0)), Some(7));
    assert_eq!(world.state::<Crate>((3, 0, 0)), Some(8));
    assert_eq!(world.entity::<Counter>((2, 0, 0)).unwrap().ticks, 1);
    assert_eq!(world.entity::<Counter>((3, 0, 0)).unwrap().ticks, 2);
    assert!(world.block_entity((1, 0, 0)).is_none());
}

#[test]
fn piston_is_blocked_by_an_entity_in_the_destination() {
    let palette = palette![
        PistonBlock::source(),
        PowerBlock::source(),
        Crate::source(),
        Empty
    ];
    let mut world: World<'_, 1, 4> = World::new(&palette);

    let piston = palette.find_type::<PistonBlock>().unwrap();
    let power = palette.find_type::<PowerBlock>().unwrap();
    let krate = palette.find_type::<Crate>().unwrap();

    world.set_block_with_state(
        piston,
        (0, 0, 0),
        PistonState::retracted(Facing::PosX).encode(),
    );
    world.set_block_with_state(krate, (1, 0, 0), 7);
    // An orphan entity occupies the cell the line would move into.
    world.set_block_entity((2, 0, 0), Box::new(Counter::default()));
    world.set_block(power, (0, 1, 0));

    for _ in 0..PISTON_EXTEND_TICKS {
        world.tick();
    }

    // A position stores at most one entity, so the stroke failed atomically.
    assert_eq!(
        world.state::<PistonBlock>((0, 0, 0)).unwrap().phase,
        PistonPhase::Retracted
    );
    assert_eq!(world.get(Pos::new(1, 0, 0)).unwrap(), krate);
    assert!(world.entity::<Counter>((1, 0, 0)).is_some());
    assert!(world.entity::<Counter>((2, 0, 0)).is_some());
}

/// Writes its count into the entity one step to the east.
#[derive(Default)]
pub struct Propagator {
    /// Number of ticks seen.
    pub value: u32,
}

impl BlockEntity for Propagator {
    fn on_scheduled_tick(&mut self, ctx: &mut EntityTickCtx<'_>) {
        self.value += 1;
        let east = ctx.pos().offset(lattice::Dir::PosX);
        if let Some(other) = ctx.entity_mut::<Counter>(east) {
            other.ticks = self.value;
        }
    }
}

#[test]
fn entities_can_reach_each_other() {
    let palette = palette![CounterBlock::source(), Empty];
    let mut world: World<'_, 1, 2> = World::new(&palette);

    world.set_block_entity(Pos::ZERO, Box::new(Propagator::default()));
    world.set_block_entity(Pos::new(1, 0, 0), Box::new(Counter::default()));
    world.schedule(Pos::ZERO);
    world.tick();

    // The running entity is detached, but its neighbour is reachable.
    assert_eq!(world.entity::<Counter>((1, 0, 0)).unwrap().ticks, 1);
    assert_eq!(world.entity::<Propagator>(Pos::ZERO).unwrap().value, 1);
}

/// An entity that only reacts to random ticks.
#[derive(Default)]
pub struct Sprout {
    /// Number of random ticks seen.
    pub grown: u32,
}

impl BlockEntity for Sprout {
    fn on_random_tick(&mut self, _ctx: &mut EntityTickCtx<'_>) {
        self.grown += 1;
    }
}

#[test]
fn random_ticks_reach_block_entities() {
    let palette = palette![CounterBlock::source(), Empty];
    let mut world: World<'_, 1, 2> = World::new(&palette);

    world.set_block_entity(Pos::ZERO, Box::new(Sprout::default()));
    let mut rng = Rng32::new(7);

    let mut grown = 0;
    for _ in 0..100 {
        world.random_tick(&mut rng, 500);
        grown = world.entity::<Sprout>(Pos::ZERO).unwrap().grown;
        if grown > 0 {
            break;
        }
    }
    assert!(grown > 0);
}
