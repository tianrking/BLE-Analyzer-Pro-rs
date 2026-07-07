use ble_analyzer_pro::capture::{run_capture, CaptureConfig};
use ble_analyzer_pro::device::find_devices;
use ble_analyzer_pro::discovery::{Candidate, DiscoverySort, DiscoveryTable};
use ble_analyzer_pro::packet::format_packet;
use ble_analyzer_pro::{normalize_mac, parse_mac};
use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

#[derive(Debug, Parser)]
#[command(author, version, about)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

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

    #[arg(long, value_name = "ADDR")]
    filter_addr: Option<String>,

    #[arg(long)]
    quiet_init: bool,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Rank BLE advertisers and track RSSI changes while moving a target.
    Discover(DiscoverCli),
}

#[derive(Debug, Parser)]
struct DiscoverCli {
    #[arg(long, default_value_t = 15000)]
    duration_ms: u64,

    #[arg(long, value_name = "ADDR")]
    target: Option<String>,

    #[arg(long, value_enum, default_value_t = DiscoverSortArg::RssiChange)]
    sort: DiscoverSortArg,

    #[arg(long, default_value_t = 25)]
    limit: usize,

    #[arg(long, default_value_t = 12)]
    significant_db: i16,

    #[arg(long, default_value_t = 1000)]
    update_ms: u64,

    #[arg(short = 'w', long = "write")]
    pcap: Option<PathBuf>,

    #[arg(short = 'p', long, default_value_t = 1)]
    phy: u8,

    #[arg(short = 'c', long, default_value_t = 0)]
    channel: u8,

    #[arg(long)]
    quiet_init: bool,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum DiscoverSortArg {
    RssiChange,
    Strongest,
    Packets,
    Name,
    Manufacturer,
    Kind,
}

impl From<DiscoverSortArg> for DiscoverySort {
    fn from(value: DiscoverSortArg) -> Self {
        match value {
            DiscoverSortArg::RssiChange => DiscoverySort::RssiChange,
            DiscoverSortArg::Strongest => DiscoverySort::Strongest,
            DiscoverSortArg::Packets => DiscoverySort::Packets,
            DiscoverSortArg::Name => DiscoverySort::Name,
            DiscoverSortArg::Manufacturer => DiscoverySort::Manufacturer,
            DiscoverSortArg::Kind => DiscoverySort::Kind,
        }
    }
}

fn main() {
    if let Err(err) = run() {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}

fn run() -> ble_analyzer_pro::Result<()> {
    let cli = Cli::parse();

    if let Some(command) = cli.command {
        return match command {
            Commands::Discover(args) => run_discover(args),
        };
    }

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

    let filter_addr = match cli.filter_addr.as_deref() {
        Some(addr) => Some(parse_mac(addr).map_err(ble_analyzer_pro::Error::InvalidConfig)?),
        None => None,
    };

    let stop = install_ctrlc_handler()?;

    let cfg = CaptureConfig {
        phy: cli.phy,
        channel: cli.channel,
        pcap_path: cli.pcap,
        duration: cli.duration_ms.map(Duration::from_millis),
        max_packets: cli.max_packets,
        filter_addr,
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

fn run_discover(args: DiscoverCli) -> ble_analyzer_pro::Result<()> {
    let target_addr = match args.target.as_deref() {
        Some(addr) => Some(normalize_mac(addr).map_err(ble_analyzer_pro::Error::InvalidConfig)?),
        None => None,
    };
    let filter_addr = match target_addr.as_deref() {
        Some(addr) => Some(parse_mac(addr).map_err(ble_analyzer_pro::Error::InvalidConfig)?),
        None => None,
    };

    let stop = install_ctrlc_handler()?;
    let duration = if args.duration_ms == 0 {
        None
    } else {
        Some(Duration::from_millis(args.duration_ms))
    };
    let cfg = CaptureConfig {
        phy: args.phy,
        channel: args.channel,
        pcap_path: args.pcap,
        duration,
        max_packets: None,
        filter_addr,
        log_device_init: !args.quiet_init,
    };

    let mut table = DiscoveryTable::default();
    let report_every = Duration::from_millis(args.update_ms.max(100));
    let mut last_report_at: Option<Instant> = None;
    let started = Instant::now();

    if let Some(addr) = target_addr.as_deref() {
        eprintln!(
            "tracking {addr}; move the target near/far and watch RSSI delta (Ctrl+C to stop)"
        );
    } else {
        eprintln!(
            "discovering BLE advertisers for {} ms; move candidate devices near/far to create RSSI deltas",
            args.duration_ms
        );
    }

    let mut handler = |packet: &ble_analyzer_pro::Packet| {
        let candidate = table.update(packet);
        if target_addr.is_some() && last_report_at.is_none_or(|last| last.elapsed() >= report_every)
        {
            print_target_update(candidate, started.elapsed(), args.significant_db);
            last_report_at = Some(Instant::now());
        }
        true
    };

    let report = run_capture(&cfg, &mut handler, &stop)?;
    eprintln!(
        "discovery complete: {} matching packet(s), {} candidate(s), {:.3}s",
        report.total_packets,
        table.len(),
        report.elapsed.as_secs_f64()
    );

    if let Some(addr) = target_addr.as_deref() {
        match table.get(addr) {
            Some(candidate) => {
                println!();
                println!("Final target summary:");
                print_target_update(candidate, report.elapsed, args.significant_db);
            }
            None => {
                println!(
                    "No packets matched {addr}. Re-check the address or scan without --target."
                );
            }
        }
    } else {
        print_discovery_table(&table, args.sort.into(), args.limit, args.significant_db);
    }

    Ok(())
}

fn install_ctrlc_handler() -> ble_analyzer_pro::Result<Arc<AtomicBool>> {
    let stop = Arc::new(AtomicBool::new(false));
    let stop_for_handler = Arc::clone(&stop);
    ctrlc::set_handler(move || {
        stop_for_handler.store(true, Ordering::SeqCst);
    })
    .map_err(|err| ble_analyzer_pro::Error::InvalidConfig(err.to_string()))?;
    Ok(stop)
}

fn print_discovery_table(
    table: &DiscoveryTable,
    sort: DiscoverySort,
    limit: usize,
    significant_db: i16,
) {
    if table.is_empty() {
        println!("No BLE advertisers were observed.");
        return;
    }

    println!(
        "{:<4} {:<17} {:<12} {:>6} {:>5} {:>6} {:>5} {:>5} {:>5} {:<24} {:<18} {:<22} {:<20}",
        "rank",
        "address",
        "kind",
        "pkts",
        "last",
        "avg",
        "min",
        "max",
        "delta",
        "name",
        "mfg",
        "service",
        "pdu"
    );

    for (idx, candidate) in table.sorted(sort).into_iter().take(limit).enumerate() {
        let marker = if candidate.rssi_delta() >= significant_db {
            "*"
        } else {
            " "
        };
        println!(
            "{:<4} {:<17} {:<12} {:>6} {:>5} {:>6.1} {:>5} {:>5} {:>4}{} {:<24} {:<18} {:<22} {:<20}",
            idx + 1,
            candidate.address,
            candidate.kind(),
            candidate.packets,
            candidate.last_rssi,
            candidate.avg_rssi(),
            candidate.min_rssi,
            candidate.max_rssi,
            candidate.rssi_delta(),
            marker,
            candidate.name_summary(),
            candidate.manufacturer_summary(),
            candidate.service_summary(),
            candidate.type_summary(),
        );
    }

    println!();
    println!("* delta >= {significant_db} dB. Move a candidate near/far, then track it:");
    println!("  ble-analyzer-pro discover --target AA:BB:CC:DD:EE:FF --duration-ms 30000");
    println!("  ble-analyzer-pro -v -w target.pcap --filter-addr AA:BB:CC:DD:EE:FF");
}

fn print_target_update(candidate: &Candidate, elapsed: Duration, significant_db: i16) {
    let marker = if candidate.rssi_delta() >= significant_db {
        "significant"
    } else {
        "watching"
    };
    println!(
        "[{:>7.2}s] {} pkts={} last={}dBm avg={:.1} min={} max={} delta={}dB name={} mfg={} service={} ch={}",
        elapsed.as_secs_f64(),
        marker,
        candidate.packets,
        candidate.last_rssi,
        candidate.avg_rssi(),
        candidate.min_rssi,
        candidate.max_rssi,
        candidate.rssi_delta(),
        candidate.name_summary(),
        candidate.manufacturer_summary(),
        candidate.service_summary(),
        candidate.channel_summary(),
    );
}
