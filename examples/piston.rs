//! Piston + redstone block: the explicit work sequence.
//!
//! 1. place the piston block (retracted, facing east),
//! 2. place the movable blocks in front of it,
//! 3. place the redstone block (`PowerBlock`) — this update starts the 6 gt
//!    countdown inside the piston,
//! 4. run six game ticks; the line moves one step.
//!
//! Run with `cargo run --example piston`.

use lattice::{
    Block, BlockDef, Empty, Facing, Movable, PISTON_EXTEND_TICKS, PistonBlock, PistonState, Pos,
    PowerBlock, StateCodec, World, palette,
};

type DemoWorld<'a> = World<'a, 1, 4>;

fn print_row(world: &DemoWorld<'_>, y: i32, piston: Block, power: Block) {
    let mut row = String::new();
    for x in -2..6 {
        let symbol = match world.get(Pos::new(x, y, 0)) {
            None => '?',
            Some(block) if block == piston => 'P',
            Some(block) if block == power => 'R',
            Some(block) if block.empty().is_some() => '.',
            Some(_) => 'M',
        };
        row.push(symbol);
    }
    println!("{row}");
}

fn main() {
    let palette = palette![PistonBlock::source(), PowerBlock::source(), Movable, Empty];
    let mut world: DemoWorld<'_> = World::new(&palette);

    let piston = palette.find_type::<PistonBlock>().unwrap();
    let power = palette.find_type::<PowerBlock>().unwrap();
    let movable = palette[2];

    // 1. Place the piston.
    world.set_block_with_state(
        piston,
        (0, 0, 0),
        PistonState::retracted(Facing::PosX).encode(),
    );

    // 2. Place the movable line in front of it.
    world.set_block(movable, (1, 0, 0));
    world.set_block(movable, (2, 0, 0));
    println!("1-2. piston + movable line:");
    print_row(&world, 0, piston, power);

    // 3. Place the redstone block next to the piston. The neighbour update
    //    reaches the piston, which enters `Extending` and schedules itself.
    world.set_block(power, (-1, 0, 0));
    let phase = world.state::<PistonBlock>((0, 0, 0)).unwrap().phase;
    println!("3. redstone placed (phase = {phase:?}):");
    print_row(&world, 0, piston, power);

    // 4. Six game ticks later the stroke completes.
    for tick in 1..=PISTON_EXTEND_TICKS {
        world.tick();
        let phase = world.state::<PistonBlock>((0, 0, 0)).unwrap().phase;
        println!("4. gt {tick:>2} (phase = {phase:?}):");
        print_row(&world, 0, piston, power);
    }

    assert!(world.get(Pos::new(1, 0, 0)).unwrap().empty().is_some());
    assert_eq!(world.get(Pos::new(2, 0, 0)).unwrap(), movable);
    assert_eq!(world.get(Pos::new(3, 0, 0)).unwrap(), movable);
    println!("line moved one step {PISTON_EXTEND_TICKS} gt after the update");
}
