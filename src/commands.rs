use std::{net::Ipv4Addr, ops::Neg};

use crate::{
    EmptyResponse, Error, ScpiDeserialize, ScpiSerialize, impl_scpi_request, impl_scpi_serialize,
    match_literal, read_all, read_until, read_while, scpi_enum,
};

// 1. *IDN?
// Command format *IDN?
// Description Query the manufacturer, product type, series No., software version and hardware version
pub struct IdentityRequest;
impl_scpi_serialize!(IdentityRequest, ["*IDN?"]);

// Return Info Manufacturer, product type, series No., software version, hardware version
// Typical Return Siglent Technologies, SPD3303X, SPD00001130025, 1.01.01.01.02,V3.0
#[derive(Debug, Clone)]
pub struct IdentityResponse {
    pub company_name: String,
    pub model_number: String,
    pub serial_number: String,
    pub software_version: String,
    pub hardware_version: String,
}

impl ScpiDeserialize for IdentityResponse {
    fn deserialize(input: &mut &str) -> Result<Self, Error> {
        Ok(IdentityResponse {
            company_name: read_until(input, ',')?.trim().to_string(),
            model_number: read_until(input, ',')?.trim().to_string(),
            serial_number: read_until(input, ',')?.trim().to_string(),
            software_version: read_until(input, ',')?.trim().to_string(),
            hardware_version: read_until(input, '\n')?.trim().to_string(),
        })
    }
}

impl_scpi_request!(IdentityRequest, IdentityResponse);

// 2. *SAV
// Command format: *SAV {1|2|3|4|5}
// Description: Save current state in nonvolatile memory
// Example: *SAV 1

scpi_enum! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum MemorySlot {
        One => "1",
        Two => "2",
        Three => "3",
        Four => "4",
        Five => "5",
    }
}

impl TryFrom<u8> for MemorySlot {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(MemorySlot::One),
            2 => Ok(MemorySlot::Two),
            3 => Ok(MemorySlot::Three),
            4 => Ok(MemorySlot::Four),
            5 => Ok(MemorySlot::Five),
            _ => Err(()),
        }
    }
}

pub struct SaveRequest {
    pub slot: MemorySlot,
}
impl_scpi_serialize!(SaveRequest, ["*SAV ", slot]);
impl_scpi_request!(SaveRequest, EmptyResponse);

// 3. *RCL
// Command format *RCL {1|2|3|4|5}
// Description Recall state that had been saved from nonvolatile memory.
// Example *RCL 1
pub struct RecallRequest {
    pub slot: MemorySlot,
}
impl_scpi_serialize!(RecallRequest, ["*RCL ", slot]);
impl_scpi_request!(RecallRequest, EmptyResponse);

// 4. INSTrument
// Command format INSTrument {CH1|CH2}
// Description Select the channel that will be operated.
// Example INSTrument CH1

scpi_enum! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum Channel {
        One => "CH1",
        Two => "CH2",
    }
}

impl TryFrom<u8> for Channel {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Channel::One),
            2 => Ok(Channel::Two),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SetInstrumentRequest {
    pub channel: Channel,
}
impl_scpi_serialize!(SetInstrumentRequest, ["INSTrument ", channel]);
impl_scpi_request!(SetInstrumentRequest, EmptyResponse);

// Command format INSTrument?
// Description Query the current operating channel
// Example INSTrument?
// Typical Return CH1
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GetInstrumentRequest;
impl_scpi_serialize!(GetInstrumentRequest, ["INSTrument?"]);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GetInstrumentResponse {
    pub channel: Channel,
}

impl ScpiDeserialize for GetInstrumentResponse {
    fn deserialize(input: &mut &str) -> Result<Self, Error> {
        Ok(GetInstrumentResponse {
            channel: Channel::deserialize(input)?,
        })
    }
}

impl_scpi_request!(GetInstrumentRequest, GetInstrumentResponse);

// 5. MEASure
// Command format MEASure:CURRent? [{CH1|CH2}]
// Description Query current value for specified channel, if there is no specified channel,
// query the current channel
// Example MEASure:CURRent? CH1
// Typical Return 3.000
// Command format MEASure:VOLTage? [{CH1|CH2}]
// Description Query voltage value for specified channel, if there is no specified channel,
// query the current channel
// Example MEASure:VOLTage? CH1
// Typical Return 30.000
// Command format MEASure:POWEr? [{CH1|CH2}]
// Description Query power value for specified channel, if there is no specified channel,
// query the current channel.
// Example MEASure:POWEr? CH1
// Typical Return 90.000

scpi_enum! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum Quantity {
        Current => "CURRent",
        Voltage => "VOLTage",
        Power => "POWEr",
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MeasureRequest {
    pub quantity: Quantity,
    pub channel: Option<Channel>,
}
impl_scpi_serialize!(MeasureRequest, ["MEASure:", quantity, "? ", channel]);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MeasureResponse(pub Reading);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Reading {
    millis: u16,
}

impl Reading {
    pub fn from_millis(millis: u16) -> Reading {
        Reading { millis }
    }

    pub fn get_millis(&self) -> u16 {
        self.millis
    }
}

impl From<Reading> for f64 {
    fn from(value: Reading) -> Self {
        f64::from(value.get_millis()) / 1000.0
    }
}

impl From<Reading> for f32 {
    fn from(value: Reading) -> Self {
        f32::from(value.get_millis()) / 1000.0
    }
}

impl From<f32> for Reading {
    fn from(value: f32) -> Self {
        Reading::from_millis((value * 1000.0).round().max(0.0) as u16)
    }
}

impl ScpiSerialize for Reading {
    fn serialize(&self, out: &mut String) {
        let whole = self.millis / 1000;
        let frac = self.millis % 1000;
        use std::fmt::Write;
        write!(out, "{whole}.{frac:03}").expect("Failed to format number");
    }
}

impl ScpiDeserialize for Reading {
    fn deserialize(input: &mut &str) -> Result<Self, Error> {
        let whole_part = u16::deserialize(input)?;
        match_literal(input, ".")?;
        let frac_part = u16::deserialize(input)?;
        Ok(Reading::from_millis(whole_part * 1000 + frac_part))
    }
}

impl ScpiDeserialize for MeasureResponse {
    fn deserialize(input: &mut &str) -> Result<Self, Error> {
        let value = Reading::deserialize(input)?;
        match_literal(input, "\n")?;
        Ok(MeasureResponse(value))
    }
}

impl_scpi_request!(MeasureRequest, MeasureResponse);

// 6. CURRent
// Command format [{CH1|CH2}:]CURRent <current>
// Description Set current value of the selected channel
// Example CH1:CURRent 0.5
// Command format [{CH1|CH2}:]CURRent?
// Description Query the current value of the selected channel.
// Example CH1:CURRent?
// Typical Return 0.500
// 7. VOLTage
// Command format [{CH1|CH2}:]VOLTage <voltage>
// Description Set voltage value of the selected channel
// Example CH1:VOLTage 25
// Command format [{CH1|CH2}:]VOLTage?
// Description Query the voltage value of the selected channel.
// Example CH1:VOLTage?
// Typical Return 25.000

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LimitQuantity {
    Current,
    Voltage,
}
impl From<&LimitQuantity> for Quantity {
    fn from(value: &LimitQuantity) -> Self {
        match value {
            LimitQuantity::Current => Quantity::Current,
            LimitQuantity::Voltage => Quantity::Voltage,
        }
    }
}

impl ScpiSerialize for LimitQuantity {
    fn serialize(&self, out: &mut String) {
        Quantity::from(self).serialize(out);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SetLimitRequest {
    pub quantity: LimitQuantity,
    pub value: Reading,
    pub channel: Option<Channel>,
}
impl ScpiSerialize for SetLimitRequest {
    fn serialize(&self, out: &mut String) {
        self.channel.serialize(out);
        if self.channel.is_some() {
            out.push(':');
        }
        self.quantity.serialize(out);
        out.push(' ');
        self.value.serialize(out);
    }
}
impl_scpi_request!(SetLimitRequest, EmptyResponse);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GetLimitRequest {
    pub quantity: LimitQuantity,
    pub channel: Option<Channel>,
}
impl ScpiSerialize for GetLimitRequest {
    fn serialize(&self, out: &mut String) {
        self.channel.serialize(out);
        if self.channel.is_some() {
            out.push(':');
        }
        self.quantity.serialize(out);
        out.push('?');
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GetLimitResponse(pub Reading);
impl ScpiDeserialize for GetLimitResponse {
    fn deserialize(input: &mut &str) -> Result<Self, Error> {
        let value = Reading::deserialize(input)?;
        match_literal(input, "\n")?;
        Ok(GetLimitResponse(value))
    }
}

impl_scpi_request!(GetLimitRequest, GetLimitResponse);

// 8. OUTPut
// Command format OUTPut {CH1|CH2|CH3},{ON|OFF}
// Description Turn on/off the specified channel output.
// Example OUTPut CH1,ON

scpi_enum! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum State {
        On => "ON",
        Off => "OFF",
    }
}

impl From<bool> for State {
    fn from(value: bool) -> Self {
        match value {
            true => State::On,
            false => State::Off,
        }
    }
}

impl From<State> for bool {
    fn from(value: State) -> Self {
        match value {
            State::On => true,
            State::Off => false,
        }
    }
}

impl Neg for State {
    type Output = State;

    fn neg(self) -> Self::Output {
        match self {
            State::On => State::Off,
            State::Off => State::On,
        }
    }
}

scpi_enum! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum OutputChannel {
        One => "CH1",
        Two => "CH2",
        Three => "CH3",
    }
}

impl TryFrom<OutputChannel> for Channel {
    type Error = ();

    fn try_from(value: OutputChannel) -> Result<Self, Self::Error> {
        match value {
            OutputChannel::One => Ok(Channel::One),
            OutputChannel::Two => Ok(Channel::Two),
            OutputChannel::Three => Err(()),
        }
    }
}

impl From<Channel> for OutputChannel {
    fn from(value: Channel) -> Self {
        match value {
            Channel::One => OutputChannel::One,
            Channel::Two => OutputChannel::Two,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SetOutputStateRequest {
    pub channel: OutputChannel,
    pub state: State,
}
impl_scpi_serialize!(SetOutputStateRequest, ["OUTPut ", channel, ",", state]);
impl_scpi_request!(SetOutputStateRequest, EmptyResponse);

// Command format OUTPut:TRACK {0|1|2}
// Description Select operation mode. Parameters {0|1|2} mean independent, series and
// parallel respectively
// Example OUTPut:TRACK 0

scpi_enum! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum OperationMode {
        Independent => "0",
        Series => "1",
        Parallel => "2",
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SetOperationModeRequest {
    pub mode: OperationMode,
}

impl_scpi_serialize!(SetOperationModeRequest, ["OUTPut:TRACK ", mode]);
impl_scpi_request!(SetOperationModeRequest, EmptyResponse);

// Command format OUTPut:WAVE {CH1|CH2},{ON|OFF}
// Description Turn on/off the Waveform Display function of specified channel
// Example OUTPut:WAVE CH1,ON

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WaveformDisplayRequest {
    pub channel: Channel,
    pub state: State,
}
impl_scpi_serialize!(
    WaveformDisplayRequest,
    ["OUTPut:WAVE ", channel, ",", state]
);
impl_scpi_request!(WaveformDisplayRequest, EmptyResponse);

// 9. TIMEr
// Command format TIMEr:SET
// {CH1|CH2},{1|2|3|4|5},<voltage>,<current>,<time>
// Description Set timing parameters of specified channel, including group{1|2|3|4|5}
// voltage, current, time
// Example TIMEr:SET CH1,2,3,0.5,2

scpi_enum! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum TimingGroup {
        One => "1",
        Two => "2",
        Three => "3",
        Four => "4",
        Five => "5",
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimeInterval(u16);

impl From<u16> for TimeInterval {
    fn from(value: u16) -> Self {
        if value > 10000 {
            panic!("Time interval value {value} exceeds accepted range for SPD3303X (max. 10000)");
        }
        TimeInterval(value)
    }
}

impl ScpiSerialize for TimeInterval {
    fn serialize(&self, out: &mut String) {
        use std::fmt::Write;
        write!(out, "{}", self.0).expect("Failed to format number");
    }
}

impl ScpiDeserialize for TimeInterval {
    fn deserialize(input: &mut &str) -> crate::Result<Self> {
        Ok(TimeInterval(u16::deserialize(input)?))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SetTimingParametersRequest {
    pub channel: Channel,
    pub group: TimingGroup,
    pub voltage: Reading,
    pub current: Reading,
    pub time: TimeInterval,
}
impl_scpi_serialize!(
    SetTimingParametersRequest,
    [
        "TIMEr:SET ",
        channel,
        ",",
        group,
        ",",
        voltage,
        ",",
        current,
        ",",
        time
    ]
);
impl_scpi_request!(SetTimingParametersRequest, EmptyResponse);

// Command format TIMEr:SET? {CH1|CH2},{1|2|3|4|5};
// Description Query the voltage/current/time parameters of specified group of specified
// channel
// Example TIMEr:SET? CH1,2
// Typical Return 3,0.5,2
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GetTimingParametersRequest {
    pub channel: Channel,
    pub group: TimingGroup,
}

impl_scpi_serialize!(
    GetTimingParametersRequest,
    ["TIMEr:SET? ", channel, ",", group]
);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GetTimingParametersResponse {
    pub voltage: Reading,
    pub current: Reading,
    pub time: Reading,
}

impl ScpiDeserialize for GetTimingParametersResponse {
    fn deserialize(input: &mut &str) -> Result<Self, Error> {
        let voltage = Reading::deserialize(input)?;
        match_literal(input, ",")?;
        let current = Reading::deserialize(input)?;
        match_literal(input, ",")?;
        let time = Reading::deserialize(input)?;

        Ok(GetTimingParametersResponse {
            voltage,
            current,
            time,
        })
    }
}

impl_scpi_request!(GetTimingParametersRequest, GetTimingParametersResponse);

// Command format TIMEr {CH1|CH2},{ON|OFF};
// Description Turn on/off Timer function of specified channel
// Example TIMEr CH1,ON
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SetTimerStateRequest {
    pub channel: Channel,
    pub state: State,
}
impl_scpi_serialize!(SetTimerStateRequest, ["TIMEr ", channel, ",", state]);
impl_scpi_request!(SetTimerStateRequest, EmptyResponse);

// 10. SYSTem
// Command format SYSTem:ERRor?
// Description Query the error code and the information of the equipment.
// Typical Return 0 No Error
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SystemErrorRequest;
impl_scpi_serialize!(SystemErrorRequest, ["SYSTem:ERRor?"]);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SystemErrorResponse {
    pub content: String,
}

impl ScpiDeserialize for SystemErrorResponse {
    fn deserialize(input: &mut &str) -> Result<Self, Error> {
        Ok(SystemErrorResponse {
            content: read_all(input)?,
        })
    }
}

impl_scpi_request!(SystemErrorRequest, SystemErrorResponse);

// Command format SYSTem:VERSion?
// Description Query the software version of the equipment
// Typical Return 1.01.01.01.02
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SystemVersionRequest;
impl_scpi_serialize!(SystemVersionRequest, ["SYSTem:VERSion?"]);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SystemVersionResponse {
    pub version: String,
}

impl ScpiDeserialize for SystemVersionResponse {
    fn deserialize(input: &mut &str) -> Result<Self, Error> {
        Ok(SystemVersionResponse {
            version: input.to_string(),
        })
    }
}

impl_scpi_request!(SystemVersionRequest, SystemVersionResponse);

// Command format SYSTem:STATus?
// Description Query the current working state of the equipment.
// Typical Return 0x0224
// Note The return info is hexadecimal format, but the actual state is binary, so you
// must change the return info into a binary. The state correspondence
// relationship is as follow.
// Bit No Corresponding state
// 0 0: CH1 CV mode; 1: CH1 CC mode
// 1 0: CH2 CV mode; 1: CH2 CC mode
// 2,3 01: Independent mode; 10: Parallel mode
// 4 0: CH1 OFF; 1: CH1 ON
// 5 0: CH2 OFF; 1: CH2 ON
// 6 0: TIMER1 OFF; 1: TIMER1 ON
// 7 0: TIMER2 OFF; 1: TIMER2 ON
// 8 0: CH1 digital display; 1: CH1 waveform display
// 9 0: CH2 digital display; 1: CH2 waveform display
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SystemStatusRequest;
impl_scpi_serialize!(SystemStatusRequest, ["SYSTem:STATus?"]);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChannelMode {
    ConstantVoltage,
    ConstantCurrent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayMode {
    DigitalDisplay,
    WaveformDisplay,
}

impl From<bool> for DisplayMode {
    fn from(value: bool) -> Self {
        match value {
            true => DisplayMode::WaveformDisplay,
            false => DisplayMode::DigitalDisplay,
        }
    }
}

impl From<bool> for ChannelMode {
    fn from(value: bool) -> Self {
        match value {
            true => ChannelMode::ConstantCurrent,
            false => ChannelMode::ConstantVoltage,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChannelStatus {
    pub mode: ChannelMode,
    pub output: State,
    pub timer: State,
    pub display: DisplayMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SystemStatus {
    pub operation_mode: OperationMode,
    pub channel_one: ChannelStatus,
    pub channel_two: ChannelStatus,
}

impl SystemStatus {
    pub fn get(&self, channel: Channel) -> &ChannelStatus {
        match channel {
            Channel::One => &self.channel_one,
            Channel::Two => &self.channel_two,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SystemStatusResponse {
    pub value: u16,
}
impl SystemStatusResponse {
    const CHANNEL_1_MODE_BIT: usize = 0;
    const CHANNEL_2_MODE_BIT: usize = 1;
    // const OPERATION_MODE_BITS: (usize, usize) = (2, 3);
    const CHANNEL_1_OUTPUT_STATE: usize = 4;
    const CHANNEL_2_OUTPUT_STATE: usize = 5;
    const TIMER_1_STATE: usize = 6;
    const TIMER_2_STATE: usize = 7;
    const CHANNEL_1_DISPLAY: usize = 8;
    const CHANNEL_2_DISPLAY: usize = 9;

    fn read_bit(&self, bit: usize) -> bool {
        let mask = 1 << bit;
        (mask & self.value) > 0
    }

    fn decode_operation_mode(&self) -> OperationMode {
        match self.value & 0b1100 {
            0b0100 => OperationMode::Independent,
            0b1000 => OperationMode::Parallel,
            0b1100 => OperationMode::Series,
            0b0000 => panic!("Received unexpected, invalid bit pattern `00` for operation mode!"),
            _ => unreachable!(),
        }
    }

    pub fn decode(&self) -> SystemStatus {
        SystemStatus {
            operation_mode: self.decode_operation_mode(),
            channel_one: ChannelStatus {
                mode: self.read_bit(Self::CHANNEL_1_MODE_BIT).into(),
                output: self.read_bit(Self::CHANNEL_1_OUTPUT_STATE).into(),
                timer: self.read_bit(Self::TIMER_1_STATE).into(),
                display: self.read_bit(Self::CHANNEL_1_DISPLAY).into(),
            },
            channel_two: ChannelStatus {
                mode: self.read_bit(Self::CHANNEL_2_MODE_BIT).into(),
                output: self.read_bit(Self::CHANNEL_2_OUTPUT_STATE).into(),
                timer: self.read_bit(Self::TIMER_2_STATE).into(),
                display: self.read_bit(Self::CHANNEL_2_DISPLAY).into(),
            },
        }
    }
}

impl ScpiDeserialize for SystemStatusResponse {
    fn deserialize(input: &mut &str) -> Result<Self, Error> {
        match_literal(input, "0x")?;
        let value =
            u16::from_str_radix(read_while(input, |c: char| char::is_ascii_hexdigit(&c)), 16)
                .map_err(|e| Error::ResponseDecoding(format!("Failed to parse hex: {e}")))?;
        match_literal(input, "\n")?;
        Ok(SystemStatusResponse { value })
    }
}

impl_scpi_request!(SystemStatusRequest, SystemStatusResponse);

// 11. IPaddr
// Command format IPaddr <IP address>
// Description Assign a static Internet Protocol (IP) address for the instrument
// Example IPaddr 10.11.13.214
// Note The command is invalid when the state of DHCP is on
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SetIpAddressRequest {
    pub addr: Ipv4Addr,
}
impl_scpi_serialize!(SetIpAddressRequest, ["IPaddr ", addr]);
impl_scpi_request!(SetIpAddressRequest, EmptyResponse);

impl ScpiSerialize for Ipv4Addr {
    fn serialize(&self, out: &mut String) {
        let result = format!(
            "{}.{}.{}.{}",
            self.octets()[0],
            self.octets()[1],
            self.octets()[2],
            self.octets()[3]
        );
        out.push_str(result.as_str());
    }
}

impl ScpiDeserialize for Ipv4Addr {
    fn deserialize(input: &mut &str) -> Result<Self, Error> {
        let address = input
            .trim()
            .parse()
            .map_err(|e| Error::ResponseDecoding(format!("Failed to parse IPv4 Address: {e}")))?;

        // advance input reader
        read_while(input, char::is_numeric);
        match_literal(input, ".")?;
        read_while(input, char::is_numeric);
        match_literal(input, ".")?;
        read_while(input, char::is_numeric);
        match_literal(input, ".")?;
        read_while(input, char::is_numeric);

        Ok(address)
    }
}

// Command format IPaddr?
// Description Query the current IP address of the instrument
// Typical Return 10.11.13.214
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GetIpAddressRequest;
impl_scpi_serialize!(GetIpAddressRequest, ["IPaddr?"]);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GetIpAddressResponse {
    pub address: Ipv4Addr,
}
impl ScpiDeserialize for GetIpAddressResponse {
    fn deserialize(input: &mut &str) -> Result<Self, Error> {
        let address = Ipv4Addr::deserialize(input)?;
        match_literal(input, "\n")?;
        Ok(GetIpAddressResponse { address })
    }
}

impl_scpi_request!(GetIpAddressRequest, GetIpAddressResponse);

// 12. MASKaddr
// Command format MASKaddr <NetMasK>
// Description Assign a subnet mask for the instrument
// Example MASKadd 255.255.255.0
// Note The command is invalid when the state of DHCP is on
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SetSubnetMaskRequest {
    pub mask: Ipv4Addr,
}
impl_scpi_serialize!(SetSubnetMaskRequest, ["MASKaddr ", mask]);
impl_scpi_request!(SetSubnetMaskRequest, EmptyResponse);

// Command format MASKaddr?
// Description Query the current subnet mask of the instrument
// Typical Return 255.255.255.0
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GetSubnetMaskRequest;
impl_scpi_serialize!(GetSubnetMaskRequest, ["MASKaddr?"]);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GetSubnetMaskResponse {
    pub mask: Ipv4Addr,
}
impl ScpiDeserialize for GetSubnetMaskResponse {
    fn deserialize(input: &mut &str) -> Result<Self, Error> {
        let mask = Ipv4Addr::deserialize(input)?;
        match_literal(input, "\n")?;
        Ok(GetSubnetMaskResponse { mask })
    }
}

impl_scpi_request!(GetSubnetMaskRequest, GetSubnetMaskResponse);

// 13. GATEaddr
// Command format GATEaddr <GateWay>
// Description Assign a gateway for the instrument
// Example GATEaddr 10.11.13.1
// Note The command is invalid when the state of DHCP is on
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SetGatewayRequest {
    pub gateway: Ipv4Addr,
}
impl_scpi_serialize!(SetGatewayRequest, ["GATEaddr ", gateway]);
impl_scpi_request!(SetGatewayRequest, EmptyResponse);

// Command format GATEaddr?
// Description Query the current gateway of the instrument
// Typical Return 10.11.13.1
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GetGatewayRequest;
impl_scpi_serialize!(GetGatewayRequest, ["GATEaddr?"]);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GetGatewayResponse {
    pub gateway: Ipv4Addr,
}
impl ScpiDeserialize for GetGatewayResponse {
    fn deserialize(input: &mut &str) -> Result<Self, Error> {
        let gateway = Ipv4Addr::deserialize(input)?;
        match_literal(input, "\n")?;
        Ok(GetGatewayResponse { gateway })
    }
}

impl_scpi_request!(GetGatewayRequest, GetGatewayResponse);

// 14. DHCP
// Command format DHCP {ON|OFF}
// Description Assign the network parameters (such as the IP address) for the instrument
// automatically.
// Example DHCP ON
pub struct SetDhcpRequest {
    pub state: State,
}
impl_scpi_serialize!(SetDhcpRequest, ["DHCP ", state]);
impl_scpi_request!(SetDhcpRequest, EmptyResponse);

// Command format DHCP?
// Description Query whether the automatic network parameters configuration function is
// turn on
// Typical Return DHCP:ON
pub struct GetDhcpRequest;
impl_scpi_serialize!(GetDhcpRequest, ["DHCP?"]);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GetDhcpResponse {
    pub state: State,
}

impl ScpiDeserialize for GetDhcpResponse {
    fn deserialize(input: &mut &str) -> Result<Self, Error> {
        match_literal(input, "DHCP:")?;
        let state = State::deserialize(input)?;
        match_literal(input, "\n")?;
        Ok(GetDhcpResponse { state })
    }
}

impl_scpi_request!(GetDhcpRequest, GetDhcpResponse);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_idn() {
        let request = IdentityRequest;
        let mut out = String::new();
        request.serialize(&mut out);
        assert_eq!(out, "*IDN?");
        let response = &mut "Siglent Technologies, SPD3303X, SPD00001130025, 1.01.01.01.02,V3.0\n";
        let response = IdentityResponse::deserialize(response).unwrap();
        assert_eq!(response.company_name, "Siglent Technologies");
        assert_eq!(response.model_number, "SPD3303X");
        assert_eq!(response.serial_number, "SPD00001130025");
        assert_eq!(response.software_version, "1.01.01.01.02");
        assert_eq!(response.hardware_version, "V3.0");
    }
}

// impl ScpiDeserialize MACRO
