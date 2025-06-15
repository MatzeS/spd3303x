# SPD3303X

Rust library for controlling the *Siglent SPD3303X* programmable power supply.

## Usage
```
let mut power_supply = Spd3303x::connect_hostname("<IP goes here>").await?;

// Double check we talk to the correct device.
power_supply
    .verify_serial_number("<your serial number>")
    .await?;

let (ch1, ch2, ch3) = power_supply.into_channels();

ch1.set_limit(LimitQuantity::Voltage, Reading::from(1.000)) .await?;
ch1.set_limit(LimitQuantity::Current, Reading::from(0.1)).await?;
ch1.set_output(State::On).await?;
```

## Limitations

Only TCP/IP is supported.
The USB interface is not (yet) implemented.

## Notes

This library implements the complete command set based on the official datasheet. See [`src/commands.rs`](src/commands.rs).  
Reference: [SPD3303X/3303X-E Programmable DC Power Supply, Quick Start, EN_02A (2025-06-13)](https://www.siglenteu.com/wp-content/uploads/dlm_uploads/2022/11/SPD3303X_QuickStart_E02A.pdf)

A convenient high-level programming interface is provided in [`src/spd3303x.rs`](src/spd3303x.rs) and [`src/channel_control.rs`](src/channel_control.rs).  
Refer to the API documentation for details: [docs.rs](https://docs.rs/spd3303x/latest)

The crate is currenlty on channel nightly for the 'pattern' feature.

## Reliability

Integration tests may sporadically fail due to "Connection reset" errors.  
This is likely caused by command overruns; apparently the device does not respond reliably to rapid sequences.  
If high reliability is required, consider adding rate limiting or retry logic.

Most commands are covered by integration tests.  
⚠️ **Disconnect any loads from the supply before running tests. The output will be activated with voltage and current!**

The instrument subsystem is not tested, as it is not used by the high-level interface.

This library has not been tested with the *SPD3303X-E* variant.

## Errata

The official documentation lacks details about the status response when the device is in *Series* channel operation mode.  
However, the device reliably advertises this mode via set bits 2 and 3 in the status response, and   
this implementation decodes those bits accordingly.
