pub const DEFAULT_T1: usize = 100;
pub const DEFAULT_T2: usize = 300;
pub const DEFAULT_N1: usize = 64;
pub const DEFAULT_N2: usize = 3;

pub const MAX_CONTROL_REQUEST_DATA: isize = 16;
pub const MAX_CHANNELS: isize = 5;

pub enum Address {
    CR = 0x02, // C/R: Command/Response bit
}

pub enum Extension {
    EA = 0x01, // EA: Extension bit
}

pub enum FrameType {
    // Table 2: Coding of Control Field
    SABM = 0x2f, // Set Asynchronous Balanced Mode
    UA = 0x63,   // Unnumbered Acknowledgement
    DM = 0x04,   // Disconnected Mode
    DISC = 0x43, // Disconnect
    UIH = 0xef,  // Unnumbered Information with Header check
    UI = 0x03,   // Unnumbered Information
    PF = 0x10,   // P/F: Poll/Final bit
}

pub enum ControlChannelCommand {
    // 5.4.6.3 Message Type and Actions
    PN = 0x80,    // 5.4.6.3.1  DLC parameter negotiation (PN)
    PSC = 0x40,   // 5.4.6.3.2  Power Saving Control (PSC)
    CLD = 0xc0,   // 5.4.6.3.3  Multiplexer close down (CLD)
    TEST = 0x20,  // 5.4.6.3.4  Test Command (Test)
    FCON = 0xa0,  // 5.4.6.3.5  Flow Control On Command (FCon)
    FCOFF = 0x60, // 5.4.6.3.6  Flow Control Off Command (FCoff)
    MSC = 0xe0,   // 5.4.6.3.7  Modem Status Command (MSC)
    NSC = 0x10,   // 5.4.6.3.8  Non Supported Command Response (NSC)
    RPN = 0x90,   // 5.4.6.3.9  Remote Port Negotiation Command (RPN)
    RLS = 0x50,   // 5.4.6.3.10 Remote Line Status Command (RLS)
    SNC = 0xd0,   // 5.4.6.3.11 Service Negotiation Command (SNC)
}

pub enum Frame {
    BASIC_FLAG = 0xf9,     // 5.2.6   Basic Option
    ADVANCED_FLAG = 0x7e,  // 5.2.7.4 Frame Structure
    CONTROL_ESCAPE = 0x7d, // 5.2.7.1 Control-octet transparency
    CONTROL_RESTORE = 0x20,
}

pub enum V24Signals {
    // Figure 10: Format of control signal octet
    // NOTE: >> 1
    FC = 0x01,  // Flow Control (FC)
    RTC = 0x02, // Ready To Communicate (RTC)
    RTR = 0x04, // Ready To Receive (RTR)
    IC = 0x20,  // Incoming call indicator (IC)
    DV = 0x40,  // Data Valid (DV)
}

pub enum SoftwareFlowControl {
    XON = 0x11,
    XOFF = 0x13,
}

pub enum ConvergenceLayer {
    UNSTRUCTURED = 1,                   // 5.5.1 Type 1 - Unstructured Octet Stream
    UNSTRUCTURED_WITH_FLOW_CONTROL = 2, // 5.5.2 Type 2 - Unstructured Octet Stream with flow control,
    // break signal handling and transmission of V.24 signal states
    UNINTERRUPTIBLE_FRAMED = 3, // 5.5.3 Type 3 - Uninterruptible Framed Data
    INTERRUPTIBLE_FRAMED = 4,   // 5.5.4 Type 4 - Interruptible Framed Data
}
