use std::{
    io::{BufRead, BufReader, Write},
    net::{Ipv4Addr, SocketAddr, TcpStream, ToSocketAddrs},
    sync::Arc,
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
use rs_usbtmc::UsbtmcClient;
pub struct NetworkDriver {
    stream: TcpStream,
}

impl NetworkDriver {
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
                return Ok(Self { stream });
            }
        }

        Err(Error::ConnectFailed(
            "Could not connect on any address".to_string(),
        ))
    }

    pub fn connect_address(addr: SocketAddr) -> Result<Self> {
        let stream = TcpStream::connect(addr)?;
        Ok(Self { stream })
    }
}

pub struct UsbDriver {
    device: UsbtmcClient,
}

impl UsbDriver {
    pub fn connect_device() -> Result<Self> {
        const VID: u16 = 0xf4ec;
        const PID: u16 = 0x1430;
        let client = UsbtmcClient::connect((VID, PID))?;
        Ok(Self { device: client })
    }
}

pub trait Driver {
    fn send(&mut self, request: &str) -> Result<()>;
    fn send_and_receive(&mut self, request: &str) -> Result<String>;
}

impl Driver for NetworkDriver {
    fn send(&mut self, request: &str) -> Result<()> {
        let mut request_with_newline = String::from(request);
        request_with_newline.push('\n');

        self.stream.write_all(request_with_newline.as_bytes())?;
        self.stream.flush()?;
        Ok(())
    }

    fn send_and_receive(&mut self, request: &str) -> Result<String> {
        self.send(request)?;

        let mut reader = BufReader::new(&mut self.stream);
        let mut line = String::new();

        reader.read_line(&mut line)?;
        Ok(line.trim_end().to_string())
    }
}

impl Driver for UsbDriver {
    fn send(&mut self, request: &str) -> Result<()> {
        self.device.command(request)?;
        Ok(())
    }

    fn send_and_receive(&mut self, request: &str) -> Result<String> {
        let response = self.device.query(request)?;
        Ok(response)
    }
}

pub struct Spd3303x<D: Driver> {
    pub driver: D,
}

impl<D: Driver> Spd3303x<D> {
    pub fn verify_serial_number(&mut self, serial_number: &str) -> Result<()> {
        let device_serial_number = self.get_identity()?.serial_number;

        device_serial_number
            .eq(serial_number)
            .then_some(Ok(()))
            .ok_or(Error::SerialMismatch(format!(
                "Device has serial number: {device_serial_number}"
            )))?
    }

    pub fn into_channels(self) -> (ChannelControl<D>, ChannelControl<D>, FixedChannelControl<D>) {
        let spd = Arc::new(std::sync::Mutex::new(self));
        (
            ChannelControl::new(spd.clone(), Channel::One),
            ChannelControl::new(spd.clone(), Channel::Two),
            FixedChannelControl::new(spd, OutputChannel::Three),
        )
    }

    fn send<Request>(&mut self, request: Request) -> Result<()>
    where
        Request: ScpiRequest<Response = EmptyResponse>,
    {
        let mut out = String::with_capacity(128);
        request.serialize(&mut out);
        out.push('\n');
        self.driver.send(out.as_str())
    }

    fn execute<Request, Response>(&mut self, request: Request) -> Result<Response>
    where
        Request: ScpiRequest<Response = Response>,
        Response: ScpiDeserialize,
    {
        // TODO copy pasted
        let mut out = String::with_capacity(128);
        request.serialize(&mut out);

        let line = self.driver.send_and_receive(out.as_str())?;
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
