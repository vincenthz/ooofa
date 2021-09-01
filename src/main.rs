use chrono::offset::Local;
use chrono::DateTime;
use console::Style;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use serde::Deserialize;
use serde_yaml;
use std::collections::BTreeMap;
use std::io::Read;
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize)]
struct Config {
    keys: BTreeMap<String, String>,
}

fn print_left(d: std::time::Duration) -> String {
    let millis = d.subsec_millis();
    let secs = d.as_secs();
    let x = millis / 10;

    format!("{:02}.{:02} seconds", secs, x)
}

fn print_time(time: std::time::SystemTime) -> String {
    let datetime: DateTime<Local> = time.into();
    format!("{}", datetime.format("%H:%M:%S"))
}

fn main() {
    let args = std::env::args().collect::<Vec<_>>();

    #[allow(deprecated)]
    let home = std::env::home_dir().unwrap();
    let mut config = PathBuf::from(home);
    config.push(".ooofa.yaml");

    let mut contents = String::new();
    let mut file = std::fs::File::open(config).unwrap();
    file.read_to_string(&mut contents).unwrap();

    // read the config
    let config: Config = serde_yaml::from_str(&contents).unwrap();

    // from the config, create all the keys
    let keys: BTreeMap<_, _> = config
        .keys
        .iter()
        .map(|(k, v)| {
            let otp = aotp::OTP::from_url(
                &url::Url::parse(&v).expect(&format!("key for '{}' is not a valid url", k)),
            )
            .expect(&format!("key for '{}' is not a valid otp construction", k));
            (k, otp)
        })
        .collect();

    let keyt = &args[1];
    if keyt == "watch" {
        let spinner_style = ProgressStyle::default_spinner()
            .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ")
            .template("{spinner} {prefix:.bold.dim} {wide_msg}");

        let m = MultiProgress::new();

        // sort the keys per type of period
        // and create multiple progress bar if necessary:
        // one per period category
        // and one per keys in those periods
        let periods = [aotp::Period::seconds30()];
        let mut keys_per_period = BTreeMap::new();
        for period in periods {
            let mut out = BTreeMap::new();
            for (name, key) in keys.iter() {
                if key.period != period {
                    continue;
                }
                out.insert(name, key.clone());
            }
            if !out.is_empty() {
                let period_progress_bar = m.add(ProgressBar::new(1));
                period_progress_bar.set_style(spinner_style.clone());
                period_progress_bar.set_prefix(format!("{:?}  : ", period));
                let mut key_entries = Vec::new();
                for (name, k) in out.into_iter() {
                    let key_bar = m.add(ProgressBar::new(1));
                    key_bar.set_style(spinner_style.clone());
                    key_bar.set_prefix(format!("{:10}  : ", name));
                    key_entries.push((k, key_bar))
                }
                keys_per_period.insert(period, (period_progress_bar, key_entries));
            }
        }

        std::thread::spawn(move || loop {
            let cyan = Style::new().cyan();
            let green = Style::new().green();
            let red = Style::new().red();

            let threshold = std::time::Duration::new(5, 0);

            let mut previous_counter = aotp::Counter::zero();
            let mut threshold_change = false;
            let sleep = std::time::Duration::from_millis(620);
            for (period, (period_bar, keys)) in keys_per_period.iter() {
                let (counter, left) = aotp::Counter::totp_now_left((*period).into());
                let next_counter = counter.incr();
                let current_start = counter.system_time((*period).into());
                let next_start = next_counter.system_time((*period).into());

                // set the color for looming stuff
                let looming = left <= threshold;
                let color = if looming { &red } else { &cyan };

                let threshold_trigger = threshold_change ^ looming;
                threshold_change = looming;

                period_bar.set_message(format!(
                    "current-period: {:?} -- started-at: {} -- left: {} -- next-at: {}",
                    counter,
                    green.apply_to(print_time(current_start)),
                    color.apply_to(print_left(left)),
                    green.apply_to(print_time(next_start)),
                ));

                // check if the counter has changed, if it has not, then we just update the period bar
                if counter == previous_counter && !threshold_trigger {
                    //
                } else {
                    for (otp, key_bar) in keys.iter() {
                        let token = otp.totp_at(counter);
                        let token_next = otp.totp_at(next_counter);
                        key_bar.set_message(format!(
                            " current: {}      | next: {}",
                            color.apply_to(token.dec6()),
                            //print_left(left),
                            green.apply_to(token_next.dec6())
                        ));
                    }
                    previous_counter = counter;
                }
            }
            std::thread::sleep(sleep);
        });

        /*
        for (k, v) in config.keys {
            let pb = m.add(ProgressBar::new(1));
            pb.set_style(spinner_style.clone());
            pb.set_prefix(format!("{:10}  : ", k));
            let otp = aotp::OTP::from_url(&url::Url::parse(&v).unwrap()).unwrap();

            let b = std::time::Duration::from_millis(430);

            let _ = std::thread::spawn(move || loop {
                let (ctr, left) = aotp::Counter::totp_now_left(otp.period.into());
                let token = otp.totp_at(ctr);
                let token_next = otp.totp_at(ctr.incr());
                let sleep = if left < b { left } else { b };
                pb.set_message(format!(
                    "{}      {}   --- {}",
                    token.dec6(),
                    print_left(left),
                    token_next.dec6()
                ));
                std::thread::sleep(sleep);
            });
        }
        */
        m.join().unwrap();
    } else {
        let x = config.keys.keys().find(|k| k.starts_with(keyt));

        match x {
            Some(x) => {
                let x = config.keys.get(x).unwrap();
                let otp = aotp::OTP::from_url(&url::Url::parse(x).unwrap()).unwrap();
                let (token, left) = otp.totp_now();
                println!("{}", token.dec6());
                eprintln!("{:?}", left);

                //println!("{} ({:?})", token.dec6(), left)
            }
            None => {
                println!("missing key")
            }
        }
    }
}
