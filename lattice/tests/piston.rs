//! Piston mechanics: power -> six scheduled ticks -> push.
//!
//! The piston is driven entirely by block hooks; `World::tick` is the only
//! clock.

use lattice::{
    BlockDef, CapSet, Empty, Facing, Movable, PISTON_EXTEND_TICKS, PISTON_PUSH_LIMIT, PistonBlock,
    PistonPhase, PistonState, Pos, PowerBlock, StateCodec, UnMovable, World, palette,
};

#[test]
fn piston_pushes_after_six_ticks() {
    let palette = palette![PistonBlock::source(), PowerBlock::source(), Movable, Empty];
    let mut world: World<'_, 1, 4> = World::new(&palette);

    let piston = palette.find_type::<PistonBlock>().unwrap();
    let power = palette.find_type::<PowerBlock>().unwrap();
    let movable = palette[2];

    world.set_block_with_state(
        piston,
        (0, 0, 0),
        PistonState::retracted(Facing::PosX).encode(),
    );
    world.set_block(movable, (1, 0, 0));

    // Placing the power block is the update that reaches the piston.
    world.set_block(power, (-1, 0, 0));
    assert_eq!(
        world.state::<PistonBlock>((0, 0, 0)).unwrap().phase,
        PistonPhase::Extending
    );

    // Five game ticks are not enough...
    for _ in 0..PISTON_EXTEND_TICKS - 1 {
        world.tick();
    }
    assert!(world.get(Pos::new(1, 0, 0)).unwrap().movable().is_some());
    assert!(world.get(Pos::new(2, 0, 0)).unwrap().empty().is_some());

    // ...the sixth completes the push.
    world.tick();
    assert!(world.get(Pos::new(1, 0, 0)).unwrap().empty().is_some());
    assert_eq!(world.get(Pos::new(2, 0, 0)).unwrap(), movable);
    assert_eq!(
        world.state::<PistonBlock>((0, 0, 0)).unwrap().phase,
        PistonPhase::Extended
    );
    // The piston itself stayed put.
    assert_eq!(world.get(Pos::new(0, 0, 0)).unwrap(), piston);
}

#[test]
fn piston_notices_power_when_placed() {
    let palette = palette![PistonBlock::source(), PowerBlock::source(), Movable, Empty];
    let mut world: World<'_, 1, 4> = World::new(&palette);

    let piston = palette.find_type::<PistonBlock>().unwrap();
    let power = palette.find_type::<PowerBlock>().unwrap();
    let movable = palette[2];

    // Power first, then the piston: `on_placed` sees the existing power.
    world.set_block(power, (-1, 0, 0));
    world.set_block(movable, (1, 0, 0));
    world.set_block_with_state(
        piston,
        (0, 0, 0),
        PistonState::retracted(Facing::PosX).encode(),
    );

    assert_eq!(
        world.state::<PistonBlock>((0, 0, 0)).unwrap().phase,
        PistonPhase::Extending
    );

    for _ in 0..PISTON_EXTEND_TICKS {
        world.tick();
    }
    assert_eq!(world.get(Pos::new(2, 0, 0)).unwrap(), movable);
}

#[test]
fn blocked_piston_stays_retracted() {
    let palette = palette![
        PistonBlock::source(),
        PowerBlock::source(),
        Movable,
        UnMovable,
        Empty
    ];
    let mut world: World<'_, 1, 5> = World::new(&palette);

    let piston = palette.find_type::<PistonBlock>().unwrap();
    let power = palette.find_type::<PowerBlock>().unwrap();
    let wall = palette[3];

    world.set_block(wall, (1, 0, 0));
    world.set_block_with_state(
        piston,
        (0, 0, 0),
        PistonState::retracted(Facing::PosX).encode(),
    );
    world.set_block(power, (-1, 0, 0));

    for _ in 0..PISTON_EXTEND_TICKS {
        world.tick();
    }

    assert_eq!(
        world.state::<PistonBlock>((0, 0, 0)).unwrap().phase,
        PistonPhase::Retracted
    );
    assert_eq!(world.get(Pos::new(1, 0, 0)).unwrap(), wall);
}

#[test]
fn removing_power_resets_piston() {
    let palette = palette![PistonBlock::source(), PowerBlock::source(), Movable, Empty];
    let mut world: World<'_, 1, 4> = World::new(&palette);

    let piston = palette.find_type::<PistonBlock>().unwrap();
    let power = palette.find_type::<PowerBlock>().unwrap();
    let movable = palette[2];

    world.set_block_with_state(
        piston,
        (0, 0, 0),
        PistonState::retracted(Facing::PosX).encode(),
    );
    world.set_block(movable, (1, 0, 0));
    world.set_block(power, (-1, 0, 0));
    assert_eq!(
        world.state::<PistonBlock>((0, 0, 0)).unwrap().phase,
        PistonPhase::Extending
    );

    // Losing power cancels the pending extension.
    world.remove_block(Pos::new(-1, 0, 0));
    assert_eq!(
        world.state::<PistonBlock>((0, 0, 0)).unwrap().phase,
        PistonPhase::Retracted
    );

    for _ in 0..PISTON_EXTEND_TICKS {
        world.tick();
    }
    assert!(world.get(Pos::new(1, 0, 0)).unwrap().movable().is_some());
    assert!(world.get(Pos::new(2, 0, 0)).unwrap().empty().is_some());
}

#[test]
fn piston_pushes_its_facing_direction() {
    let palette = palette![PistonBlock::source(), PowerBlock::source(), Movable, Empty];
    let mut world: World<'_, 1, 4> = World::new(&palette);

    let piston = palette.find_type::<PistonBlock>().unwrap();
    let power = palette.find_type::<PowerBlock>().unwrap();
    let movable = palette[2];

    world.set_block_with_state(
        piston,
        (0, 0, 0),
        PistonState::retracted(Facing::NegX).encode(),
    );
    world.set_block(movable, (-1, 0, 0));
    // Above the piston, outside the pushed line.
    world.set_block(power, (0, 1, 0));

    for _ in 0..PISTON_EXTEND_TICKS {
        world.tick();
    }

    assert!(world.get(Pos::new(-1, 0, 0)).unwrap().empty().is_some());
    assert_eq!(world.get(Pos::new(-2, 0, 0)).unwrap(), movable);
}

#[test]
fn piston_moves_a_whole_line() {
    let palette = palette![PistonBlock::source(), PowerBlock::source(), Movable, Empty];
    let mut world: World<'_, 1, 4> = World::new(&palette);

    let piston = palette.find_type::<PistonBlock>().unwrap();
    let power = palette.find_type::<PowerBlock>().unwrap();
    let movable = palette[2];

    world.set_block_with_state(
        piston,
        (0, 0, 0),
        PistonState::retracted(Facing::PosX).encode(),
    );
    for x in 1..=3 {
        world.set_block(movable, (x, 0, 0));
    }
    // Above the piston, outside the pushed line.
    world.set_block(power, (0, 1, 0));

    for _ in 0..PISTON_EXTEND_TICKS {
        world.tick();
    }

    assert!(world.get(Pos::new(1, 0, 0)).unwrap().empty().is_some());
    for x in 2..=4 {
        assert_eq!(world.get(Pos::new(x, 0, 0)).unwrap(), movable);
    }
}

#[test]
fn piston_gives_up_on_too_long_lines() {
    let palette = palette![PistonBlock::source(), PowerBlock::source(), Movable, Empty];
    let mut world: World<'_, 1, 4> = World::new(&palette);

    let piston = palette.find_type::<PistonBlock>().unwrap();
    let power = palette.find_type::<PowerBlock>().unwrap();
    let movable = palette[2];

    world.set_block_with_state(
        piston,
        (0, 0, 0),
        PistonState::retracted(Facing::PosX).encode(),
    );
    for x in 1..=PISTON_PUSH_LIMIT as i32 + 1 {
        world.set_block(movable, (x, 0, 0));
    }
    world.set_block(power, (0, 1, 0));

    for _ in 0..PISTON_EXTEND_TICKS {
        world.tick();
    }

    assert_eq!(
        world.state::<PistonBlock>((0, 0, 0)).unwrap().phase,
        PistonPhase::Retracted
    );
    for x in 1..=PISTON_PUSH_LIMIT as i32 + 1 {
        assert_eq!(world.get(Pos::new(x, 0, 0)).unwrap(), movable);
    }
}

#[test]
fn piston_preserves_moved_cell_state() {
    /// A movable block with a state value.
    struct Crate;

    impl BlockDef for Crate {
        const CAPS: CapSet = CapSet::MOVABLE;
        type State = u8;
    }

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
    world.set_block_with_state(krate, (1, 0, 0), 42);
    world.set_block(power, (0, 1, 0));

    for _ in 0..PISTON_EXTEND_TICKS {
        world.tick();
    }

    assert_eq!(world.get(Pos::new(2, 0, 0)).unwrap(), krate);
    assert_eq!(world.state::<Crate>((2, 0, 0)), Some(42));
}
