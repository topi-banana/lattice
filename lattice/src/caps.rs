//! Capability bits ([`CapSet`]) and the facet traits built on top of them.
//!
//! A [`CapSet`] is a plain 32-bit bitset. The library reserves bits `0..8`
//! (see the associated constants); bits `8..32` are free for user-defined
//! capabilities, see [`CapSet::bit`].
//!
//! Every built-in capability has *two* representations sharing the same name:
//!
//! * a value of type [`CapSet`], used by the palette DSL (`Sticky + Movable`),
//! * a facet trait, used as a compile-time bound (`T: Sticky + Movable`).
//!
//! Traits live in the type namespace and constants live in the value
//! namespace, so Rust picks the right meaning from context.

use core::ops::{Add, BitOr, BitOrAssign};

use crate::def::BlockDef;

/// The names used by [`CapSet`]'s [`Debug`](core::fmt::Debug) implementation.
const KNOWN_CAPS: &[(u32, &str)] = &[
    (CapSet::STICKY.0, "STICKY"),
    (CapSet::MOVABLE.0, "MOVABLE"),
    (CapSet::INSTANT_MINABLE.0, "INSTANT_MINABLE"),
    (CapSet::BREAKABLE.0, "BREAKABLE"),
    (CapSet::UNMOVABLE.0, "UNMOVABLE"),
    (CapSet::EMPTY.0, "EMPTY"),
    (CapSet::FLAMMABLE.0, "FLAMMABLE"),
    (CapSet::POWER.0, "POWER"),
];

/// A set of block capabilities.
///
/// Values compose with `+` (which is bitwise or), so the palette DSL reads
/// like a trait bound: `Sticky + Movable + InstantMinable`.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct CapSet(u32);

impl CapSet {
    /// The empty set.
    pub const NONE: Self = Self(0);

    /// Sticky blocks drag their neighbours along when moved.
    pub const STICKY: Self = Self(1 << 0);

    /// Blocks that a piston may move.
    pub const MOVABLE: Self = Self(1 << 1);

    /// Blocks that break in a single step when mined.
    pub const INSTANT_MINABLE: Self = Self(1 << 2);

    /// Blocks that can be mined (possibly over several ticks).
    pub const BREAKABLE: Self = Self(1 << 3);

    /// Blocks that refuse to be moved, even if they otherwise look movable.
    pub const UNMOVABLE: Self = Self(1 << 4);

    /// The absence of material ("air").
    pub const EMPTY: Self = Self(1 << 5);

    /// Blocks that can catch fire.
    pub const FLAMMABLE: Self = Self(1 << 6);

    /// Blocks that supply power to pistons and other mechanisms.
    pub const POWER: Self = Self(1 << 7);

    /// The first bit index that is not reserved by the library.
    pub const USER_BIT_START: u8 = 8;

    /// Returns the raw bits of this set.
    pub const fn bits(self) -> u32 {
        self.0
    }

    /// Builds a set from raw bits.
    pub const fn from_bits(bits: u32) -> Self {
        Self(bits)
    }

    /// Builds a set holding a single bit. `index` wraps modulo 32.
    pub const fn bit(index: u8) -> Self {
        Self(1u32 << (index % 32))
    }

    /// Returns the union of `self` and `other`.
    pub const fn or(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    /// Returns the intersection of `self` and `other`.
    pub const fn and(self, other: Self) -> Self {
        Self(self.0 & other.0)
    }

    /// Returns `true` if every bit of `other` is also set in `self`.
    pub const fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }

    /// Returns `true` if `self` and `other` share at least one bit.
    pub const fn intersects(self, other: Self) -> bool {
        self.0 & other.0 != 0
    }

    /// Returns `true` if no bit is set.
    pub const fn is_none(self) -> bool {
        self.0 == 0
    }
}

impl BitOr for CapSet {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self {
        self.or(rhs)
    }
}

impl BitOrAssign for CapSet {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

/// `+` composes capabilities, mirroring `T: A + B` bounds.
impl Add for CapSet {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        self.or(rhs)
    }
}

impl core::fmt::Debug for CapSet {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if self.0 == 0 {
            return f.write_str("NONE");
        }
        let mut first = true;
        let mut known = 0u32;
        for &(bit, name) in KNOWN_CAPS {
            if self.0 & bit != 0 {
                if !first {
                    f.write_str(" | ")?;
                }
                f.write_str(name)?;
                first = false;
                known |= bit;
            }
        }
        let unknown = self.0 & !known;
        if unknown != 0 {
            if !first {
                f.write_str(" | ")?;
            }
            write!(f, "0x{unknown:x}")?;
        }
        Ok(())
    }
}

/// Value form of [`Sticky`](trait@Sticky); see the [module docs](self).
#[allow(non_upper_case_globals)]
pub const Sticky: CapSet = CapSet::STICKY;
/// Value form of [`Movable`](trait@Movable); see the [module docs](self).
#[allow(non_upper_case_globals)]
pub const Movable: CapSet = CapSet::MOVABLE;
/// Value form of [`InstantMinable`](trait@InstantMinable); see the [module docs](self).
#[allow(non_upper_case_globals)]
pub const InstantMinable: CapSet = CapSet::INSTANT_MINABLE;
/// Value form of [`Breakable`](trait@Breakable); see the [module docs](self).
#[allow(non_upper_case_globals)]
pub const Breakable: CapSet = CapSet::BREAKABLE;
/// Value form of [`UnMovable`](trait@UnMovable); see the [module docs](self).
#[allow(non_upper_case_globals)]
pub const UnMovable: CapSet = CapSet::UNMOVABLE;
/// Value form of [`Empty`](trait@Empty); see the [module docs](self).
#[allow(non_upper_case_globals)]
pub const Empty: CapSet = CapSet::EMPTY;
/// Value form of [`Flammable`](trait@Flammable); see the [module docs](self).
#[allow(non_upper_case_globals)]
pub const Flammable: CapSet = CapSet::FLAMMABLE;
/// Value form of [`Power`](trait@Power); see the [module docs](self).
#[allow(non_upper_case_globals)]
pub const Power: CapSet = CapSet::POWER;

/// Facet: sticky blocks drag their neighbours when moved.
///
/// Implement this trait *and* include [`CapSet::STICKY`] in
/// [`BlockDef::CAPS`]; the [`block!`](crate::block) macro does both for you.
pub trait Sticky: BlockDef {
    /// The capability bit contributed by this facet.
    const CAP: CapSet = CapSet::STICKY;
}

/// Facet: a piston may move this block.
pub trait Movable: BlockDef {
    /// The capability bit contributed by this facet.
    const CAP: CapSet = CapSet::MOVABLE;
}

/// Facet: this block breaks in a single step when mined.
pub trait InstantMinable: BlockDef {
    /// The capability bit contributed by this facet.
    const CAP: CapSet = CapSet::INSTANT_MINABLE;
}

/// Facet: this block can be mined, possibly over several ticks.
pub trait Breakable: BlockDef {
    /// The capability bit contributed by this facet.
    const CAP: CapSet = CapSet::BREAKABLE;
}

/// Facet: this block refuses to be moved.
pub trait UnMovable: BlockDef {
    /// The capability bit contributed by this facet.
    const CAP: CapSet = CapSet::UNMOVABLE;
}

/// Facet: this block is the absence of material ("air").
pub trait Empty: BlockDef {
    /// The capability bit contributed by this facet.
    const CAP: CapSet = CapSet::EMPTY;
}

/// Facet: this block can catch fire.
pub trait Flammable: BlockDef {
    /// The capability bit contributed by this facet.
    const CAP: CapSet = CapSet::FLAMMABLE;
}

/// Facet: this block supplies power to pistons and other mechanisms.
pub trait Power: BlockDef {
    /// The capability bit contributed by this facet.
    const CAP: CapSet = CapSet::POWER;
}
