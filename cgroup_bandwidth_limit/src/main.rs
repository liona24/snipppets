use std::os::fd::AsFd;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::thread;
use std::time::Duration;

use aya::maps::IterableMap;
use aya::programs::{CgroupSkb, CgroupSkbAttachType};
use aya::{include_bytes_aligned, Bpf};
use clap::Parser;
use log::{info, warn};

#[repr(C)]
struct Globals {
    byte_count: AtomicU64,
    hard_quota: u64,
}

impl Globals {
    pub fn byte_count(&self) -> u64 {
        self.byte_count.load(Ordering::Relaxed)
    }
}

mod mmap;

static SIGNAL_PENDING: AtomicBool = AtomicBool::new(false);
extern "C" fn signal_handler(_signal: i32) {
    info!("CTRL-C: exiting ..");
    SIGNAL_PENDING.store(true, Ordering::Relaxed);
}

#[derive(Debug, Clone, Parser)]
struct Opt {
    /// Cgroup to attach to (absolute path)
    #[clap(short, long)]
    cgroup: String,

    /// Allowed quota for ingress / egress each per quota period. Number of bytes.
    #[clap(short, long)]
    quota: u64,

    /// Quota period given in number of seconds
    #[clap(short, long)]
    quota_period: u64,

    /// Metrics collection sample interval in number of seconds
    #[clap(short, long)]
    sample_interval: u64,
}

struct ByteCount(u64);
impl core::fmt::Display for ByteCount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.0 < 1024 {
            write!(f, "{}B", self.0)
        } else if self.0 < 1024 * 1024 {
            write!(f, "{:.1}KiB", self.0 as f64 / 1024.)
        } else if self.0 < 1024 * 1024 * 1024 {
            write!(f, "{:.1}MiB", self.0 as f64 / (1024. * 1024.))
        } else {
            write!(f, "{:.1}GiB", self.0 as f64 / (1024. * 1024. * 1024.))
        }
    }
}

fn load_program(opt: &Opt, typ: CgroupSkbAttachType) -> Result<(), anyhow::Error> {
    let dir = match typ {
        CgroupSkbAttachType::Ingress => "ingress",
        CgroupSkbAttachType::Egress => "egress",
    };
    let raw = match typ {
        CgroupSkbAttachType::Ingress => include_bytes_aligned!("../ebpf/ebpf_ingress.o"),
        CgroupSkbAttachType::Egress => include_bytes_aligned!("../ebpf/ebpf_egress.o"),
    };

    let mut bpf = Bpf::load(raw)?;

    let g = aya::maps::Array::<_, u64>::try_from(bpf.take_map("globals").unwrap())?;
    let program: &mut CgroupSkb = bpf.program_mut("bandwidth_limit").unwrap().try_into()?;
    let cgroup = std::fs::File::open(&opt.cgroup)?;
    program.load()?;
    program.attach(cgroup, typ)?;

    info!("{} loaded", dir);

    let mut globals = unsafe { mmap::Mmap::<Globals>::new(g.map().fd().as_fd()) }
        .map_err(|_| anyhow::Error::msg("MAP_FAILED"))?;

    globals.hard_quota = opt.quota;
    let mut last_reset = std::time::Instant::now();
    let mut exceeded = false;

    // TODO: persist timers and counters regularly

    while !SIGNAL_PENDING.load(Ordering::Relaxed) {
        let t0 = std::time::Instant::now();
        let bytes_t0 = globals.byte_count();

        thread::sleep(Duration::from_secs(opt.sample_interval));

        let dt = t0.elapsed();
        let bytes = ByteCount(globals.byte_count());
        let delta = ByteCount((bytes.0 - bytes_t0) / dt.as_secs());

        if bytes.0 > opt.quota {
            if !exceeded {
                exceeded = true;
                warn!("{}: {} @ {}/s - 100% of quota exceeded!", dir, bytes, delta);
            }
        } else if bytes.0 * 4 > opt.quota * 3 {
            warn!("{}: {} @ {}/s - 75% of quota exceeded!", dir, bytes, delta);
        } else if bytes.0 * 2 > opt.quota {
            warn!("{}: {} @ {}/s - 50% of quota exceeded!", dir, bytes, delta);
        } else {
            info!("{}: {} @ {}/s", dir, bytes, delta);
        }

        if last_reset.elapsed().as_secs() > opt.quota_period {
            last_reset = std::time::Instant::now();
            globals.byte_count.store(0, Ordering::Relaxed);
            exceeded = false;
        }
    }

    Ok(())
}

fn main() {
    env_logger::init();

    let opt = Opt::parse();

    let ingress = {
        let opt = opt.clone();
        std::thread::spawn(move || {
            load_program(&opt, CgroupSkbAttachType::Ingress).expect("failed to load ingress");
        })
    };

    let egress = {
        let opt = opt.clone();
        std::thread::spawn(move || {
            load_program(&opt, CgroupSkbAttachType::Egress).expect("failed to load egress");
        })
    };

    unsafe { libc::signal(libc::SIGINT, signal_handler as usize) };

    ingress.join().unwrap();
    egress.join().unwrap();
}
