//! Quick-start mirroring the crate-level design sketch.
//!
//! Run with `cargo run --example basic`.

// The sketch passes `&palette[0]`; both `&Block` and `Block` are accepted.
#![allow(clippy::needless_borrows_for_generic_args)]

use lattice::{
    BlockPalette, Breakable, Empty, InstantMinable, Movable, Pos, Sticky, UnMovable, World,
};

fn main() {
    // Definitions live in a palette; `+` composes capabilities.
    let palette = BlockPalette::from([
        Sticky + Movable + InstantMinable,
        Breakable + Movable,
        UnMovable,
        Empty,
    ]);

    // H = 2 sub-chunks (32 blocks tall); horizontally the world grows on
    // demand. Palette capacity P = 4.
    let mut world: World<'_, 2, 4> = World::new(&palette);

    let (x, y, z) = (1, 1, 1);
    world.set_block(&palette[0], (x, y, z));
    world.set_block(&palette[2], (x + 1, y, z));

    world.schedule((x, y, z));
    let processed = world.tick();
    println!("tick processed {processed} scheduled position(s)");

    let a = world.get(Pos::new(x, y, z)).unwrap();
    let b = world.get(Pos::new(x + 1, y, z)).unwrap();
    println!("a = {a:?}");
    println!("b = {b:?}");

    // Block equality is definition equality.
    println!("a == palette[0]:            {}", a == palette[0]);

    // A facet is a boundary: the same block under different boundaries is ne.
    println!("a.movable() == a.sticky():  {}", a.movable() == a.sticky());

    // Two different definitions are ne even when they share a boundary.
    println!("a.movable() == b.movable(): {}", a.movable() == b.movable());

    println!("breaking a -> {:?}", world.break_block(Pos::new(x, y, z)));
    println!(
        "breaking b -> {:?}",
        world.break_block(Pos::new(x + 1, y, z))
    );
}
