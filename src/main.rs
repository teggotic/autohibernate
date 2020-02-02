extern crate serde_yaml;

use std::thread;
use std::time::Duration;

use std::fs::{self, File};

use serde::{Deserialize, Serialize};

use std::process::Command;

use std::path::Path;

#[derive(Deserialize)]
struct Config {
    batteryInfoDirPath: String,
}

impl Config {
    fn clone(&self) -> Config {
        Config {
            batteryInfoDirPath: self.batteryInfoDirPath.clone(),
        }
    }
}

fn read_config() -> Config {
    let default_config = Config {
        batteryInfoDirPath: String::from("/sys/class/power_supply/BAT1"),
    };
    if let Ok(s) = fs::read_to_string(Path::new("/home/teggot/.config/autohibernate/config.yaml")) {
        let cfg: Config = match serde_yaml::from_str(&s) {
            Ok(val) => val,
            _ => default_config,
        };

        return cfg;
    } else {
        ()
    }

    default_config
}

fn main() {
    let cfg = read_config();

    let max_battery = {
        let contents =
            match fs::read_to_string(Path::new(&cfg.batteryInfoDirPath).join("energy_full")) {
                Ok(val) => val,
                Err(why) => panic!("couldn't read {}", why),
            };

        match contents.trim().parse::<i64>() {
            Ok(val) => val,
            Err(_) => panic!("couldn't parse \"{}\" to int", contents),
        }
    };

    loop {
        thread::sleep(Duration::from_secs(1));

        if let Ok(status) = fs::read_to_string(Path::new(&cfg.batteryInfoDirPath).join("status")) {
            let status = status.trim();
            if status != "Charging" {
                let cur_battery = match
                    fs::read_to_string(Path::new(&cfg.batteryInfoDirPath).join("energy_now"))
                {
                    Ok(contents) => match contents.trim().parse::<i64>() {
                        Ok(val) => val,
                        Err(_) => continue,
                    },
                    _ => continue
                };

                let percentage = cur_battery as f64 / max_battery as f64 * 100.;

                if percentage <= 4. {
                    Command::new("systemctl")
                        .arg("hibernate")
                        .output()
                        .expect("WTF");
                }

                // else if percentage <= 15 {
                //     Command::new("notify-send")
                //         .arg("-u")
                //         .arg("normal")
                //         .arg("\"Скоро отрублюсь\"")
                //         .output()
                //         .expect("WTF2");
                // }
            };
        };
    }
}
