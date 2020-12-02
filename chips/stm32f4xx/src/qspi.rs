use kernel::common::registers::register_bitfields;
use kernel::common::registers::{ReadOnly, ReadWrite, WriteOnly};
use kernel::common::StaticRef;

const QSPI_BASE: StaticRef<QspiRegisters> = unsafe { StaticRef::new(0x9000_000 as *const QspiRegisters) };

#[repr(C)]
struct QspiRegisters {
    /// Control register
    cr: ReadWrite<u32, Control::Register>,
    /// Device configuration register
    dcr: ReadWrite<u32, DeviceConfig::Register>,
    /// Status register
    sr: ReadOnly<u32, Status::Register>,
    /// Flag clear register
    fcr: WriteOnly<u32, FlagClear::Register>,
    /// Data length register
    dlr: ReadWrite<u32, DataLength::Register>,
    /// Communication configuration register
    ccr: ReadWrite<u32, CommConfig::Register>,
    /// Address register
    ar: ReadWrite<u32, Address::Register>,
    /// Alternate bytes register
    abr: ReadWrite<u32, AlternateBytes::Register>,
    /// Data register
    dr: ReadWrite<u32, Data::Register>,
    /// Polling status mask register
    psmkr: ReadWrite<u32, PSMask::Register>,
    /// Polling status match register
    psmar: ReadWrite<u32, PSMatch::Register>,
    /// Polling interval register
    pir: ReadWrite<u32, PInterval::Register>,
    /// Low-power timeout register
    lptr: ReadWrite<u32, LPTimeout::Register>,
}

register_bitfields! [u32,
    Control [
        /// Clock prescaler
        PRESCALER OFFSET(24) NUMBITS(8) [],
        /// Polling match mode
        PMM OFFSET(23) NUMBITS(1) [],
        /// Automatic poll mode stop
        APMS OFFSET(22) NUMBITS(1) [],
        /// TimeOut interrupt enable
        TOIE OFFSET(21) NUMBITS(1) [],
        /// Status match interrupt enable
        SMIE OFFSET(19) NUMBITS(1) [],
        /// FIFO threshold interrupt enable
        FTIE OFFSET(18) NUMBITS(1) [],
        /// Transfer complete interrupt enable
        TCIE OFFSET(17) NUMBITS(1) [],
        /// Transfer error interrupt enable
        TEIE OFFSET(16) NUMBITS(1) [],
        /// FIFO threshold level
        FTHRESH OFFSET(8) NUMBITS(5) [],
        /// Flash memory selection
        FSEL OFFSET(7) NUMBITS(1) [
            /// Flash 1 selected
            FLASH1 = 0,
            /// Flash 2 selected
            FLASH2 = 1
        ],
        /// Dual-flash mode
        DFM OFFSET(6) NUMBITS(1) [],
        /// Sample shift
        SSHIFT OFFSET(4) NUMBITS(1) [
            /// No shift
            NOSHIFT = 0,
            /// 1/2 cycle shift
            HALFCYCLESHIFT = 1
        ],
        /// Timeout counter enable
        TCEN OFFSET(3) NUMBITS(1) [],
        /// DMA enable
        DMAEN OFFSET(2) NUMBITS(1) [],
        /// Abort request
        ABORT OFFSET(1) NUMBITS(1) [],
        /// QUADSPI Enable
        EN OFFSET(0) NUMBITS(1) [],
    ],
    DeviceConfig [
        /// Flash memory size
        FSIZE OFFSET(16) NUMBITS(5) [],
        /// Chip select high time
        CSHT OFFSET(8) NUMBITS(3) [],
        /// Mode 0 / Mode 3
        CKMODE OFFSET(0) NUMBITS(1) [
            MODE0 = 0,
            MODE3 = 1,
        ],
    ],
    Status [
        /// FIFO LEVEL
        FLEVEL OFFSET(8) NUMBITS(6) [],
        /// Busy
        BUSY OFFSET(5) NUMBITS(1) [],
        /// Timeout flag
        TOF OFFSET(4) NUMBITS(1) [],
        /// Status match flag
        SMF OFFSET(3) NUMBITS(1) [],
        /// FIFO threshold flag
        FTF OFFSET(2) NUMBITS(1) [],
        /// Transfer complete flag
        TCF OFFSET(1) NUMBITS(1) [],
        /// Transfer error flag
        TEF OFFSET(0) NUMBITS(1) [],
    ],
    FlagClear [
        /// Clear timeout flag
        CTOF OFFSET(4) NUMBITS(1) [],
        /// Clear status match flag
        CSMF OFFSET(3) NUMBITS(1) [],
        /// Clear transfer complete flag
        CTCF OFFSET(1) NUMBITS(1) [],
        /// Clear transfer error flag
        CTEF OFFSET(5) NUMBITS(1) [],
    ],
    DataLength [
        /// Data length
        DL OFFSET(0) NUMBITS(32) [],
    ],
    CommConfig [
        /// Double data rate mode
        DDRM OFFSET(31) NUMBITS(1) [],
        /// DDR hold
        DHHC OFFSET(30) NUMBITS(1) [
            /// Delay data output using analog delay
            ANALOG = 0,
            /// Delay data output by 1/4 of the QUADSPI output clock cycle
            QUARTER = 1,
        ],
        /// Send instruction only once mode
        SIOO OFFSET(28) NUMBITS(1) [
            /// Send instruction on every transaction
            EVERY = 0,
            /// Send instruction only for the first command
            FIRST = 1
        ],
        /// Functional mode
        FMODE OFFSET(26) NUMBITS(2) [
            /// Indirect write mode
            INWRITE = 0,
            /// Indirect read mode
            INREAD = 1,
            /// Automatic polling mode
            AUTOPOLLING = 2,
            /// Memory-mapped mode
            MEMMAPPED = 3
        ],
        /// Data mode
        DMODE OFFSET(24) NUMBITS(2) [
            /// No data
            NODATA = 0,
            /// Data on a single line
            SINGLE = 1,
            /// Data on two lines
            TWO = 2,
            /// Data on four lines
            FOUR = 3
        ],
        /// Number of dummy cycles
        DCYC OFFSET(18) NUMBITS(5) [],
        /// Alternate bytes size
        ABSIZE OFFSET(16) NUMBITS(2) [
            /// 8-bit alternate bytes
            EIGHT = 0,
            /// 16-bit alternate bytes
            SIXTEEN = 1,
            /// 24-bit alternate bytes
            TWENTYFOUR = 2,
            /// 32-bit alternate bytes
            THIRTYTWO = 3
        ],
        /// Alternate bytes mode
        ABMODE OFFSET(14) NUMBITS(2) [
            /// No alternate bytes
            NOBYTES = 0,
            /// Alternate bytes on a single line
            SINGLE = 1,
            /// Alternate bytes on two lines
            TWO = 2,
            /// Alternate bytes on four lines
            FOUR = 3
        ],
        /// Address size
        ADSIZE OFFSET(12) NUMBITS(2) [
            /// 8-bit address
            EIGHT = 0,
            /// 16-bit address
            SIXTEEN = 1,
            /// 24-bit address
            TWENTYFOUR = 2,
            /// 32-bit address
            THIRTYTWO = 3
        ],
        /// Address mode
        ADMODE OFFSET(10) NUMBITS(2) [
            /// No address
            NOADDRESS = 0,
            /// Adddress on a single line
            SINGLE = 1,
            /// Adddress on two lines
            TWO = 2,
            /// Adddress on four lines
            FOUR = 3
        ],
        /// Instruction mode
        IMODE OFFSET(8) NUMBITS(2) [
            /// No instruction
            NOINSTR = 0,
            /// Instruction on a single line
            SINGLE = 1,
            /// Instruction on two lines
            TWO = 2,
            /// Instruction on four lines
            FOUR = 3
        ],
        /// Instruction
        INSTRUCTION OFFSET(0) NUMBITS(8) []
    ],
    Address [
        /// Address to be sent to the external Flash memory
        ADDRESS OFFSET(0) NUMBITS(32) []
    ],
    AlternateBytes [
        /// Alternate Bytes
        ALTERNATE OFFSET(0) NUMBITS(32) []
    ],
    Data [
        /// Data to be sent/received to/from the external SPI device
        DATA OFFSET(0) NUMBITS(32) []
    ],
    PSMask [
        /// Status mask
        MASK OFFSET(0) NUMBITS(32) [
            /// Bit n of the data received in automatic polling mode is masked
            /// and its value is not considered in the matching logic
            MASKED = 0,
            /// Bit n of the data received in automatic polling mode is 
            /// unmasked and its value is considered in the matching logic
            UNMASKED = 1
        ]
    ],
    PSMatch [
        /// Status match
        MATCH OFFSET(0) NUMBITS(32) []
    ],
    PInterval [
        /// Polling interval
        INTERVAL OFFSET(0) NUMBITS(32) []
    ],
    LPTimeout [
        /// Timeout period
        TIMEOUT OFFSET(0) NUMBITS(32) []
    ]
];

pub struct Qspi {
    registers: StaticRef<QspiRegisters>,
}

impl Qspi {
    pub const fn new() -> Qspi {
        Qspi {
            registers: QSPI_BASE,
        }
    }
}
