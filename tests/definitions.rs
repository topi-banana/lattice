//! Definition identity, deduplication and facet boundaries.

use lattice::{BlockDef, BlockPalette, CapSet, Empty, Movable, StateCodec, Sticky, block, palette};

block! {
    /// Sticky and movable, declared from facets.
    pub struct Slime;
    facets: Sticky, Movable;
}

/// Same capabilities as [`Slime`], but a distinct definition.
pub struct Gooey;

impl BlockDef for Gooey {
    const CAPS: CapSet = CapSet::STICKY.or(CapSet::MOVABLE);
    type State = ();
}

impl Sticky for Gooey {}
impl Movable for Gooey {}

#[test]
fn structural_definitions_deduplicate() {
    let palette = BlockPalette::from([Sticky + Movable, Movable + Sticky, Empty]);

    assert_eq!(palette.len(), 2);
    assert_eq!(palette[0].caps(), Sticky + Movable);
    assert_eq!(palette[0], palette[0]);
    assert!(palette[1].empty().is_some());
}

#[test]
fn nominal_identity_is_the_type() {
    let palette = palette![
        Slime::source(),
        Gooey::source(),
        Slime::source(), // duplicate, collapsed
        Empty,
    ];

    assert_eq!(palette.len(), 3);

    let slime = palette.find_type::<Slime>().unwrap();
    let gooey = palette.find_type::<Gooey>().unwrap();

    assert_eq!(slime, palette[0]);
    assert_ne!(slime, gooey);
    assert_eq!(slime.caps(), gooey.caps()); // same capabilities...
    assert_ne!(slime.type_id(), gooey.type_id()); // ...different types
}

#[test]
fn facets_compare_boundaries() {
    let palette = palette![Slime::source(), Gooey::source(), Empty];

    let slime = palette.find_type::<Slime>().unwrap();
    let gooey = palette.find_type::<Gooey>().unwrap();

    // The same definition under the same boundary is eq.
    assert_eq!(slime.movable(), slime.movable());
    assert_eq!(slime.sticky(), slime.sticky());

    // The same definition under different boundaries is ne.
    assert_ne!(slime.movable(), slime.sticky());

    // `T: Movable` vs `S: Movable` is ne, even with identical capabilities.
    assert_ne!(slime.movable(), gooey.movable());
    assert_ne!(slime.sticky(), gooey.sticky());

    // A block that lacks the capability has no facet at all.
    let empty = palette[2];
    assert_eq!(empty.movable(), None);
    assert_eq!(empty.sticky(), None);
}

#[test]
fn custom_capability_bits_are_free() {
    const OVERGROWN: CapSet = CapSet::bit(CapSet::USER_BIT_START);

    let palette = BlockPalette::from([OVERGROWN + Movable, Empty]);
    let block = palette[0];

    assert!(block.caps().contains(OVERGROWN));
    assert!(block.caps().contains(CapSet::MOVABLE));
    assert!(block.facet(OVERGROWN).is_some());
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Timer(u8);

impl StateCodec for Timer {
    const BITS: u8 = 8;

    fn encode(self) -> u16 {
        self.0 as u16
    }

    fn decode(raw: u16) -> Self {
        Self(raw as u8)
    }
}

#[test]
fn state_codec_packs_pairs() {
    let value = (Timer(7), true);

    assert_eq!(<(Timer, bool) as StateCodec>::BITS, 9);
    assert_eq!(<(Timer, bool) as StateCodec>::decode(value.encode()), value);
}
