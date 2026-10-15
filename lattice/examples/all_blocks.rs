//! 1196 Minecraft: Java Edition 26.2 blocks as a lattice palette.
//!
//! `AllBlocks::new()` registers every block of 26.2 ("Chaos Cubed") in one
//! [`BlockPalette`] and gives each block a `&Block` accessor. Run with:
//!
//! ```console
//! $ cargo run --example all_blocks
//! ```
//!
//! Data: `PrismarineJS/minecraft-data` (`pc_26_2`) and `misode/mcmeta` (`26.2-summary`).
//!
//! # Capability mapping
//!
//! - `Empty`: the three air blocks (`air`, `cave_air`, `void_air`).
//! - `Movable` / `UnMovable`: from the Minecraft Wiki piston table (Java Edition):
//!   blocks a piston cannot push (including block-entity blocks) are `UnMovable`;
//!   blocks that break when pushed carry neither facet, because lattice has no
//!   piston-destroy mechanic yet.
//! - `InstantMinable`: diggable with hardness `0`. `Breakable`: any other diggable block.
//! - `Flammable`: the Minecraft Wiki fire table plus the remaining wooden shapes.
//! - `Power`: `redstone_block`, `redstone_torch`, `redstone_wall_torch`.
//! - `Sticky`: `slime_block` and `honey_block`.
//!
//! # Known approximations
//!
//! - Behaviour outside the current facets is not modelled: TNT does not explode,
//!   water and lava do not flow, sand and gravel do not fall, chests have no
//!   inventory, redstone does not propagate, crops do not grow, and so on. Every
//!   block is still registered as a definition with the facets it can express;
//!   the commented-out signatures such blocks would need are future capabilities.
//! - Fluids (`water`, `lava`, `bubble_column`) are `UnMovable` and not breakable.
//! - State codecs mirror the vanilla property lists and store values as indices;
//!   `Default` matches the vanilla default state where a codec is shared.

#![allow(dead_code)] // every block is exposed; main() spot-checks only a few.

use lattice::{
    Block, BlockDef, BlockPalette, Breakable, CapSet, Empty, Flammable, InstantMinable, Movable,
    Pos, Power, StateCodec, Sticky, UnMovable, World, block, palette,
};

// State codecs for blocks whose vanilla states have properties.

/// State shared by 4 blocks of Minecraft 26.2 (e.g. `amethyst_cluster`, `large_amethyst_bud`, `medium_amethyst_bud`, ...).
///
/// Mirrors the vanilla block state properties and stores each value as its
/// 0-based index into the vanilla value list (4 bits in total).
///
/// - `facing`: 0=north 1=east 2=south 3=west 4=up 5=down
/// - `waterlogged`: bool
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct AmethystClusterState {
    /// `facing`.
    pub facing: u8,
    /// `waterlogged`.
    pub waterlogged: bool,
}

impl Default for AmethystClusterState {
    fn default() -> Self {
        Self {
            facing: 4,
            waterlogged: false,
        }
    }
}

impl StateCodec for AmethystClusterState {
    const BITS: u8 = 4;

    fn encode(self) -> u16 {
        self.facing.min(5) as u16 | ((self.waterlogged as u16) << 3)
    }

    fn decode(raw: u16) -> Self {
        Self {
            facing: (raw & 7) as u8,
            waterlogged: ((raw >> 3) & 1) == 1,
        }
    }
}

/// State shared by 1 block of Minecraft 26.2 (`bamboo`).
///
/// Mirrors the vanilla block state properties and stores each value as its
/// 0-based index into the vanilla value list (4 bits in total).
///
/// - `age`: 0=0 1=1
/// - `leaves`: 0=none 1=small 2=large
/// - `stage`: 0=0 1=1
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct BambooStalkState {
    /// `age`.
    pub age: u8,
    /// `leaves`.
    pub leaves: u8,
    /// `stage`.
    pub stage: u8,
}

impl StateCodec for BambooStalkState {
    const BITS: u8 = 4;

    fn encode(self) -> u16 {
        self.age.min(1) as u16
            | ((self.leaves.min(2) as u16) << 1)
            | ((self.stage.min(1) as u16) << 3)
    }

    fn decode(raw: u16) -> Self {
        Self {
            age: (raw & 1) as u8,
            leaves: ((raw >> 1) & 3) as u8,
            stage: ((raw >> 3) & 1) as u8,
        }
    }
}

/// State shared by 16 blocks of Minecraft 26.2 (e.g. `black_banner`, `blue_banner`, `brown_banner`, ...).
///
/// Mirrors the vanilla block state properties and stores each value as its
/// 0-based index into the vanilla value list (4 bits in total).
///
/// - `rotation`: 0=0 1=1 2=2 3=3 4=4 5=5 6=6 7=7 8=8 9=9 10=10 11=11 12=12 13=13 14=14 15=15
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BannerState {
    /// `rotation`.
    pub rotation: u8,
}

impl Default for BannerState {
    fn default() -> Self {
        Self { rotation: 8 }
    }
}

impl StateCodec for BannerState {
    const BITS: u8 = 4;

    fn encode(self) -> u16 {
        self.rotation.min(15) as u16
    }

    fn decode(raw: u16) -> Self {
        Self {
            rotation: (raw & 15) as u8,
        }
    }
}

/// State shared by 1 block of Minecraft 26.2 (`barrel`).
///
/// Mirrors the vanilla block state properties and stores each value as its
/// 0-based index into the vanilla value list (4 bits in total).
///
/// - `facing`: 0=north 1=east 2=south 3=west 4=up 5=down
/// - `open`: bool
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct BarrelState {
    /// `facing`.
    pub facing: u8,
    /// `open`.
    pub open: bool,
}

impl StateCodec for BarrelState {
    const BITS: u8 = 4;

    fn encode(self) -> u16 {
        self.facing.min(5) as u16 | ((self.open as u16) << 3)
    }

    fn decode(raw: u16) -> Self {
        Self {
            facing: (raw & 7) as u8,
            open: ((raw >> 3) & 1) == 1,
        }
    }
}

/// State shared by 16 blocks of Minecraft 26.2 (e.g. `black_bed`, `blue_bed`, `brown_bed`, ...).
///
/// Mirrors the vanilla block state properties and stores each value as its
/// 0-based index into the vanilla value list (4 bits in total).
///
/// - `facing`: 0=north 1=south 2=west 3=east
/// - `occupied`: bool
/// - `part`: 0=head 1=foot
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BedState {
    /// `facing`.
    pub facing: u8,
    /// `occupied`.
    pub occupied: bool,
    /// `part`.
    pub part: u8,
}

impl Default for BedState {
    fn default() -> Self {
        Self {
            facing: 0,
            occupied: false,
            part: 1,
        }
    }
}

impl StateCodec for BedState {
    const BITS: u8 = 4;

    fn encode(self) -> u16 {
        self.facing.min(3) as u16 | ((self.occupied as u16) << 2) | ((self.part.min(1) as u16) << 3)
    }

    fn decode(raw: u16) -> Self {
        Self {
            facing: (raw & 3) as u8,
            occupied: ((raw >> 2) & 1) == 1,
            part: ((raw >> 3) & 1) as u8,
        }
    }
}

/// State shared by 2 blocks of Minecraft 26.2 (`bee_nest`) (`beehive`).
///
/// Mirrors the vanilla block state properties and stores each value as its
/// 0-based index into the vanilla value list (5 bits in total).
///
/// - `facing`: 0=north 1=south 2=west 3=east
/// - `honey_level`: 0=0 1=1 2=2 3=3 4=4 5=5
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct BeehiveState {
    /// `facing`.
    pub facing: u8,
    /// `honey_level`.
    pub honey_level: u8,
}

impl StateCodec for BeehiveState {
    const BITS: u8 = 5;

    fn encode(self) -> u16 {
        self.facing.min(3) as u16 | ((self.honey_level.min(5) as u16) << 2)
    }

    fn decode(raw: u16) -> Self {
        Self {
            facing: (raw & 3) as u8,
            honey_level: ((raw >> 2) & 7) as u8,
        }
    }
}

/// State shared by 4 blocks of Minecraft 26.2 (e.g. `beetroots`, `frosted_ice`, `nether_wart`, ...).
///
/// Mirrors the vanilla block state properties and stores each value as its
/// 0-based index into the vanilla value list (2 bits in total).
///
/// - `age`: 0=0 1=1 2=2 3=3
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct BeetrootState {
    /// `age`.
    pub age: u8,
}

impl StateCodec for BeetrootState {
    const BITS: u8 = 2;

    fn encode(self) -> u16 {
        self.age.min(3) as u16
    }

    fn decode(raw: u16) -> Self {
        Self {
            age: (raw & 3) as u8,
        }
    }
}

/// State shared by 1 block of Minecraft 26.2 (`bell`).
///
/// Mirrors the vanilla block state properties and stores each value as its
/// 0-based index into the vanilla value list (5 bits in total).
///
/// - `attachment`: 0=floor 1=ceiling 2=single_wall 3=double_wall
/// - `facing`: 0=north 1=south 2=west 3=east
/// - `powered`: bool
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct BellState {
    /// `attachment`.
    pub attachment: u8,
    /// `facing`.
    pub facing: u8,
    /// `powered`.
    pub powered: bool,
}

impl StateCodec for BellState {
    const BITS: u8 = 5;

    fn encode(self) -> u16 {
        self.attachment.min(3) as u16
            | ((self.facing.min(3) as u16) << 2)
            | ((self.powered as u16) << 4)
    }

    fn decode(raw: u16) -> Self {
        Self {
            attachment: (raw & 3) as u8,
            facing: ((raw >> 2) & 3) as u8,
            powered: ((raw >> 4) & 1) == 1,
        }
    }
}

/// State shared by 1 block of Minecraft 26.2 (`big_dripleaf`).
///
/// Mirrors the vanilla block state properties and stores each value as its
/// 0-based index into the vanilla value list (5 bits in total).
///
/// - `facing`: 0=north 1=south 2=west 3=east
/// - `tilt`: 0=none 1=unstable 2=partial 3=full
/// - `waterlogged`: bool
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct BigDripleafState {
    /// `facing`.
    pub facing: u8,
    /// `tilt`.
    pub tilt: u8,
    /// `waterlogged`.
    pub waterlogged: bool,
}

impl StateCodec for BigDripleafState {
    const BITS: u8 = 5;

    fn encode(self) -> u16 {
        self.facing.min(3) as u16
            | ((self.tilt.min(3) as u16) << 2)
            | ((self.waterlogged as u16) << 4)
    }

    fn decode(raw: u16) -> Self {
        Self {
            facing: (raw & 3) as u8,
            tilt: ((raw >> 2) & 3) as u8,
            waterlogged: ((raw >> 4) & 1) == 1,
        }
    }
}

/// State shared by 4 blocks of Minecraft 26.2 (e.g. `blast_furnace`, `furnace`, `redstone_wall_torch`, ...).
///
/// Mirrors the vanilla block state properties and stores each value as its
/// 0-based index into the vanilla value list (3 bits in total).
///
/// - `facing`: 0=north 1=south 2=west 3=east
/// - `lit`: bool
///
/// The blocks sharing this state use different vanilla defaults, so
/// [`Default`] is all zeroes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct BlastFurnaceState {
    /// `facing`.
    pub facing: u8,
    /// `lit`.
    pub lit: bool,
}

impl StateCodec for BlastFurnaceState {
    const BITS: u8 = 3;

    fn encode(self) -> u16 {
        self.facing.min(3) as u16 | ((self.lit as u16) << 2)
    }

    fn decode(raw: u16) -> Self {
        Self {
            facing: (raw & 3) as u8,
            lit: ((raw >> 2) & 1) == 1,
        }
    }
}

/// State shared by 1 block of Minecraft 26.2 (`brewing_stand`).
///
/// Mirrors the vanilla block state properties and stores each value as its
/// 0-based index into the vanilla value list (3 bits in total).
///
/// - `has_bottle_0`: bool
/// - `has_bottle_1`: bool
/// - `has_bottle_2`: bool
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct BrewingStandState {
    /// `has_bottle_0`.
    pub has_bottle_0: bool,
    /// `has_bottle_1`.
    pub has_bottle_1: bool,
    /// `has_bottle_2`.
    pub has_bottle_2: bool,
}

impl StateCodec for BrewingStandState {
    const BITS: u8 = 3;

    fn encode(self) -> u16 {
        self.has_bottle_0 as u16
            | ((self.has_bottle_1 as u16) << 1)
            | ((self.has_bottle_2 as u16) << 2)
    }

    fn decode(raw: u16) -> Self {
        Self {
            has_bottle_0: (raw & 1) == 1,
            has_bottle_1: ((raw >> 1) & 1) == 1,
            has_bottle_2: ((raw >> 2) & 1) == 1,
        }
    }
}

/// State shared by 2 blocks of Minecraft 26.2 (`suspicious_gravel`) (`suspicious_sand`).
///
/// Mirrors the vanilla block state properties and stores each value as its
/// 0-based index into the vanilla value list (2 bits in total).
///
/// - `dusted`: 0=0 1=1 2=2 3=3
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct BrushableState {
    /// `dusted`.
    pub dusted: u8,
}

impl StateCodec for BrushableState {
    const BITS: u8 = 2;

    fn encode(self) -> u16 {
        self.dusted.min(3) as u16
    }

    fn decode(raw: u16) -> Self {
        Self {
            dusted: (raw & 3) as u8,
        }
    }
}

/// State shared by 1 block of Minecraft 26.2 (`bubble_column`).
///
/// Mirrors the vanilla block state properties and stores each value as its
/// 0-based index into the vanilla value list (1 bits in total).
///
/// - `drag`: bool
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BubbleColumnState {
    /// `drag`.
    pub drag: bool,
}

impl Default for BubbleColumnState {
    fn default() -> Self {
        Self { drag: true }
    }
}

impl StateCodec for BubbleColumnState {
    const BITS: u8 = 1;

    fn encode(self) -> u16 {
        self.drag as u16
    }

    fn decode(raw: u16) -> Self {
        Self {
            drag: (raw & 1) == 1,
        }
    }
}

/// State shared by 2 blocks of Minecraft 26.2 (`cactus`) (`sugar_cane`).
///
/// Mirrors the vanilla block state properties and stores each value as its
/// 0-based index into the vanilla value list (4 bits in total).
///
/// - `age`: 0=0 1=1 2=2 3=3 4=4 5=5 6=6 7=7 8=8 9=9 10=10 11=11 12=12 13=13 14=14 15=15
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct CactusState {
    /// `age`.
    pub age: u8,
}

impl StateCodec for CactusState {
    const BITS: u8 = 4;

    fn encode(self) -> u16 {
        self.age.min(15) as u16
    }

    fn decode(raw: u16) -> Self {
        Self {
            age: (raw & 15) as u8,
        }
    }
}

/// State shared by 1 block of Minecraft 26.2 (`cake`).
///
/// Mirrors the vanilla block state properties and stores each value as its
/// 0-based index into the vanilla value list (3 bits in total).
///
/// - `bites`: 0=0 1=1 2=2 3=3 4=4 5=5 6=6
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct CakeState {
    /// `bites`.
    pub bites: u8,
}

impl StateCodec for CakeState {
    const BITS: u8 = 3;

    fn encode(self) -> u16 {
        self.bites.min(6) as u16
    }

    fn decode(raw: u16) -> Self {
        Self {
            bites: (raw & 7) as u8,
        }
    }
}

/// State shared by 1 block of Minecraft 26.2 (`calibrated_sculk_sensor`).
///
/// Mirrors the vanilla block state properties and stores each value as its
/// 0-based index into the vanilla value list (9 bits in total).
///
/// - `facing`: 0=north 1=south 2=west 3=east
/// - `power`: 0=0 1=1 2=2 3=3 4=4 5=5 6=6 7=7 8=8 9=9 10=10 11=11 12=12 13=13 14=14 15=15
/// - `sculk_sensor_phase`: 0=inactive 1=active 2=cooldown
/// - `waterlogged`: bool
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct CalibratedSculkSensorState {
    /// `facing`.
    pub facing: u8,
    /// `power`.
    pub power: u8,
    /// `sculk_sensor_phase`.
    pub sculk_sensor_phase: u8,
    /// `waterlogged`.
    pub waterlogged: bool,
}

impl StateCodec for CalibratedSculkSensorState {
    const BITS: u8 = 9;

    fn encode(self) -> u16 {
        self.facing.min(3) as u16
            | ((self.power.min(15) as u16) << 2)
            | ((self.sculk_sensor_phase.min(2) as u16) << 6)
            | ((self.waterlogged as u16) << 8)
    }

    fn decode(raw: u16) -> Self {
        Self {
            facing: (raw & 3) as u8,
            power: ((raw >> 2) & 15) as u8,
            sculk_sensor_phase: ((raw >> 6) & 3) as u8,
            waterlogged: ((raw >> 8) & 1) == 1,
        }
    }
}

/// State shared by 2 blocks of Minecraft 26.2 (`campfire`) (`soul_campfire`).
///
/// Mirrors the vanilla block state properties and stores each value as its
/// 0-based index into the vanilla value list (5 bits in total).
///
/// - `facing`: 0=north 1=south 2=west 3=east
/// - `lit`: bool
/// - `signal_fire`: bool
/// - `waterlogged`: bool
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CampfireState {
    /// `facing`.
    pub facing: u8,
    /// `lit`.
    pub lit: bool,
    /// `signal_fire`.
    pub signal_fire: bool,
    /// `waterlogged`.
    pub waterlogged: bool,
}

impl Default for CampfireState {
    fn default() -> Self {
        Self {
            facing: 0,
            lit: true,
            signal_fire: false,
            waterlogged: false,
        }
    }
}

impl StateCodec for CampfireState {
    const BITS: u8 = 5;

    fn encode(self) -> u16 {
        self.facing.min(3) as u16
            | ((self.lit as u16) << 2)
            | ((self.signal_fire as u16) << 3)
            | ((self.waterlogged as u16) << 4)
    }

    fn decode(raw: u16) -> Self {
        Self {
            facing: (raw & 3) as u8,
            lit: ((raw >> 2) & 1) == 1,
            signal_fire: ((raw >> 3) & 1) == 1,
            waterlogged: ((raw >> 4) & 1) == 1,
        }
    }
}

/// State shared by 17 blocks of Minecraft 26.2 (e.g. `black_candle`, `blue_candle`, `brown_candle`, ...).
///
/// Mirrors the vanilla block state properties and stores each value as its
/// 0-based index into the vanilla value list (4 bits in total).
///
/// - `candles`: 0=1 1=2 2=3 3=4
/// - `lit`: bool
/// - `waterlogged`: bool
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct CandleState {
    /// `candles`.
    pub candles: u8,
    /// `lit`.
    pub lit: bool,
    /// `waterlogged`.
    pub waterlogged: bool,
}

impl StateCodec for CandleState {
    const BITS: u8 = 4;

    fn encode(self) -> u16 {
        self.candles.min(3) as u16 | ((self.lit as u16) << 2) | ((self.waterlogged as u16) << 3)
    }

    fn decode(raw: u16) -> Self {
        Self {
            candles: (raw & 3) as u8,
            lit: ((raw >> 2) & 1) == 1,
            waterlogged: ((raw >> 3) & 1) == 1,
        }
    }
}

/// State shared by 1 block of Minecraft 26.2 (`cave_vines_plant`).
///
/// Mirrors the vanilla block state properties and stores each value as its
/// 0-based index into the vanilla value list (1 bits in total).
///
/// - `berries`: bool
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct CaveVinesPlantState {
    /// `berries`.
    pub berries: bool,
}

impl StateCodec for CaveVinesPlantState {
    const BITS: u8 = 1;

    fn encode(self) -> u16 {
        self.berries as u16
    }

    fn decode(raw: u16) -> Self {
        Self {
            berries: (raw & 1) == 1,
        }
    }
}

/// State shared by 1 block of Minecraft 26.2 (`cave_vines`).
///
/// Mirrors the vanilla block state properties and stores each value as its
/// 0-based index into the vanilla value list (6 bits in total).
///
/// - `age`: 0=0 1=1 2=2 3=3 4=4 5=5 6=6 7=7 8=8 9=9 10=10 11=11 12=12 13=13 14=14 15=15 16=16 17=17 18=18 19=19 20=20 21=21 22=22 23=23 24=24 25=25
/// - `berries`: bool
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct CaveVinesState {
    /// `age`.
    pub age: u8,
    /// `berries`.
    pub berries: bool,
}

impl StateCodec for CaveVinesState {
    const BITS: u8 = 6;

    fn encode(self) -> u16 {
        self.age.min(25) as u16 | ((self.berries as u16) << 5)
    }

    fn decode(raw: u16) -> Self {
        Self {
            age: (raw & 31) as u8,
            berries: ((raw >> 5) & 1) == 1,
        }
    }
}

/// State shared by 10 blocks of Minecraft 26.2 (e.g. `chest`, `copper_chest`, `exposed_copper_chest`, ...).
///
/// Mirrors the vanilla block state properties and stores each value as its
/// 0-based index into the vanilla value list (5 bits in total).
///
/// - `facing`: 0=north 1=south 2=west 3=east
/// - `kind`: 0=single 1=left 2=right
/// - `waterlogged`: bool
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct ChestState {
    /// `facing`.
    pub facing: u8,
    /// `type`.
    pub kind: u8,
    /// `waterlogged`.
    pub waterlogged: bool,
}

impl StateCodec for ChestState {
    const BITS: u8 = 5;

    fn encode(self) -> u16 {
        self.facing.min(3) as u16
            | ((self.kind.min(2) as u16) << 2)
            | ((self.waterlogged as u16) << 4)
    }

    fn decode(raw: u16) -> Self {
        Self {
            facing: (raw & 3) as u8,
            kind: ((raw >> 2) & 3) as u8,
            waterlogged: ((raw >> 4) & 1) == 1,
        }
    }
}

/// State shared by 1 block of Minecraft 26.2 (`chiseled_bookshelf`).
///
/// Mirrors the vanilla block state properties and stores each value as its
/// 0-based index into the vanilla value list (8 bits in total).
///
/// - `facing`: 0=north 1=south 2=west 3=east
/// - `slot_0_occupied`: bool
/// - `slot_1_occupied`: bool
/// - `slot_2_occupied`: bool
/// - `slot_3_occupied`: bool
/// - `slot_4_occupied`: bool
/// - `slot_5_occupied`: bool
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct ChiseledBookShelfState {
    /// `facing`.
    pub facing: u8,
    /// `slot_0_occupied`.
    pub slot_0_occupied: bool,
    /// `slot_1_occupied`.
    pub slot_1_occupied: bool,
    /// `slot_2_occupied`.
    pub slot_2_occupied: bool,
    /// `slot_3_occupied`.
    pub slot_3_occupied: bool,
    /// `slot_4_occupied`.
    pub slot_4_occupied: bool,
    /// `slot_5_occupied`.
    pub slot_5_occupied: bool,
}

impl StateCodec for ChiseledBookShelfState {
    const BITS: u8 = 8;

    fn encode(self) -> u16 {
        self.facing.min(3) as u16
            | ((self.slot_0_occupied as u16) << 2)
            | ((self.slot_1_occupied as u16) << 3)
            | ((self.slot_2_occupied as u16) << 4)
            | ((self.slot_3_occupied as u16) << 5)
            | ((self.slot_4_occupied as u16) << 6)
            | ((self.slot_5_occupied as u16) << 7)
    }

    fn decode(raw: u16) -> Self {
        Self {
            facing: (raw & 3) as u8,
            slot_0_occupied: ((raw >> 2) & 1) == 1,
            slot_1_occupied: ((raw >> 3) & 1) == 1,
            slot_2_occupied: ((raw >> 4) & 1) == 1,
            slot_3_occupied: ((raw >> 5) & 1) == 1,
            slot_4_occupied: ((raw >> 6) & 1) == 1,
            slot_5_occupied: ((raw >> 7) & 1) == 1,
        }
    }
}

/// State shared by 1 block of Minecraft 26.2 (`chorus_flower`).
///
/// Mirrors the vanilla block state properties and stores each value as its
/// 0-based index into the vanilla value list (3 bits in total).
///
/// - `age`: 0=0 1=1 2=2 3=3 4=4 5=5
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct ChorusFlowerState {
    /// `age`.
    pub age: u8,
}

impl StateCodec for ChorusFlowerState {
    const BITS: u8 = 3;

    fn encode(self) -> u16 {
        self.age.min(5) as u16
    }

    fn decode(raw: u16) -> Self {
        Self {
            age: (raw & 7) as u8,
        }
    }
}

/// State shared by 1 block of Minecraft 26.2 (`cocoa`).
///
/// Mirrors the vanilla block state properties and stores each value as its
/// 0-based index into the vanilla value list (4 bits in total).
///
/// - `age`: 0=0 1=1 2=2
/// - `facing`: 0=north 1=south 2=west 3=east
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct CocoaState {
    /// `age`.
    pub age: u8,
    /// `facing`.
    pub facing: u8,
}

impl StateCodec for CocoaState {
    const BITS: u8 = 4;

    fn encode(self) -> u16 {
        self.age.min(2) as u16 | ((self.facing.min(3) as u16) << 2)
    }

    fn decode(raw: u16) -> Self {
        Self {
            age: (raw & 3) as u8,
            facing: ((raw >> 2) & 3) as u8,
        }
    }
}

/// State shared by 3 blocks of Minecraft 26.2 (`chain_command_block`) (`command_block`) (`repeating_command_block`).
///
/// Mirrors the vanilla block state properties and stores each value as its
/// 0-based index into the vanilla value list (4 bits in total).
///
/// - `conditional`: bool
/// - `facing`: 0=north 1=east 2=south 3=west 4=up 5=down
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct CommandState {
    /// `conditional`.
    pub conditional: bool,
    /// `facing`.
    pub facing: u8,
}

impl StateCodec for CommandState {
    const BITS: u8 = 4;

    fn encode(self) -> u16 {
        self.conditional as u16 | ((self.facing.min(5) as u16) << 1)
    }

    fn decode(raw: u16) -> Self {
        Self {
            conditional: (raw & 1) == 1,
            facing: ((raw >> 1) & 7) as u8,
        }
    }
}

/// State shared by 1 block of Minecraft 26.2 (`comparator`).
///
/// Mirrors the vanilla block state properties and stores each value as its
/// 0-based index into the vanilla value list (4 bits in total).
///
/// - `facing`: 0=north 1=south 2=west 3=east
/// - `mode`: 0=compare 1=subtract
/// - `powered`: bool
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct ComparatorState {
    /// `facing`.
    pub facing: u8,
    /// `mode`.
    pub mode: u8,
    /// `powered`.
    pub powered: bool,
}

impl StateCodec for ComparatorState {
    const BITS: u8 = 4;

    fn encode(self) -> u16 {
        self.facing.min(3) as u16 | ((self.mode.min(1) as u16) << 2) | ((self.powered as u16) << 3)
    }

    fn decode(raw: u16) -> Self {
        Self {
            facing: (raw & 3) as u8,
            mode: ((raw >> 2) & 1) as u8,
            powered: ((raw >> 3) & 1) == 1,
        }
    }
}

/// State shared by 1 block of Minecraft 26.2 (`composter`).
///
/// Mirrors the vanilla block state properties and stores each value as its
/// 0-based index into the vanilla value list (4 bits in total).
///
/// - `level`: 0=0 1=1 2=2 3=3 4=4 5=5 6=6 7=7 8=8
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct ComposterState {
    /// `level`.
    pub level: u8,
}

impl StateCodec for ComposterState {
    const BITS: u8 = 4;

    fn encode(self) -> u16 {
        self.level.min(8) as u16
    }

    fn decode(raw: u16) -> Self {
        Self {
            level: (raw & 15) as u8,
        }
    }
}

/// State shared by 39 blocks of Minecraft 26.2 (e.g. `acacia_fence`, `bamboo_fence`, `birch_fence`, ...).
///
/// Mirrors the vanilla block state properties and stores each value as its
/// 0-based index into the vanilla value list (5 bits in total).
///
/// - `east`: bool
/// - `north`: bool
/// - `south`: bool
/// - `waterlogged`: bool
/// - `west`: bool
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct ConnectionState {
    /// `east`.
    pub east: bool,
    /// `north`.
    pub north: bool,
    /// `south`.
    pub south: bool,
    /// `waterlogged`.
    pub waterlogged: bool,
    /// `west`.
    pub west: bool,
}

impl StateCodec for ConnectionState {
    const BITS: u8 = 5;

    fn encode(self) -> u16 {
        self.east as u16
            | ((self.north as u16) << 1)
            | ((self.south as u16) << 2)
            | ((self.waterlogged as u16) << 3)
            | ((self.west as u16) << 4)
    }

    fn decode(raw: u16) -> Self {
        Self {
            east: (raw & 1) == 1,
            north: ((raw >> 1) & 1) == 1,
            south: ((raw >> 2) & 1) == 1,
            waterlogged: ((raw >> 3) & 1) == 1,
            west: ((raw >> 4) & 1) == 1,
        }
    }
}

/// State shared by 1 block of Minecraft 26.2 (`crafter`).
///
/// Mirrors the vanilla block state properties and stores each value as its
/// 0-based index into the vanilla value list (6 bits in total).
///
/// - `crafting`: bool
/// - `orientation`: 0=down_east 1=down_north 2=down_south 3=down_west 4=up_east 5=up_north 6=up_south 7=up_west 8=west_up 9=east_up 10=north_up 11=south_up
/// - `triggered`: bool
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CrafterState {
    /// `crafting`.
    pub crafting: bool,
    /// `orientation`.
    pub orientation: u8,
    /// `triggered`.
    pub triggered: bool,
}

impl Default for CrafterState {
    fn default() -> Self {
        Self {
            crafting: false,
            orientation: 10,
            triggered: false,
        }
    }
}

impl StateCodec for CrafterState {
    const BITS: u8 = 6;

    fn encode(self) -> u16 {
        self.crafting as u16
            | ((self.orientation.min(11) as u16) << 1)
            | ((self.triggered as u16) << 5)
    }

    fn decode(raw: u16) -> Self {
        Self {
            crafting: (raw & 1) == 1,
            orientation: ((raw >> 1) & 15) as u8,
            triggered: ((raw >> 5) & 1) == 1,
        }
    }
}

/// State shared by 1 block of Minecraft 26.2 (`creaking_heart`).
///
/// Mirrors the vanilla block state properties and stores each value as its
/// 0-based index into the vanilla value list (5 bits in total).
///
/// - `axis`: 0=x 1=y 2=z
/// - `creaking_heart_state`: 0=uprooted 1=dormant 2=awake
/// - `natural`: bool
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CreakingHeartState {
    /// `axis`.
    pub axis: u8,
    /// `creaking_heart_state`.
    pub creaking_heart_state: u8,
    /// `natural`.
    pub natural: bool,
}

impl Default for CreakingHeartState {
    fn default() -> Self {
        Self {
            axis: 1,
            creaking_heart_state: 0,
            natural: false,
        }
    }
}

impl StateCodec for CreakingHeartState {
    const BITS: u8 = 5;

    fn encode(self) -> u16 {
        self.axis.min(2) as u16
            | ((self.creaking_heart_state.min(2) as u16) << 2)
            | ((self.natural as u16) << 4)
    }

    fn decode(raw: u16) -> Self {
        Self {
            axis: (raw & 3) as u8,
            creaking_heart_state: ((raw >> 2) & 3) as u8,
            natural: ((raw >> 4) & 1) == 1,
        }
    }
}

/// State shared by 5 blocks of Minecraft 26.2 (e.g. `carrots`, `melon_stem`, `potatoes`, ...).
///
/// Mirrors the vanilla block state properties and stores each value as its
/// 0-based index into the vanilla value list (3 bits in total).
///
/// - `age`: 0=0 1=1 2=2 3=3 4=4 5=5 6=6 7=7
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct CropAgeState {
    /// `age`.
    pub age: u8,
}

impl StateCodec for CropAgeState {
    const BITS: u8 = 3;

    fn encode(self) -> u16 {
        self.age.min(7) as u16
    }

    fn decode(raw: u16) -> Self {
        Self {
            age: (raw & 7) as u8,
        }
    }
}

/// State shared by 1 block of Minecraft 26.2 (`daylight_detector`).
///
/// Mirrors the vanilla block state properties and stores each value as its
/// 0-based index into the vanilla value list (5 bits in total).
///
/// - `inverted`: bool
/// - `power`: 0=0 1=1 2=2 3=3 4=4 5=5 6=6 7=7 8=8 9=9 10=10 11=11 12=12 13=13 14=14 15=15
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct DaylightDetectorState {
    /// `inverted`.
    pub inverted: bool,
    /// `power`.
    pub power: u8,
}

impl StateCodec for DaylightDetectorState {
    const BITS: u8 = 5;

    fn encode(self) -> u16 {
        self.inverted as u16 | ((self.power.min(15) as u16) << 1)
    }

    fn decode(raw: u16) -> Self {
        Self {
            inverted: (raw & 1) == 1,
            power: ((raw >> 1) & 15) as u8,
        }
    }
}

/// State shared by 1 block of Minecraft 26.2 (`decorated_pot`).
///
/// Mirrors the vanilla block state properties and stores each value as its
/// 0-based index into the vanilla value list (4 bits in total).
///
/// - `cracked`: bool
/// - `facing`: 0=north 1=south 2=west 3=east
/// - `waterlogged`: bool
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct DecoratedPotState {
    /// `cracked`.
    pub cracked: bool,
    /// `facing`.
    pub facing: u8,
    /// `waterlogged`.
    pub waterlogged: bool,
}

impl StateCodec for DecoratedPotState {
    const BITS: u8 = 4;

    fn encode(self) -> u16 {
        self.cracked as u16 | ((self.facing.min(3) as u16) << 1) | ((self.waterlogged as u16) << 3)
    }

    fn decode(raw: u16) -> Self {
        Self {
            cracked: (raw & 1) == 1,
            facing: ((raw >> 1) & 3) as u8,
            waterlogged: ((raw >> 3) & 1) == 1,
        }
    }
}

/// State shared by 2 blocks of Minecraft 26.2 (`dispenser`) (`dropper`).
///
/// Mirrors the vanilla block state properties and stores each value as its
/// 0-based index into the vanilla value list (4 bits in total).
///
/// - `facing`: 0=north 1=east 2=south 3=west 4=up 5=down
/// - `triggered`: bool
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct DispenserState {
    /// `facing`.
    pub facing: u8,
    /// `triggered`.
    pub triggered: bool,
}

impl StateCodec for DispenserState {
    const BITS: u8 = 4;

    fn encode(self) -> u16 {
        self.facing.min(5) as u16 | ((self.triggered as u16) << 3)
    }

    fn decode(raw: u16) -> Self {
        Self {
            facing: (raw & 7) as u8,
            triggered: ((raw >> 3) & 1) == 1,
        }
    }
}

/// State shared by 21 blocks of Minecraft 26.2 (e.g. `acacia_door`, `bamboo_door`, `birch_door`, ...).
///
/// Mirrors the vanilla block state properties and stores each value as its
/// 0-based index into the vanilla value list (6 bits in total).
///
/// - `facing`: 0=north 1=south 2=west 3=east
/// - `half`: 0=upper 1=lower
/// - `hinge`: 0=left 1=right
/// - `open`: bool
/// - `powered`: bool
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct DoorState {
    /// `facing`.
    pub facing: u8,
    /// `half`.
    pub half: u8,
    /// `hinge`.
    pub hinge: u8,
    /// `open`.
    pub open: bool,
    /// `powered`.
    pub powered: bool,
}

impl Default for DoorState {
    fn default() -> Self {
        Self {
            facing: 0,
            half: 1,
            hinge: 0,
            open: false,
            powered: false,
        }
    }
}

impl StateCodec for DoorState {
    const BITS: u8 = 6;

    fn encode(self) -> u16 {
        self.facing.min(3) as u16
            | ((self.half.min(1) as u16) << 2)
            | ((self.hinge.min(1) as u16) << 3)
            | ((self.open as u16) << 4)
            | ((self.powered as u16) << 5)
    }

    fn decode(raw: u16) -> Self {
        Self {
            facing: (raw & 3) as u8,
            half: ((raw >> 2) & 1) as u8,
            hinge: ((raw >> 3) & 1) as u8,
            open: ((raw >> 4) & 1) == 1,
            powered: ((raw >> 5) & 1) == 1,
        }
    }
}

/// State shared by 8 blocks of Minecraft 26.2 (e.g. `large_fern`, `lilac`, `peony`, ...).
///
/// Mirrors the vanilla block state properties and stores each value as its
/// 0-based index into the vanilla value list (1 bits in total).
///
/// - `half`: 0=upper 1=lower
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct DoublePlantState {
    /// `half`.
    pub half: u8,
}

impl Default for DoublePlantState {
    fn default() -> Self {
        Self { half: 1 }
    }
}

impl StateCodec for DoublePlantState {
    const BITS: u8 = 1;

    fn encode(self) -> u16 {
        self.half.min(1) as u16
    }

    fn decode(raw: u16) -> Self {
        Self {
            half: (raw & 1) as u8,
        }
    }
}

/// State shared by 1 block of Minecraft 26.2 (`dried_ghast`).
///
/// Mirrors the vanilla block state properties and stores each value as its
/// 0-based index into the vanilla value list (5 bits in total).
///
/// - `facing`: 0=north 1=south 2=west 3=east
/// - `hydration`: 0=0 1=1 2=2 3=3
/// - `waterlogged`: bool
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct DriedGhastState {
    /// `facing`.
    pub facing: u8,
    /// `hydration`.
    pub hydration: u8,
    /// `waterlogged`.
    pub waterlogged: bool,
}

impl StateCodec for DriedGhastState {
    const BITS: u8 = 5;

    fn encode(self) -> u16 {
        self.facing.min(3) as u16
            | ((self.hydration.min(3) as u16) << 2)
            | ((self.waterlogged as u16) << 4)
    }

    fn decode(raw: u16) -> Self {
        Self {
            facing: (raw & 3) as u8,
            hydration: ((raw >> 2) & 3) as u8,
            waterlogged: ((raw >> 4) & 1) == 1,
        }
    }
}

/// State shared by 1 block of Minecraft 26.2 (`end_portal_frame`).
///
/// Mirrors the vanilla block state properties and stores each value as its
/// 0-based index into the vanilla value list (3 bits in total).
///
/// - `eye`: bool
/// - `facing`: 0=north 1=south 2=west 3=east
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct EndPortalFrameState {
    /// `eye`.
    pub eye: bool,
    /// `facing`.
    pub facing: u8,
}

impl StateCodec for EndPortalFrameState {
    const BITS: u8 = 3;

    fn encode(self) -> u16 {
        self.eye as u16 | ((self.facing.min(3) as u16) << 1)
    }

    fn decode(raw: u16) -> Self {
        Self {
            eye: (raw & 1) == 1,
            facing: ((raw >> 1) & 3) as u8,
        }
    }
}

/// State shared by 1 block of Minecraft 26.2 (`observer`).
///
/// Mirrors the vanilla block state properties and stores each value as its
/// 0-based index into the vanilla value list (4 bits in total).
///
/// - `facing`: 0=north 1=east 2=south 3=west 4=up 5=down
/// - `powered`: bool
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FacingPoweredState {
    /// `facing`.
    pub facing: u8,
    /// `powered`.
    pub powered: bool,
}

impl Default for FacingPoweredState {
    fn default() -> Self {
        Self {
            facing: 2,
            powered: false,
        }
    }
}

impl StateCodec for FacingPoweredState {
    const BITS: u8 = 4;

    fn encode(self) -> u16 {
        self.facing.min(5) as u16 | ((self.powered as u16) << 3)
    }

    fn decode(raw: u16) -> Self {
        Self {
            facing: (raw & 7) as u8,
            powered: ((raw >> 3) & 1) == 1,
        }
    }
}

/// State shared by 44 blocks of Minecraft 26.2 (e.g. `anvil`, `attached_melon_stem`, `attached_pumpkin_stem`, ...).
///
/// Mirrors the vanilla block state properties and stores each value as its
/// 0-based index into the vanilla value list (2 bits in total).
///
/// - `facing`: 0=north 1=south 2=west 3=east
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct FacingState {
    /// `facing`.
    pub facing: u8,
}

impl StateCodec for FacingState {
    const BITS: u8 = 2;

    fn encode(self) -> u16 {
        self.facing.min(3) as u16
    }

    fn decode(raw: u16) -> Self {
        Self {
            facing: (raw & 3) as u8,
        }
    }
}

/// State shared by 1 block of Minecraft 26.2 (`farmland`).
///
/// Mirrors the vanilla block state properties and stores each value as its
/// 0-based index into the vanilla value list (3 bits in total).
///
/// - `moisture`: 0=0 1=1 2=2 3=3 4=4 5=5 6=6 7=7
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct FarmlandState {
    /// `moisture`.
    pub moisture: u8,
}

impl StateCodec for FarmlandState {
    const BITS: u8 = 3;

    fn encode(self) -> u16 {
        self.moisture.min(7) as u16
    }

    fn decode(raw: u16) -> Self {
        Self {
            moisture: (raw & 7) as u8,
        }
    }
}

/// State shared by 12 blocks of Minecraft 26.2 (e.g. `acacia_fence_gate`, `bamboo_fence_gate`, `birch_fence_gate`, ...).
///
/// Mirrors the vanilla block state properties and stores each value as its
/// 0-based index into the vanilla value list (5 bits in total).
///
/// - `facing`: 0=north 1=south 2=west 3=east
/// - `in_wall`: bool
/// - `open`: bool
/// - `powered`: bool
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct FenceGateState {
    /// `facing`.
    pub facing: u8,
    /// `in_wall`.
    pub in_wall: bool,
    /// `open`.
    pub open: bool,
    /// `powered`.
    pub powered: bool,
}

impl StateCodec for FenceGateState {
    const BITS: u8 = 5;

    fn encode(self) -> u16 {
        self.facing.min(3) as u16
            | ((self.in_wall as u16) << 2)
            | ((self.open as u16) << 3)
            | ((self.powered as u16) << 4)
    }

    fn decode(raw: u16) -> Self {
        Self {
            facing: (raw & 3) as u8,
            in_wall: ((raw >> 2) & 1) == 1,
            open: ((raw >> 3) & 1) == 1,
            powered: ((raw >> 4) & 1) == 1,
        }
    }
}

/// State shared by 1 block of Minecraft 26.2 (`fire`).
///
/// Mirrors the vanilla block state properties and stores each value as its
/// 0-based index into the vanilla value list (9 bits in total).
///
/// - `age`: 0=0 1=1 2=2 3=3 4=4 5=5 6=6 7=7 8=8 9=9 10=10 11=11 12=12 13=13 14=14 15=15
/// - `east`: bool
/// - `north`: bool
/// - `south`: bool
/// - `up`: bool
/// - `west`: bool
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct FireState {
    /// `age`.
    pub age: u8,
    /// `east`.
    pub east: bool,
    /// `north`.
    pub north: bool,
    /// `south`.
    pub south: bool,
    /// `up`.
    pub up: bool,
    /// `west`.
    pub west: bool,
}

impl StateCodec for FireState {
    const BITS: u8 = 9;

    fn encode(self) -> u16 {
        self.age.min(15) as u16
            | ((self.east as u16) << 4)
            | ((self.north as u16) << 5)
            | ((self.south as u16) << 6)
            | ((self.up as u16) << 7)
            | ((self.west as u16) << 8)
    }

    fn decode(raw: u16) -> Self {
        Self {
            age: (raw & 15) as u8,
            east: ((raw >> 4) & 1) == 1,
            north: ((raw >> 5) & 1) == 1,
            south: ((raw >> 6) & 1) == 1,
            up: ((raw >> 7) & 1) == 1,
            west: ((raw >> 8) & 1) == 1,
        }
    }
}

/// State shared by 2 blocks of Minecraft 26.2 (`pink_petals`) (`wildflowers`).
///
/// Mirrors the vanilla block state properties and stores each value as its
/// 0-based index into the vanilla value list (4 bits in total).
///
/// - `facing`: 0=north 1=south 2=west 3=east
/// - `flower_amount`: 0=1 1=2 2=3 3=4
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct FlowerBedState {
    /// `facing`.
    pub facing: u8,
    /// `flower_amount`.
    pub flower_amount: u8,
}

impl StateCodec for FlowerBedState {
    const BITS: u8 = 4;

    fn encode(self) -> u16 {
        self.facing.min(3) as u16 | ((self.flower_amount.min(3) as u16) << 2)
    }

    fn decode(raw: u16) -> Self {
        Self {
            facing: (raw & 3) as u8,
            flower_amount: ((raw >> 2) & 3) as u8,
        }
    }
}

/// State shared by 3 blocks of Minecraft 26.2 (`glow_lichen`) (`resin_clump`) (`sculk_vein`).
///
/// Mirrors the vanilla block state properties and stores each value as its
/// 0-based index into the vanilla value list (7 bits in total).
///
/// - `down`: bool
/// - `east`: bool
/// - `north`: bool
/// - `south`: bool
/// - `up`: bool
/// - `waterlogged`: bool
/// - `west`: bool
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct GlowLichenState {
    /// `down`.
    pub down: bool,
    /// `east`.
    pub east: bool,
    /// `north`.
    pub north: bool,
    /// `south`.
    pub south: bool,
    /// `up`.
    pub up: bool,
    /// `waterlogged`.
    pub waterlogged: bool,
    /// `west`.
    pub west: bool,
}

impl StateCodec for GlowLichenState {
    const BITS: u8 = 7;

    fn encode(self) -> u16 {
        self.down as u16
            | ((self.east as u16) << 1)
            | ((self.north as u16) << 2)
            | ((self.south as u16) << 3)
            | ((self.up as u16) << 4)
            | ((self.waterlogged as u16) << 5)
            | ((self.west as u16) << 6)
    }

    fn decode(raw: u16) -> Self {
        Self {
            down: (raw & 1) == 1,
            east: ((raw >> 1) & 1) == 1,
            north: ((raw >> 2) & 1) == 1,
            south: ((raw >> 3) & 1) == 1,
            up: ((raw >> 4) & 1) == 1,
            waterlogged: ((raw >> 5) & 1) == 1,
            west: ((raw >> 6) & 1) == 1,
        }
    }
}

/// State shared by 1 block of Minecraft 26.2 (`grindstone`).
///
/// Mirrors the vanilla block state properties and stores each value as its
/// 0-based index into the vanilla value list (4 bits in total).
///
/// - `face`: 0=floor 1=wall 2=ceiling
/// - `facing`: 0=north 1=south 2=west 3=east
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct GrindstoneState {
    /// `face`.
    pub face: u8,
    /// `facing`.
    pub facing: u8,
}

impl Default for GrindstoneState {
    fn default() -> Self {
        Self { face: 1, facing: 0 }
    }
}

impl StateCodec for GrindstoneState {
    const BITS: u8 = 4;

    fn encode(self) -> u16 {
        self.face.min(2) as u16 | ((self.facing.min(3) as u16) << 2)
    }

    fn decode(raw: u16) -> Self {
        Self {
            face: (raw & 3) as u8,
            facing: ((raw >> 2) & 3) as u8,
        }
    }
}

/// State shared by 1 block of Minecraft 26.2 (`pale_hanging_moss`).
///
/// Mirrors the vanilla block state properties and stores each value as its
/// 0-based index into the vanilla value list (1 bits in total).
///
/// - `tip`: bool
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct HangingMossState {
    /// `tip`.
    pub tip: bool,
}

impl Default for HangingMossState {
    fn default() -> Self {
        Self { tip: true }
    }
}

impl StateCodec for HangingMossState {
    const BITS: u8 = 1;

    fn encode(self) -> u16 {
        self.tip as u16
    }

    fn decode(raw: u16) -> Self {
        Self {
            tip: (raw & 1) == 1,
        }
    }
}

/// State shared by 12 blocks of Minecraft 26.2 (e.g. `acacia_hanging_sign`, `bamboo_hanging_sign`, `birch_hanging_sign`, ...).
///
/// Mirrors the vanilla block state properties and stores each value as its
/// 0-based index into the vanilla value list (6 bits in total).
///
/// - `attached`: bool
/// - `rotation`: 0=0 1=1 2=2 3=3 4=4 5=5 6=6 7=7 8=8 9=9 10=10 11=11 12=12 13=13 14=14 15=15
/// - `waterlogged`: bool
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct HangingSignState {
    /// `attached`.
    pub attached: bool,
    /// `rotation`.
    pub rotation: u8,
    /// `waterlogged`.
    pub waterlogged: bool,
}

impl Default for HangingSignState {
    fn default() -> Self {
        Self {
            attached: false,
            rotation: 8,
            waterlogged: false,
        }
    }
}

impl StateCodec for HangingSignState {
    const BITS: u8 = 6;

    fn encode(self) -> u16 {
        self.attached as u16
            | ((self.rotation.min(15) as u16) << 1)
            | ((self.waterlogged as u16) << 5)
    }

    fn decode(raw: u16) -> Self {
        Self {
            attached: (raw & 1) == 1,
            rotation: ((raw >> 1) & 15) as u8,
            waterlogged: ((raw >> 5) & 1) == 1,
        }
    }
}

/// State shared by 1 block of Minecraft 26.2 (`hopper`).
///
/// Mirrors the vanilla block state properties and stores each value as its
/// 0-based index into the vanilla value list (4 bits in total).
///
/// - `enabled`: bool
/// - `facing`: 0=down 1=north 2=south 3=west 4=east
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct HopperState {
    /// `enabled`.
    pub enabled: bool,
    /// `facing`.
    pub facing: u8,
}

impl Default for HopperState {
    fn default() -> Self {
        Self {
            enabled: true,
            facing: 0,
        }
    }
}

impl StateCodec for HopperState {
    const BITS: u8 = 4;

    fn encode(self) -> u16 {
        self.enabled as u16 | ((self.facing.min(4) as u16) << 1)
    }

    fn decode(raw: u16) -> Self {
        Self {
            enabled: (raw & 1) == 1,
            facing: ((raw >> 1) & 7) as u8,
        }
    }
}

/// State shared by 4 blocks of Minecraft 26.2 (e.g. `brown_mushroom_block`, `chorus_plant`, `mushroom_stem`, ...).
///
/// Mirrors the vanilla block state properties and stores each value as its
/// 0-based index into the vanilla value list (6 bits in total).
///
/// - `down`: bool
/// - `east`: bool
/// - `north`: bool
/// - `south`: bool
/// - `up`: bool
/// - `west`: bool
///
/// The blocks sharing this state use different vanilla defaults, so
/// [`Default`] is all zeroes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct HugeMushroomState {
    /// `down`.
    pub down: bool,
    /// `east`.
    pub east: bool,
    /// `north`.
    pub north: bool,
    /// `south`.
    pub south: bool,
    /// `up`.
    pub up: bool,
    /// `west`.
    pub west: bool,
}

impl StateCodec for HugeMushroomState {
    const BITS: u8 = 6;

    fn encode(self) -> u16 {
        self.down as u16
            | ((self.east as u16) << 1)
            | ((self.north as u16) << 2)
            | ((self.south as u16) << 3)
            | ((self.up as u16) << 4)
            | ((self.west as u16) << 5)
    }

    fn decode(raw: u16) -> Self {
        Self {
            down: (raw & 1) == 1,
            east: ((raw >> 1) & 1) == 1,
            north: ((raw >> 2) & 1) == 1,
            south: ((raw >> 3) & 1) == 1,
            up: ((raw >> 4) & 1) == 1,
            west: ((raw >> 5) & 1) == 1,
        }
    }
}

/// State shared by 1 block of Minecraft 26.2 (`jigsaw`).
///
/// Mirrors the vanilla block state properties and stores each value as its
/// 0-based index into the vanilla value list (4 bits in total).
///
/// - `orientation`: 0=down_east 1=down_north 2=down_south 3=down_west 4=up_east 5=up_north 6=up_south 7=up_west 8=west_up 9=east_up 10=north_up 11=south_up
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct JigsawState {
    /// `orientation`.
    pub orientation: u8,
}

impl Default for JigsawState {
    fn default() -> Self {
        Self { orientation: 10 }
    }
}

impl StateCodec for JigsawState {
    const BITS: u8 = 4;

    fn encode(self) -> u16 {
        self.orientation.min(11) as u16
    }

    fn decode(raw: u16) -> Self {
        Self {
            orientation: (raw & 15) as u8,
        }
    }
}

/// State shared by 1 block of Minecraft 26.2 (`jukebox`).
///
/// Mirrors the vanilla block state properties and stores each value as its
/// 0-based index into the vanilla value list (1 bits in total).
///
/// - `has_record`: bool
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct JukeboxState {
    /// `has_record`.
    pub has_record: bool,
}

impl StateCodec for JukeboxState {
    const BITS: u8 = 1;

    fn encode(self) -> u16 {
        self.has_record as u16
    }

    fn decode(raw: u16) -> Self {
        Self {
            has_record: (raw & 1) == 1,
        }
    }
}

/// State shared by 3 blocks of Minecraft 26.2 (`kelp`) (`twisting_vines`) (`weeping_vines`).
///
/// Mirrors the vanilla block state properties and stores each value as its
/// 0-based index into the vanilla value list (5 bits in total).
///
/// - `age`: 0=0 1=1 2=2 3=3 4=4 5=5 6=6 7=7 8=8 9=9 10=10 11=11 12=12 13=13 14=14 15=15 16=16 17=17 18=18 19=19 20=20 21=21 22=22 23=23 24=24 25=25
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct KelpState {
    /// `age`.
    pub age: u8,
}

impl StateCodec for KelpState {
    const BITS: u8 = 5;

    fn encode(self) -> u16 {
        self.age.min(25) as u16
    }

    fn decode(raw: u16) -> Self {
        Self {
            age: (raw & 31) as u8,
        }
    }
}

/// State shared by 2 blocks of Minecraft 26.2 (`powder_snow_cauldron`) (`water_cauldron`).
///
/// Mirrors the vanilla block state properties and stores each value as its
/// 0-based index into the vanilla value list (2 bits in total).
///
/// - `level`: 0=1 1=2 2=3
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct LayeredCauldronState {
    /// `level`.
    pub level: u8,
}

impl StateCodec for LayeredCauldronState {
    const BITS: u8 = 2;

    fn encode(self) -> u16 {
        self.level.min(2) as u16
    }

    fn decode(raw: u16) -> Self {
        Self {
            level: (raw & 3) as u8,
        }
    }
}

/// State shared by 1 block of Minecraft 26.2 (`leaf_litter`).
///
/// Mirrors the vanilla block state properties and stores each value as its
/// 0-based index into the vanilla value list (4 bits in total).
///
/// - `facing`: 0=north 1=south 2=west 3=east
/// - `segment_amount`: 0=1 1=2 2=3 3=4
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct LeafLitterState {
    /// `facing`.
    pub facing: u8,
    /// `segment_amount`.
    pub segment_amount: u8,
}

impl StateCodec for LeafLitterState {
    const BITS: u8 = 4;

    fn encode(self) -> u16 {
        self.facing.min(3) as u16 | ((self.segment_amount.min(3) as u16) << 2)
    }

    fn decode(raw: u16) -> Self {
        Self {
            facing: (raw & 3) as u8,
            segment_amount: ((raw >> 2) & 3) as u8,
        }
    }
}

/// State shared by 11 blocks of Minecraft 26.2 (e.g. `acacia_leaves`, `azalea_leaves`, `birch_leaves`, ...).
///
/// Mirrors the vanilla block state properties and stores each value as its
/// 0-based index into the vanilla value list (5 bits in total).
///
/// - `distance`: 0=1 1=2 2=3 3=4 4=5 5=6 6=7
/// - `persistent`: bool
/// - `waterlogged`: bool
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct LeavesState {
    /// `distance`.
    pub distance: u8,
    /// `persistent`.
    pub persistent: bool,
    /// `waterlogged`.
    pub waterlogged: bool,
}

impl Default for LeavesState {
    fn default() -> Self {
        Self {
            distance: 6,
            persistent: false,
            waterlogged: false,
        }
    }
}

impl StateCodec for LeavesState {
    const BITS: u8 = 5;

    fn encode(self) -> u16 {
        self.distance.min(6) as u16
            | ((self.persistent as u16) << 3)
            | ((self.waterlogged as u16) << 4)
    }

    fn decode(raw: u16) -> Self {
        Self {
            distance: (raw & 7) as u8,
            persistent: ((raw >> 3) & 1) == 1,
            waterlogged: ((raw >> 4) & 1) == 1,
        }
    }
}

/// State shared by 1 block of Minecraft 26.2 (`lectern`).
///
/// Mirrors the vanilla block state properties and stores each value as its
/// 0-based index into the vanilla value list (4 bits in total).
///
/// - `facing`: 0=north 1=south 2=west 3=east
/// - `has_book`: bool
/// - `powered`: bool
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct LecternState {
    /// `facing`.
    pub facing: u8,
    /// `has_book`.
    pub has_book: bool,
    /// `powered`.
    pub powered: bool,
}

impl StateCodec for LecternState {
    const BITS: u8 = 4;

    fn encode(self) -> u16 {
        self.facing.min(3) as u16 | ((self.has_book as u16) << 2) | ((self.powered as u16) << 3)
    }

    fn decode(raw: u16) -> Self {
        Self {
            facing: (raw & 3) as u8,
            has_book: ((raw >> 2) & 1) == 1,
            powered: ((raw >> 3) & 1) == 1,
        }
    }
}

/// State shared by 1 block of Minecraft 26.2 (`light`).
///
/// Mirrors the vanilla block state properties and stores each value as its
/// 0-based index into the vanilla value list (5 bits in total).
///
/// - `level`: 0=0 1=1 2=2 3=3 4=4 5=5 6=6 7=7 8=8 9=9 10=10 11=11 12=12 13=13 14=14 15=15
/// - `waterlogged`: bool
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct LightState {
    /// `level`.
    pub level: u8,
    /// `waterlogged`.
    pub waterlogged: bool,
}

impl Default for LightState {
    fn default() -> Self {
        Self {
            level: 15,
            waterlogged: false,
        }
    }
}

impl StateCodec for LightState {
    const BITS: u8 = 5;

    fn encode(self) -> u16 {
        self.level.min(15) as u16 | ((self.waterlogged as u16) << 4)
    }

    fn decode(raw: u16) -> Self {
        Self {
            level: (raw & 15) as u8,
            waterlogged: ((raw >> 4) & 1) == 1,
        }
    }
}

/// State shared by 2 blocks of Minecraft 26.2 (`lava`) (`water`).
///
/// Mirrors the vanilla block state properties and stores each value as its
/// 0-based index into the vanilla value list (4 bits in total).
///
/// - `level`: 0=0 1=1 2=2 3=3 4=4 5=5 6=6 7=7 8=8 9=9 10=10 11=11 12=12 13=13 14=14 15=15
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct LiquidState {
    /// `level`.
    pub level: u8,
}

impl StateCodec for LiquidState {
    const BITS: u8 = 4;

    fn encode(self) -> u16 {
        self.level.min(15) as u16
    }

    fn decode(raw: u16) -> Self {
        Self {
            level: (raw & 15) as u8,
        }
    }
}

/// State shared by 21 blocks of Minecraft 26.2 (e.g. `black_candle_cake`, `blue_candle_cake`, `brown_candle_cake`, ...).
///
/// Mirrors the vanilla block state properties and stores each value as its
/// 0-based index into the vanilla value list (1 bits in total).
///
/// - `lit`: bool
///
/// The blocks sharing this state use different vanilla defaults, so
/// [`Default`] is all zeroes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct LitState {
    /// `lit`.
    pub lit: bool,
}

impl StateCodec for LitState {
    const BITS: u8 = 1;

    fn encode(self) -> u16 {
        self.lit as u16
    }

    fn decode(raw: u16) -> Self {
        Self {
            lit: (raw & 1) == 1,
        }
    }
}

/// State shared by 1 block of Minecraft 26.2 (`mangrove_propagule`).
///
/// Mirrors the vanilla block state properties and stores each value as its
/// 0-based index into the vanilla value list (6 bits in total).
///
/// - `age`: 0=0 1=1 2=2 3=3 4=4
/// - `hanging`: bool
/// - `stage`: 0=0 1=1
/// - `waterlogged`: bool
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct MangrovePropaguleState {
    /// `age`.
    pub age: u8,
    /// `hanging`.
    pub hanging: bool,
    /// `stage`.
    pub stage: u8,
    /// `waterlogged`.
    pub waterlogged: bool,
}

impl StateCodec for MangrovePropaguleState {
    const BITS: u8 = 6;

    fn encode(self) -> u16 {
        self.age.min(4) as u16
            | ((self.hanging as u16) << 3)
            | ((self.stage.min(1) as u16) << 4)
            | ((self.waterlogged as u16) << 5)
    }

    fn decode(raw: u16) -> Self {
        Self {
            age: (raw & 7) as u8,
            hanging: ((raw >> 3) & 1) == 1,
            stage: ((raw >> 4) & 1) as u8,
            waterlogged: ((raw >> 5) & 1) == 1,
        }
    }
}

/// State shared by 1 block of Minecraft 26.2 (`pale_moss_carpet`).
///
/// Mirrors the vanilla block state properties and stores each value as its
/// 0-based index into the vanilla value list (9 bits in total).
///
/// - `bottom`: bool
/// - `east`: 0=none 1=low 2=tall
/// - `north`: 0=none 1=low 2=tall
/// - `south`: 0=none 1=low 2=tall
/// - `west`: 0=none 1=low 2=tall
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct MossyCarpetState {
    /// `bottom`.
    pub bottom: bool,
    /// `east`.
    pub east: u8,
    /// `north`.
    pub north: u8,
    /// `south`.
    pub south: u8,
    /// `west`.
    pub west: u8,
}

impl Default for MossyCarpetState {
    fn default() -> Self {
        Self {
            bottom: true,
            east: 0,
            north: 0,
            south: 0,
            west: 0,
        }
    }
}

impl StateCodec for MossyCarpetState {
    const BITS: u8 = 9;

    fn encode(self) -> u16 {
        self.bottom as u16
            | ((self.east.min(2) as u16) << 1)
            | ((self.north.min(2) as u16) << 3)
            | ((self.south.min(2) as u16) << 5)
            | ((self.west.min(2) as u16) << 7)
    }

    fn decode(raw: u16) -> Self {
        Self {
            bottom: (raw & 1) == 1,
            east: ((raw >> 1) & 3) as u8,
            north: ((raw >> 3) & 3) as u8,
            south: ((raw >> 5) & 3) as u8,
            west: ((raw >> 7) & 3) as u8,
        }
    }
}

/// State shared by 1 block of Minecraft 26.2 (`moving_piston`).
///
/// Mirrors the vanilla block state properties and stores each value as its
/// 0-based index into the vanilla value list (4 bits in total).
///
/// - `facing`: 0=north 1=east 2=south 3=west 4=up 5=down
/// - `kind`: 0=normal 1=sticky
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct MovingPistonState {
    /// `facing`.
    pub facing: u8,
    /// `type`.
    pub kind: u8,
}

impl StateCodec for MovingPistonState {
    const BITS: u8 = 4;

    fn encode(self) -> u16 {
        self.facing.min(5) as u16 | ((self.kind.min(1) as u16) << 3)
    }

    fn decode(raw: u16) -> Self {
        Self {
            facing: (raw & 7) as u8,
            kind: ((raw >> 3) & 1) as u8,
        }
    }
}

/// State shared by 1 block of Minecraft 26.2 (`nether_portal`).
///
/// Mirrors the vanilla block state properties and stores each value as its
/// 0-based index into the vanilla value list (1 bits in total).
///
/// - `axis`: 0=x 1=z
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct NetherPortalState {
    /// `axis`.
    pub axis: u8,
}

impl StateCodec for NetherPortalState {
    const BITS: u8 = 1;

    fn encode(self) -> u16 {
        self.axis.min(1) as u16
    }

    fn decode(raw: u16) -> Self {
        Self {
            axis: (raw & 1) as u8,
        }
    }
}

/// State shared by 1 block of Minecraft 26.2 (`note_block`).
///
/// Mirrors the vanilla block state properties and stores each value as its
/// 0-based index into the vanilla value list (11 bits in total).
///
/// - `instrument`: 0=harp 1=basedrum 2=snare 3=hat 4=bass 5=flute 6=bell 7=guitar 8=chime 9=xylophone 10=iron_xylophone 11=cow_bell 12=didgeridoo 13=bit 14=banjo 15=pling 16=trumpet 17=trumpet_exposed 18=trumpet_oxidized 19=trumpet_weathered 20=zombie 21=skeleton 22=creeper 23=dragon 24=wither_skeleton 25=piglin 26=custom_head
/// - `note`: 0=0 1=1 2=2 3=3 4=4 5=5 6=6 7=7 8=8 9=9 10=10 11=11 12=12 13=13 14=14 15=15 16=16 17=17 18=18 19=19 20=20 21=21 22=22 23=23 24=24
/// - `powered`: bool
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct NoteState {
    /// `instrument`.
    pub instrument: u8,
    /// `note`.
    pub note: u8,
    /// `powered`.
    pub powered: bool,
}

impl StateCodec for NoteState {
    const BITS: u8 = 11;

    fn encode(self) -> u16 {
        self.instrument.min(26) as u16
            | ((self.note.min(24) as u16) << 5)
            | ((self.powered as u16) << 10)
    }

    fn decode(raw: u16) -> Self {
        Self {
            instrument: (raw & 31) as u8,
            note: ((raw >> 5) & 31) as u8,
            powered: ((raw >> 10) & 1) == 1,
        }
    }
}

/// State shared by 58 blocks of Minecraft 26.2 (e.g. `acacia_log`, `acacia_wood`, `bamboo_block`, ...).
///
/// Mirrors the vanilla block state properties and stores each value as its
/// 0-based index into the vanilla value list (2 bits in total).
///
/// - `axis`: 0=x 1=y 2=z
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PillarState {
    /// `axis`.
    pub axis: u8,
}

impl Default for PillarState {
    fn default() -> Self {
        Self { axis: 1 }
    }
}

impl StateCodec for PillarState {
    const BITS: u8 = 2;

    fn encode(self) -> u16 {
        self.axis.min(2) as u16
    }

    fn decode(raw: u16) -> Self {
        Self {
            axis: (raw & 3) as u8,
        }
    }
}

/// State shared by 2 blocks of Minecraft 26.2 (`piston`) (`sticky_piston`).
///
/// Mirrors the vanilla block state properties and stores each value as its
/// 0-based index into the vanilla value list (4 bits in total).
///
/// - `extended`: bool
/// - `facing`: 0=north 1=east 2=south 3=west 4=up 5=down
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct PistonBaseState {
    /// `extended`.
    pub extended: bool,
    /// `facing`.
    pub facing: u8,
}

impl StateCodec for PistonBaseState {
    const BITS: u8 = 4;

    fn encode(self) -> u16 {
        self.extended as u16 | ((self.facing.min(5) as u16) << 1)
    }

    fn decode(raw: u16) -> Self {
        Self {
            extended: (raw & 1) == 1,
            facing: ((raw >> 1) & 7) as u8,
        }
    }
}

/// State shared by 1 block of Minecraft 26.2 (`piston_head`).
///
/// Mirrors the vanilla block state properties and stores each value as its
/// 0-based index into the vanilla value list (5 bits in total).
///
/// - `facing`: 0=north 1=east 2=south 3=west 4=up 5=down
/// - `short`: bool
/// - `kind`: 0=normal 1=sticky
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct PistonHeadState {
    /// `facing`.
    pub facing: u8,
    /// `short`.
    pub short: bool,
    /// `type`.
    pub kind: u8,
}

impl StateCodec for PistonHeadState {
    const BITS: u8 = 5;

    fn encode(self) -> u16 {
        self.facing.min(5) as u16 | ((self.short as u16) << 3) | ((self.kind.min(1) as u16) << 4)
    }

    fn decode(raw: u16) -> Self {
        Self {
            facing: (raw & 7) as u8,
            short: ((raw >> 3) & 1) == 1,
            kind: ((raw >> 4) & 1) as u8,
        }
    }
}

/// State shared by 1 block of Minecraft 26.2 (`pitcher_crop`).
///
/// Mirrors the vanilla block state properties and stores each value as its
/// 0-based index into the vanilla value list (4 bits in total).
///
/// - `age`: 0=0 1=1 2=2 3=3 4=4
/// - `half`: 0=upper 1=lower
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PitcherCropState {
    /// `age`.
    pub age: u8,
    /// `half`.
    pub half: u8,
}

impl Default for PitcherCropState {
    fn default() -> Self {
        Self { age: 0, half: 1 }
    }
}

impl StateCodec for PitcherCropState {
    const BITS: u8 = 4;

    fn encode(self) -> u16 {
        self.age.min(4) as u16 | ((self.half.min(1) as u16) << 3)
    }

    fn decode(raw: u16) -> Self {
        Self {
            age: (raw & 7) as u8,
            half: ((raw >> 3) & 1) as u8,
        }
    }
}

/// State shared by 2 blocks of Minecraft 26.2 (`pointed_dripstone`) (`sulfur_spike`).
///
/// Mirrors the vanilla block state properties and stores each value as its
/// 0-based index into the vanilla value list (5 bits in total).
///
/// - `thickness`: 0=tip_merge 1=tip 2=frustum 3=middle 4=base
/// - `vertical_direction`: 0=up 1=down
/// - `waterlogged`: bool
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PointedDripstoneState {
    /// `thickness`.
    pub thickness: u8,
    /// `vertical_direction`.
    pub vertical_direction: u8,
    /// `waterlogged`.
    pub waterlogged: bool,
}

impl Default for PointedDripstoneState {
    fn default() -> Self {
        Self {
            thickness: 1,
            vertical_direction: 0,
            waterlogged: false,
        }
    }
}

impl StateCodec for PointedDripstoneState {
    const BITS: u8 = 5;

    fn encode(self) -> u16 {
        self.thickness.min(4) as u16
            | ((self.vertical_direction.min(1) as u16) << 3)
            | ((self.waterlogged as u16) << 4)
    }

    fn decode(raw: u16) -> Self {
        Self {
            thickness: (raw & 7) as u8,
            vertical_direction: ((raw >> 3) & 1) as u8,
            waterlogged: ((raw >> 4) & 1) == 1,
        }
    }
}

/// State shared by 1 block of Minecraft 26.2 (`potent_sulfur`).
///
/// Mirrors the vanilla block state properties and stores each value as its
/// 0-based index into the vanilla value list (3 bits in total).
///
/// - `potent_sulfur_state`: 0=dry 1=wet 2=dormant 3=erupting 4=continuous
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct PotentSulfurState {
    /// `potent_sulfur_state`.
    pub potent_sulfur_state: u8,
}

impl StateCodec for PotentSulfurState {
    const BITS: u8 = 3;

    fn encode(self) -> u16 {
        self.potent_sulfur_state.min(4) as u16
    }

    fn decode(raw: u16) -> Self {
        Self {
            potent_sulfur_state: (raw & 7) as u8,
        }
    }
}

/// State shared by 3 blocks of Minecraft 26.2 (`activator_rail`) (`detector_rail`) (`powered_rail`).
///
/// Mirrors the vanilla block state properties and stores each value as its
/// 0-based index into the vanilla value list (5 bits in total).
///
/// - `powered`: bool
/// - `shape`: 0=north_south 1=east_west 2=ascending_east 3=ascending_west 4=ascending_north 5=ascending_south
/// - `waterlogged`: bool
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct PoweredRailState {
    /// `powered`.
    pub powered: bool,
    /// `shape`.
    pub shape: u8,
    /// `waterlogged`.
    pub waterlogged: bool,
}

impl StateCodec for PoweredRailState {
    const BITS: u8 = 5;

    fn encode(self) -> u16 {
        self.powered as u16 | ((self.shape.min(5) as u16) << 1) | ((self.waterlogged as u16) << 4)
    }

    fn decode(raw: u16) -> Self {
        Self {
            powered: (raw & 1) == 1,
            shape: ((raw >> 1) & 7) as u8,
            waterlogged: ((raw >> 4) & 1) == 1,
        }
    }
}

/// State shared by 14 blocks of Minecraft 26.2 (e.g. `acacia_pressure_plate`, `bamboo_pressure_plate`, `birch_pressure_plate`, ...).
///
/// Mirrors the vanilla block state properties and stores each value as its
/// 0-based index into the vanilla value list (1 bits in total).
///
/// - `powered`: bool
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct PressurePlateState {
    /// `powered`.
    pub powered: bool,
}

impl StateCodec for PressurePlateState {
    const BITS: u8 = 1;

    fn encode(self) -> u16 {
        self.powered as u16
    }

    fn decode(raw: u16) -> Self {
        Self {
            powered: (raw & 1) == 1,
        }
    }
}

/// State shared by 1 block of Minecraft 26.2 (`rail`).
///
/// Mirrors the vanilla block state properties and stores each value as its
/// 0-based index into the vanilla value list (5 bits in total).
///
/// - `shape`: 0=north_south 1=east_west 2=ascending_east 3=ascending_west 4=ascending_north 5=ascending_south 6=south_east 7=south_west 8=north_west 9=north_east
/// - `waterlogged`: bool
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct RailState {
    /// `shape`.
    pub shape: u8,
    /// `waterlogged`.
    pub waterlogged: bool,
}

impl StateCodec for RailState {
    const BITS: u8 = 5;

    fn encode(self) -> u16 {
        self.shape.min(9) as u16 | ((self.waterlogged as u16) << 4)
    }

    fn decode(raw: u16) -> Self {
        Self {
            shape: (raw & 15) as u8,
            waterlogged: ((raw >> 4) & 1) == 1,
        }
    }
}

/// State shared by 1 block of Minecraft 26.2 (`redstone_wire`).
///
/// Mirrors the vanilla block state properties and stores each value as its
/// 0-based index into the vanilla value list (12 bits in total).
///
/// - `east`: 0=up 1=side 2=none
/// - `north`: 0=up 1=side 2=none
/// - `power`: 0=0 1=1 2=2 3=3 4=4 5=5 6=6 7=7 8=8 9=9 10=10 11=11 12=12 13=13 14=14 15=15
/// - `south`: 0=up 1=side 2=none
/// - `west`: 0=up 1=side 2=none
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RedstoneWireState {
    /// `east`.
    pub east: u8,
    /// `north`.
    pub north: u8,
    /// `power`.
    pub power: u8,
    /// `south`.
    pub south: u8,
    /// `west`.
    pub west: u8,
}

impl Default for RedstoneWireState {
    fn default() -> Self {
        Self {
            east: 2,
            north: 2,
            power: 0,
            south: 2,
            west: 2,
        }
    }
}

impl StateCodec for RedstoneWireState {
    const BITS: u8 = 12;

    fn encode(self) -> u16 {
        self.east.min(2) as u16
            | ((self.north.min(2) as u16) << 2)
            | ((self.power.min(15) as u16) << 4)
            | ((self.south.min(2) as u16) << 8)
            | ((self.west.min(2) as u16) << 10)
    }

    fn decode(raw: u16) -> Self {
        Self {
            east: (raw & 3) as u8,
            north: ((raw >> 2) & 3) as u8,
            power: ((raw >> 4) & 15) as u8,
            south: ((raw >> 8) & 3) as u8,
            west: ((raw >> 10) & 3) as u8,
        }
    }
}

/// State shared by 1 block of Minecraft 26.2 (`repeater`).
///
/// Mirrors the vanilla block state properties and stores each value as its
/// 0-based index into the vanilla value list (6 bits in total).
///
/// - `delay`: 0=1 1=2 2=3 3=4
/// - `facing`: 0=north 1=south 2=west 3=east
/// - `locked`: bool
/// - `powered`: bool
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct RepeaterState {
    /// `delay`.
    pub delay: u8,
    /// `facing`.
    pub facing: u8,
    /// `locked`.
    pub locked: bool,
    /// `powered`.
    pub powered: bool,
}

impl StateCodec for RepeaterState {
    const BITS: u8 = 6;

    fn encode(self) -> u16 {
        self.delay.min(3) as u16
            | ((self.facing.min(3) as u16) << 2)
            | ((self.locked as u16) << 4)
            | ((self.powered as u16) << 5)
    }

    fn decode(raw: u16) -> Self {
        Self {
            delay: (raw & 3) as u8,
            facing: ((raw >> 2) & 3) as u8,
            locked: ((raw >> 4) & 1) == 1,
            powered: ((raw >> 5) & 1) == 1,
        }
    }
}

/// State shared by 1 block of Minecraft 26.2 (`respawn_anchor`).
///
/// Mirrors the vanilla block state properties and stores each value as its
/// 0-based index into the vanilla value list (3 bits in total).
///
/// - `charges`: 0=0 1=1 2=2 3=3 4=4
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct RespawnAnchorState {
    /// `charges`.
    pub charges: u8,
}

impl StateCodec for RespawnAnchorState {
    const BITS: u8 = 3;

    fn encode(self) -> u16 {
        self.charges.min(4) as u16
    }

    fn decode(raw: u16) -> Self {
        Self {
            charges: (raw & 7) as u8,
        }
    }
}

/// State shared by 8 blocks of Minecraft 26.2 (e.g. `acacia_sapling`, `birch_sapling`, `cherry_sapling`, ...).
///
/// Mirrors the vanilla block state properties and stores each value as its
/// 0-based index into the vanilla value list (1 bits in total).
///
/// - `stage`: 0=0 1=1
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct SaplingState {
    /// `stage`.
    pub stage: u8,
}

impl StateCodec for SaplingState {
    const BITS: u8 = 1;

    fn encode(self) -> u16 {
        self.stage.min(1) as u16
    }

    fn decode(raw: u16) -> Self {
        Self {
            stage: (raw & 1) as u8,
        }
    }
}

/// State shared by 1 block of Minecraft 26.2 (`scaffolding`).
///
/// Mirrors the vanilla block state properties and stores each value as its
/// 0-based index into the vanilla value list (5 bits in total).
///
/// - `bottom`: bool
/// - `distance`: 0=0 1=1 2=2 3=3 4=4 5=5 6=6 7=7
/// - `waterlogged`: bool
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ScaffoldingState {
    /// `bottom`.
    pub bottom: bool,
    /// `distance`.
    pub distance: u8,
    /// `waterlogged`.
    pub waterlogged: bool,
}

impl Default for ScaffoldingState {
    fn default() -> Self {
        Self {
            bottom: false,
            distance: 7,
            waterlogged: false,
        }
    }
}

impl StateCodec for ScaffoldingState {
    const BITS: u8 = 5;

    fn encode(self) -> u16 {
        self.bottom as u16 | ((self.distance.min(7) as u16) << 1) | ((self.waterlogged as u16) << 4)
    }

    fn decode(raw: u16) -> Self {
        Self {
            bottom: (raw & 1) == 1,
            distance: ((raw >> 1) & 7) as u8,
            waterlogged: ((raw >> 4) & 1) == 1,
        }
    }
}

/// State shared by 1 block of Minecraft 26.2 (`sculk_catalyst`).
///
/// Mirrors the vanilla block state properties and stores each value as its
/// 0-based index into the vanilla value list (1 bits in total).
///
/// - `bloom`: bool
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct SculkCatalystState {
    /// `bloom`.
    pub bloom: bool,
}

impl StateCodec for SculkCatalystState {
    const BITS: u8 = 1;

    fn encode(self) -> u16 {
        self.bloom as u16
    }

    fn decode(raw: u16) -> Self {
        Self {
            bloom: (raw & 1) == 1,
        }
    }
}

/// State shared by 1 block of Minecraft 26.2 (`sculk_sensor`).
///
/// Mirrors the vanilla block state properties and stores each value as its
/// 0-based index into the vanilla value list (7 bits in total).
///
/// - `power`: 0=0 1=1 2=2 3=3 4=4 5=5 6=6 7=7 8=8 9=9 10=10 11=11 12=12 13=13 14=14 15=15
/// - `sculk_sensor_phase`: 0=inactive 1=active 2=cooldown
/// - `waterlogged`: bool
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct SculkSensorState {
    /// `power`.
    pub power: u8,
    /// `sculk_sensor_phase`.
    pub sculk_sensor_phase: u8,
    /// `waterlogged`.
    pub waterlogged: bool,
}

impl StateCodec for SculkSensorState {
    const BITS: u8 = 7;

    fn encode(self) -> u16 {
        self.power.min(15) as u16
            | ((self.sculk_sensor_phase.min(2) as u16) << 4)
            | ((self.waterlogged as u16) << 6)
    }

    fn decode(raw: u16) -> Self {
        Self {
            power: (raw & 15) as u8,
            sculk_sensor_phase: ((raw >> 4) & 3) as u8,
            waterlogged: ((raw >> 6) & 1) == 1,
        }
    }
}

/// State shared by 1 block of Minecraft 26.2 (`sculk_shrieker`).
///
/// Mirrors the vanilla block state properties and stores each value as its
/// 0-based index into the vanilla value list (3 bits in total).
///
/// - `can_summon`: bool
/// - `shrieking`: bool
/// - `waterlogged`: bool
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct SculkShriekerState {
    /// `can_summon`.
    pub can_summon: bool,
    /// `shrieking`.
    pub shrieking: bool,
    /// `waterlogged`.
    pub waterlogged: bool,
}

impl StateCodec for SculkShriekerState {
    const BITS: u8 = 3;

    fn encode(self) -> u16 {
        self.can_summon as u16 | ((self.shrieking as u16) << 1) | ((self.waterlogged as u16) << 2)
    }

    fn decode(raw: u16) -> Self {
        Self {
            can_summon: (raw & 1) == 1,
            shrieking: ((raw >> 1) & 1) == 1,
            waterlogged: ((raw >> 2) & 1) == 1,
        }
    }
}

/// State shared by 1 block of Minecraft 26.2 (`sea_pickle`).
///
/// Mirrors the vanilla block state properties and stores each value as its
/// 0-based index into the vanilla value list (3 bits in total).
///
/// - `pickles`: 0=1 1=2 2=3 3=4
/// - `waterlogged`: bool
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SeaPickleState {
    /// `pickles`.
    pub pickles: u8,
    /// `waterlogged`.
    pub waterlogged: bool,
}

impl Default for SeaPickleState {
    fn default() -> Self {
        Self {
            pickles: 0,
            waterlogged: true,
        }
    }
}

impl StateCodec for SeaPickleState {
    const BITS: u8 = 3;

    fn encode(self) -> u16 {
        self.pickles.min(3) as u16 | ((self.waterlogged as u16) << 2)
    }

    fn decode(raw: u16) -> Self {
        Self {
            pickles: (raw & 3) as u8,
            waterlogged: ((raw >> 2) & 1) == 1,
        }
    }
}

/// State shared by 12 blocks of Minecraft 26.2 (e.g. `acacia_shelf`, `bamboo_shelf`, `birch_shelf`, ...).
///
/// Mirrors the vanilla block state properties and stores each value as its
/// 0-based index into the vanilla value list (6 bits in total).
///
/// - `facing`: 0=north 1=south 2=west 3=east
/// - `powered`: bool
/// - `side_chain`: 0=unconnected 1=right 2=center 3=left
/// - `waterlogged`: bool
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct ShelfState {
    /// `facing`.
    pub facing: u8,
    /// `powered`.
    pub powered: bool,
    /// `side_chain`.
    pub side_chain: u8,
    /// `waterlogged`.
    pub waterlogged: bool,
}

impl StateCodec for ShelfState {
    const BITS: u8 = 6;

    fn encode(self) -> u16 {
        self.facing.min(3) as u16
            | ((self.powered as u16) << 2)
            | ((self.side_chain.min(3) as u16) << 3)
            | ((self.waterlogged as u16) << 5)
    }

    fn decode(raw: u16) -> Self {
        Self {
            facing: (raw & 3) as u8,
            powered: ((raw >> 2) & 1) == 1,
            side_chain: ((raw >> 3) & 3) as u8,
            waterlogged: ((raw >> 5) & 1) == 1,
        }
    }
}

/// State shared by 18 blocks of Minecraft 26.2 (e.g. `black_shulker_box`, `blue_shulker_box`, `brown_shulker_box`, ...).
///
/// Mirrors the vanilla block state properties and stores each value as its
/// 0-based index into the vanilla value list (3 bits in total).
///
/// - `facing`: 0=north 1=east 2=south 3=west 4=up 5=down
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ShulkerBoxState {
    /// `facing`.
    pub facing: u8,
}

impl Default for ShulkerBoxState {
    fn default() -> Self {
        Self { facing: 4 }
    }
}

impl StateCodec for ShulkerBoxState {
    const BITS: u8 = 3;

    fn encode(self) -> u16 {
        self.facing.min(5) as u16
    }

    fn decode(raw: u16) -> Self {
        Self {
            facing: (raw & 7) as u8,
        }
    }
}

/// State shared by 7 blocks of Minecraft 26.2 (e.g. `creeper_head`, `dragon_head`, `piglin_head`, ...).
///
/// Mirrors the vanilla block state properties and stores each value as its
/// 0-based index into the vanilla value list (5 bits in total).
///
/// - `powered`: bool
/// - `rotation`: 0=0 1=1 2=2 3=3 4=4 5=5 6=6 7=7 8=8 9=9 10=10 11=11 12=12 13=13 14=14 15=15
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct SkullState {
    /// `powered`.
    pub powered: bool,
    /// `rotation`.
    pub rotation: u8,
}

impl StateCodec for SkullState {
    const BITS: u8 = 5;

    fn encode(self) -> u16 {
        self.powered as u16 | ((self.rotation.min(15) as u16) << 1)
    }

    fn decode(raw: u16) -> Self {
        Self {
            powered: (raw & 1) == 1,
            rotation: ((raw >> 1) & 15) as u8,
        }
    }
}

/// State shared by 68 blocks of Minecraft 26.2 (e.g. `acacia_slab`, `andesite_slab`, `bamboo_mosaic_slab`, ...).
///
/// Mirrors the vanilla block state properties and stores each value as its
/// 0-based index into the vanilla value list (3 bits in total).
///
/// - `kind`: 0=top 1=bottom 2=double
/// - `waterlogged`: bool
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SlabState {
    /// `type`.
    pub kind: u8,
    /// `waterlogged`.
    pub waterlogged: bool,
}

impl Default for SlabState {
    fn default() -> Self {
        Self {
            kind: 1,
            waterlogged: false,
        }
    }
}

impl StateCodec for SlabState {
    const BITS: u8 = 3;

    fn encode(self) -> u16 {
        self.kind.min(2) as u16 | ((self.waterlogged as u16) << 2)
    }

    fn decode(raw: u16) -> Self {
        Self {
            kind: (raw & 3) as u8,
            waterlogged: ((raw >> 2) & 1) == 1,
        }
    }
}

/// State shared by 1 block of Minecraft 26.2 (`small_dripleaf`).
///
/// Mirrors the vanilla block state properties and stores each value as its
/// 0-based index into the vanilla value list (4 bits in total).
///
/// - `facing`: 0=north 1=south 2=west 3=east
/// - `half`: 0=upper 1=lower
/// - `waterlogged`: bool
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SmallDripleafState {
    /// `facing`.
    pub facing: u8,
    /// `half`.
    pub half: u8,
    /// `waterlogged`.
    pub waterlogged: bool,
}

impl Default for SmallDripleafState {
    fn default() -> Self {
        Self {
            facing: 0,
            half: 1,
            waterlogged: false,
        }
    }
}

impl StateCodec for SmallDripleafState {
    const BITS: u8 = 4;

    fn encode(self) -> u16 {
        self.facing.min(3) as u16
            | ((self.half.min(1) as u16) << 2)
            | ((self.waterlogged as u16) << 3)
    }

    fn decode(raw: u16) -> Self {
        Self {
            facing: (raw & 3) as u8,
            half: ((raw >> 2) & 1) as u8,
            waterlogged: ((raw >> 3) & 1) == 1,
        }
    }
}

/// State shared by 1 block of Minecraft 26.2 (`sniffer_egg`).
///
/// Mirrors the vanilla block state properties and stores each value as its
/// 0-based index into the vanilla value list (2 bits in total).
///
/// - `hatch`: 0=0 1=1 2=2
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct SnifferEggState {
    /// `hatch`.
    pub hatch: u8,
}

impl StateCodec for SnifferEggState {
    const BITS: u8 = 2;

    fn encode(self) -> u16 {
        self.hatch.min(2) as u16
    }

    fn decode(raw: u16) -> Self {
        Self {
            hatch: (raw & 3) as u8,
        }
    }
}

/// State shared by 1 block of Minecraft 26.2 (`snow`).
///
/// Mirrors the vanilla block state properties and stores each value as its
/// 0-based index into the vanilla value list (3 bits in total).
///
/// - `layers`: 0=1 1=2 2=3 3=4 4=5 5=6 6=7 7=8
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct SnowLayerState {
    /// `layers`.
    pub layers: u8,
}

impl StateCodec for SnowLayerState {
    const BITS: u8 = 3;

    fn encode(self) -> u16 {
        self.layers.min(7) as u16
    }

    fn decode(raw: u16) -> Self {
        Self {
            layers: (raw & 7) as u8,
        }
    }
}

/// State shared by 3 blocks of Minecraft 26.2 (`grass_block`) (`mycelium`) (`podzol`).
///
/// Mirrors the vanilla block state properties and stores each value as its
/// 0-based index into the vanilla value list (1 bits in total).
///
/// - `snowy`: bool
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct SnowyState {
    /// `snowy`.
    pub snowy: bool,
}

impl StateCodec for SnowyState {
    const BITS: u8 = 1;

    fn encode(self) -> u16 {
        self.snowy as u16
    }

    fn decode(raw: u16) -> Self {
        Self {
            snowy: (raw & 1) == 1,
        }
    }
}

/// State shared by 64 blocks of Minecraft 26.2 (e.g. `acacia_stairs`, `andesite_stairs`, `bamboo_mosaic_stairs`, ...).
///
/// Mirrors the vanilla block state properties and stores each value as its
/// 0-based index into the vanilla value list (7 bits in total).
///
/// - `facing`: 0=north 1=south 2=west 3=east
/// - `half`: 0=top 1=bottom
/// - `shape`: 0=straight 1=inner_left 2=inner_right 3=outer_left 4=outer_right
/// - `waterlogged`: bool
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct StairState {
    /// `facing`.
    pub facing: u8,
    /// `half`.
    pub half: u8,
    /// `shape`.
    pub shape: u8,
    /// `waterlogged`.
    pub waterlogged: bool,
}

impl Default for StairState {
    fn default() -> Self {
        Self {
            facing: 0,
            half: 1,
            shape: 0,
            waterlogged: false,
        }
    }
}

impl StateCodec for StairState {
    const BITS: u8 = 7;

    fn encode(self) -> u16 {
        self.facing.min(3) as u16
            | ((self.half.min(1) as u16) << 2)
            | ((self.shape.min(4) as u16) << 3)
            | ((self.waterlogged as u16) << 6)
    }

    fn decode(raw: u16) -> Self {
        Self {
            facing: (raw & 3) as u8,
            half: ((raw >> 2) & 1) as u8,
            shape: ((raw >> 3) & 7) as u8,
            waterlogged: ((raw >> 6) & 1) == 1,
        }
    }
}

/// State shared by 12 blocks of Minecraft 26.2 (e.g. `acacia_sign`, `bamboo_sign`, `birch_sign`, ...).
///
/// Mirrors the vanilla block state properties and stores each value as its
/// 0-based index into the vanilla value list (5 bits in total).
///
/// - `rotation`: 0=0 1=1 2=2 3=3 4=4 5=5 6=6 7=7 8=8 9=9 10=10 11=11 12=12 13=13 14=14 15=15
/// - `waterlogged`: bool
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct StandingSignState {
    /// `rotation`.
    pub rotation: u8,
    /// `waterlogged`.
    pub waterlogged: bool,
}

impl Default for StandingSignState {
    fn default() -> Self {
        Self {
            rotation: 8,
            waterlogged: false,
        }
    }
}

impl StateCodec for StandingSignState {
    const BITS: u8 = 5;

    fn encode(self) -> u16 {
        self.rotation.min(15) as u16 | ((self.waterlogged as u16) << 4)
    }

    fn decode(raw: u16) -> Self {
        Self {
            rotation: (raw & 15) as u8,
            waterlogged: ((raw >> 4) & 1) == 1,
        }
    }
}

/// State shared by 1 block of Minecraft 26.2 (`structure_block`).
///
/// Mirrors the vanilla block state properties and stores each value as its
/// 0-based index into the vanilla value list (2 bits in total).
///
/// - `mode`: 0=save 1=load 2=corner 3=data
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct StructureState {
    /// `mode`.
    pub mode: u8,
}

impl Default for StructureState {
    fn default() -> Self {
        Self { mode: 1 }
    }
}

impl StateCodec for StructureState {
    const BITS: u8 = 2;

    fn encode(self) -> u16 {
        self.mode.min(3) as u16
    }

    fn decode(raw: u16) -> Self {
        Self {
            mode: (raw & 3) as u8,
        }
    }
}

/// State shared by 15 blocks of Minecraft 26.2 (e.g. `acacia_button`, `bamboo_button`, `birch_button`, ...).
///
/// Mirrors the vanilla block state properties and stores each value as its
/// 0-based index into the vanilla value list (5 bits in total).
///
/// - `face`: 0=floor 1=wall 2=ceiling
/// - `facing`: 0=north 1=south 2=west 3=east
/// - `powered`: bool
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SwitchState {
    /// `face`.
    pub face: u8,
    /// `facing`.
    pub facing: u8,
    /// `powered`.
    pub powered: bool,
}

impl Default for SwitchState {
    fn default() -> Self {
        Self {
            face: 1,
            facing: 0,
            powered: false,
        }
    }
}

impl StateCodec for SwitchState {
    const BITS: u8 = 5;

    fn encode(self) -> u16 {
        self.face.min(2) as u16 | ((self.facing.min(3) as u16) << 2) | ((self.powered as u16) << 4)
    }

    fn decode(raw: u16) -> Self {
        Self {
            face: (raw & 3) as u8,
            facing: ((raw >> 2) & 3) as u8,
            powered: ((raw >> 4) & 1) == 1,
        }
    }
}

/// State shared by 1 block of Minecraft 26.2 (`test_block`).
///
/// Mirrors the vanilla block state properties and stores each value as its
/// 0-based index into the vanilla value list (2 bits in total).
///
/// - `mode`: 0=start 1=log 2=fail 3=accept
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct TestState {
    /// `mode`.
    pub mode: u8,
}

impl StateCodec for TestState {
    const BITS: u8 = 2;

    fn encode(self) -> u16 {
        self.mode.min(3) as u16
    }

    fn decode(raw: u16) -> Self {
        Self {
            mode: (raw & 3) as u8,
        }
    }
}

/// State shared by 1 block of Minecraft 26.2 (`tnt`).
///
/// Mirrors the vanilla block state properties and stores each value as its
/// 0-based index into the vanilla value list (1 bits in total).
///
/// - `unstable`: bool
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct TntState {
    /// `unstable`.
    pub unstable: bool,
}

impl StateCodec for TntState {
    const BITS: u8 = 1;

    fn encode(self) -> u16 {
        self.unstable as u16
    }

    fn decode(raw: u16) -> Self {
        Self {
            unstable: (raw & 1) == 1,
        }
    }
}

/// State shared by 1 block of Minecraft 26.2 (`torchflower_crop`).
///
/// Mirrors the vanilla block state properties and stores each value as its
/// 0-based index into the vanilla value list (1 bits in total).
///
/// - `age`: 0=0 1=1
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct TorchflowerCropState {
    /// `age`.
    pub age: u8,
}

impl StateCodec for TorchflowerCropState {
    const BITS: u8 = 1;

    fn encode(self) -> u16 {
        self.age.min(1) as u16
    }

    fn decode(raw: u16) -> Self {
        Self {
            age: (raw & 1) as u8,
        }
    }
}

/// State shared by 21 blocks of Minecraft 26.2 (e.g. `acacia_trapdoor`, `bamboo_trapdoor`, `birch_trapdoor`, ...).
///
/// Mirrors the vanilla block state properties and stores each value as its
/// 0-based index into the vanilla value list (6 bits in total).
///
/// - `facing`: 0=north 1=south 2=west 3=east
/// - `half`: 0=top 1=bottom
/// - `open`: bool
/// - `powered`: bool
/// - `waterlogged`: bool
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TrapdoorState {
    /// `facing`.
    pub facing: u8,
    /// `half`.
    pub half: u8,
    /// `open`.
    pub open: bool,
    /// `powered`.
    pub powered: bool,
    /// `waterlogged`.
    pub waterlogged: bool,
}

impl Default for TrapdoorState {
    fn default() -> Self {
        Self {
            facing: 0,
            half: 1,
            open: false,
            powered: false,
            waterlogged: false,
        }
    }
}

impl StateCodec for TrapdoorState {
    const BITS: u8 = 6;

    fn encode(self) -> u16 {
        self.facing.min(3) as u16
            | ((self.half.min(1) as u16) << 2)
            | ((self.open as u16) << 3)
            | ((self.powered as u16) << 4)
            | ((self.waterlogged as u16) << 5)
    }

    fn decode(raw: u16) -> Self {
        Self {
            facing: (raw & 3) as u8,
            half: ((raw >> 2) & 1) as u8,
            open: ((raw >> 3) & 1) == 1,
            powered: ((raw >> 4) & 1) == 1,
            waterlogged: ((raw >> 5) & 1) == 1,
        }
    }
}

/// State shared by 1 block of Minecraft 26.2 (`trial_spawner`).
///
/// Mirrors the vanilla block state properties and stores each value as its
/// 0-based index into the vanilla value list (4 bits in total).
///
/// - `ominous`: bool
/// - `trial_spawner_state`: 0=inactive 1=waiting_for_players 2=active 3=waiting_for_reward_ejection 4=ejecting_reward 5=cooldown
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct TrialSpawnerState {
    /// `ominous`.
    pub ominous: bool,
    /// `trial_spawner_state`.
    pub trial_spawner_state: u8,
}

impl StateCodec for TrialSpawnerState {
    const BITS: u8 = 4;

    fn encode(self) -> u16 {
        self.ominous as u16 | ((self.trial_spawner_state.min(5) as u16) << 1)
    }

    fn decode(raw: u16) -> Self {
        Self {
            ominous: (raw & 1) == 1,
            trial_spawner_state: ((raw >> 1) & 7) as u8,
        }
    }
}

/// State shared by 1 block of Minecraft 26.2 (`tripwire_hook`).
///
/// Mirrors the vanilla block state properties and stores each value as its
/// 0-based index into the vanilla value list (4 bits in total).
///
/// - `attached`: bool
/// - `facing`: 0=north 1=south 2=west 3=east
/// - `powered`: bool
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct TripWireHookState {
    /// `attached`.
    pub attached: bool,
    /// `facing`.
    pub facing: u8,
    /// `powered`.
    pub powered: bool,
}

impl StateCodec for TripWireHookState {
    const BITS: u8 = 4;

    fn encode(self) -> u16 {
        self.attached as u16 | ((self.facing.min(3) as u16) << 1) | ((self.powered as u16) << 3)
    }

    fn decode(raw: u16) -> Self {
        Self {
            attached: (raw & 1) == 1,
            facing: ((raw >> 1) & 3) as u8,
            powered: ((raw >> 3) & 1) == 1,
        }
    }
}

/// State shared by 1 block of Minecraft 26.2 (`tripwire`).
///
/// Mirrors the vanilla block state properties and stores each value as its
/// 0-based index into the vanilla value list (7 bits in total).
///
/// - `attached`: bool
/// - `disarmed`: bool
/// - `east`: bool
/// - `north`: bool
/// - `powered`: bool
/// - `south`: bool
/// - `west`: bool
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct TripwireState {
    /// `attached`.
    pub attached: bool,
    /// `disarmed`.
    pub disarmed: bool,
    /// `east`.
    pub east: bool,
    /// `north`.
    pub north: bool,
    /// `powered`.
    pub powered: bool,
    /// `south`.
    pub south: bool,
    /// `west`.
    pub west: bool,
}

impl StateCodec for TripwireState {
    const BITS: u8 = 7;

    fn encode(self) -> u16 {
        self.attached as u16
            | ((self.disarmed as u16) << 1)
            | ((self.east as u16) << 2)
            | ((self.north as u16) << 3)
            | ((self.powered as u16) << 4)
            | ((self.south as u16) << 5)
            | ((self.west as u16) << 6)
    }

    fn decode(raw: u16) -> Self {
        Self {
            attached: (raw & 1) == 1,
            disarmed: ((raw >> 1) & 1) == 1,
            east: ((raw >> 2) & 1) == 1,
            north: ((raw >> 3) & 1) == 1,
            powered: ((raw >> 4) & 1) == 1,
            south: ((raw >> 5) & 1) == 1,
            west: ((raw >> 6) & 1) == 1,
        }
    }
}

/// State shared by 1 block of Minecraft 26.2 (`turtle_egg`).
///
/// Mirrors the vanilla block state properties and stores each value as its
/// 0-based index into the vanilla value list (4 bits in total).
///
/// - `eggs`: 0=1 1=2 2=3 3=4
/// - `hatch`: 0=0 1=1 2=2
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct TurtleEggState {
    /// `eggs`.
    pub eggs: u8,
    /// `hatch`.
    pub hatch: u8,
}

impl StateCodec for TurtleEggState {
    const BITS: u8 = 4;

    fn encode(self) -> u16 {
        self.eggs.min(3) as u16 | ((self.hatch.min(2) as u16) << 2)
    }

    fn decode(raw: u16) -> Self {
        Self {
            eggs: (raw & 3) as u8,
            hatch: ((raw >> 2) & 3) as u8,
        }
    }
}

/// State shared by 1 block of Minecraft 26.2 (`vault`).
///
/// Mirrors the vanilla block state properties and stores each value as its
/// 0-based index into the vanilla value list (5 bits in total).
///
/// - `facing`: 0=north 1=south 2=west 3=east
/// - `ominous`: bool
/// - `vault_state`: 0=inactive 1=active 2=unlocking 3=ejecting
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct VaultState {
    /// `facing`.
    pub facing: u8,
    /// `ominous`.
    pub ominous: bool,
    /// `vault_state`.
    pub vault_state: u8,
}

impl StateCodec for VaultState {
    const BITS: u8 = 5;

    fn encode(self) -> u16 {
        self.facing.min(3) as u16
            | ((self.ominous as u16) << 2)
            | ((self.vault_state.min(3) as u16) << 3)
    }

    fn decode(raw: u16) -> Self {
        Self {
            facing: (raw & 3) as u8,
            ominous: ((raw >> 2) & 1) == 1,
            vault_state: ((raw >> 3) & 3) as u8,
        }
    }
}

/// State shared by 1 block of Minecraft 26.2 (`vine`).
///
/// Mirrors the vanilla block state properties and stores each value as its
/// 0-based index into the vanilla value list (5 bits in total).
///
/// - `east`: bool
/// - `north`: bool
/// - `south`: bool
/// - `up`: bool
/// - `west`: bool
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct VineState {
    /// `east`.
    pub east: bool,
    /// `north`.
    pub north: bool,
    /// `south`.
    pub south: bool,
    /// `up`.
    pub up: bool,
    /// `west`.
    pub west: bool,
}

impl StateCodec for VineState {
    const BITS: u8 = 5;

    fn encode(self) -> u16 {
        self.east as u16
            | ((self.north as u16) << 1)
            | ((self.south as u16) << 2)
            | ((self.up as u16) << 3)
            | ((self.west as u16) << 4)
    }

    fn decode(raw: u16) -> Self {
        Self {
            east: (raw & 1) == 1,
            north: ((raw >> 1) & 1) == 1,
            south: ((raw >> 2) & 1) == 1,
            up: ((raw >> 3) & 1) == 1,
            west: ((raw >> 4) & 1) == 1,
        }
    }
}

/// State shared by 37 blocks of Minecraft 26.2 (e.g. `acacia_wall_hanging_sign`, `acacia_wall_sign`, `bamboo_wall_hanging_sign`, ...).
///
/// Mirrors the vanilla block state properties and stores each value as its
/// 0-based index into the vanilla value list (3 bits in total).
///
/// - `facing`: 0=north 1=south 2=west 3=east
/// - `waterlogged`: bool
///
/// The blocks sharing this state use different vanilla defaults, so
/// [`Default`] is all zeroes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct WallMountedState {
    /// `facing`.
    pub facing: u8,
    /// `waterlogged`.
    pub waterlogged: bool,
}

impl StateCodec for WallMountedState {
    const BITS: u8 = 3;

    fn encode(self) -> u16 {
        self.facing.min(3) as u16 | ((self.waterlogged as u16) << 2)
    }

    fn decode(raw: u16) -> Self {
        Self {
            facing: (raw & 3) as u8,
            waterlogged: ((raw >> 2) & 1) == 1,
        }
    }
}

/// State shared by 7 blocks of Minecraft 26.2 (e.g. `creeper_wall_head`, `dragon_wall_head`, `piglin_wall_head`, ...).
///
/// Mirrors the vanilla block state properties and stores each value as its
/// 0-based index into the vanilla value list (3 bits in total).
///
/// - `facing`: 0=north 1=south 2=west 3=east
/// - `powered`: bool
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct WallSkullState {
    /// `facing`.
    pub facing: u8,
    /// `powered`.
    pub powered: bool,
}

impl StateCodec for WallSkullState {
    const BITS: u8 = 3;

    fn encode(self) -> u16 {
        self.facing.min(3) as u16 | ((self.powered as u16) << 2)
    }

    fn decode(raw: u16) -> Self {
        Self {
            facing: (raw & 3) as u8,
            powered: ((raw >> 2) & 1) == 1,
        }
    }
}

/// State shared by 32 blocks of Minecraft 26.2 (e.g. `andesite_wall`, `blackstone_wall`, `brick_wall`, ...).
///
/// Mirrors the vanilla block state properties and stores each value as its
/// 0-based index into the vanilla value list (10 bits in total).
///
/// - `east`: 0=none 1=low 2=tall
/// - `north`: 0=none 1=low 2=tall
/// - `south`: 0=none 1=low 2=tall
/// - `up`: bool
/// - `waterlogged`: bool
/// - `west`: 0=none 1=low 2=tall
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct WallState {
    /// `east`.
    pub east: u8,
    /// `north`.
    pub north: u8,
    /// `south`.
    pub south: u8,
    /// `up`.
    pub up: bool,
    /// `waterlogged`.
    pub waterlogged: bool,
    /// `west`.
    pub west: u8,
}

impl Default for WallState {
    fn default() -> Self {
        Self {
            east: 0,
            north: 0,
            south: 0,
            up: true,
            waterlogged: false,
            west: 0,
        }
    }
}

impl StateCodec for WallState {
    const BITS: u8 = 10;

    fn encode(self) -> u16 {
        self.east.min(2) as u16
            | ((self.north.min(2) as u16) << 2)
            | ((self.south.min(2) as u16) << 4)
            | ((self.up as u16) << 6)
            | ((self.waterlogged as u16) << 7)
            | ((self.west.min(2) as u16) << 8)
    }

    fn decode(raw: u16) -> Self {
        Self {
            east: (raw & 3) as u8,
            north: ((raw >> 2) & 3) as u8,
            south: ((raw >> 4) & 3) as u8,
            up: ((raw >> 6) & 1) == 1,
            waterlogged: ((raw >> 7) & 1) == 1,
            west: ((raw >> 8) & 3) as u8,
        }
    }
}

/// State shared by 33 blocks of Minecraft 26.2 (e.g. `barrier`, `brain_coral`, `brain_coral_fan`, ...).
///
/// Mirrors the vanilla block state properties and stores each value as its
/// 0-based index into the vanilla value list (1 bits in total).
///
/// - `waterlogged`: bool
///
/// The blocks sharing this state use different vanilla defaults, so
/// [`Default`] is all zeroes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct WaterloggedState {
    /// `waterlogged`.
    pub waterlogged: bool,
}

impl StateCodec for WaterloggedState {
    const BITS: u8 = 1;

    fn encode(self) -> u16 {
        self.waterlogged as u16
    }

    fn decode(raw: u16) -> Self {
        Self {
            waterlogged: (raw & 1) == 1,
        }
    }
}

/// State shared by 8 blocks of Minecraft 26.2 (e.g. `copper_bulb`, `exposed_copper_bulb`, `oxidized_copper_bulb`, ...).
///
/// Mirrors the vanilla block state properties and stores each value as its
/// 0-based index into the vanilla value list (2 bits in total).
///
/// - `lit`: bool
/// - `powered`: bool
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct WeatheringCopperBulbState {
    /// `lit`.
    pub lit: bool,
    /// `powered`.
    pub powered: bool,
}

impl StateCodec for WeatheringCopperBulbState {
    const BITS: u8 = 2;

    fn encode(self) -> u16 {
        self.lit as u16 | ((self.powered as u16) << 1)
    }

    fn decode(raw: u16) -> Self {
        Self {
            lit: (raw & 1) == 1,
            powered: ((raw >> 1) & 1) == 1,
        }
    }
}

/// State shared by 9 blocks of Minecraft 26.2 (e.g. `copper_chain`, `exposed_copper_chain`, `iron_chain`, ...).
///
/// Mirrors the vanilla block state properties and stores each value as its
/// 0-based index into the vanilla value list (3 bits in total).
///
/// - `axis`: 0=x 1=y 2=z
/// - `waterlogged`: bool
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct WeatheringCopperChainState {
    /// `axis`.
    pub axis: u8,
    /// `waterlogged`.
    pub waterlogged: bool,
}

impl Default for WeatheringCopperChainState {
    fn default() -> Self {
        Self {
            axis: 1,
            waterlogged: false,
        }
    }
}

impl StateCodec for WeatheringCopperChainState {
    const BITS: u8 = 3;

    fn encode(self) -> u16 {
        self.axis.min(2) as u16 | ((self.waterlogged as u16) << 2)
    }

    fn decode(raw: u16) -> Self {
        Self {
            axis: (raw & 3) as u8,
            waterlogged: ((raw >> 2) & 1) == 1,
        }
    }
}

/// State shared by 8 blocks of Minecraft 26.2 (e.g. `copper_golem_statue`, `exposed_copper_golem_statue`, `oxidized_copper_golem_statue`, ...).
///
/// Mirrors the vanilla block state properties and stores each value as its
/// 0-based index into the vanilla value list (5 bits in total).
///
/// - `copper_golem_pose`: 0=standing 1=sitting 2=running 3=star
/// - `facing`: 0=north 1=south 2=west 3=east
/// - `waterlogged`: bool
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct WeatheringCopperGolemStatueState {
    /// `copper_golem_pose`.
    pub copper_golem_pose: u8,
    /// `facing`.
    pub facing: u8,
    /// `waterlogged`.
    pub waterlogged: bool,
}

impl StateCodec for WeatheringCopperGolemStatueState {
    const BITS: u8 = 5;

    fn encode(self) -> u16 {
        self.copper_golem_pose.min(3) as u16
            | ((self.facing.min(3) as u16) << 2)
            | ((self.waterlogged as u16) << 4)
    }

    fn decode(raw: u16) -> Self {
        Self {
            copper_golem_pose: (raw & 3) as u8,
            facing: ((raw >> 2) & 3) as u8,
            waterlogged: ((raw >> 4) & 1) == 1,
        }
    }
}

/// State shared by 10 blocks of Minecraft 26.2 (e.g. `copper_lantern`, `exposed_copper_lantern`, `lantern`, ...).
///
/// Mirrors the vanilla block state properties and stores each value as its
/// 0-based index into the vanilla value list (2 bits in total).
///
/// - `hanging`: bool
/// - `waterlogged`: bool
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct WeatheringLanternState {
    /// `hanging`.
    pub hanging: bool,
    /// `waterlogged`.
    pub waterlogged: bool,
}

impl StateCodec for WeatheringLanternState {
    const BITS: u8 = 2;

    fn encode(self) -> u16 {
        self.hanging as u16 | ((self.waterlogged as u16) << 1)
    }

    fn decode(raw: u16) -> Self {
        Self {
            hanging: (raw & 1) == 1,
            waterlogged: ((raw >> 1) & 1) == 1,
        }
    }
}

/// State shared by 8 blocks of Minecraft 26.2 (e.g. `exposed_lightning_rod`, `lightning_rod`, `oxidized_lightning_rod`, ...).
///
/// Mirrors the vanilla block state properties and stores each value as its
/// 0-based index into the vanilla value list (5 bits in total).
///
/// - `facing`: 0=north 1=east 2=south 3=west 4=up 5=down
/// - `powered`: bool
/// - `waterlogged`: bool
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct WeatheringLightningRodState {
    /// `facing`.
    pub facing: u8,
    /// `powered`.
    pub powered: bool,
    /// `waterlogged`.
    pub waterlogged: bool,
}

impl Default for WeatheringLightningRodState {
    fn default() -> Self {
        Self {
            facing: 4,
            powered: false,
            waterlogged: false,
        }
    }
}

impl StateCodec for WeatheringLightningRodState {
    const BITS: u8 = 5;

    fn encode(self) -> u16 {
        self.facing.min(5) as u16 | ((self.powered as u16) << 3) | ((self.waterlogged as u16) << 4)
    }

    fn decode(raw: u16) -> Self {
        Self {
            facing: (raw & 7) as u8,
            powered: ((raw >> 3) & 1) == 1,
            waterlogged: ((raw >> 4) & 1) == 1,
        }
    }
}

/// State shared by 3 blocks of Minecraft 26.2 (`heavy_weighted_pressure_plate`) (`light_weighted_pressure_plate`) (`target`).
///
/// Mirrors the vanilla block state properties and stores each value as its
/// 0-based index into the vanilla value list (4 bits in total).
///
/// - `power`: 0=0 1=1 2=2 3=3 4=4 5=5 6=6 7=7 8=8 9=9 10=10 11=11 12=12 13=13 14=14 15=15
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct WeightedPressurePlateState {
    /// `power`.
    pub power: u8,
}

impl StateCodec for WeightedPressurePlateState {
    const BITS: u8 = 4;

    fn encode(self) -> u16 {
        self.power.min(15) as u16
    }

    fn decode(raw: u16) -> Self {
        Self {
            power: (raw & 15) as u8,
        }
    }
}

// Block definitions, one nominal type per block.

block! {
    /// `acacia_button` — Acacia Button. `Breakable | Flammable`
    pub struct AcaciaButton;
    facets: Breakable, Flammable;
    state: SwitchState;
}

block! {
    /// `acacia_door` — Acacia Door. `Breakable | Flammable`
    pub struct AcaciaDoor;
    facets: Breakable, Flammable;
    state: DoorState;
}

block! {
    /// `acacia_fence` — Acacia Fence. `Movable | Breakable | Flammable`
    pub struct AcaciaFence;
    facets: Movable, Breakable, Flammable;
    state: ConnectionState;
}

block! {
    /// `acacia_fence_gate` — Acacia Fence Gate. `Movable | Breakable | Flammable`
    pub struct AcaciaFenceGate;
    facets: Movable, Breakable, Flammable;
    state: FenceGateState;
}

block! {
    /// `acacia_hanging_sign` — Acacia Hanging Sign. `Breakable | Flammable`
    pub struct AcaciaHangingSign;
    facets: Breakable, Flammable;
    state: HangingSignState;
}

block! {
    /// `acacia_leaves` — Acacia Leaves. `Movable | Breakable | Flammable`
    pub struct AcaciaLeaves;
    facets: Movable, Breakable, Flammable;
    state: LeavesState;
}

block! {
    /// `acacia_log` — Acacia Log. `Movable | Breakable | Flammable`
    pub struct AcaciaLog;
    facets: Movable, Breakable, Flammable;
    state: PillarState;
}

block! {
    /// `acacia_planks` — Acacia Planks. `Movable | Breakable | Flammable`
    pub struct AcaciaPlanks;
    facets: Movable, Breakable, Flammable;
}

block! {
    /// `acacia_pressure_plate` — Acacia Pressure Plate. `Breakable | Flammable`
    pub struct AcaciaPressurePlate;
    facets: Breakable, Flammable;
    state: PressurePlateState;
}

block! {
    /// `acacia_sapling` — Acacia Sapling. `InstantMinable`
    pub struct AcaciaSapling;
    facets: InstantMinable;
    state: SaplingState;
}

block! {
    /// `acacia_shelf` — Acacia Shelf. `UnMovable | Breakable`
    pub struct AcaciaShelf;
    facets: UnMovable, Breakable;
    state: ShelfState;
}

block! {
    /// `acacia_sign` — Acacia Sign. `Breakable | Flammable`
    pub struct AcaciaSign;
    facets: Breakable, Flammable;
    state: StandingSignState;
}

block! {
    /// `acacia_slab` — Acacia Slab. `Movable | Breakable | Flammable`
    pub struct AcaciaSlab;
    facets: Movable, Breakable, Flammable;
    state: SlabState;
}

block! {
    /// `acacia_stairs` — Acacia Stairs. `Movable | Breakable | Flammable`
    pub struct AcaciaStairs;
    facets: Movable, Breakable, Flammable;
    state: StairState;
}

block! {
    /// `acacia_trapdoor` — Acacia Trapdoor. `Movable | Breakable | Flammable`
    pub struct AcaciaTrapdoor;
    facets: Movable, Breakable, Flammable;
    state: TrapdoorState;
}

block! {
    /// `acacia_wall_hanging_sign` — Acacia Hanging Sign. `Breakable | Flammable`
    pub struct AcaciaWallHangingSign;
    facets: Breakable, Flammable;
    state: WallMountedState;
}

block! {
    /// `acacia_wall_sign` — Acacia Sign. `Breakable | Flammable`
    pub struct AcaciaWallSign;
    facets: Breakable, Flammable;
    state: WallMountedState;
}

block! {
    /// `acacia_wood` — Acacia Wood. `Movable | Breakable | Flammable`
    pub struct AcaciaWood;
    facets: Movable, Breakable, Flammable;
    state: PillarState;
}

block! {
    /// `activator_rail` — Activator Rail. `Movable | Breakable`
    pub struct ActivatorRail;
    facets: Movable, Breakable;
    state: PoweredRailState;
}

block! {
    /// `air` — Air. `Empty`
    pub struct Air;
    facets: Empty;
}

block! {
    /// `allium` — Allium. `InstantMinable | Flammable`
    pub struct Allium;
    facets: InstantMinable, Flammable;
}

block! {
    /// `amethyst_block` — Block of Amethyst. `Breakable`
    pub struct AmethystBlock;
    facets: Breakable;
}

block! {
    /// `amethyst_cluster` — Amethyst Cluster. `Breakable`
    pub struct AmethystCluster;
    facets: Breakable;
    state: AmethystClusterState;
}

block! {
    /// `ancient_debris` — Ancient Debris. `Movable | Breakable`
    pub struct AncientDebris;
    facets: Movable, Breakable;
}

block! {
    /// `andesite` — Andesite. `Movable | Breakable`
    pub struct Andesite;
    facets: Movable, Breakable;
}

block! {
    /// `andesite_slab` — Andesite Slab. `Movable | Breakable`
    pub struct AndesiteSlab;
    facets: Movable, Breakable;
    state: SlabState;
}

block! {
    /// `andesite_stairs` — Andesite Stairs. `Movable | Breakable`
    pub struct AndesiteStairs;
    facets: Movable, Breakable;
    state: StairState;
}

block! {
    /// `andesite_wall` — Andesite Wall. `Movable | Breakable`
    pub struct AndesiteWall;
    facets: Movable, Breakable;
    state: WallState;
}

block! {
    /// `anvil` — Anvil. `Movable | Breakable`
    pub struct Anvil;
    facets: Movable, Breakable;
    state: FacingState;
}

block! {
    /// `attached_melon_stem` — Attached Melon Stem. `InstantMinable`
    pub struct AttachedMelonStem;
    facets: InstantMinable;
    state: FacingState;
}

block! {
    /// `attached_pumpkin_stem` — Attached Pumpkin Stem. `InstantMinable`
    pub struct AttachedPumpkinStem;
    facets: InstantMinable;
    state: FacingState;
}

block! {
    /// `azalea` — Azalea. `Movable | InstantMinable | Flammable`
    pub struct Azalea;
    facets: Movable, InstantMinable, Flammable;
}

block! {
    /// `azalea_leaves` — Azalea Leaves. `Movable | Breakable | Flammable`
    pub struct AzaleaLeaves;
    facets: Movable, Breakable, Flammable;
    state: LeavesState;
}

block! {
    /// `azure_bluet` — Azure Bluet. `InstantMinable | Flammable`
    pub struct AzureBluet;
    facets: InstantMinable, Flammable;
}

block! {
    /// `bamboo` — Bamboo. `Breakable | Flammable`
    pub struct Bamboo;
    facets: Breakable, Flammable;
    state: BambooStalkState;
}

block! {
    /// `bamboo_block` — Block of Bamboo. `Movable | Breakable | Flammable`
    pub struct BambooBlock;
    facets: Movable, Breakable, Flammable;
    state: PillarState;
}

block! {
    /// `bamboo_button` — Bamboo Button. `Breakable | Flammable`
    pub struct BambooButton;
    facets: Breakable, Flammable;
    state: SwitchState;
}

block! {
    /// `bamboo_door` — Bamboo Door. `Breakable | Flammable`
    pub struct BambooDoor;
    facets: Breakable, Flammable;
    state: DoorState;
}

block! {
    /// `bamboo_fence` — Bamboo Fence. `Movable | Breakable | Flammable`
    pub struct BambooFence;
    facets: Movable, Breakable, Flammable;
    state: ConnectionState;
}

block! {
    /// `bamboo_fence_gate` — Bamboo Fence Gate. `Movable | Breakable | Flammable`
    pub struct BambooFenceGate;
    facets: Movable, Breakable, Flammable;
    state: FenceGateState;
}

block! {
    /// `bamboo_hanging_sign` — Bamboo Hanging Sign. `Breakable | Flammable`
    pub struct BambooHangingSign;
    facets: Breakable, Flammable;
    state: HangingSignState;
}

block! {
    /// `bamboo_mosaic` — Bamboo Mosaic. `Movable | Breakable | Flammable`
    pub struct BambooMosaic;
    facets: Movable, Breakable, Flammable;
}

block! {
    /// `bamboo_mosaic_slab` — Bamboo Mosaic Slab. `Movable | Breakable | Flammable`
    pub struct BambooMosaicSlab;
    facets: Movable, Breakable, Flammable;
    state: SlabState;
}

block! {
    /// `bamboo_mosaic_stairs` — Bamboo Mosaic Stairs. `Movable | Breakable | Flammable`
    pub struct BambooMosaicStairs;
    facets: Movable, Breakable, Flammable;
    state: StairState;
}

block! {
    /// `bamboo_planks` — Bamboo Planks. `Movable | Breakable | Flammable`
    pub struct BambooPlanks;
    facets: Movable, Breakable, Flammable;
}

block! {
    /// `bamboo_pressure_plate` — Bamboo Pressure Plate. `Breakable | Flammable`
    pub struct BambooPressurePlate;
    facets: Breakable, Flammable;
    state: PressurePlateState;
}

block! {
    /// `bamboo_sapling` — Bamboo Shoot. `Breakable | Flammable`
    pub struct BambooSapling;
    facets: Breakable, Flammable;
}

block! {
    /// `bamboo_shelf` — Bamboo Shelf. `UnMovable | Breakable`
    pub struct BambooShelf;
    facets: UnMovable, Breakable;
    state: ShelfState;
}

block! {
    /// `bamboo_sign` — Bamboo Sign. `Breakable | Flammable`
    pub struct BambooSign;
    facets: Breakable, Flammable;
    state: StandingSignState;
}

block! {
    /// `bamboo_slab` — Bamboo Slab. `Movable | Breakable | Flammable`
    pub struct BambooSlab;
    facets: Movable, Breakable, Flammable;
    state: SlabState;
}

block! {
    /// `bamboo_stairs` — Bamboo Stairs. `Movable | Breakable | Flammable`
    pub struct BambooStairs;
    facets: Movable, Breakable, Flammable;
    state: StairState;
}

block! {
    /// `bamboo_trapdoor` — Bamboo Trapdoor. `Movable | Breakable | Flammable`
    pub struct BambooTrapdoor;
    facets: Movable, Breakable, Flammable;
    state: TrapdoorState;
}

block! {
    /// `bamboo_wall_hanging_sign` — Bamboo Hanging Sign. `Breakable | Flammable`
    pub struct BambooWallHangingSign;
    facets: Breakable, Flammable;
    state: WallMountedState;
}

block! {
    /// `bamboo_wall_sign` — Bamboo Sign. `Breakable | Flammable`
    pub struct BambooWallSign;
    facets: Breakable, Flammable;
    state: WallMountedState;
}

block! {
    /// `barrel` — Barrel. `UnMovable | Breakable`
    pub struct Barrel;
    facets: UnMovable, Breakable;
    state: BarrelState;
}

block! {
    /// `barrier` — Barrier. `UnMovable`
    pub struct Barrier;
    facets: UnMovable;
    state: WaterloggedState;
}

block! {
    /// `basalt` — Basalt. `Movable | Breakable`
    pub struct Basalt;
    facets: Movable, Breakable;
    state: PillarState;
}

block! {
    /// `beacon` — Beacon. `UnMovable | Breakable`
    pub struct Beacon;
    facets: UnMovable, Breakable;
}

block! {
    /// `bedrock` — Bedrock. `UnMovable`
    pub struct Bedrock;
    facets: UnMovable;
}

block! {
    /// `bee_nest` — Bee Nest. `UnMovable | Breakable | Flammable`
    pub struct BeeNest;
    facets: UnMovable, Breakable, Flammable;
    state: BeehiveState;
}

block! {
    /// `beehive` — Beehive. `UnMovable | Breakable | Flammable`
    pub struct Beehive;
    facets: UnMovable, Breakable, Flammable;
    state: BeehiveState;
}

block! {
    /// `beetroots` — Beetroots. `InstantMinable`
    pub struct Beetroots;
    facets: InstantMinable;
    state: BeetrootState;
}

block! {
    /// `bell` — Bell. `Breakable`
    pub struct Bell;
    facets: Breakable;
    state: BellState;
}

block! {
    /// `big_dripleaf` — Big Dripleaf. `Movable | Breakable | Flammable`
    pub struct BigDripleaf;
    facets: Movable, Breakable, Flammable;
    state: BigDripleafState;
}

block! {
    /// `big_dripleaf_stem` — Big Dripleaf Stem. `Movable | Breakable | Flammable`
    pub struct BigDripleafStem;
    facets: Movable, Breakable, Flammable;
    state: WallMountedState;
}

block! {
    /// `birch_button` — Birch Button. `Breakable | Flammable`
    pub struct BirchButton;
    facets: Breakable, Flammable;
    state: SwitchState;
}

block! {
    /// `birch_door` — Birch Door. `Breakable | Flammable`
    pub struct BirchDoor;
    facets: Breakable, Flammable;
    state: DoorState;
}

block! {
    /// `birch_fence` — Birch Fence. `Movable | Breakable | Flammable`
    pub struct BirchFence;
    facets: Movable, Breakable, Flammable;
    state: ConnectionState;
}

block! {
    /// `birch_fence_gate` — Birch Fence Gate. `Movable | Breakable | Flammable`
    pub struct BirchFenceGate;
    facets: Movable, Breakable, Flammable;
    state: FenceGateState;
}

block! {
    /// `birch_hanging_sign` — Birch Hanging Sign. `Breakable | Flammable`
    pub struct BirchHangingSign;
    facets: Breakable, Flammable;
    state: HangingSignState;
}

block! {
    /// `birch_leaves` — Birch Leaves. `Movable | Breakable | Flammable`
    pub struct BirchLeaves;
    facets: Movable, Breakable, Flammable;
    state: LeavesState;
}

block! {
    /// `birch_log` — Birch Log. `Movable | Breakable | Flammable`
    pub struct BirchLog;
    facets: Movable, Breakable, Flammable;
    state: PillarState;
}

block! {
    /// `birch_planks` — Birch Planks. `Movable | Breakable | Flammable`
    pub struct BirchPlanks;
    facets: Movable, Breakable, Flammable;
}

block! {
    /// `birch_pressure_plate` — Birch Pressure Plate. `Breakable | Flammable`
    pub struct BirchPressurePlate;
    facets: Breakable, Flammable;
    state: PressurePlateState;
}

block! {
    /// `birch_sapling` — Birch Sapling. `InstantMinable`
    pub struct BirchSapling;
    facets: InstantMinable;
    state: SaplingState;
}

block! {
    /// `birch_shelf` — Birch Shelf. `UnMovable | Breakable`
    pub struct BirchShelf;
    facets: UnMovable, Breakable;
    state: ShelfState;
}

block! {
    /// `birch_sign` — Birch Sign. `Breakable | Flammable`
    pub struct BirchSign;
    facets: Breakable, Flammable;
    state: StandingSignState;
}

block! {
    /// `birch_slab` — Birch Slab. `Movable | Breakable | Flammable`
    pub struct BirchSlab;
    facets: Movable, Breakable, Flammable;
    state: SlabState;
}

block! {
    /// `birch_stairs` — Birch Stairs. `Movable | Breakable | Flammable`
    pub struct BirchStairs;
    facets: Movable, Breakable, Flammable;
    state: StairState;
}

block! {
    /// `birch_trapdoor` — Birch Trapdoor. `Movable | Breakable | Flammable`
    pub struct BirchTrapdoor;
    facets: Movable, Breakable, Flammable;
    state: TrapdoorState;
}

block! {
    /// `birch_wall_hanging_sign` — Birch Hanging Sign. `Breakable | Flammable`
    pub struct BirchWallHangingSign;
    facets: Breakable, Flammable;
    state: WallMountedState;
}

block! {
    /// `birch_wall_sign` — Birch Sign. `Breakable | Flammable`
    pub struct BirchWallSign;
    facets: Breakable, Flammable;
    state: WallMountedState;
}

block! {
    /// `birch_wood` — Birch Wood. `Movable | Breakable | Flammable`
    pub struct BirchWood;
    facets: Movable, Breakable, Flammable;
    state: PillarState;
}

block! {
    /// `black_banner` — Black Banner. `Breakable`
    pub struct BlackBanner;
    facets: Breakable;
    state: BannerState;
}

block! {
    /// `black_bed` — Black Bed. `Breakable`
    pub struct BlackBed;
    facets: Breakable;
    state: BedState;
}

block! {
    /// `black_candle` — Black Candle. `Movable | Breakable`
    pub struct BlackCandle;
    facets: Movable, Breakable;
    state: CandleState;
}

block! {
    /// `black_candle_cake` — Cake with Black Candle. `Movable | Breakable`
    pub struct BlackCandleCake;
    facets: Movable, Breakable;
    state: LitState;
}

block! {
    /// `black_carpet` — Black Carpet. `Movable | Breakable | Flammable`
    pub struct BlackCarpet;
    facets: Movable, Breakable, Flammable;
}

block! {
    /// `black_concrete` — Black Concrete. `Movable | Breakable`
    pub struct BlackConcrete;
    facets: Movable, Breakable;
}

block! {
    /// `black_concrete_powder` — Black Concrete Powder. `Movable | Breakable`
    pub struct BlackConcretePowder;
    facets: Movable, Breakable;
}

block! {
    /// `black_glazed_terracotta` — Black Glazed Terracotta. `Movable | Breakable`
    pub struct BlackGlazedTerracotta;
    facets: Movable, Breakable;
    state: FacingState;
}

block! {
    /// `black_shulker_box` — Black Shulker Box. `Breakable`
    pub struct BlackShulkerBox;
    facets: Breakable;
    state: ShulkerBoxState;
}

block! {
    /// `black_stained_glass` — Black Stained Glass. `Movable | Breakable`
    pub struct BlackStainedGlass;
    facets: Movable, Breakable;
}

block! {
    /// `black_stained_glass_pane` — Black Stained Glass Pane. `Movable | Breakable`
    pub struct BlackStainedGlassPane;
    facets: Movable, Breakable;
    state: ConnectionState;
}

block! {
    /// `black_terracotta` — Black Terracotta. `Movable | Breakable`
    pub struct BlackTerracotta;
    facets: Movable, Breakable;
}

block! {
    /// `black_wall_banner` — Black Banner. `Breakable`
    pub struct BlackWallBanner;
    facets: Breakable;
    state: FacingState;
}

block! {
    /// `black_wool` — Black Wool. `Movable | Breakable | Flammable`
    pub struct BlackWool;
    facets: Movable, Breakable, Flammable;
}

block! {
    /// `blackstone` — Blackstone. `Movable | Breakable`
    pub struct Blackstone;
    facets: Movable, Breakable;
}

block! {
    /// `blackstone_slab` — Blackstone Slab. `Movable | Breakable`
    pub struct BlackstoneSlab;
    facets: Movable, Breakable;
    state: SlabState;
}

block! {
    /// `blackstone_stairs` — Blackstone Stairs. `Movable | Breakable`
    pub struct BlackstoneStairs;
    facets: Movable, Breakable;
    state: StairState;
}

block! {
    /// `blackstone_wall` — Blackstone Wall. `Movable | Breakable`
    pub struct BlackstoneWall;
    facets: Movable, Breakable;
    state: WallState;
}

block! {
    /// `blast_furnace` — Blast Furnace. `UnMovable | Breakable`
    pub struct BlastFurnace;
    facets: UnMovable, Breakable;
    state: BlastFurnaceState;
}

block! {
    /// `blue_banner` — Blue Banner. `Breakable`
    pub struct BlueBanner;
    facets: Breakable;
    state: BannerState;
}

block! {
    /// `blue_bed` — Blue Bed. `Breakable`
    pub struct BlueBed;
    facets: Breakable;
    state: BedState;
}

block! {
    /// `blue_candle` — Blue Candle. `Movable | Breakable`
    pub struct BlueCandle;
    facets: Movable, Breakable;
    state: CandleState;
}

block! {
    /// `blue_candle_cake` — Cake with Blue Candle. `Movable | Breakable`
    pub struct BlueCandleCake;
    facets: Movable, Breakable;
    state: LitState;
}

block! {
    /// `blue_carpet` — Blue Carpet. `Movable | Breakable | Flammable`
    pub struct BlueCarpet;
    facets: Movable, Breakable, Flammable;
}

block! {
    /// `blue_concrete` — Blue Concrete. `Movable | Breakable`
    pub struct BlueConcrete;
    facets: Movable, Breakable;
}

block! {
    /// `blue_concrete_powder` — Blue Concrete Powder. `Movable | Breakable`
    pub struct BlueConcretePowder;
    facets: Movable, Breakable;
}

block! {
    /// `blue_glazed_terracotta` — Blue Glazed Terracotta. `Movable | Breakable`
    pub struct BlueGlazedTerracotta;
    facets: Movable, Breakable;
    state: FacingState;
}

block! {
    /// `blue_ice` — Blue Ice. `Movable | Breakable`
    pub struct BlueIce;
    facets: Movable, Breakable;
}

block! {
    /// `blue_orchid` — Blue Orchid. `InstantMinable | Flammable`
    pub struct BlueOrchid;
    facets: InstantMinable, Flammable;
}

block! {
    /// `blue_shulker_box` — Blue Shulker Box. `Breakable`
    pub struct BlueShulkerBox;
    facets: Breakable;
    state: ShulkerBoxState;
}

block! {
    /// `blue_stained_glass` — Blue Stained Glass. `Movable | Breakable`
    pub struct BlueStainedGlass;
    facets: Movable, Breakable;
}

block! {
    /// `blue_stained_glass_pane` — Blue Stained Glass Pane. `Movable | Breakable`
    pub struct BlueStainedGlassPane;
    facets: Movable, Breakable;
    state: ConnectionState;
}

block! {
    /// `blue_terracotta` — Blue Terracotta. `Movable | Breakable`
    pub struct BlueTerracotta;
    facets: Movable, Breakable;
}

block! {
    /// `blue_wall_banner` — Blue Banner. `Breakable`
    pub struct BlueWallBanner;
    facets: Breakable;
    state: FacingState;
}

block! {
    /// `blue_wool` — Blue Wool. `Movable | Breakable | Flammable`
    pub struct BlueWool;
    facets: Movable, Breakable, Flammable;
}

block! {
    /// `bone_block` — Bone Block. `Movable | Breakable`
    pub struct BoneBlock;
    facets: Movable, Breakable;
    state: PillarState;
}

block! {
    /// `bookshelf` — Bookshelf. `Movable | Breakable | Flammable`
    pub struct Bookshelf;
    facets: Movable, Breakable, Flammable;
}

block! {
    /// `brain_coral` — Brain Coral. `Movable | InstantMinable`
    pub struct BrainCoral;
    facets: Movable, InstantMinable;
    state: WaterloggedState;
}

block! {
    /// `brain_coral_block` — Brain Coral Block. `Movable | Breakable`
    pub struct BrainCoralBlock;
    facets: Movable, Breakable;
}

block! {
    /// `brain_coral_fan` — Brain Coral Fan. `Movable | InstantMinable`
    pub struct BrainCoralFan;
    facets: Movable, InstantMinable;
    state: WaterloggedState;
}

block! {
    /// `brain_coral_wall_fan` — Brain Coral Wall Fan. `Movable | InstantMinable`
    pub struct BrainCoralWallFan;
    facets: Movable, InstantMinable;
    state: WallMountedState;
}

block! {
    /// `brewing_stand` — Brewing Stand. `UnMovable | Breakable`
    pub struct BrewingStand;
    facets: UnMovable, Breakable;
    state: BrewingStandState;
}

block! {
    /// `brick_slab` — Brick Slab. `Movable | Breakable`
    pub struct BrickSlab;
    facets: Movable, Breakable;
    state: SlabState;
}

block! {
    /// `brick_stairs` — Brick Stairs. `Movable | Breakable`
    pub struct BrickStairs;
    facets: Movable, Breakable;
    state: StairState;
}

block! {
    /// `brick_wall` — Brick Wall. `Movable | Breakable`
    pub struct BrickWall;
    facets: Movable, Breakable;
    state: WallState;
}

block! {
    /// `bricks` — Bricks. `Movable | Breakable`
    pub struct Bricks;
    facets: Movable, Breakable;
}

block! {
    /// `brown_banner` — Brown Banner. `Breakable`
    pub struct BrownBanner;
    facets: Breakable;
    state: BannerState;
}

block! {
    /// `brown_bed` — Brown Bed. `Breakable`
    pub struct BrownBed;
    facets: Breakable;
    state: BedState;
}

block! {
    /// `brown_candle` — Brown Candle. `Movable | Breakable`
    pub struct BrownCandle;
    facets: Movable, Breakable;
    state: CandleState;
}

block! {
    /// `brown_candle_cake` — Cake with Brown Candle. `Movable | Breakable`
    pub struct BrownCandleCake;
    facets: Movable, Breakable;
    state: LitState;
}

block! {
    /// `brown_carpet` — Brown Carpet. `Movable | Breakable | Flammable`
    pub struct BrownCarpet;
    facets: Movable, Breakable, Flammable;
}

block! {
    /// `brown_concrete` — Brown Concrete. `Movable | Breakable`
    pub struct BrownConcrete;
    facets: Movable, Breakable;
}

block! {
    /// `brown_concrete_powder` — Brown Concrete Powder. `Movable | Breakable`
    pub struct BrownConcretePowder;
    facets: Movable, Breakable;
}

block! {
    /// `brown_glazed_terracotta` — Brown Glazed Terracotta. `Movable | Breakable`
    pub struct BrownGlazedTerracotta;
    facets: Movable, Breakable;
    state: FacingState;
}

block! {
    /// `brown_mushroom` — Brown Mushroom. `InstantMinable`
    pub struct BrownMushroom;
    facets: InstantMinable;
}

block! {
    /// `brown_mushroom_block` — Brown Mushroom Block. `Movable | Breakable`
    pub struct BrownMushroomBlock;
    facets: Movable, Breakable;
    state: HugeMushroomState;
}

block! {
    /// `brown_shulker_box` — Brown Shulker Box. `Breakable`
    pub struct BrownShulkerBox;
    facets: Breakable;
    state: ShulkerBoxState;
}

block! {
    /// `brown_stained_glass` — Brown Stained Glass. `Movable | Breakable`
    pub struct BrownStainedGlass;
    facets: Movable, Breakable;
}

block! {
    /// `brown_stained_glass_pane` — Brown Stained Glass Pane. `Movable | Breakable`
    pub struct BrownStainedGlassPane;
    facets: Movable, Breakable;
    state: ConnectionState;
}

block! {
    /// `brown_terracotta` — Brown Terracotta. `Movable | Breakable`
    pub struct BrownTerracotta;
    facets: Movable, Breakable;
}

block! {
    /// `brown_wall_banner` — Brown Banner. `Breakable`
    pub struct BrownWallBanner;
    facets: Breakable;
    state: FacingState;
}

block! {
    /// `brown_wool` — Brown Wool. `Movable | Breakable | Flammable`
    pub struct BrownWool;
    facets: Movable, Breakable, Flammable;
}

block! {
    /// `bubble_column` — Bubble Column. `UnMovable`
    pub struct BubbleColumn;
    facets: UnMovable;
    state: BubbleColumnState;
}

block! {
    /// `bubble_coral` — Bubble Coral. `Movable | InstantMinable`
    pub struct BubbleCoral;
    facets: Movable, InstantMinable;
    state: WaterloggedState;
}

block! {
    /// `bubble_coral_block` — Bubble Coral Block. `Movable | Breakable`
    pub struct BubbleCoralBlock;
    facets: Movable, Breakable;
}

block! {
    /// `bubble_coral_fan` — Bubble Coral Fan. `Movable | InstantMinable`
    pub struct BubbleCoralFan;
    facets: Movable, InstantMinable;
    state: WaterloggedState;
}

block! {
    /// `bubble_coral_wall_fan` — Bubble Coral Wall Fan. `Movable | InstantMinable`
    pub struct BubbleCoralWallFan;
    facets: Movable, InstantMinable;
    state: WallMountedState;
}

block! {
    /// `budding_amethyst` — Budding Amethyst. `Breakable`
    pub struct BuddingAmethyst;
    facets: Breakable;
}

block! {
    /// `bush` — Bush. `InstantMinable | Flammable`
    pub struct Bush;
    facets: InstantMinable, Flammable;
}

block! {
    /// `cactus` — Cactus. `Breakable`
    pub struct Cactus;
    facets: Breakable;
    state: CactusState;
}

block! {
    /// `cactus_flower` — Cactus Flower. `InstantMinable | Flammable`
    pub struct CactusFlower;
    facets: InstantMinable, Flammable;
}

block! {
    /// `cake` — Cake. `Breakable`
    pub struct Cake;
    facets: Breakable;
    state: CakeState;
}

block! {
    /// `calcite` — Calcite. `Movable | Breakable`
    pub struct Calcite;
    facets: Movable, Breakable;
}

block! {
    /// `calibrated_sculk_sensor` — Calibrated Sculk Sensor. `UnMovable | Breakable`
    pub struct CalibratedSculkSensor;
    facets: UnMovable, Breakable;
    state: CalibratedSculkSensorState;
}

block! {
    /// `campfire` — Campfire. `Breakable`
    pub struct Campfire;
    facets: Breakable;
    state: CampfireState;
}

block! {
    /// `candle` — Candle. `Movable | Breakable`
    pub struct Candle;
    facets: Movable, Breakable;
    state: CandleState;
}

block! {
    /// `candle_cake` — Cake with Candle. `Movable | Breakable`
    pub struct CandleCake;
    facets: Movable, Breakable;
    state: LitState;
}

block! {
    /// `carrots` — Carrots. `InstantMinable`
    pub struct Carrots;
    facets: InstantMinable;
    state: CropAgeState;
}

block! {
    /// `cartography_table` — Cartography Table. `Movable | Breakable`
    pub struct CartographyTable;
    facets: Movable, Breakable;
}

block! {
    /// `carved_pumpkin` — Carved Pumpkin. `Breakable`
    pub struct CarvedPumpkin;
    facets: Breakable;
    state: FacingState;
}

block! {
    /// `cauldron` — Cauldron. `Movable | Breakable`
    pub struct Cauldron;
    facets: Movable, Breakable;
}

block! {
    /// `cave_air` — Cave Air. `Empty`
    pub struct CaveAir;
    facets: Empty;
}

block! {
    /// `cave_vines` — Cave Vines. `InstantMinable | Flammable`
    pub struct CaveVines;
    facets: InstantMinable, Flammable;
    state: CaveVinesState;
}

block! {
    /// `cave_vines_plant` — Cave Vines Plant. `InstantMinable | Flammable`
    pub struct CaveVinesPlant;
    facets: InstantMinable, Flammable;
    state: CaveVinesPlantState;
}

block! {
    /// `chain_command_block` — Chain Command Block. `UnMovable`
    pub struct ChainCommandBlock;
    facets: UnMovable;
    state: CommandState;
}

block! {
    /// `cherry_button` — Cherry Button. `Breakable | Flammable`
    pub struct CherryButton;
    facets: Breakable, Flammable;
    state: SwitchState;
}

block! {
    /// `cherry_door` — Cherry Door. `Breakable | Flammable`
    pub struct CherryDoor;
    facets: Breakable, Flammable;
    state: DoorState;
}

block! {
    /// `cherry_fence` — Cherry Fence. `Movable | Breakable | Flammable`
    pub struct CherryFence;
    facets: Movable, Breakable, Flammable;
    state: ConnectionState;
}

block! {
    /// `cherry_fence_gate` — Cherry Fence Gate. `Movable | Breakable | Flammable`
    pub struct CherryFenceGate;
    facets: Movable, Breakable, Flammable;
    state: FenceGateState;
}

block! {
    /// `cherry_hanging_sign` — Cherry Hanging Sign. `Breakable | Flammable`
    pub struct CherryHangingSign;
    facets: Breakable, Flammable;
    state: HangingSignState;
}

block! {
    /// `cherry_leaves` — Cherry Leaves. `Movable | Breakable | Flammable`
    pub struct CherryLeaves;
    facets: Movable, Breakable, Flammable;
    state: LeavesState;
}

block! {
    /// `cherry_log` — Cherry Log. `Movable | Breakable | Flammable`
    pub struct CherryLog;
    facets: Movable, Breakable, Flammable;
    state: PillarState;
}

block! {
    /// `cherry_planks` — Cherry Planks. `Movable | Breakable | Flammable`
    pub struct CherryPlanks;
    facets: Movable, Breakable, Flammable;
}

block! {
    /// `cherry_pressure_plate` — Cherry Pressure Plate. `Breakable | Flammable`
    pub struct CherryPressurePlate;
    facets: Breakable, Flammable;
    state: PressurePlateState;
}

block! {
    /// `cherry_sapling` — Cherry Sapling. `InstantMinable`
    pub struct CherrySapling;
    facets: InstantMinable;
    state: SaplingState;
}

block! {
    /// `cherry_shelf` — Cherry Shelf. `UnMovable | Breakable`
    pub struct CherryShelf;
    facets: UnMovable, Breakable;
    state: ShelfState;
}

block! {
    /// `cherry_sign` — Cherry Sign. `Breakable | Flammable`
    pub struct CherrySign;
    facets: Breakable, Flammable;
    state: StandingSignState;
}

block! {
    /// `cherry_slab` — Cherry Slab. `Movable | Breakable | Flammable`
    pub struct CherrySlab;
    facets: Movable, Breakable, Flammable;
    state: SlabState;
}

block! {
    /// `cherry_stairs` — Cherry Stairs. `Movable | Breakable | Flammable`
    pub struct CherryStairs;
    facets: Movable, Breakable, Flammable;
    state: StairState;
}

block! {
    /// `cherry_trapdoor` — Cherry Trapdoor. `Movable | Breakable | Flammable`
    pub struct CherryTrapdoor;
    facets: Movable, Breakable, Flammable;
    state: TrapdoorState;
}

block! {
    /// `cherry_wall_hanging_sign` — Cherry Hanging Sign. `Breakable | Flammable`
    pub struct CherryWallHangingSign;
    facets: Breakable, Flammable;
    state: WallMountedState;
}

block! {
    /// `cherry_wall_sign` — Cherry Sign. `Breakable | Flammable`
    pub struct CherryWallSign;
    facets: Breakable, Flammable;
    state: WallMountedState;
}

block! {
    /// `cherry_wood` — Cherry Wood. `Movable | Breakable | Flammable`
    pub struct CherryWood;
    facets: Movable, Breakable, Flammable;
    state: PillarState;
}

block! {
    /// `chest` — Chest. `UnMovable | Breakable`
    pub struct Chest;
    facets: UnMovable, Breakable;
    state: ChestState;
}

block! {
    /// `chipped_anvil` — Chipped Anvil. `Movable | Breakable`
    pub struct ChippedAnvil;
    facets: Movable, Breakable;
    state: FacingState;
}

block! {
    /// `chiseled_bookshelf` — Chiseled Bookshelf. `UnMovable | Breakable`
    pub struct ChiseledBookshelf;
    facets: UnMovable, Breakable;
    state: ChiseledBookShelfState;
}

block! {
    /// `chiseled_cinnabar` — Chiseled Cinnabar. `Movable | Breakable`
    pub struct ChiseledCinnabar;
    facets: Movable, Breakable;
}

block! {
    /// `chiseled_copper` — Chiseled Copper. `Movable | Breakable`
    pub struct ChiseledCopper;
    facets: Movable, Breakable;
}

block! {
    /// `chiseled_deepslate` — Chiseled Deepslate. `Movable | Breakable`
    pub struct ChiseledDeepslate;
    facets: Movable, Breakable;
}

block! {
    /// `chiseled_nether_bricks` — Chiseled Nether Bricks. `Movable | Breakable`
    pub struct ChiseledNetherBricks;
    facets: Movable, Breakable;
}

block! {
    /// `chiseled_polished_blackstone` — Chiseled Polished Blackstone. `Movable | Breakable`
    pub struct ChiseledPolishedBlackstone;
    facets: Movable, Breakable;
}

block! {
    /// `chiseled_quartz_block` — Chiseled Quartz Block. `Movable | Breakable`
    pub struct ChiseledQuartzBlock;
    facets: Movable, Breakable;
}

block! {
    /// `chiseled_red_sandstone` — Chiseled Red Sandstone. `Movable | Breakable`
    pub struct ChiseledRedSandstone;
    facets: Movable, Breakable;
}

block! {
    /// `chiseled_resin_bricks` — Chiseled Resin Bricks. `Movable | Breakable`
    pub struct ChiseledResinBricks;
    facets: Movable, Breakable;
}

block! {
    /// `chiseled_sandstone` — Chiseled Sandstone. `Movable | Breakable`
    pub struct ChiseledSandstone;
    facets: Movable, Breakable;
}

block! {
    /// `chiseled_stone_bricks` — Chiseled Stone Bricks. `Movable | Breakable`
    pub struct ChiseledStoneBricks;
    facets: Movable, Breakable;
}

block! {
    /// `chiseled_sulfur` — Chiseled Sulfur. `Movable | Breakable`
    pub struct ChiseledSulfur;
    facets: Movable, Breakable;
}

block! {
    /// `chiseled_tuff` — Chiseled Tuff. `Movable | Breakable`
    pub struct ChiseledTuff;
    facets: Movable, Breakable;
}

block! {
    /// `chiseled_tuff_bricks` — Chiseled Tuff Bricks. `Movable | Breakable`
    pub struct ChiseledTuffBricks;
    facets: Movable, Breakable;
}

block! {
    /// `chorus_flower` — Chorus Flower. `Breakable`
    pub struct ChorusFlower;
    facets: Breakable;
    state: ChorusFlowerState;
}

block! {
    /// `chorus_plant` — Chorus Plant. `Breakable`
    pub struct ChorusPlant;
    facets: Breakable;
    state: HugeMushroomState;
}

block! {
    /// `cinnabar` — Cinnabar. `Movable | Breakable`
    pub struct Cinnabar;
    facets: Movable, Breakable;
}

block! {
    /// `cinnabar_brick_slab` — Cinnabar Brick Slab. `Movable | Breakable`
    pub struct CinnabarBrickSlab;
    facets: Movable, Breakable;
    state: SlabState;
}

block! {
    /// `cinnabar_brick_stairs` — Cinnabar Brick Stairs. `Movable | Breakable`
    pub struct CinnabarBrickStairs;
    facets: Movable, Breakable;
    state: StairState;
}

block! {
    /// `cinnabar_brick_wall` — Cinnabar Brick Wall. `Movable | Breakable`
    pub struct CinnabarBrickWall;
    facets: Movable, Breakable;
    state: WallState;
}

block! {
    /// `cinnabar_bricks` — Cinnabar Bricks. `Movable | Breakable`
    pub struct CinnabarBricks;
    facets: Movable, Breakable;
}

block! {
    /// `cinnabar_slab` — Cinnabar Slab. `Movable | Breakable`
    pub struct CinnabarSlab;
    facets: Movable, Breakable;
    state: SlabState;
}

block! {
    /// `cinnabar_stairs` — Cinnabar Stairs. `Movable | Breakable`
    pub struct CinnabarStairs;
    facets: Movable, Breakable;
    state: StairState;
}

block! {
    /// `cinnabar_wall` — Cinnabar Wall. `Movable | Breakable`
    pub struct CinnabarWall;
    facets: Movable, Breakable;
    state: WallState;
}

block! {
    /// `clay` — Clay. `Movable | Breakable`
    pub struct Clay;
    facets: Movable, Breakable;
}

block! {
    /// `closed_eyeblossom` — Closed Eyeblossom. `InstantMinable`
    pub struct ClosedEyeblossom;
    facets: InstantMinable;
}

block! {
    /// `coal_block` — Block of Coal. `Movable | Breakable | Flammable`
    pub struct CoalBlock;
    facets: Movable, Breakable, Flammable;
}

block! {
    /// `coal_ore` — Coal Ore. `Movable | Breakable`
    pub struct CoalOre;
    facets: Movable, Breakable;
}

block! {
    /// `coarse_dirt` — Coarse Dirt. `Movable | Breakable`
    pub struct CoarseDirt;
    facets: Movable, Breakable;
}

block! {
    /// `cobbled_deepslate` — Cobbled Deepslate. `Movable | Breakable`
    pub struct CobbledDeepslate;
    facets: Movable, Breakable;
}

block! {
    /// `cobbled_deepslate_slab` — Cobbled Deepslate Slab. `Movable | Breakable`
    pub struct CobbledDeepslateSlab;
    facets: Movable, Breakable;
    state: SlabState;
}

block! {
    /// `cobbled_deepslate_stairs` — Cobbled Deepslate Stairs. `Movable | Breakable`
    pub struct CobbledDeepslateStairs;
    facets: Movable, Breakable;
    state: StairState;
}

block! {
    /// `cobbled_deepslate_wall` — Cobbled Deepslate Wall. `Movable | Breakable`
    pub struct CobbledDeepslateWall;
    facets: Movable, Breakable;
    state: WallState;
}

block! {
    /// `cobblestone` — Cobblestone. `Movable | Breakable`
    pub struct Cobblestone;
    facets: Movable, Breakable;
}

block! {
    /// `cobblestone_slab` — Cobblestone Slab. `Movable | Breakable`
    pub struct CobblestoneSlab;
    facets: Movable, Breakable;
    state: SlabState;
}

block! {
    /// `cobblestone_stairs` — Cobblestone Stairs. `Movable | Breakable`
    pub struct CobblestoneStairs;
    facets: Movable, Breakable;
    state: StairState;
}

block! {
    /// `cobblestone_wall` — Cobblestone Wall. `Movable | Breakable`
    pub struct CobblestoneWall;
    facets: Movable, Breakable;
    state: WallState;
}

block! {
    /// `cobweb` — Cobweb. `Breakable`
    pub struct Cobweb;
    facets: Breakable;
}

block! {
    /// `cocoa` — Cocoa. `Breakable`
    pub struct Cocoa;
    facets: Breakable;
    state: CocoaState;
}

block! {
    /// `command_block` — Command Block. `UnMovable`
    pub struct CommandBlock;
    facets: UnMovable;
    state: CommandState;
}

block! {
    /// `comparator` — Redstone Comparator. `InstantMinable`
    pub struct Comparator;
    facets: InstantMinable;
    state: ComparatorState;
}

block! {
    /// `composter` — Composter. `Movable | Breakable | Flammable`
    pub struct Composter;
    facets: Movable, Breakable, Flammable;
    state: ComposterState;
}

block! {
    /// `conduit` — Conduit. `UnMovable | Breakable`
    pub struct Conduit;
    facets: UnMovable, Breakable;
    state: WaterloggedState;
}

block! {
    /// `copper_bars` — Copper Bars. `Movable | Breakable`
    pub struct CopperBars;
    facets: Movable, Breakable;
    state: ConnectionState;
}

block! {
    /// `copper_block` — Block of Copper. `Movable | Breakable`
    pub struct CopperBlock;
    facets: Movable, Breakable;
}

block! {
    /// `copper_bulb` — Copper Bulb. `Movable | Breakable`
    pub struct CopperBulb;
    facets: Movable, Breakable;
    state: WeatheringCopperBulbState;
}

block! {
    /// `copper_chain` — Copper Chain. `Movable | Breakable`
    pub struct CopperChain;
    facets: Movable, Breakable;
    state: WeatheringCopperChainState;
}

block! {
    /// `copper_chest` — Copper Chest. `UnMovable | Breakable`
    pub struct CopperChest;
    facets: UnMovable, Breakable;
    state: ChestState;
}

block! {
    /// `copper_door` — Copper Door. `Breakable`
    pub struct CopperDoor;
    facets: Breakable;
    state: DoorState;
}

block! {
    /// `copper_golem_statue` — Copper Golem Statue. `Movable | Breakable`
    pub struct CopperGolemStatue;
    facets: Movable, Breakable;
    state: WeatheringCopperGolemStatueState;
}

block! {
    /// `copper_grate` — Copper Grate. `Movable | Breakable`
    pub struct CopperGrate;
    facets: Movable, Breakable;
    state: WaterloggedState;
}

block! {
    /// `copper_lantern` — Copper Lantern. `Breakable`
    pub struct CopperLantern;
    facets: Breakable;
    state: WeatheringLanternState;
}

block! {
    /// `copper_ore` — Copper Ore. `Movable | Breakable`
    pub struct CopperOre;
    facets: Movable, Breakable;
}

block! {
    /// `copper_torch` — Copper Torch. `InstantMinable`
    pub struct CopperTorch;
    facets: InstantMinable;
}

block! {
    /// `copper_trapdoor` — Copper Trapdoor. `Movable | Breakable`
    pub struct CopperTrapdoor;
    facets: Movable, Breakable;
    state: TrapdoorState;
}

block! {
    /// `copper_wall_torch` — Copper Torch. `InstantMinable`
    pub struct CopperWallTorch;
    facets: InstantMinable;
    state: FacingState;
}

block! {
    /// `cornflower` — Cornflower. `InstantMinable | Flammable`
    pub struct Cornflower;
    facets: InstantMinable, Flammable;
}

block! {
    /// `cracked_deepslate_bricks` — Cracked Deepslate Bricks. `Movable | Breakable`
    pub struct CrackedDeepslateBricks;
    facets: Movable, Breakable;
}

block! {
    /// `cracked_deepslate_tiles` — Cracked Deepslate Tiles. `Movable | Breakable`
    pub struct CrackedDeepslateTiles;
    facets: Movable, Breakable;
}

block! {
    /// `cracked_nether_bricks` — Cracked Nether Bricks. `Movable | Breakable`
    pub struct CrackedNetherBricks;
    facets: Movable, Breakable;
}

block! {
    /// `cracked_polished_blackstone_bricks` — Cracked Polished Blackstone Bricks. `Movable | Breakable`
    pub struct CrackedPolishedBlackstoneBricks;
    facets: Movable, Breakable;
}

block! {
    /// `cracked_stone_bricks` — Cracked Stone Bricks. `Movable | Breakable`
    pub struct CrackedStoneBricks;
    facets: Movable, Breakable;
}

block! {
    /// `crafter` — Crafter. `UnMovable | Breakable`
    pub struct Crafter;
    facets: UnMovable, Breakable;
    state: CrafterState;
}

block! {
    /// `crafting_table` — Crafting Table. `Movable | Breakable`
    pub struct CraftingTable;
    facets: Movable, Breakable;
}

block! {
    /// `creaking_heart` — Creaking Heart. `UnMovable | Breakable`
    pub struct CreakingHeart;
    facets: UnMovable, Breakable;
    state: CreakingHeartState;
}

block! {
    /// `creeper_head` — Creeper Head. `Breakable`
    pub struct CreeperHead;
    facets: Breakable;
    state: SkullState;
}

block! {
    /// `creeper_wall_head` — Creeper Head. `Breakable`
    pub struct CreeperWallHead;
    facets: Breakable;
    state: WallSkullState;
}

block! {
    /// `crimson_button` — Crimson Button. `Breakable`
    pub struct CrimsonButton;
    facets: Breakable;
    state: SwitchState;
}

block! {
    /// `crimson_door` — Crimson Door. `Breakable`
    pub struct CrimsonDoor;
    facets: Breakable;
    state: DoorState;
}

block! {
    /// `crimson_fence` — Crimson Fence. `Movable | Breakable`
    pub struct CrimsonFence;
    facets: Movable, Breakable;
    state: ConnectionState;
}

block! {
    /// `crimson_fence_gate` — Crimson Fence Gate. `Movable | Breakable`
    pub struct CrimsonFenceGate;
    facets: Movable, Breakable;
    state: FenceGateState;
}

block! {
    /// `crimson_fungus` — Crimson Fungus. `InstantMinable`
    pub struct CrimsonFungus;
    facets: InstantMinable;
}

block! {
    /// `crimson_hanging_sign` — Crimson Hanging Sign. `Breakable`
    pub struct CrimsonHangingSign;
    facets: Breakable;
    state: HangingSignState;
}

block! {
    /// `crimson_hyphae` — Crimson Hyphae. `Movable | Breakable`
    pub struct CrimsonHyphae;
    facets: Movable, Breakable;
    state: PillarState;
}

block! {
    /// `crimson_nylium` — Crimson Nylium. `Movable | Breakable`
    pub struct CrimsonNylium;
    facets: Movable, Breakable;
}

block! {
    /// `crimson_planks` — Crimson Planks. `Movable | Breakable`
    pub struct CrimsonPlanks;
    facets: Movable, Breakable;
}

block! {
    /// `crimson_pressure_plate` — Crimson Pressure Plate. `Breakable`
    pub struct CrimsonPressurePlate;
    facets: Breakable;
    state: PressurePlateState;
}

block! {
    /// `crimson_roots` — Crimson Roots. `Movable | InstantMinable`
    pub struct CrimsonRoots;
    facets: Movable, InstantMinable;
}

block! {
    /// `crimson_shelf` — Crimson Shelf. `UnMovable | Breakable`
    pub struct CrimsonShelf;
    facets: UnMovable, Breakable;
    state: ShelfState;
}

block! {
    /// `crimson_sign` — Crimson Sign. `Breakable`
    pub struct CrimsonSign;
    facets: Breakable;
    state: StandingSignState;
}

block! {
    /// `crimson_slab` — Crimson Slab. `Movable | Breakable`
    pub struct CrimsonSlab;
    facets: Movable, Breakable;
    state: SlabState;
}

block! {
    /// `crimson_stairs` — Crimson Stairs. `Movable | Breakable`
    pub struct CrimsonStairs;
    facets: Movable, Breakable;
    state: StairState;
}

block! {
    /// `crimson_stem` — Crimson Stem. `Movable | Breakable`
    pub struct CrimsonStem;
    facets: Movable, Breakable;
    state: PillarState;
}

block! {
    /// `crimson_trapdoor` — Crimson Trapdoor. `Movable | Breakable`
    pub struct CrimsonTrapdoor;
    facets: Movable, Breakable;
    state: TrapdoorState;
}

block! {
    /// `crimson_wall_hanging_sign` — Crimson Hanging Sign. `Breakable`
    pub struct CrimsonWallHangingSign;
    facets: Breakable;
    state: WallMountedState;
}

block! {
    /// `crimson_wall_sign` — Crimson Sign. `Breakable`
    pub struct CrimsonWallSign;
    facets: Breakable;
    state: WallMountedState;
}

block! {
    /// `crying_obsidian` — Crying Obsidian. `UnMovable | Breakable`
    pub struct CryingObsidian;
    facets: UnMovable, Breakable;
}

block! {
    /// `cut_copper` — Cut Copper. `Movable | Breakable`
    pub struct CutCopper;
    facets: Movable, Breakable;
}

block! {
    /// `cut_copper_slab` — Cut Copper Slab. `Movable | Breakable`
    pub struct CutCopperSlab;
    facets: Movable, Breakable;
    state: SlabState;
}

block! {
    /// `cut_copper_stairs` — Cut Copper Stairs. `Movable | Breakable`
    pub struct CutCopperStairs;
    facets: Movable, Breakable;
    state: StairState;
}

block! {
    /// `cut_red_sandstone` — Cut Red Sandstone. `Movable | Breakable`
    pub struct CutRedSandstone;
    facets: Movable, Breakable;
}

block! {
    /// `cut_red_sandstone_slab` — Cut Red Sandstone Slab. `Movable | Breakable`
    pub struct CutRedSandstoneSlab;
    facets: Movable, Breakable;
    state: SlabState;
}

block! {
    /// `cut_sandstone` — Cut Sandstone. `Movable | Breakable`
    pub struct CutSandstone;
    facets: Movable, Breakable;
}

block! {
    /// `cut_sandstone_slab` — Cut Sandstone Slab. `Movable | Breakable`
    pub struct CutSandstoneSlab;
    facets: Movable, Breakable;
    state: SlabState;
}

block! {
    /// `cyan_banner` — Cyan Banner. `Breakable`
    pub struct CyanBanner;
    facets: Breakable;
    state: BannerState;
}

block! {
    /// `cyan_bed` — Cyan Bed. `Breakable`
    pub struct CyanBed;
    facets: Breakable;
    state: BedState;
}

block! {
    /// `cyan_candle` — Cyan Candle. `Movable | Breakable`
    pub struct CyanCandle;
    facets: Movable, Breakable;
    state: CandleState;
}

block! {
    /// `cyan_candle_cake` — Cake with Cyan Candle. `Movable | Breakable`
    pub struct CyanCandleCake;
    facets: Movable, Breakable;
    state: LitState;
}

block! {
    /// `cyan_carpet` — Cyan Carpet. `Movable | Breakable | Flammable`
    pub struct CyanCarpet;
    facets: Movable, Breakable, Flammable;
}

block! {
    /// `cyan_concrete` — Cyan Concrete. `Movable | Breakable`
    pub struct CyanConcrete;
    facets: Movable, Breakable;
}

block! {
    /// `cyan_concrete_powder` — Cyan Concrete Powder. `Movable | Breakable`
    pub struct CyanConcretePowder;
    facets: Movable, Breakable;
}

block! {
    /// `cyan_glazed_terracotta` — Cyan Glazed Terracotta. `Movable | Breakable`
    pub struct CyanGlazedTerracotta;
    facets: Movable, Breakable;
    state: FacingState;
}

block! {
    /// `cyan_shulker_box` — Cyan Shulker Box. `Breakable`
    pub struct CyanShulkerBox;
    facets: Breakable;
    state: ShulkerBoxState;
}

block! {
    /// `cyan_stained_glass` — Cyan Stained Glass. `Movable | Breakable`
    pub struct CyanStainedGlass;
    facets: Movable, Breakable;
}

block! {
    /// `cyan_stained_glass_pane` — Cyan Stained Glass Pane. `Movable | Breakable`
    pub struct CyanStainedGlassPane;
    facets: Movable, Breakable;
    state: ConnectionState;
}

block! {
    /// `cyan_terracotta` — Cyan Terracotta. `Movable | Breakable`
    pub struct CyanTerracotta;
    facets: Movable, Breakable;
}

block! {
    /// `cyan_wall_banner` — Cyan Banner. `Breakable`
    pub struct CyanWallBanner;
    facets: Breakable;
    state: FacingState;
}

block! {
    /// `cyan_wool` — Cyan Wool. `Movable | Breakable | Flammable`
    pub struct CyanWool;
    facets: Movable, Breakable, Flammable;
}

block! {
    /// `damaged_anvil` — Damaged Anvil. `Movable | Breakable`
    pub struct DamagedAnvil;
    facets: Movable, Breakable;
    state: FacingState;
}

block! {
    /// `dandelion` — Dandelion. `InstantMinable | Flammable`
    pub struct Dandelion;
    facets: InstantMinable, Flammable;
}

block! {
    /// `dark_oak_button` — Dark Oak Button. `Breakable | Flammable`
    pub struct DarkOakButton;
    facets: Breakable, Flammable;
    state: SwitchState;
}

block! {
    /// `dark_oak_door` — Dark Oak Door. `Breakable | Flammable`
    pub struct DarkOakDoor;
    facets: Breakable, Flammable;
    state: DoorState;
}

block! {
    /// `dark_oak_fence` — Dark Oak Fence. `Movable | Breakable | Flammable`
    pub struct DarkOakFence;
    facets: Movable, Breakable, Flammable;
    state: ConnectionState;
}

block! {
    /// `dark_oak_fence_gate` — Dark Oak Fence Gate. `Movable | Breakable | Flammable`
    pub struct DarkOakFenceGate;
    facets: Movable, Breakable, Flammable;
    state: FenceGateState;
}

block! {
    /// `dark_oak_hanging_sign` — Dark Oak Hanging Sign. `Breakable | Flammable`
    pub struct DarkOakHangingSign;
    facets: Breakable, Flammable;
    state: HangingSignState;
}

block! {
    /// `dark_oak_leaves` — Dark Oak Leaves. `Movable | Breakable | Flammable`
    pub struct DarkOakLeaves;
    facets: Movable, Breakable, Flammable;
    state: LeavesState;
}

block! {
    /// `dark_oak_log` — Dark Oak Log. `Movable | Breakable | Flammable`
    pub struct DarkOakLog;
    facets: Movable, Breakable, Flammable;
    state: PillarState;
}

block! {
    /// `dark_oak_planks` — Dark Oak Planks. `Movable | Breakable | Flammable`
    pub struct DarkOakPlanks;
    facets: Movable, Breakable, Flammable;
}

block! {
    /// `dark_oak_pressure_plate` — Dark Oak Pressure Plate. `Breakable | Flammable`
    pub struct DarkOakPressurePlate;
    facets: Breakable, Flammable;
    state: PressurePlateState;
}

block! {
    /// `dark_oak_sapling` — Dark Oak Sapling. `InstantMinable`
    pub struct DarkOakSapling;
    facets: InstantMinable;
    state: SaplingState;
}

block! {
    /// `dark_oak_shelf` — Dark Oak Shelf. `UnMovable | Breakable`
    pub struct DarkOakShelf;
    facets: UnMovable, Breakable;
    state: ShelfState;
}

block! {
    /// `dark_oak_sign` — Dark Oak Sign. `Breakable | Flammable`
    pub struct DarkOakSign;
    facets: Breakable, Flammable;
    state: StandingSignState;
}

block! {
    /// `dark_oak_slab` — Dark Oak Slab. `Movable | Breakable | Flammable`
    pub struct DarkOakSlab;
    facets: Movable, Breakable, Flammable;
    state: SlabState;
}

block! {
    /// `dark_oak_stairs` — Dark Oak Stairs. `Movable | Breakable | Flammable`
    pub struct DarkOakStairs;
    facets: Movable, Breakable, Flammable;
    state: StairState;
}

block! {
    /// `dark_oak_trapdoor` — Dark Oak Trapdoor. `Movable | Breakable | Flammable`
    pub struct DarkOakTrapdoor;
    facets: Movable, Breakable, Flammable;
    state: TrapdoorState;
}

block! {
    /// `dark_oak_wall_hanging_sign` — Dark Oak Hanging Sign. `Breakable | Flammable`
    pub struct DarkOakWallHangingSign;
    facets: Breakable, Flammable;
    state: WallMountedState;
}

block! {
    /// `dark_oak_wall_sign` — Dark Oak Sign. `Breakable | Flammable`
    pub struct DarkOakWallSign;
    facets: Breakable, Flammable;
    state: WallMountedState;
}

block! {
    /// `dark_oak_wood` — Dark Oak Wood. `Movable | Breakable | Flammable`
    pub struct DarkOakWood;
    facets: Movable, Breakable, Flammable;
    state: PillarState;
}

block! {
    /// `dark_prismarine` — Dark Prismarine. `Movable | Breakable`
    pub struct DarkPrismarine;
    facets: Movable, Breakable;
}

block! {
    /// `dark_prismarine_slab` — Dark Prismarine Slab. `Movable | Breakable`
    pub struct DarkPrismarineSlab;
    facets: Movable, Breakable;
    state: SlabState;
}

block! {
    /// `dark_prismarine_stairs` — Dark Prismarine Stairs. `Movable | Breakable`
    pub struct DarkPrismarineStairs;
    facets: Movable, Breakable;
    state: StairState;
}

block! {
    /// `daylight_detector` — Daylight Detector. `UnMovable | Breakable`
    pub struct DaylightDetector;
    facets: UnMovable, Breakable;
    state: DaylightDetectorState;
}

block! {
    /// `dead_brain_coral` — Dead Brain Coral. `Movable | InstantMinable`
    pub struct DeadBrainCoral;
    facets: Movable, InstantMinable;
    state: WaterloggedState;
}

block! {
    /// `dead_brain_coral_block` — Dead Brain Coral Block. `Movable | Breakable`
    pub struct DeadBrainCoralBlock;
    facets: Movable, Breakable;
}

block! {
    /// `dead_brain_coral_fan` — Dead Brain Coral Fan. `Movable | InstantMinable`
    pub struct DeadBrainCoralFan;
    facets: Movable, InstantMinable;
    state: WaterloggedState;
}

block! {
    /// `dead_brain_coral_wall_fan` — Dead Brain Coral Wall Fan. `Movable | InstantMinable`
    pub struct DeadBrainCoralWallFan;
    facets: Movable, InstantMinable;
    state: WallMountedState;
}

block! {
    /// `dead_bubble_coral` — Dead Bubble Coral. `Movable | InstantMinable`
    pub struct DeadBubbleCoral;
    facets: Movable, InstantMinable;
    state: WaterloggedState;
}

block! {
    /// `dead_bubble_coral_block` — Dead Bubble Coral Block. `Movable | Breakable`
    pub struct DeadBubbleCoralBlock;
    facets: Movable, Breakable;
}

block! {
    /// `dead_bubble_coral_fan` — Dead Bubble Coral Fan. `Movable | InstantMinable`
    pub struct DeadBubbleCoralFan;
    facets: Movable, InstantMinable;
    state: WaterloggedState;
}

block! {
    /// `dead_bubble_coral_wall_fan` — Dead Bubble Coral Wall Fan. `Movable | InstantMinable`
    pub struct DeadBubbleCoralWallFan;
    facets: Movable, InstantMinable;
    state: WallMountedState;
}

block! {
    /// `dead_bush` — Dead Bush. `InstantMinable | Flammable`
    pub struct DeadBush;
    facets: InstantMinable, Flammable;
}

block! {
    /// `dead_fire_coral` — Dead Fire Coral. `Movable | InstantMinable`
    pub struct DeadFireCoral;
    facets: Movable, InstantMinable;
    state: WaterloggedState;
}

block! {
    /// `dead_fire_coral_block` — Dead Fire Coral Block. `Movable | Breakable`
    pub struct DeadFireCoralBlock;
    facets: Movable, Breakable;
}

block! {
    /// `dead_fire_coral_fan` — Dead Fire Coral Fan. `Movable | InstantMinable`
    pub struct DeadFireCoralFan;
    facets: Movable, InstantMinable;
    state: WaterloggedState;
}

block! {
    /// `dead_fire_coral_wall_fan` — Dead Fire Coral Wall Fan. `Movable | InstantMinable`
    pub struct DeadFireCoralWallFan;
    facets: Movable, InstantMinable;
    state: WallMountedState;
}

block! {
    /// `dead_horn_coral` — Dead Horn Coral. `Movable | InstantMinable`
    pub struct DeadHornCoral;
    facets: Movable, InstantMinable;
    state: WaterloggedState;
}

block! {
    /// `dead_horn_coral_block` — Dead Horn Coral Block. `Movable | Breakable`
    pub struct DeadHornCoralBlock;
    facets: Movable, Breakable;
}

block! {
    /// `dead_horn_coral_fan` — Dead Horn Coral Fan. `Movable | InstantMinable`
    pub struct DeadHornCoralFan;
    facets: Movable, InstantMinable;
    state: WaterloggedState;
}

block! {
    /// `dead_horn_coral_wall_fan` — Dead Horn Coral Wall Fan. `Movable | InstantMinable`
    pub struct DeadHornCoralWallFan;
    facets: Movable, InstantMinable;
    state: WallMountedState;
}

block! {
    /// `dead_tube_coral` — Dead Tube Coral. `Movable | InstantMinable`
    pub struct DeadTubeCoral;
    facets: Movable, InstantMinable;
    state: WaterloggedState;
}

block! {
    /// `dead_tube_coral_block` — Dead Tube Coral Block. `Movable | Breakable`
    pub struct DeadTubeCoralBlock;
    facets: Movable, Breakable;
}

block! {
    /// `dead_tube_coral_fan` — Dead Tube Coral Fan. `Movable | InstantMinable`
    pub struct DeadTubeCoralFan;
    facets: Movable, InstantMinable;
    state: WaterloggedState;
}

block! {
    /// `dead_tube_coral_wall_fan` — Dead Tube Coral Wall Fan. `Movable | InstantMinable`
    pub struct DeadTubeCoralWallFan;
    facets: Movable, InstantMinable;
    state: WallMountedState;
}

block! {
    /// `decorated_pot` — Decorated Pot. `InstantMinable`
    pub struct DecoratedPot;
    facets: InstantMinable;
    state: DecoratedPotState;
}

block! {
    /// `deepslate` — Deepslate. `Movable | Breakable`
    pub struct Deepslate;
    facets: Movable, Breakable;
    state: PillarState;
}

block! {
    /// `deepslate_brick_slab` — Deepslate Brick Slab. `Movable | Breakable`
    pub struct DeepslateBrickSlab;
    facets: Movable, Breakable;
    state: SlabState;
}

block! {
    /// `deepslate_brick_stairs` — Deepslate Brick Stairs. `Movable | Breakable`
    pub struct DeepslateBrickStairs;
    facets: Movable, Breakable;
    state: StairState;
}

block! {
    /// `deepslate_brick_wall` — Deepslate Brick Wall. `Movable | Breakable`
    pub struct DeepslateBrickWall;
    facets: Movable, Breakable;
    state: WallState;
}

block! {
    /// `deepslate_bricks` — Deepslate Bricks. `Movable | Breakable`
    pub struct DeepslateBricks;
    facets: Movable, Breakable;
}

block! {
    /// `deepslate_coal_ore` — Deepslate Coal Ore. `Movable | Breakable`
    pub struct DeepslateCoalOre;
    facets: Movable, Breakable;
}

block! {
    /// `deepslate_copper_ore` — Deepslate Copper Ore. `Movable | Breakable`
    pub struct DeepslateCopperOre;
    facets: Movable, Breakable;
}

block! {
    /// `deepslate_diamond_ore` — Deepslate Diamond Ore. `Movable | Breakable`
    pub struct DeepslateDiamondOre;
    facets: Movable, Breakable;
}

block! {
    /// `deepslate_emerald_ore` — Deepslate Emerald Ore. `Movable | Breakable`
    pub struct DeepslateEmeraldOre;
    facets: Movable, Breakable;
}

block! {
    /// `deepslate_gold_ore` — Deepslate Gold Ore. `Movable | Breakable`
    pub struct DeepslateGoldOre;
    facets: Movable, Breakable;
}

block! {
    /// `deepslate_iron_ore` — Deepslate Iron Ore. `Movable | Breakable`
    pub struct DeepslateIronOre;
    facets: Movable, Breakable;
}

block! {
    /// `deepslate_lapis_ore` — Deepslate Lapis Lazuli Ore. `Movable | Breakable`
    pub struct DeepslateLapisOre;
    facets: Movable, Breakable;
}

block! {
    /// `deepslate_redstone_ore` — Deepslate Redstone Ore. `Movable | Breakable`
    pub struct DeepslateRedstoneOre;
    facets: Movable, Breakable;
    state: LitState;
}

block! {
    /// `deepslate_tile_slab` — Deepslate Tile Slab. `Movable | Breakable`
    pub struct DeepslateTileSlab;
    facets: Movable, Breakable;
    state: SlabState;
}

block! {
    /// `deepslate_tile_stairs` — Deepslate Tile Stairs. `Movable | Breakable`
    pub struct DeepslateTileStairs;
    facets: Movable, Breakable;
    state: StairState;
}

block! {
    /// `deepslate_tile_wall` — Deepslate Tile Wall. `Movable | Breakable`
    pub struct DeepslateTileWall;
    facets: Movable, Breakable;
    state: WallState;
}

block! {
    /// `deepslate_tiles` — Deepslate Tiles. `Movable | Breakable`
    pub struct DeepslateTiles;
    facets: Movable, Breakable;
}

block! {
    /// `detector_rail` — Detector Rail. `Movable | Breakable`
    pub struct DetectorRail;
    facets: Movable, Breakable;
    state: PoweredRailState;
}

block! {
    /// `diamond_block` — Block of Diamond. `Movable | Breakable`
    pub struct DiamondBlock;
    facets: Movable, Breakable;
}

block! {
    /// `diamond_ore` — Diamond Ore. `Movable | Breakable`
    pub struct DiamondOre;
    facets: Movable, Breakable;
}

block! {
    /// `diorite` — Diorite. `Movable | Breakable`
    pub struct Diorite;
    facets: Movable, Breakable;
}

block! {
    /// `diorite_slab` — Diorite Slab. `Movable | Breakable`
    pub struct DioriteSlab;
    facets: Movable, Breakable;
    state: SlabState;
}

block! {
    /// `diorite_stairs` — Diorite Stairs. `Movable | Breakable`
    pub struct DioriteStairs;
    facets: Movable, Breakable;
    state: StairState;
}

block! {
    /// `diorite_wall` — Diorite Wall. `Movable | Breakable`
    pub struct DioriteWall;
    facets: Movable, Breakable;
    state: WallState;
}

block! {
    /// `dirt` — Dirt. `Movable | Breakable`
    pub struct Dirt;
    facets: Movable, Breakable;
}

block! {
    /// `dirt_path` — Dirt Path. `Movable | Breakable`
    pub struct DirtPath;
    facets: Movable, Breakable;
}

block! {
    /// `dispenser` — Dispenser. `UnMovable | Breakable`
    pub struct Dispenser;
    facets: UnMovable, Breakable;
    state: DispenserState;
}

block! {
    /// `dragon_egg` — Dragon Egg. `Breakable`
    pub struct DragonEgg;
    facets: Breakable;
}

block! {
    /// `dragon_head` — Dragon Head. `Breakable`
    pub struct DragonHead;
    facets: Breakable;
    state: SkullState;
}

block! {
    /// `dragon_wall_head` — Dragon Head. `Breakable`
    pub struct DragonWallHead;
    facets: Breakable;
    state: WallSkullState;
}

block! {
    /// `dried_ghast` — Dried Ghast. `Movable | InstantMinable`
    pub struct DriedGhast;
    facets: Movable, InstantMinable;
    state: DriedGhastState;
}

block! {
    /// `dried_kelp_block` — Dried Kelp Block. `Movable | Breakable | Flammable`
    pub struct DriedKelpBlock;
    facets: Movable, Breakable, Flammable;
}

block! {
    /// `dripstone_block` — Dripstone Block. `Movable | Breakable`
    pub struct DripstoneBlock;
    facets: Movable, Breakable;
}

block! {
    /// `dropper` — Dropper. `UnMovable | Breakable`
    pub struct Dropper;
    facets: UnMovable, Breakable;
    state: DispenserState;
}

block! {
    /// `emerald_block` — Block of Emerald. `Movable | Breakable`
    pub struct EmeraldBlock;
    facets: Movable, Breakable;
}

block! {
    /// `emerald_ore` — Emerald Ore. `Movable | Breakable`
    pub struct EmeraldOre;
    facets: Movable, Breakable;
}

block! {
    /// `enchanting_table` — Enchanting Table. `UnMovable | Breakable`
    pub struct EnchantingTable;
    facets: UnMovable, Breakable;
}

block! {
    /// `end_gateway` — End Gateway. `UnMovable`
    pub struct EndGateway;
    facets: UnMovable;
}

block! {
    /// `end_portal` — End Portal. `UnMovable`
    pub struct EndPortal;
    facets: UnMovable;
}

block! {
    /// `end_portal_frame` — End Portal Frame. `UnMovable`
    pub struct EndPortalFrame;
    facets: UnMovable;
    state: EndPortalFrameState;
}

block! {
    /// `end_rod` — End Rod. `Movable | InstantMinable`
    pub struct EndRod;
    facets: Movable, InstantMinable;
    state: ShulkerBoxState;
}

block! {
    /// `end_stone` — End Stone. `Movable | Breakable`
    pub struct EndStone;
    facets: Movable, Breakable;
}

block! {
    /// `end_stone_brick_slab` — End Stone Brick Slab. `Movable | Breakable`
    pub struct EndStoneBrickSlab;
    facets: Movable, Breakable;
    state: SlabState;
}

block! {
    /// `end_stone_brick_stairs` — End Stone Brick Stairs. `Movable | Breakable`
    pub struct EndStoneBrickStairs;
    facets: Movable, Breakable;
    state: StairState;
}

block! {
    /// `end_stone_brick_wall` — End Stone Brick Wall. `Movable | Breakable`
    pub struct EndStoneBrickWall;
    facets: Movable, Breakable;
    state: WallState;
}

block! {
    /// `end_stone_bricks` — End Stone Bricks. `Movable | Breakable`
    pub struct EndStoneBricks;
    facets: Movable, Breakable;
}

block! {
    /// `ender_chest` — Ender Chest. `UnMovable | Breakable`
    pub struct EnderChest;
    facets: UnMovable, Breakable;
    state: WallMountedState;
}

block! {
    /// `exposed_chiseled_copper` — Exposed Chiseled Copper. `Movable | Breakable`
    pub struct ExposedChiseledCopper;
    facets: Movable, Breakable;
}

block! {
    /// `exposed_copper` — Exposed Copper. `Movable | Breakable`
    pub struct ExposedCopper;
    facets: Movable, Breakable;
}

block! {
    /// `exposed_copper_bars` — Exposed Copper Bars. `Movable | Breakable`
    pub struct ExposedCopperBars;
    facets: Movable, Breakable;
    state: ConnectionState;
}

block! {
    /// `exposed_copper_bulb` — Exposed Copper Bulb. `Movable | Breakable`
    pub struct ExposedCopperBulb;
    facets: Movable, Breakable;
    state: WeatheringCopperBulbState;
}

block! {
    /// `exposed_copper_chain` — Exposed Copper Chain. `Movable | Breakable`
    pub struct ExposedCopperChain;
    facets: Movable, Breakable;
    state: WeatheringCopperChainState;
}

block! {
    /// `exposed_copper_chest` — Exposed Copper Chest. `UnMovable | Breakable`
    pub struct ExposedCopperChest;
    facets: UnMovable, Breakable;
    state: ChestState;
}

block! {
    /// `exposed_copper_door` — Exposed Copper Door. `Breakable`
    pub struct ExposedCopperDoor;
    facets: Breakable;
    state: DoorState;
}

block! {
    /// `exposed_copper_golem_statue` — Exposed Copper Golem Statue. `Movable | Breakable`
    pub struct ExposedCopperGolemStatue;
    facets: Movable, Breakable;
    state: WeatheringCopperGolemStatueState;
}

block! {
    /// `exposed_copper_grate` — Exposed Copper Grate. `Movable | Breakable`
    pub struct ExposedCopperGrate;
    facets: Movable, Breakable;
    state: WaterloggedState;
}

block! {
    /// `exposed_copper_lantern` — Exposed Copper Lantern. `Breakable`
    pub struct ExposedCopperLantern;
    facets: Breakable;
    state: WeatheringLanternState;
}

block! {
    /// `exposed_copper_trapdoor` — Exposed Copper Trapdoor. `Movable | Breakable`
    pub struct ExposedCopperTrapdoor;
    facets: Movable, Breakable;
    state: TrapdoorState;
}

block! {
    /// `exposed_cut_copper` — Exposed Cut Copper. `Movable | Breakable`
    pub struct ExposedCutCopper;
    facets: Movable, Breakable;
}

block! {
    /// `exposed_cut_copper_slab` — Exposed Cut Copper Slab. `Movable | Breakable`
    pub struct ExposedCutCopperSlab;
    facets: Movable, Breakable;
    state: SlabState;
}

block! {
    /// `exposed_cut_copper_stairs` — Exposed Cut Copper Stairs. `Movable | Breakable`
    pub struct ExposedCutCopperStairs;
    facets: Movable, Breakable;
    state: StairState;
}

block! {
    /// `exposed_lightning_rod` — Exposed Lightning Rod. `Movable | Breakable`
    pub struct ExposedLightningRod;
    facets: Movable, Breakable;
    state: WeatheringLightningRodState;
}

block! {
    /// `farmland` — Farmland. `Movable | Breakable`
    pub struct Farmland;
    facets: Movable, Breakable;
    state: FarmlandState;
}

block! {
    /// `fern` — Fern. `InstantMinable | Flammable`
    pub struct Fern;
    facets: InstantMinable, Flammable;
}

block! {
    /// `fire` — Fire. `InstantMinable`
    pub struct Fire;
    facets: InstantMinable;
    state: FireState;
}

block! {
    /// `fire_coral` — Fire Coral. `Movable | InstantMinable`
    pub struct FireCoral;
    facets: Movable, InstantMinable;
    state: WaterloggedState;
}

block! {
    /// `fire_coral_block` — Fire Coral Block. `Movable | Breakable`
    pub struct FireCoralBlock;
    facets: Movable, Breakable;
}

block! {
    /// `fire_coral_fan` — Fire Coral Fan. `Movable | InstantMinable`
    pub struct FireCoralFan;
    facets: Movable, InstantMinable;
    state: WaterloggedState;
}

block! {
    /// `fire_coral_wall_fan` — Fire Coral Wall Fan. `Movable | InstantMinable`
    pub struct FireCoralWallFan;
    facets: Movable, InstantMinable;
    state: WallMountedState;
}

block! {
    /// `firefly_bush` — Firefly Bush. `InstantMinable | Flammable`
    pub struct FireflyBush;
    facets: InstantMinable, Flammable;
}

block! {
    /// `fletching_table` — Fletching Table. `Movable | Breakable`
    pub struct FletchingTable;
    facets: Movable, Breakable;
}

block! {
    /// `flower_pot` — Flower Pot. `InstantMinable`
    pub struct FlowerPot;
    facets: InstantMinable;
}

block! {
    /// `flowering_azalea` — Flowering Azalea. `Movable | InstantMinable | Flammable`
    pub struct FloweringAzalea;
    facets: Movable, InstantMinable, Flammable;
}

block! {
    /// `flowering_azalea_leaves` — Flowering Azalea Leaves. `Movable | Breakable | Flammable`
    pub struct FloweringAzaleaLeaves;
    facets: Movable, Breakable, Flammable;
    state: LeavesState;
}

block! {
    /// `frogspawn` — Frogspawn. `InstantMinable`
    pub struct Frogspawn;
    facets: InstantMinable;
}

block! {
    /// `frosted_ice` — Frosted Ice. `Breakable`
    pub struct FrostedIce;
    facets: Breakable;
    state: BeetrootState;
}

block! {
    /// `furnace` — Furnace. `UnMovable | Breakable`
    pub struct Furnace;
    facets: UnMovable, Breakable;
    state: BlastFurnaceState;
}

block! {
    /// `gilded_blackstone` — Gilded Blackstone. `Movable | Breakable`
    pub struct GildedBlackstone;
    facets: Movable, Breakable;
}

block! {
    /// `glass` — Glass. `Movable | Breakable`
    pub struct Glass;
    facets: Movable, Breakable;
}

block! {
    /// `glass_pane` — Glass Pane. `Movable | Breakable`
    pub struct GlassPane;
    facets: Movable, Breakable;
    state: ConnectionState;
}

block! {
    /// `glow_lichen` — Glow Lichen. `Breakable | Flammable`
    pub struct GlowLichen;
    facets: Breakable, Flammable;
    state: GlowLichenState;
}

block! {
    /// `glowstone` — Glowstone. `Movable | Breakable`
    pub struct Glowstone;
    facets: Movable, Breakable;
}

block! {
    /// `gold_block` — Block of Gold. `Movable | Breakable`
    pub struct GoldBlock;
    facets: Movable, Breakable;
}

block! {
    /// `gold_ore` — Gold Ore. `Movable | Breakable`
    pub struct GoldOre;
    facets: Movable, Breakable;
}

block! {
    /// `golden_dandelion` — Golden Dandelion. `InstantMinable`
    pub struct GoldenDandelion;
    facets: InstantMinable;
}

block! {
    /// `granite` — Granite. `Movable | Breakable`
    pub struct Granite;
    facets: Movable, Breakable;
}

block! {
    /// `granite_slab` — Granite Slab. `Movable | Breakable`
    pub struct GraniteSlab;
    facets: Movable, Breakable;
    state: SlabState;
}

block! {
    /// `granite_stairs` — Granite Stairs. `Movable | Breakable`
    pub struct GraniteStairs;
    facets: Movable, Breakable;
    state: StairState;
}

block! {
    /// `granite_wall` — Granite Wall. `Movable | Breakable`
    pub struct GraniteWall;
    facets: Movable, Breakable;
    state: WallState;
}

block! {
    /// `grass_block` — Grass Block. `Breakable`
    pub struct GrassBlock;
    facets: Breakable;
    state: SnowyState;
}

block! {
    /// `gravel` — Gravel. `Movable | Breakable`
    pub struct Gravel;
    facets: Movable, Breakable;
}

block! {
    /// `gray_banner` — Gray Banner. `Breakable`
    pub struct GrayBanner;
    facets: Breakable;
    state: BannerState;
}

block! {
    /// `gray_bed` — Gray Bed. `Breakable`
    pub struct GrayBed;
    facets: Breakable;
    state: BedState;
}

block! {
    /// `gray_candle` — Gray Candle. `Movable | Breakable`
    pub struct GrayCandle;
    facets: Movable, Breakable;
    state: CandleState;
}

block! {
    /// `gray_candle_cake` — Cake with Gray Candle. `Movable | Breakable`
    pub struct GrayCandleCake;
    facets: Movable, Breakable;
    state: LitState;
}

block! {
    /// `gray_carpet` — Gray Carpet. `Movable | Breakable | Flammable`
    pub struct GrayCarpet;
    facets: Movable, Breakable, Flammable;
}

block! {
    /// `gray_concrete` — Gray Concrete. `Movable | Breakable`
    pub struct GrayConcrete;
    facets: Movable, Breakable;
}

block! {
    /// `gray_concrete_powder` — Gray Concrete Powder. `Movable | Breakable`
    pub struct GrayConcretePowder;
    facets: Movable, Breakable;
}

block! {
    /// `gray_glazed_terracotta` — Gray Glazed Terracotta. `Movable | Breakable`
    pub struct GrayGlazedTerracotta;
    facets: Movable, Breakable;
    state: FacingState;
}

block! {
    /// `gray_shulker_box` — Gray Shulker Box. `Breakable`
    pub struct GrayShulkerBox;
    facets: Breakable;
    state: ShulkerBoxState;
}

block! {
    /// `gray_stained_glass` — Gray Stained Glass. `Movable | Breakable`
    pub struct GrayStainedGlass;
    facets: Movable, Breakable;
}

block! {
    /// `gray_stained_glass_pane` — Gray Stained Glass Pane. `Movable | Breakable`
    pub struct GrayStainedGlassPane;
    facets: Movable, Breakable;
    state: ConnectionState;
}

block! {
    /// `gray_terracotta` — Gray Terracotta. `Movable | Breakable`
    pub struct GrayTerracotta;
    facets: Movable, Breakable;
}

block! {
    /// `gray_wall_banner` — Gray Banner. `Breakable`
    pub struct GrayWallBanner;
    facets: Breakable;
    state: FacingState;
}

block! {
    /// `gray_wool` — Gray Wool. `Movable | Breakable | Flammable`
    pub struct GrayWool;
    facets: Movable, Breakable, Flammable;
}

block! {
    /// `green_banner` — Green Banner. `Breakable`
    pub struct GreenBanner;
    facets: Breakable;
    state: BannerState;
}

block! {
    /// `green_bed` — Green Bed. `Breakable`
    pub struct GreenBed;
    facets: Breakable;
    state: BedState;
}

block! {
    /// `green_candle` — Green Candle. `Movable | Breakable`
    pub struct GreenCandle;
    facets: Movable, Breakable;
    state: CandleState;
}

block! {
    /// `green_candle_cake` — Cake with Green Candle. `Movable | Breakable`
    pub struct GreenCandleCake;
    facets: Movable, Breakable;
    state: LitState;
}

block! {
    /// `green_carpet` — Green Carpet. `Movable | Breakable | Flammable`
    pub struct GreenCarpet;
    facets: Movable, Breakable, Flammable;
}

block! {
    /// `green_concrete` — Green Concrete. `Movable | Breakable`
    pub struct GreenConcrete;
    facets: Movable, Breakable;
}

block! {
    /// `green_concrete_powder` — Green Concrete Powder. `Movable | Breakable`
    pub struct GreenConcretePowder;
    facets: Movable, Breakable;
}

block! {
    /// `green_glazed_terracotta` — Green Glazed Terracotta. `Movable | Breakable`
    pub struct GreenGlazedTerracotta;
    facets: Movable, Breakable;
    state: FacingState;
}

block! {
    /// `green_shulker_box` — Green Shulker Box. `Breakable`
    pub struct GreenShulkerBox;
    facets: Breakable;
    state: ShulkerBoxState;
}

block! {
    /// `green_stained_glass` — Green Stained Glass. `Movable | Breakable`
    pub struct GreenStainedGlass;
    facets: Movable, Breakable;
}

block! {
    /// `green_stained_glass_pane` — Green Stained Glass Pane. `Movable | Breakable`
    pub struct GreenStainedGlassPane;
    facets: Movable, Breakable;
    state: ConnectionState;
}

block! {
    /// `green_terracotta` — Green Terracotta. `Movable | Breakable`
    pub struct GreenTerracotta;
    facets: Movable, Breakable;
}

block! {
    /// `green_wall_banner` — Green Banner. `Breakable`
    pub struct GreenWallBanner;
    facets: Breakable;
    state: FacingState;
}

block! {
    /// `green_wool` — Green Wool. `Movable | Breakable | Flammable`
    pub struct GreenWool;
    facets: Movable, Breakable, Flammable;
}

block! {
    /// `grindstone` — Grindstone. `UnMovable | Breakable`
    pub struct Grindstone;
    facets: UnMovable, Breakable;
    state: GrindstoneState;
}

block! {
    /// `hanging_roots` — Hanging Roots. `Movable | InstantMinable | Flammable`
    pub struct HangingRoots;
    facets: Movable, InstantMinable, Flammable;
    state: WaterloggedState;
}

block! {
    /// `hay_block` — Hay Bale. `Movable | Breakable | Flammable`
    pub struct HayBlock;
    facets: Movable, Breakable, Flammable;
    state: PillarState;
}

block! {
    /// `heavy_core` — Heavy Core. `Movable | Breakable`
    pub struct HeavyCore;
    facets: Movable, Breakable;
    state: WaterloggedState;
}

block! {
    /// `heavy_weighted_pressure_plate` — Heavy Weighted Pressure Plate. `Breakable`
    pub struct HeavyWeightedPressurePlate;
    facets: Breakable;
    state: WeightedPressurePlateState;
}

block! {
    /// `honey_block` — Honey Block. `Sticky | Movable | InstantMinable`
    pub struct HoneyBlock;
    facets: Sticky, Movable, InstantMinable;
}

block! {
    /// `honeycomb_block` — Honeycomb Block. `Movable | Breakable`
    pub struct HoneycombBlock;
    facets: Movable, Breakable;
}

block! {
    /// `hopper` — Hopper. `UnMovable | Breakable`
    pub struct Hopper;
    facets: UnMovable, Breakable;
    state: HopperState;
}

block! {
    /// `horn_coral` — Horn Coral. `Movable | InstantMinable`
    pub struct HornCoral;
    facets: Movable, InstantMinable;
    state: WaterloggedState;
}

block! {
    /// `horn_coral_block` — Horn Coral Block. `Movable | Breakable`
    pub struct HornCoralBlock;
    facets: Movable, Breakable;
}

block! {
    /// `horn_coral_fan` — Horn Coral Fan. `Movable | InstantMinable`
    pub struct HornCoralFan;
    facets: Movable, InstantMinable;
    state: WaterloggedState;
}

block! {
    /// `horn_coral_wall_fan` — Horn Coral Wall Fan. `Movable | InstantMinable`
    pub struct HornCoralWallFan;
    facets: Movable, InstantMinable;
    state: WallMountedState;
}

block! {
    /// `ice` — Ice. `Movable | Breakable`
    pub struct Ice;
    facets: Movable, Breakable;
}

block! {
    /// `infested_chiseled_stone_bricks` — Infested Chiseled Stone Bricks. `Movable | Breakable`
    pub struct InfestedChiseledStoneBricks;
    facets: Movable, Breakable;
}

block! {
    /// `infested_cobblestone` — Infested Cobblestone. `Movable | Breakable`
    pub struct InfestedCobblestone;
    facets: Movable, Breakable;
}

block! {
    /// `infested_cracked_stone_bricks` — Infested Cracked Stone Bricks. `Movable | Breakable`
    pub struct InfestedCrackedStoneBricks;
    facets: Movable, Breakable;
}

block! {
    /// `infested_deepslate` — Infested Deepslate. `Movable | Breakable`
    pub struct InfestedDeepslate;
    facets: Movable, Breakable;
    state: PillarState;
}

block! {
    /// `infested_mossy_stone_bricks` — Infested Mossy Stone Bricks. `Movable | Breakable`
    pub struct InfestedMossyStoneBricks;
    facets: Movable, Breakable;
}

block! {
    /// `infested_stone` — Infested Stone. `Movable | Breakable`
    pub struct InfestedStone;
    facets: Movable, Breakable;
}

block! {
    /// `infested_stone_bricks` — Infested Stone Bricks. `Movable | Breakable`
    pub struct InfestedStoneBricks;
    facets: Movable, Breakable;
}

block! {
    /// `iron_bars` — Iron Bars. `Movable | Breakable`
    pub struct IronBars;
    facets: Movable, Breakable;
    state: ConnectionState;
}

block! {
    /// `iron_block` — Block of Iron. `Movable | Breakable`
    pub struct IronBlock;
    facets: Movable, Breakable;
}

block! {
    /// `iron_chain` — Iron Chain. `Movable | Breakable`
    pub struct IronChain;
    facets: Movable, Breakable;
    state: WeatheringCopperChainState;
}

block! {
    /// `iron_door` — Iron Door. `Breakable`
    pub struct IronDoor;
    facets: Breakable;
    state: DoorState;
}

block! {
    /// `iron_ore` — Iron Ore. `Movable | Breakable`
    pub struct IronOre;
    facets: Movable, Breakable;
}

block! {
    /// `iron_trapdoor` — Iron Trapdoor. `Movable | Breakable`
    pub struct IronTrapdoor;
    facets: Movable, Breakable;
    state: TrapdoorState;
}

block! {
    /// `jack_o_lantern` — Jack o'Lantern. `Breakable`
    pub struct JackOLantern;
    facets: Breakable;
    state: FacingState;
}

block! {
    /// `jigsaw` — Jigsaw Block. `UnMovable`
    pub struct Jigsaw;
    facets: UnMovable;
    state: JigsawState;
}

block! {
    /// `jukebox` — Jukebox. `UnMovable | Breakable`
    pub struct Jukebox;
    facets: UnMovable, Breakable;
    state: JukeboxState;
}

block! {
    /// `jungle_button` — Jungle Button. `Breakable | Flammable`
    pub struct JungleButton;
    facets: Breakable, Flammable;
    state: SwitchState;
}

block! {
    /// `jungle_door` — Jungle Door. `Breakable | Flammable`
    pub struct JungleDoor;
    facets: Breakable, Flammable;
    state: DoorState;
}

block! {
    /// `jungle_fence` — Jungle Fence. `Movable | Breakable | Flammable`
    pub struct JungleFence;
    facets: Movable, Breakable, Flammable;
    state: ConnectionState;
}

block! {
    /// `jungle_fence_gate` — Jungle Fence Gate. `Movable | Breakable | Flammable`
    pub struct JungleFenceGate;
    facets: Movable, Breakable, Flammable;
    state: FenceGateState;
}

block! {
    /// `jungle_hanging_sign` — Jungle Hanging Sign. `Breakable | Flammable`
    pub struct JungleHangingSign;
    facets: Breakable, Flammable;
    state: HangingSignState;
}

block! {
    /// `jungle_leaves` — Jungle Leaves. `Movable | Breakable | Flammable`
    pub struct JungleLeaves;
    facets: Movable, Breakable, Flammable;
    state: LeavesState;
}

block! {
    /// `jungle_log` — Jungle Log. `Movable | Breakable | Flammable`
    pub struct JungleLog;
    facets: Movable, Breakable, Flammable;
    state: PillarState;
}

block! {
    /// `jungle_planks` — Jungle Planks. `Movable | Breakable | Flammable`
    pub struct JunglePlanks;
    facets: Movable, Breakable, Flammable;
}

block! {
    /// `jungle_pressure_plate` — Jungle Pressure Plate. `Breakable | Flammable`
    pub struct JunglePressurePlate;
    facets: Breakable, Flammable;
    state: PressurePlateState;
}

block! {
    /// `jungle_sapling` — Jungle Sapling. `InstantMinable`
    pub struct JungleSapling;
    facets: InstantMinable;
    state: SaplingState;
}

block! {
    /// `jungle_shelf` — Jungle Shelf. `UnMovable | Breakable`
    pub struct JungleShelf;
    facets: UnMovable, Breakable;
    state: ShelfState;
}

block! {
    /// `jungle_sign` — Jungle Sign. `Breakable | Flammable`
    pub struct JungleSign;
    facets: Breakable, Flammable;
    state: StandingSignState;
}

block! {
    /// `jungle_slab` — Jungle Slab. `Movable | Breakable | Flammable`
    pub struct JungleSlab;
    facets: Movable, Breakable, Flammable;
    state: SlabState;
}

block! {
    /// `jungle_stairs` — Jungle Stairs. `Movable | Breakable | Flammable`
    pub struct JungleStairs;
    facets: Movable, Breakable, Flammable;
    state: StairState;
}

block! {
    /// `jungle_trapdoor` — Jungle Trapdoor. `Movable | Breakable | Flammable`
    pub struct JungleTrapdoor;
    facets: Movable, Breakable, Flammable;
    state: TrapdoorState;
}

block! {
    /// `jungle_wall_hanging_sign` — Jungle Hanging Sign. `Breakable | Flammable`
    pub struct JungleWallHangingSign;
    facets: Breakable, Flammable;
    state: WallMountedState;
}

block! {
    /// `jungle_wall_sign` — Jungle Sign. `Breakable | Flammable`
    pub struct JungleWallSign;
    facets: Breakable, Flammable;
    state: WallMountedState;
}

block! {
    /// `jungle_wood` — Jungle Wood. `Movable | Breakable | Flammable`
    pub struct JungleWood;
    facets: Movable, Breakable, Flammable;
    state: PillarState;
}

block! {
    /// `kelp` — Kelp. `Movable | InstantMinable`
    pub struct Kelp;
    facets: Movable, InstantMinable;
    state: KelpState;
}

block! {
    /// `kelp_plant` — Kelp Plant. `Movable | InstantMinable`
    pub struct KelpPlant;
    facets: Movable, InstantMinable;
}

block! {
    /// `ladder` — Ladder. `Breakable`
    pub struct Ladder;
    facets: Breakable;
    state: WallMountedState;
}

block! {
    /// `lantern` — Lantern. `Breakable`
    pub struct Lantern;
    facets: Breakable;
    state: WeatheringLanternState;
}

block! {
    /// `lapis_block` — Block of Lapis Lazuli. `Movable | Breakable`
    pub struct LapisBlock;
    facets: Movable, Breakable;
}

block! {
    /// `lapis_ore` — Lapis Lazuli Ore. `Movable | Breakable`
    pub struct LapisOre;
    facets: Movable, Breakable;
}

block! {
    /// `large_amethyst_bud` — Large Amethyst Bud. `Breakable`
    pub struct LargeAmethystBud;
    facets: Breakable;
    state: AmethystClusterState;
}

block! {
    /// `large_fern` — Large Fern. `InstantMinable | Flammable`
    pub struct LargeFern;
    facets: InstantMinable, Flammable;
    state: DoublePlantState;
}

block! {
    /// `lava` — Lava. `UnMovable`
    pub struct Lava;
    facets: UnMovable;
    state: LiquidState;
}

block! {
    /// `lava_cauldron` — Lava Cauldron. `Movable | Breakable`
    pub struct LavaCauldron;
    facets: Movable, Breakable;
}

block! {
    /// `leaf_litter` — Leaf Litter. `InstantMinable | Flammable`
    pub struct LeafLitter;
    facets: InstantMinable, Flammable;
    state: LeafLitterState;
}

block! {
    /// `lectern` — Lectern. `UnMovable | Breakable | Flammable`
    pub struct Lectern;
    facets: UnMovable, Breakable, Flammable;
    state: LecternState;
}

block! {
    /// `lever` — Lever. `Breakable`
    pub struct Lever;
    facets: Breakable;
    state: SwitchState;
}

block! {
    /// `light` — Light. `UnMovable`
    pub struct Light;
    facets: UnMovable;
    state: LightState;
}

block! {
    /// `light_blue_banner` — Light Blue Banner. `Breakable`
    pub struct LightBlueBanner;
    facets: Breakable;
    state: BannerState;
}

block! {
    /// `light_blue_bed` — Light Blue Bed. `Breakable`
    pub struct LightBlueBed;
    facets: Breakable;
    state: BedState;
}

block! {
    /// `light_blue_candle` — Light Blue Candle. `Movable | Breakable`
    pub struct LightBlueCandle;
    facets: Movable, Breakable;
    state: CandleState;
}

block! {
    /// `light_blue_candle_cake` — Cake with Light Blue Candle. `Movable | Breakable`
    pub struct LightBlueCandleCake;
    facets: Movable, Breakable;
    state: LitState;
}

block! {
    /// `light_blue_carpet` — Light Blue Carpet. `Movable | Breakable | Flammable`
    pub struct LightBlueCarpet;
    facets: Movable, Breakable, Flammable;
}

block! {
    /// `light_blue_concrete` — Light Blue Concrete. `Movable | Breakable`
    pub struct LightBlueConcrete;
    facets: Movable, Breakable;
}

block! {
    /// `light_blue_concrete_powder` — Light Blue Concrete Powder. `Movable | Breakable`
    pub struct LightBlueConcretePowder;
    facets: Movable, Breakable;
}

block! {
    /// `light_blue_glazed_terracotta` — Light Blue Glazed Terracotta. `Movable | Breakable`
    pub struct LightBlueGlazedTerracotta;
    facets: Movable, Breakable;
    state: FacingState;
}

block! {
    /// `light_blue_shulker_box` — Light Blue Shulker Box. `Breakable`
    pub struct LightBlueShulkerBox;
    facets: Breakable;
    state: ShulkerBoxState;
}

block! {
    /// `light_blue_stained_glass` — Light Blue Stained Glass. `Movable | Breakable`
    pub struct LightBlueStainedGlass;
    facets: Movable, Breakable;
}

block! {
    /// `light_blue_stained_glass_pane` — Light Blue Stained Glass Pane. `Movable | Breakable`
    pub struct LightBlueStainedGlassPane;
    facets: Movable, Breakable;
    state: ConnectionState;
}

block! {
    /// `light_blue_terracotta` — Light Blue Terracotta. `Movable | Breakable`
    pub struct LightBlueTerracotta;
    facets: Movable, Breakable;
}

block! {
    /// `light_blue_wall_banner` — Light Blue Banner. `Breakable`
    pub struct LightBlueWallBanner;
    facets: Breakable;
    state: FacingState;
}

block! {
    /// `light_blue_wool` — Light Blue Wool. `Movable | Breakable | Flammable`
    pub struct LightBlueWool;
    facets: Movable, Breakable, Flammable;
}

block! {
    /// `light_gray_banner` — Light Gray Banner. `Breakable`
    pub struct LightGrayBanner;
    facets: Breakable;
    state: BannerState;
}

block! {
    /// `light_gray_bed` — Light Gray Bed. `Breakable`
    pub struct LightGrayBed;
    facets: Breakable;
    state: BedState;
}

block! {
    /// `light_gray_candle` — Light Gray Candle. `Movable | Breakable`
    pub struct LightGrayCandle;
    facets: Movable, Breakable;
    state: CandleState;
}

block! {
    /// `light_gray_candle_cake` — Cake with Light Gray Candle. `Movable | Breakable`
    pub struct LightGrayCandleCake;
    facets: Movable, Breakable;
    state: LitState;
}

block! {
    /// `light_gray_carpet` — Light Gray Carpet. `Movable | Breakable | Flammable`
    pub struct LightGrayCarpet;
    facets: Movable, Breakable, Flammable;
}

block! {
    /// `light_gray_concrete` — Light Gray Concrete. `Movable | Breakable`
    pub struct LightGrayConcrete;
    facets: Movable, Breakable;
}

block! {
    /// `light_gray_concrete_powder` — Light Gray Concrete Powder. `Movable | Breakable`
    pub struct LightGrayConcretePowder;
    facets: Movable, Breakable;
}

block! {
    /// `light_gray_glazed_terracotta` — Light Gray Glazed Terracotta. `Movable | Breakable`
    pub struct LightGrayGlazedTerracotta;
    facets: Movable, Breakable;
    state: FacingState;
}

block! {
    /// `light_gray_shulker_box` — Light Gray Shulker Box. `Breakable`
    pub struct LightGrayShulkerBox;
    facets: Breakable;
    state: ShulkerBoxState;
}

block! {
    /// `light_gray_stained_glass` — Light Gray Stained Glass. `Movable | Breakable`
    pub struct LightGrayStainedGlass;
    facets: Movable, Breakable;
}

block! {
    /// `light_gray_stained_glass_pane` — Light Gray Stained Glass Pane. `Movable | Breakable`
    pub struct LightGrayStainedGlassPane;
    facets: Movable, Breakable;
    state: ConnectionState;
}

block! {
    /// `light_gray_terracotta` — Light Gray Terracotta. `Movable | Breakable`
    pub struct LightGrayTerracotta;
    facets: Movable, Breakable;
}

block! {
    /// `light_gray_wall_banner` — Light Gray Banner. `Breakable`
    pub struct LightGrayWallBanner;
    facets: Breakable;
    state: FacingState;
}

block! {
    /// `light_gray_wool` — Light Gray Wool. `Movable | Breakable | Flammable`
    pub struct LightGrayWool;
    facets: Movable, Breakable, Flammable;
}

block! {
    /// `light_weighted_pressure_plate` — Light Weighted Pressure Plate. `Breakable`
    pub struct LightWeightedPressurePlate;
    facets: Breakable;
    state: WeightedPressurePlateState;
}

block! {
    /// `lightning_rod` — Lightning Rod. `Movable | Breakable`
    pub struct LightningRod;
    facets: Movable, Breakable;
    state: WeatheringLightningRodState;
}

block! {
    /// `lilac` — Lilac. `InstantMinable | Flammable`
    pub struct Lilac;
    facets: InstantMinable, Flammable;
    state: DoublePlantState;
}

block! {
    /// `lily_of_the_valley` — Lily of the Valley. `InstantMinable | Flammable`
    pub struct LilyOfTheValley;
    facets: InstantMinable, Flammable;
}

block! {
    /// `lily_pad` — Lily Pad. `InstantMinable`
    pub struct LilyPad;
    facets: InstantMinable;
}

block! {
    /// `lime_banner` — Lime Banner. `Breakable`
    pub struct LimeBanner;
    facets: Breakable;
    state: BannerState;
}

block! {
    /// `lime_bed` — Lime Bed. `Breakable`
    pub struct LimeBed;
    facets: Breakable;
    state: BedState;
}

block! {
    /// `lime_candle` — Lime Candle. `Movable | Breakable`
    pub struct LimeCandle;
    facets: Movable, Breakable;
    state: CandleState;
}

block! {
    /// `lime_candle_cake` — Cake with Lime Candle. `Movable | Breakable`
    pub struct LimeCandleCake;
    facets: Movable, Breakable;
    state: LitState;
}

block! {
    /// `lime_carpet` — Lime Carpet. `Movable | Breakable | Flammable`
    pub struct LimeCarpet;
    facets: Movable, Breakable, Flammable;
}

block! {
    /// `lime_concrete` — Lime Concrete. `Movable | Breakable`
    pub struct LimeConcrete;
    facets: Movable, Breakable;
}

block! {
    /// `lime_concrete_powder` — Lime Concrete Powder. `Movable | Breakable`
    pub struct LimeConcretePowder;
    facets: Movable, Breakable;
}

block! {
    /// `lime_glazed_terracotta` — Lime Glazed Terracotta. `Movable | Breakable`
    pub struct LimeGlazedTerracotta;
    facets: Movable, Breakable;
    state: FacingState;
}

block! {
    /// `lime_shulker_box` — Lime Shulker Box. `Breakable`
    pub struct LimeShulkerBox;
    facets: Breakable;
    state: ShulkerBoxState;
}

block! {
    /// `lime_stained_glass` — Lime Stained Glass. `Movable | Breakable`
    pub struct LimeStainedGlass;
    facets: Movable, Breakable;
}

block! {
    /// `lime_stained_glass_pane` — Lime Stained Glass Pane. `Movable | Breakable`
    pub struct LimeStainedGlassPane;
    facets: Movable, Breakable;
    state: ConnectionState;
}

block! {
    /// `lime_terracotta` — Lime Terracotta. `Movable | Breakable`
    pub struct LimeTerracotta;
    facets: Movable, Breakable;
}

block! {
    /// `lime_wall_banner` — Lime Banner. `Breakable`
    pub struct LimeWallBanner;
    facets: Breakable;
    state: FacingState;
}

block! {
    /// `lime_wool` — Lime Wool. `Movable | Breakable | Flammable`
    pub struct LimeWool;
    facets: Movable, Breakable, Flammable;
}

block! {
    /// `lodestone` — Lodestone. `UnMovable | Breakable`
    pub struct Lodestone;
    facets: UnMovable, Breakable;
}

block! {
    /// `loom` — Loom. `Movable | Breakable`
    pub struct Loom;
    facets: Movable, Breakable;
    state: FacingState;
}

block! {
    /// `magenta_banner` — Magenta Banner. `Breakable`
    pub struct MagentaBanner;
    facets: Breakable;
    state: BannerState;
}

block! {
    /// `magenta_bed` — Magenta Bed. `Breakable`
    pub struct MagentaBed;
    facets: Breakable;
    state: BedState;
}

block! {
    /// `magenta_candle` — Magenta Candle. `Movable | Breakable`
    pub struct MagentaCandle;
    facets: Movable, Breakable;
    state: CandleState;
}

block! {
    /// `magenta_candle_cake` — Cake with Magenta Candle. `Movable | Breakable`
    pub struct MagentaCandleCake;
    facets: Movable, Breakable;
    state: LitState;
}

block! {
    /// `magenta_carpet` — Magenta Carpet. `Movable | Breakable | Flammable`
    pub struct MagentaCarpet;
    facets: Movable, Breakable, Flammable;
}

block! {
    /// `magenta_concrete` — Magenta Concrete. `Movable | Breakable`
    pub struct MagentaConcrete;
    facets: Movable, Breakable;
}

block! {
    /// `magenta_concrete_powder` — Magenta Concrete Powder. `Movable | Breakable`
    pub struct MagentaConcretePowder;
    facets: Movable, Breakable;
}

block! {
    /// `magenta_glazed_terracotta` — Magenta Glazed Terracotta. `Movable | Breakable`
    pub struct MagentaGlazedTerracotta;
    facets: Movable, Breakable;
    state: FacingState;
}

block! {
    /// `magenta_shulker_box` — Magenta Shulker Box. `Breakable`
    pub struct MagentaShulkerBox;
    facets: Breakable;
    state: ShulkerBoxState;
}

block! {
    /// `magenta_stained_glass` — Magenta Stained Glass. `Movable | Breakable`
    pub struct MagentaStainedGlass;
    facets: Movable, Breakable;
}

block! {
    /// `magenta_stained_glass_pane` — Magenta Stained Glass Pane. `Movable | Breakable`
    pub struct MagentaStainedGlassPane;
    facets: Movable, Breakable;
    state: ConnectionState;
}

block! {
    /// `magenta_terracotta` — Magenta Terracotta. `Movable | Breakable`
    pub struct MagentaTerracotta;
    facets: Movable, Breakable;
}

block! {
    /// `magenta_wall_banner` — Magenta Banner. `Breakable`
    pub struct MagentaWallBanner;
    facets: Breakable;
    state: FacingState;
}

block! {
    /// `magenta_wool` — Magenta Wool. `Movable | Breakable | Flammable`
    pub struct MagentaWool;
    facets: Movable, Breakable, Flammable;
}

block! {
    /// `magma_block` — Magma Block. `Movable | Breakable`
    pub struct MagmaBlock;
    facets: Movable, Breakable;
}

block! {
    /// `mangrove_button` — Mangrove Button. `Breakable | Flammable`
    pub struct MangroveButton;
    facets: Breakable, Flammable;
    state: SwitchState;
}

block! {
    /// `mangrove_door` — Mangrove Door. `Breakable | Flammable`
    pub struct MangroveDoor;
    facets: Breakable, Flammable;
    state: DoorState;
}

block! {
    /// `mangrove_fence` — Mangrove Fence. `Movable | Breakable | Flammable`
    pub struct MangroveFence;
    facets: Movable, Breakable, Flammable;
    state: ConnectionState;
}

block! {
    /// `mangrove_fence_gate` — Mangrove Fence Gate. `Movable | Breakable | Flammable`
    pub struct MangroveFenceGate;
    facets: Movable, Breakable, Flammable;
    state: FenceGateState;
}

block! {
    /// `mangrove_hanging_sign` — Mangrove Hanging Sign. `Breakable | Flammable`
    pub struct MangroveHangingSign;
    facets: Breakable, Flammable;
    state: HangingSignState;
}

block! {
    /// `mangrove_leaves` — Mangrove Leaves. `Movable | Breakable | Flammable`
    pub struct MangroveLeaves;
    facets: Movable, Breakable, Flammable;
    state: LeavesState;
}

block! {
    /// `mangrove_log` — Mangrove Log. `Movable | Breakable | Flammable`
    pub struct MangroveLog;
    facets: Movable, Breakable, Flammable;
    state: PillarState;
}

block! {
    /// `mangrove_planks` — Mangrove Planks. `Movable | Breakable | Flammable`
    pub struct MangrovePlanks;
    facets: Movable, Breakable, Flammable;
}

block! {
    /// `mangrove_pressure_plate` — Mangrove Pressure Plate. `Breakable | Flammable`
    pub struct MangrovePressurePlate;
    facets: Breakable, Flammable;
    state: PressurePlateState;
}

block! {
    /// `mangrove_propagule` — Mangrove Propagule. `InstantMinable`
    pub struct MangrovePropagule;
    facets: InstantMinable;
    state: MangrovePropaguleState;
}

block! {
    /// `mangrove_roots` — Mangrove Roots. `Movable | Breakable | Flammable`
    pub struct MangroveRoots;
    facets: Movable, Breakable, Flammable;
    state: WaterloggedState;
}

block! {
    /// `mangrove_shelf` — Mangrove Shelf. `UnMovable | Breakable`
    pub struct MangroveShelf;
    facets: UnMovable, Breakable;
    state: ShelfState;
}

block! {
    /// `mangrove_sign` — Mangrove Sign. `Breakable | Flammable`
    pub struct MangroveSign;
    facets: Breakable, Flammable;
    state: StandingSignState;
}

block! {
    /// `mangrove_slab` — Mangrove Slab. `Movable | Breakable | Flammable`
    pub struct MangroveSlab;
    facets: Movable, Breakable, Flammable;
    state: SlabState;
}

block! {
    /// `mangrove_stairs` — Mangrove Stairs. `Movable | Breakable | Flammable`
    pub struct MangroveStairs;
    facets: Movable, Breakable, Flammable;
    state: StairState;
}

block! {
    /// `mangrove_trapdoor` — Mangrove Trapdoor. `Movable | Breakable | Flammable`
    pub struct MangroveTrapdoor;
    facets: Movable, Breakable, Flammable;
    state: TrapdoorState;
}

block! {
    /// `mangrove_wall_hanging_sign` — Mangrove Hanging Sign. `Breakable | Flammable`
    pub struct MangroveWallHangingSign;
    facets: Breakable, Flammable;
    state: WallMountedState;
}

block! {
    /// `mangrove_wall_sign` — Mangrove Sign. `Breakable | Flammable`
    pub struct MangroveWallSign;
    facets: Breakable, Flammable;
    state: WallMountedState;
}

block! {
    /// `mangrove_wood` — Mangrove Wood. `Movable | Breakable | Flammable`
    pub struct MangroveWood;
    facets: Movable, Breakable, Flammable;
    state: PillarState;
}

block! {
    /// `medium_amethyst_bud` — Medium Amethyst Bud. `Breakable`
    pub struct MediumAmethystBud;
    facets: Breakable;
    state: AmethystClusterState;
}

block! {
    /// `melon` — Melon. `Breakable`
    pub struct Melon;
    facets: Breakable;
}

block! {
    /// `melon_stem` — Melon Stem. `InstantMinable`
    pub struct MelonStem;
    facets: InstantMinable;
    state: CropAgeState;
}

block! {
    /// `moss_block` — Moss Block. `Breakable`
    pub struct MossBlock;
    facets: Breakable;
}

block! {
    /// `moss_carpet` — Moss Carpet. `Breakable`
    pub struct MossCarpet;
    facets: Breakable;
}

block! {
    /// `mossy_cobblestone` — Mossy Cobblestone. `Movable | Breakable`
    pub struct MossyCobblestone;
    facets: Movable, Breakable;
}

block! {
    /// `mossy_cobblestone_slab` — Mossy Cobblestone Slab. `Movable | Breakable`
    pub struct MossyCobblestoneSlab;
    facets: Movable, Breakable;
    state: SlabState;
}

block! {
    /// `mossy_cobblestone_stairs` — Mossy Cobblestone Stairs. `Movable | Breakable`
    pub struct MossyCobblestoneStairs;
    facets: Movable, Breakable;
    state: StairState;
}

block! {
    /// `mossy_cobblestone_wall` — Mossy Cobblestone Wall. `Movable | Breakable`
    pub struct MossyCobblestoneWall;
    facets: Movable, Breakable;
    state: WallState;
}

block! {
    /// `mossy_stone_brick_slab` — Mossy Stone Brick Slab. `Movable | Breakable`
    pub struct MossyStoneBrickSlab;
    facets: Movable, Breakable;
    state: SlabState;
}

block! {
    /// `mossy_stone_brick_stairs` — Mossy Stone Brick Stairs. `Movable | Breakable`
    pub struct MossyStoneBrickStairs;
    facets: Movable, Breakable;
    state: StairState;
}

block! {
    /// `mossy_stone_brick_wall` — Mossy Stone Brick Wall. `Movable | Breakable`
    pub struct MossyStoneBrickWall;
    facets: Movable, Breakable;
    state: WallState;
}

block! {
    /// `mossy_stone_bricks` — Mossy Stone Bricks. `Movable | Breakable`
    pub struct MossyStoneBricks;
    facets: Movable, Breakable;
}

block! {
    /// `moving_piston` — Moving Piston. `UnMovable`
    pub struct MovingPiston;
    facets: UnMovable;
    state: MovingPistonState;
}

block! {
    /// `mud` — Mud. `Movable | Breakable`
    pub struct Mud;
    facets: Movable, Breakable;
}

block! {
    /// `mud_brick_slab` — Mud Brick Slab. `Movable | Breakable`
    pub struct MudBrickSlab;
    facets: Movable, Breakable;
    state: SlabState;
}

block! {
    /// `mud_brick_stairs` — Mud Brick Stairs. `Movable | Breakable`
    pub struct MudBrickStairs;
    facets: Movable, Breakable;
    state: StairState;
}

block! {
    /// `mud_brick_wall` — Mud Brick Wall. `Movable | Breakable`
    pub struct MudBrickWall;
    facets: Movable, Breakable;
    state: WallState;
}

block! {
    /// `mud_bricks` — Mud Bricks. `Movable | Breakable`
    pub struct MudBricks;
    facets: Movable, Breakable;
}

block! {
    /// `muddy_mangrove_roots` — Muddy Mangrove Roots. `Movable | Breakable`
    pub struct MuddyMangroveRoots;
    facets: Movable, Breakable;
    state: PillarState;
}

block! {
    /// `mushroom_stem` — Mushroom Stem. `Movable | Breakable`
    pub struct MushroomStem;
    facets: Movable, Breakable;
    state: HugeMushroomState;
}

block! {
    /// `mycelium` — Mycelium. `Movable | Breakable`
    pub struct Mycelium;
    facets: Movable, Breakable;
    state: SnowyState;
}

block! {
    /// `nether_brick_fence` — Nether Brick Fence. `Movable | Breakable`
    pub struct NetherBrickFence;
    facets: Movable, Breakable;
    state: ConnectionState;
}

block! {
    /// `nether_brick_slab` — Nether Brick Slab. `Movable | Breakable`
    pub struct NetherBrickSlab;
    facets: Movable, Breakable;
    state: SlabState;
}

block! {
    /// `nether_brick_stairs` — Nether Brick Stairs. `Movable | Breakable`
    pub struct NetherBrickStairs;
    facets: Movable, Breakable;
    state: StairState;
}

block! {
    /// `nether_brick_wall` — Nether Brick Wall. `Movable | Breakable`
    pub struct NetherBrickWall;
    facets: Movable, Breakable;
    state: WallState;
}

block! {
    /// `nether_bricks` — Nether Bricks. `Movable | Breakable`
    pub struct NetherBricks;
    facets: Movable, Breakable;
}

block! {
    /// `nether_gold_ore` — Nether Gold Ore. `Movable | Breakable`
    pub struct NetherGoldOre;
    facets: Movable, Breakable;
}

block! {
    /// `nether_portal` — Nether Portal. `UnMovable`
    pub struct NetherPortal;
    facets: UnMovable;
    state: NetherPortalState;
}

block! {
    /// `nether_quartz_ore` — Nether Quartz Ore. `Movable | Breakable`
    pub struct NetherQuartzOre;
    facets: Movable, Breakable;
}

block! {
    /// `nether_sprouts` — Nether Sprouts. `InstantMinable`
    pub struct NetherSprouts;
    facets: InstantMinable;
}

block! {
    /// `nether_wart` — Nether Wart. `InstantMinable`
    pub struct NetherWart;
    facets: InstantMinable;
    state: BeetrootState;
}

block! {
    /// `nether_wart_block` — Nether Wart Block. `Movable | Breakable`
    pub struct NetherWartBlock;
    facets: Movable, Breakable;
}

block! {
    /// `netherite_block` — Block of Netherite. `Movable | Breakable`
    pub struct NetheriteBlock;
    facets: Movable, Breakable;
}

block! {
    /// `netherrack` — Netherrack. `Movable | Breakable`
    pub struct Netherrack;
    facets: Movable, Breakable;
}

block! {
    /// `note_block` — Note Block. `Movable | Breakable`
    pub struct NoteBlock;
    facets: Movable, Breakable;
    state: NoteState;
}

block! {
    /// `oak_button` — Oak Button. `Breakable | Flammable`
    pub struct OakButton;
    facets: Breakable, Flammable;
    state: SwitchState;
}

block! {
    /// `oak_door` — Oak Door. `Breakable | Flammable`
    pub struct OakDoor;
    facets: Breakable, Flammable;
    state: DoorState;
}

block! {
    /// `oak_fence` — Oak Fence. `Movable | Breakable | Flammable`
    pub struct OakFence;
    facets: Movable, Breakable, Flammable;
    state: ConnectionState;
}

block! {
    /// `oak_fence_gate` — Oak Fence Gate. `Movable | Breakable | Flammable`
    pub struct OakFenceGate;
    facets: Movable, Breakable, Flammable;
    state: FenceGateState;
}

block! {
    /// `oak_hanging_sign` — Oak Hanging Sign. `Breakable | Flammable`
    pub struct OakHangingSign;
    facets: Breakable, Flammable;
    state: HangingSignState;
}

block! {
    /// `oak_leaves` — Oak Leaves. `Movable | Breakable | Flammable`
    pub struct OakLeaves;
    facets: Movable, Breakable, Flammable;
    state: LeavesState;
}

block! {
    /// `oak_log` — Oak Log. `Movable | Breakable | Flammable`
    pub struct OakLog;
    facets: Movable, Breakable, Flammable;
    state: PillarState;
}

block! {
    /// `oak_planks` — Oak Planks. `Movable | Breakable | Flammable`
    pub struct OakPlanks;
    facets: Movable, Breakable, Flammable;
}

block! {
    /// `oak_pressure_plate` — Oak Pressure Plate. `Breakable | Flammable`
    pub struct OakPressurePlate;
    facets: Breakable, Flammable;
    state: PressurePlateState;
}

block! {
    /// `oak_sapling` — Oak Sapling. `InstantMinable`
    pub struct OakSapling;
    facets: InstantMinable;
    state: SaplingState;
}

block! {
    /// `oak_shelf` — Oak Shelf. `UnMovable | Breakable`
    pub struct OakShelf;
    facets: UnMovable, Breakable;
    state: ShelfState;
}

block! {
    /// `oak_sign` — Oak Sign. `Breakable | Flammable`
    pub struct OakSign;
    facets: Breakable, Flammable;
    state: StandingSignState;
}

block! {
    /// `oak_slab` — Oak Slab. `Movable | Breakable | Flammable`
    pub struct OakSlab;
    facets: Movable, Breakable, Flammable;
    state: SlabState;
}

block! {
    /// `oak_stairs` — Oak Stairs. `Movable | Breakable | Flammable`
    pub struct OakStairs;
    facets: Movable, Breakable, Flammable;
    state: StairState;
}

block! {
    /// `oak_trapdoor` — Oak Trapdoor. `Movable | Breakable | Flammable`
    pub struct OakTrapdoor;
    facets: Movable, Breakable, Flammable;
    state: TrapdoorState;
}

block! {
    /// `oak_wall_hanging_sign` — Oak Hanging Sign. `Breakable | Flammable`
    pub struct OakWallHangingSign;
    facets: Breakable, Flammable;
    state: WallMountedState;
}

block! {
    /// `oak_wall_sign` — Oak Sign. `Breakable | Flammable`
    pub struct OakWallSign;
    facets: Breakable, Flammable;
    state: WallMountedState;
}

block! {
    /// `oak_wood` — Oak Wood. `Movable | Breakable | Flammable`
    pub struct OakWood;
    facets: Movable, Breakable, Flammable;
    state: PillarState;
}

block! {
    /// `observer` — Observer. `Movable | Breakable`
    pub struct Observer;
    facets: Movable, Breakable;
    state: FacingPoweredState;
}

block! {
    /// `obsidian` — Obsidian. `UnMovable | Breakable`
    pub struct Obsidian;
    facets: UnMovable, Breakable;
}

block! {
    /// `ochre_froglight` — Ochre Froglight. `Movable | Breakable`
    pub struct OchreFroglight;
    facets: Movable, Breakable;
    state: PillarState;
}

block! {
    /// `open_eyeblossom` — Open Eyeblossom. `InstantMinable`
    pub struct OpenEyeblossom;
    facets: InstantMinable;
}

block! {
    /// `orange_banner` — Orange Banner. `Breakable`
    pub struct OrangeBanner;
    facets: Breakable;
    state: BannerState;
}

block! {
    /// `orange_bed` — Orange Bed. `Breakable`
    pub struct OrangeBed;
    facets: Breakable;
    state: BedState;
}

block! {
    /// `orange_candle` — Orange Candle. `Movable | Breakable`
    pub struct OrangeCandle;
    facets: Movable, Breakable;
    state: CandleState;
}

block! {
    /// `orange_candle_cake` — Cake with Orange Candle. `Movable | Breakable`
    pub struct OrangeCandleCake;
    facets: Movable, Breakable;
    state: LitState;
}

block! {
    /// `orange_carpet` — Orange Carpet. `Movable | Breakable | Flammable`
    pub struct OrangeCarpet;
    facets: Movable, Breakable, Flammable;
}

block! {
    /// `orange_concrete` — Orange Concrete. `Movable | Breakable`
    pub struct OrangeConcrete;
    facets: Movable, Breakable;
}

block! {
    /// `orange_concrete_powder` — Orange Concrete Powder. `Movable | Breakable`
    pub struct OrangeConcretePowder;
    facets: Movable, Breakable;
}

block! {
    /// `orange_glazed_terracotta` — Orange Glazed Terracotta. `Movable | Breakable`
    pub struct OrangeGlazedTerracotta;
    facets: Movable, Breakable;
    state: FacingState;
}

block! {
    /// `orange_shulker_box` — Orange Shulker Box. `Breakable`
    pub struct OrangeShulkerBox;
    facets: Breakable;
    state: ShulkerBoxState;
}

block! {
    /// `orange_stained_glass` — Orange Stained Glass. `Movable | Breakable`
    pub struct OrangeStainedGlass;
    facets: Movable, Breakable;
}

block! {
    /// `orange_stained_glass_pane` — Orange Stained Glass Pane. `Movable | Breakable`
    pub struct OrangeStainedGlassPane;
    facets: Movable, Breakable;
    state: ConnectionState;
}

block! {
    /// `orange_terracotta` — Orange Terracotta. `Movable | Breakable`
    pub struct OrangeTerracotta;
    facets: Movable, Breakable;
}

block! {
    /// `orange_tulip` — Orange Tulip. `InstantMinable | Flammable`
    pub struct OrangeTulip;
    facets: InstantMinable, Flammable;
}

block! {
    /// `orange_wall_banner` — Orange Banner. `Breakable`
    pub struct OrangeWallBanner;
    facets: Breakable;
    state: FacingState;
}

block! {
    /// `orange_wool` — Orange Wool. `Movable | Breakable | Flammable`
    pub struct OrangeWool;
    facets: Movable, Breakable, Flammable;
}

block! {
    /// `oxeye_daisy` — Oxeye Daisy. `InstantMinable | Flammable`
    pub struct OxeyeDaisy;
    facets: InstantMinable, Flammable;
}

block! {
    /// `oxidized_chiseled_copper` — Oxidized Chiseled Copper. `Movable | Breakable`
    pub struct OxidizedChiseledCopper;
    facets: Movable, Breakable;
}

block! {
    /// `oxidized_copper` — Oxidized Copper. `Movable | Breakable`
    pub struct OxidizedCopper;
    facets: Movable, Breakable;
}

block! {
    /// `oxidized_copper_bars` — Oxidized Copper Bars. `Movable | Breakable`
    pub struct OxidizedCopperBars;
    facets: Movable, Breakable;
    state: ConnectionState;
}

block! {
    /// `oxidized_copper_bulb` — Oxidized Copper Bulb. `Movable | Breakable`
    pub struct OxidizedCopperBulb;
    facets: Movable, Breakable;
    state: WeatheringCopperBulbState;
}

block! {
    /// `oxidized_copper_chain` — Oxidized Copper Chain. `Movable | Breakable`
    pub struct OxidizedCopperChain;
    facets: Movable, Breakable;
    state: WeatheringCopperChainState;
}

block! {
    /// `oxidized_copper_chest` — Oxidized Copper Chest. `UnMovable | Breakable`
    pub struct OxidizedCopperChest;
    facets: UnMovable, Breakable;
    state: ChestState;
}

block! {
    /// `oxidized_copper_door` — Oxidized Copper Door. `Breakable`
    pub struct OxidizedCopperDoor;
    facets: Breakable;
    state: DoorState;
}

block! {
    /// `oxidized_copper_golem_statue` — Oxidized Copper Golem Statue. `Movable | Breakable`
    pub struct OxidizedCopperGolemStatue;
    facets: Movable, Breakable;
    state: WeatheringCopperGolemStatueState;
}

block! {
    /// `oxidized_copper_grate` — Oxidized Copper Grate. `Movable | Breakable`
    pub struct OxidizedCopperGrate;
    facets: Movable, Breakable;
    state: WaterloggedState;
}

block! {
    /// `oxidized_copper_lantern` — Oxidized Copper Lantern. `Breakable`
    pub struct OxidizedCopperLantern;
    facets: Breakable;
    state: WeatheringLanternState;
}

block! {
    /// `oxidized_copper_trapdoor` — Oxidized Copper Trapdoor. `Movable | Breakable`
    pub struct OxidizedCopperTrapdoor;
    facets: Movable, Breakable;
    state: TrapdoorState;
}

block! {
    /// `oxidized_cut_copper` — Oxidized Cut Copper. `Movable | Breakable`
    pub struct OxidizedCutCopper;
    facets: Movable, Breakable;
}

block! {
    /// `oxidized_cut_copper_slab` — Oxidized Cut Copper Slab. `Movable | Breakable`
    pub struct OxidizedCutCopperSlab;
    facets: Movable, Breakable;
    state: SlabState;
}

block! {
    /// `oxidized_cut_copper_stairs` — Oxidized Cut Copper Stairs. `Movable | Breakable`
    pub struct OxidizedCutCopperStairs;
    facets: Movable, Breakable;
    state: StairState;
}

block! {
    /// `oxidized_lightning_rod` — Oxidized Lightning Rod. `Movable | Breakable`
    pub struct OxidizedLightningRod;
    facets: Movable, Breakable;
    state: WeatheringLightningRodState;
}

block! {
    /// `packed_ice` — Packed Ice. `Movable | Breakable`
    pub struct PackedIce;
    facets: Movable, Breakable;
}

block! {
    /// `packed_mud` — Packed Mud. `Movable | Breakable`
    pub struct PackedMud;
    facets: Movable, Breakable;
}

block! {
    /// `pale_hanging_moss` — Pale Hanging Moss. `InstantMinable | Flammable`
    pub struct PaleHangingMoss;
    facets: InstantMinable, Flammable;
    state: HangingMossState;
}

block! {
    /// `pale_moss_block` — Pale Moss Block. `Breakable | Flammable`
    pub struct PaleMossBlock;
    facets: Breakable, Flammable;
}

block! {
    /// `pale_moss_carpet` — Pale Moss Carpet. `Breakable`
    pub struct PaleMossCarpet;
    facets: Breakable;
    state: MossyCarpetState;
}

block! {
    /// `pale_oak_button` — Pale Oak Button. `Breakable | Flammable`
    pub struct PaleOakButton;
    facets: Breakable, Flammable;
    state: SwitchState;
}

block! {
    /// `pale_oak_door` — Pale Oak Door. `Breakable | Flammable`
    pub struct PaleOakDoor;
    facets: Breakable, Flammable;
    state: DoorState;
}

block! {
    /// `pale_oak_fence` — Pale Oak Fence. `Movable | Breakable | Flammable`
    pub struct PaleOakFence;
    facets: Movable, Breakable, Flammable;
    state: ConnectionState;
}

block! {
    /// `pale_oak_fence_gate` — Pale Oak Fence Gate. `Movable | Breakable | Flammable`
    pub struct PaleOakFenceGate;
    facets: Movable, Breakable, Flammable;
    state: FenceGateState;
}

block! {
    /// `pale_oak_hanging_sign` — Pale Oak Hanging Sign. `Breakable | Flammable`
    pub struct PaleOakHangingSign;
    facets: Breakable, Flammable;
    state: HangingSignState;
}

block! {
    /// `pale_oak_leaves` — Pale Oak Leaves. `Movable | Breakable | Flammable`
    pub struct PaleOakLeaves;
    facets: Movable, Breakable, Flammable;
    state: LeavesState;
}

block! {
    /// `pale_oak_log` — Pale Oak Log. `Movable | Breakable | Flammable`
    pub struct PaleOakLog;
    facets: Movable, Breakable, Flammable;
    state: PillarState;
}

block! {
    /// `pale_oak_planks` — Pale Oak Planks. `Movable | Breakable | Flammable`
    pub struct PaleOakPlanks;
    facets: Movable, Breakable, Flammable;
}

block! {
    /// `pale_oak_pressure_plate` — Pale Oak Pressure Plate. `Breakable | Flammable`
    pub struct PaleOakPressurePlate;
    facets: Breakable, Flammable;
    state: PressurePlateState;
}

block! {
    /// `pale_oak_sapling` — Pale Oak Sapling. `InstantMinable`
    pub struct PaleOakSapling;
    facets: InstantMinable;
    state: SaplingState;
}

block! {
    /// `pale_oak_shelf` — Pale Oak Shelf. `UnMovable | Breakable`
    pub struct PaleOakShelf;
    facets: UnMovable, Breakable;
    state: ShelfState;
}

block! {
    /// `pale_oak_sign` — Pale Oak Sign. `Breakable | Flammable`
    pub struct PaleOakSign;
    facets: Breakable, Flammable;
    state: StandingSignState;
}

block! {
    /// `pale_oak_slab` — Pale Oak Slab. `Movable | Breakable | Flammable`
    pub struct PaleOakSlab;
    facets: Movable, Breakable, Flammable;
    state: SlabState;
}

block! {
    /// `pale_oak_stairs` — Pale Oak Stairs. `Movable | Breakable | Flammable`
    pub struct PaleOakStairs;
    facets: Movable, Breakable, Flammable;
    state: StairState;
}

block! {
    /// `pale_oak_trapdoor` — Pale Oak Trapdoor. `Movable | Breakable | Flammable`
    pub struct PaleOakTrapdoor;
    facets: Movable, Breakable, Flammable;
    state: TrapdoorState;
}

block! {
    /// `pale_oak_wall_hanging_sign` — Pale Oak Hanging Sign. `Breakable | Flammable`
    pub struct PaleOakWallHangingSign;
    facets: Breakable, Flammable;
    state: WallMountedState;
}

block! {
    /// `pale_oak_wall_sign` — Pale Oak Sign. `Breakable | Flammable`
    pub struct PaleOakWallSign;
    facets: Breakable, Flammable;
    state: WallMountedState;
}

block! {
    /// `pale_oak_wood` — Pale Oak Wood. `Movable | Breakable | Flammable`
    pub struct PaleOakWood;
    facets: Movable, Breakable, Flammable;
    state: PillarState;
}

block! {
    /// `pearlescent_froglight` — Pearlescent Froglight. `Movable | Breakable`
    pub struct PearlescentFroglight;
    facets: Movable, Breakable;
    state: PillarState;
}

block! {
    /// `peony` — Peony. `InstantMinable | Flammable`
    pub struct Peony;
    facets: InstantMinable, Flammable;
    state: DoublePlantState;
}

block! {
    /// `petrified_oak_slab` — Petrified Oak Slab. `Movable | Breakable`
    pub struct PetrifiedOakSlab;
    facets: Movable, Breakable;
    state: SlabState;
}

block! {
    /// `piglin_head` — Piglin Head. `Breakable`
    pub struct PiglinHead;
    facets: Breakable;
    state: SkullState;
}

block! {
    /// `piglin_wall_head` — Piglin Head. `Breakable`
    pub struct PiglinWallHead;
    facets: Breakable;
    state: WallSkullState;
}

block! {
    /// `pink_banner` — Pink Banner. `Breakable`
    pub struct PinkBanner;
    facets: Breakable;
    state: BannerState;
}

block! {
    /// `pink_bed` — Pink Bed. `Breakable`
    pub struct PinkBed;
    facets: Breakable;
    state: BedState;
}

block! {
    /// `pink_candle` — Pink Candle. `Movable | Breakable`
    pub struct PinkCandle;
    facets: Movable, Breakable;
    state: CandleState;
}

block! {
    /// `pink_candle_cake` — Cake with Pink Candle. `Movable | Breakable`
    pub struct PinkCandleCake;
    facets: Movable, Breakable;
    state: LitState;
}

block! {
    /// `pink_carpet` — Pink Carpet. `Movable | Breakable | Flammable`
    pub struct PinkCarpet;
    facets: Movable, Breakable, Flammable;
}

block! {
    /// `pink_concrete` — Pink Concrete. `Movable | Breakable`
    pub struct PinkConcrete;
    facets: Movable, Breakable;
}

block! {
    /// `pink_concrete_powder` — Pink Concrete Powder. `Movable | Breakable`
    pub struct PinkConcretePowder;
    facets: Movable, Breakable;
}

block! {
    /// `pink_glazed_terracotta` — Pink Glazed Terracotta. `Movable | Breakable`
    pub struct PinkGlazedTerracotta;
    facets: Movable, Breakable;
    state: FacingState;
}

block! {
    /// `pink_petals` — Pink Petals. `InstantMinable | Flammable`
    pub struct PinkPetals;
    facets: InstantMinable, Flammable;
    state: FlowerBedState;
}

block! {
    /// `pink_shulker_box` — Pink Shulker Box. `Breakable`
    pub struct PinkShulkerBox;
    facets: Breakable;
    state: ShulkerBoxState;
}

block! {
    /// `pink_stained_glass` — Pink Stained Glass. `Movable | Breakable`
    pub struct PinkStainedGlass;
    facets: Movable, Breakable;
}

block! {
    /// `pink_stained_glass_pane` — Pink Stained Glass Pane. `Movable | Breakable`
    pub struct PinkStainedGlassPane;
    facets: Movable, Breakable;
    state: ConnectionState;
}

block! {
    /// `pink_terracotta` — Pink Terracotta. `Movable | Breakable`
    pub struct PinkTerracotta;
    facets: Movable, Breakable;
}

block! {
    /// `pink_tulip` — Pink Tulip. `InstantMinable | Flammable`
    pub struct PinkTulip;
    facets: InstantMinable, Flammable;
}

block! {
    /// `pink_wall_banner` — Pink Banner. `Breakable`
    pub struct PinkWallBanner;
    facets: Breakable;
    state: FacingState;
}

block! {
    /// `pink_wool` — Pink Wool. `Movable | Breakable | Flammable`
    pub struct PinkWool;
    facets: Movable, Breakable, Flammable;
}

block! {
    /// `piston` — Piston. `Movable | Breakable`
    pub struct Piston;
    facets: Movable, Breakable;
    state: PistonBaseState;
}

block! {
    /// `piston_head` — Piston Head. `UnMovable | Breakable`
    pub struct PistonHead;
    facets: UnMovable, Breakable;
    state: PistonHeadState;
}

block! {
    /// `pitcher_crop` — Pitcher Crop. `InstantMinable`
    pub struct PitcherCrop;
    facets: InstantMinable;
    state: PitcherCropState;
}

block! {
    /// `pitcher_plant` — Pitcher Plant. `InstantMinable | Flammable`
    pub struct PitcherPlant;
    facets: InstantMinable, Flammable;
    state: DoublePlantState;
}

block! {
    /// `player_head` — Player Head. `Breakable`
    pub struct PlayerHead;
    facets: Breakable;
    state: SkullState;
}

block! {
    /// `player_wall_head` — Player Head. `Breakable`
    pub struct PlayerWallHead;
    facets: Breakable;
    state: WallSkullState;
}

block! {
    /// `podzol` — Podzol. `Movable | Breakable`
    pub struct Podzol;
    facets: Movable, Breakable;
    state: SnowyState;
}

block! {
    /// `pointed_dripstone` — Pointed Dripstone. `Breakable`
    pub struct PointedDripstone;
    facets: Breakable;
    state: PointedDripstoneState;
}

block! {
    /// `polished_andesite` — Polished Andesite. `Movable | Breakable`
    pub struct PolishedAndesite;
    facets: Movable, Breakable;
}

block! {
    /// `polished_andesite_slab` — Polished Andesite Slab. `Movable | Breakable`
    pub struct PolishedAndesiteSlab;
    facets: Movable, Breakable;
    state: SlabState;
}

block! {
    /// `polished_andesite_stairs` — Polished Andesite Stairs. `Movable | Breakable`
    pub struct PolishedAndesiteStairs;
    facets: Movable, Breakable;
    state: StairState;
}

block! {
    /// `polished_basalt` — Polished Basalt. `Movable | Breakable`
    pub struct PolishedBasalt;
    facets: Movable, Breakable;
    state: PillarState;
}

block! {
    /// `polished_blackstone` — Polished Blackstone. `Movable | Breakable`
    pub struct PolishedBlackstone;
    facets: Movable, Breakable;
}

block! {
    /// `polished_blackstone_brick_slab` — Polished Blackstone Brick Slab. `Movable | Breakable`
    pub struct PolishedBlackstoneBrickSlab;
    facets: Movable, Breakable;
    state: SlabState;
}

block! {
    /// `polished_blackstone_brick_stairs` — Polished Blackstone Brick Stairs. `Movable | Breakable`
    pub struct PolishedBlackstoneBrickStairs;
    facets: Movable, Breakable;
    state: StairState;
}

block! {
    /// `polished_blackstone_brick_wall` — Polished Blackstone Brick Wall. `Movable | Breakable`
    pub struct PolishedBlackstoneBrickWall;
    facets: Movable, Breakable;
    state: WallState;
}

block! {
    /// `polished_blackstone_bricks` — Polished Blackstone Bricks. `Movable | Breakable`
    pub struct PolishedBlackstoneBricks;
    facets: Movable, Breakable;
}

block! {
    /// `polished_blackstone_button` — Polished Blackstone Button. `Breakable`
    pub struct PolishedBlackstoneButton;
    facets: Breakable;
    state: SwitchState;
}

block! {
    /// `polished_blackstone_pressure_plate` — Polished Blackstone Pressure Plate. `Breakable`
    pub struct PolishedBlackstonePressurePlate;
    facets: Breakable;
    state: PressurePlateState;
}

block! {
    /// `polished_blackstone_slab` — Polished Blackstone Slab. `Movable | Breakable`
    pub struct PolishedBlackstoneSlab;
    facets: Movable, Breakable;
    state: SlabState;
}

block! {
    /// `polished_blackstone_stairs` — Polished Blackstone Stairs. `Movable | Breakable`
    pub struct PolishedBlackstoneStairs;
    facets: Movable, Breakable;
    state: StairState;
}

block! {
    /// `polished_blackstone_wall` — Polished Blackstone Wall. `Movable | Breakable`
    pub struct PolishedBlackstoneWall;
    facets: Movable, Breakable;
    state: WallState;
}

block! {
    /// `polished_cinnabar` — Polished Cinnabar. `Movable | Breakable`
    pub struct PolishedCinnabar;
    facets: Movable, Breakable;
}

block! {
    /// `polished_cinnabar_slab` — Polished Cinnabar Slab. `Movable | Breakable`
    pub struct PolishedCinnabarSlab;
    facets: Movable, Breakable;
    state: SlabState;
}

block! {
    /// `polished_cinnabar_stairs` — Polished Cinnabar Stairs. `Movable | Breakable`
    pub struct PolishedCinnabarStairs;
    facets: Movable, Breakable;
    state: StairState;
}

block! {
    /// `polished_cinnabar_wall` — Polished Cinnabar Wall. `Movable | Breakable`
    pub struct PolishedCinnabarWall;
    facets: Movable, Breakable;
    state: WallState;
}

block! {
    /// `polished_deepslate` — Polished Deepslate. `Movable | Breakable`
    pub struct PolishedDeepslate;
    facets: Movable, Breakable;
}

block! {
    /// `polished_deepslate_slab` — Polished Deepslate Slab. `Movable | Breakable`
    pub struct PolishedDeepslateSlab;
    facets: Movable, Breakable;
    state: SlabState;
}

block! {
    /// `polished_deepslate_stairs` — Polished Deepslate Stairs. `Movable | Breakable`
    pub struct PolishedDeepslateStairs;
    facets: Movable, Breakable;
    state: StairState;
}

block! {
    /// `polished_deepslate_wall` — Polished Deepslate Wall. `Movable | Breakable`
    pub struct PolishedDeepslateWall;
    facets: Movable, Breakable;
    state: WallState;
}

block! {
    /// `polished_diorite` — Polished Diorite. `Movable | Breakable`
    pub struct PolishedDiorite;
    facets: Movable, Breakable;
}

block! {
    /// `polished_diorite_slab` — Polished Diorite Slab. `Movable | Breakable`
    pub struct PolishedDioriteSlab;
    facets: Movable, Breakable;
    state: SlabState;
}

block! {
    /// `polished_diorite_stairs` — Polished Diorite Stairs. `Movable | Breakable`
    pub struct PolishedDioriteStairs;
    facets: Movable, Breakable;
    state: StairState;
}

block! {
    /// `polished_granite` — Polished Granite. `Movable | Breakable`
    pub struct PolishedGranite;
    facets: Movable, Breakable;
}

block! {
    /// `polished_granite_slab` — Polished Granite Slab. `Movable | Breakable`
    pub struct PolishedGraniteSlab;
    facets: Movable, Breakable;
    state: SlabState;
}

block! {
    /// `polished_granite_stairs` — Polished Granite Stairs. `Movable | Breakable`
    pub struct PolishedGraniteStairs;
    facets: Movable, Breakable;
    state: StairState;
}

block! {
    /// `polished_sulfur` — Polished Sulfur. `Movable | Breakable`
    pub struct PolishedSulfur;
    facets: Movable, Breakable;
}

block! {
    /// `polished_sulfur_slab` — Polished Sulfur Slab. `Movable | Breakable`
    pub struct PolishedSulfurSlab;
    facets: Movable, Breakable;
    state: SlabState;
}

block! {
    /// `polished_sulfur_stairs` — Polished Sulfur Stairs. `Movable | Breakable`
    pub struct PolishedSulfurStairs;
    facets: Movable, Breakable;
    state: StairState;
}

block! {
    /// `polished_sulfur_wall` — Polished Sulfur Wall. `Movable | Breakable`
    pub struct PolishedSulfurWall;
    facets: Movable, Breakable;
    state: WallState;
}

block! {
    /// `polished_tuff` — Polished Tuff. `Movable | Breakable`
    pub struct PolishedTuff;
    facets: Movable, Breakable;
}

block! {
    /// `polished_tuff_slab` — Polished Tuff Slab. `Movable | Breakable`
    pub struct PolishedTuffSlab;
    facets: Movable, Breakable;
    state: SlabState;
}

block! {
    /// `polished_tuff_stairs` — Polished Tuff Stairs. `Movable | Breakable`
    pub struct PolishedTuffStairs;
    facets: Movable, Breakable;
    state: StairState;
}

block! {
    /// `polished_tuff_wall` — Polished Tuff Wall. `Movable | Breakable`
    pub struct PolishedTuffWall;
    facets: Movable, Breakable;
    state: WallState;
}

block! {
    /// `poppy` — Poppy. `InstantMinable | Flammable`
    pub struct Poppy;
    facets: InstantMinable, Flammable;
}

block! {
    /// `potatoes` — Potatoes. `InstantMinable`
    pub struct Potatoes;
    facets: InstantMinable;
    state: CropAgeState;
}

block! {
    /// `potent_sulfur` — Potent Sulfur. `UnMovable | Breakable`
    pub struct PotentSulfur;
    facets: UnMovable, Breakable;
    state: PotentSulfurState;
}

block! {
    /// `potted_acacia_sapling` — Potted Acacia Sapling. `InstantMinable`
    pub struct PottedAcaciaSapling;
    facets: InstantMinable;
}

block! {
    /// `potted_allium` — Potted Allium. `InstantMinable`
    pub struct PottedAllium;
    facets: InstantMinable;
}

block! {
    /// `potted_azalea_bush` — Potted Azalea. `InstantMinable`
    pub struct PottedAzaleaBush;
    facets: InstantMinable;
}

block! {
    /// `potted_azure_bluet` — Potted Azure Bluet. `InstantMinable`
    pub struct PottedAzureBluet;
    facets: InstantMinable;
}

block! {
    /// `potted_bamboo` — Potted Bamboo. `InstantMinable`
    pub struct PottedBamboo;
    facets: InstantMinable;
}

block! {
    /// `potted_birch_sapling` — Potted Birch Sapling. `InstantMinable`
    pub struct PottedBirchSapling;
    facets: InstantMinable;
}

block! {
    /// `potted_blue_orchid` — Potted Blue Orchid. `InstantMinable`
    pub struct PottedBlueOrchid;
    facets: InstantMinable;
}

block! {
    /// `potted_brown_mushroom` — Potted Brown Mushroom. `InstantMinable`
    pub struct PottedBrownMushroom;
    facets: InstantMinable;
}

block! {
    /// `potted_cactus` — Potted Cactus. `InstantMinable`
    pub struct PottedCactus;
    facets: InstantMinable;
}

block! {
    /// `potted_cherry_sapling` — Potted Cherry Sapling. `InstantMinable`
    pub struct PottedCherrySapling;
    facets: InstantMinable;
}

block! {
    /// `potted_closed_eyeblossom` — Potted Closed Eyeblossom. `InstantMinable`
    pub struct PottedClosedEyeblossom;
    facets: InstantMinable;
}

block! {
    /// `potted_cornflower` — Potted Cornflower. `InstantMinable`
    pub struct PottedCornflower;
    facets: InstantMinable;
}

block! {
    /// `potted_crimson_fungus` — Potted Crimson Fungus. `InstantMinable`
    pub struct PottedCrimsonFungus;
    facets: InstantMinable;
}

block! {
    /// `potted_crimson_roots` — Potted Crimson Roots. `InstantMinable`
    pub struct PottedCrimsonRoots;
    facets: InstantMinable;
}

block! {
    /// `potted_dandelion` — Potted Dandelion. `InstantMinable`
    pub struct PottedDandelion;
    facets: InstantMinable;
}

block! {
    /// `potted_dark_oak_sapling` — Potted Dark Oak Sapling. `InstantMinable`
    pub struct PottedDarkOakSapling;
    facets: InstantMinable;
}

block! {
    /// `potted_dead_bush` — Potted Dead Bush. `InstantMinable`
    pub struct PottedDeadBush;
    facets: InstantMinable;
}

block! {
    /// `potted_fern` — Potted Fern. `InstantMinable`
    pub struct PottedFern;
    facets: InstantMinable;
}

block! {
    /// `potted_flowering_azalea_bush` — Potted Flowering Azalea. `InstantMinable`
    pub struct PottedFloweringAzaleaBush;
    facets: InstantMinable;
}

block! {
    /// `potted_golden_dandelion` — Potted Golden Dandelion. `InstantMinable`
    pub struct PottedGoldenDandelion;
    facets: InstantMinable;
}

block! {
    /// `potted_jungle_sapling` — Potted Jungle Sapling. `InstantMinable`
    pub struct PottedJungleSapling;
    facets: InstantMinable;
}

block! {
    /// `potted_lily_of_the_valley` — Potted Lily of the Valley. `InstantMinable`
    pub struct PottedLilyOfTheValley;
    facets: InstantMinable;
}

block! {
    /// `potted_mangrove_propagule` — Potted Mangrove Propagule. `InstantMinable`
    pub struct PottedMangrovePropagule;
    facets: InstantMinable;
}

block! {
    /// `potted_oak_sapling` — Potted Oak Sapling. `InstantMinable`
    pub struct PottedOakSapling;
    facets: InstantMinable;
}

block! {
    /// `potted_open_eyeblossom` — Potted Open Eyeblossom. `InstantMinable`
    pub struct PottedOpenEyeblossom;
    facets: InstantMinable;
}

block! {
    /// `potted_orange_tulip` — Potted Orange Tulip. `InstantMinable`
    pub struct PottedOrangeTulip;
    facets: InstantMinable;
}

block! {
    /// `potted_oxeye_daisy` — Potted Oxeye Daisy. `InstantMinable`
    pub struct PottedOxeyeDaisy;
    facets: InstantMinable;
}

block! {
    /// `potted_pale_oak_sapling` — Potted Pale Oak Sapling. `InstantMinable`
    pub struct PottedPaleOakSapling;
    facets: InstantMinable;
}

block! {
    /// `potted_pink_tulip` — Potted Pink Tulip. `InstantMinable`
    pub struct PottedPinkTulip;
    facets: InstantMinable;
}

block! {
    /// `potted_poppy` — Potted Poppy. `InstantMinable`
    pub struct PottedPoppy;
    facets: InstantMinable;
}

block! {
    /// `potted_red_mushroom` — Potted Red Mushroom. `InstantMinable`
    pub struct PottedRedMushroom;
    facets: InstantMinable;
}

block! {
    /// `potted_red_tulip` — Potted Red Tulip. `InstantMinable`
    pub struct PottedRedTulip;
    facets: InstantMinable;
}

block! {
    /// `potted_spruce_sapling` — Potted Spruce Sapling. `InstantMinable`
    pub struct PottedSpruceSapling;
    facets: InstantMinable;
}

block! {
    /// `potted_torchflower` — Potted Torchflower. `InstantMinable`
    pub struct PottedTorchflower;
    facets: InstantMinable;
}

block! {
    /// `potted_warped_fungus` — Potted Warped Fungus. `InstantMinable`
    pub struct PottedWarpedFungus;
    facets: InstantMinable;
}

block! {
    /// `potted_warped_roots` — Potted Warped Roots. `InstantMinable`
    pub struct PottedWarpedRoots;
    facets: InstantMinable;
}

block! {
    /// `potted_white_tulip` — Potted White Tulip. `InstantMinable`
    pub struct PottedWhiteTulip;
    facets: InstantMinable;
}

block! {
    /// `potted_wither_rose` — Potted Wither Rose. `InstantMinable`
    pub struct PottedWitherRose;
    facets: InstantMinable;
}

block! {
    /// `powder_snow` — Powder Snow. `Movable | Breakable`
    pub struct PowderSnow;
    facets: Movable, Breakable;
}

block! {
    /// `powder_snow_cauldron` — Powder Snow Cauldron. `Movable | Breakable`
    pub struct PowderSnowCauldron;
    facets: Movable, Breakable;
    state: LayeredCauldronState;
}

block! {
    /// `powered_rail` — Powered Rail. `Movable | Breakable`
    pub struct PoweredRail;
    facets: Movable, Breakable;
    state: PoweredRailState;
}

block! {
    /// `prismarine` — Prismarine. `Movable | Breakable`
    pub struct Prismarine;
    facets: Movable, Breakable;
}

block! {
    /// `prismarine_brick_slab` — Prismarine Brick Slab. `Movable | Breakable`
    pub struct PrismarineBrickSlab;
    facets: Movable, Breakable;
    state: SlabState;
}

block! {
    /// `prismarine_brick_stairs` — Prismarine Brick Stairs. `Movable | Breakable`
    pub struct PrismarineBrickStairs;
    facets: Movable, Breakable;
    state: StairState;
}

block! {
    /// `prismarine_bricks` — Prismarine Bricks. `Movable | Breakable`
    pub struct PrismarineBricks;
    facets: Movable, Breakable;
}

block! {
    /// `prismarine_slab` — Prismarine Slab. `Movable | Breakable`
    pub struct PrismarineSlab;
    facets: Movable, Breakable;
    state: SlabState;
}

block! {
    /// `prismarine_stairs` — Prismarine Stairs. `Movable | Breakable`
    pub struct PrismarineStairs;
    facets: Movable, Breakable;
    state: StairState;
}

block! {
    /// `prismarine_wall` — Prismarine Wall. `Movable | Breakable`
    pub struct PrismarineWall;
    facets: Movable, Breakable;
    state: WallState;
}

block! {
    /// `pumpkin` — Pumpkin. `Breakable`
    pub struct Pumpkin;
    facets: Breakable;
}

block! {
    /// `pumpkin_stem` — Pumpkin Stem. `InstantMinable`
    pub struct PumpkinStem;
    facets: InstantMinable;
    state: CropAgeState;
}

block! {
    /// `purple_banner` — Purple Banner. `Breakable`
    pub struct PurpleBanner;
    facets: Breakable;
    state: BannerState;
}

block! {
    /// `purple_bed` — Purple Bed. `Breakable`
    pub struct PurpleBed;
    facets: Breakable;
    state: BedState;
}

block! {
    /// `purple_candle` — Purple Candle. `Movable | Breakable`
    pub struct PurpleCandle;
    facets: Movable, Breakable;
    state: CandleState;
}

block! {
    /// `purple_candle_cake` — Cake with Purple Candle. `Movable | Breakable`
    pub struct PurpleCandleCake;
    facets: Movable, Breakable;
    state: LitState;
}

block! {
    /// `purple_carpet` — Purple Carpet. `Movable | Breakable | Flammable`
    pub struct PurpleCarpet;
    facets: Movable, Breakable, Flammable;
}

block! {
    /// `purple_concrete` — Purple Concrete. `Movable | Breakable`
    pub struct PurpleConcrete;
    facets: Movable, Breakable;
}

block! {
    /// `purple_concrete_powder` — Purple Concrete Powder. `Movable | Breakable`
    pub struct PurpleConcretePowder;
    facets: Movable, Breakable;
}

block! {
    /// `purple_glazed_terracotta` — Purple Glazed Terracotta. `Movable | Breakable`
    pub struct PurpleGlazedTerracotta;
    facets: Movable, Breakable;
    state: FacingState;
}

block! {
    /// `purple_shulker_box` — Purple Shulker Box. `Breakable`
    pub struct PurpleShulkerBox;
    facets: Breakable;
    state: ShulkerBoxState;
}

block! {
    /// `purple_stained_glass` — Purple Stained Glass. `Movable | Breakable`
    pub struct PurpleStainedGlass;
    facets: Movable, Breakable;
}

block! {
    /// `purple_stained_glass_pane` — Purple Stained Glass Pane. `Movable | Breakable`
    pub struct PurpleStainedGlassPane;
    facets: Movable, Breakable;
    state: ConnectionState;
}

block! {
    /// `purple_terracotta` — Purple Terracotta. `Movable | Breakable`
    pub struct PurpleTerracotta;
    facets: Movable, Breakable;
}

block! {
    /// `purple_wall_banner` — Purple Banner. `Breakable`
    pub struct PurpleWallBanner;
    facets: Breakable;
    state: FacingState;
}

block! {
    /// `purple_wool` — Purple Wool. `Movable | Breakable | Flammable`
    pub struct PurpleWool;
    facets: Movable, Breakable, Flammable;
}

block! {
    /// `purpur_block` — Purpur Block. `Movable | Breakable`
    pub struct PurpurBlock;
    facets: Movable, Breakable;
}

block! {
    /// `purpur_pillar` — Purpur Pillar. `Movable | Breakable`
    pub struct PurpurPillar;
    facets: Movable, Breakable;
    state: PillarState;
}

block! {
    /// `purpur_slab` — Purpur Slab. `Movable | Breakable`
    pub struct PurpurSlab;
    facets: Movable, Breakable;
    state: SlabState;
}

block! {
    /// `purpur_stairs` — Purpur Stairs. `Movable | Breakable`
    pub struct PurpurStairs;
    facets: Movable, Breakable;
    state: StairState;
}

block! {
    /// `quartz_block` — Block of Quartz. `Movable | Breakable`
    pub struct QuartzBlock;
    facets: Movable, Breakable;
}

block! {
    /// `quartz_bricks` — Quartz Bricks. `Movable | Breakable`
    pub struct QuartzBricks;
    facets: Movable, Breakable;
}

block! {
    /// `quartz_pillar` — Quartz Pillar. `Movable | Breakable`
    pub struct QuartzPillar;
    facets: Movable, Breakable;
    state: PillarState;
}

block! {
    /// `quartz_slab` — Quartz Slab. `Movable | Breakable`
    pub struct QuartzSlab;
    facets: Movable, Breakable;
    state: SlabState;
}

block! {
    /// `quartz_stairs` — Quartz Stairs. `Movable | Breakable`
    pub struct QuartzStairs;
    facets: Movable, Breakable;
    state: StairState;
}

block! {
    /// `rail` — Rail. `Movable | Breakable`
    pub struct Rail;
    facets: Movable, Breakable;
    state: RailState;
}

block! {
    /// `raw_copper_block` — Block of Raw Copper. `Movable | Breakable`
    pub struct RawCopperBlock;
    facets: Movable, Breakable;
}

block! {
    /// `raw_gold_block` — Block of Raw Gold. `Movable | Breakable`
    pub struct RawGoldBlock;
    facets: Movable, Breakable;
}

block! {
    /// `raw_iron_block` — Block of Raw Iron. `Movable | Breakable`
    pub struct RawIronBlock;
    facets: Movable, Breakable;
}

block! {
    /// `red_banner` — Red Banner. `Breakable`
    pub struct RedBanner;
    facets: Breakable;
    state: BannerState;
}

block! {
    /// `red_bed` — Red Bed. `Breakable`
    pub struct RedBed;
    facets: Breakable;
    state: BedState;
}

block! {
    /// `red_candle` — Red Candle. `Movable | Breakable`
    pub struct RedCandle;
    facets: Movable, Breakable;
    state: CandleState;
}

block! {
    /// `red_candle_cake` — Cake with Red Candle. `Movable | Breakable`
    pub struct RedCandleCake;
    facets: Movable, Breakable;
    state: LitState;
}

block! {
    /// `red_carpet` — Red Carpet. `Movable | Breakable | Flammable`
    pub struct RedCarpet;
    facets: Movable, Breakable, Flammable;
}

block! {
    /// `red_concrete` — Red Concrete. `Movable | Breakable`
    pub struct RedConcrete;
    facets: Movable, Breakable;
}

block! {
    /// `red_concrete_powder` — Red Concrete Powder. `Movable | Breakable`
    pub struct RedConcretePowder;
    facets: Movable, Breakable;
}

block! {
    /// `red_glazed_terracotta` — Red Glazed Terracotta. `Movable | Breakable`
    pub struct RedGlazedTerracotta;
    facets: Movable, Breakable;
    state: FacingState;
}

block! {
    /// `red_mushroom` — Red Mushroom. `InstantMinable`
    pub struct RedMushroom;
    facets: InstantMinable;
}

block! {
    /// `red_mushroom_block` — Red Mushroom Block. `Movable | Breakable`
    pub struct RedMushroomBlock;
    facets: Movable, Breakable;
    state: HugeMushroomState;
}

block! {
    /// `red_nether_brick_slab` — Red Nether Brick Slab. `Movable | Breakable`
    pub struct RedNetherBrickSlab;
    facets: Movable, Breakable;
    state: SlabState;
}

block! {
    /// `red_nether_brick_stairs` — Red Nether Brick Stairs. `Movable | Breakable`
    pub struct RedNetherBrickStairs;
    facets: Movable, Breakable;
    state: StairState;
}

block! {
    /// `red_nether_brick_wall` — Red Nether Brick Wall. `Movable | Breakable`
    pub struct RedNetherBrickWall;
    facets: Movable, Breakable;
    state: WallState;
}

block! {
    /// `red_nether_bricks` — Red Nether Bricks. `Movable | Breakable`
    pub struct RedNetherBricks;
    facets: Movable, Breakable;
}

block! {
    /// `red_sand` — Red Sand. `Movable | Breakable`
    pub struct RedSand;
    facets: Movable, Breakable;
}

block! {
    /// `red_sandstone` — Red Sandstone. `Movable | Breakable`
    pub struct RedSandstone;
    facets: Movable, Breakable;
}

block! {
    /// `red_sandstone_slab` — Red Sandstone Slab. `Movable | Breakable`
    pub struct RedSandstoneSlab;
    facets: Movable, Breakable;
    state: SlabState;
}

block! {
    /// `red_sandstone_stairs` — Red Sandstone Stairs. `Movable | Breakable`
    pub struct RedSandstoneStairs;
    facets: Movable, Breakable;
    state: StairState;
}

block! {
    /// `red_sandstone_wall` — Red Sandstone Wall. `Movable | Breakable`
    pub struct RedSandstoneWall;
    facets: Movable, Breakable;
    state: WallState;
}

block! {
    /// `red_shulker_box` — Red Shulker Box. `Breakable`
    pub struct RedShulkerBox;
    facets: Breakable;
    state: ShulkerBoxState;
}

block! {
    /// `red_stained_glass` — Red Stained Glass. `Movable | Breakable`
    pub struct RedStainedGlass;
    facets: Movable, Breakable;
}

block! {
    /// `red_stained_glass_pane` — Red Stained Glass Pane. `Movable | Breakable`
    pub struct RedStainedGlassPane;
    facets: Movable, Breakable;
    state: ConnectionState;
}

block! {
    /// `red_terracotta` — Red Terracotta. `Movable | Breakable`
    pub struct RedTerracotta;
    facets: Movable, Breakable;
}

block! {
    /// `red_tulip` — Red Tulip. `InstantMinable | Flammable`
    pub struct RedTulip;
    facets: InstantMinable, Flammable;
}

block! {
    /// `red_wall_banner` — Red Banner. `Breakable`
    pub struct RedWallBanner;
    facets: Breakable;
    state: FacingState;
}

block! {
    /// `red_wool` — Red Wool. `Movable | Breakable | Flammable`
    pub struct RedWool;
    facets: Movable, Breakable, Flammable;
}

block! {
    /// `redstone_block` — Block of Redstone. `Movable | Breakable | Power`
    pub struct RedstoneBlock;
    facets: Movable, Breakable, Power;
}

block! {
    /// `redstone_lamp` — Redstone Lamp. `Movable | Breakable`
    pub struct RedstoneLamp;
    facets: Movable, Breakable;
    state: LitState;
}

block! {
    /// `redstone_ore` — Redstone Ore. `Movable | Breakable`
    pub struct RedstoneOre;
    facets: Movable, Breakable;
    state: LitState;
}

block! {
    /// `redstone_torch` — Redstone Torch. `InstantMinable | Power`
    pub struct RedstoneTorch;
    facets: InstantMinable, Power;
    state: LitState;
}

block! {
    /// `redstone_wall_torch` — Redstone Torch. `InstantMinable | Power`
    pub struct RedstoneWallTorch;
    facets: InstantMinable, Power;
    state: BlastFurnaceState;
}

block! {
    /// `redstone_wire` — Redstone Wire. `InstantMinable`
    pub struct RedstoneWire;
    facets: InstantMinable;
    state: RedstoneWireState;
}

block! {
    /// `reinforced_deepslate` — Reinforced Deepslate. `UnMovable | Breakable`
    pub struct ReinforcedDeepslate;
    facets: UnMovable, Breakable;
}

block! {
    /// `repeater` — Redstone Repeater. `InstantMinable`
    pub struct Repeater;
    facets: InstantMinable;
    state: RepeaterState;
}

block! {
    /// `repeating_command_block` — Repeating Command Block. `UnMovable`
    pub struct RepeatingCommandBlock;
    facets: UnMovable;
    state: CommandState;
}

block! {
    /// `resin_block` — Block of Resin. `Movable | InstantMinable`
    pub struct ResinBlock;
    facets: Movable, InstantMinable;
}

block! {
    /// `resin_brick_slab` — Resin Brick Slab. `Movable | Breakable`
    pub struct ResinBrickSlab;
    facets: Movable, Breakable;
    state: SlabState;
}

block! {
    /// `resin_brick_stairs` — Resin Brick Stairs. `Movable | Breakable`
    pub struct ResinBrickStairs;
    facets: Movable, Breakable;
    state: StairState;
}

block! {
    /// `resin_brick_wall` — Resin Brick Wall. `Movable | Breakable`
    pub struct ResinBrickWall;
    facets: Movable, Breakable;
    state: WallState;
}

block! {
    /// `resin_bricks` — Resin Bricks. `Movable | Breakable`
    pub struct ResinBricks;
    facets: Movable, Breakable;
}

block! {
    /// `resin_clump` — Resin Clump. `InstantMinable`
    pub struct ResinClump;
    facets: InstantMinable;
    state: GlowLichenState;
}

block! {
    /// `respawn_anchor` — Respawn Anchor. `UnMovable | Breakable`
    pub struct RespawnAnchor;
    facets: UnMovable, Breakable;
    state: RespawnAnchorState;
}

block! {
    /// `rooted_dirt` — Rooted Dirt. `Movable | Breakable`
    pub struct RootedDirt;
    facets: Movable, Breakable;
}

block! {
    /// `rose_bush` — Rose Bush. `InstantMinable | Flammable`
    pub struct RoseBush;
    facets: InstantMinable, Flammable;
    state: DoublePlantState;
}

block! {
    /// `sand` — Sand. `Movable | Breakable`
    pub struct Sand;
    facets: Movable, Breakable;
}

block! {
    /// `sandstone` — Sandstone. `Movable | Breakable`
    pub struct Sandstone;
    facets: Movable, Breakable;
}

block! {
    /// `sandstone_slab` — Sandstone Slab. `Movable | Breakable`
    pub struct SandstoneSlab;
    facets: Movable, Breakable;
    state: SlabState;
}

block! {
    /// `sandstone_stairs` — Sandstone Stairs. `Movable | Breakable`
    pub struct SandstoneStairs;
    facets: Movable, Breakable;
    state: StairState;
}

block! {
    /// `sandstone_wall` — Sandstone Wall. `Movable | Breakable`
    pub struct SandstoneWall;
    facets: Movable, Breakable;
    state: WallState;
}

block! {
    /// `scaffolding` — Scaffolding. `InstantMinable | Flammable`
    pub struct Scaffolding;
    facets: InstantMinable, Flammable;
    state: ScaffoldingState;
}

block! {
    /// `sculk` — Sculk. `Movable | Breakable`
    pub struct Sculk;
    facets: Movable, Breakable;
}

block! {
    /// `sculk_catalyst` — Sculk Catalyst. `UnMovable | Breakable`
    pub struct SculkCatalyst;
    facets: UnMovable, Breakable;
    state: SculkCatalystState;
}

block! {
    /// `sculk_sensor` — Sculk Sensor. `UnMovable | Breakable`
    pub struct SculkSensor;
    facets: UnMovable, Breakable;
    state: SculkSensorState;
}

block! {
    /// `sculk_shrieker` — Sculk Shrieker. `UnMovable | Breakable`
    pub struct SculkShrieker;
    facets: UnMovable, Breakable;
    state: SculkShriekerState;
}

block! {
    /// `sculk_vein` — Sculk Vein. `Breakable`
    pub struct SculkVein;
    facets: Breakable;
    state: GlowLichenState;
}

block! {
    /// `sea_lantern` — Sea Lantern. `Movable | Breakable`
    pub struct SeaLantern;
    facets: Movable, Breakable;
}

block! {
    /// `sea_pickle` — Sea Pickle. `InstantMinable`
    pub struct SeaPickle;
    facets: InstantMinable;
    state: SeaPickleState;
}

block! {
    /// `seagrass` — Seagrass. `Movable | InstantMinable`
    pub struct Seagrass;
    facets: Movable, InstantMinable;
}

block! {
    /// `short_dry_grass` — Short Dry Grass. `InstantMinable | Flammable`
    pub struct ShortDryGrass;
    facets: InstantMinable, Flammable;
}

block! {
    /// `short_grass` — Short Grass. `InstantMinable | Flammable`
    pub struct ShortGrass;
    facets: InstantMinable, Flammable;
}

block! {
    /// `shroomlight` — Shroomlight. `Movable | Breakable`
    pub struct Shroomlight;
    facets: Movable, Breakable;
}

block! {
    /// `shulker_box` — Shulker Box. `Breakable`
    pub struct ShulkerBox;
    facets: Breakable;
    state: ShulkerBoxState;
}

block! {
    /// `skeleton_skull` — Skeleton Skull. `Breakable`
    pub struct SkeletonSkull;
    facets: Breakable;
    state: SkullState;
}

block! {
    /// `skeleton_wall_skull` — Skeleton Skull. `Breakable`
    pub struct SkeletonWallSkull;
    facets: Breakable;
    state: WallSkullState;
}

block! {
    /// `slime_block` — Slime Block. `Sticky | Movable | InstantMinable`
    pub struct SlimeBlock;
    facets: Sticky, Movable, InstantMinable;
}

block! {
    /// `small_amethyst_bud` — Small Amethyst Bud. `Breakable`
    pub struct SmallAmethystBud;
    facets: Breakable;
    state: AmethystClusterState;
}

block! {
    /// `small_dripleaf` — Small Dripleaf. `Movable | InstantMinable | Flammable`
    pub struct SmallDripleaf;
    facets: Movable, InstantMinable, Flammable;
    state: SmallDripleafState;
}

block! {
    /// `smithing_table` — Smithing Table. `Movable | Breakable`
    pub struct SmithingTable;
    facets: Movable, Breakable;
}

block! {
    /// `smoker` — Smoker. `UnMovable | Breakable`
    pub struct Smoker;
    facets: UnMovable, Breakable;
    state: BlastFurnaceState;
}

block! {
    /// `smooth_basalt` — Smooth Basalt. `Movable | Breakable`
    pub struct SmoothBasalt;
    facets: Movable, Breakable;
}

block! {
    /// `smooth_quartz` — Smooth Quartz Block. `Movable | Breakable`
    pub struct SmoothQuartz;
    facets: Movable, Breakable;
}

block! {
    /// `smooth_quartz_slab` — Smooth Quartz Slab. `Movable | Breakable`
    pub struct SmoothQuartzSlab;
    facets: Movable, Breakable;
    state: SlabState;
}

block! {
    /// `smooth_quartz_stairs` — Smooth Quartz Stairs. `Movable | Breakable`
    pub struct SmoothQuartzStairs;
    facets: Movable, Breakable;
    state: StairState;
}

block! {
    /// `smooth_red_sandstone` — Smooth Red Sandstone. `Movable | Breakable`
    pub struct SmoothRedSandstone;
    facets: Movable, Breakable;
}

block! {
    /// `smooth_red_sandstone_slab` — Smooth Red Sandstone Slab. `Movable | Breakable`
    pub struct SmoothRedSandstoneSlab;
    facets: Movable, Breakable;
    state: SlabState;
}

block! {
    /// `smooth_red_sandstone_stairs` — Smooth Red Sandstone Stairs. `Movable | Breakable`
    pub struct SmoothRedSandstoneStairs;
    facets: Movable, Breakable;
    state: StairState;
}

block! {
    /// `smooth_sandstone` — Smooth Sandstone. `Movable | Breakable`
    pub struct SmoothSandstone;
    facets: Movable, Breakable;
}

block! {
    /// `smooth_sandstone_slab` — Smooth Sandstone Slab. `Movable | Breakable`
    pub struct SmoothSandstoneSlab;
    facets: Movable, Breakable;
    state: SlabState;
}

block! {
    /// `smooth_sandstone_stairs` — Smooth Sandstone Stairs. `Movable | Breakable`
    pub struct SmoothSandstoneStairs;
    facets: Movable, Breakable;
    state: StairState;
}

block! {
    /// `smooth_stone` — Smooth Stone. `Movable | Breakable`
    pub struct SmoothStone;
    facets: Movable, Breakable;
}

block! {
    /// `smooth_stone_slab` — Smooth Stone Slab. `Movable | Breakable`
    pub struct SmoothStoneSlab;
    facets: Movable, Breakable;
    state: SlabState;
}

block! {
    /// `sniffer_egg` — Sniffer Egg. `Movable | Breakable`
    pub struct SnifferEgg;
    facets: Movable, Breakable;
    state: SnifferEggState;
}

block! {
    /// `snow` — Snow. `Breakable`
    pub struct Snow;
    facets: Breakable;
    state: SnowLayerState;
}

block! {
    /// `snow_block` — Snow Block. `Movable | Breakable`
    pub struct SnowBlock;
    facets: Movable, Breakable;
}

block! {
    /// `soul_campfire` — Soul Campfire. `Breakable`
    pub struct SoulCampfire;
    facets: Breakable;
    state: CampfireState;
}

block! {
    /// `soul_fire` — Soul Fire. `InstantMinable`
    pub struct SoulFire;
    facets: InstantMinable;
}

block! {
    /// `soul_lantern` — Soul Lantern. `Breakable`
    pub struct SoulLantern;
    facets: Breakable;
    state: WeatheringLanternState;
}

block! {
    /// `soul_sand` — Soul Sand. `Movable | Breakable`
    pub struct SoulSand;
    facets: Movable, Breakable;
}

block! {
    /// `soul_soil` — Soul Soil. `Movable | Breakable`
    pub struct SoulSoil;
    facets: Movable, Breakable;
}

block! {
    /// `soul_torch` — Soul Torch. `InstantMinable`
    pub struct SoulTorch;
    facets: InstantMinable;
}

block! {
    /// `soul_wall_torch` — Soul Torch. `InstantMinable`
    pub struct SoulWallTorch;
    facets: InstantMinable;
    state: FacingState;
}

block! {
    /// `spawner` — Monster Spawner. `UnMovable | Breakable`
    pub struct Spawner;
    facets: UnMovable, Breakable;
}

block! {
    /// `sponge` — Sponge. `Movable | Breakable`
    pub struct Sponge;
    facets: Movable, Breakable;
}

block! {
    /// `spore_blossom` — Spore Blossom. `Movable | InstantMinable | Flammable`
    pub struct SporeBlossom;
    facets: Movable, InstantMinable, Flammable;
}

block! {
    /// `spruce_button` — Spruce Button. `Breakable | Flammable`
    pub struct SpruceButton;
    facets: Breakable, Flammable;
    state: SwitchState;
}

block! {
    /// `spruce_door` — Spruce Door. `Breakable | Flammable`
    pub struct SpruceDoor;
    facets: Breakable, Flammable;
    state: DoorState;
}

block! {
    /// `spruce_fence` — Spruce Fence. `Movable | Breakable | Flammable`
    pub struct SpruceFence;
    facets: Movable, Breakable, Flammable;
    state: ConnectionState;
}

block! {
    /// `spruce_fence_gate` — Spruce Fence Gate. `Movable | Breakable | Flammable`
    pub struct SpruceFenceGate;
    facets: Movable, Breakable, Flammable;
    state: FenceGateState;
}

block! {
    /// `spruce_hanging_sign` — Spruce Hanging Sign. `Breakable | Flammable`
    pub struct SpruceHangingSign;
    facets: Breakable, Flammable;
    state: HangingSignState;
}

block! {
    /// `spruce_leaves` — Spruce Leaves. `Movable | Breakable | Flammable`
    pub struct SpruceLeaves;
    facets: Movable, Breakable, Flammable;
    state: LeavesState;
}

block! {
    /// `spruce_log` — Spruce Log. `Movable | Breakable | Flammable`
    pub struct SpruceLog;
    facets: Movable, Breakable, Flammable;
    state: PillarState;
}

block! {
    /// `spruce_planks` — Spruce Planks. `Movable | Breakable | Flammable`
    pub struct SprucePlanks;
    facets: Movable, Breakable, Flammable;
}

block! {
    /// `spruce_pressure_plate` — Spruce Pressure Plate. `Breakable | Flammable`
    pub struct SprucePressurePlate;
    facets: Breakable, Flammable;
    state: PressurePlateState;
}

block! {
    /// `spruce_sapling` — Spruce Sapling. `InstantMinable`
    pub struct SpruceSapling;
    facets: InstantMinable;
    state: SaplingState;
}

block! {
    /// `spruce_shelf` — Spruce Shelf. `UnMovable | Breakable`
    pub struct SpruceShelf;
    facets: UnMovable, Breakable;
    state: ShelfState;
}

block! {
    /// `spruce_sign` — Spruce Sign. `Breakable | Flammable`
    pub struct SpruceSign;
    facets: Breakable, Flammable;
    state: StandingSignState;
}

block! {
    /// `spruce_slab` — Spruce Slab. `Movable | Breakable | Flammable`
    pub struct SpruceSlab;
    facets: Movable, Breakable, Flammable;
    state: SlabState;
}

block! {
    /// `spruce_stairs` — Spruce Stairs. `Movable | Breakable | Flammable`
    pub struct SpruceStairs;
    facets: Movable, Breakable, Flammable;
    state: StairState;
}

block! {
    /// `spruce_trapdoor` — Spruce Trapdoor. `Movable | Breakable | Flammable`
    pub struct SpruceTrapdoor;
    facets: Movable, Breakable, Flammable;
    state: TrapdoorState;
}

block! {
    /// `spruce_wall_hanging_sign` — Spruce Hanging Sign. `Breakable | Flammable`
    pub struct SpruceWallHangingSign;
    facets: Breakable, Flammable;
    state: WallMountedState;
}

block! {
    /// `spruce_wall_sign` — Spruce Sign. `Breakable | Flammable`
    pub struct SpruceWallSign;
    facets: Breakable, Flammable;
    state: WallMountedState;
}

block! {
    /// `spruce_wood` — Spruce Wood. `Movable | Breakable | Flammable`
    pub struct SpruceWood;
    facets: Movable, Breakable, Flammable;
    state: PillarState;
}

block! {
    /// `sticky_piston` — Sticky Piston. `Movable | Breakable`
    pub struct StickyPiston;
    facets: Movable, Breakable;
    state: PistonBaseState;
}

block! {
    /// `stone` — Stone. `Movable | Breakable`
    pub struct Stone;
    facets: Movable, Breakable;
}

block! {
    /// `stone_brick_slab` — Stone Brick Slab. `Movable | Breakable`
    pub struct StoneBrickSlab;
    facets: Movable, Breakable;
    state: SlabState;
}

block! {
    /// `stone_brick_stairs` — Stone Brick Stairs. `Movable | Breakable`
    pub struct StoneBrickStairs;
    facets: Movable, Breakable;
    state: StairState;
}

block! {
    /// `stone_brick_wall` — Stone Brick Wall. `Movable | Breakable`
    pub struct StoneBrickWall;
    facets: Movable, Breakable;
    state: WallState;
}

block! {
    /// `stone_bricks` — Stone Bricks. `Movable | Breakable`
    pub struct StoneBricks;
    facets: Movable, Breakable;
}

block! {
    /// `stone_button` — Stone Button. `Breakable`
    pub struct StoneButton;
    facets: Breakable;
    state: SwitchState;
}

block! {
    /// `stone_pressure_plate` — Stone Pressure Plate. `Breakable`
    pub struct StonePressurePlate;
    facets: Breakable;
    state: PressurePlateState;
}

block! {
    /// `stone_slab` — Stone Slab. `Movable | Breakable`
    pub struct StoneSlab;
    facets: Movable, Breakable;
    state: SlabState;
}

block! {
    /// `stone_stairs` — Stone Stairs. `Movable | Breakable`
    pub struct StoneStairs;
    facets: Movable, Breakable;
    state: StairState;
}

block! {
    /// `stonecutter` — Stonecutter. `Movable | Breakable`
    pub struct Stonecutter;
    facets: Movable, Breakable;
    state: FacingState;
}

block! {
    /// `stripped_acacia_log` — Stripped Acacia Log. `Movable | Breakable | Flammable`
    pub struct StrippedAcaciaLog;
    facets: Movable, Breakable, Flammable;
    state: PillarState;
}

block! {
    /// `stripped_acacia_wood` — Stripped Acacia Wood. `Movable | Breakable | Flammable`
    pub struct StrippedAcaciaWood;
    facets: Movable, Breakable, Flammable;
    state: PillarState;
}

block! {
    /// `stripped_bamboo_block` — Block of Stripped Bamboo. `Movable | Breakable | Flammable`
    pub struct StrippedBambooBlock;
    facets: Movable, Breakable, Flammable;
    state: PillarState;
}

block! {
    /// `stripped_birch_log` — Stripped Birch Log. `Movable | Breakable | Flammable`
    pub struct StrippedBirchLog;
    facets: Movable, Breakable, Flammable;
    state: PillarState;
}

block! {
    /// `stripped_birch_wood` — Stripped Birch Wood. `Movable | Breakable | Flammable`
    pub struct StrippedBirchWood;
    facets: Movable, Breakable, Flammable;
    state: PillarState;
}

block! {
    /// `stripped_cherry_log` — Stripped Cherry Log. `Movable | Breakable | Flammable`
    pub struct StrippedCherryLog;
    facets: Movable, Breakable, Flammable;
    state: PillarState;
}

block! {
    /// `stripped_cherry_wood` — Stripped Cherry Wood. `Movable | Breakable | Flammable`
    pub struct StrippedCherryWood;
    facets: Movable, Breakable, Flammable;
    state: PillarState;
}

block! {
    /// `stripped_crimson_hyphae` — Stripped Crimson Hyphae. `Movable | Breakable`
    pub struct StrippedCrimsonHyphae;
    facets: Movable, Breakable;
    state: PillarState;
}

block! {
    /// `stripped_crimson_stem` — Stripped Crimson Stem. `Movable | Breakable`
    pub struct StrippedCrimsonStem;
    facets: Movable, Breakable;
    state: PillarState;
}

block! {
    /// `stripped_dark_oak_log` — Stripped Dark Oak Log. `Movable | Breakable | Flammable`
    pub struct StrippedDarkOakLog;
    facets: Movable, Breakable, Flammable;
    state: PillarState;
}

block! {
    /// `stripped_dark_oak_wood` — Stripped Dark Oak Wood. `Movable | Breakable | Flammable`
    pub struct StrippedDarkOakWood;
    facets: Movable, Breakable, Flammable;
    state: PillarState;
}

block! {
    /// `stripped_jungle_log` — Stripped Jungle Log. `Movable | Breakable | Flammable`
    pub struct StrippedJungleLog;
    facets: Movable, Breakable, Flammable;
    state: PillarState;
}

block! {
    /// `stripped_jungle_wood` — Stripped Jungle Wood. `Movable | Breakable | Flammable`
    pub struct StrippedJungleWood;
    facets: Movable, Breakable, Flammable;
    state: PillarState;
}

block! {
    /// `stripped_mangrove_log` — Stripped Mangrove Log. `Movable | Breakable | Flammable`
    pub struct StrippedMangroveLog;
    facets: Movable, Breakable, Flammable;
    state: PillarState;
}

block! {
    /// `stripped_mangrove_wood` — Stripped Mangrove Wood. `Movable | Breakable | Flammable`
    pub struct StrippedMangroveWood;
    facets: Movable, Breakable, Flammable;
    state: PillarState;
}

block! {
    /// `stripped_oak_log` — Stripped Oak Log. `Movable | Breakable | Flammable`
    pub struct StrippedOakLog;
    facets: Movable, Breakable, Flammable;
    state: PillarState;
}

block! {
    /// `stripped_oak_wood` — Stripped Oak Wood. `Movable | Breakable | Flammable`
    pub struct StrippedOakWood;
    facets: Movable, Breakable, Flammable;
    state: PillarState;
}

block! {
    /// `stripped_pale_oak_log` — Stripped Pale Oak Log. `Movable | Breakable | Flammable`
    pub struct StrippedPaleOakLog;
    facets: Movable, Breakable, Flammable;
    state: PillarState;
}

block! {
    /// `stripped_pale_oak_wood` — Stripped Pale Oak Wood. `Movable | Breakable | Flammable`
    pub struct StrippedPaleOakWood;
    facets: Movable, Breakable, Flammable;
    state: PillarState;
}

block! {
    /// `stripped_spruce_log` — Stripped Spruce Log. `Movable | Breakable | Flammable`
    pub struct StrippedSpruceLog;
    facets: Movable, Breakable, Flammable;
    state: PillarState;
}

block! {
    /// `stripped_spruce_wood` — Stripped Spruce Wood. `Movable | Breakable | Flammable`
    pub struct StrippedSpruceWood;
    facets: Movable, Breakable, Flammable;
    state: PillarState;
}

block! {
    /// `stripped_warped_hyphae` — Stripped Warped Hyphae. `Movable | Breakable`
    pub struct StrippedWarpedHyphae;
    facets: Movable, Breakable;
    state: PillarState;
}

block! {
    /// `stripped_warped_stem` — Stripped Warped Stem. `Movable | Breakable`
    pub struct StrippedWarpedStem;
    facets: Movable, Breakable;
    state: PillarState;
}

block! {
    /// `structure_block` — Structure Block. `UnMovable`
    pub struct StructureBlock;
    facets: UnMovable;
    state: StructureState;
}

block! {
    /// `structure_void` — Structure Void. `InstantMinable`
    pub struct StructureVoid;
    facets: InstantMinable;
}

block! {
    /// `sugar_cane` — Sugar Cane. `InstantMinable`
    pub struct SugarCane;
    facets: InstantMinable;
    state: CactusState;
}

block! {
    /// `sulfur` — Sulfur. `Movable | Breakable`
    pub struct Sulfur;
    facets: Movable, Breakable;
}

block! {
    /// `sulfur_brick_slab` — Sulfur Brick Slab. `Movable | Breakable`
    pub struct SulfurBrickSlab;
    facets: Movable, Breakable;
    state: SlabState;
}

block! {
    /// `sulfur_brick_stairs` — Sulfur Brick Stairs. `Movable | Breakable`
    pub struct SulfurBrickStairs;
    facets: Movable, Breakable;
    state: StairState;
}

block! {
    /// `sulfur_brick_wall` — Sulfur Brick Wall. `Movable | Breakable`
    pub struct SulfurBrickWall;
    facets: Movable, Breakable;
    state: WallState;
}

block! {
    /// `sulfur_bricks` — Sulfur Bricks. `Movable | Breakable`
    pub struct SulfurBricks;
    facets: Movable, Breakable;
}

block! {
    /// `sulfur_slab` — Sulfur Slab. `Movable | Breakable`
    pub struct SulfurSlab;
    facets: Movable, Breakable;
    state: SlabState;
}

block! {
    /// `sulfur_spike` — Sulfur Spike. `Breakable`
    pub struct SulfurSpike;
    facets: Breakable;
    state: PointedDripstoneState;
}

block! {
    /// `sulfur_stairs` — Sulfur Stairs. `Movable | Breakable`
    pub struct SulfurStairs;
    facets: Movable, Breakable;
    state: StairState;
}

block! {
    /// `sulfur_wall` — Sulfur Wall. `Movable | Breakable`
    pub struct SulfurWall;
    facets: Movable, Breakable;
    state: WallState;
}

block! {
    /// `sunflower` — Sunflower. `InstantMinable | Flammable`
    pub struct Sunflower;
    facets: InstantMinable, Flammable;
    state: DoublePlantState;
}

block! {
    /// `suspicious_gravel` — Suspicious Gravel. `Breakable`
    pub struct SuspiciousGravel;
    facets: Breakable;
    state: BrushableState;
}

block! {
    /// `suspicious_sand` — Suspicious Sand. `Breakable`
    pub struct SuspiciousSand;
    facets: Breakable;
    state: BrushableState;
}

block! {
    /// `sweet_berry_bush` — Sweet Berry Bush. `Movable | InstantMinable | Flammable`
    pub struct SweetBerryBush;
    facets: Movable, InstantMinable, Flammable;
    state: BeetrootState;
}

block! {
    /// `tall_dry_grass` — Tall Dry Grass. `InstantMinable | Flammable`
    pub struct TallDryGrass;
    facets: InstantMinable, Flammable;
}

block! {
    /// `tall_grass` — Tall Grass. `InstantMinable | Flammable`
    pub struct TallGrass;
    facets: InstantMinable, Flammable;
    state: DoublePlantState;
}

block! {
    /// `tall_seagrass` — Tall Seagrass. `Movable | InstantMinable`
    pub struct TallSeagrass;
    facets: Movable, InstantMinable;
    state: DoublePlantState;
}

block! {
    /// `target` — Target. `Movable | Breakable | Flammable`
    pub struct Target;
    facets: Movable, Breakable, Flammable;
    state: WeightedPressurePlateState;
}

block! {
    /// `terracotta` — Terracotta. `Movable | Breakable`
    pub struct Terracotta;
    facets: Movable, Breakable;
}

block! {
    /// `test_block` — Test Block. `UnMovable`
    pub struct TestBlock;
    facets: UnMovable;
    state: TestState;
}

block! {
    /// `test_instance_block` — Test Instance Block. `UnMovable`
    pub struct TestInstanceBlock;
    facets: UnMovable;
}

block! {
    /// `tinted_glass` — Tinted Glass. `Movable | Breakable`
    pub struct TintedGlass;
    facets: Movable, Breakable;
}

block! {
    /// `tnt` — TNT. `Movable | InstantMinable | Flammable`
    pub struct Tnt;
    facets: Movable, InstantMinable, Flammable;
    state: TntState;
}

block! {
    /// `torch` — Torch. `InstantMinable`
    pub struct Torch;
    facets: InstantMinable;
}

block! {
    /// `torchflower` — Torchflower. `InstantMinable | Flammable`
    pub struct Torchflower;
    facets: InstantMinable, Flammable;
}

block! {
    /// `torchflower_crop` — Torchflower Crop. `InstantMinable`
    pub struct TorchflowerCrop;
    facets: InstantMinable;
    state: TorchflowerCropState;
}

block! {
    /// `trapped_chest` — Trapped Chest. `UnMovable | Breakable`
    pub struct TrappedChest;
    facets: UnMovable, Breakable;
    state: ChestState;
}

block! {
    /// `trial_spawner` — Trial Spawner. `UnMovable | Breakable`
    pub struct TrialSpawner;
    facets: UnMovable, Breakable;
    state: TrialSpawnerState;
}

block! {
    /// `tripwire` — Tripwire. `InstantMinable`
    pub struct Tripwire;
    facets: InstantMinable;
    state: TripwireState;
}

block! {
    /// `tripwire_hook` — Tripwire Hook. `InstantMinable`
    pub struct TripwireHook;
    facets: InstantMinable;
    state: TripWireHookState;
}

block! {
    /// `tube_coral` — Tube Coral. `Movable | InstantMinable`
    pub struct TubeCoral;
    facets: Movable, InstantMinable;
    state: WaterloggedState;
}

block! {
    /// `tube_coral_block` — Tube Coral Block. `Movable | Breakable`
    pub struct TubeCoralBlock;
    facets: Movable, Breakable;
}

block! {
    /// `tube_coral_fan` — Tube Coral Fan. `Movable | InstantMinable`
    pub struct TubeCoralFan;
    facets: Movable, InstantMinable;
    state: WaterloggedState;
}

block! {
    /// `tube_coral_wall_fan` — Tube Coral Wall Fan. `Movable | InstantMinable`
    pub struct TubeCoralWallFan;
    facets: Movable, InstantMinable;
    state: WallMountedState;
}

block! {
    /// `tuff` — Tuff. `Movable | Breakable`
    pub struct Tuff;
    facets: Movable, Breakable;
}

block! {
    /// `tuff_brick_slab` — Tuff Brick Slab. `Movable | Breakable`
    pub struct TuffBrickSlab;
    facets: Movable, Breakable;
    state: SlabState;
}

block! {
    /// `tuff_brick_stairs` — Tuff Brick Stairs. `Movable | Breakable`
    pub struct TuffBrickStairs;
    facets: Movable, Breakable;
    state: StairState;
}

block! {
    /// `tuff_brick_wall` — Tuff Brick Wall. `Movable | Breakable`
    pub struct TuffBrickWall;
    facets: Movable, Breakable;
    state: WallState;
}

block! {
    /// `tuff_bricks` — Tuff Bricks. `Movable | Breakable`
    pub struct TuffBricks;
    facets: Movable, Breakable;
}

block! {
    /// `tuff_slab` — Tuff Slab. `Movable | Breakable`
    pub struct TuffSlab;
    facets: Movable, Breakable;
    state: SlabState;
}

block! {
    /// `tuff_stairs` — Tuff Stairs. `Movable | Breakable`
    pub struct TuffStairs;
    facets: Movable, Breakable;
    state: StairState;
}

block! {
    /// `tuff_wall` — Tuff Wall. `Movable | Breakable`
    pub struct TuffWall;
    facets: Movable, Breakable;
    state: WallState;
}

block! {
    /// `turtle_egg` — Turtle Egg. `Breakable`
    pub struct TurtleEgg;
    facets: Breakable;
    state: TurtleEggState;
}

block! {
    /// `twisting_vines` — Twisting Vines. `InstantMinable`
    pub struct TwistingVines;
    facets: InstantMinable;
    state: KelpState;
}

block! {
    /// `twisting_vines_plant` — Twisting Vines Plant. `InstantMinable`
    pub struct TwistingVinesPlant;
    facets: InstantMinable;
}

block! {
    /// `vault` — Vault. `UnMovable | Breakable`
    pub struct Vault;
    facets: UnMovable, Breakable;
    state: VaultState;
}

block! {
    /// `verdant_froglight` — Verdant Froglight. `Movable | Breakable`
    pub struct VerdantFroglight;
    facets: Movable, Breakable;
    state: PillarState;
}

block! {
    /// `vine` — Vines. `Breakable | Flammable`
    pub struct Vine;
    facets: Breakable, Flammable;
    state: VineState;
}

block! {
    /// `void_air` — Void Air. `Empty`
    pub struct VoidAir;
    facets: Empty;
}

block! {
    /// `wall_torch` — Torch. `InstantMinable`
    pub struct WallTorch;
    facets: InstantMinable;
    state: FacingState;
}

block! {
    /// `warped_button` — Warped Button. `Breakable`
    pub struct WarpedButton;
    facets: Breakable;
    state: SwitchState;
}

block! {
    /// `warped_door` — Warped Door. `Breakable`
    pub struct WarpedDoor;
    facets: Breakable;
    state: DoorState;
}

block! {
    /// `warped_fence` — Warped Fence. `Movable | Breakable`
    pub struct WarpedFence;
    facets: Movable, Breakable;
    state: ConnectionState;
}

block! {
    /// `warped_fence_gate` — Warped Fence Gate. `Movable | Breakable`
    pub struct WarpedFenceGate;
    facets: Movable, Breakable;
    state: FenceGateState;
}

block! {
    /// `warped_fungus` — Warped Fungus. `InstantMinable`
    pub struct WarpedFungus;
    facets: InstantMinable;
}

block! {
    /// `warped_hanging_sign` — Warped Hanging Sign. `Breakable`
    pub struct WarpedHangingSign;
    facets: Breakable;
    state: HangingSignState;
}

block! {
    /// `warped_hyphae` — Warped Hyphae. `Movable | Breakable`
    pub struct WarpedHyphae;
    facets: Movable, Breakable;
    state: PillarState;
}

block! {
    /// `warped_nylium` — Warped Nylium. `Movable | Breakable`
    pub struct WarpedNylium;
    facets: Movable, Breakable;
}

block! {
    /// `warped_planks` — Warped Planks. `Movable | Breakable`
    pub struct WarpedPlanks;
    facets: Movable, Breakable;
}

block! {
    /// `warped_pressure_plate` — Warped Pressure Plate. `Breakable`
    pub struct WarpedPressurePlate;
    facets: Breakable;
    state: PressurePlateState;
}

block! {
    /// `warped_roots` — Warped Roots. `Movable | InstantMinable`
    pub struct WarpedRoots;
    facets: Movable, InstantMinable;
}

block! {
    /// `warped_shelf` — Warped Shelf. `UnMovable | Breakable`
    pub struct WarpedShelf;
    facets: UnMovable, Breakable;
    state: ShelfState;
}

block! {
    /// `warped_sign` — Warped Sign. `Breakable`
    pub struct WarpedSign;
    facets: Breakable;
    state: StandingSignState;
}

block! {
    /// `warped_slab` — Warped Slab. `Movable | Breakable`
    pub struct WarpedSlab;
    facets: Movable, Breakable;
    state: SlabState;
}

block! {
    /// `warped_stairs` — Warped Stairs. `Movable | Breakable`
    pub struct WarpedStairs;
    facets: Movable, Breakable;
    state: StairState;
}

block! {
    /// `warped_stem` — Warped Stem. `Movable | Breakable`
    pub struct WarpedStem;
    facets: Movable, Breakable;
    state: PillarState;
}

block! {
    /// `warped_trapdoor` — Warped Trapdoor. `Movable | Breakable`
    pub struct WarpedTrapdoor;
    facets: Movable, Breakable;
    state: TrapdoorState;
}

block! {
    /// `warped_wall_hanging_sign` — Warped Hanging Sign. `Breakable`
    pub struct WarpedWallHangingSign;
    facets: Breakable;
    state: WallMountedState;
}

block! {
    /// `warped_wall_sign` — Warped Sign. `Breakable`
    pub struct WarpedWallSign;
    facets: Breakable;
    state: WallMountedState;
}

block! {
    /// `warped_wart_block` — Warped Wart Block. `Movable | Breakable`
    pub struct WarpedWartBlock;
    facets: Movable, Breakable;
}

block! {
    /// `water` — Water. `UnMovable`
    pub struct Water;
    facets: UnMovable;
    state: LiquidState;
}

block! {
    /// `water_cauldron` — Water Cauldron. `Movable | Breakable`
    pub struct WaterCauldron;
    facets: Movable, Breakable;
    state: LayeredCauldronState;
}

block! {
    /// `waxed_chiseled_copper` — Waxed Chiseled Copper. `Movable | Breakable`
    pub struct WaxedChiseledCopper;
    facets: Movable, Breakable;
}

block! {
    /// `waxed_copper_bars` — Waxed Copper Bars. `Movable | Breakable`
    pub struct WaxedCopperBars;
    facets: Movable, Breakable;
    state: ConnectionState;
}

block! {
    /// `waxed_copper_block` — Waxed Block of Copper. `Movable | Breakable`
    pub struct WaxedCopperBlock;
    facets: Movable, Breakable;
}

block! {
    /// `waxed_copper_bulb` — Waxed Copper Bulb. `Movable | Breakable`
    pub struct WaxedCopperBulb;
    facets: Movable, Breakable;
    state: WeatheringCopperBulbState;
}

block! {
    /// `waxed_copper_chain` — Waxed Copper Chain. `Movable | Breakable`
    pub struct WaxedCopperChain;
    facets: Movable, Breakable;
    state: WeatheringCopperChainState;
}

block! {
    /// `waxed_copper_chest` — Waxed Copper Chest. `UnMovable | Breakable`
    pub struct WaxedCopperChest;
    facets: UnMovable, Breakable;
    state: ChestState;
}

block! {
    /// `waxed_copper_door` — Waxed Copper Door. `Breakable`
    pub struct WaxedCopperDoor;
    facets: Breakable;
    state: DoorState;
}

block! {
    /// `waxed_copper_golem_statue` — Waxed Copper Golem Statue. `Movable | Breakable`
    pub struct WaxedCopperGolemStatue;
    facets: Movable, Breakable;
    state: WeatheringCopperGolemStatueState;
}

block! {
    /// `waxed_copper_grate` — Waxed Copper Grate. `Movable | Breakable`
    pub struct WaxedCopperGrate;
    facets: Movable, Breakable;
    state: WaterloggedState;
}

block! {
    /// `waxed_copper_lantern` — Waxed Copper Lantern. `Breakable`
    pub struct WaxedCopperLantern;
    facets: Breakable;
    state: WeatheringLanternState;
}

block! {
    /// `waxed_copper_trapdoor` — Waxed Copper Trapdoor. `Movable | Breakable`
    pub struct WaxedCopperTrapdoor;
    facets: Movable, Breakable;
    state: TrapdoorState;
}

block! {
    /// `waxed_cut_copper` — Waxed Cut Copper. `Movable | Breakable`
    pub struct WaxedCutCopper;
    facets: Movable, Breakable;
}

block! {
    /// `waxed_cut_copper_slab` — Waxed Cut Copper Slab. `Movable | Breakable`
    pub struct WaxedCutCopperSlab;
    facets: Movable, Breakable;
    state: SlabState;
}

block! {
    /// `waxed_cut_copper_stairs` — Waxed Cut Copper Stairs. `Movable | Breakable`
    pub struct WaxedCutCopperStairs;
    facets: Movable, Breakable;
    state: StairState;
}

block! {
    /// `waxed_exposed_chiseled_copper` — Waxed Exposed Chiseled Copper. `Movable | Breakable`
    pub struct WaxedExposedChiseledCopper;
    facets: Movable, Breakable;
}

block! {
    /// `waxed_exposed_copper` — Waxed Exposed Copper. `Movable | Breakable`
    pub struct WaxedExposedCopper;
    facets: Movable, Breakable;
}

block! {
    /// `waxed_exposed_copper_bars` — Waxed Exposed Copper Bars. `Movable | Breakable`
    pub struct WaxedExposedCopperBars;
    facets: Movable, Breakable;
    state: ConnectionState;
}

block! {
    /// `waxed_exposed_copper_bulb` — Waxed Exposed Copper Bulb. `Movable | Breakable`
    pub struct WaxedExposedCopperBulb;
    facets: Movable, Breakable;
    state: WeatheringCopperBulbState;
}

block! {
    /// `waxed_exposed_copper_chain` — Waxed Exposed Copper Chain. `Movable | Breakable`
    pub struct WaxedExposedCopperChain;
    facets: Movable, Breakable;
    state: WeatheringCopperChainState;
}

block! {
    /// `waxed_exposed_copper_chest` — Waxed Exposed Copper Chest. `UnMovable | Breakable`
    pub struct WaxedExposedCopperChest;
    facets: UnMovable, Breakable;
    state: ChestState;
}

block! {
    /// `waxed_exposed_copper_door` — Waxed Exposed Copper Door. `Breakable`
    pub struct WaxedExposedCopperDoor;
    facets: Breakable;
    state: DoorState;
}

block! {
    /// `waxed_exposed_copper_golem_statue` — Waxed Exposed Copper Golem Statue. `Movable | Breakable`
    pub struct WaxedExposedCopperGolemStatue;
    facets: Movable, Breakable;
    state: WeatheringCopperGolemStatueState;
}

block! {
    /// `waxed_exposed_copper_grate` — Waxed Exposed Copper Grate. `Movable | Breakable`
    pub struct WaxedExposedCopperGrate;
    facets: Movable, Breakable;
    state: WaterloggedState;
}

block! {
    /// `waxed_exposed_copper_lantern` — Waxed Exposed Copper Lantern. `Breakable`
    pub struct WaxedExposedCopperLantern;
    facets: Breakable;
    state: WeatheringLanternState;
}

block! {
    /// `waxed_exposed_copper_trapdoor` — Waxed Exposed Copper Trapdoor. `Movable | Breakable`
    pub struct WaxedExposedCopperTrapdoor;
    facets: Movable, Breakable;
    state: TrapdoorState;
}

block! {
    /// `waxed_exposed_cut_copper` — Waxed Exposed Cut Copper. `Movable | Breakable`
    pub struct WaxedExposedCutCopper;
    facets: Movable, Breakable;
}

block! {
    /// `waxed_exposed_cut_copper_slab` — Waxed Exposed Cut Copper Slab. `Movable | Breakable`
    pub struct WaxedExposedCutCopperSlab;
    facets: Movable, Breakable;
    state: SlabState;
}

block! {
    /// `waxed_exposed_cut_copper_stairs` — Waxed Exposed Cut Copper Stairs. `Movable | Breakable`
    pub struct WaxedExposedCutCopperStairs;
    facets: Movable, Breakable;
    state: StairState;
}

block! {
    /// `waxed_exposed_lightning_rod` — Waxed Exposed Lightning Rod. `Movable | Breakable`
    pub struct WaxedExposedLightningRod;
    facets: Movable, Breakable;
    state: WeatheringLightningRodState;
}

block! {
    /// `waxed_lightning_rod` — Waxed Lightning Rod. `Movable | Breakable`
    pub struct WaxedLightningRod;
    facets: Movable, Breakable;
    state: WeatheringLightningRodState;
}

block! {
    /// `waxed_oxidized_chiseled_copper` — Waxed Oxidized Chiseled Copper. `Movable | Breakable`
    pub struct WaxedOxidizedChiseledCopper;
    facets: Movable, Breakable;
}

block! {
    /// `waxed_oxidized_copper` — Waxed Oxidized Copper. `Movable | Breakable`
    pub struct WaxedOxidizedCopper;
    facets: Movable, Breakable;
}

block! {
    /// `waxed_oxidized_copper_bars` — Waxed Oxidized Copper Bars. `Movable | Breakable`
    pub struct WaxedOxidizedCopperBars;
    facets: Movable, Breakable;
    state: ConnectionState;
}

block! {
    /// `waxed_oxidized_copper_bulb` — Waxed Oxidized Copper Bulb. `Movable | Breakable`
    pub struct WaxedOxidizedCopperBulb;
    facets: Movable, Breakable;
    state: WeatheringCopperBulbState;
}

block! {
    /// `waxed_oxidized_copper_chain` — Waxed Oxidized Copper Chain. `Movable | Breakable`
    pub struct WaxedOxidizedCopperChain;
    facets: Movable, Breakable;
    state: WeatheringCopperChainState;
}

block! {
    /// `waxed_oxidized_copper_chest` — Waxed Oxidized Copper Chest. `UnMovable | Breakable`
    pub struct WaxedOxidizedCopperChest;
    facets: UnMovable, Breakable;
    state: ChestState;
}

block! {
    /// `waxed_oxidized_copper_door` — Waxed Oxidized Copper Door. `Breakable`
    pub struct WaxedOxidizedCopperDoor;
    facets: Breakable;
    state: DoorState;
}

block! {
    /// `waxed_oxidized_copper_golem_statue` — Waxed Oxidized Copper Golem Statue. `Movable | Breakable`
    pub struct WaxedOxidizedCopperGolemStatue;
    facets: Movable, Breakable;
    state: WeatheringCopperGolemStatueState;
}

block! {
    /// `waxed_oxidized_copper_grate` — Waxed Oxidized Copper Grate. `Movable | Breakable`
    pub struct WaxedOxidizedCopperGrate;
    facets: Movable, Breakable;
    state: WaterloggedState;
}

block! {
    /// `waxed_oxidized_copper_lantern` — Waxed Oxidized Copper Lantern. `Breakable`
    pub struct WaxedOxidizedCopperLantern;
    facets: Breakable;
    state: WeatheringLanternState;
}

block! {
    /// `waxed_oxidized_copper_trapdoor` — Waxed Oxidized Copper Trapdoor. `Movable | Breakable`
    pub struct WaxedOxidizedCopperTrapdoor;
    facets: Movable, Breakable;
    state: TrapdoorState;
}

block! {
    /// `waxed_oxidized_cut_copper` — Waxed Oxidized Cut Copper. `Movable | Breakable`
    pub struct WaxedOxidizedCutCopper;
    facets: Movable, Breakable;
}

block! {
    /// `waxed_oxidized_cut_copper_slab` — Waxed Oxidized Cut Copper Slab. `Movable | Breakable`
    pub struct WaxedOxidizedCutCopperSlab;
    facets: Movable, Breakable;
    state: SlabState;
}

block! {
    /// `waxed_oxidized_cut_copper_stairs` — Waxed Oxidized Cut Copper Stairs. `Movable | Breakable`
    pub struct WaxedOxidizedCutCopperStairs;
    facets: Movable, Breakable;
    state: StairState;
}

block! {
    /// `waxed_oxidized_lightning_rod` — Waxed Oxidized Lightning Rod. `Movable | Breakable`
    pub struct WaxedOxidizedLightningRod;
    facets: Movable, Breakable;
    state: WeatheringLightningRodState;
}

block! {
    /// `waxed_weathered_chiseled_copper` — Waxed Weathered Chiseled Copper. `Movable | Breakable`
    pub struct WaxedWeatheredChiseledCopper;
    facets: Movable, Breakable;
}

block! {
    /// `waxed_weathered_copper` — Waxed Weathered Copper. `Movable | Breakable`
    pub struct WaxedWeatheredCopper;
    facets: Movable, Breakable;
}

block! {
    /// `waxed_weathered_copper_bars` — Waxed Weathered Copper Bars. `Movable | Breakable`
    pub struct WaxedWeatheredCopperBars;
    facets: Movable, Breakable;
    state: ConnectionState;
}

block! {
    /// `waxed_weathered_copper_bulb` — Waxed Weathered Copper Bulb. `Movable | Breakable`
    pub struct WaxedWeatheredCopperBulb;
    facets: Movable, Breakable;
    state: WeatheringCopperBulbState;
}

block! {
    /// `waxed_weathered_copper_chain` — Waxed Weathered Copper Chain. `Movable | Breakable`
    pub struct WaxedWeatheredCopperChain;
    facets: Movable, Breakable;
    state: WeatheringCopperChainState;
}

block! {
    /// `waxed_weathered_copper_chest` — Waxed Weathered Copper Chest. `UnMovable | Breakable`
    pub struct WaxedWeatheredCopperChest;
    facets: UnMovable, Breakable;
    state: ChestState;
}

block! {
    /// `waxed_weathered_copper_door` — Waxed Weathered Copper Door. `Breakable`
    pub struct WaxedWeatheredCopperDoor;
    facets: Breakable;
    state: DoorState;
}

block! {
    /// `waxed_weathered_copper_golem_statue` — Waxed Weathered Copper Golem Statue. `Movable | Breakable`
    pub struct WaxedWeatheredCopperGolemStatue;
    facets: Movable, Breakable;
    state: WeatheringCopperGolemStatueState;
}

block! {
    /// `waxed_weathered_copper_grate` — Waxed Weathered Copper Grate. `Movable | Breakable`
    pub struct WaxedWeatheredCopperGrate;
    facets: Movable, Breakable;
    state: WaterloggedState;
}

block! {
    /// `waxed_weathered_copper_lantern` — Waxed Weathered Copper Lantern. `Breakable`
    pub struct WaxedWeatheredCopperLantern;
    facets: Breakable;
    state: WeatheringLanternState;
}

block! {
    /// `waxed_weathered_copper_trapdoor` — Waxed Weathered Copper Trapdoor. `Movable | Breakable`
    pub struct WaxedWeatheredCopperTrapdoor;
    facets: Movable, Breakable;
    state: TrapdoorState;
}

block! {
    /// `waxed_weathered_cut_copper` — Waxed Weathered Cut Copper. `Movable | Breakable`
    pub struct WaxedWeatheredCutCopper;
    facets: Movable, Breakable;
}

block! {
    /// `waxed_weathered_cut_copper_slab` — Waxed Weathered Cut Copper Slab. `Movable | Breakable`
    pub struct WaxedWeatheredCutCopperSlab;
    facets: Movable, Breakable;
    state: SlabState;
}

block! {
    /// `waxed_weathered_cut_copper_stairs` — Waxed Weathered Cut Copper Stairs. `Movable | Breakable`
    pub struct WaxedWeatheredCutCopperStairs;
    facets: Movable, Breakable;
    state: StairState;
}

block! {
    /// `waxed_weathered_lightning_rod` — Waxed Weathered Lightning Rod. `Movable | Breakable`
    pub struct WaxedWeatheredLightningRod;
    facets: Movable, Breakable;
    state: WeatheringLightningRodState;
}

block! {
    /// `weathered_chiseled_copper` — Weathered Chiseled Copper. `Movable | Breakable`
    pub struct WeatheredChiseledCopper;
    facets: Movable, Breakable;
}

block! {
    /// `weathered_copper` — Weathered Copper. `Movable | Breakable`
    pub struct WeatheredCopper;
    facets: Movable, Breakable;
}

block! {
    /// `weathered_copper_bars` — Weathered Copper Bars. `Movable | Breakable`
    pub struct WeatheredCopperBars;
    facets: Movable, Breakable;
    state: ConnectionState;
}

block! {
    /// `weathered_copper_bulb` — Weathered Copper Bulb. `Movable | Breakable`
    pub struct WeatheredCopperBulb;
    facets: Movable, Breakable;
    state: WeatheringCopperBulbState;
}

block! {
    /// `weathered_copper_chain` — Weathered Copper Chain. `Movable | Breakable`
    pub struct WeatheredCopperChain;
    facets: Movable, Breakable;
    state: WeatheringCopperChainState;
}

block! {
    /// `weathered_copper_chest` — Weathered Copper Chest. `UnMovable | Breakable`
    pub struct WeatheredCopperChest;
    facets: UnMovable, Breakable;
    state: ChestState;
}

block! {
    /// `weathered_copper_door` — Weathered Copper Door. `Breakable`
    pub struct WeatheredCopperDoor;
    facets: Breakable;
    state: DoorState;
}

block! {
    /// `weathered_copper_golem_statue` — Weathered Copper Golem Statue. `Movable | Breakable`
    pub struct WeatheredCopperGolemStatue;
    facets: Movable, Breakable;
    state: WeatheringCopperGolemStatueState;
}

block! {
    /// `weathered_copper_grate` — Weathered Copper Grate. `Movable | Breakable`
    pub struct WeatheredCopperGrate;
    facets: Movable, Breakable;
    state: WaterloggedState;
}

block! {
    /// `weathered_copper_lantern` — Weathered Copper Lantern. `Breakable`
    pub struct WeatheredCopperLantern;
    facets: Breakable;
    state: WeatheringLanternState;
}

block! {
    /// `weathered_copper_trapdoor` — Weathered Copper Trapdoor. `Movable | Breakable`
    pub struct WeatheredCopperTrapdoor;
    facets: Movable, Breakable;
    state: TrapdoorState;
}

block! {
    /// `weathered_cut_copper` — Weathered Cut Copper. `Movable | Breakable`
    pub struct WeatheredCutCopper;
    facets: Movable, Breakable;
}

block! {
    /// `weathered_cut_copper_slab` — Weathered Cut Copper Slab. `Movable | Breakable`
    pub struct WeatheredCutCopperSlab;
    facets: Movable, Breakable;
    state: SlabState;
}

block! {
    /// `weathered_cut_copper_stairs` — Weathered Cut Copper Stairs. `Movable | Breakable`
    pub struct WeatheredCutCopperStairs;
    facets: Movable, Breakable;
    state: StairState;
}

block! {
    /// `weathered_lightning_rod` — Weathered Lightning Rod. `Movable | Breakable`
    pub struct WeatheredLightningRod;
    facets: Movable, Breakable;
    state: WeatheringLightningRodState;
}

block! {
    /// `weeping_vines` — Weeping Vines. `InstantMinable`
    pub struct WeepingVines;
    facets: InstantMinable;
    state: KelpState;
}

block! {
    /// `weeping_vines_plant` — Weeping Vines Plant. `InstantMinable`
    pub struct WeepingVinesPlant;
    facets: InstantMinable;
}

block! {
    /// `wet_sponge` — Wet Sponge. `Movable | Breakable`
    pub struct WetSponge;
    facets: Movable, Breakable;
}

block! {
    /// `wheat` — Wheat Crops. `InstantMinable`
    pub struct Wheat;
    facets: InstantMinable;
    state: CropAgeState;
}

block! {
    /// `white_banner` — White Banner. `Breakable`
    pub struct WhiteBanner;
    facets: Breakable;
    state: BannerState;
}

block! {
    /// `white_bed` — White Bed. `Breakable`
    pub struct WhiteBed;
    facets: Breakable;
    state: BedState;
}

block! {
    /// `white_candle` — White Candle. `Movable | Breakable`
    pub struct WhiteCandle;
    facets: Movable, Breakable;
    state: CandleState;
}

block! {
    /// `white_candle_cake` — Cake with White Candle. `Movable | Breakable`
    pub struct WhiteCandleCake;
    facets: Movable, Breakable;
    state: LitState;
}

block! {
    /// `white_carpet` — White Carpet. `Movable | Breakable | Flammable`
    pub struct WhiteCarpet;
    facets: Movable, Breakable, Flammable;
}

block! {
    /// `white_concrete` — White Concrete. `Movable | Breakable`
    pub struct WhiteConcrete;
    facets: Movable, Breakable;
}

block! {
    /// `white_concrete_powder` — White Concrete Powder. `Movable | Breakable`
    pub struct WhiteConcretePowder;
    facets: Movable, Breakable;
}

block! {
    /// `white_glazed_terracotta` — White Glazed Terracotta. `Movable | Breakable`
    pub struct WhiteGlazedTerracotta;
    facets: Movable, Breakable;
    state: FacingState;
}

block! {
    /// `white_shulker_box` — White Shulker Box. `Breakable`
    pub struct WhiteShulkerBox;
    facets: Breakable;
    state: ShulkerBoxState;
}

block! {
    /// `white_stained_glass` — White Stained Glass. `Movable | Breakable`
    pub struct WhiteStainedGlass;
    facets: Movable, Breakable;
}

block! {
    /// `white_stained_glass_pane` — White Stained Glass Pane. `Movable | Breakable`
    pub struct WhiteStainedGlassPane;
    facets: Movable, Breakable;
    state: ConnectionState;
}

block! {
    /// `white_terracotta` — White Terracotta. `Movable | Breakable`
    pub struct WhiteTerracotta;
    facets: Movable, Breakable;
}

block! {
    /// `white_tulip` — White Tulip. `InstantMinable | Flammable`
    pub struct WhiteTulip;
    facets: InstantMinable, Flammable;
}

block! {
    /// `white_wall_banner` — White Banner. `Breakable`
    pub struct WhiteWallBanner;
    facets: Breakable;
    state: FacingState;
}

block! {
    /// `white_wool` — White Wool. `Movable | Breakable | Flammable`
    pub struct WhiteWool;
    facets: Movable, Breakable, Flammable;
}

block! {
    /// `wildflowers` — Wildflowers. `InstantMinable | Flammable`
    pub struct Wildflowers;
    facets: InstantMinable, Flammable;
    state: FlowerBedState;
}

block! {
    /// `wither_rose` — Wither Rose. `InstantMinable | Flammable`
    pub struct WitherRose;
    facets: InstantMinable, Flammable;
}

block! {
    /// `wither_skeleton_skull` — Wither Skeleton Skull. `Breakable`
    pub struct WitherSkeletonSkull;
    facets: Breakable;
    state: SkullState;
}

block! {
    /// `wither_skeleton_wall_skull` — Wither Skeleton Skull. `Breakable`
    pub struct WitherSkeletonWallSkull;
    facets: Breakable;
    state: WallSkullState;
}

block! {
    /// `yellow_banner` — Yellow Banner. `Breakable`
    pub struct YellowBanner;
    facets: Breakable;
    state: BannerState;
}

block! {
    /// `yellow_bed` — Yellow Bed. `Breakable`
    pub struct YellowBed;
    facets: Breakable;
    state: BedState;
}

block! {
    /// `yellow_candle` — Yellow Candle. `Movable | Breakable`
    pub struct YellowCandle;
    facets: Movable, Breakable;
    state: CandleState;
}

block! {
    /// `yellow_candle_cake` — Cake with Yellow Candle. `Movable | Breakable`
    pub struct YellowCandleCake;
    facets: Movable, Breakable;
    state: LitState;
}

block! {
    /// `yellow_carpet` — Yellow Carpet. `Movable | Breakable | Flammable`
    pub struct YellowCarpet;
    facets: Movable, Breakable, Flammable;
}

block! {
    /// `yellow_concrete` — Yellow Concrete. `Movable | Breakable`
    pub struct YellowConcrete;
    facets: Movable, Breakable;
}

block! {
    /// `yellow_concrete_powder` — Yellow Concrete Powder. `Movable | Breakable`
    pub struct YellowConcretePowder;
    facets: Movable, Breakable;
}

block! {
    /// `yellow_glazed_terracotta` — Yellow Glazed Terracotta. `Movable | Breakable`
    pub struct YellowGlazedTerracotta;
    facets: Movable, Breakable;
    state: FacingState;
}

block! {
    /// `yellow_shulker_box` — Yellow Shulker Box. `Breakable`
    pub struct YellowShulkerBox;
    facets: Breakable;
    state: ShulkerBoxState;
}

block! {
    /// `yellow_stained_glass` — Yellow Stained Glass. `Movable | Breakable`
    pub struct YellowStainedGlass;
    facets: Movable, Breakable;
}

block! {
    /// `yellow_stained_glass_pane` — Yellow Stained Glass Pane. `Movable | Breakable`
    pub struct YellowStainedGlassPane;
    facets: Movable, Breakable;
    state: ConnectionState;
}

block! {
    /// `yellow_terracotta` — Yellow Terracotta. `Movable | Breakable`
    pub struct YellowTerracotta;
    facets: Movable, Breakable;
}

block! {
    /// `yellow_wall_banner` — Yellow Banner. `Breakable`
    pub struct YellowWallBanner;
    facets: Breakable;
    state: FacingState;
}

block! {
    /// `yellow_wool` — Yellow Wool. `Movable | Breakable | Flammable`
    pub struct YellowWool;
    facets: Movable, Breakable, Flammable;
}

block! {
    /// `zombie_head` — Zombie Head. `Breakable`
    pub struct ZombieHead;
    facets: Breakable;
    state: SkullState;
}

block! {
    /// `zombie_wall_head` — Zombie Head. `Breakable`
    pub struct ZombieWallHead;
    facets: Breakable;
    state: WallSkullState;
}

/// Every Minecraft 26.2 block, in one palette.
pub struct AllBlocks {
    palette: BlockPalette<1196>,
}

impl AllBlocks {
    /// Number of registered blocks.
    pub const LEN: usize = 1196;

    /// Builds the full 26.2 palette.
    pub fn new() -> Self {
        Self {
            palette: palette![
                AcaciaButton::source(),
                AcaciaDoor::source(),
                AcaciaFence::source(),
                AcaciaFenceGate::source(),
                AcaciaHangingSign::source(),
                AcaciaLeaves::source(),
                AcaciaLog::source(),
                AcaciaPlanks::source(),
                AcaciaPressurePlate::source(),
                AcaciaSapling::source(),
                AcaciaShelf::source(),
                AcaciaSign::source(),
                AcaciaSlab::source(),
                AcaciaStairs::source(),
                AcaciaTrapdoor::source(),
                AcaciaWallHangingSign::source(),
                AcaciaWallSign::source(),
                AcaciaWood::source(),
                ActivatorRail::source(),
                Air::source(),
                Allium::source(),
                AmethystBlock::source(),
                AmethystCluster::source(),
                AncientDebris::source(),
                Andesite::source(),
                AndesiteSlab::source(),
                AndesiteStairs::source(),
                AndesiteWall::source(),
                Anvil::source(),
                AttachedMelonStem::source(),
                AttachedPumpkinStem::source(),
                Azalea::source(),
                AzaleaLeaves::source(),
                AzureBluet::source(),
                Bamboo::source(),
                BambooBlock::source(),
                BambooButton::source(),
                BambooDoor::source(),
                BambooFence::source(),
                BambooFenceGate::source(),
                BambooHangingSign::source(),
                BambooMosaic::source(),
                BambooMosaicSlab::source(),
                BambooMosaicStairs::source(),
                BambooPlanks::source(),
                BambooPressurePlate::source(),
                BambooSapling::source(),
                BambooShelf::source(),
                BambooSign::source(),
                BambooSlab::source(),
                BambooStairs::source(),
                BambooTrapdoor::source(),
                BambooWallHangingSign::source(),
                BambooWallSign::source(),
                Barrel::source(),
                Barrier::source(),
                Basalt::source(),
                Beacon::source(),
                Bedrock::source(),
                BeeNest::source(),
                Beehive::source(),
                Beetroots::source(),
                Bell::source(),
                BigDripleaf::source(),
                BigDripleafStem::source(),
                BirchButton::source(),
                BirchDoor::source(),
                BirchFence::source(),
                BirchFenceGate::source(),
                BirchHangingSign::source(),
                BirchLeaves::source(),
                BirchLog::source(),
                BirchPlanks::source(),
                BirchPressurePlate::source(),
                BirchSapling::source(),
                BirchShelf::source(),
                BirchSign::source(),
                BirchSlab::source(),
                BirchStairs::source(),
                BirchTrapdoor::source(),
                BirchWallHangingSign::source(),
                BirchWallSign::source(),
                BirchWood::source(),
                BlackBanner::source(),
                BlackBed::source(),
                BlackCandle::source(),
                BlackCandleCake::source(),
                BlackCarpet::source(),
                BlackConcrete::source(),
                BlackConcretePowder::source(),
                BlackGlazedTerracotta::source(),
                BlackShulkerBox::source(),
                BlackStainedGlass::source(),
                BlackStainedGlassPane::source(),
                BlackTerracotta::source(),
                BlackWallBanner::source(),
                BlackWool::source(),
                Blackstone::source(),
                BlackstoneSlab::source(),
                BlackstoneStairs::source(),
                BlackstoneWall::source(),
                BlastFurnace::source(),
                BlueBanner::source(),
                BlueBed::source(),
                BlueCandle::source(),
                BlueCandleCake::source(),
                BlueCarpet::source(),
                BlueConcrete::source(),
                BlueConcretePowder::source(),
                BlueGlazedTerracotta::source(),
                BlueIce::source(),
                BlueOrchid::source(),
                BlueShulkerBox::source(),
                BlueStainedGlass::source(),
                BlueStainedGlassPane::source(),
                BlueTerracotta::source(),
                BlueWallBanner::source(),
                BlueWool::source(),
                BoneBlock::source(),
                Bookshelf::source(),
                BrainCoral::source(),
                BrainCoralBlock::source(),
                BrainCoralFan::source(),
                BrainCoralWallFan::source(),
                BrewingStand::source(),
                BrickSlab::source(),
                BrickStairs::source(),
                BrickWall::source(),
                Bricks::source(),
                BrownBanner::source(),
                BrownBed::source(),
                BrownCandle::source(),
                BrownCandleCake::source(),
                BrownCarpet::source(),
                BrownConcrete::source(),
                BrownConcretePowder::source(),
                BrownGlazedTerracotta::source(),
                BrownMushroom::source(),
                BrownMushroomBlock::source(),
                BrownShulkerBox::source(),
                BrownStainedGlass::source(),
                BrownStainedGlassPane::source(),
                BrownTerracotta::source(),
                BrownWallBanner::source(),
                BrownWool::source(),
                BubbleColumn::source(),
                BubbleCoral::source(),
                BubbleCoralBlock::source(),
                BubbleCoralFan::source(),
                BubbleCoralWallFan::source(),
                BuddingAmethyst::source(),
                Bush::source(),
                Cactus::source(),
                CactusFlower::source(),
                Cake::source(),
                Calcite::source(),
                CalibratedSculkSensor::source(),
                Campfire::source(),
                Candle::source(),
                CandleCake::source(),
                Carrots::source(),
                CartographyTable::source(),
                CarvedPumpkin::source(),
                Cauldron::source(),
                CaveAir::source(),
                CaveVines::source(),
                CaveVinesPlant::source(),
                ChainCommandBlock::source(),
                CherryButton::source(),
                CherryDoor::source(),
                CherryFence::source(),
                CherryFenceGate::source(),
                CherryHangingSign::source(),
                CherryLeaves::source(),
                CherryLog::source(),
                CherryPlanks::source(),
                CherryPressurePlate::source(),
                CherrySapling::source(),
                CherryShelf::source(),
                CherrySign::source(),
                CherrySlab::source(),
                CherryStairs::source(),
                CherryTrapdoor::source(),
                CherryWallHangingSign::source(),
                CherryWallSign::source(),
                CherryWood::source(),
                Chest::source(),
                ChippedAnvil::source(),
                ChiseledBookshelf::source(),
                ChiseledCinnabar::source(),
                ChiseledCopper::source(),
                ChiseledDeepslate::source(),
                ChiseledNetherBricks::source(),
                ChiseledPolishedBlackstone::source(),
                ChiseledQuartzBlock::source(),
                ChiseledRedSandstone::source(),
                ChiseledResinBricks::source(),
                ChiseledSandstone::source(),
                ChiseledStoneBricks::source(),
                ChiseledSulfur::source(),
                ChiseledTuff::source(),
                ChiseledTuffBricks::source(),
                ChorusFlower::source(),
                ChorusPlant::source(),
                Cinnabar::source(),
                CinnabarBrickSlab::source(),
                CinnabarBrickStairs::source(),
                CinnabarBrickWall::source(),
                CinnabarBricks::source(),
                CinnabarSlab::source(),
                CinnabarStairs::source(),
                CinnabarWall::source(),
                Clay::source(),
                ClosedEyeblossom::source(),
                CoalBlock::source(),
                CoalOre::source(),
                CoarseDirt::source(),
                CobbledDeepslate::source(),
                CobbledDeepslateSlab::source(),
                CobbledDeepslateStairs::source(),
                CobbledDeepslateWall::source(),
                Cobblestone::source(),
                CobblestoneSlab::source(),
                CobblestoneStairs::source(),
                CobblestoneWall::source(),
                Cobweb::source(),
                Cocoa::source(),
                CommandBlock::source(),
                Comparator::source(),
                Composter::source(),
                Conduit::source(),
                CopperBars::source(),
                CopperBlock::source(),
                CopperBulb::source(),
                CopperChain::source(),
                CopperChest::source(),
                CopperDoor::source(),
                CopperGolemStatue::source(),
                CopperGrate::source(),
                CopperLantern::source(),
                CopperOre::source(),
                CopperTorch::source(),
                CopperTrapdoor::source(),
                CopperWallTorch::source(),
                Cornflower::source(),
                CrackedDeepslateBricks::source(),
                CrackedDeepslateTiles::source(),
                CrackedNetherBricks::source(),
                CrackedPolishedBlackstoneBricks::source(),
                CrackedStoneBricks::source(),
                Crafter::source(),
                CraftingTable::source(),
                CreakingHeart::source(),
                CreeperHead::source(),
                CreeperWallHead::source(),
                CrimsonButton::source(),
                CrimsonDoor::source(),
                CrimsonFence::source(),
                CrimsonFenceGate::source(),
                CrimsonFungus::source(),
                CrimsonHangingSign::source(),
                CrimsonHyphae::source(),
                CrimsonNylium::source(),
                CrimsonPlanks::source(),
                CrimsonPressurePlate::source(),
                CrimsonRoots::source(),
                CrimsonShelf::source(),
                CrimsonSign::source(),
                CrimsonSlab::source(),
                CrimsonStairs::source(),
                CrimsonStem::source(),
                CrimsonTrapdoor::source(),
                CrimsonWallHangingSign::source(),
                CrimsonWallSign::source(),
                CryingObsidian::source(),
                CutCopper::source(),
                CutCopperSlab::source(),
                CutCopperStairs::source(),
                CutRedSandstone::source(),
                CutRedSandstoneSlab::source(),
                CutSandstone::source(),
                CutSandstoneSlab::source(),
                CyanBanner::source(),
                CyanBed::source(),
                CyanCandle::source(),
                CyanCandleCake::source(),
                CyanCarpet::source(),
                CyanConcrete::source(),
                CyanConcretePowder::source(),
                CyanGlazedTerracotta::source(),
                CyanShulkerBox::source(),
                CyanStainedGlass::source(),
                CyanStainedGlassPane::source(),
                CyanTerracotta::source(),
                CyanWallBanner::source(),
                CyanWool::source(),
                DamagedAnvil::source(),
                Dandelion::source(),
                DarkOakButton::source(),
                DarkOakDoor::source(),
                DarkOakFence::source(),
                DarkOakFenceGate::source(),
                DarkOakHangingSign::source(),
                DarkOakLeaves::source(),
                DarkOakLog::source(),
                DarkOakPlanks::source(),
                DarkOakPressurePlate::source(),
                DarkOakSapling::source(),
                DarkOakShelf::source(),
                DarkOakSign::source(),
                DarkOakSlab::source(),
                DarkOakStairs::source(),
                DarkOakTrapdoor::source(),
                DarkOakWallHangingSign::source(),
                DarkOakWallSign::source(),
                DarkOakWood::source(),
                DarkPrismarine::source(),
                DarkPrismarineSlab::source(),
                DarkPrismarineStairs::source(),
                DaylightDetector::source(),
                DeadBrainCoral::source(),
                DeadBrainCoralBlock::source(),
                DeadBrainCoralFan::source(),
                DeadBrainCoralWallFan::source(),
                DeadBubbleCoral::source(),
                DeadBubbleCoralBlock::source(),
                DeadBubbleCoralFan::source(),
                DeadBubbleCoralWallFan::source(),
                DeadBush::source(),
                DeadFireCoral::source(),
                DeadFireCoralBlock::source(),
                DeadFireCoralFan::source(),
                DeadFireCoralWallFan::source(),
                DeadHornCoral::source(),
                DeadHornCoralBlock::source(),
                DeadHornCoralFan::source(),
                DeadHornCoralWallFan::source(),
                DeadTubeCoral::source(),
                DeadTubeCoralBlock::source(),
                DeadTubeCoralFan::source(),
                DeadTubeCoralWallFan::source(),
                DecoratedPot::source(),
                Deepslate::source(),
                DeepslateBrickSlab::source(),
                DeepslateBrickStairs::source(),
                DeepslateBrickWall::source(),
                DeepslateBricks::source(),
                DeepslateCoalOre::source(),
                DeepslateCopperOre::source(),
                DeepslateDiamondOre::source(),
                DeepslateEmeraldOre::source(),
                DeepslateGoldOre::source(),
                DeepslateIronOre::source(),
                DeepslateLapisOre::source(),
                DeepslateRedstoneOre::source(),
                DeepslateTileSlab::source(),
                DeepslateTileStairs::source(),
                DeepslateTileWall::source(),
                DeepslateTiles::source(),
                DetectorRail::source(),
                DiamondBlock::source(),
                DiamondOre::source(),
                Diorite::source(),
                DioriteSlab::source(),
                DioriteStairs::source(),
                DioriteWall::source(),
                Dirt::source(),
                DirtPath::source(),
                Dispenser::source(),
                DragonEgg::source(),
                DragonHead::source(),
                DragonWallHead::source(),
                DriedGhast::source(),
                DriedKelpBlock::source(),
                DripstoneBlock::source(),
                Dropper::source(),
                EmeraldBlock::source(),
                EmeraldOre::source(),
                EnchantingTable::source(),
                EndGateway::source(),
                EndPortal::source(),
                EndPortalFrame::source(),
                EndRod::source(),
                EndStone::source(),
                EndStoneBrickSlab::source(),
                EndStoneBrickStairs::source(),
                EndStoneBrickWall::source(),
                EndStoneBricks::source(),
                EnderChest::source(),
                ExposedChiseledCopper::source(),
                ExposedCopper::source(),
                ExposedCopperBars::source(),
                ExposedCopperBulb::source(),
                ExposedCopperChain::source(),
                ExposedCopperChest::source(),
                ExposedCopperDoor::source(),
                ExposedCopperGolemStatue::source(),
                ExposedCopperGrate::source(),
                ExposedCopperLantern::source(),
                ExposedCopperTrapdoor::source(),
                ExposedCutCopper::source(),
                ExposedCutCopperSlab::source(),
                ExposedCutCopperStairs::source(),
                ExposedLightningRod::source(),
                Farmland::source(),
                Fern::source(),
                Fire::source(),
                FireCoral::source(),
                FireCoralBlock::source(),
                FireCoralFan::source(),
                FireCoralWallFan::source(),
                FireflyBush::source(),
                FletchingTable::source(),
                FlowerPot::source(),
                FloweringAzalea::source(),
                FloweringAzaleaLeaves::source(),
                Frogspawn::source(),
                FrostedIce::source(),
                Furnace::source(),
                GildedBlackstone::source(),
                Glass::source(),
                GlassPane::source(),
                GlowLichen::source(),
                Glowstone::source(),
                GoldBlock::source(),
                GoldOre::source(),
                GoldenDandelion::source(),
                Granite::source(),
                GraniteSlab::source(),
                GraniteStairs::source(),
                GraniteWall::source(),
                GrassBlock::source(),
                Gravel::source(),
                GrayBanner::source(),
                GrayBed::source(),
                GrayCandle::source(),
                GrayCandleCake::source(),
                GrayCarpet::source(),
                GrayConcrete::source(),
                GrayConcretePowder::source(),
                GrayGlazedTerracotta::source(),
                GrayShulkerBox::source(),
                GrayStainedGlass::source(),
                GrayStainedGlassPane::source(),
                GrayTerracotta::source(),
                GrayWallBanner::source(),
                GrayWool::source(),
                GreenBanner::source(),
                GreenBed::source(),
                GreenCandle::source(),
                GreenCandleCake::source(),
                GreenCarpet::source(),
                GreenConcrete::source(),
                GreenConcretePowder::source(),
                GreenGlazedTerracotta::source(),
                GreenShulkerBox::source(),
                GreenStainedGlass::source(),
                GreenStainedGlassPane::source(),
                GreenTerracotta::source(),
                GreenWallBanner::source(),
                GreenWool::source(),
                Grindstone::source(),
                HangingRoots::source(),
                HayBlock::source(),
                HeavyCore::source(),
                HeavyWeightedPressurePlate::source(),
                HoneyBlock::source(),
                HoneycombBlock::source(),
                Hopper::source(),
                HornCoral::source(),
                HornCoralBlock::source(),
                HornCoralFan::source(),
                HornCoralWallFan::source(),
                Ice::source(),
                InfestedChiseledStoneBricks::source(),
                InfestedCobblestone::source(),
                InfestedCrackedStoneBricks::source(),
                InfestedDeepslate::source(),
                InfestedMossyStoneBricks::source(),
                InfestedStone::source(),
                InfestedStoneBricks::source(),
                IronBars::source(),
                IronBlock::source(),
                IronChain::source(),
                IronDoor::source(),
                IronOre::source(),
                IronTrapdoor::source(),
                JackOLantern::source(),
                Jigsaw::source(),
                Jukebox::source(),
                JungleButton::source(),
                JungleDoor::source(),
                JungleFence::source(),
                JungleFenceGate::source(),
                JungleHangingSign::source(),
                JungleLeaves::source(),
                JungleLog::source(),
                JunglePlanks::source(),
                JunglePressurePlate::source(),
                JungleSapling::source(),
                JungleShelf::source(),
                JungleSign::source(),
                JungleSlab::source(),
                JungleStairs::source(),
                JungleTrapdoor::source(),
                JungleWallHangingSign::source(),
                JungleWallSign::source(),
                JungleWood::source(),
                Kelp::source(),
                KelpPlant::source(),
                Ladder::source(),
                Lantern::source(),
                LapisBlock::source(),
                LapisOre::source(),
                LargeAmethystBud::source(),
                LargeFern::source(),
                Lava::source(),
                LavaCauldron::source(),
                LeafLitter::source(),
                Lectern::source(),
                Lever::source(),
                Light::source(),
                LightBlueBanner::source(),
                LightBlueBed::source(),
                LightBlueCandle::source(),
                LightBlueCandleCake::source(),
                LightBlueCarpet::source(),
                LightBlueConcrete::source(),
                LightBlueConcretePowder::source(),
                LightBlueGlazedTerracotta::source(),
                LightBlueShulkerBox::source(),
                LightBlueStainedGlass::source(),
                LightBlueStainedGlassPane::source(),
                LightBlueTerracotta::source(),
                LightBlueWallBanner::source(),
                LightBlueWool::source(),
                LightGrayBanner::source(),
                LightGrayBed::source(),
                LightGrayCandle::source(),
                LightGrayCandleCake::source(),
                LightGrayCarpet::source(),
                LightGrayConcrete::source(),
                LightGrayConcretePowder::source(),
                LightGrayGlazedTerracotta::source(),
                LightGrayShulkerBox::source(),
                LightGrayStainedGlass::source(),
                LightGrayStainedGlassPane::source(),
                LightGrayTerracotta::source(),
                LightGrayWallBanner::source(),
                LightGrayWool::source(),
                LightWeightedPressurePlate::source(),
                LightningRod::source(),
                Lilac::source(),
                LilyOfTheValley::source(),
                LilyPad::source(),
                LimeBanner::source(),
                LimeBed::source(),
                LimeCandle::source(),
                LimeCandleCake::source(),
                LimeCarpet::source(),
                LimeConcrete::source(),
                LimeConcretePowder::source(),
                LimeGlazedTerracotta::source(),
                LimeShulkerBox::source(),
                LimeStainedGlass::source(),
                LimeStainedGlassPane::source(),
                LimeTerracotta::source(),
                LimeWallBanner::source(),
                LimeWool::source(),
                Lodestone::source(),
                Loom::source(),
                MagentaBanner::source(),
                MagentaBed::source(),
                MagentaCandle::source(),
                MagentaCandleCake::source(),
                MagentaCarpet::source(),
                MagentaConcrete::source(),
                MagentaConcretePowder::source(),
                MagentaGlazedTerracotta::source(),
                MagentaShulkerBox::source(),
                MagentaStainedGlass::source(),
                MagentaStainedGlassPane::source(),
                MagentaTerracotta::source(),
                MagentaWallBanner::source(),
                MagentaWool::source(),
                MagmaBlock::source(),
                MangroveButton::source(),
                MangroveDoor::source(),
                MangroveFence::source(),
                MangroveFenceGate::source(),
                MangroveHangingSign::source(),
                MangroveLeaves::source(),
                MangroveLog::source(),
                MangrovePlanks::source(),
                MangrovePressurePlate::source(),
                MangrovePropagule::source(),
                MangroveRoots::source(),
                MangroveShelf::source(),
                MangroveSign::source(),
                MangroveSlab::source(),
                MangroveStairs::source(),
                MangroveTrapdoor::source(),
                MangroveWallHangingSign::source(),
                MangroveWallSign::source(),
                MangroveWood::source(),
                MediumAmethystBud::source(),
                Melon::source(),
                MelonStem::source(),
                MossBlock::source(),
                MossCarpet::source(),
                MossyCobblestone::source(),
                MossyCobblestoneSlab::source(),
                MossyCobblestoneStairs::source(),
                MossyCobblestoneWall::source(),
                MossyStoneBrickSlab::source(),
                MossyStoneBrickStairs::source(),
                MossyStoneBrickWall::source(),
                MossyStoneBricks::source(),
                MovingPiston::source(),
                Mud::source(),
                MudBrickSlab::source(),
                MudBrickStairs::source(),
                MudBrickWall::source(),
                MudBricks::source(),
                MuddyMangroveRoots::source(),
                MushroomStem::source(),
                Mycelium::source(),
                NetherBrickFence::source(),
                NetherBrickSlab::source(),
                NetherBrickStairs::source(),
                NetherBrickWall::source(),
                NetherBricks::source(),
                NetherGoldOre::source(),
                NetherPortal::source(),
                NetherQuartzOre::source(),
                NetherSprouts::source(),
                NetherWart::source(),
                NetherWartBlock::source(),
                NetheriteBlock::source(),
                Netherrack::source(),
                NoteBlock::source(),
                OakButton::source(),
                OakDoor::source(),
                OakFence::source(),
                OakFenceGate::source(),
                OakHangingSign::source(),
                OakLeaves::source(),
                OakLog::source(),
                OakPlanks::source(),
                OakPressurePlate::source(),
                OakSapling::source(),
                OakShelf::source(),
                OakSign::source(),
                OakSlab::source(),
                OakStairs::source(),
                OakTrapdoor::source(),
                OakWallHangingSign::source(),
                OakWallSign::source(),
                OakWood::source(),
                Observer::source(),
                Obsidian::source(),
                OchreFroglight::source(),
                OpenEyeblossom::source(),
                OrangeBanner::source(),
                OrangeBed::source(),
                OrangeCandle::source(),
                OrangeCandleCake::source(),
                OrangeCarpet::source(),
                OrangeConcrete::source(),
                OrangeConcretePowder::source(),
                OrangeGlazedTerracotta::source(),
                OrangeShulkerBox::source(),
                OrangeStainedGlass::source(),
                OrangeStainedGlassPane::source(),
                OrangeTerracotta::source(),
                OrangeTulip::source(),
                OrangeWallBanner::source(),
                OrangeWool::source(),
                OxeyeDaisy::source(),
                OxidizedChiseledCopper::source(),
                OxidizedCopper::source(),
                OxidizedCopperBars::source(),
                OxidizedCopperBulb::source(),
                OxidizedCopperChain::source(),
                OxidizedCopperChest::source(),
                OxidizedCopperDoor::source(),
                OxidizedCopperGolemStatue::source(),
                OxidizedCopperGrate::source(),
                OxidizedCopperLantern::source(),
                OxidizedCopperTrapdoor::source(),
                OxidizedCutCopper::source(),
                OxidizedCutCopperSlab::source(),
                OxidizedCutCopperStairs::source(),
                OxidizedLightningRod::source(),
                PackedIce::source(),
                PackedMud::source(),
                PaleHangingMoss::source(),
                PaleMossBlock::source(),
                PaleMossCarpet::source(),
                PaleOakButton::source(),
                PaleOakDoor::source(),
                PaleOakFence::source(),
                PaleOakFenceGate::source(),
                PaleOakHangingSign::source(),
                PaleOakLeaves::source(),
                PaleOakLog::source(),
                PaleOakPlanks::source(),
                PaleOakPressurePlate::source(),
                PaleOakSapling::source(),
                PaleOakShelf::source(),
                PaleOakSign::source(),
                PaleOakSlab::source(),
                PaleOakStairs::source(),
                PaleOakTrapdoor::source(),
                PaleOakWallHangingSign::source(),
                PaleOakWallSign::source(),
                PaleOakWood::source(),
                PearlescentFroglight::source(),
                Peony::source(),
                PetrifiedOakSlab::source(),
                PiglinHead::source(),
                PiglinWallHead::source(),
                PinkBanner::source(),
                PinkBed::source(),
                PinkCandle::source(),
                PinkCandleCake::source(),
                PinkCarpet::source(),
                PinkConcrete::source(),
                PinkConcretePowder::source(),
                PinkGlazedTerracotta::source(),
                PinkPetals::source(),
                PinkShulkerBox::source(),
                PinkStainedGlass::source(),
                PinkStainedGlassPane::source(),
                PinkTerracotta::source(),
                PinkTulip::source(),
                PinkWallBanner::source(),
                PinkWool::source(),
                Piston::source(),
                PistonHead::source(),
                PitcherCrop::source(),
                PitcherPlant::source(),
                PlayerHead::source(),
                PlayerWallHead::source(),
                Podzol::source(),
                PointedDripstone::source(),
                PolishedAndesite::source(),
                PolishedAndesiteSlab::source(),
                PolishedAndesiteStairs::source(),
                PolishedBasalt::source(),
                PolishedBlackstone::source(),
                PolishedBlackstoneBrickSlab::source(),
                PolishedBlackstoneBrickStairs::source(),
                PolishedBlackstoneBrickWall::source(),
                PolishedBlackstoneBricks::source(),
                PolishedBlackstoneButton::source(),
                PolishedBlackstonePressurePlate::source(),
                PolishedBlackstoneSlab::source(),
                PolishedBlackstoneStairs::source(),
                PolishedBlackstoneWall::source(),
                PolishedCinnabar::source(),
                PolishedCinnabarSlab::source(),
                PolishedCinnabarStairs::source(),
                PolishedCinnabarWall::source(),
                PolishedDeepslate::source(),
                PolishedDeepslateSlab::source(),
                PolishedDeepslateStairs::source(),
                PolishedDeepslateWall::source(),
                PolishedDiorite::source(),
                PolishedDioriteSlab::source(),
                PolishedDioriteStairs::source(),
                PolishedGranite::source(),
                PolishedGraniteSlab::source(),
                PolishedGraniteStairs::source(),
                PolishedSulfur::source(),
                PolishedSulfurSlab::source(),
                PolishedSulfurStairs::source(),
                PolishedSulfurWall::source(),
                PolishedTuff::source(),
                PolishedTuffSlab::source(),
                PolishedTuffStairs::source(),
                PolishedTuffWall::source(),
                Poppy::source(),
                Potatoes::source(),
                PotentSulfur::source(),
                PottedAcaciaSapling::source(),
                PottedAllium::source(),
                PottedAzaleaBush::source(),
                PottedAzureBluet::source(),
                PottedBamboo::source(),
                PottedBirchSapling::source(),
                PottedBlueOrchid::source(),
                PottedBrownMushroom::source(),
                PottedCactus::source(),
                PottedCherrySapling::source(),
                PottedClosedEyeblossom::source(),
                PottedCornflower::source(),
                PottedCrimsonFungus::source(),
                PottedCrimsonRoots::source(),
                PottedDandelion::source(),
                PottedDarkOakSapling::source(),
                PottedDeadBush::source(),
                PottedFern::source(),
                PottedFloweringAzaleaBush::source(),
                PottedGoldenDandelion::source(),
                PottedJungleSapling::source(),
                PottedLilyOfTheValley::source(),
                PottedMangrovePropagule::source(),
                PottedOakSapling::source(),
                PottedOpenEyeblossom::source(),
                PottedOrangeTulip::source(),
                PottedOxeyeDaisy::source(),
                PottedPaleOakSapling::source(),
                PottedPinkTulip::source(),
                PottedPoppy::source(),
                PottedRedMushroom::source(),
                PottedRedTulip::source(),
                PottedSpruceSapling::source(),
                PottedTorchflower::source(),
                PottedWarpedFungus::source(),
                PottedWarpedRoots::source(),
                PottedWhiteTulip::source(),
                PottedWitherRose::source(),
                PowderSnow::source(),
                PowderSnowCauldron::source(),
                PoweredRail::source(),
                Prismarine::source(),
                PrismarineBrickSlab::source(),
                PrismarineBrickStairs::source(),
                PrismarineBricks::source(),
                PrismarineSlab::source(),
                PrismarineStairs::source(),
                PrismarineWall::source(),
                Pumpkin::source(),
                PumpkinStem::source(),
                PurpleBanner::source(),
                PurpleBed::source(),
                PurpleCandle::source(),
                PurpleCandleCake::source(),
                PurpleCarpet::source(),
                PurpleConcrete::source(),
                PurpleConcretePowder::source(),
                PurpleGlazedTerracotta::source(),
                PurpleShulkerBox::source(),
                PurpleStainedGlass::source(),
                PurpleStainedGlassPane::source(),
                PurpleTerracotta::source(),
                PurpleWallBanner::source(),
                PurpleWool::source(),
                PurpurBlock::source(),
                PurpurPillar::source(),
                PurpurSlab::source(),
                PurpurStairs::source(),
                QuartzBlock::source(),
                QuartzBricks::source(),
                QuartzPillar::source(),
                QuartzSlab::source(),
                QuartzStairs::source(),
                Rail::source(),
                RawCopperBlock::source(),
                RawGoldBlock::source(),
                RawIronBlock::source(),
                RedBanner::source(),
                RedBed::source(),
                RedCandle::source(),
                RedCandleCake::source(),
                RedCarpet::source(),
                RedConcrete::source(),
                RedConcretePowder::source(),
                RedGlazedTerracotta::source(),
                RedMushroom::source(),
                RedMushroomBlock::source(),
                RedNetherBrickSlab::source(),
                RedNetherBrickStairs::source(),
                RedNetherBrickWall::source(),
                RedNetherBricks::source(),
                RedSand::source(),
                RedSandstone::source(),
                RedSandstoneSlab::source(),
                RedSandstoneStairs::source(),
                RedSandstoneWall::source(),
                RedShulkerBox::source(),
                RedStainedGlass::source(),
                RedStainedGlassPane::source(),
                RedTerracotta::source(),
                RedTulip::source(),
                RedWallBanner::source(),
                RedWool::source(),
                RedstoneBlock::source(),
                RedstoneLamp::source(),
                RedstoneOre::source(),
                RedstoneTorch::source(),
                RedstoneWallTorch::source(),
                RedstoneWire::source(),
                ReinforcedDeepslate::source(),
                Repeater::source(),
                RepeatingCommandBlock::source(),
                ResinBlock::source(),
                ResinBrickSlab::source(),
                ResinBrickStairs::source(),
                ResinBrickWall::source(),
                ResinBricks::source(),
                ResinClump::source(),
                RespawnAnchor::source(),
                RootedDirt::source(),
                RoseBush::source(),
                Sand::source(),
                Sandstone::source(),
                SandstoneSlab::source(),
                SandstoneStairs::source(),
                SandstoneWall::source(),
                Scaffolding::source(),
                Sculk::source(),
                SculkCatalyst::source(),
                SculkSensor::source(),
                SculkShrieker::source(),
                SculkVein::source(),
                SeaLantern::source(),
                SeaPickle::source(),
                Seagrass::source(),
                ShortDryGrass::source(),
                ShortGrass::source(),
                Shroomlight::source(),
                ShulkerBox::source(),
                SkeletonSkull::source(),
                SkeletonWallSkull::source(),
                SlimeBlock::source(),
                SmallAmethystBud::source(),
                SmallDripleaf::source(),
                SmithingTable::source(),
                Smoker::source(),
                SmoothBasalt::source(),
                SmoothQuartz::source(),
                SmoothQuartzSlab::source(),
                SmoothQuartzStairs::source(),
                SmoothRedSandstone::source(),
                SmoothRedSandstoneSlab::source(),
                SmoothRedSandstoneStairs::source(),
                SmoothSandstone::source(),
                SmoothSandstoneSlab::source(),
                SmoothSandstoneStairs::source(),
                SmoothStone::source(),
                SmoothStoneSlab::source(),
                SnifferEgg::source(),
                Snow::source(),
                SnowBlock::source(),
                SoulCampfire::source(),
                SoulFire::source(),
                SoulLantern::source(),
                SoulSand::source(),
                SoulSoil::source(),
                SoulTorch::source(),
                SoulWallTorch::source(),
                Spawner::source(),
                Sponge::source(),
                SporeBlossom::source(),
                SpruceButton::source(),
                SpruceDoor::source(),
                SpruceFence::source(),
                SpruceFenceGate::source(),
                SpruceHangingSign::source(),
                SpruceLeaves::source(),
                SpruceLog::source(),
                SprucePlanks::source(),
                SprucePressurePlate::source(),
                SpruceSapling::source(),
                SpruceShelf::source(),
                SpruceSign::source(),
                SpruceSlab::source(),
                SpruceStairs::source(),
                SpruceTrapdoor::source(),
                SpruceWallHangingSign::source(),
                SpruceWallSign::source(),
                SpruceWood::source(),
                StickyPiston::source(),
                Stone::source(),
                StoneBrickSlab::source(),
                StoneBrickStairs::source(),
                StoneBrickWall::source(),
                StoneBricks::source(),
                StoneButton::source(),
                StonePressurePlate::source(),
                StoneSlab::source(),
                StoneStairs::source(),
                Stonecutter::source(),
                StrippedAcaciaLog::source(),
                StrippedAcaciaWood::source(),
                StrippedBambooBlock::source(),
                StrippedBirchLog::source(),
                StrippedBirchWood::source(),
                StrippedCherryLog::source(),
                StrippedCherryWood::source(),
                StrippedCrimsonHyphae::source(),
                StrippedCrimsonStem::source(),
                StrippedDarkOakLog::source(),
                StrippedDarkOakWood::source(),
                StrippedJungleLog::source(),
                StrippedJungleWood::source(),
                StrippedMangroveLog::source(),
                StrippedMangroveWood::source(),
                StrippedOakLog::source(),
                StrippedOakWood::source(),
                StrippedPaleOakLog::source(),
                StrippedPaleOakWood::source(),
                StrippedSpruceLog::source(),
                StrippedSpruceWood::source(),
                StrippedWarpedHyphae::source(),
                StrippedWarpedStem::source(),
                StructureBlock::source(),
                StructureVoid::source(),
                SugarCane::source(),
                Sulfur::source(),
                SulfurBrickSlab::source(),
                SulfurBrickStairs::source(),
                SulfurBrickWall::source(),
                SulfurBricks::source(),
                SulfurSlab::source(),
                SulfurSpike::source(),
                SulfurStairs::source(),
                SulfurWall::source(),
                Sunflower::source(),
                SuspiciousGravel::source(),
                SuspiciousSand::source(),
                SweetBerryBush::source(),
                TallDryGrass::source(),
                TallGrass::source(),
                TallSeagrass::source(),
                Target::source(),
                Terracotta::source(),
                TestBlock::source(),
                TestInstanceBlock::source(),
                TintedGlass::source(),
                Tnt::source(),
                Torch::source(),
                Torchflower::source(),
                TorchflowerCrop::source(),
                TrappedChest::source(),
                TrialSpawner::source(),
                Tripwire::source(),
                TripwireHook::source(),
                TubeCoral::source(),
                TubeCoralBlock::source(),
                TubeCoralFan::source(),
                TubeCoralWallFan::source(),
                Tuff::source(),
                TuffBrickSlab::source(),
                TuffBrickStairs::source(),
                TuffBrickWall::source(),
                TuffBricks::source(),
                TuffSlab::source(),
                TuffStairs::source(),
                TuffWall::source(),
                TurtleEgg::source(),
                TwistingVines::source(),
                TwistingVinesPlant::source(),
                Vault::source(),
                VerdantFroglight::source(),
                Vine::source(),
                VoidAir::source(),
                WallTorch::source(),
                WarpedButton::source(),
                WarpedDoor::source(),
                WarpedFence::source(),
                WarpedFenceGate::source(),
                WarpedFungus::source(),
                WarpedHangingSign::source(),
                WarpedHyphae::source(),
                WarpedNylium::source(),
                WarpedPlanks::source(),
                WarpedPressurePlate::source(),
                WarpedRoots::source(),
                WarpedShelf::source(),
                WarpedSign::source(),
                WarpedSlab::source(),
                WarpedStairs::source(),
                WarpedStem::source(),
                WarpedTrapdoor::source(),
                WarpedWallHangingSign::source(),
                WarpedWallSign::source(),
                WarpedWartBlock::source(),
                Water::source(),
                WaterCauldron::source(),
                WaxedChiseledCopper::source(),
                WaxedCopperBars::source(),
                WaxedCopperBlock::source(),
                WaxedCopperBulb::source(),
                WaxedCopperChain::source(),
                WaxedCopperChest::source(),
                WaxedCopperDoor::source(),
                WaxedCopperGolemStatue::source(),
                WaxedCopperGrate::source(),
                WaxedCopperLantern::source(),
                WaxedCopperTrapdoor::source(),
                WaxedCutCopper::source(),
                WaxedCutCopperSlab::source(),
                WaxedCutCopperStairs::source(),
                WaxedExposedChiseledCopper::source(),
                WaxedExposedCopper::source(),
                WaxedExposedCopperBars::source(),
                WaxedExposedCopperBulb::source(),
                WaxedExposedCopperChain::source(),
                WaxedExposedCopperChest::source(),
                WaxedExposedCopperDoor::source(),
                WaxedExposedCopperGolemStatue::source(),
                WaxedExposedCopperGrate::source(),
                WaxedExposedCopperLantern::source(),
                WaxedExposedCopperTrapdoor::source(),
                WaxedExposedCutCopper::source(),
                WaxedExposedCutCopperSlab::source(),
                WaxedExposedCutCopperStairs::source(),
                WaxedExposedLightningRod::source(),
                WaxedLightningRod::source(),
                WaxedOxidizedChiseledCopper::source(),
                WaxedOxidizedCopper::source(),
                WaxedOxidizedCopperBars::source(),
                WaxedOxidizedCopperBulb::source(),
                WaxedOxidizedCopperChain::source(),
                WaxedOxidizedCopperChest::source(),
                WaxedOxidizedCopperDoor::source(),
                WaxedOxidizedCopperGolemStatue::source(),
                WaxedOxidizedCopperGrate::source(),
                WaxedOxidizedCopperLantern::source(),
                WaxedOxidizedCopperTrapdoor::source(),
                WaxedOxidizedCutCopper::source(),
                WaxedOxidizedCutCopperSlab::source(),
                WaxedOxidizedCutCopperStairs::source(),
                WaxedOxidizedLightningRod::source(),
                WaxedWeatheredChiseledCopper::source(),
                WaxedWeatheredCopper::source(),
                WaxedWeatheredCopperBars::source(),
                WaxedWeatheredCopperBulb::source(),
                WaxedWeatheredCopperChain::source(),
                WaxedWeatheredCopperChest::source(),
                WaxedWeatheredCopperDoor::source(),
                WaxedWeatheredCopperGolemStatue::source(),
                WaxedWeatheredCopperGrate::source(),
                WaxedWeatheredCopperLantern::source(),
                WaxedWeatheredCopperTrapdoor::source(),
                WaxedWeatheredCutCopper::source(),
                WaxedWeatheredCutCopperSlab::source(),
                WaxedWeatheredCutCopperStairs::source(),
                WaxedWeatheredLightningRod::source(),
                WeatheredChiseledCopper::source(),
                WeatheredCopper::source(),
                WeatheredCopperBars::source(),
                WeatheredCopperBulb::source(),
                WeatheredCopperChain::source(),
                WeatheredCopperChest::source(),
                WeatheredCopperDoor::source(),
                WeatheredCopperGolemStatue::source(),
                WeatheredCopperGrate::source(),
                WeatheredCopperLantern::source(),
                WeatheredCopperTrapdoor::source(),
                WeatheredCutCopper::source(),
                WeatheredCutCopperSlab::source(),
                WeatheredCutCopperStairs::source(),
                WeatheredLightningRod::source(),
                WeepingVines::source(),
                WeepingVinesPlant::source(),
                WetSponge::source(),
                Wheat::source(),
                WhiteBanner::source(),
                WhiteBed::source(),
                WhiteCandle::source(),
                WhiteCandleCake::source(),
                WhiteCarpet::source(),
                WhiteConcrete::source(),
                WhiteConcretePowder::source(),
                WhiteGlazedTerracotta::source(),
                WhiteShulkerBox::source(),
                WhiteStainedGlass::source(),
                WhiteStainedGlassPane::source(),
                WhiteTerracotta::source(),
                WhiteTulip::source(),
                WhiteWallBanner::source(),
                WhiteWool::source(),
                Wildflowers::source(),
                WitherRose::source(),
                WitherSkeletonSkull::source(),
                WitherSkeletonWallSkull::source(),
                YellowBanner::source(),
                YellowBed::source(),
                YellowCandle::source(),
                YellowCandleCake::source(),
                YellowCarpet::source(),
                YellowConcrete::source(),
                YellowConcretePowder::source(),
                YellowGlazedTerracotta::source(),
                YellowShulkerBox::source(),
                YellowStainedGlass::source(),
                YellowStainedGlassPane::source(),
                YellowTerracotta::source(),
                YellowWallBanner::source(),
                YellowWool::source(),
                ZombieHead::source(),
                ZombieWallHead::source(),
            ],
        }
    }

    /// The underlying palette.
    pub fn palette(&self) -> &BlockPalette<1196> {
        &self.palette
    }

    /// `acacia_button` — Acacia Button.
    pub fn acacia_button(&self) -> &Block {
        &self.palette[0]
    }

    /// `acacia_door` — Acacia Door.
    pub fn acacia_door(&self) -> &Block {
        &self.palette[1]
    }

    /// `acacia_fence` — Acacia Fence.
    pub fn acacia_fence(&self) -> &Block {
        &self.palette[2]
    }

    /// `acacia_fence_gate` — Acacia Fence Gate.
    pub fn acacia_fence_gate(&self) -> &Block {
        &self.palette[3]
    }

    /// `acacia_hanging_sign` — Acacia Hanging Sign.
    pub fn acacia_hanging_sign(&self) -> &Block {
        &self.palette[4]
    }

    /// `acacia_leaves` — Acacia Leaves.
    pub fn acacia_leaves(&self) -> &Block {
        &self.palette[5]
    }

    /// `acacia_log` — Acacia Log.
    pub fn acacia_log(&self) -> &Block {
        &self.palette[6]
    }

    /// `acacia_planks` — Acacia Planks.
    pub fn acacia_planks(&self) -> &Block {
        &self.palette[7]
    }

    /// `acacia_pressure_plate` — Acacia Pressure Plate.
    pub fn acacia_pressure_plate(&self) -> &Block {
        &self.palette[8]
    }

    /// `acacia_sapling` — Acacia Sapling.
    pub fn acacia_sapling(&self) -> &Block {
        &self.palette[9]
    }

    /// `acacia_shelf` — Acacia Shelf.
    pub fn acacia_shelf(&self) -> &Block {
        &self.palette[10]
    }

    /// `acacia_sign` — Acacia Sign.
    pub fn acacia_sign(&self) -> &Block {
        &self.palette[11]
    }

    /// `acacia_slab` — Acacia Slab.
    pub fn acacia_slab(&self) -> &Block {
        &self.palette[12]
    }

    /// `acacia_stairs` — Acacia Stairs.
    pub fn acacia_stairs(&self) -> &Block {
        &self.palette[13]
    }

    /// `acacia_trapdoor` — Acacia Trapdoor.
    pub fn acacia_trapdoor(&self) -> &Block {
        &self.palette[14]
    }

    /// `acacia_wall_hanging_sign` — Acacia Hanging Sign.
    pub fn acacia_wall_hanging_sign(&self) -> &Block {
        &self.palette[15]
    }

    /// `acacia_wall_sign` — Acacia Sign.
    pub fn acacia_wall_sign(&self) -> &Block {
        &self.palette[16]
    }

    /// `acacia_wood` — Acacia Wood.
    pub fn acacia_wood(&self) -> &Block {
        &self.palette[17]
    }

    /// `activator_rail` — Activator Rail.
    pub fn activator_rail(&self) -> &Block {
        &self.palette[18]
    }

    /// `air` — Air.
    pub fn air(&self) -> &Block {
        &self.palette[19]
    }

    /// `allium` — Allium.
    pub fn allium(&self) -> &Block {
        &self.palette[20]
    }

    /// `amethyst_block` — Block of Amethyst.
    pub fn amethyst_block(&self) -> &Block {
        &self.palette[21]
    }

    /// `amethyst_cluster` — Amethyst Cluster.
    pub fn amethyst_cluster(&self) -> &Block {
        &self.palette[22]
    }

    /// `ancient_debris` — Ancient Debris.
    pub fn ancient_debris(&self) -> &Block {
        &self.palette[23]
    }

    /// `andesite` — Andesite.
    pub fn andesite(&self) -> &Block {
        &self.palette[24]
    }

    /// `andesite_slab` — Andesite Slab.
    pub fn andesite_slab(&self) -> &Block {
        &self.palette[25]
    }

    /// `andesite_stairs` — Andesite Stairs.
    pub fn andesite_stairs(&self) -> &Block {
        &self.palette[26]
    }

    /// `andesite_wall` — Andesite Wall.
    pub fn andesite_wall(&self) -> &Block {
        &self.palette[27]
    }

    /// `anvil` — Anvil.
    pub fn anvil(&self) -> &Block {
        &self.palette[28]
    }

    /// `attached_melon_stem` — Attached Melon Stem.
    pub fn attached_melon_stem(&self) -> &Block {
        &self.palette[29]
    }

    /// `attached_pumpkin_stem` — Attached Pumpkin Stem.
    pub fn attached_pumpkin_stem(&self) -> &Block {
        &self.palette[30]
    }

    /// `azalea` — Azalea.
    pub fn azalea(&self) -> &Block {
        &self.palette[31]
    }

    /// `azalea_leaves` — Azalea Leaves.
    pub fn azalea_leaves(&self) -> &Block {
        &self.palette[32]
    }

    /// `azure_bluet` — Azure Bluet.
    pub fn azure_bluet(&self) -> &Block {
        &self.palette[33]
    }

    /// `bamboo` — Bamboo.
    pub fn bamboo(&self) -> &Block {
        &self.palette[34]
    }

    /// `bamboo_block` — Block of Bamboo.
    pub fn bamboo_block(&self) -> &Block {
        &self.palette[35]
    }

    /// `bamboo_button` — Bamboo Button.
    pub fn bamboo_button(&self) -> &Block {
        &self.palette[36]
    }

    /// `bamboo_door` — Bamboo Door.
    pub fn bamboo_door(&self) -> &Block {
        &self.palette[37]
    }

    /// `bamboo_fence` — Bamboo Fence.
    pub fn bamboo_fence(&self) -> &Block {
        &self.palette[38]
    }

    /// `bamboo_fence_gate` — Bamboo Fence Gate.
    pub fn bamboo_fence_gate(&self) -> &Block {
        &self.palette[39]
    }

    /// `bamboo_hanging_sign` — Bamboo Hanging Sign.
    pub fn bamboo_hanging_sign(&self) -> &Block {
        &self.palette[40]
    }

    /// `bamboo_mosaic` — Bamboo Mosaic.
    pub fn bamboo_mosaic(&self) -> &Block {
        &self.palette[41]
    }

    /// `bamboo_mosaic_slab` — Bamboo Mosaic Slab.
    pub fn bamboo_mosaic_slab(&self) -> &Block {
        &self.palette[42]
    }

    /// `bamboo_mosaic_stairs` — Bamboo Mosaic Stairs.
    pub fn bamboo_mosaic_stairs(&self) -> &Block {
        &self.palette[43]
    }

    /// `bamboo_planks` — Bamboo Planks.
    pub fn bamboo_planks(&self) -> &Block {
        &self.palette[44]
    }

    /// `bamboo_pressure_plate` — Bamboo Pressure Plate.
    pub fn bamboo_pressure_plate(&self) -> &Block {
        &self.palette[45]
    }

    /// `bamboo_sapling` — Bamboo Shoot.
    pub fn bamboo_sapling(&self) -> &Block {
        &self.palette[46]
    }

    /// `bamboo_shelf` — Bamboo Shelf.
    pub fn bamboo_shelf(&self) -> &Block {
        &self.palette[47]
    }

    /// `bamboo_sign` — Bamboo Sign.
    pub fn bamboo_sign(&self) -> &Block {
        &self.palette[48]
    }

    /// `bamboo_slab` — Bamboo Slab.
    pub fn bamboo_slab(&self) -> &Block {
        &self.palette[49]
    }

    /// `bamboo_stairs` — Bamboo Stairs.
    pub fn bamboo_stairs(&self) -> &Block {
        &self.palette[50]
    }

    /// `bamboo_trapdoor` — Bamboo Trapdoor.
    pub fn bamboo_trapdoor(&self) -> &Block {
        &self.palette[51]
    }

    /// `bamboo_wall_hanging_sign` — Bamboo Hanging Sign.
    pub fn bamboo_wall_hanging_sign(&self) -> &Block {
        &self.palette[52]
    }

    /// `bamboo_wall_sign` — Bamboo Sign.
    pub fn bamboo_wall_sign(&self) -> &Block {
        &self.palette[53]
    }

    /// `barrel` — Barrel.
    pub fn barrel(&self) -> &Block {
        &self.palette[54]
    }

    /// `barrier` — Barrier.
    pub fn barrier(&self) -> &Block {
        &self.palette[55]
    }

    /// `basalt` — Basalt.
    pub fn basalt(&self) -> &Block {
        &self.palette[56]
    }

    /// `beacon` — Beacon.
    pub fn beacon(&self) -> &Block {
        &self.palette[57]
    }

    /// `bedrock` — Bedrock.
    pub fn bedrock(&self) -> &Block {
        &self.palette[58]
    }

    /// `bee_nest` — Bee Nest.
    pub fn bee_nest(&self) -> &Block {
        &self.palette[59]
    }

    /// `beehive` — Beehive.
    pub fn beehive(&self) -> &Block {
        &self.palette[60]
    }

    /// `beetroots` — Beetroots.
    pub fn beetroots(&self) -> &Block {
        &self.palette[61]
    }

    /// `bell` — Bell.
    pub fn bell(&self) -> &Block {
        &self.palette[62]
    }

    /// `big_dripleaf` — Big Dripleaf.
    pub fn big_dripleaf(&self) -> &Block {
        &self.palette[63]
    }

    /// `big_dripleaf_stem` — Big Dripleaf Stem.
    pub fn big_dripleaf_stem(&self) -> &Block {
        &self.palette[64]
    }

    /// `birch_button` — Birch Button.
    pub fn birch_button(&self) -> &Block {
        &self.palette[65]
    }

    /// `birch_door` — Birch Door.
    pub fn birch_door(&self) -> &Block {
        &self.palette[66]
    }

    /// `birch_fence` — Birch Fence.
    pub fn birch_fence(&self) -> &Block {
        &self.palette[67]
    }

    /// `birch_fence_gate` — Birch Fence Gate.
    pub fn birch_fence_gate(&self) -> &Block {
        &self.palette[68]
    }

    /// `birch_hanging_sign` — Birch Hanging Sign.
    pub fn birch_hanging_sign(&self) -> &Block {
        &self.palette[69]
    }

    /// `birch_leaves` — Birch Leaves.
    pub fn birch_leaves(&self) -> &Block {
        &self.palette[70]
    }

    /// `birch_log` — Birch Log.
    pub fn birch_log(&self) -> &Block {
        &self.palette[71]
    }

    /// `birch_planks` — Birch Planks.
    pub fn birch_planks(&self) -> &Block {
        &self.palette[72]
    }

    /// `birch_pressure_plate` — Birch Pressure Plate.
    pub fn birch_pressure_plate(&self) -> &Block {
        &self.palette[73]
    }

    /// `birch_sapling` — Birch Sapling.
    pub fn birch_sapling(&self) -> &Block {
        &self.palette[74]
    }

    /// `birch_shelf` — Birch Shelf.
    pub fn birch_shelf(&self) -> &Block {
        &self.palette[75]
    }

    /// `birch_sign` — Birch Sign.
    pub fn birch_sign(&self) -> &Block {
        &self.palette[76]
    }

    /// `birch_slab` — Birch Slab.
    pub fn birch_slab(&self) -> &Block {
        &self.palette[77]
    }

    /// `birch_stairs` — Birch Stairs.
    pub fn birch_stairs(&self) -> &Block {
        &self.palette[78]
    }

    /// `birch_trapdoor` — Birch Trapdoor.
    pub fn birch_trapdoor(&self) -> &Block {
        &self.palette[79]
    }

    /// `birch_wall_hanging_sign` — Birch Hanging Sign.
    pub fn birch_wall_hanging_sign(&self) -> &Block {
        &self.palette[80]
    }

    /// `birch_wall_sign` — Birch Sign.
    pub fn birch_wall_sign(&self) -> &Block {
        &self.palette[81]
    }

    /// `birch_wood` — Birch Wood.
    pub fn birch_wood(&self) -> &Block {
        &self.palette[82]
    }

    /// `black_banner` — Black Banner.
    pub fn black_banner(&self) -> &Block {
        &self.palette[83]
    }

    /// `black_bed` — Black Bed.
    pub fn black_bed(&self) -> &Block {
        &self.palette[84]
    }

    /// `black_candle` — Black Candle.
    pub fn black_candle(&self) -> &Block {
        &self.palette[85]
    }

    /// `black_candle_cake` — Cake with Black Candle.
    pub fn black_candle_cake(&self) -> &Block {
        &self.palette[86]
    }

    /// `black_carpet` — Black Carpet.
    pub fn black_carpet(&self) -> &Block {
        &self.palette[87]
    }

    /// `black_concrete` — Black Concrete.
    pub fn black_concrete(&self) -> &Block {
        &self.palette[88]
    }

    /// `black_concrete_powder` — Black Concrete Powder.
    pub fn black_concrete_powder(&self) -> &Block {
        &self.palette[89]
    }

    /// `black_glazed_terracotta` — Black Glazed Terracotta.
    pub fn black_glazed_terracotta(&self) -> &Block {
        &self.palette[90]
    }

    /// `black_shulker_box` — Black Shulker Box.
    pub fn black_shulker_box(&self) -> &Block {
        &self.palette[91]
    }

    /// `black_stained_glass` — Black Stained Glass.
    pub fn black_stained_glass(&self) -> &Block {
        &self.palette[92]
    }

    /// `black_stained_glass_pane` — Black Stained Glass Pane.
    pub fn black_stained_glass_pane(&self) -> &Block {
        &self.palette[93]
    }

    /// `black_terracotta` — Black Terracotta.
    pub fn black_terracotta(&self) -> &Block {
        &self.palette[94]
    }

    /// `black_wall_banner` — Black Banner.
    pub fn black_wall_banner(&self) -> &Block {
        &self.palette[95]
    }

    /// `black_wool` — Black Wool.
    pub fn black_wool(&self) -> &Block {
        &self.palette[96]
    }

    /// `blackstone` — Blackstone.
    pub fn blackstone(&self) -> &Block {
        &self.palette[97]
    }

    /// `blackstone_slab` — Blackstone Slab.
    pub fn blackstone_slab(&self) -> &Block {
        &self.palette[98]
    }

    /// `blackstone_stairs` — Blackstone Stairs.
    pub fn blackstone_stairs(&self) -> &Block {
        &self.palette[99]
    }

    /// `blackstone_wall` — Blackstone Wall.
    pub fn blackstone_wall(&self) -> &Block {
        &self.palette[100]
    }

    /// `blast_furnace` — Blast Furnace.
    pub fn blast_furnace(&self) -> &Block {
        &self.palette[101]
    }

    /// `blue_banner` — Blue Banner.
    pub fn blue_banner(&self) -> &Block {
        &self.palette[102]
    }

    /// `blue_bed` — Blue Bed.
    pub fn blue_bed(&self) -> &Block {
        &self.palette[103]
    }

    /// `blue_candle` — Blue Candle.
    pub fn blue_candle(&self) -> &Block {
        &self.palette[104]
    }

    /// `blue_candle_cake` — Cake with Blue Candle.
    pub fn blue_candle_cake(&self) -> &Block {
        &self.palette[105]
    }

    /// `blue_carpet` — Blue Carpet.
    pub fn blue_carpet(&self) -> &Block {
        &self.palette[106]
    }

    /// `blue_concrete` — Blue Concrete.
    pub fn blue_concrete(&self) -> &Block {
        &self.palette[107]
    }

    /// `blue_concrete_powder` — Blue Concrete Powder.
    pub fn blue_concrete_powder(&self) -> &Block {
        &self.palette[108]
    }

    /// `blue_glazed_terracotta` — Blue Glazed Terracotta.
    pub fn blue_glazed_terracotta(&self) -> &Block {
        &self.palette[109]
    }

    /// `blue_ice` — Blue Ice.
    pub fn blue_ice(&self) -> &Block {
        &self.palette[110]
    }

    /// `blue_orchid` — Blue Orchid.
    pub fn blue_orchid(&self) -> &Block {
        &self.palette[111]
    }

    /// `blue_shulker_box` — Blue Shulker Box.
    pub fn blue_shulker_box(&self) -> &Block {
        &self.palette[112]
    }

    /// `blue_stained_glass` — Blue Stained Glass.
    pub fn blue_stained_glass(&self) -> &Block {
        &self.palette[113]
    }

    /// `blue_stained_glass_pane` — Blue Stained Glass Pane.
    pub fn blue_stained_glass_pane(&self) -> &Block {
        &self.palette[114]
    }

    /// `blue_terracotta` — Blue Terracotta.
    pub fn blue_terracotta(&self) -> &Block {
        &self.palette[115]
    }

    /// `blue_wall_banner` — Blue Banner.
    pub fn blue_wall_banner(&self) -> &Block {
        &self.palette[116]
    }

    /// `blue_wool` — Blue Wool.
    pub fn blue_wool(&self) -> &Block {
        &self.palette[117]
    }

    /// `bone_block` — Bone Block.
    pub fn bone_block(&self) -> &Block {
        &self.palette[118]
    }

    /// `bookshelf` — Bookshelf.
    pub fn bookshelf(&self) -> &Block {
        &self.palette[119]
    }

    /// `brain_coral` — Brain Coral.
    pub fn brain_coral(&self) -> &Block {
        &self.palette[120]
    }

    /// `brain_coral_block` — Brain Coral Block.
    pub fn brain_coral_block(&self) -> &Block {
        &self.palette[121]
    }

    /// `brain_coral_fan` — Brain Coral Fan.
    pub fn brain_coral_fan(&self) -> &Block {
        &self.palette[122]
    }

    /// `brain_coral_wall_fan` — Brain Coral Wall Fan.
    pub fn brain_coral_wall_fan(&self) -> &Block {
        &self.palette[123]
    }

    /// `brewing_stand` — Brewing Stand.
    pub fn brewing_stand(&self) -> &Block {
        &self.palette[124]
    }

    /// `brick_slab` — Brick Slab.
    pub fn brick_slab(&self) -> &Block {
        &self.palette[125]
    }

    /// `brick_stairs` — Brick Stairs.
    pub fn brick_stairs(&self) -> &Block {
        &self.palette[126]
    }

    /// `brick_wall` — Brick Wall.
    pub fn brick_wall(&self) -> &Block {
        &self.palette[127]
    }

    /// `bricks` — Bricks.
    pub fn bricks(&self) -> &Block {
        &self.palette[128]
    }

    /// `brown_banner` — Brown Banner.
    pub fn brown_banner(&self) -> &Block {
        &self.palette[129]
    }

    /// `brown_bed` — Brown Bed.
    pub fn brown_bed(&self) -> &Block {
        &self.palette[130]
    }

    /// `brown_candle` — Brown Candle.
    pub fn brown_candle(&self) -> &Block {
        &self.palette[131]
    }

    /// `brown_candle_cake` — Cake with Brown Candle.
    pub fn brown_candle_cake(&self) -> &Block {
        &self.palette[132]
    }

    /// `brown_carpet` — Brown Carpet.
    pub fn brown_carpet(&self) -> &Block {
        &self.palette[133]
    }

    /// `brown_concrete` — Brown Concrete.
    pub fn brown_concrete(&self) -> &Block {
        &self.palette[134]
    }

    /// `brown_concrete_powder` — Brown Concrete Powder.
    pub fn brown_concrete_powder(&self) -> &Block {
        &self.palette[135]
    }

    /// `brown_glazed_terracotta` — Brown Glazed Terracotta.
    pub fn brown_glazed_terracotta(&self) -> &Block {
        &self.palette[136]
    }

    /// `brown_mushroom` — Brown Mushroom.
    pub fn brown_mushroom(&self) -> &Block {
        &self.palette[137]
    }

    /// `brown_mushroom_block` — Brown Mushroom Block.
    pub fn brown_mushroom_block(&self) -> &Block {
        &self.palette[138]
    }

    /// `brown_shulker_box` — Brown Shulker Box.
    pub fn brown_shulker_box(&self) -> &Block {
        &self.palette[139]
    }

    /// `brown_stained_glass` — Brown Stained Glass.
    pub fn brown_stained_glass(&self) -> &Block {
        &self.palette[140]
    }

    /// `brown_stained_glass_pane` — Brown Stained Glass Pane.
    pub fn brown_stained_glass_pane(&self) -> &Block {
        &self.palette[141]
    }

    /// `brown_terracotta` — Brown Terracotta.
    pub fn brown_terracotta(&self) -> &Block {
        &self.palette[142]
    }

    /// `brown_wall_banner` — Brown Banner.
    pub fn brown_wall_banner(&self) -> &Block {
        &self.palette[143]
    }

    /// `brown_wool` — Brown Wool.
    pub fn brown_wool(&self) -> &Block {
        &self.palette[144]
    }

    /// `bubble_column` — Bubble Column.
    pub fn bubble_column(&self) -> &Block {
        &self.palette[145]
    }

    /// `bubble_coral` — Bubble Coral.
    pub fn bubble_coral(&self) -> &Block {
        &self.palette[146]
    }

    /// `bubble_coral_block` — Bubble Coral Block.
    pub fn bubble_coral_block(&self) -> &Block {
        &self.palette[147]
    }

    /// `bubble_coral_fan` — Bubble Coral Fan.
    pub fn bubble_coral_fan(&self) -> &Block {
        &self.palette[148]
    }

    /// `bubble_coral_wall_fan` — Bubble Coral Wall Fan.
    pub fn bubble_coral_wall_fan(&self) -> &Block {
        &self.palette[149]
    }

    /// `budding_amethyst` — Budding Amethyst.
    pub fn budding_amethyst(&self) -> &Block {
        &self.palette[150]
    }

    /// `bush` — Bush.
    pub fn bush(&self) -> &Block {
        &self.palette[151]
    }

    /// `cactus` — Cactus.
    pub fn cactus(&self) -> &Block {
        &self.palette[152]
    }

    /// `cactus_flower` — Cactus Flower.
    pub fn cactus_flower(&self) -> &Block {
        &self.palette[153]
    }

    /// `cake` — Cake.
    pub fn cake(&self) -> &Block {
        &self.palette[154]
    }

    /// `calcite` — Calcite.
    pub fn calcite(&self) -> &Block {
        &self.palette[155]
    }

    /// `calibrated_sculk_sensor` — Calibrated Sculk Sensor.
    pub fn calibrated_sculk_sensor(&self) -> &Block {
        &self.palette[156]
    }

    /// `campfire` — Campfire.
    pub fn campfire(&self) -> &Block {
        &self.palette[157]
    }

    /// `candle` — Candle.
    pub fn candle(&self) -> &Block {
        &self.palette[158]
    }

    /// `candle_cake` — Cake with Candle.
    pub fn candle_cake(&self) -> &Block {
        &self.palette[159]
    }

    /// `carrots` — Carrots.
    pub fn carrots(&self) -> &Block {
        &self.palette[160]
    }

    /// `cartography_table` — Cartography Table.
    pub fn cartography_table(&self) -> &Block {
        &self.palette[161]
    }

    /// `carved_pumpkin` — Carved Pumpkin.
    pub fn carved_pumpkin(&self) -> &Block {
        &self.palette[162]
    }

    /// `cauldron` — Cauldron.
    pub fn cauldron(&self) -> &Block {
        &self.palette[163]
    }

    /// `cave_air` — Cave Air.
    pub fn cave_air(&self) -> &Block {
        &self.palette[164]
    }

    /// `cave_vines` — Cave Vines.
    pub fn cave_vines(&self) -> &Block {
        &self.palette[165]
    }

    /// `cave_vines_plant` — Cave Vines Plant.
    pub fn cave_vines_plant(&self) -> &Block {
        &self.palette[166]
    }

    /// `chain_command_block` — Chain Command Block.
    pub fn chain_command_block(&self) -> &Block {
        &self.palette[167]
    }

    /// `cherry_button` — Cherry Button.
    pub fn cherry_button(&self) -> &Block {
        &self.palette[168]
    }

    /// `cherry_door` — Cherry Door.
    pub fn cherry_door(&self) -> &Block {
        &self.palette[169]
    }

    /// `cherry_fence` — Cherry Fence.
    pub fn cherry_fence(&self) -> &Block {
        &self.palette[170]
    }

    /// `cherry_fence_gate` — Cherry Fence Gate.
    pub fn cherry_fence_gate(&self) -> &Block {
        &self.palette[171]
    }

    /// `cherry_hanging_sign` — Cherry Hanging Sign.
    pub fn cherry_hanging_sign(&self) -> &Block {
        &self.palette[172]
    }

    /// `cherry_leaves` — Cherry Leaves.
    pub fn cherry_leaves(&self) -> &Block {
        &self.palette[173]
    }

    /// `cherry_log` — Cherry Log.
    pub fn cherry_log(&self) -> &Block {
        &self.palette[174]
    }

    /// `cherry_planks` — Cherry Planks.
    pub fn cherry_planks(&self) -> &Block {
        &self.palette[175]
    }

    /// `cherry_pressure_plate` — Cherry Pressure Plate.
    pub fn cherry_pressure_plate(&self) -> &Block {
        &self.palette[176]
    }

    /// `cherry_sapling` — Cherry Sapling.
    pub fn cherry_sapling(&self) -> &Block {
        &self.palette[177]
    }

    /// `cherry_shelf` — Cherry Shelf.
    pub fn cherry_shelf(&self) -> &Block {
        &self.palette[178]
    }

    /// `cherry_sign` — Cherry Sign.
    pub fn cherry_sign(&self) -> &Block {
        &self.palette[179]
    }

    /// `cherry_slab` — Cherry Slab.
    pub fn cherry_slab(&self) -> &Block {
        &self.palette[180]
    }

    /// `cherry_stairs` — Cherry Stairs.
    pub fn cherry_stairs(&self) -> &Block {
        &self.palette[181]
    }

    /// `cherry_trapdoor` — Cherry Trapdoor.
    pub fn cherry_trapdoor(&self) -> &Block {
        &self.palette[182]
    }

    /// `cherry_wall_hanging_sign` — Cherry Hanging Sign.
    pub fn cherry_wall_hanging_sign(&self) -> &Block {
        &self.palette[183]
    }

    /// `cherry_wall_sign` — Cherry Sign.
    pub fn cherry_wall_sign(&self) -> &Block {
        &self.palette[184]
    }

    /// `cherry_wood` — Cherry Wood.
    pub fn cherry_wood(&self) -> &Block {
        &self.palette[185]
    }

    /// `chest` — Chest.
    pub fn chest(&self) -> &Block {
        &self.palette[186]
    }

    /// `chipped_anvil` — Chipped Anvil.
    pub fn chipped_anvil(&self) -> &Block {
        &self.palette[187]
    }

    /// `chiseled_bookshelf` — Chiseled Bookshelf.
    pub fn chiseled_bookshelf(&self) -> &Block {
        &self.palette[188]
    }

    /// `chiseled_cinnabar` — Chiseled Cinnabar.
    pub fn chiseled_cinnabar(&self) -> &Block {
        &self.palette[189]
    }

    /// `chiseled_copper` — Chiseled Copper.
    pub fn chiseled_copper(&self) -> &Block {
        &self.palette[190]
    }

    /// `chiseled_deepslate` — Chiseled Deepslate.
    pub fn chiseled_deepslate(&self) -> &Block {
        &self.palette[191]
    }

    /// `chiseled_nether_bricks` — Chiseled Nether Bricks.
    pub fn chiseled_nether_bricks(&self) -> &Block {
        &self.palette[192]
    }

    /// `chiseled_polished_blackstone` — Chiseled Polished Blackstone.
    pub fn chiseled_polished_blackstone(&self) -> &Block {
        &self.palette[193]
    }

    /// `chiseled_quartz_block` — Chiseled Quartz Block.
    pub fn chiseled_quartz_block(&self) -> &Block {
        &self.palette[194]
    }

    /// `chiseled_red_sandstone` — Chiseled Red Sandstone.
    pub fn chiseled_red_sandstone(&self) -> &Block {
        &self.palette[195]
    }

    /// `chiseled_resin_bricks` — Chiseled Resin Bricks.
    pub fn chiseled_resin_bricks(&self) -> &Block {
        &self.palette[196]
    }

    /// `chiseled_sandstone` — Chiseled Sandstone.
    pub fn chiseled_sandstone(&self) -> &Block {
        &self.palette[197]
    }

    /// `chiseled_stone_bricks` — Chiseled Stone Bricks.
    pub fn chiseled_stone_bricks(&self) -> &Block {
        &self.palette[198]
    }

    /// `chiseled_sulfur` — Chiseled Sulfur.
    pub fn chiseled_sulfur(&self) -> &Block {
        &self.palette[199]
    }

    /// `chiseled_tuff` — Chiseled Tuff.
    pub fn chiseled_tuff(&self) -> &Block {
        &self.palette[200]
    }

    /// `chiseled_tuff_bricks` — Chiseled Tuff Bricks.
    pub fn chiseled_tuff_bricks(&self) -> &Block {
        &self.palette[201]
    }

    /// `chorus_flower` — Chorus Flower.
    pub fn chorus_flower(&self) -> &Block {
        &self.palette[202]
    }

    /// `chorus_plant` — Chorus Plant.
    pub fn chorus_plant(&self) -> &Block {
        &self.palette[203]
    }

    /// `cinnabar` — Cinnabar.
    pub fn cinnabar(&self) -> &Block {
        &self.palette[204]
    }

    /// `cinnabar_brick_slab` — Cinnabar Brick Slab.
    pub fn cinnabar_brick_slab(&self) -> &Block {
        &self.palette[205]
    }

    /// `cinnabar_brick_stairs` — Cinnabar Brick Stairs.
    pub fn cinnabar_brick_stairs(&self) -> &Block {
        &self.palette[206]
    }

    /// `cinnabar_brick_wall` — Cinnabar Brick Wall.
    pub fn cinnabar_brick_wall(&self) -> &Block {
        &self.palette[207]
    }

    /// `cinnabar_bricks` — Cinnabar Bricks.
    pub fn cinnabar_bricks(&self) -> &Block {
        &self.palette[208]
    }

    /// `cinnabar_slab` — Cinnabar Slab.
    pub fn cinnabar_slab(&self) -> &Block {
        &self.palette[209]
    }

    /// `cinnabar_stairs` — Cinnabar Stairs.
    pub fn cinnabar_stairs(&self) -> &Block {
        &self.palette[210]
    }

    /// `cinnabar_wall` — Cinnabar Wall.
    pub fn cinnabar_wall(&self) -> &Block {
        &self.palette[211]
    }

    /// `clay` — Clay.
    pub fn clay(&self) -> &Block {
        &self.palette[212]
    }

    /// `closed_eyeblossom` — Closed Eyeblossom.
    pub fn closed_eyeblossom(&self) -> &Block {
        &self.palette[213]
    }

    /// `coal_block` — Block of Coal.
    pub fn coal_block(&self) -> &Block {
        &self.palette[214]
    }

    /// `coal_ore` — Coal Ore.
    pub fn coal_ore(&self) -> &Block {
        &self.palette[215]
    }

    /// `coarse_dirt` — Coarse Dirt.
    pub fn coarse_dirt(&self) -> &Block {
        &self.palette[216]
    }

    /// `cobbled_deepslate` — Cobbled Deepslate.
    pub fn cobbled_deepslate(&self) -> &Block {
        &self.palette[217]
    }

    /// `cobbled_deepslate_slab` — Cobbled Deepslate Slab.
    pub fn cobbled_deepslate_slab(&self) -> &Block {
        &self.palette[218]
    }

    /// `cobbled_deepslate_stairs` — Cobbled Deepslate Stairs.
    pub fn cobbled_deepslate_stairs(&self) -> &Block {
        &self.palette[219]
    }

    /// `cobbled_deepslate_wall` — Cobbled Deepslate Wall.
    pub fn cobbled_deepslate_wall(&self) -> &Block {
        &self.palette[220]
    }

    /// `cobblestone` — Cobblestone.
    pub fn cobblestone(&self) -> &Block {
        &self.palette[221]
    }

    /// `cobblestone_slab` — Cobblestone Slab.
    pub fn cobblestone_slab(&self) -> &Block {
        &self.palette[222]
    }

    /// `cobblestone_stairs` — Cobblestone Stairs.
    pub fn cobblestone_stairs(&self) -> &Block {
        &self.palette[223]
    }

    /// `cobblestone_wall` — Cobblestone Wall.
    pub fn cobblestone_wall(&self) -> &Block {
        &self.palette[224]
    }

    /// `cobweb` — Cobweb.
    pub fn cobweb(&self) -> &Block {
        &self.palette[225]
    }

    /// `cocoa` — Cocoa.
    pub fn cocoa(&self) -> &Block {
        &self.palette[226]
    }

    /// `command_block` — Command Block.
    pub fn command_block(&self) -> &Block {
        &self.palette[227]
    }

    /// `comparator` — Redstone Comparator.
    pub fn comparator(&self) -> &Block {
        &self.palette[228]
    }

    /// `composter` — Composter.
    pub fn composter(&self) -> &Block {
        &self.palette[229]
    }

    /// `conduit` — Conduit.
    pub fn conduit(&self) -> &Block {
        &self.palette[230]
    }

    /// `copper_bars` — Copper Bars.
    pub fn copper_bars(&self) -> &Block {
        &self.palette[231]
    }

    /// `copper_block` — Block of Copper.
    pub fn copper_block(&self) -> &Block {
        &self.palette[232]
    }

    /// `copper_bulb` — Copper Bulb.
    pub fn copper_bulb(&self) -> &Block {
        &self.palette[233]
    }

    /// `copper_chain` — Copper Chain.
    pub fn copper_chain(&self) -> &Block {
        &self.palette[234]
    }

    /// `copper_chest` — Copper Chest.
    pub fn copper_chest(&self) -> &Block {
        &self.palette[235]
    }

    /// `copper_door` — Copper Door.
    pub fn copper_door(&self) -> &Block {
        &self.palette[236]
    }

    /// `copper_golem_statue` — Copper Golem Statue.
    pub fn copper_golem_statue(&self) -> &Block {
        &self.palette[237]
    }

    /// `copper_grate` — Copper Grate.
    pub fn copper_grate(&self) -> &Block {
        &self.palette[238]
    }

    /// `copper_lantern` — Copper Lantern.
    pub fn copper_lantern(&self) -> &Block {
        &self.palette[239]
    }

    /// `copper_ore` — Copper Ore.
    pub fn copper_ore(&self) -> &Block {
        &self.palette[240]
    }

    /// `copper_torch` — Copper Torch.
    pub fn copper_torch(&self) -> &Block {
        &self.palette[241]
    }

    /// `copper_trapdoor` — Copper Trapdoor.
    pub fn copper_trapdoor(&self) -> &Block {
        &self.palette[242]
    }

    /// `copper_wall_torch` — Copper Torch.
    pub fn copper_wall_torch(&self) -> &Block {
        &self.palette[243]
    }

    /// `cornflower` — Cornflower.
    pub fn cornflower(&self) -> &Block {
        &self.palette[244]
    }

    /// `cracked_deepslate_bricks` — Cracked Deepslate Bricks.
    pub fn cracked_deepslate_bricks(&self) -> &Block {
        &self.palette[245]
    }

    /// `cracked_deepslate_tiles` — Cracked Deepslate Tiles.
    pub fn cracked_deepslate_tiles(&self) -> &Block {
        &self.palette[246]
    }

    /// `cracked_nether_bricks` — Cracked Nether Bricks.
    pub fn cracked_nether_bricks(&self) -> &Block {
        &self.palette[247]
    }

    /// `cracked_polished_blackstone_bricks` — Cracked Polished Blackstone Bricks.
    pub fn cracked_polished_blackstone_bricks(&self) -> &Block {
        &self.palette[248]
    }

    /// `cracked_stone_bricks` — Cracked Stone Bricks.
    pub fn cracked_stone_bricks(&self) -> &Block {
        &self.palette[249]
    }

    /// `crafter` — Crafter.
    pub fn crafter(&self) -> &Block {
        &self.palette[250]
    }

    /// `crafting_table` — Crafting Table.
    pub fn crafting_table(&self) -> &Block {
        &self.palette[251]
    }

    /// `creaking_heart` — Creaking Heart.
    pub fn creaking_heart(&self) -> &Block {
        &self.palette[252]
    }

    /// `creeper_head` — Creeper Head.
    pub fn creeper_head(&self) -> &Block {
        &self.palette[253]
    }

    /// `creeper_wall_head` — Creeper Head.
    pub fn creeper_wall_head(&self) -> &Block {
        &self.palette[254]
    }

    /// `crimson_button` — Crimson Button.
    pub fn crimson_button(&self) -> &Block {
        &self.palette[255]
    }

    /// `crimson_door` — Crimson Door.
    pub fn crimson_door(&self) -> &Block {
        &self.palette[256]
    }

    /// `crimson_fence` — Crimson Fence.
    pub fn crimson_fence(&self) -> &Block {
        &self.palette[257]
    }

    /// `crimson_fence_gate` — Crimson Fence Gate.
    pub fn crimson_fence_gate(&self) -> &Block {
        &self.palette[258]
    }

    /// `crimson_fungus` — Crimson Fungus.
    pub fn crimson_fungus(&self) -> &Block {
        &self.palette[259]
    }

    /// `crimson_hanging_sign` — Crimson Hanging Sign.
    pub fn crimson_hanging_sign(&self) -> &Block {
        &self.palette[260]
    }

    /// `crimson_hyphae` — Crimson Hyphae.
    pub fn crimson_hyphae(&self) -> &Block {
        &self.palette[261]
    }

    /// `crimson_nylium` — Crimson Nylium.
    pub fn crimson_nylium(&self) -> &Block {
        &self.palette[262]
    }

    /// `crimson_planks` — Crimson Planks.
    pub fn crimson_planks(&self) -> &Block {
        &self.palette[263]
    }

    /// `crimson_pressure_plate` — Crimson Pressure Plate.
    pub fn crimson_pressure_plate(&self) -> &Block {
        &self.palette[264]
    }

    /// `crimson_roots` — Crimson Roots.
    pub fn crimson_roots(&self) -> &Block {
        &self.palette[265]
    }

    /// `crimson_shelf` — Crimson Shelf.
    pub fn crimson_shelf(&self) -> &Block {
        &self.palette[266]
    }

    /// `crimson_sign` — Crimson Sign.
    pub fn crimson_sign(&self) -> &Block {
        &self.palette[267]
    }

    /// `crimson_slab` — Crimson Slab.
    pub fn crimson_slab(&self) -> &Block {
        &self.palette[268]
    }

    /// `crimson_stairs` — Crimson Stairs.
    pub fn crimson_stairs(&self) -> &Block {
        &self.palette[269]
    }

    /// `crimson_stem` — Crimson Stem.
    pub fn crimson_stem(&self) -> &Block {
        &self.palette[270]
    }

    /// `crimson_trapdoor` — Crimson Trapdoor.
    pub fn crimson_trapdoor(&self) -> &Block {
        &self.palette[271]
    }

    /// `crimson_wall_hanging_sign` — Crimson Hanging Sign.
    pub fn crimson_wall_hanging_sign(&self) -> &Block {
        &self.palette[272]
    }

    /// `crimson_wall_sign` — Crimson Sign.
    pub fn crimson_wall_sign(&self) -> &Block {
        &self.palette[273]
    }

    /// `crying_obsidian` — Crying Obsidian.
    pub fn crying_obsidian(&self) -> &Block {
        &self.palette[274]
    }

    /// `cut_copper` — Cut Copper.
    pub fn cut_copper(&self) -> &Block {
        &self.palette[275]
    }

    /// `cut_copper_slab` — Cut Copper Slab.
    pub fn cut_copper_slab(&self) -> &Block {
        &self.palette[276]
    }

    /// `cut_copper_stairs` — Cut Copper Stairs.
    pub fn cut_copper_stairs(&self) -> &Block {
        &self.palette[277]
    }

    /// `cut_red_sandstone` — Cut Red Sandstone.
    pub fn cut_red_sandstone(&self) -> &Block {
        &self.palette[278]
    }

    /// `cut_red_sandstone_slab` — Cut Red Sandstone Slab.
    pub fn cut_red_sandstone_slab(&self) -> &Block {
        &self.palette[279]
    }

    /// `cut_sandstone` — Cut Sandstone.
    pub fn cut_sandstone(&self) -> &Block {
        &self.palette[280]
    }

    /// `cut_sandstone_slab` — Cut Sandstone Slab.
    pub fn cut_sandstone_slab(&self) -> &Block {
        &self.palette[281]
    }

    /// `cyan_banner` — Cyan Banner.
    pub fn cyan_banner(&self) -> &Block {
        &self.palette[282]
    }

    /// `cyan_bed` — Cyan Bed.
    pub fn cyan_bed(&self) -> &Block {
        &self.palette[283]
    }

    /// `cyan_candle` — Cyan Candle.
    pub fn cyan_candle(&self) -> &Block {
        &self.palette[284]
    }

    /// `cyan_candle_cake` — Cake with Cyan Candle.
    pub fn cyan_candle_cake(&self) -> &Block {
        &self.palette[285]
    }

    /// `cyan_carpet` — Cyan Carpet.
    pub fn cyan_carpet(&self) -> &Block {
        &self.palette[286]
    }

    /// `cyan_concrete` — Cyan Concrete.
    pub fn cyan_concrete(&self) -> &Block {
        &self.palette[287]
    }

    /// `cyan_concrete_powder` — Cyan Concrete Powder.
    pub fn cyan_concrete_powder(&self) -> &Block {
        &self.palette[288]
    }

    /// `cyan_glazed_terracotta` — Cyan Glazed Terracotta.
    pub fn cyan_glazed_terracotta(&self) -> &Block {
        &self.palette[289]
    }

    /// `cyan_shulker_box` — Cyan Shulker Box.
    pub fn cyan_shulker_box(&self) -> &Block {
        &self.palette[290]
    }

    /// `cyan_stained_glass` — Cyan Stained Glass.
    pub fn cyan_stained_glass(&self) -> &Block {
        &self.palette[291]
    }

    /// `cyan_stained_glass_pane` — Cyan Stained Glass Pane.
    pub fn cyan_stained_glass_pane(&self) -> &Block {
        &self.palette[292]
    }

    /// `cyan_terracotta` — Cyan Terracotta.
    pub fn cyan_terracotta(&self) -> &Block {
        &self.palette[293]
    }

    /// `cyan_wall_banner` — Cyan Banner.
    pub fn cyan_wall_banner(&self) -> &Block {
        &self.palette[294]
    }

    /// `cyan_wool` — Cyan Wool.
    pub fn cyan_wool(&self) -> &Block {
        &self.palette[295]
    }

    /// `damaged_anvil` — Damaged Anvil.
    pub fn damaged_anvil(&self) -> &Block {
        &self.palette[296]
    }

    /// `dandelion` — Dandelion.
    pub fn dandelion(&self) -> &Block {
        &self.palette[297]
    }

    /// `dark_oak_button` — Dark Oak Button.
    pub fn dark_oak_button(&self) -> &Block {
        &self.palette[298]
    }

    /// `dark_oak_door` — Dark Oak Door.
    pub fn dark_oak_door(&self) -> &Block {
        &self.palette[299]
    }

    /// `dark_oak_fence` — Dark Oak Fence.
    pub fn dark_oak_fence(&self) -> &Block {
        &self.palette[300]
    }

    /// `dark_oak_fence_gate` — Dark Oak Fence Gate.
    pub fn dark_oak_fence_gate(&self) -> &Block {
        &self.palette[301]
    }

    /// `dark_oak_hanging_sign` — Dark Oak Hanging Sign.
    pub fn dark_oak_hanging_sign(&self) -> &Block {
        &self.palette[302]
    }

    /// `dark_oak_leaves` — Dark Oak Leaves.
    pub fn dark_oak_leaves(&self) -> &Block {
        &self.palette[303]
    }

    /// `dark_oak_log` — Dark Oak Log.
    pub fn dark_oak_log(&self) -> &Block {
        &self.palette[304]
    }

    /// `dark_oak_planks` — Dark Oak Planks.
    pub fn dark_oak_planks(&self) -> &Block {
        &self.palette[305]
    }

    /// `dark_oak_pressure_plate` — Dark Oak Pressure Plate.
    pub fn dark_oak_pressure_plate(&self) -> &Block {
        &self.palette[306]
    }

    /// `dark_oak_sapling` — Dark Oak Sapling.
    pub fn dark_oak_sapling(&self) -> &Block {
        &self.palette[307]
    }

    /// `dark_oak_shelf` — Dark Oak Shelf.
    pub fn dark_oak_shelf(&self) -> &Block {
        &self.palette[308]
    }

    /// `dark_oak_sign` — Dark Oak Sign.
    pub fn dark_oak_sign(&self) -> &Block {
        &self.palette[309]
    }

    /// `dark_oak_slab` — Dark Oak Slab.
    pub fn dark_oak_slab(&self) -> &Block {
        &self.palette[310]
    }

    /// `dark_oak_stairs` — Dark Oak Stairs.
    pub fn dark_oak_stairs(&self) -> &Block {
        &self.palette[311]
    }

    /// `dark_oak_trapdoor` — Dark Oak Trapdoor.
    pub fn dark_oak_trapdoor(&self) -> &Block {
        &self.palette[312]
    }

    /// `dark_oak_wall_hanging_sign` — Dark Oak Hanging Sign.
    pub fn dark_oak_wall_hanging_sign(&self) -> &Block {
        &self.palette[313]
    }

    /// `dark_oak_wall_sign` — Dark Oak Sign.
    pub fn dark_oak_wall_sign(&self) -> &Block {
        &self.palette[314]
    }

    /// `dark_oak_wood` — Dark Oak Wood.
    pub fn dark_oak_wood(&self) -> &Block {
        &self.palette[315]
    }

    /// `dark_prismarine` — Dark Prismarine.
    pub fn dark_prismarine(&self) -> &Block {
        &self.palette[316]
    }

    /// `dark_prismarine_slab` — Dark Prismarine Slab.
    pub fn dark_prismarine_slab(&self) -> &Block {
        &self.palette[317]
    }

    /// `dark_prismarine_stairs` — Dark Prismarine Stairs.
    pub fn dark_prismarine_stairs(&self) -> &Block {
        &self.palette[318]
    }

    /// `daylight_detector` — Daylight Detector.
    pub fn daylight_detector(&self) -> &Block {
        &self.palette[319]
    }

    /// `dead_brain_coral` — Dead Brain Coral.
    pub fn dead_brain_coral(&self) -> &Block {
        &self.palette[320]
    }

    /// `dead_brain_coral_block` — Dead Brain Coral Block.
    pub fn dead_brain_coral_block(&self) -> &Block {
        &self.palette[321]
    }

    /// `dead_brain_coral_fan` — Dead Brain Coral Fan.
    pub fn dead_brain_coral_fan(&self) -> &Block {
        &self.palette[322]
    }

    /// `dead_brain_coral_wall_fan` — Dead Brain Coral Wall Fan.
    pub fn dead_brain_coral_wall_fan(&self) -> &Block {
        &self.palette[323]
    }

    /// `dead_bubble_coral` — Dead Bubble Coral.
    pub fn dead_bubble_coral(&self) -> &Block {
        &self.palette[324]
    }

    /// `dead_bubble_coral_block` — Dead Bubble Coral Block.
    pub fn dead_bubble_coral_block(&self) -> &Block {
        &self.palette[325]
    }

    /// `dead_bubble_coral_fan` — Dead Bubble Coral Fan.
    pub fn dead_bubble_coral_fan(&self) -> &Block {
        &self.palette[326]
    }

    /// `dead_bubble_coral_wall_fan` — Dead Bubble Coral Wall Fan.
    pub fn dead_bubble_coral_wall_fan(&self) -> &Block {
        &self.palette[327]
    }

    /// `dead_bush` — Dead Bush.
    pub fn dead_bush(&self) -> &Block {
        &self.palette[328]
    }

    /// `dead_fire_coral` — Dead Fire Coral.
    pub fn dead_fire_coral(&self) -> &Block {
        &self.palette[329]
    }

    /// `dead_fire_coral_block` — Dead Fire Coral Block.
    pub fn dead_fire_coral_block(&self) -> &Block {
        &self.palette[330]
    }

    /// `dead_fire_coral_fan` — Dead Fire Coral Fan.
    pub fn dead_fire_coral_fan(&self) -> &Block {
        &self.palette[331]
    }

    /// `dead_fire_coral_wall_fan` — Dead Fire Coral Wall Fan.
    pub fn dead_fire_coral_wall_fan(&self) -> &Block {
        &self.palette[332]
    }

    /// `dead_horn_coral` — Dead Horn Coral.
    pub fn dead_horn_coral(&self) -> &Block {
        &self.palette[333]
    }

    /// `dead_horn_coral_block` — Dead Horn Coral Block.
    pub fn dead_horn_coral_block(&self) -> &Block {
        &self.palette[334]
    }

    /// `dead_horn_coral_fan` — Dead Horn Coral Fan.
    pub fn dead_horn_coral_fan(&self) -> &Block {
        &self.palette[335]
    }

    /// `dead_horn_coral_wall_fan` — Dead Horn Coral Wall Fan.
    pub fn dead_horn_coral_wall_fan(&self) -> &Block {
        &self.palette[336]
    }

    /// `dead_tube_coral` — Dead Tube Coral.
    pub fn dead_tube_coral(&self) -> &Block {
        &self.palette[337]
    }

    /// `dead_tube_coral_block` — Dead Tube Coral Block.
    pub fn dead_tube_coral_block(&self) -> &Block {
        &self.palette[338]
    }

    /// `dead_tube_coral_fan` — Dead Tube Coral Fan.
    pub fn dead_tube_coral_fan(&self) -> &Block {
        &self.palette[339]
    }

    /// `dead_tube_coral_wall_fan` — Dead Tube Coral Wall Fan.
    pub fn dead_tube_coral_wall_fan(&self) -> &Block {
        &self.palette[340]
    }

    /// `decorated_pot` — Decorated Pot.
    pub fn decorated_pot(&self) -> &Block {
        &self.palette[341]
    }

    /// `deepslate` — Deepslate.
    pub fn deepslate(&self) -> &Block {
        &self.palette[342]
    }

    /// `deepslate_brick_slab` — Deepslate Brick Slab.
    pub fn deepslate_brick_slab(&self) -> &Block {
        &self.palette[343]
    }

    /// `deepslate_brick_stairs` — Deepslate Brick Stairs.
    pub fn deepslate_brick_stairs(&self) -> &Block {
        &self.palette[344]
    }

    /// `deepslate_brick_wall` — Deepslate Brick Wall.
    pub fn deepslate_brick_wall(&self) -> &Block {
        &self.palette[345]
    }

    /// `deepslate_bricks` — Deepslate Bricks.
    pub fn deepslate_bricks(&self) -> &Block {
        &self.palette[346]
    }

    /// `deepslate_coal_ore` — Deepslate Coal Ore.
    pub fn deepslate_coal_ore(&self) -> &Block {
        &self.palette[347]
    }

    /// `deepslate_copper_ore` — Deepslate Copper Ore.
    pub fn deepslate_copper_ore(&self) -> &Block {
        &self.palette[348]
    }

    /// `deepslate_diamond_ore` — Deepslate Diamond Ore.
    pub fn deepslate_diamond_ore(&self) -> &Block {
        &self.palette[349]
    }

    /// `deepslate_emerald_ore` — Deepslate Emerald Ore.
    pub fn deepslate_emerald_ore(&self) -> &Block {
        &self.palette[350]
    }

    /// `deepslate_gold_ore` — Deepslate Gold Ore.
    pub fn deepslate_gold_ore(&self) -> &Block {
        &self.palette[351]
    }

    /// `deepslate_iron_ore` — Deepslate Iron Ore.
    pub fn deepslate_iron_ore(&self) -> &Block {
        &self.palette[352]
    }

    /// `deepslate_lapis_ore` — Deepslate Lapis Lazuli Ore.
    pub fn deepslate_lapis_ore(&self) -> &Block {
        &self.palette[353]
    }

    /// `deepslate_redstone_ore` — Deepslate Redstone Ore.
    pub fn deepslate_redstone_ore(&self) -> &Block {
        &self.palette[354]
    }

    /// `deepslate_tile_slab` — Deepslate Tile Slab.
    pub fn deepslate_tile_slab(&self) -> &Block {
        &self.palette[355]
    }

    /// `deepslate_tile_stairs` — Deepslate Tile Stairs.
    pub fn deepslate_tile_stairs(&self) -> &Block {
        &self.palette[356]
    }

    /// `deepslate_tile_wall` — Deepslate Tile Wall.
    pub fn deepslate_tile_wall(&self) -> &Block {
        &self.palette[357]
    }

    /// `deepslate_tiles` — Deepslate Tiles.
    pub fn deepslate_tiles(&self) -> &Block {
        &self.palette[358]
    }

    /// `detector_rail` — Detector Rail.
    pub fn detector_rail(&self) -> &Block {
        &self.palette[359]
    }

    /// `diamond_block` — Block of Diamond.
    pub fn diamond_block(&self) -> &Block {
        &self.palette[360]
    }

    /// `diamond_ore` — Diamond Ore.
    pub fn diamond_ore(&self) -> &Block {
        &self.palette[361]
    }

    /// `diorite` — Diorite.
    pub fn diorite(&self) -> &Block {
        &self.palette[362]
    }

    /// `diorite_slab` — Diorite Slab.
    pub fn diorite_slab(&self) -> &Block {
        &self.palette[363]
    }

    /// `diorite_stairs` — Diorite Stairs.
    pub fn diorite_stairs(&self) -> &Block {
        &self.palette[364]
    }

    /// `diorite_wall` — Diorite Wall.
    pub fn diorite_wall(&self) -> &Block {
        &self.palette[365]
    }

    /// `dirt` — Dirt.
    pub fn dirt(&self) -> &Block {
        &self.palette[366]
    }

    /// `dirt_path` — Dirt Path.
    pub fn dirt_path(&self) -> &Block {
        &self.palette[367]
    }

    /// `dispenser` — Dispenser.
    pub fn dispenser(&self) -> &Block {
        &self.palette[368]
    }

    /// `dragon_egg` — Dragon Egg.
    pub fn dragon_egg(&self) -> &Block {
        &self.palette[369]
    }

    /// `dragon_head` — Dragon Head.
    pub fn dragon_head(&self) -> &Block {
        &self.palette[370]
    }

    /// `dragon_wall_head` — Dragon Head.
    pub fn dragon_wall_head(&self) -> &Block {
        &self.palette[371]
    }

    /// `dried_ghast` — Dried Ghast.
    pub fn dried_ghast(&self) -> &Block {
        &self.palette[372]
    }

    /// `dried_kelp_block` — Dried Kelp Block.
    pub fn dried_kelp_block(&self) -> &Block {
        &self.palette[373]
    }

    /// `dripstone_block` — Dripstone Block.
    pub fn dripstone_block(&self) -> &Block {
        &self.palette[374]
    }

    /// `dropper` — Dropper.
    pub fn dropper(&self) -> &Block {
        &self.palette[375]
    }

    /// `emerald_block` — Block of Emerald.
    pub fn emerald_block(&self) -> &Block {
        &self.palette[376]
    }

    /// `emerald_ore` — Emerald Ore.
    pub fn emerald_ore(&self) -> &Block {
        &self.palette[377]
    }

    /// `enchanting_table` — Enchanting Table.
    pub fn enchanting_table(&self) -> &Block {
        &self.palette[378]
    }

    /// `end_gateway` — End Gateway.
    pub fn end_gateway(&self) -> &Block {
        &self.palette[379]
    }

    /// `end_portal` — End Portal.
    pub fn end_portal(&self) -> &Block {
        &self.palette[380]
    }

    /// `end_portal_frame` — End Portal Frame.
    pub fn end_portal_frame(&self) -> &Block {
        &self.palette[381]
    }

    /// `end_rod` — End Rod.
    pub fn end_rod(&self) -> &Block {
        &self.palette[382]
    }

    /// `end_stone` — End Stone.
    pub fn end_stone(&self) -> &Block {
        &self.palette[383]
    }

    /// `end_stone_brick_slab` — End Stone Brick Slab.
    pub fn end_stone_brick_slab(&self) -> &Block {
        &self.palette[384]
    }

    /// `end_stone_brick_stairs` — End Stone Brick Stairs.
    pub fn end_stone_brick_stairs(&self) -> &Block {
        &self.palette[385]
    }

    /// `end_stone_brick_wall` — End Stone Brick Wall.
    pub fn end_stone_brick_wall(&self) -> &Block {
        &self.palette[386]
    }

    /// `end_stone_bricks` — End Stone Bricks.
    pub fn end_stone_bricks(&self) -> &Block {
        &self.palette[387]
    }

    /// `ender_chest` — Ender Chest.
    pub fn ender_chest(&self) -> &Block {
        &self.palette[388]
    }

    /// `exposed_chiseled_copper` — Exposed Chiseled Copper.
    pub fn exposed_chiseled_copper(&self) -> &Block {
        &self.palette[389]
    }

    /// `exposed_copper` — Exposed Copper.
    pub fn exposed_copper(&self) -> &Block {
        &self.palette[390]
    }

    /// `exposed_copper_bars` — Exposed Copper Bars.
    pub fn exposed_copper_bars(&self) -> &Block {
        &self.palette[391]
    }

    /// `exposed_copper_bulb` — Exposed Copper Bulb.
    pub fn exposed_copper_bulb(&self) -> &Block {
        &self.palette[392]
    }

    /// `exposed_copper_chain` — Exposed Copper Chain.
    pub fn exposed_copper_chain(&self) -> &Block {
        &self.palette[393]
    }

    /// `exposed_copper_chest` — Exposed Copper Chest.
    pub fn exposed_copper_chest(&self) -> &Block {
        &self.palette[394]
    }

    /// `exposed_copper_door` — Exposed Copper Door.
    pub fn exposed_copper_door(&self) -> &Block {
        &self.palette[395]
    }

    /// `exposed_copper_golem_statue` — Exposed Copper Golem Statue.
    pub fn exposed_copper_golem_statue(&self) -> &Block {
        &self.palette[396]
    }

    /// `exposed_copper_grate` — Exposed Copper Grate.
    pub fn exposed_copper_grate(&self) -> &Block {
        &self.palette[397]
    }

    /// `exposed_copper_lantern` — Exposed Copper Lantern.
    pub fn exposed_copper_lantern(&self) -> &Block {
        &self.palette[398]
    }

    /// `exposed_copper_trapdoor` — Exposed Copper Trapdoor.
    pub fn exposed_copper_trapdoor(&self) -> &Block {
        &self.palette[399]
    }

    /// `exposed_cut_copper` — Exposed Cut Copper.
    pub fn exposed_cut_copper(&self) -> &Block {
        &self.palette[400]
    }

    /// `exposed_cut_copper_slab` — Exposed Cut Copper Slab.
    pub fn exposed_cut_copper_slab(&self) -> &Block {
        &self.palette[401]
    }

    /// `exposed_cut_copper_stairs` — Exposed Cut Copper Stairs.
    pub fn exposed_cut_copper_stairs(&self) -> &Block {
        &self.palette[402]
    }

    /// `exposed_lightning_rod` — Exposed Lightning Rod.
    pub fn exposed_lightning_rod(&self) -> &Block {
        &self.palette[403]
    }

    /// `farmland` — Farmland.
    pub fn farmland(&self) -> &Block {
        &self.palette[404]
    }

    /// `fern` — Fern.
    pub fn fern(&self) -> &Block {
        &self.palette[405]
    }

    /// `fire` — Fire.
    pub fn fire(&self) -> &Block {
        &self.palette[406]
    }

    /// `fire_coral` — Fire Coral.
    pub fn fire_coral(&self) -> &Block {
        &self.palette[407]
    }

    /// `fire_coral_block` — Fire Coral Block.
    pub fn fire_coral_block(&self) -> &Block {
        &self.palette[408]
    }

    /// `fire_coral_fan` — Fire Coral Fan.
    pub fn fire_coral_fan(&self) -> &Block {
        &self.palette[409]
    }

    /// `fire_coral_wall_fan` — Fire Coral Wall Fan.
    pub fn fire_coral_wall_fan(&self) -> &Block {
        &self.palette[410]
    }

    /// `firefly_bush` — Firefly Bush.
    pub fn firefly_bush(&self) -> &Block {
        &self.palette[411]
    }

    /// `fletching_table` — Fletching Table.
    pub fn fletching_table(&self) -> &Block {
        &self.palette[412]
    }

    /// `flower_pot` — Flower Pot.
    pub fn flower_pot(&self) -> &Block {
        &self.palette[413]
    }

    /// `flowering_azalea` — Flowering Azalea.
    pub fn flowering_azalea(&self) -> &Block {
        &self.palette[414]
    }

    /// `flowering_azalea_leaves` — Flowering Azalea Leaves.
    pub fn flowering_azalea_leaves(&self) -> &Block {
        &self.palette[415]
    }

    /// `frogspawn` — Frogspawn.
    pub fn frogspawn(&self) -> &Block {
        &self.palette[416]
    }

    /// `frosted_ice` — Frosted Ice.
    pub fn frosted_ice(&self) -> &Block {
        &self.palette[417]
    }

    /// `furnace` — Furnace.
    pub fn furnace(&self) -> &Block {
        &self.palette[418]
    }

    /// `gilded_blackstone` — Gilded Blackstone.
    pub fn gilded_blackstone(&self) -> &Block {
        &self.palette[419]
    }

    /// `glass` — Glass.
    pub fn glass(&self) -> &Block {
        &self.palette[420]
    }

    /// `glass_pane` — Glass Pane.
    pub fn glass_pane(&self) -> &Block {
        &self.palette[421]
    }

    /// `glow_lichen` — Glow Lichen.
    pub fn glow_lichen(&self) -> &Block {
        &self.palette[422]
    }

    /// `glowstone` — Glowstone.
    pub fn glowstone(&self) -> &Block {
        &self.palette[423]
    }

    /// `gold_block` — Block of Gold.
    pub fn gold_block(&self) -> &Block {
        &self.palette[424]
    }

    /// `gold_ore` — Gold Ore.
    pub fn gold_ore(&self) -> &Block {
        &self.palette[425]
    }

    /// `golden_dandelion` — Golden Dandelion.
    pub fn golden_dandelion(&self) -> &Block {
        &self.palette[426]
    }

    /// `granite` — Granite.
    pub fn granite(&self) -> &Block {
        &self.palette[427]
    }

    /// `granite_slab` — Granite Slab.
    pub fn granite_slab(&self) -> &Block {
        &self.palette[428]
    }

    /// `granite_stairs` — Granite Stairs.
    pub fn granite_stairs(&self) -> &Block {
        &self.palette[429]
    }

    /// `granite_wall` — Granite Wall.
    pub fn granite_wall(&self) -> &Block {
        &self.palette[430]
    }

    /// `grass_block` — Grass Block.
    pub fn grass_block(&self) -> &Block {
        &self.palette[431]
    }

    /// `gravel` — Gravel.
    pub fn gravel(&self) -> &Block {
        &self.palette[432]
    }

    /// `gray_banner` — Gray Banner.
    pub fn gray_banner(&self) -> &Block {
        &self.palette[433]
    }

    /// `gray_bed` — Gray Bed.
    pub fn gray_bed(&self) -> &Block {
        &self.palette[434]
    }

    /// `gray_candle` — Gray Candle.
    pub fn gray_candle(&self) -> &Block {
        &self.palette[435]
    }

    /// `gray_candle_cake` — Cake with Gray Candle.
    pub fn gray_candle_cake(&self) -> &Block {
        &self.palette[436]
    }

    /// `gray_carpet` — Gray Carpet.
    pub fn gray_carpet(&self) -> &Block {
        &self.palette[437]
    }

    /// `gray_concrete` — Gray Concrete.
    pub fn gray_concrete(&self) -> &Block {
        &self.palette[438]
    }

    /// `gray_concrete_powder` — Gray Concrete Powder.
    pub fn gray_concrete_powder(&self) -> &Block {
        &self.palette[439]
    }

    /// `gray_glazed_terracotta` — Gray Glazed Terracotta.
    pub fn gray_glazed_terracotta(&self) -> &Block {
        &self.palette[440]
    }

    /// `gray_shulker_box` — Gray Shulker Box.
    pub fn gray_shulker_box(&self) -> &Block {
        &self.palette[441]
    }

    /// `gray_stained_glass` — Gray Stained Glass.
    pub fn gray_stained_glass(&self) -> &Block {
        &self.palette[442]
    }

    /// `gray_stained_glass_pane` — Gray Stained Glass Pane.
    pub fn gray_stained_glass_pane(&self) -> &Block {
        &self.palette[443]
    }

    /// `gray_terracotta` — Gray Terracotta.
    pub fn gray_terracotta(&self) -> &Block {
        &self.palette[444]
    }

    /// `gray_wall_banner` — Gray Banner.
    pub fn gray_wall_banner(&self) -> &Block {
        &self.palette[445]
    }

    /// `gray_wool` — Gray Wool.
    pub fn gray_wool(&self) -> &Block {
        &self.palette[446]
    }

    /// `green_banner` — Green Banner.
    pub fn green_banner(&self) -> &Block {
        &self.palette[447]
    }

    /// `green_bed` — Green Bed.
    pub fn green_bed(&self) -> &Block {
        &self.palette[448]
    }

    /// `green_candle` — Green Candle.
    pub fn green_candle(&self) -> &Block {
        &self.palette[449]
    }

    /// `green_candle_cake` — Cake with Green Candle.
    pub fn green_candle_cake(&self) -> &Block {
        &self.palette[450]
    }

    /// `green_carpet` — Green Carpet.
    pub fn green_carpet(&self) -> &Block {
        &self.palette[451]
    }

    /// `green_concrete` — Green Concrete.
    pub fn green_concrete(&self) -> &Block {
        &self.palette[452]
    }

    /// `green_concrete_powder` — Green Concrete Powder.
    pub fn green_concrete_powder(&self) -> &Block {
        &self.palette[453]
    }

    /// `green_glazed_terracotta` — Green Glazed Terracotta.
    pub fn green_glazed_terracotta(&self) -> &Block {
        &self.palette[454]
    }

    /// `green_shulker_box` — Green Shulker Box.
    pub fn green_shulker_box(&self) -> &Block {
        &self.palette[455]
    }

    /// `green_stained_glass` — Green Stained Glass.
    pub fn green_stained_glass(&self) -> &Block {
        &self.palette[456]
    }

    /// `green_stained_glass_pane` — Green Stained Glass Pane.
    pub fn green_stained_glass_pane(&self) -> &Block {
        &self.palette[457]
    }

    /// `green_terracotta` — Green Terracotta.
    pub fn green_terracotta(&self) -> &Block {
        &self.palette[458]
    }

    /// `green_wall_banner` — Green Banner.
    pub fn green_wall_banner(&self) -> &Block {
        &self.palette[459]
    }

    /// `green_wool` — Green Wool.
    pub fn green_wool(&self) -> &Block {
        &self.palette[460]
    }

    /// `grindstone` — Grindstone.
    pub fn grindstone(&self) -> &Block {
        &self.palette[461]
    }

    /// `hanging_roots` — Hanging Roots.
    pub fn hanging_roots(&self) -> &Block {
        &self.palette[462]
    }

    /// `hay_block` — Hay Bale.
    pub fn hay_block(&self) -> &Block {
        &self.palette[463]
    }

    /// `heavy_core` — Heavy Core.
    pub fn heavy_core(&self) -> &Block {
        &self.palette[464]
    }

    /// `heavy_weighted_pressure_plate` — Heavy Weighted Pressure Plate.
    pub fn heavy_weighted_pressure_plate(&self) -> &Block {
        &self.palette[465]
    }

    /// `honey_block` — Honey Block.
    pub fn honey_block(&self) -> &Block {
        &self.palette[466]
    }

    /// `honeycomb_block` — Honeycomb Block.
    pub fn honeycomb_block(&self) -> &Block {
        &self.palette[467]
    }

    /// `hopper` — Hopper.
    pub fn hopper(&self) -> &Block {
        &self.palette[468]
    }

    /// `horn_coral` — Horn Coral.
    pub fn horn_coral(&self) -> &Block {
        &self.palette[469]
    }

    /// `horn_coral_block` — Horn Coral Block.
    pub fn horn_coral_block(&self) -> &Block {
        &self.palette[470]
    }

    /// `horn_coral_fan` — Horn Coral Fan.
    pub fn horn_coral_fan(&self) -> &Block {
        &self.palette[471]
    }

    /// `horn_coral_wall_fan` — Horn Coral Wall Fan.
    pub fn horn_coral_wall_fan(&self) -> &Block {
        &self.palette[472]
    }

    /// `ice` — Ice.
    pub fn ice(&self) -> &Block {
        &self.palette[473]
    }

    /// `infested_chiseled_stone_bricks` — Infested Chiseled Stone Bricks.
    pub fn infested_chiseled_stone_bricks(&self) -> &Block {
        &self.palette[474]
    }

    /// `infested_cobblestone` — Infested Cobblestone.
    pub fn infested_cobblestone(&self) -> &Block {
        &self.palette[475]
    }

    /// `infested_cracked_stone_bricks` — Infested Cracked Stone Bricks.
    pub fn infested_cracked_stone_bricks(&self) -> &Block {
        &self.palette[476]
    }

    /// `infested_deepslate` — Infested Deepslate.
    pub fn infested_deepslate(&self) -> &Block {
        &self.palette[477]
    }

    /// `infested_mossy_stone_bricks` — Infested Mossy Stone Bricks.
    pub fn infested_mossy_stone_bricks(&self) -> &Block {
        &self.palette[478]
    }

    /// `infested_stone` — Infested Stone.
    pub fn infested_stone(&self) -> &Block {
        &self.palette[479]
    }

    /// `infested_stone_bricks` — Infested Stone Bricks.
    pub fn infested_stone_bricks(&self) -> &Block {
        &self.palette[480]
    }

    /// `iron_bars` — Iron Bars.
    pub fn iron_bars(&self) -> &Block {
        &self.palette[481]
    }

    /// `iron_block` — Block of Iron.
    pub fn iron_block(&self) -> &Block {
        &self.palette[482]
    }

    /// `iron_chain` — Iron Chain.
    pub fn iron_chain(&self) -> &Block {
        &self.palette[483]
    }

    /// `iron_door` — Iron Door.
    pub fn iron_door(&self) -> &Block {
        &self.palette[484]
    }

    /// `iron_ore` — Iron Ore.
    pub fn iron_ore(&self) -> &Block {
        &self.palette[485]
    }

    /// `iron_trapdoor` — Iron Trapdoor.
    pub fn iron_trapdoor(&self) -> &Block {
        &self.palette[486]
    }

    /// `jack_o_lantern` — Jack o'Lantern.
    pub fn jack_o_lantern(&self) -> &Block {
        &self.palette[487]
    }

    /// `jigsaw` — Jigsaw Block.
    pub fn jigsaw(&self) -> &Block {
        &self.palette[488]
    }

    /// `jukebox` — Jukebox.
    pub fn jukebox(&self) -> &Block {
        &self.palette[489]
    }

    /// `jungle_button` — Jungle Button.
    pub fn jungle_button(&self) -> &Block {
        &self.palette[490]
    }

    /// `jungle_door` — Jungle Door.
    pub fn jungle_door(&self) -> &Block {
        &self.palette[491]
    }

    /// `jungle_fence` — Jungle Fence.
    pub fn jungle_fence(&self) -> &Block {
        &self.palette[492]
    }

    /// `jungle_fence_gate` — Jungle Fence Gate.
    pub fn jungle_fence_gate(&self) -> &Block {
        &self.palette[493]
    }

    /// `jungle_hanging_sign` — Jungle Hanging Sign.
    pub fn jungle_hanging_sign(&self) -> &Block {
        &self.palette[494]
    }

    /// `jungle_leaves` — Jungle Leaves.
    pub fn jungle_leaves(&self) -> &Block {
        &self.palette[495]
    }

    /// `jungle_log` — Jungle Log.
    pub fn jungle_log(&self) -> &Block {
        &self.palette[496]
    }

    /// `jungle_planks` — Jungle Planks.
    pub fn jungle_planks(&self) -> &Block {
        &self.palette[497]
    }

    /// `jungle_pressure_plate` — Jungle Pressure Plate.
    pub fn jungle_pressure_plate(&self) -> &Block {
        &self.palette[498]
    }

    /// `jungle_sapling` — Jungle Sapling.
    pub fn jungle_sapling(&self) -> &Block {
        &self.palette[499]
    }

    /// `jungle_shelf` — Jungle Shelf.
    pub fn jungle_shelf(&self) -> &Block {
        &self.palette[500]
    }

    /// `jungle_sign` — Jungle Sign.
    pub fn jungle_sign(&self) -> &Block {
        &self.palette[501]
    }

    /// `jungle_slab` — Jungle Slab.
    pub fn jungle_slab(&self) -> &Block {
        &self.palette[502]
    }

    /// `jungle_stairs` — Jungle Stairs.
    pub fn jungle_stairs(&self) -> &Block {
        &self.palette[503]
    }

    /// `jungle_trapdoor` — Jungle Trapdoor.
    pub fn jungle_trapdoor(&self) -> &Block {
        &self.palette[504]
    }

    /// `jungle_wall_hanging_sign` — Jungle Hanging Sign.
    pub fn jungle_wall_hanging_sign(&self) -> &Block {
        &self.palette[505]
    }

    /// `jungle_wall_sign` — Jungle Sign.
    pub fn jungle_wall_sign(&self) -> &Block {
        &self.palette[506]
    }

    /// `jungle_wood` — Jungle Wood.
    pub fn jungle_wood(&self) -> &Block {
        &self.palette[507]
    }

    /// `kelp` — Kelp.
    pub fn kelp(&self) -> &Block {
        &self.palette[508]
    }

    /// `kelp_plant` — Kelp Plant.
    pub fn kelp_plant(&self) -> &Block {
        &self.palette[509]
    }

    /// `ladder` — Ladder.
    pub fn ladder(&self) -> &Block {
        &self.palette[510]
    }

    /// `lantern` — Lantern.
    pub fn lantern(&self) -> &Block {
        &self.palette[511]
    }

    /// `lapis_block` — Block of Lapis Lazuli.
    pub fn lapis_block(&self) -> &Block {
        &self.palette[512]
    }

    /// `lapis_ore` — Lapis Lazuli Ore.
    pub fn lapis_ore(&self) -> &Block {
        &self.palette[513]
    }

    /// `large_amethyst_bud` — Large Amethyst Bud.
    pub fn large_amethyst_bud(&self) -> &Block {
        &self.palette[514]
    }

    /// `large_fern` — Large Fern.
    pub fn large_fern(&self) -> &Block {
        &self.palette[515]
    }

    /// `lava` — Lava.
    pub fn lava(&self) -> &Block {
        &self.palette[516]
    }

    /// `lava_cauldron` — Lava Cauldron.
    pub fn lava_cauldron(&self) -> &Block {
        &self.palette[517]
    }

    /// `leaf_litter` — Leaf Litter.
    pub fn leaf_litter(&self) -> &Block {
        &self.palette[518]
    }

    /// `lectern` — Lectern.
    pub fn lectern(&self) -> &Block {
        &self.palette[519]
    }

    /// `lever` — Lever.
    pub fn lever(&self) -> &Block {
        &self.palette[520]
    }

    /// `light` — Light.
    pub fn light(&self) -> &Block {
        &self.palette[521]
    }

    /// `light_blue_banner` — Light Blue Banner.
    pub fn light_blue_banner(&self) -> &Block {
        &self.palette[522]
    }

    /// `light_blue_bed` — Light Blue Bed.
    pub fn light_blue_bed(&self) -> &Block {
        &self.palette[523]
    }

    /// `light_blue_candle` — Light Blue Candle.
    pub fn light_blue_candle(&self) -> &Block {
        &self.palette[524]
    }

    /// `light_blue_candle_cake` — Cake with Light Blue Candle.
    pub fn light_blue_candle_cake(&self) -> &Block {
        &self.palette[525]
    }

    /// `light_blue_carpet` — Light Blue Carpet.
    pub fn light_blue_carpet(&self) -> &Block {
        &self.palette[526]
    }

    /// `light_blue_concrete` — Light Blue Concrete.
    pub fn light_blue_concrete(&self) -> &Block {
        &self.palette[527]
    }

    /// `light_blue_concrete_powder` — Light Blue Concrete Powder.
    pub fn light_blue_concrete_powder(&self) -> &Block {
        &self.palette[528]
    }

    /// `light_blue_glazed_terracotta` — Light Blue Glazed Terracotta.
    pub fn light_blue_glazed_terracotta(&self) -> &Block {
        &self.palette[529]
    }

    /// `light_blue_shulker_box` — Light Blue Shulker Box.
    pub fn light_blue_shulker_box(&self) -> &Block {
        &self.palette[530]
    }

    /// `light_blue_stained_glass` — Light Blue Stained Glass.
    pub fn light_blue_stained_glass(&self) -> &Block {
        &self.palette[531]
    }

    /// `light_blue_stained_glass_pane` — Light Blue Stained Glass Pane.
    pub fn light_blue_stained_glass_pane(&self) -> &Block {
        &self.palette[532]
    }

    /// `light_blue_terracotta` — Light Blue Terracotta.
    pub fn light_blue_terracotta(&self) -> &Block {
        &self.palette[533]
    }

    /// `light_blue_wall_banner` — Light Blue Banner.
    pub fn light_blue_wall_banner(&self) -> &Block {
        &self.palette[534]
    }

    /// `light_blue_wool` — Light Blue Wool.
    pub fn light_blue_wool(&self) -> &Block {
        &self.palette[535]
    }

    /// `light_gray_banner` — Light Gray Banner.
    pub fn light_gray_banner(&self) -> &Block {
        &self.palette[536]
    }

    /// `light_gray_bed` — Light Gray Bed.
    pub fn light_gray_bed(&self) -> &Block {
        &self.palette[537]
    }

    /// `light_gray_candle` — Light Gray Candle.
    pub fn light_gray_candle(&self) -> &Block {
        &self.palette[538]
    }

    /// `light_gray_candle_cake` — Cake with Light Gray Candle.
    pub fn light_gray_candle_cake(&self) -> &Block {
        &self.palette[539]
    }

    /// `light_gray_carpet` — Light Gray Carpet.
    pub fn light_gray_carpet(&self) -> &Block {
        &self.palette[540]
    }

    /// `light_gray_concrete` — Light Gray Concrete.
    pub fn light_gray_concrete(&self) -> &Block {
        &self.palette[541]
    }

    /// `light_gray_concrete_powder` — Light Gray Concrete Powder.
    pub fn light_gray_concrete_powder(&self) -> &Block {
        &self.palette[542]
    }

    /// `light_gray_glazed_terracotta` — Light Gray Glazed Terracotta.
    pub fn light_gray_glazed_terracotta(&self) -> &Block {
        &self.palette[543]
    }

    /// `light_gray_shulker_box` — Light Gray Shulker Box.
    pub fn light_gray_shulker_box(&self) -> &Block {
        &self.palette[544]
    }

    /// `light_gray_stained_glass` — Light Gray Stained Glass.
    pub fn light_gray_stained_glass(&self) -> &Block {
        &self.palette[545]
    }

    /// `light_gray_stained_glass_pane` — Light Gray Stained Glass Pane.
    pub fn light_gray_stained_glass_pane(&self) -> &Block {
        &self.palette[546]
    }

    /// `light_gray_terracotta` — Light Gray Terracotta.
    pub fn light_gray_terracotta(&self) -> &Block {
        &self.palette[547]
    }

    /// `light_gray_wall_banner` — Light Gray Banner.
    pub fn light_gray_wall_banner(&self) -> &Block {
        &self.palette[548]
    }

    /// `light_gray_wool` — Light Gray Wool.
    pub fn light_gray_wool(&self) -> &Block {
        &self.palette[549]
    }

    /// `light_weighted_pressure_plate` — Light Weighted Pressure Plate.
    pub fn light_weighted_pressure_plate(&self) -> &Block {
        &self.palette[550]
    }

    /// `lightning_rod` — Lightning Rod.
    pub fn lightning_rod(&self) -> &Block {
        &self.palette[551]
    }

    /// `lilac` — Lilac.
    pub fn lilac(&self) -> &Block {
        &self.palette[552]
    }

    /// `lily_of_the_valley` — Lily of the Valley.
    pub fn lily_of_the_valley(&self) -> &Block {
        &self.palette[553]
    }

    /// `lily_pad` — Lily Pad.
    pub fn lily_pad(&self) -> &Block {
        &self.palette[554]
    }

    /// `lime_banner` — Lime Banner.
    pub fn lime_banner(&self) -> &Block {
        &self.palette[555]
    }

    /// `lime_bed` — Lime Bed.
    pub fn lime_bed(&self) -> &Block {
        &self.palette[556]
    }

    /// `lime_candle` — Lime Candle.
    pub fn lime_candle(&self) -> &Block {
        &self.palette[557]
    }

    /// `lime_candle_cake` — Cake with Lime Candle.
    pub fn lime_candle_cake(&self) -> &Block {
        &self.palette[558]
    }

    /// `lime_carpet` — Lime Carpet.
    pub fn lime_carpet(&self) -> &Block {
        &self.palette[559]
    }

    /// `lime_concrete` — Lime Concrete.
    pub fn lime_concrete(&self) -> &Block {
        &self.palette[560]
    }

    /// `lime_concrete_powder` — Lime Concrete Powder.
    pub fn lime_concrete_powder(&self) -> &Block {
        &self.palette[561]
    }

    /// `lime_glazed_terracotta` — Lime Glazed Terracotta.
    pub fn lime_glazed_terracotta(&self) -> &Block {
        &self.palette[562]
    }

    /// `lime_shulker_box` — Lime Shulker Box.
    pub fn lime_shulker_box(&self) -> &Block {
        &self.palette[563]
    }

    /// `lime_stained_glass` — Lime Stained Glass.
    pub fn lime_stained_glass(&self) -> &Block {
        &self.palette[564]
    }

    /// `lime_stained_glass_pane` — Lime Stained Glass Pane.
    pub fn lime_stained_glass_pane(&self) -> &Block {
        &self.palette[565]
    }

    /// `lime_terracotta` — Lime Terracotta.
    pub fn lime_terracotta(&self) -> &Block {
        &self.palette[566]
    }

    /// `lime_wall_banner` — Lime Banner.
    pub fn lime_wall_banner(&self) -> &Block {
        &self.palette[567]
    }

    /// `lime_wool` — Lime Wool.
    pub fn lime_wool(&self) -> &Block {
        &self.palette[568]
    }

    /// `lodestone` — Lodestone.
    pub fn lodestone(&self) -> &Block {
        &self.palette[569]
    }

    /// `loom` — Loom.
    pub fn loom(&self) -> &Block {
        &self.palette[570]
    }

    /// `magenta_banner` — Magenta Banner.
    pub fn magenta_banner(&self) -> &Block {
        &self.palette[571]
    }

    /// `magenta_bed` — Magenta Bed.
    pub fn magenta_bed(&self) -> &Block {
        &self.palette[572]
    }

    /// `magenta_candle` — Magenta Candle.
    pub fn magenta_candle(&self) -> &Block {
        &self.palette[573]
    }

    /// `magenta_candle_cake` — Cake with Magenta Candle.
    pub fn magenta_candle_cake(&self) -> &Block {
        &self.palette[574]
    }

    /// `magenta_carpet` — Magenta Carpet.
    pub fn magenta_carpet(&self) -> &Block {
        &self.palette[575]
    }

    /// `magenta_concrete` — Magenta Concrete.
    pub fn magenta_concrete(&self) -> &Block {
        &self.palette[576]
    }

    /// `magenta_concrete_powder` — Magenta Concrete Powder.
    pub fn magenta_concrete_powder(&self) -> &Block {
        &self.palette[577]
    }

    /// `magenta_glazed_terracotta` — Magenta Glazed Terracotta.
    pub fn magenta_glazed_terracotta(&self) -> &Block {
        &self.palette[578]
    }

    /// `magenta_shulker_box` — Magenta Shulker Box.
    pub fn magenta_shulker_box(&self) -> &Block {
        &self.palette[579]
    }

    /// `magenta_stained_glass` — Magenta Stained Glass.
    pub fn magenta_stained_glass(&self) -> &Block {
        &self.palette[580]
    }

    /// `magenta_stained_glass_pane` — Magenta Stained Glass Pane.
    pub fn magenta_stained_glass_pane(&self) -> &Block {
        &self.palette[581]
    }

    /// `magenta_terracotta` — Magenta Terracotta.
    pub fn magenta_terracotta(&self) -> &Block {
        &self.palette[582]
    }

    /// `magenta_wall_banner` — Magenta Banner.
    pub fn magenta_wall_banner(&self) -> &Block {
        &self.palette[583]
    }

    /// `magenta_wool` — Magenta Wool.
    pub fn magenta_wool(&self) -> &Block {
        &self.palette[584]
    }

    /// `magma_block` — Magma Block.
    pub fn magma_block(&self) -> &Block {
        &self.palette[585]
    }

    /// `mangrove_button` — Mangrove Button.
    pub fn mangrove_button(&self) -> &Block {
        &self.palette[586]
    }

    /// `mangrove_door` — Mangrove Door.
    pub fn mangrove_door(&self) -> &Block {
        &self.palette[587]
    }

    /// `mangrove_fence` — Mangrove Fence.
    pub fn mangrove_fence(&self) -> &Block {
        &self.palette[588]
    }

    /// `mangrove_fence_gate` — Mangrove Fence Gate.
    pub fn mangrove_fence_gate(&self) -> &Block {
        &self.palette[589]
    }

    /// `mangrove_hanging_sign` — Mangrove Hanging Sign.
    pub fn mangrove_hanging_sign(&self) -> &Block {
        &self.palette[590]
    }

    /// `mangrove_leaves` — Mangrove Leaves.
    pub fn mangrove_leaves(&self) -> &Block {
        &self.palette[591]
    }

    /// `mangrove_log` — Mangrove Log.
    pub fn mangrove_log(&self) -> &Block {
        &self.palette[592]
    }

    /// `mangrove_planks` — Mangrove Planks.
    pub fn mangrove_planks(&self) -> &Block {
        &self.palette[593]
    }

    /// `mangrove_pressure_plate` — Mangrove Pressure Plate.
    pub fn mangrove_pressure_plate(&self) -> &Block {
        &self.palette[594]
    }

    /// `mangrove_propagule` — Mangrove Propagule.
    pub fn mangrove_propagule(&self) -> &Block {
        &self.palette[595]
    }

    /// `mangrove_roots` — Mangrove Roots.
    pub fn mangrove_roots(&self) -> &Block {
        &self.palette[596]
    }

    /// `mangrove_shelf` — Mangrove Shelf.
    pub fn mangrove_shelf(&self) -> &Block {
        &self.palette[597]
    }

    /// `mangrove_sign` — Mangrove Sign.
    pub fn mangrove_sign(&self) -> &Block {
        &self.palette[598]
    }

    /// `mangrove_slab` — Mangrove Slab.
    pub fn mangrove_slab(&self) -> &Block {
        &self.palette[599]
    }

    /// `mangrove_stairs` — Mangrove Stairs.
    pub fn mangrove_stairs(&self) -> &Block {
        &self.palette[600]
    }

    /// `mangrove_trapdoor` — Mangrove Trapdoor.
    pub fn mangrove_trapdoor(&self) -> &Block {
        &self.palette[601]
    }

    /// `mangrove_wall_hanging_sign` — Mangrove Hanging Sign.
    pub fn mangrove_wall_hanging_sign(&self) -> &Block {
        &self.palette[602]
    }

    /// `mangrove_wall_sign` — Mangrove Sign.
    pub fn mangrove_wall_sign(&self) -> &Block {
        &self.palette[603]
    }

    /// `mangrove_wood` — Mangrove Wood.
    pub fn mangrove_wood(&self) -> &Block {
        &self.palette[604]
    }

    /// `medium_amethyst_bud` — Medium Amethyst Bud.
    pub fn medium_amethyst_bud(&self) -> &Block {
        &self.palette[605]
    }

    /// `melon` — Melon.
    pub fn melon(&self) -> &Block {
        &self.palette[606]
    }

    /// `melon_stem` — Melon Stem.
    pub fn melon_stem(&self) -> &Block {
        &self.palette[607]
    }

    /// `moss_block` — Moss Block.
    pub fn moss_block(&self) -> &Block {
        &self.palette[608]
    }

    /// `moss_carpet` — Moss Carpet.
    pub fn moss_carpet(&self) -> &Block {
        &self.palette[609]
    }

    /// `mossy_cobblestone` — Mossy Cobblestone.
    pub fn mossy_cobblestone(&self) -> &Block {
        &self.palette[610]
    }

    /// `mossy_cobblestone_slab` — Mossy Cobblestone Slab.
    pub fn mossy_cobblestone_slab(&self) -> &Block {
        &self.palette[611]
    }

    /// `mossy_cobblestone_stairs` — Mossy Cobblestone Stairs.
    pub fn mossy_cobblestone_stairs(&self) -> &Block {
        &self.palette[612]
    }

    /// `mossy_cobblestone_wall` — Mossy Cobblestone Wall.
    pub fn mossy_cobblestone_wall(&self) -> &Block {
        &self.palette[613]
    }

    /// `mossy_stone_brick_slab` — Mossy Stone Brick Slab.
    pub fn mossy_stone_brick_slab(&self) -> &Block {
        &self.palette[614]
    }

    /// `mossy_stone_brick_stairs` — Mossy Stone Brick Stairs.
    pub fn mossy_stone_brick_stairs(&self) -> &Block {
        &self.palette[615]
    }

    /// `mossy_stone_brick_wall` — Mossy Stone Brick Wall.
    pub fn mossy_stone_brick_wall(&self) -> &Block {
        &self.palette[616]
    }

    /// `mossy_stone_bricks` — Mossy Stone Bricks.
    pub fn mossy_stone_bricks(&self) -> &Block {
        &self.palette[617]
    }

    /// `moving_piston` — Moving Piston.
    pub fn moving_piston(&self) -> &Block {
        &self.palette[618]
    }

    /// `mud` — Mud.
    pub fn mud(&self) -> &Block {
        &self.palette[619]
    }

    /// `mud_brick_slab` — Mud Brick Slab.
    pub fn mud_brick_slab(&self) -> &Block {
        &self.palette[620]
    }

    /// `mud_brick_stairs` — Mud Brick Stairs.
    pub fn mud_brick_stairs(&self) -> &Block {
        &self.palette[621]
    }

    /// `mud_brick_wall` — Mud Brick Wall.
    pub fn mud_brick_wall(&self) -> &Block {
        &self.palette[622]
    }

    /// `mud_bricks` — Mud Bricks.
    pub fn mud_bricks(&self) -> &Block {
        &self.palette[623]
    }

    /// `muddy_mangrove_roots` — Muddy Mangrove Roots.
    pub fn muddy_mangrove_roots(&self) -> &Block {
        &self.palette[624]
    }

    /// `mushroom_stem` — Mushroom Stem.
    pub fn mushroom_stem(&self) -> &Block {
        &self.palette[625]
    }

    /// `mycelium` — Mycelium.
    pub fn mycelium(&self) -> &Block {
        &self.palette[626]
    }

    /// `nether_brick_fence` — Nether Brick Fence.
    pub fn nether_brick_fence(&self) -> &Block {
        &self.palette[627]
    }

    /// `nether_brick_slab` — Nether Brick Slab.
    pub fn nether_brick_slab(&self) -> &Block {
        &self.palette[628]
    }

    /// `nether_brick_stairs` — Nether Brick Stairs.
    pub fn nether_brick_stairs(&self) -> &Block {
        &self.palette[629]
    }

    /// `nether_brick_wall` — Nether Brick Wall.
    pub fn nether_brick_wall(&self) -> &Block {
        &self.palette[630]
    }

    /// `nether_bricks` — Nether Bricks.
    pub fn nether_bricks(&self) -> &Block {
        &self.palette[631]
    }

    /// `nether_gold_ore` — Nether Gold Ore.
    pub fn nether_gold_ore(&self) -> &Block {
        &self.palette[632]
    }

    /// `nether_portal` — Nether Portal.
    pub fn nether_portal(&self) -> &Block {
        &self.palette[633]
    }

    /// `nether_quartz_ore` — Nether Quartz Ore.
    pub fn nether_quartz_ore(&self) -> &Block {
        &self.palette[634]
    }

    /// `nether_sprouts` — Nether Sprouts.
    pub fn nether_sprouts(&self) -> &Block {
        &self.palette[635]
    }

    /// `nether_wart` — Nether Wart.
    pub fn nether_wart(&self) -> &Block {
        &self.palette[636]
    }

    /// `nether_wart_block` — Nether Wart Block.
    pub fn nether_wart_block(&self) -> &Block {
        &self.palette[637]
    }

    /// `netherite_block` — Block of Netherite.
    pub fn netherite_block(&self) -> &Block {
        &self.palette[638]
    }

    /// `netherrack` — Netherrack.
    pub fn netherrack(&self) -> &Block {
        &self.palette[639]
    }

    /// `note_block` — Note Block.
    pub fn note_block(&self) -> &Block {
        &self.palette[640]
    }

    /// `oak_button` — Oak Button.
    pub fn oak_button(&self) -> &Block {
        &self.palette[641]
    }

    /// `oak_door` — Oak Door.
    pub fn oak_door(&self) -> &Block {
        &self.palette[642]
    }

    /// `oak_fence` — Oak Fence.
    pub fn oak_fence(&self) -> &Block {
        &self.palette[643]
    }

    /// `oak_fence_gate` — Oak Fence Gate.
    pub fn oak_fence_gate(&self) -> &Block {
        &self.palette[644]
    }

    /// `oak_hanging_sign` — Oak Hanging Sign.
    pub fn oak_hanging_sign(&self) -> &Block {
        &self.palette[645]
    }

    /// `oak_leaves` — Oak Leaves.
    pub fn oak_leaves(&self) -> &Block {
        &self.palette[646]
    }

    /// `oak_log` — Oak Log.
    pub fn oak_log(&self) -> &Block {
        &self.palette[647]
    }

    /// `oak_planks` — Oak Planks.
    pub fn oak_planks(&self) -> &Block {
        &self.palette[648]
    }

    /// `oak_pressure_plate` — Oak Pressure Plate.
    pub fn oak_pressure_plate(&self) -> &Block {
        &self.palette[649]
    }

    /// `oak_sapling` — Oak Sapling.
    pub fn oak_sapling(&self) -> &Block {
        &self.palette[650]
    }

    /// `oak_shelf` — Oak Shelf.
    pub fn oak_shelf(&self) -> &Block {
        &self.palette[651]
    }

    /// `oak_sign` — Oak Sign.
    pub fn oak_sign(&self) -> &Block {
        &self.palette[652]
    }

    /// `oak_slab` — Oak Slab.
    pub fn oak_slab(&self) -> &Block {
        &self.palette[653]
    }

    /// `oak_stairs` — Oak Stairs.
    pub fn oak_stairs(&self) -> &Block {
        &self.palette[654]
    }

    /// `oak_trapdoor` — Oak Trapdoor.
    pub fn oak_trapdoor(&self) -> &Block {
        &self.palette[655]
    }

    /// `oak_wall_hanging_sign` — Oak Hanging Sign.
    pub fn oak_wall_hanging_sign(&self) -> &Block {
        &self.palette[656]
    }

    /// `oak_wall_sign` — Oak Sign.
    pub fn oak_wall_sign(&self) -> &Block {
        &self.palette[657]
    }

    /// `oak_wood` — Oak Wood.
    pub fn oak_wood(&self) -> &Block {
        &self.palette[658]
    }

    /// `observer` — Observer.
    pub fn observer(&self) -> &Block {
        &self.palette[659]
    }

    /// `obsidian` — Obsidian.
    pub fn obsidian(&self) -> &Block {
        &self.palette[660]
    }

    /// `ochre_froglight` — Ochre Froglight.
    pub fn ochre_froglight(&self) -> &Block {
        &self.palette[661]
    }

    /// `open_eyeblossom` — Open Eyeblossom.
    pub fn open_eyeblossom(&self) -> &Block {
        &self.palette[662]
    }

    /// `orange_banner` — Orange Banner.
    pub fn orange_banner(&self) -> &Block {
        &self.palette[663]
    }

    /// `orange_bed` — Orange Bed.
    pub fn orange_bed(&self) -> &Block {
        &self.palette[664]
    }

    /// `orange_candle` — Orange Candle.
    pub fn orange_candle(&self) -> &Block {
        &self.palette[665]
    }

    /// `orange_candle_cake` — Cake with Orange Candle.
    pub fn orange_candle_cake(&self) -> &Block {
        &self.palette[666]
    }

    /// `orange_carpet` — Orange Carpet.
    pub fn orange_carpet(&self) -> &Block {
        &self.palette[667]
    }

    /// `orange_concrete` — Orange Concrete.
    pub fn orange_concrete(&self) -> &Block {
        &self.palette[668]
    }

    /// `orange_concrete_powder` — Orange Concrete Powder.
    pub fn orange_concrete_powder(&self) -> &Block {
        &self.palette[669]
    }

    /// `orange_glazed_terracotta` — Orange Glazed Terracotta.
    pub fn orange_glazed_terracotta(&self) -> &Block {
        &self.palette[670]
    }

    /// `orange_shulker_box` — Orange Shulker Box.
    pub fn orange_shulker_box(&self) -> &Block {
        &self.palette[671]
    }

    /// `orange_stained_glass` — Orange Stained Glass.
    pub fn orange_stained_glass(&self) -> &Block {
        &self.palette[672]
    }

    /// `orange_stained_glass_pane` — Orange Stained Glass Pane.
    pub fn orange_stained_glass_pane(&self) -> &Block {
        &self.palette[673]
    }

    /// `orange_terracotta` — Orange Terracotta.
    pub fn orange_terracotta(&self) -> &Block {
        &self.palette[674]
    }

    /// `orange_tulip` — Orange Tulip.
    pub fn orange_tulip(&self) -> &Block {
        &self.palette[675]
    }

    /// `orange_wall_banner` — Orange Banner.
    pub fn orange_wall_banner(&self) -> &Block {
        &self.palette[676]
    }

    /// `orange_wool` — Orange Wool.
    pub fn orange_wool(&self) -> &Block {
        &self.palette[677]
    }

    /// `oxeye_daisy` — Oxeye Daisy.
    pub fn oxeye_daisy(&self) -> &Block {
        &self.palette[678]
    }

    /// `oxidized_chiseled_copper` — Oxidized Chiseled Copper.
    pub fn oxidized_chiseled_copper(&self) -> &Block {
        &self.palette[679]
    }

    /// `oxidized_copper` — Oxidized Copper.
    pub fn oxidized_copper(&self) -> &Block {
        &self.palette[680]
    }

    /// `oxidized_copper_bars` — Oxidized Copper Bars.
    pub fn oxidized_copper_bars(&self) -> &Block {
        &self.palette[681]
    }

    /// `oxidized_copper_bulb` — Oxidized Copper Bulb.
    pub fn oxidized_copper_bulb(&self) -> &Block {
        &self.palette[682]
    }

    /// `oxidized_copper_chain` — Oxidized Copper Chain.
    pub fn oxidized_copper_chain(&self) -> &Block {
        &self.palette[683]
    }

    /// `oxidized_copper_chest` — Oxidized Copper Chest.
    pub fn oxidized_copper_chest(&self) -> &Block {
        &self.palette[684]
    }

    /// `oxidized_copper_door` — Oxidized Copper Door.
    pub fn oxidized_copper_door(&self) -> &Block {
        &self.palette[685]
    }

    /// `oxidized_copper_golem_statue` — Oxidized Copper Golem Statue.
    pub fn oxidized_copper_golem_statue(&self) -> &Block {
        &self.palette[686]
    }

    /// `oxidized_copper_grate` — Oxidized Copper Grate.
    pub fn oxidized_copper_grate(&self) -> &Block {
        &self.palette[687]
    }

    /// `oxidized_copper_lantern` — Oxidized Copper Lantern.
    pub fn oxidized_copper_lantern(&self) -> &Block {
        &self.palette[688]
    }

    /// `oxidized_copper_trapdoor` — Oxidized Copper Trapdoor.
    pub fn oxidized_copper_trapdoor(&self) -> &Block {
        &self.palette[689]
    }

    /// `oxidized_cut_copper` — Oxidized Cut Copper.
    pub fn oxidized_cut_copper(&self) -> &Block {
        &self.palette[690]
    }

    /// `oxidized_cut_copper_slab` — Oxidized Cut Copper Slab.
    pub fn oxidized_cut_copper_slab(&self) -> &Block {
        &self.palette[691]
    }

    /// `oxidized_cut_copper_stairs` — Oxidized Cut Copper Stairs.
    pub fn oxidized_cut_copper_stairs(&self) -> &Block {
        &self.palette[692]
    }

    /// `oxidized_lightning_rod` — Oxidized Lightning Rod.
    pub fn oxidized_lightning_rod(&self) -> &Block {
        &self.palette[693]
    }

    /// `packed_ice` — Packed Ice.
    pub fn packed_ice(&self) -> &Block {
        &self.palette[694]
    }

    /// `packed_mud` — Packed Mud.
    pub fn packed_mud(&self) -> &Block {
        &self.palette[695]
    }

    /// `pale_hanging_moss` — Pale Hanging Moss.
    pub fn pale_hanging_moss(&self) -> &Block {
        &self.palette[696]
    }

    /// `pale_moss_block` — Pale Moss Block.
    pub fn pale_moss_block(&self) -> &Block {
        &self.palette[697]
    }

    /// `pale_moss_carpet` — Pale Moss Carpet.
    pub fn pale_moss_carpet(&self) -> &Block {
        &self.palette[698]
    }

    /// `pale_oak_button` — Pale Oak Button.
    pub fn pale_oak_button(&self) -> &Block {
        &self.palette[699]
    }

    /// `pale_oak_door` — Pale Oak Door.
    pub fn pale_oak_door(&self) -> &Block {
        &self.palette[700]
    }

    /// `pale_oak_fence` — Pale Oak Fence.
    pub fn pale_oak_fence(&self) -> &Block {
        &self.palette[701]
    }

    /// `pale_oak_fence_gate` — Pale Oak Fence Gate.
    pub fn pale_oak_fence_gate(&self) -> &Block {
        &self.palette[702]
    }

    /// `pale_oak_hanging_sign` — Pale Oak Hanging Sign.
    pub fn pale_oak_hanging_sign(&self) -> &Block {
        &self.palette[703]
    }

    /// `pale_oak_leaves` — Pale Oak Leaves.
    pub fn pale_oak_leaves(&self) -> &Block {
        &self.palette[704]
    }

    /// `pale_oak_log` — Pale Oak Log.
    pub fn pale_oak_log(&self) -> &Block {
        &self.palette[705]
    }

    /// `pale_oak_planks` — Pale Oak Planks.
    pub fn pale_oak_planks(&self) -> &Block {
        &self.palette[706]
    }

    /// `pale_oak_pressure_plate` — Pale Oak Pressure Plate.
    pub fn pale_oak_pressure_plate(&self) -> &Block {
        &self.palette[707]
    }

    /// `pale_oak_sapling` — Pale Oak Sapling.
    pub fn pale_oak_sapling(&self) -> &Block {
        &self.palette[708]
    }

    /// `pale_oak_shelf` — Pale Oak Shelf.
    pub fn pale_oak_shelf(&self) -> &Block {
        &self.palette[709]
    }

    /// `pale_oak_sign` — Pale Oak Sign.
    pub fn pale_oak_sign(&self) -> &Block {
        &self.palette[710]
    }

    /// `pale_oak_slab` — Pale Oak Slab.
    pub fn pale_oak_slab(&self) -> &Block {
        &self.palette[711]
    }

    /// `pale_oak_stairs` — Pale Oak Stairs.
    pub fn pale_oak_stairs(&self) -> &Block {
        &self.palette[712]
    }

    /// `pale_oak_trapdoor` — Pale Oak Trapdoor.
    pub fn pale_oak_trapdoor(&self) -> &Block {
        &self.palette[713]
    }

    /// `pale_oak_wall_hanging_sign` — Pale Oak Hanging Sign.
    pub fn pale_oak_wall_hanging_sign(&self) -> &Block {
        &self.palette[714]
    }

    /// `pale_oak_wall_sign` — Pale Oak Sign.
    pub fn pale_oak_wall_sign(&self) -> &Block {
        &self.palette[715]
    }

    /// `pale_oak_wood` — Pale Oak Wood.
    pub fn pale_oak_wood(&self) -> &Block {
        &self.palette[716]
    }

    /// `pearlescent_froglight` — Pearlescent Froglight.
    pub fn pearlescent_froglight(&self) -> &Block {
        &self.palette[717]
    }

    /// `peony` — Peony.
    pub fn peony(&self) -> &Block {
        &self.palette[718]
    }

    /// `petrified_oak_slab` — Petrified Oak Slab.
    pub fn petrified_oak_slab(&self) -> &Block {
        &self.palette[719]
    }

    /// `piglin_head` — Piglin Head.
    pub fn piglin_head(&self) -> &Block {
        &self.palette[720]
    }

    /// `piglin_wall_head` — Piglin Head.
    pub fn piglin_wall_head(&self) -> &Block {
        &self.palette[721]
    }

    /// `pink_banner` — Pink Banner.
    pub fn pink_banner(&self) -> &Block {
        &self.palette[722]
    }

    /// `pink_bed` — Pink Bed.
    pub fn pink_bed(&self) -> &Block {
        &self.palette[723]
    }

    /// `pink_candle` — Pink Candle.
    pub fn pink_candle(&self) -> &Block {
        &self.palette[724]
    }

    /// `pink_candle_cake` — Cake with Pink Candle.
    pub fn pink_candle_cake(&self) -> &Block {
        &self.palette[725]
    }

    /// `pink_carpet` — Pink Carpet.
    pub fn pink_carpet(&self) -> &Block {
        &self.palette[726]
    }

    /// `pink_concrete` — Pink Concrete.
    pub fn pink_concrete(&self) -> &Block {
        &self.palette[727]
    }

    /// `pink_concrete_powder` — Pink Concrete Powder.
    pub fn pink_concrete_powder(&self) -> &Block {
        &self.palette[728]
    }

    /// `pink_glazed_terracotta` — Pink Glazed Terracotta.
    pub fn pink_glazed_terracotta(&self) -> &Block {
        &self.palette[729]
    }

    /// `pink_petals` — Pink Petals.
    pub fn pink_petals(&self) -> &Block {
        &self.palette[730]
    }

    /// `pink_shulker_box` — Pink Shulker Box.
    pub fn pink_shulker_box(&self) -> &Block {
        &self.palette[731]
    }

    /// `pink_stained_glass` — Pink Stained Glass.
    pub fn pink_stained_glass(&self) -> &Block {
        &self.palette[732]
    }

    /// `pink_stained_glass_pane` — Pink Stained Glass Pane.
    pub fn pink_stained_glass_pane(&self) -> &Block {
        &self.palette[733]
    }

    /// `pink_terracotta` — Pink Terracotta.
    pub fn pink_terracotta(&self) -> &Block {
        &self.palette[734]
    }

    /// `pink_tulip` — Pink Tulip.
    pub fn pink_tulip(&self) -> &Block {
        &self.palette[735]
    }

    /// `pink_wall_banner` — Pink Banner.
    pub fn pink_wall_banner(&self) -> &Block {
        &self.palette[736]
    }

    /// `pink_wool` — Pink Wool.
    pub fn pink_wool(&self) -> &Block {
        &self.palette[737]
    }

    /// `piston` — Piston.
    pub fn piston(&self) -> &Block {
        &self.palette[738]
    }

    /// `piston_head` — Piston Head.
    pub fn piston_head(&self) -> &Block {
        &self.palette[739]
    }

    /// `pitcher_crop` — Pitcher Crop.
    pub fn pitcher_crop(&self) -> &Block {
        &self.palette[740]
    }

    /// `pitcher_plant` — Pitcher Plant.
    pub fn pitcher_plant(&self) -> &Block {
        &self.palette[741]
    }

    /// `player_head` — Player Head.
    pub fn player_head(&self) -> &Block {
        &self.palette[742]
    }

    /// `player_wall_head` — Player Head.
    pub fn player_wall_head(&self) -> &Block {
        &self.palette[743]
    }

    /// `podzol` — Podzol.
    pub fn podzol(&self) -> &Block {
        &self.palette[744]
    }

    /// `pointed_dripstone` — Pointed Dripstone.
    pub fn pointed_dripstone(&self) -> &Block {
        &self.palette[745]
    }

    /// `polished_andesite` — Polished Andesite.
    pub fn polished_andesite(&self) -> &Block {
        &self.palette[746]
    }

    /// `polished_andesite_slab` — Polished Andesite Slab.
    pub fn polished_andesite_slab(&self) -> &Block {
        &self.palette[747]
    }

    /// `polished_andesite_stairs` — Polished Andesite Stairs.
    pub fn polished_andesite_stairs(&self) -> &Block {
        &self.palette[748]
    }

    /// `polished_basalt` — Polished Basalt.
    pub fn polished_basalt(&self) -> &Block {
        &self.palette[749]
    }

    /// `polished_blackstone` — Polished Blackstone.
    pub fn polished_blackstone(&self) -> &Block {
        &self.palette[750]
    }

    /// `polished_blackstone_brick_slab` — Polished Blackstone Brick Slab.
    pub fn polished_blackstone_brick_slab(&self) -> &Block {
        &self.palette[751]
    }

    /// `polished_blackstone_brick_stairs` — Polished Blackstone Brick Stairs.
    pub fn polished_blackstone_brick_stairs(&self) -> &Block {
        &self.palette[752]
    }

    /// `polished_blackstone_brick_wall` — Polished Blackstone Brick Wall.
    pub fn polished_blackstone_brick_wall(&self) -> &Block {
        &self.palette[753]
    }

    /// `polished_blackstone_bricks` — Polished Blackstone Bricks.
    pub fn polished_blackstone_bricks(&self) -> &Block {
        &self.palette[754]
    }

    /// `polished_blackstone_button` — Polished Blackstone Button.
    pub fn polished_blackstone_button(&self) -> &Block {
        &self.palette[755]
    }

    /// `polished_blackstone_pressure_plate` — Polished Blackstone Pressure Plate.
    pub fn polished_blackstone_pressure_plate(&self) -> &Block {
        &self.palette[756]
    }

    /// `polished_blackstone_slab` — Polished Blackstone Slab.
    pub fn polished_blackstone_slab(&self) -> &Block {
        &self.palette[757]
    }

    /// `polished_blackstone_stairs` — Polished Blackstone Stairs.
    pub fn polished_blackstone_stairs(&self) -> &Block {
        &self.palette[758]
    }

    /// `polished_blackstone_wall` — Polished Blackstone Wall.
    pub fn polished_blackstone_wall(&self) -> &Block {
        &self.palette[759]
    }

    /// `polished_cinnabar` — Polished Cinnabar.
    pub fn polished_cinnabar(&self) -> &Block {
        &self.palette[760]
    }

    /// `polished_cinnabar_slab` — Polished Cinnabar Slab.
    pub fn polished_cinnabar_slab(&self) -> &Block {
        &self.palette[761]
    }

    /// `polished_cinnabar_stairs` — Polished Cinnabar Stairs.
    pub fn polished_cinnabar_stairs(&self) -> &Block {
        &self.palette[762]
    }

    /// `polished_cinnabar_wall` — Polished Cinnabar Wall.
    pub fn polished_cinnabar_wall(&self) -> &Block {
        &self.palette[763]
    }

    /// `polished_deepslate` — Polished Deepslate.
    pub fn polished_deepslate(&self) -> &Block {
        &self.palette[764]
    }

    /// `polished_deepslate_slab` — Polished Deepslate Slab.
    pub fn polished_deepslate_slab(&self) -> &Block {
        &self.palette[765]
    }

    /// `polished_deepslate_stairs` — Polished Deepslate Stairs.
    pub fn polished_deepslate_stairs(&self) -> &Block {
        &self.palette[766]
    }

    /// `polished_deepslate_wall` — Polished Deepslate Wall.
    pub fn polished_deepslate_wall(&self) -> &Block {
        &self.palette[767]
    }

    /// `polished_diorite` — Polished Diorite.
    pub fn polished_diorite(&self) -> &Block {
        &self.palette[768]
    }

    /// `polished_diorite_slab` — Polished Diorite Slab.
    pub fn polished_diorite_slab(&self) -> &Block {
        &self.palette[769]
    }

    /// `polished_diorite_stairs` — Polished Diorite Stairs.
    pub fn polished_diorite_stairs(&self) -> &Block {
        &self.palette[770]
    }

    /// `polished_granite` — Polished Granite.
    pub fn polished_granite(&self) -> &Block {
        &self.palette[771]
    }

    /// `polished_granite_slab` — Polished Granite Slab.
    pub fn polished_granite_slab(&self) -> &Block {
        &self.palette[772]
    }

    /// `polished_granite_stairs` — Polished Granite Stairs.
    pub fn polished_granite_stairs(&self) -> &Block {
        &self.palette[773]
    }

    /// `polished_sulfur` — Polished Sulfur.
    pub fn polished_sulfur(&self) -> &Block {
        &self.palette[774]
    }

    /// `polished_sulfur_slab` — Polished Sulfur Slab.
    pub fn polished_sulfur_slab(&self) -> &Block {
        &self.palette[775]
    }

    /// `polished_sulfur_stairs` — Polished Sulfur Stairs.
    pub fn polished_sulfur_stairs(&self) -> &Block {
        &self.palette[776]
    }

    /// `polished_sulfur_wall` — Polished Sulfur Wall.
    pub fn polished_sulfur_wall(&self) -> &Block {
        &self.palette[777]
    }

    /// `polished_tuff` — Polished Tuff.
    pub fn polished_tuff(&self) -> &Block {
        &self.palette[778]
    }

    /// `polished_tuff_slab` — Polished Tuff Slab.
    pub fn polished_tuff_slab(&self) -> &Block {
        &self.palette[779]
    }

    /// `polished_tuff_stairs` — Polished Tuff Stairs.
    pub fn polished_tuff_stairs(&self) -> &Block {
        &self.palette[780]
    }

    /// `polished_tuff_wall` — Polished Tuff Wall.
    pub fn polished_tuff_wall(&self) -> &Block {
        &self.palette[781]
    }

    /// `poppy` — Poppy.
    pub fn poppy(&self) -> &Block {
        &self.palette[782]
    }

    /// `potatoes` — Potatoes.
    pub fn potatoes(&self) -> &Block {
        &self.palette[783]
    }

    /// `potent_sulfur` — Potent Sulfur.
    pub fn potent_sulfur(&self) -> &Block {
        &self.palette[784]
    }

    /// `potted_acacia_sapling` — Potted Acacia Sapling.
    pub fn potted_acacia_sapling(&self) -> &Block {
        &self.palette[785]
    }

    /// `potted_allium` — Potted Allium.
    pub fn potted_allium(&self) -> &Block {
        &self.palette[786]
    }

    /// `potted_azalea_bush` — Potted Azalea.
    pub fn potted_azalea_bush(&self) -> &Block {
        &self.palette[787]
    }

    /// `potted_azure_bluet` — Potted Azure Bluet.
    pub fn potted_azure_bluet(&self) -> &Block {
        &self.palette[788]
    }

    /// `potted_bamboo` — Potted Bamboo.
    pub fn potted_bamboo(&self) -> &Block {
        &self.palette[789]
    }

    /// `potted_birch_sapling` — Potted Birch Sapling.
    pub fn potted_birch_sapling(&self) -> &Block {
        &self.palette[790]
    }

    /// `potted_blue_orchid` — Potted Blue Orchid.
    pub fn potted_blue_orchid(&self) -> &Block {
        &self.palette[791]
    }

    /// `potted_brown_mushroom` — Potted Brown Mushroom.
    pub fn potted_brown_mushroom(&self) -> &Block {
        &self.palette[792]
    }

    /// `potted_cactus` — Potted Cactus.
    pub fn potted_cactus(&self) -> &Block {
        &self.palette[793]
    }

    /// `potted_cherry_sapling` — Potted Cherry Sapling.
    pub fn potted_cherry_sapling(&self) -> &Block {
        &self.palette[794]
    }

    /// `potted_closed_eyeblossom` — Potted Closed Eyeblossom.
    pub fn potted_closed_eyeblossom(&self) -> &Block {
        &self.palette[795]
    }

    /// `potted_cornflower` — Potted Cornflower.
    pub fn potted_cornflower(&self) -> &Block {
        &self.palette[796]
    }

    /// `potted_crimson_fungus` — Potted Crimson Fungus.
    pub fn potted_crimson_fungus(&self) -> &Block {
        &self.palette[797]
    }

    /// `potted_crimson_roots` — Potted Crimson Roots.
    pub fn potted_crimson_roots(&self) -> &Block {
        &self.palette[798]
    }

    /// `potted_dandelion` — Potted Dandelion.
    pub fn potted_dandelion(&self) -> &Block {
        &self.palette[799]
    }

    /// `potted_dark_oak_sapling` — Potted Dark Oak Sapling.
    pub fn potted_dark_oak_sapling(&self) -> &Block {
        &self.palette[800]
    }

    /// `potted_dead_bush` — Potted Dead Bush.
    pub fn potted_dead_bush(&self) -> &Block {
        &self.palette[801]
    }

    /// `potted_fern` — Potted Fern.
    pub fn potted_fern(&self) -> &Block {
        &self.palette[802]
    }

    /// `potted_flowering_azalea_bush` — Potted Flowering Azalea.
    pub fn potted_flowering_azalea_bush(&self) -> &Block {
        &self.palette[803]
    }

    /// `potted_golden_dandelion` — Potted Golden Dandelion.
    pub fn potted_golden_dandelion(&self) -> &Block {
        &self.palette[804]
    }

    /// `potted_jungle_sapling` — Potted Jungle Sapling.
    pub fn potted_jungle_sapling(&self) -> &Block {
        &self.palette[805]
    }

    /// `potted_lily_of_the_valley` — Potted Lily of the Valley.
    pub fn potted_lily_of_the_valley(&self) -> &Block {
        &self.palette[806]
    }

    /// `potted_mangrove_propagule` — Potted Mangrove Propagule.
    pub fn potted_mangrove_propagule(&self) -> &Block {
        &self.palette[807]
    }

    /// `potted_oak_sapling` — Potted Oak Sapling.
    pub fn potted_oak_sapling(&self) -> &Block {
        &self.palette[808]
    }

    /// `potted_open_eyeblossom` — Potted Open Eyeblossom.
    pub fn potted_open_eyeblossom(&self) -> &Block {
        &self.palette[809]
    }

    /// `potted_orange_tulip` — Potted Orange Tulip.
    pub fn potted_orange_tulip(&self) -> &Block {
        &self.palette[810]
    }

    /// `potted_oxeye_daisy` — Potted Oxeye Daisy.
    pub fn potted_oxeye_daisy(&self) -> &Block {
        &self.palette[811]
    }

    /// `potted_pale_oak_sapling` — Potted Pale Oak Sapling.
    pub fn potted_pale_oak_sapling(&self) -> &Block {
        &self.palette[812]
    }

    /// `potted_pink_tulip` — Potted Pink Tulip.
    pub fn potted_pink_tulip(&self) -> &Block {
        &self.palette[813]
    }

    /// `potted_poppy` — Potted Poppy.
    pub fn potted_poppy(&self) -> &Block {
        &self.palette[814]
    }

    /// `potted_red_mushroom` — Potted Red Mushroom.
    pub fn potted_red_mushroom(&self) -> &Block {
        &self.palette[815]
    }

    /// `potted_red_tulip` — Potted Red Tulip.
    pub fn potted_red_tulip(&self) -> &Block {
        &self.palette[816]
    }

    /// `potted_spruce_sapling` — Potted Spruce Sapling.
    pub fn potted_spruce_sapling(&self) -> &Block {
        &self.palette[817]
    }

    /// `potted_torchflower` — Potted Torchflower.
    pub fn potted_torchflower(&self) -> &Block {
        &self.palette[818]
    }

    /// `potted_warped_fungus` — Potted Warped Fungus.
    pub fn potted_warped_fungus(&self) -> &Block {
        &self.palette[819]
    }

    /// `potted_warped_roots` — Potted Warped Roots.
    pub fn potted_warped_roots(&self) -> &Block {
        &self.palette[820]
    }

    /// `potted_white_tulip` — Potted White Tulip.
    pub fn potted_white_tulip(&self) -> &Block {
        &self.palette[821]
    }

    /// `potted_wither_rose` — Potted Wither Rose.
    pub fn potted_wither_rose(&self) -> &Block {
        &self.palette[822]
    }

    /// `powder_snow` — Powder Snow.
    pub fn powder_snow(&self) -> &Block {
        &self.palette[823]
    }

    /// `powder_snow_cauldron` — Powder Snow Cauldron.
    pub fn powder_snow_cauldron(&self) -> &Block {
        &self.palette[824]
    }

    /// `powered_rail` — Powered Rail.
    pub fn powered_rail(&self) -> &Block {
        &self.palette[825]
    }

    /// `prismarine` — Prismarine.
    pub fn prismarine(&self) -> &Block {
        &self.palette[826]
    }

    /// `prismarine_brick_slab` — Prismarine Brick Slab.
    pub fn prismarine_brick_slab(&self) -> &Block {
        &self.palette[827]
    }

    /// `prismarine_brick_stairs` — Prismarine Brick Stairs.
    pub fn prismarine_brick_stairs(&self) -> &Block {
        &self.palette[828]
    }

    /// `prismarine_bricks` — Prismarine Bricks.
    pub fn prismarine_bricks(&self) -> &Block {
        &self.palette[829]
    }

    /// `prismarine_slab` — Prismarine Slab.
    pub fn prismarine_slab(&self) -> &Block {
        &self.palette[830]
    }

    /// `prismarine_stairs` — Prismarine Stairs.
    pub fn prismarine_stairs(&self) -> &Block {
        &self.palette[831]
    }

    /// `prismarine_wall` — Prismarine Wall.
    pub fn prismarine_wall(&self) -> &Block {
        &self.palette[832]
    }

    /// `pumpkin` — Pumpkin.
    pub fn pumpkin(&self) -> &Block {
        &self.palette[833]
    }

    /// `pumpkin_stem` — Pumpkin Stem.
    pub fn pumpkin_stem(&self) -> &Block {
        &self.palette[834]
    }

    /// `purple_banner` — Purple Banner.
    pub fn purple_banner(&self) -> &Block {
        &self.palette[835]
    }

    /// `purple_bed` — Purple Bed.
    pub fn purple_bed(&self) -> &Block {
        &self.palette[836]
    }

    /// `purple_candle` — Purple Candle.
    pub fn purple_candle(&self) -> &Block {
        &self.palette[837]
    }

    /// `purple_candle_cake` — Cake with Purple Candle.
    pub fn purple_candle_cake(&self) -> &Block {
        &self.palette[838]
    }

    /// `purple_carpet` — Purple Carpet.
    pub fn purple_carpet(&self) -> &Block {
        &self.palette[839]
    }

    /// `purple_concrete` — Purple Concrete.
    pub fn purple_concrete(&self) -> &Block {
        &self.palette[840]
    }

    /// `purple_concrete_powder` — Purple Concrete Powder.
    pub fn purple_concrete_powder(&self) -> &Block {
        &self.palette[841]
    }

    /// `purple_glazed_terracotta` — Purple Glazed Terracotta.
    pub fn purple_glazed_terracotta(&self) -> &Block {
        &self.palette[842]
    }

    /// `purple_shulker_box` — Purple Shulker Box.
    pub fn purple_shulker_box(&self) -> &Block {
        &self.palette[843]
    }

    /// `purple_stained_glass` — Purple Stained Glass.
    pub fn purple_stained_glass(&self) -> &Block {
        &self.palette[844]
    }

    /// `purple_stained_glass_pane` — Purple Stained Glass Pane.
    pub fn purple_stained_glass_pane(&self) -> &Block {
        &self.palette[845]
    }

    /// `purple_terracotta` — Purple Terracotta.
    pub fn purple_terracotta(&self) -> &Block {
        &self.palette[846]
    }

    /// `purple_wall_banner` — Purple Banner.
    pub fn purple_wall_banner(&self) -> &Block {
        &self.palette[847]
    }

    /// `purple_wool` — Purple Wool.
    pub fn purple_wool(&self) -> &Block {
        &self.palette[848]
    }

    /// `purpur_block` — Purpur Block.
    pub fn purpur_block(&self) -> &Block {
        &self.palette[849]
    }

    /// `purpur_pillar` — Purpur Pillar.
    pub fn purpur_pillar(&self) -> &Block {
        &self.palette[850]
    }

    /// `purpur_slab` — Purpur Slab.
    pub fn purpur_slab(&self) -> &Block {
        &self.palette[851]
    }

    /// `purpur_stairs` — Purpur Stairs.
    pub fn purpur_stairs(&self) -> &Block {
        &self.palette[852]
    }

    /// `quartz_block` — Block of Quartz.
    pub fn quartz_block(&self) -> &Block {
        &self.palette[853]
    }

    /// `quartz_bricks` — Quartz Bricks.
    pub fn quartz_bricks(&self) -> &Block {
        &self.palette[854]
    }

    /// `quartz_pillar` — Quartz Pillar.
    pub fn quartz_pillar(&self) -> &Block {
        &self.palette[855]
    }

    /// `quartz_slab` — Quartz Slab.
    pub fn quartz_slab(&self) -> &Block {
        &self.palette[856]
    }

    /// `quartz_stairs` — Quartz Stairs.
    pub fn quartz_stairs(&self) -> &Block {
        &self.palette[857]
    }

    /// `rail` — Rail.
    pub fn rail(&self) -> &Block {
        &self.palette[858]
    }

    /// `raw_copper_block` — Block of Raw Copper.
    pub fn raw_copper_block(&self) -> &Block {
        &self.palette[859]
    }

    /// `raw_gold_block` — Block of Raw Gold.
    pub fn raw_gold_block(&self) -> &Block {
        &self.palette[860]
    }

    /// `raw_iron_block` — Block of Raw Iron.
    pub fn raw_iron_block(&self) -> &Block {
        &self.palette[861]
    }

    /// `red_banner` — Red Banner.
    pub fn red_banner(&self) -> &Block {
        &self.palette[862]
    }

    /// `red_bed` — Red Bed.
    pub fn red_bed(&self) -> &Block {
        &self.palette[863]
    }

    /// `red_candle` — Red Candle.
    pub fn red_candle(&self) -> &Block {
        &self.palette[864]
    }

    /// `red_candle_cake` — Cake with Red Candle.
    pub fn red_candle_cake(&self) -> &Block {
        &self.palette[865]
    }

    /// `red_carpet` — Red Carpet.
    pub fn red_carpet(&self) -> &Block {
        &self.palette[866]
    }

    /// `red_concrete` — Red Concrete.
    pub fn red_concrete(&self) -> &Block {
        &self.palette[867]
    }

    /// `red_concrete_powder` — Red Concrete Powder.
    pub fn red_concrete_powder(&self) -> &Block {
        &self.palette[868]
    }

    /// `red_glazed_terracotta` — Red Glazed Terracotta.
    pub fn red_glazed_terracotta(&self) -> &Block {
        &self.palette[869]
    }

    /// `red_mushroom` — Red Mushroom.
    pub fn red_mushroom(&self) -> &Block {
        &self.palette[870]
    }

    /// `red_mushroom_block` — Red Mushroom Block.
    pub fn red_mushroom_block(&self) -> &Block {
        &self.palette[871]
    }

    /// `red_nether_brick_slab` — Red Nether Brick Slab.
    pub fn red_nether_brick_slab(&self) -> &Block {
        &self.palette[872]
    }

    /// `red_nether_brick_stairs` — Red Nether Brick Stairs.
    pub fn red_nether_brick_stairs(&self) -> &Block {
        &self.palette[873]
    }

    /// `red_nether_brick_wall` — Red Nether Brick Wall.
    pub fn red_nether_brick_wall(&self) -> &Block {
        &self.palette[874]
    }

    /// `red_nether_bricks` — Red Nether Bricks.
    pub fn red_nether_bricks(&self) -> &Block {
        &self.palette[875]
    }

    /// `red_sand` — Red Sand.
    pub fn red_sand(&self) -> &Block {
        &self.palette[876]
    }

    /// `red_sandstone` — Red Sandstone.
    pub fn red_sandstone(&self) -> &Block {
        &self.palette[877]
    }

    /// `red_sandstone_slab` — Red Sandstone Slab.
    pub fn red_sandstone_slab(&self) -> &Block {
        &self.palette[878]
    }

    /// `red_sandstone_stairs` — Red Sandstone Stairs.
    pub fn red_sandstone_stairs(&self) -> &Block {
        &self.palette[879]
    }

    /// `red_sandstone_wall` — Red Sandstone Wall.
    pub fn red_sandstone_wall(&self) -> &Block {
        &self.palette[880]
    }

    /// `red_shulker_box` — Red Shulker Box.
    pub fn red_shulker_box(&self) -> &Block {
        &self.palette[881]
    }

    /// `red_stained_glass` — Red Stained Glass.
    pub fn red_stained_glass(&self) -> &Block {
        &self.palette[882]
    }

    /// `red_stained_glass_pane` — Red Stained Glass Pane.
    pub fn red_stained_glass_pane(&self) -> &Block {
        &self.palette[883]
    }

    /// `red_terracotta` — Red Terracotta.
    pub fn red_terracotta(&self) -> &Block {
        &self.palette[884]
    }

    /// `red_tulip` — Red Tulip.
    pub fn red_tulip(&self) -> &Block {
        &self.palette[885]
    }

    /// `red_wall_banner` — Red Banner.
    pub fn red_wall_banner(&self) -> &Block {
        &self.palette[886]
    }

    /// `red_wool` — Red Wool.
    pub fn red_wool(&self) -> &Block {
        &self.palette[887]
    }

    /// `redstone_block` — Block of Redstone.
    pub fn redstone_block(&self) -> &Block {
        &self.palette[888]
    }

    /// `redstone_lamp` — Redstone Lamp.
    pub fn redstone_lamp(&self) -> &Block {
        &self.palette[889]
    }

    /// `redstone_ore` — Redstone Ore.
    pub fn redstone_ore(&self) -> &Block {
        &self.palette[890]
    }

    /// `redstone_torch` — Redstone Torch.
    pub fn redstone_torch(&self) -> &Block {
        &self.palette[891]
    }

    /// `redstone_wall_torch` — Redstone Torch.
    pub fn redstone_wall_torch(&self) -> &Block {
        &self.palette[892]
    }

    /// `redstone_wire` — Redstone Wire.
    pub fn redstone_wire(&self) -> &Block {
        &self.palette[893]
    }

    /// `reinforced_deepslate` — Reinforced Deepslate.
    pub fn reinforced_deepslate(&self) -> &Block {
        &self.palette[894]
    }

    /// `repeater` — Redstone Repeater.
    pub fn repeater(&self) -> &Block {
        &self.palette[895]
    }

    /// `repeating_command_block` — Repeating Command Block.
    pub fn repeating_command_block(&self) -> &Block {
        &self.palette[896]
    }

    /// `resin_block` — Block of Resin.
    pub fn resin_block(&self) -> &Block {
        &self.palette[897]
    }

    /// `resin_brick_slab` — Resin Brick Slab.
    pub fn resin_brick_slab(&self) -> &Block {
        &self.palette[898]
    }

    /// `resin_brick_stairs` — Resin Brick Stairs.
    pub fn resin_brick_stairs(&self) -> &Block {
        &self.palette[899]
    }

    /// `resin_brick_wall` — Resin Brick Wall.
    pub fn resin_brick_wall(&self) -> &Block {
        &self.palette[900]
    }

    /// `resin_bricks` — Resin Bricks.
    pub fn resin_bricks(&self) -> &Block {
        &self.palette[901]
    }

    /// `resin_clump` — Resin Clump.
    pub fn resin_clump(&self) -> &Block {
        &self.palette[902]
    }

    /// `respawn_anchor` — Respawn Anchor.
    pub fn respawn_anchor(&self) -> &Block {
        &self.palette[903]
    }

    /// `rooted_dirt` — Rooted Dirt.
    pub fn rooted_dirt(&self) -> &Block {
        &self.palette[904]
    }

    /// `rose_bush` — Rose Bush.
    pub fn rose_bush(&self) -> &Block {
        &self.palette[905]
    }

    /// `sand` — Sand.
    pub fn sand(&self) -> &Block {
        &self.palette[906]
    }

    /// `sandstone` — Sandstone.
    pub fn sandstone(&self) -> &Block {
        &self.palette[907]
    }

    /// `sandstone_slab` — Sandstone Slab.
    pub fn sandstone_slab(&self) -> &Block {
        &self.palette[908]
    }

    /// `sandstone_stairs` — Sandstone Stairs.
    pub fn sandstone_stairs(&self) -> &Block {
        &self.palette[909]
    }

    /// `sandstone_wall` — Sandstone Wall.
    pub fn sandstone_wall(&self) -> &Block {
        &self.palette[910]
    }

    /// `scaffolding` — Scaffolding.
    pub fn scaffolding(&self) -> &Block {
        &self.palette[911]
    }

    /// `sculk` — Sculk.
    pub fn sculk(&self) -> &Block {
        &self.palette[912]
    }

    /// `sculk_catalyst` — Sculk Catalyst.
    pub fn sculk_catalyst(&self) -> &Block {
        &self.palette[913]
    }

    /// `sculk_sensor` — Sculk Sensor.
    pub fn sculk_sensor(&self) -> &Block {
        &self.palette[914]
    }

    /// `sculk_shrieker` — Sculk Shrieker.
    pub fn sculk_shrieker(&self) -> &Block {
        &self.palette[915]
    }

    /// `sculk_vein` — Sculk Vein.
    pub fn sculk_vein(&self) -> &Block {
        &self.palette[916]
    }

    /// `sea_lantern` — Sea Lantern.
    pub fn sea_lantern(&self) -> &Block {
        &self.palette[917]
    }

    /// `sea_pickle` — Sea Pickle.
    pub fn sea_pickle(&self) -> &Block {
        &self.palette[918]
    }

    /// `seagrass` — Seagrass.
    pub fn seagrass(&self) -> &Block {
        &self.palette[919]
    }

    /// `short_dry_grass` — Short Dry Grass.
    pub fn short_dry_grass(&self) -> &Block {
        &self.palette[920]
    }

    /// `short_grass` — Short Grass.
    pub fn short_grass(&self) -> &Block {
        &self.palette[921]
    }

    /// `shroomlight` — Shroomlight.
    pub fn shroomlight(&self) -> &Block {
        &self.palette[922]
    }

    /// `shulker_box` — Shulker Box.
    pub fn shulker_box(&self) -> &Block {
        &self.palette[923]
    }

    /// `skeleton_skull` — Skeleton Skull.
    pub fn skeleton_skull(&self) -> &Block {
        &self.palette[924]
    }

    /// `skeleton_wall_skull` — Skeleton Skull.
    pub fn skeleton_wall_skull(&self) -> &Block {
        &self.palette[925]
    }

    /// `slime_block` — Slime Block.
    pub fn slime_block(&self) -> &Block {
        &self.palette[926]
    }

    /// `small_amethyst_bud` — Small Amethyst Bud.
    pub fn small_amethyst_bud(&self) -> &Block {
        &self.palette[927]
    }

    /// `small_dripleaf` — Small Dripleaf.
    pub fn small_dripleaf(&self) -> &Block {
        &self.palette[928]
    }

    /// `smithing_table` — Smithing Table.
    pub fn smithing_table(&self) -> &Block {
        &self.palette[929]
    }

    /// `smoker` — Smoker.
    pub fn smoker(&self) -> &Block {
        &self.palette[930]
    }

    /// `smooth_basalt` — Smooth Basalt.
    pub fn smooth_basalt(&self) -> &Block {
        &self.palette[931]
    }

    /// `smooth_quartz` — Smooth Quartz Block.
    pub fn smooth_quartz(&self) -> &Block {
        &self.palette[932]
    }

    /// `smooth_quartz_slab` — Smooth Quartz Slab.
    pub fn smooth_quartz_slab(&self) -> &Block {
        &self.palette[933]
    }

    /// `smooth_quartz_stairs` — Smooth Quartz Stairs.
    pub fn smooth_quartz_stairs(&self) -> &Block {
        &self.palette[934]
    }

    /// `smooth_red_sandstone` — Smooth Red Sandstone.
    pub fn smooth_red_sandstone(&self) -> &Block {
        &self.palette[935]
    }

    /// `smooth_red_sandstone_slab` — Smooth Red Sandstone Slab.
    pub fn smooth_red_sandstone_slab(&self) -> &Block {
        &self.palette[936]
    }

    /// `smooth_red_sandstone_stairs` — Smooth Red Sandstone Stairs.
    pub fn smooth_red_sandstone_stairs(&self) -> &Block {
        &self.palette[937]
    }

    /// `smooth_sandstone` — Smooth Sandstone.
    pub fn smooth_sandstone(&self) -> &Block {
        &self.palette[938]
    }

    /// `smooth_sandstone_slab` — Smooth Sandstone Slab.
    pub fn smooth_sandstone_slab(&self) -> &Block {
        &self.palette[939]
    }

    /// `smooth_sandstone_stairs` — Smooth Sandstone Stairs.
    pub fn smooth_sandstone_stairs(&self) -> &Block {
        &self.palette[940]
    }

    /// `smooth_stone` — Smooth Stone.
    pub fn smooth_stone(&self) -> &Block {
        &self.palette[941]
    }

    /// `smooth_stone_slab` — Smooth Stone Slab.
    pub fn smooth_stone_slab(&self) -> &Block {
        &self.palette[942]
    }

    /// `sniffer_egg` — Sniffer Egg.
    pub fn sniffer_egg(&self) -> &Block {
        &self.palette[943]
    }

    /// `snow` — Snow.
    pub fn snow(&self) -> &Block {
        &self.palette[944]
    }

    /// `snow_block` — Snow Block.
    pub fn snow_block(&self) -> &Block {
        &self.palette[945]
    }

    /// `soul_campfire` — Soul Campfire.
    pub fn soul_campfire(&self) -> &Block {
        &self.palette[946]
    }

    /// `soul_fire` — Soul Fire.
    pub fn soul_fire(&self) -> &Block {
        &self.palette[947]
    }

    /// `soul_lantern` — Soul Lantern.
    pub fn soul_lantern(&self) -> &Block {
        &self.palette[948]
    }

    /// `soul_sand` — Soul Sand.
    pub fn soul_sand(&self) -> &Block {
        &self.palette[949]
    }

    /// `soul_soil` — Soul Soil.
    pub fn soul_soil(&self) -> &Block {
        &self.palette[950]
    }

    /// `soul_torch` — Soul Torch.
    pub fn soul_torch(&self) -> &Block {
        &self.palette[951]
    }

    /// `soul_wall_torch` — Soul Torch.
    pub fn soul_wall_torch(&self) -> &Block {
        &self.palette[952]
    }

    /// `spawner` — Monster Spawner.
    pub fn spawner(&self) -> &Block {
        &self.palette[953]
    }

    /// `sponge` — Sponge.
    pub fn sponge(&self) -> &Block {
        &self.palette[954]
    }

    /// `spore_blossom` — Spore Blossom.
    pub fn spore_blossom(&self) -> &Block {
        &self.palette[955]
    }

    /// `spruce_button` — Spruce Button.
    pub fn spruce_button(&self) -> &Block {
        &self.palette[956]
    }

    /// `spruce_door` — Spruce Door.
    pub fn spruce_door(&self) -> &Block {
        &self.palette[957]
    }

    /// `spruce_fence` — Spruce Fence.
    pub fn spruce_fence(&self) -> &Block {
        &self.palette[958]
    }

    /// `spruce_fence_gate` — Spruce Fence Gate.
    pub fn spruce_fence_gate(&self) -> &Block {
        &self.palette[959]
    }

    /// `spruce_hanging_sign` — Spruce Hanging Sign.
    pub fn spruce_hanging_sign(&self) -> &Block {
        &self.palette[960]
    }

    /// `spruce_leaves` — Spruce Leaves.
    pub fn spruce_leaves(&self) -> &Block {
        &self.palette[961]
    }

    /// `spruce_log` — Spruce Log.
    pub fn spruce_log(&self) -> &Block {
        &self.palette[962]
    }

    /// `spruce_planks` — Spruce Planks.
    pub fn spruce_planks(&self) -> &Block {
        &self.palette[963]
    }

    /// `spruce_pressure_plate` — Spruce Pressure Plate.
    pub fn spruce_pressure_plate(&self) -> &Block {
        &self.palette[964]
    }

    /// `spruce_sapling` — Spruce Sapling.
    pub fn spruce_sapling(&self) -> &Block {
        &self.palette[965]
    }

    /// `spruce_shelf` — Spruce Shelf.
    pub fn spruce_shelf(&self) -> &Block {
        &self.palette[966]
    }

    /// `spruce_sign` — Spruce Sign.
    pub fn spruce_sign(&self) -> &Block {
        &self.palette[967]
    }

    /// `spruce_slab` — Spruce Slab.
    pub fn spruce_slab(&self) -> &Block {
        &self.palette[968]
    }

    /// `spruce_stairs` — Spruce Stairs.
    pub fn spruce_stairs(&self) -> &Block {
        &self.palette[969]
    }

    /// `spruce_trapdoor` — Spruce Trapdoor.
    pub fn spruce_trapdoor(&self) -> &Block {
        &self.palette[970]
    }

    /// `spruce_wall_hanging_sign` — Spruce Hanging Sign.
    pub fn spruce_wall_hanging_sign(&self) -> &Block {
        &self.palette[971]
    }

    /// `spruce_wall_sign` — Spruce Sign.
    pub fn spruce_wall_sign(&self) -> &Block {
        &self.palette[972]
    }

    /// `spruce_wood` — Spruce Wood.
    pub fn spruce_wood(&self) -> &Block {
        &self.palette[973]
    }

    /// `sticky_piston` — Sticky Piston.
    pub fn sticky_piston(&self) -> &Block {
        &self.palette[974]
    }

    /// `stone` — Stone.
    pub fn stone(&self) -> &Block {
        &self.palette[975]
    }

    /// `stone_brick_slab` — Stone Brick Slab.
    pub fn stone_brick_slab(&self) -> &Block {
        &self.palette[976]
    }

    /// `stone_brick_stairs` — Stone Brick Stairs.
    pub fn stone_brick_stairs(&self) -> &Block {
        &self.palette[977]
    }

    /// `stone_brick_wall` — Stone Brick Wall.
    pub fn stone_brick_wall(&self) -> &Block {
        &self.palette[978]
    }

    /// `stone_bricks` — Stone Bricks.
    pub fn stone_bricks(&self) -> &Block {
        &self.palette[979]
    }

    /// `stone_button` — Stone Button.
    pub fn stone_button(&self) -> &Block {
        &self.palette[980]
    }

    /// `stone_pressure_plate` — Stone Pressure Plate.
    pub fn stone_pressure_plate(&self) -> &Block {
        &self.palette[981]
    }

    /// `stone_slab` — Stone Slab.
    pub fn stone_slab(&self) -> &Block {
        &self.palette[982]
    }

    /// `stone_stairs` — Stone Stairs.
    pub fn stone_stairs(&self) -> &Block {
        &self.palette[983]
    }

    /// `stonecutter` — Stonecutter.
    pub fn stonecutter(&self) -> &Block {
        &self.palette[984]
    }

    /// `stripped_acacia_log` — Stripped Acacia Log.
    pub fn stripped_acacia_log(&self) -> &Block {
        &self.palette[985]
    }

    /// `stripped_acacia_wood` — Stripped Acacia Wood.
    pub fn stripped_acacia_wood(&self) -> &Block {
        &self.palette[986]
    }

    /// `stripped_bamboo_block` — Block of Stripped Bamboo.
    pub fn stripped_bamboo_block(&self) -> &Block {
        &self.palette[987]
    }

    /// `stripped_birch_log` — Stripped Birch Log.
    pub fn stripped_birch_log(&self) -> &Block {
        &self.palette[988]
    }

    /// `stripped_birch_wood` — Stripped Birch Wood.
    pub fn stripped_birch_wood(&self) -> &Block {
        &self.palette[989]
    }

    /// `stripped_cherry_log` — Stripped Cherry Log.
    pub fn stripped_cherry_log(&self) -> &Block {
        &self.palette[990]
    }

    /// `stripped_cherry_wood` — Stripped Cherry Wood.
    pub fn stripped_cherry_wood(&self) -> &Block {
        &self.palette[991]
    }

    /// `stripped_crimson_hyphae` — Stripped Crimson Hyphae.
    pub fn stripped_crimson_hyphae(&self) -> &Block {
        &self.palette[992]
    }

    /// `stripped_crimson_stem` — Stripped Crimson Stem.
    pub fn stripped_crimson_stem(&self) -> &Block {
        &self.palette[993]
    }

    /// `stripped_dark_oak_log` — Stripped Dark Oak Log.
    pub fn stripped_dark_oak_log(&self) -> &Block {
        &self.palette[994]
    }

    /// `stripped_dark_oak_wood` — Stripped Dark Oak Wood.
    pub fn stripped_dark_oak_wood(&self) -> &Block {
        &self.palette[995]
    }

    /// `stripped_jungle_log` — Stripped Jungle Log.
    pub fn stripped_jungle_log(&self) -> &Block {
        &self.palette[996]
    }

    /// `stripped_jungle_wood` — Stripped Jungle Wood.
    pub fn stripped_jungle_wood(&self) -> &Block {
        &self.palette[997]
    }

    /// `stripped_mangrove_log` — Stripped Mangrove Log.
    pub fn stripped_mangrove_log(&self) -> &Block {
        &self.palette[998]
    }

    /// `stripped_mangrove_wood` — Stripped Mangrove Wood.
    pub fn stripped_mangrove_wood(&self) -> &Block {
        &self.palette[999]
    }

    /// `stripped_oak_log` — Stripped Oak Log.
    pub fn stripped_oak_log(&self) -> &Block {
        &self.palette[1000]
    }

    /// `stripped_oak_wood` — Stripped Oak Wood.
    pub fn stripped_oak_wood(&self) -> &Block {
        &self.palette[1001]
    }

    /// `stripped_pale_oak_log` — Stripped Pale Oak Log.
    pub fn stripped_pale_oak_log(&self) -> &Block {
        &self.palette[1002]
    }

    /// `stripped_pale_oak_wood` — Stripped Pale Oak Wood.
    pub fn stripped_pale_oak_wood(&self) -> &Block {
        &self.palette[1003]
    }

    /// `stripped_spruce_log` — Stripped Spruce Log.
    pub fn stripped_spruce_log(&self) -> &Block {
        &self.palette[1004]
    }

    /// `stripped_spruce_wood` — Stripped Spruce Wood.
    pub fn stripped_spruce_wood(&self) -> &Block {
        &self.palette[1005]
    }

    /// `stripped_warped_hyphae` — Stripped Warped Hyphae.
    pub fn stripped_warped_hyphae(&self) -> &Block {
        &self.palette[1006]
    }

    /// `stripped_warped_stem` — Stripped Warped Stem.
    pub fn stripped_warped_stem(&self) -> &Block {
        &self.palette[1007]
    }

    /// `structure_block` — Structure Block.
    pub fn structure_block(&self) -> &Block {
        &self.palette[1008]
    }

    /// `structure_void` — Structure Void.
    pub fn structure_void(&self) -> &Block {
        &self.palette[1009]
    }

    /// `sugar_cane` — Sugar Cane.
    pub fn sugar_cane(&self) -> &Block {
        &self.palette[1010]
    }

    /// `sulfur` — Sulfur.
    pub fn sulfur(&self) -> &Block {
        &self.palette[1011]
    }

    /// `sulfur_brick_slab` — Sulfur Brick Slab.
    pub fn sulfur_brick_slab(&self) -> &Block {
        &self.palette[1012]
    }

    /// `sulfur_brick_stairs` — Sulfur Brick Stairs.
    pub fn sulfur_brick_stairs(&self) -> &Block {
        &self.palette[1013]
    }

    /// `sulfur_brick_wall` — Sulfur Brick Wall.
    pub fn sulfur_brick_wall(&self) -> &Block {
        &self.palette[1014]
    }

    /// `sulfur_bricks` — Sulfur Bricks.
    pub fn sulfur_bricks(&self) -> &Block {
        &self.palette[1015]
    }

    /// `sulfur_slab` — Sulfur Slab.
    pub fn sulfur_slab(&self) -> &Block {
        &self.palette[1016]
    }

    /// `sulfur_spike` — Sulfur Spike.
    pub fn sulfur_spike(&self) -> &Block {
        &self.palette[1017]
    }

    /// `sulfur_stairs` — Sulfur Stairs.
    pub fn sulfur_stairs(&self) -> &Block {
        &self.palette[1018]
    }

    /// `sulfur_wall` — Sulfur Wall.
    pub fn sulfur_wall(&self) -> &Block {
        &self.palette[1019]
    }

    /// `sunflower` — Sunflower.
    pub fn sunflower(&self) -> &Block {
        &self.palette[1020]
    }

    /// `suspicious_gravel` — Suspicious Gravel.
    pub fn suspicious_gravel(&self) -> &Block {
        &self.palette[1021]
    }

    /// `suspicious_sand` — Suspicious Sand.
    pub fn suspicious_sand(&self) -> &Block {
        &self.palette[1022]
    }

    /// `sweet_berry_bush` — Sweet Berry Bush.
    pub fn sweet_berry_bush(&self) -> &Block {
        &self.palette[1023]
    }

    /// `tall_dry_grass` — Tall Dry Grass.
    pub fn tall_dry_grass(&self) -> &Block {
        &self.palette[1024]
    }

    /// `tall_grass` — Tall Grass.
    pub fn tall_grass(&self) -> &Block {
        &self.palette[1025]
    }

    /// `tall_seagrass` — Tall Seagrass.
    pub fn tall_seagrass(&self) -> &Block {
        &self.palette[1026]
    }

    /// `target` — Target.
    pub fn target(&self) -> &Block {
        &self.palette[1027]
    }

    /// `terracotta` — Terracotta.
    pub fn terracotta(&self) -> &Block {
        &self.palette[1028]
    }

    /// `test_block` — Test Block.
    pub fn test_block(&self) -> &Block {
        &self.palette[1029]
    }

    /// `test_instance_block` — Test Instance Block.
    pub fn test_instance_block(&self) -> &Block {
        &self.palette[1030]
    }

    /// `tinted_glass` — Tinted Glass.
    pub fn tinted_glass(&self) -> &Block {
        &self.palette[1031]
    }

    /// `tnt` — TNT.
    pub fn tnt(&self) -> &Block {
        &self.palette[1032]
    }

    /// `torch` — Torch.
    pub fn torch(&self) -> &Block {
        &self.palette[1033]
    }

    /// `torchflower` — Torchflower.
    pub fn torchflower(&self) -> &Block {
        &self.palette[1034]
    }

    /// `torchflower_crop` — Torchflower Crop.
    pub fn torchflower_crop(&self) -> &Block {
        &self.palette[1035]
    }

    /// `trapped_chest` — Trapped Chest.
    pub fn trapped_chest(&self) -> &Block {
        &self.palette[1036]
    }

    /// `trial_spawner` — Trial Spawner.
    pub fn trial_spawner(&self) -> &Block {
        &self.palette[1037]
    }

    /// `tripwire` — Tripwire.
    pub fn tripwire(&self) -> &Block {
        &self.palette[1038]
    }

    /// `tripwire_hook` — Tripwire Hook.
    pub fn tripwire_hook(&self) -> &Block {
        &self.palette[1039]
    }

    /// `tube_coral` — Tube Coral.
    pub fn tube_coral(&self) -> &Block {
        &self.palette[1040]
    }

    /// `tube_coral_block` — Tube Coral Block.
    pub fn tube_coral_block(&self) -> &Block {
        &self.palette[1041]
    }

    /// `tube_coral_fan` — Tube Coral Fan.
    pub fn tube_coral_fan(&self) -> &Block {
        &self.palette[1042]
    }

    /// `tube_coral_wall_fan` — Tube Coral Wall Fan.
    pub fn tube_coral_wall_fan(&self) -> &Block {
        &self.palette[1043]
    }

    /// `tuff` — Tuff.
    pub fn tuff(&self) -> &Block {
        &self.palette[1044]
    }

    /// `tuff_brick_slab` — Tuff Brick Slab.
    pub fn tuff_brick_slab(&self) -> &Block {
        &self.palette[1045]
    }

    /// `tuff_brick_stairs` — Tuff Brick Stairs.
    pub fn tuff_brick_stairs(&self) -> &Block {
        &self.palette[1046]
    }

    /// `tuff_brick_wall` — Tuff Brick Wall.
    pub fn tuff_brick_wall(&self) -> &Block {
        &self.palette[1047]
    }

    /// `tuff_bricks` — Tuff Bricks.
    pub fn tuff_bricks(&self) -> &Block {
        &self.palette[1048]
    }

    /// `tuff_slab` — Tuff Slab.
    pub fn tuff_slab(&self) -> &Block {
        &self.palette[1049]
    }

    /// `tuff_stairs` — Tuff Stairs.
    pub fn tuff_stairs(&self) -> &Block {
        &self.palette[1050]
    }

    /// `tuff_wall` — Tuff Wall.
    pub fn tuff_wall(&self) -> &Block {
        &self.palette[1051]
    }

    /// `turtle_egg` — Turtle Egg.
    pub fn turtle_egg(&self) -> &Block {
        &self.palette[1052]
    }

    /// `twisting_vines` — Twisting Vines.
    pub fn twisting_vines(&self) -> &Block {
        &self.palette[1053]
    }

    /// `twisting_vines_plant` — Twisting Vines Plant.
    pub fn twisting_vines_plant(&self) -> &Block {
        &self.palette[1054]
    }

    /// `vault` — Vault.
    pub fn vault(&self) -> &Block {
        &self.palette[1055]
    }

    /// `verdant_froglight` — Verdant Froglight.
    pub fn verdant_froglight(&self) -> &Block {
        &self.palette[1056]
    }

    /// `vine` — Vines.
    pub fn vine(&self) -> &Block {
        &self.palette[1057]
    }

    /// `void_air` — Void Air.
    pub fn void_air(&self) -> &Block {
        &self.palette[1058]
    }

    /// `wall_torch` — Torch.
    pub fn wall_torch(&self) -> &Block {
        &self.palette[1059]
    }

    /// `warped_button` — Warped Button.
    pub fn warped_button(&self) -> &Block {
        &self.palette[1060]
    }

    /// `warped_door` — Warped Door.
    pub fn warped_door(&self) -> &Block {
        &self.palette[1061]
    }

    /// `warped_fence` — Warped Fence.
    pub fn warped_fence(&self) -> &Block {
        &self.palette[1062]
    }

    /// `warped_fence_gate` — Warped Fence Gate.
    pub fn warped_fence_gate(&self) -> &Block {
        &self.palette[1063]
    }

    /// `warped_fungus` — Warped Fungus.
    pub fn warped_fungus(&self) -> &Block {
        &self.palette[1064]
    }

    /// `warped_hanging_sign` — Warped Hanging Sign.
    pub fn warped_hanging_sign(&self) -> &Block {
        &self.palette[1065]
    }

    /// `warped_hyphae` — Warped Hyphae.
    pub fn warped_hyphae(&self) -> &Block {
        &self.palette[1066]
    }

    /// `warped_nylium` — Warped Nylium.
    pub fn warped_nylium(&self) -> &Block {
        &self.palette[1067]
    }

    /// `warped_planks` — Warped Planks.
    pub fn warped_planks(&self) -> &Block {
        &self.palette[1068]
    }

    /// `warped_pressure_plate` — Warped Pressure Plate.
    pub fn warped_pressure_plate(&self) -> &Block {
        &self.palette[1069]
    }

    /// `warped_roots` — Warped Roots.
    pub fn warped_roots(&self) -> &Block {
        &self.palette[1070]
    }

    /// `warped_shelf` — Warped Shelf.
    pub fn warped_shelf(&self) -> &Block {
        &self.palette[1071]
    }

    /// `warped_sign` — Warped Sign.
    pub fn warped_sign(&self) -> &Block {
        &self.palette[1072]
    }

    /// `warped_slab` — Warped Slab.
    pub fn warped_slab(&self) -> &Block {
        &self.palette[1073]
    }

    /// `warped_stairs` — Warped Stairs.
    pub fn warped_stairs(&self) -> &Block {
        &self.palette[1074]
    }

    /// `warped_stem` — Warped Stem.
    pub fn warped_stem(&self) -> &Block {
        &self.palette[1075]
    }

    /// `warped_trapdoor` — Warped Trapdoor.
    pub fn warped_trapdoor(&self) -> &Block {
        &self.palette[1076]
    }

    /// `warped_wall_hanging_sign` — Warped Hanging Sign.
    pub fn warped_wall_hanging_sign(&self) -> &Block {
        &self.palette[1077]
    }

    /// `warped_wall_sign` — Warped Sign.
    pub fn warped_wall_sign(&self) -> &Block {
        &self.palette[1078]
    }

    /// `warped_wart_block` — Warped Wart Block.
    pub fn warped_wart_block(&self) -> &Block {
        &self.palette[1079]
    }

    /// `water` — Water.
    pub fn water(&self) -> &Block {
        &self.palette[1080]
    }

    /// `water_cauldron` — Water Cauldron.
    pub fn water_cauldron(&self) -> &Block {
        &self.palette[1081]
    }

    /// `waxed_chiseled_copper` — Waxed Chiseled Copper.
    pub fn waxed_chiseled_copper(&self) -> &Block {
        &self.palette[1082]
    }

    /// `waxed_copper_bars` — Waxed Copper Bars.
    pub fn waxed_copper_bars(&self) -> &Block {
        &self.palette[1083]
    }

    /// `waxed_copper_block` — Waxed Block of Copper.
    pub fn waxed_copper_block(&self) -> &Block {
        &self.palette[1084]
    }

    /// `waxed_copper_bulb` — Waxed Copper Bulb.
    pub fn waxed_copper_bulb(&self) -> &Block {
        &self.palette[1085]
    }

    /// `waxed_copper_chain` — Waxed Copper Chain.
    pub fn waxed_copper_chain(&self) -> &Block {
        &self.palette[1086]
    }

    /// `waxed_copper_chest` — Waxed Copper Chest.
    pub fn waxed_copper_chest(&self) -> &Block {
        &self.palette[1087]
    }

    /// `waxed_copper_door` — Waxed Copper Door.
    pub fn waxed_copper_door(&self) -> &Block {
        &self.palette[1088]
    }

    /// `waxed_copper_golem_statue` — Waxed Copper Golem Statue.
    pub fn waxed_copper_golem_statue(&self) -> &Block {
        &self.palette[1089]
    }

    /// `waxed_copper_grate` — Waxed Copper Grate.
    pub fn waxed_copper_grate(&self) -> &Block {
        &self.palette[1090]
    }

    /// `waxed_copper_lantern` — Waxed Copper Lantern.
    pub fn waxed_copper_lantern(&self) -> &Block {
        &self.palette[1091]
    }

    /// `waxed_copper_trapdoor` — Waxed Copper Trapdoor.
    pub fn waxed_copper_trapdoor(&self) -> &Block {
        &self.palette[1092]
    }

    /// `waxed_cut_copper` — Waxed Cut Copper.
    pub fn waxed_cut_copper(&self) -> &Block {
        &self.palette[1093]
    }

    /// `waxed_cut_copper_slab` — Waxed Cut Copper Slab.
    pub fn waxed_cut_copper_slab(&self) -> &Block {
        &self.palette[1094]
    }

    /// `waxed_cut_copper_stairs` — Waxed Cut Copper Stairs.
    pub fn waxed_cut_copper_stairs(&self) -> &Block {
        &self.palette[1095]
    }

    /// `waxed_exposed_chiseled_copper` — Waxed Exposed Chiseled Copper.
    pub fn waxed_exposed_chiseled_copper(&self) -> &Block {
        &self.palette[1096]
    }

    /// `waxed_exposed_copper` — Waxed Exposed Copper.
    pub fn waxed_exposed_copper(&self) -> &Block {
        &self.palette[1097]
    }

    /// `waxed_exposed_copper_bars` — Waxed Exposed Copper Bars.
    pub fn waxed_exposed_copper_bars(&self) -> &Block {
        &self.palette[1098]
    }

    /// `waxed_exposed_copper_bulb` — Waxed Exposed Copper Bulb.
    pub fn waxed_exposed_copper_bulb(&self) -> &Block {
        &self.palette[1099]
    }

    /// `waxed_exposed_copper_chain` — Waxed Exposed Copper Chain.
    pub fn waxed_exposed_copper_chain(&self) -> &Block {
        &self.palette[1100]
    }

    /// `waxed_exposed_copper_chest` — Waxed Exposed Copper Chest.
    pub fn waxed_exposed_copper_chest(&self) -> &Block {
        &self.palette[1101]
    }

    /// `waxed_exposed_copper_door` — Waxed Exposed Copper Door.
    pub fn waxed_exposed_copper_door(&self) -> &Block {
        &self.palette[1102]
    }

    /// `waxed_exposed_copper_golem_statue` — Waxed Exposed Copper Golem Statue.
    pub fn waxed_exposed_copper_golem_statue(&self) -> &Block {
        &self.palette[1103]
    }

    /// `waxed_exposed_copper_grate` — Waxed Exposed Copper Grate.
    pub fn waxed_exposed_copper_grate(&self) -> &Block {
        &self.palette[1104]
    }

    /// `waxed_exposed_copper_lantern` — Waxed Exposed Copper Lantern.
    pub fn waxed_exposed_copper_lantern(&self) -> &Block {
        &self.palette[1105]
    }

    /// `waxed_exposed_copper_trapdoor` — Waxed Exposed Copper Trapdoor.
    pub fn waxed_exposed_copper_trapdoor(&self) -> &Block {
        &self.palette[1106]
    }

    /// `waxed_exposed_cut_copper` — Waxed Exposed Cut Copper.
    pub fn waxed_exposed_cut_copper(&self) -> &Block {
        &self.palette[1107]
    }

    /// `waxed_exposed_cut_copper_slab` — Waxed Exposed Cut Copper Slab.
    pub fn waxed_exposed_cut_copper_slab(&self) -> &Block {
        &self.palette[1108]
    }

    /// `waxed_exposed_cut_copper_stairs` — Waxed Exposed Cut Copper Stairs.
    pub fn waxed_exposed_cut_copper_stairs(&self) -> &Block {
        &self.palette[1109]
    }

    /// `waxed_exposed_lightning_rod` — Waxed Exposed Lightning Rod.
    pub fn waxed_exposed_lightning_rod(&self) -> &Block {
        &self.palette[1110]
    }

    /// `waxed_lightning_rod` — Waxed Lightning Rod.
    pub fn waxed_lightning_rod(&self) -> &Block {
        &self.palette[1111]
    }

    /// `waxed_oxidized_chiseled_copper` — Waxed Oxidized Chiseled Copper.
    pub fn waxed_oxidized_chiseled_copper(&self) -> &Block {
        &self.palette[1112]
    }

    /// `waxed_oxidized_copper` — Waxed Oxidized Copper.
    pub fn waxed_oxidized_copper(&self) -> &Block {
        &self.palette[1113]
    }

    /// `waxed_oxidized_copper_bars` — Waxed Oxidized Copper Bars.
    pub fn waxed_oxidized_copper_bars(&self) -> &Block {
        &self.palette[1114]
    }

    /// `waxed_oxidized_copper_bulb` — Waxed Oxidized Copper Bulb.
    pub fn waxed_oxidized_copper_bulb(&self) -> &Block {
        &self.palette[1115]
    }

    /// `waxed_oxidized_copper_chain` — Waxed Oxidized Copper Chain.
    pub fn waxed_oxidized_copper_chain(&self) -> &Block {
        &self.palette[1116]
    }

    /// `waxed_oxidized_copper_chest` — Waxed Oxidized Copper Chest.
    pub fn waxed_oxidized_copper_chest(&self) -> &Block {
        &self.palette[1117]
    }

    /// `waxed_oxidized_copper_door` — Waxed Oxidized Copper Door.
    pub fn waxed_oxidized_copper_door(&self) -> &Block {
        &self.palette[1118]
    }

    /// `waxed_oxidized_copper_golem_statue` — Waxed Oxidized Copper Golem Statue.
    pub fn waxed_oxidized_copper_golem_statue(&self) -> &Block {
        &self.palette[1119]
    }

    /// `waxed_oxidized_copper_grate` — Waxed Oxidized Copper Grate.
    pub fn waxed_oxidized_copper_grate(&self) -> &Block {
        &self.palette[1120]
    }

    /// `waxed_oxidized_copper_lantern` — Waxed Oxidized Copper Lantern.
    pub fn waxed_oxidized_copper_lantern(&self) -> &Block {
        &self.palette[1121]
    }

    /// `waxed_oxidized_copper_trapdoor` — Waxed Oxidized Copper Trapdoor.
    pub fn waxed_oxidized_copper_trapdoor(&self) -> &Block {
        &self.palette[1122]
    }

    /// `waxed_oxidized_cut_copper` — Waxed Oxidized Cut Copper.
    pub fn waxed_oxidized_cut_copper(&self) -> &Block {
        &self.palette[1123]
    }

    /// `waxed_oxidized_cut_copper_slab` — Waxed Oxidized Cut Copper Slab.
    pub fn waxed_oxidized_cut_copper_slab(&self) -> &Block {
        &self.palette[1124]
    }

    /// `waxed_oxidized_cut_copper_stairs` — Waxed Oxidized Cut Copper Stairs.
    pub fn waxed_oxidized_cut_copper_stairs(&self) -> &Block {
        &self.palette[1125]
    }

    /// `waxed_oxidized_lightning_rod` — Waxed Oxidized Lightning Rod.
    pub fn waxed_oxidized_lightning_rod(&self) -> &Block {
        &self.palette[1126]
    }

    /// `waxed_weathered_chiseled_copper` — Waxed Weathered Chiseled Copper.
    pub fn waxed_weathered_chiseled_copper(&self) -> &Block {
        &self.palette[1127]
    }

    /// `waxed_weathered_copper` — Waxed Weathered Copper.
    pub fn waxed_weathered_copper(&self) -> &Block {
        &self.palette[1128]
    }

    /// `waxed_weathered_copper_bars` — Waxed Weathered Copper Bars.
    pub fn waxed_weathered_copper_bars(&self) -> &Block {
        &self.palette[1129]
    }

    /// `waxed_weathered_copper_bulb` — Waxed Weathered Copper Bulb.
    pub fn waxed_weathered_copper_bulb(&self) -> &Block {
        &self.palette[1130]
    }

    /// `waxed_weathered_copper_chain` — Waxed Weathered Copper Chain.
    pub fn waxed_weathered_copper_chain(&self) -> &Block {
        &self.palette[1131]
    }

    /// `waxed_weathered_copper_chest` — Waxed Weathered Copper Chest.
    pub fn waxed_weathered_copper_chest(&self) -> &Block {
        &self.palette[1132]
    }

    /// `waxed_weathered_copper_door` — Waxed Weathered Copper Door.
    pub fn waxed_weathered_copper_door(&self) -> &Block {
        &self.palette[1133]
    }

    /// `waxed_weathered_copper_golem_statue` — Waxed Weathered Copper Golem Statue.
    pub fn waxed_weathered_copper_golem_statue(&self) -> &Block {
        &self.palette[1134]
    }

    /// `waxed_weathered_copper_grate` — Waxed Weathered Copper Grate.
    pub fn waxed_weathered_copper_grate(&self) -> &Block {
        &self.palette[1135]
    }

    /// `waxed_weathered_copper_lantern` — Waxed Weathered Copper Lantern.
    pub fn waxed_weathered_copper_lantern(&self) -> &Block {
        &self.palette[1136]
    }

    /// `waxed_weathered_copper_trapdoor` — Waxed Weathered Copper Trapdoor.
    pub fn waxed_weathered_copper_trapdoor(&self) -> &Block {
        &self.palette[1137]
    }

    /// `waxed_weathered_cut_copper` — Waxed Weathered Cut Copper.
    pub fn waxed_weathered_cut_copper(&self) -> &Block {
        &self.palette[1138]
    }

    /// `waxed_weathered_cut_copper_slab` — Waxed Weathered Cut Copper Slab.
    pub fn waxed_weathered_cut_copper_slab(&self) -> &Block {
        &self.palette[1139]
    }

    /// `waxed_weathered_cut_copper_stairs` — Waxed Weathered Cut Copper Stairs.
    pub fn waxed_weathered_cut_copper_stairs(&self) -> &Block {
        &self.palette[1140]
    }

    /// `waxed_weathered_lightning_rod` — Waxed Weathered Lightning Rod.
    pub fn waxed_weathered_lightning_rod(&self) -> &Block {
        &self.palette[1141]
    }

    /// `weathered_chiseled_copper` — Weathered Chiseled Copper.
    pub fn weathered_chiseled_copper(&self) -> &Block {
        &self.palette[1142]
    }

    /// `weathered_copper` — Weathered Copper.
    pub fn weathered_copper(&self) -> &Block {
        &self.palette[1143]
    }

    /// `weathered_copper_bars` — Weathered Copper Bars.
    pub fn weathered_copper_bars(&self) -> &Block {
        &self.palette[1144]
    }

    /// `weathered_copper_bulb` — Weathered Copper Bulb.
    pub fn weathered_copper_bulb(&self) -> &Block {
        &self.palette[1145]
    }

    /// `weathered_copper_chain` — Weathered Copper Chain.
    pub fn weathered_copper_chain(&self) -> &Block {
        &self.palette[1146]
    }

    /// `weathered_copper_chest` — Weathered Copper Chest.
    pub fn weathered_copper_chest(&self) -> &Block {
        &self.palette[1147]
    }

    /// `weathered_copper_door` — Weathered Copper Door.
    pub fn weathered_copper_door(&self) -> &Block {
        &self.palette[1148]
    }

    /// `weathered_copper_golem_statue` — Weathered Copper Golem Statue.
    pub fn weathered_copper_golem_statue(&self) -> &Block {
        &self.palette[1149]
    }

    /// `weathered_copper_grate` — Weathered Copper Grate.
    pub fn weathered_copper_grate(&self) -> &Block {
        &self.palette[1150]
    }

    /// `weathered_copper_lantern` — Weathered Copper Lantern.
    pub fn weathered_copper_lantern(&self) -> &Block {
        &self.palette[1151]
    }

    /// `weathered_copper_trapdoor` — Weathered Copper Trapdoor.
    pub fn weathered_copper_trapdoor(&self) -> &Block {
        &self.palette[1152]
    }

    /// `weathered_cut_copper` — Weathered Cut Copper.
    pub fn weathered_cut_copper(&self) -> &Block {
        &self.palette[1153]
    }

    /// `weathered_cut_copper_slab` — Weathered Cut Copper Slab.
    pub fn weathered_cut_copper_slab(&self) -> &Block {
        &self.palette[1154]
    }

    /// `weathered_cut_copper_stairs` — Weathered Cut Copper Stairs.
    pub fn weathered_cut_copper_stairs(&self) -> &Block {
        &self.palette[1155]
    }

    /// `weathered_lightning_rod` — Weathered Lightning Rod.
    pub fn weathered_lightning_rod(&self) -> &Block {
        &self.palette[1156]
    }

    /// `weeping_vines` — Weeping Vines.
    pub fn weeping_vines(&self) -> &Block {
        &self.palette[1157]
    }

    /// `weeping_vines_plant` — Weeping Vines Plant.
    pub fn weeping_vines_plant(&self) -> &Block {
        &self.palette[1158]
    }

    /// `wet_sponge` — Wet Sponge.
    pub fn wet_sponge(&self) -> &Block {
        &self.palette[1159]
    }

    /// `wheat` — Wheat Crops.
    pub fn wheat(&self) -> &Block {
        &self.palette[1160]
    }

    /// `white_banner` — White Banner.
    pub fn white_banner(&self) -> &Block {
        &self.palette[1161]
    }

    /// `white_bed` — White Bed.
    pub fn white_bed(&self) -> &Block {
        &self.palette[1162]
    }

    /// `white_candle` — White Candle.
    pub fn white_candle(&self) -> &Block {
        &self.palette[1163]
    }

    /// `white_candle_cake` — Cake with White Candle.
    pub fn white_candle_cake(&self) -> &Block {
        &self.palette[1164]
    }

    /// `white_carpet` — White Carpet.
    pub fn white_carpet(&self) -> &Block {
        &self.palette[1165]
    }

    /// `white_concrete` — White Concrete.
    pub fn white_concrete(&self) -> &Block {
        &self.palette[1166]
    }

    /// `white_concrete_powder` — White Concrete Powder.
    pub fn white_concrete_powder(&self) -> &Block {
        &self.palette[1167]
    }

    /// `white_glazed_terracotta` — White Glazed Terracotta.
    pub fn white_glazed_terracotta(&self) -> &Block {
        &self.palette[1168]
    }

    /// `white_shulker_box` — White Shulker Box.
    pub fn white_shulker_box(&self) -> &Block {
        &self.palette[1169]
    }

    /// `white_stained_glass` — White Stained Glass.
    pub fn white_stained_glass(&self) -> &Block {
        &self.palette[1170]
    }

    /// `white_stained_glass_pane` — White Stained Glass Pane.
    pub fn white_stained_glass_pane(&self) -> &Block {
        &self.palette[1171]
    }

    /// `white_terracotta` — White Terracotta.
    pub fn white_terracotta(&self) -> &Block {
        &self.palette[1172]
    }

    /// `white_tulip` — White Tulip.
    pub fn white_tulip(&self) -> &Block {
        &self.palette[1173]
    }

    /// `white_wall_banner` — White Banner.
    pub fn white_wall_banner(&self) -> &Block {
        &self.palette[1174]
    }

    /// `white_wool` — White Wool.
    pub fn white_wool(&self) -> &Block {
        &self.palette[1175]
    }

    /// `wildflowers` — Wildflowers.
    pub fn wildflowers(&self) -> &Block {
        &self.palette[1176]
    }

    /// `wither_rose` — Wither Rose.
    pub fn wither_rose(&self) -> &Block {
        &self.palette[1177]
    }

    /// `wither_skeleton_skull` — Wither Skeleton Skull.
    pub fn wither_skeleton_skull(&self) -> &Block {
        &self.palette[1178]
    }

    /// `wither_skeleton_wall_skull` — Wither Skeleton Skull.
    pub fn wither_skeleton_wall_skull(&self) -> &Block {
        &self.palette[1179]
    }

    /// `yellow_banner` — Yellow Banner.
    pub fn yellow_banner(&self) -> &Block {
        &self.palette[1180]
    }

    /// `yellow_bed` — Yellow Bed.
    pub fn yellow_bed(&self) -> &Block {
        &self.palette[1181]
    }

    /// `yellow_candle` — Yellow Candle.
    pub fn yellow_candle(&self) -> &Block {
        &self.palette[1182]
    }

    /// `yellow_candle_cake` — Cake with Yellow Candle.
    pub fn yellow_candle_cake(&self) -> &Block {
        &self.palette[1183]
    }

    /// `yellow_carpet` — Yellow Carpet.
    pub fn yellow_carpet(&self) -> &Block {
        &self.palette[1184]
    }

    /// `yellow_concrete` — Yellow Concrete.
    pub fn yellow_concrete(&self) -> &Block {
        &self.palette[1185]
    }

    /// `yellow_concrete_powder` — Yellow Concrete Powder.
    pub fn yellow_concrete_powder(&self) -> &Block {
        &self.palette[1186]
    }

    /// `yellow_glazed_terracotta` — Yellow Glazed Terracotta.
    pub fn yellow_glazed_terracotta(&self) -> &Block {
        &self.palette[1187]
    }

    /// `yellow_shulker_box` — Yellow Shulker Box.
    pub fn yellow_shulker_box(&self) -> &Block {
        &self.palette[1188]
    }

    /// `yellow_stained_glass` — Yellow Stained Glass.
    pub fn yellow_stained_glass(&self) -> &Block {
        &self.palette[1189]
    }

    /// `yellow_stained_glass_pane` — Yellow Stained Glass Pane.
    pub fn yellow_stained_glass_pane(&self) -> &Block {
        &self.palette[1190]
    }

    /// `yellow_terracotta` — Yellow Terracotta.
    pub fn yellow_terracotta(&self) -> &Block {
        &self.palette[1191]
    }

    /// `yellow_wall_banner` — Yellow Banner.
    pub fn yellow_wall_banner(&self) -> &Block {
        &self.palette[1192]
    }

    /// `yellow_wool` — Yellow Wool.
    pub fn yellow_wool(&self) -> &Block {
        &self.palette[1193]
    }

    /// `zombie_head` — Zombie Head.
    pub fn zombie_head(&self) -> &Block {
        &self.palette[1194]
    }

    /// `zombie_wall_head` — Zombie Head.
    pub fn zombie_wall_head(&self) -> &Block {
        &self.palette[1195]
    }
}

impl Default for AllBlocks {
    fn default() -> Self {
        Self::new()
    }
}

fn main() {
    let all = AllBlocks::new();

    println!(
        "Minecraft Java Edition 26.2: {} blocks",
        all.palette().len()
    );
    assert_eq!(all.palette().len(), AllBlocks::LEN);

    // Spot checks for the mapped capabilities.
    assert!(all.air().empty().is_some());
    assert!(all.slime_block().sticky().is_some());
    assert!(all.honey_block().sticky().is_some());
    assert!(all.bedrock().unmovable().is_some());
    assert!(all.water().unmovable().is_some());
    assert!(all.stone().movable().is_some());
    assert!(all.stone().breakable().is_some());
    assert!(all.torch().instant_minable().is_some());
    assert!(all.tnt().flammable().is_some());
    assert!(all.redstone_block().caps().contains(CapSet::POWER));

    // Place a couple of them in a world.
    let mut world: World<'_, 1, 1196> = World::new(all.palette());
    world.set_block(all.white_stained_glass(), (0, 0, 0));
    world.set_block(all.green_stained_glass(), (1, 0, 0));
    assert_eq!(
        world.get(Pos::new(0, 0, 0)).unwrap(),
        *all.white_stained_glass()
    );

    println!("white_stained_glass: {:?}", all.white_stained_glass());
    println!("green_stained_glass: {:?}", all.green_stained_glass());
}
