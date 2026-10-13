//! Declaring custom block types with `block!` or by hand, including state.
//!
//! Run with `cargo run --example custom_blocks`.

use lattice::{
    BlockDef, Breakable, CapSet, Empty, Movable, Pos, StateCodec, Sticky, TickCtx, World, block,
    palette,
};

block! {
    /// A slime-like block: sticky and movable, declared from facets.
    pub struct Slime;
    facets: Sticky, Movable;
}

/// A block that removes itself a few scheduled ticks after being triggered.
pub struct Fuse;

/// The fuse's per-cell state: remaining ticks.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Timer(pub u8);

impl StateCodec for Timer {
    const BITS: u8 = 8;

    fn encode(self) -> u16 {
        u16::from(self.0)
    }

    fn decode(raw: u16) -> Self {
        Self(raw as u8)
    }
}

impl BlockDef for Fuse {
    const CAPS: CapSet = CapSet::BREAKABLE;
    type State = Timer;

    fn break_ticks() -> u32 {
        4
    }

    fn on_scheduled_tick(ctx: &mut TickCtx<'_>) {
        let Timer(remaining) = ctx.state::<Self>();
        if remaining == 0 {
            ctx.remove_self();
        } else {
            ctx.set_state::<Self>(Timer(remaining - 1));
            ctx.schedule_self();
        }
    }
}

impl Breakable for Fuse {}

/// Facets are also ordinary generic bounds.
fn sticky_movable_caps<T: Sticky + Movable>() -> CapSet {
    <T as Sticky>::CAP.or(<T as Movable>::CAP)
}

fn main() {
    let palette = palette![Slime::source(), Fuse::source(), Empty];
    let mut world: World<'_, 1, 3> = World::new(&palette);

    let slime = palette.find_type::<Slime>().unwrap();
    let fuse = palette.find_type::<Fuse>().unwrap();

    println!(
        "Slime satisfies `T: Sticky + Movable` -> {:?}",
        sticky_movable_caps::<Slime>()
    );

    world.set_block(slime, (0, 0, 0));
    world.set_block_with_state(fuse, (1, 0, 0), Timer(2).encode());
    world.schedule((1, 0, 0));

    for tick in 0..4 {
        let processed = world.tick();
        let present = world.get(Pos::new(1, 0, 0)).unwrap().empty().is_none();
        println!("tick {tick}: processed={processed}, fuse present={present}");
    }

    let slime_positions: Vec<Pos> = world.positions_of::<Slime>().collect();
    println!("Slime positions: {slime_positions:?}");
}
