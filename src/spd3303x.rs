use std::{
    io::{BufRead, BufReader, Write},
    net::{Ipv4Addr, SocketAddr, TcpStream, ToSocketAddrs},
    sync::{Arc, Mutex},
    time::Duration,
};

use crate::{
    EmptyResponse, Error, Result, ScpiDeserialize, ScpiRequest,
    channel_control::ChannelControl,
    check_empty,
    commands::{
        Channel, GetDhcpRequest, GetGatewayRequest, GetInstrumentRequest, GetIpAddressRequest,
        GetLimitRequest, GetSubnetMaskRequest, GetTimingParametersRequest,
        GetTimingParametersResponse, IdentityRequest, IdentityResponse, LimitQuantity,
        MeasureRequest, MemorySlot, OperationMode, OutputChannel, Quantity, Reading, RecallRequest,
        SaveRequest, SetDhcpRequest, SetGatewayRequest, SetIpAddressRequest, SetLimitRequest,
        SetOperationModeRequest, SetOutputStateRequest, SetSubnetMaskRequest, SetTimerStateRequest,
        SetTimingParametersRequest, State, SystemErrorRequest, SystemErrorResponse, SystemStatus,
        SystemStatusRequest, SystemVersionRequest, SystemVersionResponse, TimeInterval,
        TimingGroup, WaveformDisplayRequest,
    },
    fixed_channel_control::FixedChannelControl,
};
use anyhow::anyhow;
pub struct Spd3303x {
    stream: TcpStream,
}

impl Spd3303x {
    /// Looks up the address(es) for `host` and tries connecting to the device.
    /// Attempts all addresses,
    /// fails if connection could not be established on any address.
    pub fn connect_hostname(host: &str) -> Result<Self> {
        let (hostname, port_str) = host
            .rsplit_once(':')
            .ok_or_else(|| anyhow!("Missing ':' separator in host string"))?;
        let port = port_str
            .parse::<u16>()
            .map_err(|_| Error::Other("Invalid port".to_string()))?;

        let addresses = (hostname, port).to_socket_addrs()?.collect::<Vec<_>>();
        if addresses.is_empty() {
            return Err(Error::ConnectFailed(format!(
                "Lookup provided no addresses for `{host}`"
            )));
        }

        for addr in addresses {
            if let Ok(stream) = TcpStream::connect(addr) {
                stream.set_read_timeout(Some(Duration::from_secs(2))).ok();
                stream.set_write_timeout(Some(Duration::from_secs(2))).ok();
                return Ok(Self::new(stream));
            }
        }

        Err(Error::ConnectFailed(
            "Could not connect on any address".to_string(),
        ))
    }

    pub fn connect_address(addr: SocketAddr) -> Result<Self> {
        Ok(Self::new(TcpStream::connect(addr)?))
    }

    pub fn new(stream: TcpStream) -> Self {
        Spd3303x { stream }
    }

    pub fn verify_serial_number(&mut self, serial_number: &str) -> Result<()> {
        let device_serial_number = self.get_identity()?.serial_number;

        device_serial_number
            .eq(serial_number)
            .then_some(Ok(()))
            .ok_or(Error::SerialMismatch(format!(
                "Device has serial number: {device_serial_number}"
            )))?
    }

    pub fn into_channels(self) -> (ChannelControl, ChannelControl, FixedChannelControl) {
        let spd = Arc::new(Mutex::new(self));
        (
            ChannelControl::new(spd.clone(), Channel::One),
            ChannelControl::new(spd.clone(), Channel::Two),
            FixedChannelControl::new(spd, OutputChannel::Three),
        )
    }

    fn send_raw<Request>(&mut self, request: Request) -> Result<()>
    where
        Request: ScpiRequest,
    {
        let mut out = String::with_capacity(128);
        request.serialize(&mut out);
        out.push('\n');
        self.stream.write_all(out.as_bytes())?;
        self.stream.flush()?;

        Ok(())
    }

    fn send<Request>(&mut self, request: Request) -> Result<()>
    where
        Request: ScpiRequest<Response = EmptyResponse>,
    {
        self.send_raw(request)
    }
    fn execute<Request, Response>(&mut self, request: Request) -> Result<Response>
    where
        Request: ScpiRequest<Response = Response>,
        Response: ScpiDeserialize,
    {
        self.send_raw(request)?;

        let mut reader = BufReader::new(&mut self.stream);
        let mut line = String::new();
        reader.read_line(&mut line)?;

        let mut data = line.as_str();
        let response = Response::deserialize(&mut data)?;
        check_empty(&mut data)?;

        Ok(response)
    }

    pub fn get_identity(&mut self) -> Result<IdentityResponse> {
        self.execute(IdentityRequest)
    }

    pub fn save(&mut self, slot: MemorySlot) -> Result<()> {
        self.send(SaveRequest { slot })
    }

    pub fn recall(&mut self, slot: MemorySlot) -> Result<()> {
        self.send(RecallRequest { slot })
    }

    pub fn get_selected_channel(&mut self) -> Result<Channel> {
        self.execute(GetInstrumentRequest).map(|e| e.channel)
    }

    pub fn measure(&mut self, channel: Channel, quantity: Quantity) -> Result<f32> {
        let response = self.execute(MeasureRequest {
            quantity,
            channel: Some(channel),
        })?;
        Ok(response.0.into())
    }

    pub fn set_limit(
        &mut self,
        channel: Channel,
        quantity: LimitQuantity,
        value: Reading,
    ) -> Result<()> {
        self.send(SetLimitRequest {
            quantity,
            value,
            channel: Some(channel),
        })
    }

    pub fn get_limit(&mut self, channel: Channel, quantity: LimitQuantity) -> Result<f32> {
        let response = self.execute(GetLimitRequest {
            quantity,
            channel: Some(channel),
        })?;
        Ok(response.0.into())
    }

    pub fn set_output(&mut self, channel: OutputChannel, state: State) -> Result<()> {
        self.send(SetOutputStateRequest { channel, state })
    }

    pub fn set_output_mode(&mut self, mode: OperationMode) -> Result<()> {
        self.send(SetOperationModeRequest { mode })
    }

    pub fn set_waveform_display(&mut self, channel: Channel, state: State) -> Result<()> {
        self.send(WaveformDisplayRequest { channel, state })
    }

    pub fn set_timing_parameters(
        &mut self,
        channel: Channel,
        group: TimingGroup,
        voltage: Reading,
        current: Reading,
        time: TimeInterval,
    ) -> Result<()> {
        self.send(SetTimingParametersRequest {
            channel,
            group,
            voltage,
            current,
            time,
        })?;
        Ok(())
    }

    pub fn get_timing_parameters(
        &mut self,
        channel: Channel,
        group: TimingGroup,
    ) -> Result<GetTimingParametersResponse> {
        self.execute(GetTimingParametersRequest { channel, group })
    }

    pub fn set_timer(&mut self, channel: Channel, state: State) -> Result<()> {
        self.send(SetTimerStateRequest { channel, state })
    }

    pub fn get_error(&mut self) -> Result<SystemErrorResponse> {
        self.execute(SystemErrorRequest)
    }

    pub fn get_version(&mut self) -> Result<SystemVersionResponse> {
        self.execute(SystemVersionRequest)
    }

    pub fn get_status(&mut self) -> Result<SystemStatus> {
        self.execute(SystemStatusRequest).map(|e| e.decode())
    }

    pub fn set_ip_address(&mut self, addr: Ipv4Addr) -> Result<()> {
        self.send(SetIpAddressRequest { addr })
    }

    pub fn get_ip_address(&mut self) -> Result<Ipv4Addr> {
        self.execute(GetIpAddressRequest).map(|e| e.address)
    }

    pub fn set_subnet_mask(&mut self, mask: Ipv4Addr) -> Result<()> {
        self.send(SetSubnetMaskRequest { mask })
    }

    pub fn get_subnet_mask(&mut self) -> Result<Ipv4Addr> {
        self.execute(GetSubnetMaskRequest).map(|e| e.mask)
    }

    pub fn set_gateway(&mut self, gateway: Ipv4Addr) -> Result<()> {
        self.send(SetGatewayRequest { gateway })
    }

    pub fn get_gateway(&mut self) -> Result<Ipv4Addr> {
        self.execute(GetGatewayRequest).map(|e| e.gateway)
    }

    pub fn set_dhcp(&mut self, state: State) -> Result<()> {
        self.send(SetDhcpRequest { state })
    }

    pub fn get_dhcp(&mut self) -> Result<State> {
        self.execute(GetDhcpRequest).map(|e| e.state)
    }

    pub fn get_output(&mut self, channel: Channel) -> Result<State> {
        let status = self.get_status()?;
        Ok(status.get(channel).output)
    }
}
