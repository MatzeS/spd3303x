use std::{
    net::{Ipv4Addr, SocketAddr},
    sync::Arc,
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
    match_literal,
};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader, ReadHalf, WriteHalf},
    net::{TcpSocket, TcpStream, lookup_host},
    sync::Mutex,
};

use rs_usbtmc::UsbtmcClient;

pub struct NetworkDriver {
    reader: BufReader<ReadHalf<TcpStream>>,
    writer: WriteHalf<TcpStream>,
}

impl NetworkDriver {
    /// Looks up the address(es) for `host` and tries connecting to the device.
    /// Attempts all addresses,
    /// fails if connection could not be established on any address.
    pub async fn connect_hostname(host: &str) -> Result<Self> {
        let addresses = lookup_host(host).await?.collect::<Vec<_>>();
        if addresses.is_empty() {
            return Err(Error::ConnectFailed(format!(
                "Lookup provided no addresses for `{host}`"
            )));
        }

        for addr in addresses {
            let socket = TcpSocket::new_v4()?;
            let stream = socket.connect(addr).await;
            match stream {
                Ok(e) => return Ok(Self::new(e)),
                Err(_) => continue,
            }
        }

        Err(Error::ConnectFailed(
            "Could not connect on any address".to_string(),
        ))
    }

    pub async fn connect_address(addr: SocketAddr) -> Result<Self> {
        let socket = TcpSocket::new_v4()?;
        let stream = socket.connect(addr).await?;
        Ok(NetworkDriver::new(stream))
    }

    pub fn new(stream: TcpStream) -> Self {
        let (read_half, write_half) = tokio::io::split(stream);
        let reader = BufReader::new(read_half);

        NetworkDriver {
            reader,
            writer: write_half,
        }
    }
}

pub struct UsbDriver {
    device: rs_usbtmc::UsbtmcClient,
}

impl UsbDriver {
    /// Connect to the device
    /// siglent_vid: siglent device vendor ID (0xf4ec)
    /// siglent_pid: siglent device product ID (0x1430)
    pub fn connect_device(siglent_vid: u16, siglent_pid: u16) -> Result<Self> {
        match UsbtmcClient::connect((siglent_vid, siglent_pid)) {
            Ok(client) => {
                println!("Connected via USBTMC!");
                Ok(Self { device: client })
            }
            Err(e) => {
                eprintln!("USB connection failed: {e:?}");
                Err(Error::ConnectFailed("USB device not found".to_string()))
            }
        }
    }
}

pub trait Driver {
    fn send(&mut self, request: &str) -> impl Future<Output = Result<()>>;
    fn send_and_receive(&mut self, request: &str) -> impl Future<Output = Result<String>>;
}

impl Driver for NetworkDriver {
    async fn send(&mut self, request: &str) -> Result<()> {
        let mut request_with_newline = String::from(request);
        request_with_newline.push('\n');

        self.writer
            .write_all(request_with_newline.as_bytes())
            .await?;
        Ok(())
    }

    async fn send_and_receive(&mut self, request: &str) -> Result<String> {
        self.send(request).await?;

        let mut line = String::new();

        self.reader.read_line(&mut line).await?;
        Ok(line.trim_end().to_string())
    }
}

impl Driver for UsbDriver {
    async fn send(&mut self, request: &str) -> Result<()> {
        self.device.command(request).unwrap(); // TODO unwrap
        Ok(())
    }

    async fn send_and_receive(&mut self, request: &str) -> Result<String> {
        let response = self.device.query(request).map_err(|e| {
            eprintln!("Failed to send query: {e:?}");
            Error::Other("Send query failed".to_string())
        })?;

        Ok(response)
    }
}

pub struct Spd3303x<D: Driver> {
    pub driver: D,
}

impl<D: Driver> Spd3303x<D> {
    pub async fn verify_serial_number(&mut self, serial_number: &str) -> Result<()> {
        let device_serial_number = self.get_identity().await?.serial_number;

        device_serial_number
            .eq(serial_number)
            .then_some(Ok(()))
            .ok_or(Error::SerialMismatch(format!(
                "Device has serial number: {device_serial_number}"
            )))?
    }

    pub fn into_channels(self) -> (ChannelControl<D>, ChannelControl<D>, FixedChannelControl<D>) {
        let spd = Arc::new(Mutex::new(self));
        (
            ChannelControl::new(spd.clone(), Channel::One),
            ChannelControl::new(spd.clone(), Channel::Two),
            FixedChannelControl::new(spd, OutputChannel::Three),
        )
    }

    async fn send<Request>(&mut self, request: Request) -> Result<()>
    where
        Request: ScpiRequest<Response = EmptyResponse>,
    {
        let mut out = String::with_capacity(128);
        request.serialize(&mut out);
        out.push('\n');
        self.driver.send(out.as_str()).await
    }

    async fn execute<Request, Response>(&mut self, request: Request) -> Result<Response>
    where
        Request: ScpiRequest<Response = Response>,
        Response: ScpiDeserialize,
    {
        // TODO copy pasted
        let mut out = String::with_capacity(128);
        request.serialize(&mut out);

        let line = self.driver.send_and_receive(out.as_str()).await?;
        let mut data = line.as_str();
        let response = Response::deserialize(&mut data)?;
        check_empty(&mut data)?;

        Ok(response)
    }

    pub async fn get_identity(&mut self) -> Result<IdentityResponse> {
        self.execute(IdentityRequest).await
    }

    pub async fn save(&mut self, slot: MemorySlot) -> Result<()> {
        self.send(SaveRequest { slot }).await
    }

    pub async fn recall(&mut self, slot: MemorySlot) -> Result<()> {
        self.send(RecallRequest { slot }).await
    }

    pub async fn get_selected_channel(&mut self) -> Result<Channel> {
        self.execute(GetInstrumentRequest).await.map(|e| e.channel)
    }

    pub async fn measure(&mut self, channel: Channel, quantity: Quantity) -> Result<f32> {
        let response = self
            .execute(MeasureRequest {
                quantity,
                channel: Some(channel),
            })
            .await?;
        Ok(response.0.into())
    }

    pub async fn set_limit(
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
        .await
    }

    pub async fn get_limit(&mut self, channel: Channel, quantity: LimitQuantity) -> Result<f32> {
        let response = self
            .execute(GetLimitRequest {
                quantity,
                channel: Some(channel),
            })
            .await?;
        Ok(response.0.into())
    }

    pub async fn set_output(&mut self, channel: OutputChannel, state: State) -> Result<()> {
        self.send(SetOutputStateRequest { channel, state }).await
    }

    pub async fn set_output_mode(&mut self, mode: OperationMode) -> Result<()> {
        self.send(SetOperationModeRequest { mode }).await
    }

    pub async fn set_waveform_display(&mut self, channel: Channel, state: State) -> Result<()> {
        self.send(WaveformDisplayRequest { channel, state }).await
    }

    pub async fn set_timing_parameters(
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
        })
        .await?;
        Ok(())
    }

    pub async fn get_timing_parameters(
        &mut self,
        channel: Channel,
        group: TimingGroup,
    ) -> Result<GetTimingParametersResponse> {
        self.execute(GetTimingParametersRequest { channel, group })
            .await
    }

    pub async fn set_timer(&mut self, channel: Channel, state: State) -> Result<()> {
        self.send(SetTimerStateRequest { channel, state }).await
    }

    pub async fn get_error(&mut self) -> Result<SystemErrorResponse> {
        self.execute(SystemErrorRequest).await
    }

    pub async fn get_version(&mut self) -> Result<SystemVersionResponse> {
        self.execute(SystemVersionRequest).await
    }

    pub async fn get_status(&mut self) -> Result<SystemStatus> {
        self.execute(SystemStatusRequest).await.map(|e| e.decode())
    }

    pub async fn set_ip_address(&mut self, addr: Ipv4Addr) -> Result<()> {
        self.send(SetIpAddressRequest { addr }).await
    }

    pub async fn get_ip_address(&mut self) -> Result<Ipv4Addr> {
        self.execute(GetIpAddressRequest).await.map(|e| e.address)
    }

    pub async fn set_subnet_mask(&mut self, mask: Ipv4Addr) -> Result<()> {
        self.send(SetSubnetMaskRequest { mask }).await
    }

    pub async fn get_subnet_mask(&mut self) -> Result<Ipv4Addr> {
        self.execute(GetSubnetMaskRequest).await.map(|e| e.mask)
    }

    pub async fn set_gateway(&mut self, gateway: Ipv4Addr) -> Result<()> {
        self.send(SetGatewayRequest { gateway }).await
    }

    pub async fn get_gateway(&mut self) -> Result<Ipv4Addr> {
        self.execute(GetGatewayRequest).await.map(|e| e.gateway)
    }

    pub async fn set_dhcp(&mut self, state: State) -> Result<()> {
        self.send(SetDhcpRequest { state }).await
    }

    pub async fn get_dhcp(&mut self) -> Result<State> {
        self.execute(GetDhcpRequest).await.map(|e| e.state)
    }

    pub async fn get_output(&mut self, channel: Channel) -> Result<State> {
        let status = self.get_status().await?;
        Ok(status.get(channel).output)
    }
}
