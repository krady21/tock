//! Embedded Flash Memory Controller

use core::cell::Cell;
use core::ops::{Index, IndexMut};
use kernel::common::cells::OptionalCell;
use kernel::common::cells::TakeCell;
use kernel::common::cells::VolatileCell;
use kernel::common::deferred_call::DeferredCall;
use kernel::common::registers::register_bitfields;
use kernel::common::registers::{ReadOnly, ReadWrite, WriteOnly};
use kernel::common::StaticRef;
use kernel::hil;
use kernel::ReturnCode;

use crate::deferred_call_tasks::DeferredCallTask;

// TODO: Change address
const FLASH_BASE: StaticRef<FlashRegisters> =
    unsafe { StaticRef::new(0x4001E400 as *const FlashRegisters) };

#[repr(C)]
struct FlashRegisters {
    /// Flash access control register
    /// Adress offset 0x00
    pub acr: ReadWrite<u32, AccessControl::Register>,
    /// Flash key register
    /// Adress offset 0x04
    pub kr: ReadWrite<u32, Key::Register>,
    /// Flash option key register
    /// Adress offset 0x08
    pub okr: ReadWrite<u32, Key::Register>,
    /// Flash status register
    /// Adress offset 0x0C
    pub sr: ReadWrite<u32, Status::Register>,
    /// Flash control register
    /// Adress offset 0x10
    pub cr: ReadWrite<u32, Control::Register>,
    /// Flash option control register
    /// Adress offset 0x14
    pub ocr: ReadWrite<u32, OptionControl::Register>,
}

register_bitfields! [u32,
    AccessControl [
        /// Data cache reset
        DCRST OFFSET(12) NUMBITS(1) [],
        /// Instruction cache reset
        ICRST OFFSET(11) NUMBITS(1) [],
        /// Data cache enable
        DCEN OFFSET(10) NUMBITS(1) [],
        /// Instruction cache enable
        ICEN OFFSET(9) NUMBITS(1) [],
        /// Prefetch enable
        PRFTEN OFFSET(8) NUMBITS(1) [],
        /// Represents the ratio of the CPU clock period to the Flash
        /// memory access time.
        /// ex: 0x0111 = 7 wait states
        LATENCY OFFSET(0) NUMBITS(4) []
    ],
    Key [
        /// Flash or option key
        /// Represents the keys to unlock the flash control register
        /// or the flash option control register.
        KEYR OFFSET(0) NUMBITS(32) []
    ],
    Status [
        /// Busy
        /// Indicates that a flash operation is in progress. This is set on
        /// the beginning of a flash operation and reset when the operation
        /// finishes or an error occurs.
        BSY OFFSET(16) NUMBITS(1) [],
        /// Read protection error
        /// Set by hardware when an address to be read through Dbus belongs
        /// to a read protected part of the flash.
        RDERR OFFSET(8) NUMBITS(1) [],
        /// Programming sequence error
        /// Set by hardware when a write access to the flash is performed by the
        /// code while the control register has not been correctly configured.
        PGSERR OFFSET(7) NUMBITS(1) [],
        /// Programming parallelism error
        /// Set by hardware when the size of the access during the program
        /// sequence does not correspond to the parallelism configuration PSIZE.
        PGPERR OFFSET(6) NUMBITS(1) [],
        /// Programming alignment error
        /// Set by hardware when the data to program cannot be contained in the
        /// same 128-bit flash memory row.
        PGAERR OFFSET(5) NUMBITS(1) [],
        /// Write protection error
        /// Set by hardware when an address to be erased/programmed belongs to
        /// a write-protected part of the flash memory.
        WRPERR OFFSET(4) NUMBITS(1) [],
        /// Operation error
        /// Set by hardware when a flash operation request is detected and can
        /// not be run because of parallelism, alignment, or write protection.
        OPERR OFFSET(1) NUMBITS(1) [],
        /// End of operation
        /// Set by hardware when one or more flash memory operations has/have
        /// completed successfully.
        EOP OFFSET(0) NUMBITS(1) [],
    ],
    Control [
        /// When set, this bit indicates that the control register is locked.
        /// It is clearedby hardware after detetcting the unlock sequence.
        LOCK OFFSET(31) NUMBITS(1) [],
        /// Error interrupt enable
        /// This bit enables interrupt generation when the OPERR bit in the
        /// status register is set.
        ERRIE OFFSET(25) NUMBITS(1) [],
        /// End of operation interrupt enable
        /// This bit enables interrupt generation when the EOP bit in the
        /// status register is set.
        EOPIE OFFSET(24) NUMBITS(1) [],
        /// This bit triggers an erase operation when set. It is set only by
        /// software and cleared when the BSY bit is cleared.
        STRT OFFSET(16) NUMBITS(1) [],
        /// Program size
        /// These bits select the program parallelism.
        PSIZE OFFSET(8) NUMBITS(2) [
            /// Program x8
            Byte = 0,
            /// Program x16
            HalfWord = 1,
            /// Program x32
            Word = 2,
            /// Program x64
            DoubleWord = 3
        ],
        /// Sector number
        /// These bits select the sector to erase.
        /// 0-11: sectors 0-11
        /// 12: user specific sector
        /// 13: user configuration sector
        /// 14-15: not allowed
        SNB OFFSET(3) NUMBITS(4) [],
        /// Mass erase
        /// Erase activated for all user sectors.
        MER OFFSET(2) NUMBITS(1) [],
        /// Sector erase
        /// Erase activated for a specific user sector.
        SER OFFSET(1) NUMBITS(1) [],
        /// Programming
        /// Programming activated.
        PG OFFSET(0) NUMBITS(1) []
    ],
    OptionControl [
        /// Selection of protection mode of nWPRi bits
        SPRMOD OFFSET(31) NUMBITS(1) [
            /// PCROP disabled, nWPRi bits used write protection on sector i
            DISABLED = 0,
            /// PCROP enabled, nWPRi bits used PCROP protection on sector i
            ENABLED = 1
        ],
        /// Not write protect
        /// These bits contain the value of the write-protection option bytes
        /// of sectors after reset. They can be written to program a new write
        /// protection value into flash memory.
        NWRP OFFSET(16) NUMBITS(12) [
            // TODO
        ],
        /// Read protect
        /// These bits contain the value of the read-protection option level
        /// after reset. They can be written to program a new read protection
        /// value into flash memory.
        /// 0xAA: Level 0, read protection not active
        /// 0xCC: Level 2, chip read protection active
        /// others: Level 1, read protection of memories active
        RDP OFFSET(8) NUMBITS(8) [],
        /// User option bytes
        /// These bits contain the value of the user option byte after reset.
        /// They can be written to program a new user option byte value into
        /// flash memory.
        USER [
            NRSTSTDBY 7,
            NRSTSTOP  6,
            WDGSW     5
        ],
        /// BOR reset level
        /// These bits contain the supply level threshold that activates
        /// or releases the reset. They can be written to program a new BOR
        /// level. By default, BOR is off.
        BORLEVEL OFFSET(2) NUMBITS(2) [
            /// brownout threshold level 3
            VBOR3 = 0,
            /// brownout threshold level 2
            VBOR2 = 1,
            /// brownout threshold level 1
            VBOR1 = 2,
            /// POR/PDR reset threshold level is applied
            OFF = 3
        ],
        /// Option start
        /// This bit triggers a user option operation when set. It is set only
        /// by software and cleared when the BSY bit is cleared.
        OPTSTRT OFFSET(1) NUMBITS(1) [],
        /// Option lock
        /// When this bit is set, it indicates that the OptionControl register
        /// is locked. This bit is cleared by hardware after detecting the
        /// unlock sequence.
        OPTLOCK OFFSET(1) NUMBITS(1) []
    ]

];

/// This mechanism allows us to schedule "interrupts" even if the hardware
/// does not support them.
static DEFERRED_CALL: DeferredCall<DeferredCallTask> =
    unsafe { DeferredCall::new(DeferredCallTask::Nvmc) };

pub struct StmF4Page();

pub static mut FLASH: Flash = Flash::new();

/// FlashState is used to track the current state and command of the flash.
#[derive(Clone, Copy, PartialEq)]
pub enum FlashState {
    // TODO
}

pub struct Flash {
    registers: StaticRef<FlashRegisters>,
    client: OptionalCell<&'static dyn hil::flash::Client<Flash>>,
    buffer: TakeCell<'static, StmF4Page>,
    state: Cell<FlashState>,
}

impl<C: hil::flash::Client<Self>> hil::flash::HasClient<'static, C> for Flash {
    fn set_client(&self, client: &'static C) {
        self.client.set(client);
    }
}

impl hil::flash::Flash for Flash {
    type Page = NrfPage;

    fn read_page(
        &self,
        page_number: usize,
        buf: &'static mut Self::Page,
    ) -> Result<(), (ReturnCode, &'static mut Self::Page)> {
        self.read_range(page_number, buf)
    }

    fn write_page(
        &self,
        page_number: usize,
        buf: &'static mut Self::Page,
    ) -> Result<(), (ReturnCode, &'static mut Self::Page)> {
        self.write_page(page_number, buf)
    }

    fn erase_page(&self, page_number: usize) -> ReturnCode {
        self.erase_page(page_number)
    }
}
