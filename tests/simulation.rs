//! Scheduling, ticking, hooks and breaking.

use std::sync::Mutex;

use lattice::{
    BlockDef, BlockPalette, BreakOutcome, Breakable, CapSet, ChunkPos, Dir, Empty, InstantMinable,
    Movable, Pos, Rng32, Sticky, TickCtx, UnMovable, World, palette,
};

static TICK_LOG: Mutex<Vec<Pos>> = Mutex::new(Vec::new());

/// Records every scheduled tick it receives.
struct Logger;

impl BlockDef for Logger {
    const CAPS: CapSet = CapSet::NONE;
    type State = ();

    fn on_scheduled_tick(ctx: &mut TickCtx<'_>) {
        TICK_LOG.lock().unwrap().push(ctx.pos());
    }
}

/// Schedules itself once, then stops.
struct Rescheduler;

impl BlockDef for Rescheduler {
    const CAPS: CapSet = CapSet::NONE;
    type State = u8;

    fn on_scheduled_tick(ctx: &mut TickCtx<'_>) {
        let count = ctx.state::<Self>();
        if count == 0 {
            ctx.set_state::<Self>(1);
            ctx.schedule_self();
        }
    }
}

/// Counts how many times a neighbour changed.
struct Watcher;

impl BlockDef for Watcher {
    const CAPS: CapSet = CapSet::NONE;
    type State = u8;

    fn on_neighbor_changed(ctx: &mut TickCtx<'_>, _from: Dir) {
        let count = ctx.state::<Self>();
        ctx.set_state::<Self>(count + 1);
    }
}

#[test]
fn scheduled_ticks_are_deterministic() {
    TICK_LOG.lock().unwrap().clear();

    let palette = palette![Logger::source(), Empty];
    let mut world: World<'_, 1, 2> = World::new(&palette);
    let logger = palette.find_type::<Logger>().unwrap();

    world.set_block(logger, (2, 1, 3));
    world.set_block(logger, (0, 0, 0));
    world.schedule((2, 1, 3));
    world.schedule((0, 0, 0));

    assert_eq!(world.tick(), 2);
    assert_eq!(
        *TICK_LOG.lock().unwrap(),
        vec![Pos::ZERO, Pos::new(2, 1, 3)]
    );
}

#[test]
fn rescheduling_runs_on_the_next_tick() {
    let palette = palette![Rescheduler::source(), Empty];
    let mut world: World<'_, 1, 2> = World::new(&palette);
    let block = palette.find_type::<Rescheduler>().unwrap();

    world.set_block(block, (0, 0, 0));
    world.schedule((0, 0, 0));

    assert_eq!(world.tick(), 1);
    assert_eq!(world.state::<Rescheduler>(Pos::ZERO), Some(1));

    // The hook scheduled the position again for the next pass.
    assert_eq!(world.tick(), 1);
    assert_eq!(world.tick(), 0);
}

#[test]
fn neighbor_hooks_fire_once_per_change() {
    let palette = palette![Watcher::source(), Empty, Sticky];
    let mut world: World<'_, 1, 3> = World::new(&palette);
    let watcher = palette.find_type::<Watcher>().unwrap();
    let block = palette[2];

    world.set_block(watcher, (0, 0, 0));
    assert_eq!(world.state::<Watcher>((0, 0, 0)), Some(0));

    world.set_block(block, (1, 0, 0));
    assert_eq!(world.state::<Watcher>((0, 0, 0)), Some(1));

    world.set_block(block, (0, 1, 0));
    assert_eq!(world.state::<Watcher>((0, 0, 0)), Some(2));
}

#[test]
fn breaking_follows_capabilities() {
    let palette = BlockPalette::from([
        Sticky + Movable + InstantMinable,
        Breakable,
        UnMovable,
        Empty,
    ]);
    let mut world: World<'_, 1, 4> = World::new(&palette);

    world.set_block(palette[0], (0, 0, 0));
    world.set_block(palette[1], (1, 0, 0));
    world.set_block(palette[2], (2, 0, 0));

    assert_eq!(world.break_block(Pos::new(0, 0, 0)), BreakOutcome::Broken);
    assert!(world.get(Pos::new(0, 0, 0)).unwrap().empty().is_some());

    assert_eq!(
        world.break_block(Pos::new(1, 0, 0)),
        BreakOutcome::NeedsTicks(20)
    );
    assert_eq!(
        world.break_block(Pos::new(2, 0, 0)),
        BreakOutcome::Unbreakable
    );
    assert_eq!(world.break_block(Pos::new(0, 1, 0)), BreakOutcome::Empty);
    // An unloaded chunk and the space above the world both report
    // OutOfBounds.
    assert_eq!(
        world.break_block(Pos::new(16, 0, 0)),
        BreakOutcome::OutOfBounds
    );
    assert_eq!(
        world.break_block(Pos::new(0, 16, 0)),
        BreakOutcome::OutOfBounds
    );
}

#[test]
fn random_ticks_are_deterministic() {
    let palette = BlockPalette::from([Sticky, Empty]);
    let mut world: World<'_, 1, 2> = World::new(&palette);

    let mut first = Rng32::new(7);
    let mut second = Rng32::new(7);

    for _ in 0..8 {
        assert_eq!(first.next_below(10), second.next_below(10));
    }

    // No chunks are loaded yet, so there is nothing to visit.
    let mut rng = Rng32::new(42);
    assert_eq!(world.random_tick(&mut rng, 5), 0);

    // Writing a block loads its chunk; visits always land in a loaded chunk.
    world.set_block(palette[0], (0, 0, 0));
    assert_eq!(world.random_tick(&mut rng, 5), 5);
}

#[test]
fn chunks_are_sparse_and_vertical() {
    let palette = BlockPalette::from([Sticky, Empty]);
    let mut world: World<'_, 2, 2> = World::new(&palette);
    let block = palette[0];

    // Nothing is loaded until the first write.
    assert_eq!(world.chunk_count(), 0);
    assert!(!world.is_loaded(ChunkPos::new(0, 0)));
    assert_eq!(world.get(Pos::new(0, 0, 0)), None);
    assert!(!world.schedule(Pos::new(0, 0, 0)));

    // Negative horizontal coordinates map to negative chunk coordinates.
    world.set_block(block, (-1, 0, -1));
    assert!(world.is_loaded(ChunkPos::new(-1, -1)));
    assert!(world.get(Pos::new(-1, 0, -1)).unwrap().sticky().is_some());
    assert!(world.schedule(Pos::new(-1, 0, -1)));
    assert!(world.is_scheduled(Pos::new(-1, 0, -1)));

    // H = 2 gives 32 blocks of height: y = 31 is the last section, y = 32
    // is outside.
    world.set_block(block, (0, 31, 0));
    assert!(world.get(Pos::new(0, 31, 0)).unwrap().sticky().is_some());
    assert_eq!(world.get(Pos::new(0, 32, 0)), None);
    assert_eq!(world.set_block(block, (0, 32, 0)), None);
    assert!(world.get(Pos::new(0, 0, 0)).unwrap().empty().is_some());

    // Unloading drops the chunk.
    let chunk = world.unload_chunk(ChunkPos::new(-1, -1)).unwrap();
    assert_eq!(chunk.section_count(), 2);
    assert_eq!(chunk.scheduled_count(), 1);
    assert!(!world.is_loaded(ChunkPos::new(-1, -1)));
    assert_eq!(world.get(Pos::new(-1, 0, -1)), None);
}

#[test]
fn positions_iterates_loaded_chunks() {
    let palette = BlockPalette::from([Sticky, Empty]);
    let mut world: World<'_, 1, 2> = World::new(&palette);
    let block = palette[0];

    world.set_block(block, (0, 0, 0));
    world.set_block(block, (16, 5, 0));
    world.set_block(block, (-1, 0, 0));

    let mut found: Vec<Pos> = world.positions(Sticky).collect();
    found.sort();

    assert_eq!(
        found,
        vec![Pos::new(-1, 0, 0), Pos::new(0, 0, 0), Pos::new(16, 5, 0)]
    );
}
