//! Embedded Flash Memory Controller

use core::cell::Cell;
use core::ptr;
use kernel::common::cells::OptionalCell;
use kernel::common::cells::TakeCell;
use kernel::common::cells::VolatileCell;
use kernel::common::deferred_call::DeferredCall;
use kernel::common::registers::register_bitfields;
use kernel::common::registers::{ReadWrite, WriteOnly};
use kernel::common::StaticRef;
use kernel::hil;
use kernel::ReturnCode;

use crate::deferred_call_tasks::DeferredCallTask;

const FLASH_BASE: StaticRef<FlashRegisters> =
    unsafe { StaticRef::new(0x40023C00 as *const FlashRegisters) };

#[repr(C)]
struct FlashRegisters {
    /// Flash access control register
    /// Adress offset 0x00
    pub acr: ReadWrite<u32, AccessControl::Register>,
    /// Flash key register
    /// Adress offset 0x04
    pub kr: WriteOnly<u32, Key::Register>,
    /// Flash option key register
    /// Adress offset 0x08
    pub okr: WriteOnly<u32, Key::Register>,
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
        EOP OFFSET(0) NUMBITS(1) []
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
        NWRP OFFSET(16) NUMBITS(12) [],
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
        /// Bit 7: NRSTSTDBY
        /// Bit 6: NRSTSTOP
        /// Bit 5: WDGSW
        USER OFFSET(5) NUMBITS(3) [],
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
    unsafe { DeferredCall::new(DeferredCallTask::Flash) };

const KEY1: u32 = 0x45670123;
const KEY2: u32 = 0xCDEF89AB;

const OPTKEY1: u32 = 0x08192A3B;
const OPTKEY2: u32 = 0x4C5D6E7F;

const FLASH_START: usize = 0x08000000;
const FLASH_END: usize = 0x080FFFFF;

pub static mut FLASH: Flash = Flash::new();

/// FlashState is used to track the current state and command of the flash.
#[derive(Clone, Copy, PartialEq)]
pub enum FlashState {
    Ready,
    Read,
    Write,
    Erase,
    WriteOption,
}

pub struct Flash {
    registers: StaticRef<FlashRegisters>,
    client: OptionalCell<&'static dyn hil::flash::ClientPageless>,
    buffer: TakeCell<'static, [u8]>,
    buffer_length: Cell<usize>,
    write_address: Cell<usize>,
    write_counter: Cell<usize>,
    state: Cell<FlashState>,
}

impl Flash {
    pub const fn new() -> Flash {
        Flash {
            registers: FLASH_BASE,
            client: OptionalCell::empty(),
            buffer: TakeCell::empty(),
            buffer_length: Cell::new(0),
            state: Cell::new(FlashState::Ready),
            write_address: Cell::new(0),
            write_counter: Cell::new(0),
        }
    }

    // Enable hardware interrupts
    pub fn enable(&self) {
        self.registers.cr.modify(Control::EOPIE::SET);
        self.registers.cr.modify(Control::ERRIE::SET);
    }

    pub fn is_locked(&self) -> bool {
        self.registers.cr.is_set(Control::LOCK)
    }

    pub fn unlock(&self) {
        self.registers.kr.write(Key::KEYR.val(KEY1));
        self.registers.kr.write(Key::KEYR.val(KEY2));
    }

    pub fn lock(&self) {
        self.registers.cr.modify(Control::LOCK::SET);
    }

    pub fn is_locked_option(&self) -> bool {
        self.registers.ocr.is_set(OptionControl::OPTLOCK)
    }

    pub fn unlock_option(&self) {
        self.registers.okr.write(Key::KEYR.val(OPTKEY1));
        self.registers.okr.write(Key::KEYR.val(OPTKEY2));
    }

    pub fn lock_option(&self) {
        self.registers.ocr.modify(OptionControl::OPTLOCK::SET);
    }

    /// Allows configuring the number of bytes to be programmed each time
    /// a write operation occurs. The erase time also depends on the PSIZE value.
    ///
    /// Note: any program or erase operation started with inconsistent
    /// parallelism/voltage settings may lead to unpredicted results.
    pub fn set_parallelism(&self, parallelism: u32) -> ReturnCode {
        match parallelism {
            0 => {
                self.registers.cr.modify(Control::PSIZE::Byte);
                ReturnCode::SUCCESS
            }
            1 => {
                self.registers.cr.modify(Control::PSIZE::HalfWord);
                ReturnCode::SUCCESS
            }
            2 => {
                self.registers.cr.modify(Control::PSIZE::Word);
                ReturnCode::SUCCESS
            }
            3 => {
                self.registers.cr.modify(Control::PSIZE::DoubleWord);
                ReturnCode::SUCCESS
            }
            _ => ReturnCode::EINVAL,
        }
    }

    pub fn get_parallelism(&self) -> u32 {
        self.registers.cr.read(Control::PSIZE)
    }

    fn program_byte(&self) {
        self.buffer.take().map(|buffer| {
            let i = self.write_counter.get();
            let address = self.write_address.get();

            let location = unsafe { &*((address + i) as *const VolatileCell<u8>) };
            location.set(buffer[i]);

            self.buffer.replace(buffer);
        });
    }

    pub fn handle_interrupt(&self) {
        if self.registers.sr.is_set(Status::EOP) {
            // Cleared by writing a 1
            self.registers.sr.modify(Status::EOP::SET);
            match self.state.get() {
                FlashState::Write => {
                    self.write_counter.set(self.write_counter.get() + 1);

                    if self.write_counter.get() == self.buffer_length.get() {
                        self.registers.cr.modify(Control::PG::CLEAR);
                        self.state.set(FlashState::Ready);
                        self.write_counter.set(0);

                        self.client.map(|client| {
                            self.buffer.take().map(|buffer| {
                                client.write_complete(
                                    buffer,
                                    self.buffer_length.get(),
                                    hil::flash::Error::CommandComplete,
                                );
                            });
                        });
                    }
                }
                FlashState::Erase => {
                    if self.registers.cr.is_set(Control::SER) {
                        self.registers.cr.modify(Control::SER::CLEAR);
                    }

                    if self.registers.cr.is_set(Control::MER) {
                        self.registers.cr.modify(Control::MER::CLEAR);
                    }

                    self.state.set(FlashState::Ready);
                    self.client.map(|client| {
                        client.erase_complete(hil::flash::Error::CommandComplete);
                    });
                }
                _ => {}
            }
        }

        if self.registers.sr.is_set(Status::RDERR) {
            // Cleared by writing a 1.
            self.registers.sr.modify(Status::RDERR::SET);
            self.client.map(|client| {
                self.buffer.take().map(|buffer| {
                    client.read_complete(
                        buffer,
                        self.buffer_length.get(),
                        hil::flash::Error::FlashErrorSpecific("Read Protection Error"),
                    );
                });
            });
        }

        if self.registers.sr.is_set(Status::PGSERR) {
            // Cleared by writing a 1.
            self.registers.sr.modify(Status::PGSERR::SET);
            self.client.map(|client| {
                self.buffer.take().map(|buffer| {
                    client.write_complete(
                        buffer,
                        self.buffer_length.get(),
                        hil::flash::Error::FlashErrorSpecific("Programming Sequence Error"),
                    );
                });
            });
        }

        if self.registers.sr.is_set(Status::PGPERR) {
            // Cleared by writing a 1.
            self.registers.sr.modify(Status::PGPERR::SET);
            self.client.map(|client| {
                self.buffer.take().map(|buffer| {
                    client.write_complete(
                        buffer,
                        self.buffer_length.get(),
                        hil::flash::Error::FlashErrorSpecific("Programming Parallelism Error"),
                    );
                });
            });
        }

        if self.registers.sr.is_set(Status::PGAERR) {
            // Cleared by writing a 1.
            self.registers.sr.modify(Status::PGAERR::SET);
            self.client.map(|client| {
                self.buffer.take().map(|buffer| {
                    client.write_complete(
                        buffer,
                        self.buffer_length.get(),
                        hil::flash::Error::FlashErrorSpecific("Programming Alignment Error"),
                    );
                });
            });
        }

        if self.registers.sr.is_set(Status::WRPERR) {
            // Cleared by writing a 1.
            self.registers.sr.modify(Status::WRPERR::SET);
            match self.state.get() {
                FlashState::Write => {
                    self.client.map(|client| {
                        self.buffer.take().map(|buffer| {
                            client.write_complete(
                                buffer,
                                self.buffer_length.get(),
                                hil::flash::Error::FlashErrorSpecific("Write Protection Error"),
                            );
                        });
                    });
                }
                FlashState::Erase => {
                    self.client.map(|client| {
                        client.erase_complete(hil::flash::Error::FlashErrorSpecific(
                            "Write Protection Error",
                        ));
                    });
                }
                _ => {}
            }
        }

        if self.state.get() == FlashState::Read {
            self.state.set(FlashState::Ready);
            self.client.map(|client| {
                self.buffer.take().map(|buffer| {
                    client.read_complete(
                        buffer,
                        self.buffer_length.get(),
                        hil::flash::Error::CommandComplete,
                    );
                });
            });
        }
    }

    pub fn read(
        &self,
        buffer: &'static mut [u8],
        address: usize,
        length: usize,
    ) -> Result<(), (ReturnCode, &'static mut [u8])> {
        let mut byte: *const u8 = address as *const u8;
        unsafe {
            for i in 0..length {
                buffer[i] = ptr::read_volatile(byte);
                byte = byte.offset(1);
            }
        }

        self.buffer.replace(buffer);
        self.state.set(FlashState::Read);
        DEFERRED_CALL.set();

        Ok(())
    }

    pub fn write(
        &self,
        buffer: &'static mut [u8],
        address: usize,
        length: usize,
    ) -> Result<(), (ReturnCode, &'static mut [u8])> {
        if address < FLASH_START && address + length > FLASH_END {
            return Err((ReturnCode::EINVAL, buffer));
        }

        if self.is_locked() {
            self.unlock();
        }

        self.enable();
        self.state.set(FlashState::Write);
        self.registers.cr.modify(Control::PG::SET);

        self.buffer.replace(buffer);
        self.buffer_length.set(length);
        self.write_address.set(address);

        match self.get_parallelism() {
            0 => self.program_byte(),
            _ => {}
        }

        Ok(())
    }

    pub fn erase_sector(&self, sector_number: usize) -> ReturnCode {
        if self.is_locked() {
            self.unlock();
        }

        self.enable();
        self.state.set(FlashState::Erase);

        self.registers.cr.modify(Control::SER::SET);
        self.registers
            .cr
            .modify(Control::SNB.val(sector_number as u32));
        self.registers.cr.modify(Control::STRT::SET);

        ReturnCode::SUCCESS
    }

    pub fn erase_all(&self) -> ReturnCode {
        if self.is_locked() {
            self.unlock();
        }

        self.enable();
        self.state.set(FlashState::Erase);

        self.registers.cr.modify(Control::MER::SET);
        self.registers.cr.modify(Control::STRT::SET);

        ReturnCode::SUCCESS
    }

    pub fn write_option(&self, value: u32) -> ReturnCode {
        if self.is_locked_option() {
            self.unlock_option();
        }

        self.enable();
        self.state.set(FlashState::WriteOption);
        self.registers.ocr.set(value);
        self.registers.ocr.modify(OptionControl::OPTSTRT::SET);

        ReturnCode::SUCCESS
    }
}

impl<C: hil::flash::ClientPageless> hil::flash::HasClient<'static, C> for Flash {
    fn set_client(&self, client: &'static C) {
        self.client.set(client);
    }
}

impl hil::flash::FlashPageless for Flash {
    fn read(
        &self,
        buffer: &'static mut [u8],
        address: usize,
        length: usize,
    ) -> Result<(), (ReturnCode, &'static mut [u8])> {
        self.read(buffer, address, length)
    }

    fn write(
        &self,
        buffer: &'static mut [u8],
        address: usize,
        length: usize,
    ) -> Result<(), (ReturnCode, &'static mut [u8])> {
        self.write(buffer, address, length)
    }

    fn erase(&self, erase_identifier: usize) -> ReturnCode {
        self.erase_sector(erase_identifier)
    }
}
