//! A capability-driven piston and its power source.
//!
//! [`PowerBlock`] carries the [`Power`](const@Power) capability. [`PistonBlock`] watches
//! for that capability on its six neighbours:
//!
//! 1. An update (a neighbour change or its own placement) reaches the piston.
//! 2. If it is powered and idle, it flips to [`PistonPhase::Extending`],
//!    stores [`PISTON_EXTEND_TICKS`] in its state and schedules itself.
//! 3. Every scheduled tick decrements the timer; on the sixth tick the piston
//!    scans the line of movable blocks in front of it and moves the line one
//!    step.
//! 4. Losing power cancels a running extension or resets an extended piston.
//!
//! There is no block entity in this library yet, so the move is applied in a
//! single step when the timer expires instead of animating a moving block.

use crate::caps::{CapSet, Power, UnMovable};
use crate::def::StateCodec;
use crate::pos::{Dir, Pos};
use crate::world::TickCtx;

/// Number of scheduled ticks between receiving power and completing the move.
pub const PISTON_EXTEND_TICKS: u8 = 6;

/// Maximum number of blocks a piston moves in one stroke.
///
/// Longer lines make the piston give up and stay retracted.
pub const PISTON_PUSH_LIMIT: usize = 16;

/// Where a [`PistonBlock`] points.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Facing {
    /// Positive `x`.
    #[default]
    PosX,
    /// Negative `x`.
    NegX,
    /// Positive `y`.
    PosY,
    /// Negative `y`.
    NegY,
    /// Positive `z`.
    PosZ,
    /// Negative `z`.
    NegZ,
}

impl Facing {
    /// All six facings.
    pub const ALL: [Self; 6] = [
        Self::PosX,
        Self::NegX,
        Self::PosY,
        Self::NegY,
        Self::PosZ,
        Self::NegZ,
    ];

    /// The direction this facing points at.
    pub const fn dir(self) -> Dir {
        match self {
            Self::PosX => Dir::PosX,
            Self::NegX => Dir::NegX,
            Self::PosY => Dir::PosY,
            Self::NegY => Dir::NegY,
            Self::PosZ => Dir::PosZ,
            Self::NegZ => Dir::NegZ,
        }
    }
}

impl From<Dir> for Facing {
    fn from(dir: Dir) -> Self {
        match dir {
            Dir::PosX => Self::PosX,
            Dir::NegX => Self::NegX,
            Dir::PosY => Self::PosY,
            Dir::NegY => Self::NegY,
            Dir::PosZ => Self::PosZ,
            Dir::NegZ => Self::NegZ,
        }
    }
}

impl From<Facing> for Dir {
    fn from(facing: Facing) -> Self {
        facing.dir()
    }
}

/// The phase of a [`PistonBlock`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum PistonPhase {
    /// Idle; waiting for power.
    #[default]
    Retracted,
    /// Countdown running; the stroke happens when the timer reaches zero.
    Extending,
    /// The stroke completed.
    Extended,
}

/// Packed per-cell state of [`PistonBlock`].
///
/// The state fits in eight bits: two for the phase, three for the facing and
/// three for the timer.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PistonState {
    /// Current phase.
    pub phase: PistonPhase,
    /// Direction the piston moves towards.
    pub facing: Facing,
    /// Remaining ticks while [`PistonPhase::Extending`].
    pub timer: u8,
}

impl PistonState {
    /// A retracted piston facing `facing`.
    pub const fn retracted(facing: Facing) -> Self {
        Self {
            phase: PistonPhase::Retracted,
            facing,
            timer: 0,
        }
    }

    /// The direction this piston moves towards.
    pub const fn dir(&self) -> Dir {
        self.facing.dir()
    }
}

impl Default for PistonState {
    fn default() -> Self {
        Self::retracted(Facing::PosX)
    }
}

impl StateCodec for PistonState {
    const BITS: u8 = 8;

    fn encode(self) -> u16 {
        let phase = match self.phase {
            PistonPhase::Retracted => 0,
            PistonPhase::Extending => 1,
            PistonPhase::Extended => 2,
        };
        let facing = match self.facing {
            Facing::PosX => 0,
            Facing::NegX => 1,
            Facing::PosY => 2,
            Facing::NegY => 3,
            Facing::PosZ => 4,
            Facing::NegZ => 5,
        };
        let timer = self.timer.min(7);
        (phase as u16) | ((facing as u16) << 2) | ((timer as u16) << 5)
    }

    fn decode(raw: u16) -> Self {
        let phase = match raw & 0b11 {
            1 => PistonPhase::Extending,
            2 => PistonPhase::Extended,
            _ => PistonPhase::Retracted,
        };
        let facing = match (raw >> 2) & 0b111 {
            1 => Facing::NegX,
            2 => Facing::PosY,
            3 => Facing::NegY,
            4 => Facing::PosZ,
            5 => Facing::NegZ,
            _ => Facing::PosX,
        };
        Self {
            phase,
            facing,
            timer: ((raw >> 5) & 0b111) as u8,
        }
    }
}

crate::block! {
    /// A piston: waits [`PISTON_EXTEND_TICKS`] scheduled ticks after seeing
    /// the [`Power`](const@Power) capability on a neighbour, then moves the
    /// line of movable blocks in front of it one step.
    ///
    /// The piston itself is [`UnMovable`](trait@UnMovable). It does not pull on retraction
    /// (there is no moving block entity yet); losing power simply resets its
    /// state.
    pub struct PistonBlock;
    facets: UnMovable;
    state: PistonState;
    behavior {
        fn on_placed(ctx: &mut TickCtx<'_>) {
            Self::update(ctx);
        }

        fn on_neighbor_changed(ctx: &mut TickCtx<'_>, _from: Dir) {
            Self::update(ctx);
        }

        fn on_scheduled_tick(ctx: &mut TickCtx<'_>) {
            Self::extend(ctx);
        }
    }
}

impl PistonBlock {
    /// Starts, cancels or resets the extension according to the current power.
    fn update(ctx: &mut TickCtx<'_>) {
        let powered = Self::powered(ctx);
        let mut state = ctx.state::<Self>();

        match state.phase {
            PistonPhase::Retracted if powered => {
                state.phase = PistonPhase::Extending;
                state.timer = PISTON_EXTEND_TICKS;
                ctx.set_state::<Self>(state);
                ctx.schedule_self();
            }
            PistonPhase::Extending | PistonPhase::Extended if !powered => {
                state.phase = PistonPhase::Retracted;
                state.timer = 0;
                ctx.set_state::<Self>(state);
            }
            _ => {}
        }
    }

    /// Advances the countdown; completes the stroke on the last tick.
    fn extend(ctx: &mut TickCtx<'_>) {
        let mut state = ctx.state::<Self>();
        if state.phase != PistonPhase::Extending {
            // Stale schedule (for example after the power was removed).
            return;
        }

        if state.timer > 1 {
            state.timer -= 1;
            ctx.set_state::<Self>(state);
            ctx.schedule_self();
            return;
        }

        // The timer expired: move the line in front of the piston.
        let dir = state.facing.dir();
        let front = ctx.pos().offset(dir);
        state.phase = if Self::stroke(ctx, front, dir) {
            PistonPhase::Extended
        } else {
            // Blocked or too long: stay retracted and wait for a new update.
            PistonPhase::Retracted
        };
        state.timer = 0;
        ctx.set_state::<Self>(state);
    }

    /// Moves the contiguous line of movable blocks starting at `start` one
    /// step in `dir`, preserving cell states.
    ///
    /// Returns `false` when the line is blocked, longer than
    /// [`PISTON_PUSH_LIMIT`], or runs into an unloaded chunk; in that case
    /// nothing moves.
    fn stroke(ctx: &mut TickCtx<'_>, start: Pos, dir: Dir) -> bool {
        // 1. Collect the line, ending at the first air cell.
        let mut line = LineBuf::new();
        let mut cursor = start;
        loop {
            let Some(block) = ctx.get(cursor) else {
                return false;
            };
            if block.caps().contains(CapSet::EMPTY) {
                break;
            }
            if !Self::movable(block.caps()) {
                return false;
            }
            if line.len() >= PISTON_PUSH_LIMIT {
                return false;
            }
            line.push(cursor);
            cursor = cursor.offset(dir);
        }

        // Nothing in front: extending is a no-op.
        if line.is_empty() {
            return true;
        }

        // 2. Apply far to near, so every cell is vacated before it is filled
        //    again.
        for index in (0..line.len()).rev() {
            let from = line.get(index);
            if !ctx.move_cell(from, from.offset(dir)) {
                return false;
            }
        }

        // 3. Tell the neighbourhood that the line moved.
        for index in 0..line.len() {
            let from = line.get(index);
            ctx.notify(from);
            ctx.notify(from.offset(dir));
        }
        true
    }

    /// Whether a capability set allows this piston to move the block.
    fn movable(caps: CapSet) -> bool {
        caps.contains(CapSet::MOVABLE) && !caps.contains(CapSet::UNMOVABLE)
    }

    /// Whether any of the six neighbours carries the [`Power`] capability.
    fn powered(ctx: &TickCtx<'_>) -> bool {
        let pos = ctx.pos();
        Dir::ALL.into_iter().any(|dir| {
            ctx.get(pos.offset(dir))
                .map(|block| block.caps().contains(CapSet::POWER))
                .unwrap_or(false)
        })
    }
}

crate::block! {
    /// A block that supplies power to adjacent [`PistonBlock`]s.
    ///
    /// This is the minimal reference power source; custom power providers can
    /// instead implement the [`Power`](trait@Power) facet directly.
    pub struct PowerBlock;
    facets: Power, UnMovable;
}

/// Small fixed buffer for a piston stroke line.
struct LineBuf {
    items: [Pos; PISTON_PUSH_LIMIT],
    len: usize,
}

impl LineBuf {
    const fn new() -> Self {
        Self {
            items: [Pos::ZERO; PISTON_PUSH_LIMIT],
            len: 0,
        }
    }

    fn len(&self) -> usize {
        self.len
    }

    fn is_empty(&self) -> bool {
        self.len == 0
    }

    fn get(&self, index: usize) -> Pos {
        self.items[index]
    }

    fn push(&mut self, pos: Pos) {
        debug_assert!(self.len < PISTON_PUSH_LIMIT);
        self.items[self.len] = pos;
        self.len += 1;
    }
}
