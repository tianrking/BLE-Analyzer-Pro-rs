use ble_analyzer_pro::capture::{run_capture, CaptureConfig};
use ble_analyzer_pro::device::find_devices;
use ble_analyzer_pro::packet::format_packet;
use clap::Parser;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

#[derive(Debug, Parser)]
#[command(author, version, about)]
struct Cli {
    #[arg(short = 'l', long)]
    list: bool,

    #[arg(short = 'v', long)]
    verbose: bool,

    #[arg(short = 'w', long = "write")]
    pcap: Option<PathBuf>,

    #[arg(short = 'p', long, default_value_t = 1)]
    phy: u8,

    #[arg(short = 'c', long, default_value_t = 0)]
    channel: u8,

    #[arg(long)]
    duration_ms: Option<u64>,

    #[arg(long)]
    max_packets: Option<u64>,

    #[arg(long)]
    quiet_init: bool,
}

fn main() {
    if let Err(err) = run() {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}

fn run() -> ble_analyzer_pro::Result<()> {
    let cli = Cli::parse();

    if cli.list {
        for dev in find_devices()? {
            println!(
                "bus={} addr={} vid={:04x} pid={:04x}",
                dev.bus, dev.address, dev.vendor_id, dev.product_id
            );
        }
        return Ok(());
    }

    if !cli.verbose && cli.pcap.is_none() {
        return Err(ble_analyzer_pro::Error::InvalidConfig(
            "use --verbose and/or --write FILE.pcap".to_string(),
        ));
    }

    let stop = Arc::new(AtomicBool::new(false));
    let stop_for_handler = Arc::clone(&stop);
    ctrlc::set_handler(move || {
        stop_for_handler.store(true, Ordering::SeqCst);
    })
    .map_err(|err| ble_analyzer_pro::Error::InvalidConfig(err.to_string()))?;

    let cfg = CaptureConfig {
        phy: cli.phy,
        channel: cli.channel,
        pcap_path: cli.pcap,
        duration: cli.duration_ms.map(Duration::from_millis),
        max_packets: cli.max_packets,
        log_device_init: !cli.quiet_init,
    };

    let mut handler = |packet: &ble_analyzer_pro::Packet| {
        if cli.verbose {
            println!("{}", format_packet(packet));
        }
        true
    };

    let report = run_capture(&cfg, &mut handler, &stop)?;
    eprintln!(
        "capture complete: {} packet(s), {} device(s), {:.3}s",
        report.total_packets,
        report.devices_opened,
        report.elapsed.as_secs_f64()
    );
    for dev in report.device_reports {
        eprintln!(
            "  bus={} addr={}: rx={} err={}",
            dev.bus, dev.address, dev.rx_count, dev.err_count
        );
    }
    Ok(())
}
