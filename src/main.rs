use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use serde::Deserialize;
use serde_yaml;
use std::collections::HashMap;
use std::io::Read;
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize)]
struct Config {
    keys: HashMap<String, String>,
}

fn print_left(d: std::time::Duration) -> String {
    let millis = d.subsec_millis();
    let secs = d.as_secs();
    let x = millis / 10;

    format!("{:02}.{:02} seconds", secs, x)
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

    let config: Config = serde_yaml::from_str(&contents).unwrap();

    let keyt = &args[1];
    if keyt == "watch" {
        let spinner_style = ProgressStyle::default_spinner()
            .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ")
            .template("{spinner} {prefix:.bold.dim} {wide_msg}");

        let m = MultiProgress::new();
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
