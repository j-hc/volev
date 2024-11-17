use crate::cprintf;
use evdev::Device;
pub use evdev::KeyCode;
use libc::{input_event, ioctl, timeval};
use std::{
    fs::{File, OpenOptions},
    io::{self, Read, Write},
    mem::size_of,
    os::fd::AsRawFd,
    path::Path,
};

const fn ioc(a: u32, b: u32, c: u32, d: u32) -> u32 {
    (a << 30) | (b << 8) | c | (d << 16)
}
const fn eviocgbit(ev: u32, len: u32) -> u32 {
    ioc(2, 'E' as u32, 0x20 + ev, len)
}
const fn _eviocgkey(len: u32) -> u32 {
    ioc(2, 'E' as u32, 0x18, len)
}
const EV_KEY: u32 = 0x01;

#[derive(Debug)]
pub struct EventHandler {
    device: File,
}

#[allow(dead_code)]
impl EventHandler {
    fn new(dir: impl AsRef<Path>) -> io::Result<Self> {
        let device = OpenOptions::new().write(true).read(true).open(&dir)?;
        Ok(Self { device })
    }

    pub fn get_input_event(&mut self) -> io::Result<input_event> {
        let mut buf = [0u8; size_of::<input_event>()];
        let r = self.device.read(&mut buf)?;
        assert_eq!(r, size_of::<input_event>());
        let ev: input_event = unsafe { std::mem::transmute(buf) };
        Ok(ev)
    }

    pub fn write_event(&mut self, type_: u16, code: u16, value: i32) -> io::Result<()> {
        let ev = input_event {
            time: timeval {
                tv_sec: 0,
                tv_usec: 0,
            },
            type_,
            code,
            value,
        };
        let ev: &[u8; size_of::<input_event>()] = unsafe { std::mem::transmute(&ev) };
        let r = self.device.write(ev)?;
        assert_eq!(r, size_of::<input_event>());
        Ok(())
    }

    // pub fn send_vol_down(&mut self) -> io::Result<()> {
    //     self.write_event(1, KEY_VOLUMEDOWN, 1)?;
    //     self.write_event(0, 0, 0)?;
    //     self.write_event(1, KEY_VOLUMEDOWN, 0)?;
    //     self.write_event(0, 0, 0)?;
    //     Ok(())
    // }

    fn get_possible_events(&self) -> Vec<usize> {
        let mut res;
        let mut bits = Vec::<u8>::new();
        let mut bits_size: usize = 0;
        loop {
            res = unsafe {
                ioctl(
                    self.device.as_raw_fd(),
                    eviocgbit(EV_KEY, bits_size as u32) as _,
                    bits.as_mut_ptr(),
                )
            };
            if (res as usize) < bits_size {
                break;
            }
            bits_size = res as usize + 16;
            bits.resize(bits_size * 2, 0);
        }

        let mut es = Vec::with_capacity(8);
        for (j, bit) in bits.iter().enumerate().take(res as usize) {
            for k in 0usize..8 {
                if (bit & 1 << k) != 0 {
                    let e = j * 8 + k;
                    es.push(e);
                }
            }
        }
        es
    }
}

pub fn get_vol_dev() -> io::Result<(Device, Device)> {
    let mut volu_dev: Option<Device> = None;
    let mut volu_dev_n = usize::MAX;
    let mut vold_dev: Option<Device> = None;
    let mut vold_dev_n = usize::MAX;
    for dir in std::fs::read_dir("/dev/input")?.flatten().filter_map(|d| {
        let p = d.path();
        p.file_name()
            .map(|fname| fname.to_str().unwrap().starts_with("event"))
            .and_then(|b| b.then_some(p))
    }) {
        let dev = Device::from_fd(OpenOptions::new().write(true).read(true).open(&dir)?.into())?;
        let keys = match dev.supported_keys() {
            Some(keys) => keys,
            None => continue,
        };
        if keys.contains(KeyCode::KEY_VOLUMEUP) {
            let keys_len = keys.iter().count();
            if keys_len < volu_dev_n {
                let dirstr = dir.to_str().unwrap();
                cprintf!(
                    "KEY_VOLUMEUP: %.*s\n",
                    dirstr.bytes().len(),
                    dirstr.as_ptr() as *const _
                );
                volu_dev = Some(dev);
                volu_dev_n = keys_len;
            }
        } else if keys.contains(KeyCode::KEY_VOLUMEDOWN) {
            let keys_len = keys.iter().count();
            if keys_len < vold_dev_n {
                let dirstr = dir.to_str().unwrap();
                cprintf!(
                    "KEY_VOLUMEDOWN: %.*s\n",
                    dirstr.bytes().len(),
                    dirstr.as_ptr() as *const _
                );
                vold_dev = Some(dev);
                vold_dev_n = keys_len;
            }
        }
    }
    Ok((vold_dev.unwrap(), volu_dev.unwrap()))
}
