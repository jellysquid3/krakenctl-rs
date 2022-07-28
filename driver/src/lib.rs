use hidapi::{DeviceInfo, HidApi, HidDevice};

pub mod devices;

pub struct DeviceFilter {
    vendor: u16,
    product: u16,
}

impl DeviceFilter {
    fn matches(&self, desc: &DeviceInfo) -> bool {
        desc.vendor_id() == self.vendor && desc.product_id() == self.product
    }
}

pub trait KrakenUsbDriver<D> {
    fn create(device: HidDevice) -> Box<D>;

    fn filters() -> &'static [DeviceFilter];
}

pub trait KrakenUsbDevice {
    fn set_fixed_speed(&mut self, duty: u8);
}

pub fn find_device() -> Option<Box<dyn KrakenUsbDevice>> {
    let api = HidApi::new()
        .expect("Failed to initialize HidApi");

    let mut devices = api.device_list();
    devices
        .find_map(|info| find_driver_for_device(&api, info))
}

fn find_driver_for_device(api: &HidApi, info: &DeviceInfo) -> Option<Box<dyn KrakenUsbDevice>> {
    if cfg!(feature = "nzxt_kraken") {
        return try_init_driver::<devices::kraken3::KrakenV3Device>(api, info);
    }

    None
}

fn try_init_driver<T>(api: &HidApi, device_info: &DeviceInfo) -> Option<Box<dyn KrakenUsbDevice>>
    where T: KrakenUsbDriver<T> + KrakenUsbDevice + 'static
{
    for filter in T::filters() {
        if filter.matches(device_info) {
            let device = device_info
                .open_device(&api)
                .expect("Failed to open device");

            return Some(T::create(device));
        }
    }

    None
}