use icp_distribution::Distribution;
use std::{env, str::FromStr};

fn main() {
    println!("cargo:rerun-if-env-changed=DISTRIBUTION");

    if let Some(distribution) = env::var("DISTRIBUTION").ok() {
        if let Err(e) = Distribution::from_str(&distribution) {
            panic!(
                "❌ ERROR: Invalid DISTRIBUTION value '{}': {}",
                distribution, e
            );
        }
        println!("cargo:rustc-env=DISTRIBUTION={}", distribution);
    } else {
        println!("cargo:rustc-env=DISTRIBUTION=standard");
    }
}
