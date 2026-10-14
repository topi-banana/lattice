//! Block entities: heap state kept beside — and separate from — block states.
//!
//! 1. `Ticker` counts in a `u32` block entity (its cell only has 16 bits),
//! 2. one scheduled position drives both the block hook and the entity hook,
//! 3. an orphan entity can live at an air position and still tick.
//!
//! Run with `cargo run --example entities`.

use lattice::{
    BlockDef, BlockEntity, Empty, EntityTickCtx, Pos, TickCtx, UnMovable, World, block, palette,
};

block! {
    /// A `u16` of cell state plus a `u32` block entity.
    pub struct Ticker;
    facets: UnMovable;
    state: u16;
    entity: Counter;
    behavior {
        fn on_scheduled_tick(ctx: &mut TickCtx<'_>) {
            let cells = ctx.state::<Self>();
            ctx.set_state::<Self>(cells.wrapping_add(1));
        }
    }
}

/// The heap half of [`Ticker`]: does not fit into a cell.
#[derive(Default)]
pub struct Counter {
    /// Number of scheduled ticks seen.
    pub ticks: u32,
    /// Whether the entity keeps scheduling itself.
    pub running: bool,
}

impl BlockEntity for Counter {
    fn on_scheduled_tick(&mut self, ctx: &mut EntityTickCtx<'_>) {
        self.ticks += 1;
        self.running = self.ticks < 5;
        if self.running {
            ctx.schedule_self();
        }
    }
}

fn main() {
    let palette = palette![Ticker::source(), Empty];
    let mut world: World<'_, 1, 2> = World::new(&palette);
    let ticker = palette.find_type::<Ticker>().unwrap();

    // Placing the block creates the entity from `Counter::default()`.
    world.set_block(ticker, (0, 0, 0));
    world.schedule((0, 0, 0));

    // One schedule, two hooks: the block increments its cell, the entity its
    // heap counter, until the entity stops rescheduling.
    for _ in 0..6 {
        world.tick();
        let cells = world.state::<Ticker>((0, 0, 0)).unwrap();
        let ticks = world.entity::<Counter>((0, 0, 0)).unwrap().ticks;
        println!("tick: cell = {cells:>2}, entity = {ticks}");
    }

    // Block states and block entities are separate stores: an entity can
    // exist where the block is air.
    world.set_block_entity(
        (4, 1, 0),
        Box::new(Counter {
            ticks: 40,
            running: false,
        }),
    );
    println!(
        "orphan at (4, 1, 0): block empty = {}, ticks = {}",
        world.get(Pos::new(4, 1, 0)).unwrap().empty().is_some(),
        world.entity::<Counter>((4, 1, 0)).unwrap().ticks
    );

    // The orphan ticks from its own schedule, with no block hook involved.
    world.entity_mut::<Counter>((4, 1, 0)).unwrap().running = true;
    world.schedule((4, 1, 0));
    world.tick();
    println!(
        "orphan after one tick: ticks = {}",
        world.entity::<Counter>((4, 1, 0)).unwrap().ticks
    );

    println!(
        "block entities in the world: {}",
        world.block_entity_count()
    );
}
