//! unwind target's program

use anyhow::{anyhow, Context as _};
use gimli::{
    BaseAddresses, DebugFrame, LittleEndian, UninitializedUnwindContext, UnwindSection as _,
};
use probe_rs::{config::RamRegion, Core};

use crate::{
    cortexm,
    registers::{self, Registers},
    stacked::Stacked,
    Outcome, VectorTable,
};

static MISSING_DEBUG_INFO: &str = "debug information is missing. Likely fixes:
1. compile the Rust code with `debug = 1` or higher. This is configured in the `profile.{release,bench}` sections of Cargo.toml (`profile.{dev,test}` default to `debug = 2`)
2. use a recent version of the `cortex-m` crates (e.g. cortex-m 0.6.3 or newer). Check versions in Cargo.lock
3. if linking to C code, compile the C code with the `-g` flag";


/// Virtually* unwinds the target's program
/// \* destructors are not run
// On error, returns all info collected so far in `Output` and the error that occurred
//                   in `Output::processing_error`
pub(crate) fn target(
    core: &mut Core,
    debug_frame: &[u8],
    vector_table: &VectorTable,
    sp_ram_region: &Option<RamRegion>,
) -> Output {

    let mut output = Output {
        corrupted: true,
        outcome: Outcome::Ok,
        raw_frames: vec![],
        processing_error: Ok(()),
    };

    // returns all info collected until the error occurred and puts the error into `processing_error`
    macro_rules! unwrap_or_return {
        ( $e:expr ) => {
            match $e {
                Ok(x) => {
                    x},
                Err(err) => {
                    output.processing_error = Err(anyhow!(err));

                    // TODO rm
                    println!("xoxo {:?}", output);

                    return output
                },
            }
        }
    }


    let mut debug_frame = DebugFrame::new(debug_frame, LittleEndian);
    debug_frame.set_address_size(cortexm::ADDRESS_SIZE);

    let mut pc = unwrap_or_return!(core.read_core_reg(registers::PC));
    let sp = unwrap_or_return!(core.read_core_reg(registers::SP));
    let lr = unwrap_or_return!(core.read_core_reg(registers::LR));
    let base_addresses = BaseAddresses::default();
    let mut unwind_context = UninitializedUnwindContext::new();
    let mut registers = Registers::new(lr, sp, core);

    loop {
        if cortexm::is_hard_fault(pc, vector_table) {
            assert!(
                output.raw_frames.is_empty(),
                "when present HardFault handler must be the first frame we unwind but wasn't"
            );

            output.outcome = if overflowed_stack(sp, sp_ram_region) {
                Outcome::StackOverflow
            } else {
                Outcome::HardFault
            };
        }

        output.raw_frames.push(RawFrame::Subroutine { pc });

        let uwt_row = unwrap_or_return!(debug_frame
            .unwind_info_for_address(
                &base_addresses,
                &mut unwind_context,
                pc.into(),
                DebugFrame::cie_from_offset,
            )
            .with_context(|| MISSING_DEBUG_INFO));

        let cfa_changed = unwrap_or_return!(registers.update_cfa(uwt_row.cfa()));

        for (reg, rule) in uwt_row.registers() {
            unwrap_or_return!(registers.update(reg, rule));
        }

        let lr = unwrap_or_return!(registers.get(registers::LR));

        log::debug!("LR={:#010X} PC={:#010X}", lr, pc);

        if lr == registers::LR_END {
            break;
        }

        // Link Register contains an EXC_RETURN value. This deliberately also includes
        // invalid combinations of final bits 0-4 to prevent futile backtrace re-generation attempts
        let exception_entry = lr >= cortexm::EXC_RETURN_MARKER;

        let program_counter_changed = !cortexm::subroutine_eq(lr, pc);

        // If the frame didn't move, and the program counter didn't change, bail out (otherwise we
        // might print the same frame over and over).
        output.corrupted = !cfa_changed && !program_counter_changed;

        if output.corrupted {
            break;
        }

        if exception_entry {
            output.raw_frames.push(RawFrame::Exception);

            let fpu = match lr {
                0xFFFFFFF1 | 0xFFFFFFF9 | 0xFFFFFFFD => false,
                0xFFFFFFE1 | 0xFFFFFFE9 | 0xFFFFFFED => true,
                _ => {
                    output.processing_error = Err(anyhow!("LR contains invalid EXC_RETURN value {:#010X}", lr));
                    return output;
                },
            };

            let sp = unwrap_or_return!(registers.get(registers::SP));
            let ram_bounds = sp_ram_region
                .as_ref()
                .map(|ram_region| ram_region.range.clone())
                .unwrap_or(cortexm::VALID_RAM_ADDRESS);
            let stacked = if let Some(stacked) = unwrap_or_return!(
                Stacked::read(registers.core, sp, fpu, ram_bounds))
            {
                stacked
            } else {
                output.corrupted = true;
                break;
            };

            registers.insert(registers::LR, stacked.lr);
            // adjust the stack pointer for stacked registers
            registers.insert(registers::SP, sp + stacked.size());

            pc = stacked.pc;
        } else {
            if cortexm::is_thumb_bit_set(lr) {
                pc = cortexm::clear_thumb_bit(lr);
            } else {
                output.processing_error = Err(anyhow!(
                                              "bug? LR ({:#010x}) didn't have the Thumb bit set",
                                              lr));
                return output;
            }
        }
    }

    output
}

#[derive(Debug)]
pub struct Output {
    pub(crate) corrupted: bool,
    pub(crate) outcome: Outcome,
    pub(crate) raw_frames: Vec<RawFrame>,
    // will be `Some` if an error occured while putting together the output.
    // `outcome` and `raw_frames` will contain all info collected until the error occurred.
    pub(crate) processing_error: anyhow::Result<()>, // TODO this feels clunky?
}

/// Backtrace frame prior to 'symbolication'
#[derive(Debug)]
pub(crate) enum RawFrame {
    Subroutine { pc: u32 },
    Exception,
}

impl RawFrame {
    /// Returns `true` if the raw_frame is [`Exception`].
    pub(crate) fn is_exception(&self) -> bool {
        matches!(self, Self::Exception)
    }
}

fn overflowed_stack(sp: u32, sp_ram_region: &Option<RamRegion>) -> bool {
    if let Some(sp_ram_region) = sp_ram_region {
        // NOTE stack is full descending; meaning the stack pointer can be
        // `ORIGIN(RAM) + LENGTH(RAM)`
        let range = sp_ram_region.range.start..=sp_ram_region.range.end;
        !range.contains(&sp)
    } else {
        log::warn!("no RAM region appears to contain the stack; cannot determine if this was a stack overflow");
        false
    }
}
