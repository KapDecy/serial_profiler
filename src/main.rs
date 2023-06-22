use std::io::{self, BufRead, BufReader};
use std::process::exit;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use colored::{Color, Colorize};

#[derive(Default, Debug)]
struct Mesurs {
    inst: Option<Instant>,
    imu_count: u32,
    imu_all: Vec<Duration>,
    imu_sum: Duration,
    imu_min: Option<Duration>,
    imu_min_count: u32,
    imu_max: Option<Duration>,
    imu_max_count: u32,
    gps_count: u32,
    gps_all: Vec<Duration>,
    gps_sum: Duration,
    gps_min: Option<Duration>,
    gps_min_count: u32,
    gps_max: Option<Duration>,
    gps_max_count: u32,
}

fn main() {
    let port = serialport::new("/dev/ttyUSB0", 115200)
        .timeout(Duration::from_millis(10))
        .open()
        .unwrap();

    let mut port = BufReader::new(port);
    let mut colors = [
        Color::Blue,
        Color::Red,
        Color::Yellow,
        Color::Green,
        Color::White,
        Color::Cyan,
        Color::Magenta,
    ]
    .iter()
    .cloned()
    .cycle();

    let mesurs = Arc::new(Mutex::new(Mesurs::default()));
    {
        mesurs.lock().unwrap().inst = Some(Instant::now());
    }
    let mcl = mesurs.clone();

    ctrlc::set_handler(move || {
        let mut m = mcl.lock().unwrap();
        println!("");
        println!("full time: {}s", m.inst.unwrap().elapsed().as_secs_f32());
        println!("");
        println!("imu count: {}", m.imu_count);
        println!("imu avr: {}ms", m.imu_sum.as_millis() / m.imu_count as u128);
        m.imu_all.sort();
        println!("imu med: {}ms", m.imu_all[m.imu_all.len() / 2].as_millis());
        println!("imu min: {}ms, at {}", m.imu_min.unwrap().as_millis(), m.imu_min_count);
        println!("imu max: {}ms, at {}", m.imu_max.unwrap().as_millis(), m.imu_max_count);
        println!("");
        println!("gps count: {}", m.gps_count);
        println!("gps avr: {}ms", m.gps_sum.as_millis() / m.gps_count as u128);
        m.gps_all.sort();
        println!("gps med: {}ms", m.gps_all[m.gps_all.len() / 2].as_millis());
        println!("gps min: {}ms, at {}", m.gps_min.unwrap().as_millis(), m.gps_min_count);
        println!("gps max: {}ms, at {}", m.gps_max.unwrap().as_millis(), m.gps_max_count);
        exit(0);
    })
    .unwrap();

    let mut imu = Instant::now();
    let mut gps = Instant::now();

    loop {
        let mut v = vec![];
        match port.read_until('\t' as u8, &mut v) {
            Ok(_t) => {
                let mut m = mesurs.lock().unwrap();
                let s = String::from_utf8_lossy(v.as_slice());
                let ss = s.trim();
                println!("{}", ss.color(colors.next().unwrap()));

                if ss.starts_with("imu") {
                    let imud = imu.elapsed();
                    m.imu_all.push(imud);
                    m.imu_sum += imud;
                    m.imu_count += 1;
                    m.imu_min = if m.imu_count < 3 {
                        m.imu_min
                    } else {
                        match m.imu_min {
                            Some(dur) => {
                                // Some(dur.min(imud));
                                if imud < dur {
                                    m.imu_min_count = m.imu_count;
                                    Some(imud)
                                } else {
                                    Some(dur)
                                }
                            }
                            None => Some(imud),
                        }
                    };
                    m.imu_max = match m.imu_max {
                        Some(dur) => {
                            // Some(dur.min(imud));
                            if imud > dur {
                                m.imu_max_count = m.imu_count;
                                Some(imud)
                            } else {
                                Some(dur)
                            }
                        }
                        None => Some(imud),
                    };
                    imu = Instant::now();
                } else {
                    let gpsd = gps.elapsed();
                    m.gps_sum += gpsd;
                    m.gps_all.push(gpsd);
                    m.gps_count += 1;
                    m.gps_min = if m.gps_count < 3 {
                        m.gps_min
                    } else {
                        match m.gps_min {
                            Some(dur) => {
                                // Some(dur.min(imud));
                                if gpsd < dur {
                                    m.gps_min_count = m.gps_count;
                                    Some(gpsd)
                                } else {
                                    Some(dur)
                                }
                            }
                            None => Some(gpsd),
                        }
                    };
                    m.gps_max = match m.gps_max {
                        Some(dur) => {
                            // Some(dur.min(imud));
                            if gpsd > dur {
                                m.gps_max_count = m.gps_count;
                                Some(gpsd)
                            } else {
                                Some(dur)
                            }
                        }
                        None => Some(gpsd),
                    };
                    gps = Instant::now();
                }
            }
            Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
            Err(e) => eprintln!("{:?}", e),
        }
    }
}
