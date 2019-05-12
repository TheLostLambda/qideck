extern crate clap;
extern crate evdev_rs as evdev;
extern crate rand;
extern crate serialport;

use clap::{App, Arg};
use evdev::enums::*;
use evdev::*;
use rand::Rng;
use serialport::prelude::*;
use std::fs::File;
use std::io;
use std::process::Command;
use std::str::from_utf8;
use std::thread::sleep;
use std::time::{Duration, Instant};

fn press_keys(keys: &Vec<EV_KEY>, dev: &UInputDevice) {
    let t = TimeVal::new(0, 0);
    let syn = vec![InputEvent::new(
        &t,
        &EventCode::EV_SYN(EV_SYN::SYN_REPORT),
        0,
    )];
    let events = |val| {
        keys.iter()
            .map(|key| InputEvent::new(&t, &EventCode::EV_KEY(key.clone()), val))
            .collect::<Vec<InputEvent>>()
    };
    for e in vec![events(1), syn.clone(), events(0), syn]
        .into_iter()
        .flatten()
    {
        dev.write_event(&e).expect("Scheiße");
    }
}

fn memes(dev: &UInputDevice) {
    let power = 15;
    let t = Duration::from_millis(100);
    let mut rng = rand::thread_rng();
    for _ in 0..power {
        vol_up(&dev);
        if rng.gen() {
            up_space(&dev)
        } else {
            down_space(&dev)
        }
        sleep(t);
    }
    play_pause(&dev);
}

fn down_space(dev: &UInputDevice) {
    press_keys(
        &vec![EV_KEY::KEY_LEFTCTRL, EV_KEY::KEY_LEFTALT, EV_KEY::KEY_DOWN],
        &dev,
    );
}

fn up_space(dev: &UInputDevice) {
    press_keys(
        &vec![EV_KEY::KEY_LEFTCTRL, EV_KEY::KEY_LEFTALT, EV_KEY::KEY_UP],
        &dev,
    );
}

fn play_pause(dev: &UInputDevice) {
    press_keys(&vec![EV_KEY::KEY_PLAYPAUSE], &dev);
}

fn vol_up(dev: &UInputDevice) {
    press_keys(&vec![EV_KEY::KEY_VOLUMEUP], &dev);
}

fn vol_down(dev: &UInputDevice) {
    press_keys(&vec![EV_KEY::KEY_VOLUMEDOWN], &dev);
}

fn launch_terminal(dev: &UInputDevice) {
    press_keys(
        &vec![EV_KEY::KEY_LEFTCTRL, EV_KEY::KEY_LEFTALT, EV_KEY::KEY_T],
        &dev,
    );
}

fn lights_on() {
    Command::new("HueWheel.sh").spawn().unwrap();
}

fn lights_off() {
    Command::new("Blackout.sh").spawn().unwrap();
}

fn main() {
    let args = App::new("QiDeck")
        .version("1.0.0")
        .author("Brooks J Rady <b.j.rady@gmail.com>")
        .about("Userspace driver and macro configuration tool for the QiDeck")
        .arg(
            Arg::with_name("keyboard")
                .index(1)
                .value_name("/dev/input/eventX")
                .help("Points the program to a keyboard device to emulate"),
        )
        .arg(
            Arg::with_name("serial")
                .index(2)
                .value_name("/dev/ttyUSBX")
                .help("Points the program to the serial port of the QiDeck"),
        )
        .get_matches();
    let path = args.value_of("keyboard").unwrap_or("/dev/input/event2");
    let port = args.value_of("serial").unwrap_or("/dev/ttyUSB0");
    let baud = 115200;

    println!("Cloning: {}", path);
    let f = File::open(path).unwrap();
    let d = Device::new_from_fd(f).unwrap();
    let i = UInputDevice::create_from_device(&d).unwrap();

    let mut settings: SerialPortSettings = Default::default();
    settings.timeout = Duration::from_millis(10);
    settings.baud_rate = baud;

    sleep(Duration::from_millis(10));

    match serialport::open_with_settings(&port, &settings) {
        Ok(mut p) => {
            let mut serial_buf: Vec<u8> = vec![0; 3];
            println!("Receiving data on {} at {} baud:", &port, &baud);
            let mut tick = Instant::now();
            loop {
                match p.read(serial_buf.as_mut_slice()) {
                    Ok(n) if tick.elapsed().as_millis() >= 200 => {
                        let digit_str = from_utf8(&serial_buf[..n]).unwrap().trim();
                        if digit_str.is_empty() {
                            continue;
                        }
                        match digit_str.parse::<u8>() {
                            Ok(1) => up_space(&i),
                            Ok(2) => vol_up(&i),
                            Ok(3) => lights_on(),
                            Ok(4) => play_pause(&i),
                            Ok(5) => memes(&i),
                            Ok(6) => launch_terminal(&i),
                            Ok(7) => down_space(&i),
                            Ok(8) => vol_down(&i),
                            Ok(9) => lights_off(),
                            _ => println!("{:?}", digit_str),
                        };
                        tick = Instant::now();
                    }
                    Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
                    Err(e) => eprintln!("{:?}", e),
                    _ => (),
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to open \"{}\". Error: {}", port, e);
            ::std::process::exit(1);
        }
    }
}
