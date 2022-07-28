use hidapi::HidDevice;
use zerocopy::AsBytes;

use crate::{KrakenUsbDriver, DeviceFilter, KrakenUsbDevice};

const HEADER_SIZE: usize = 4;
const TABLE_SIZE: usize = 40;

pub struct KrakenV3Device {
    pub device: HidDevice,
}

impl KrakenUsbDriver<KrakenV3Device> for KrakenV3Device {
    fn create(device: HidDevice) -> Box<KrakenV3Device> {
        device.write(&[0x10, 0x01]).unwrap();
        device.write(&[0x20, 0x03]).unwrap();

        Box::new(KrakenV3Device { device })
    }

    fn filters() -> &'static [DeviceFilter] {
        &[DeviceFilter {
            vendor: 0x1e71,
            product: 0x2007,
        }]
    }
}

impl KrakenUsbDevice for KrakenV3Device {
    fn set_fixed_speed(&mut self, duty: u8) {
        let channel = Channel {
            id: 0x1,
            min_duty: 20,
            max_duty: 100,
        };

        let duty = duty.clamp(channel.min_duty, channel.max_duty);

        #[derive(AsBytes)]
        #[repr(packed)]
        #[allow(dead_code)]
        struct Payload {
            header: [u8; HEADER_SIZE],
            table: [u8; TABLE_SIZE],
        }

        let payload = Payload {
            header: [0x72, channel.id, 0x00, 0x00],
            table: create_fixed_speed_table(duty),
        };

        self.device
            .write(payload.as_bytes())
            .expect("failed to write payload");
    }
}

fn create_fixed_speed_table(duty: u8) -> [u8; TABLE_SIZE] {
    let mut table = [duty; TABLE_SIZE];
    table[TABLE_SIZE - 1] = 100;
    table
}

struct Channel {
    id: u8,
    min_duty: u8,
    max_duty: u8,
}
