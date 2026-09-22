//! First-boot hardware survey. Everything the design doc lists as "verify on the device"
//! (V-1 … V-13) that can be answered by reading a file, in one report, so one boot of a
//! card answers all of them. Written to stderr (BaseOS keeps that in `/tmp/frontend.log`)
//! and to `System/slot2-diag.txt` on the card, which a card reader can show without adb.

use std::fmt::Write as _;
use std::path::Path;
use std::process::Command;

#[allow(dead_code)]
pub fn report(root: &Path) -> String {
    let mut out = String::new();
    let mut section = |title: &str, body: String| {
        let _ = writeln!(out, "## {title}\n{}\n", body.trim_end());
    };

    section("baseos-release", read("/etc/baseos-release"));
    section(
        "glibc (V-3)",
        run("/lib/ld-linux-aarch64.so.1", &["--version"]),
    );
    section("cwd / mounts (V-4)", {
        let cwd = std::env::current_dir()
            .map(|p| p.display().to_string())
            .unwrap_or_default();
        format!(
            "cwd={cwd}\n{}",
            grep(&read("/proc/mounts"), &["sdcard", "mmcblk"])
        )
    });
    section(
        "input devices (V-1)",
        grep(
            &read("/proc/bus/input/devices"),
            &["N: Name", "H: Handlers", "B: KEY", "B: ABS"],
        ),
    );
    section("backlight / power / lid paths (V-2)", {
        let mut s = String::new();
        for d in [
            "/sys/class/backlight",
            "/sys/class/power_supply",
            "/sys/class/input",
            "/sys/class/leds",
            "/sys/class/pwm",
        ] {
            let _ = writeln!(s, "{d}: {}", ls(d));
        }
        let _ = writeln!(
            s,
            "lid candidates: {}",
            grep(
                &read("/proc/bus/input/devices"),
                &["lid", "Lid", "hall", "gpio-keys"]
            )
        );
        s
    });
    section("framebuffer / drm (V-6, V-13)", {
        let mut s = String::new();
        for f in [
            "/sys/class/graphics/fb0/virtual_size",
            "/sys/class/graphics/fb0/bits_per_pixel",
            "/sys/class/graphics/fb0/stride",
            "/sys/class/graphics/fb0/modes",
        ] {
            let _ = writeln!(s, "{f}: {}", read(f).trim());
        }
        let _ = writeln!(s, "/sys/class/drm: {}", ls("/sys/class/drm"));
        let _ = writeln!(
            s,
            "hdmi status: {}",
            read("/sys/class/drm/card0-HDMI-A-1/status").trim()
        );
        let _ = writeln!(s, "/dev/mali0: {}", Path::new("/dev/mali0").exists());
        for lib in [
            "libEGL.so.1",
            "libEGL.so",
            "libGLESv2.so.2",
            "libGLESv2.so",
            "libmali.so",
            "libasound.so.2",
        ] {
            let _ = writeln!(s, "{lib}: {}", find_lib(lib));
        }
        s
    });
    section(
        "audio (V-7)",
        format!("{}\n{}", read("/proc/asound/cards"), ls("/dev/snd")),
    );
    section("network tools (V-11)", {
        let mut s = String::new();
        for t in [
            "wpa_supplicant",
            "udhcpc",
            "rfkill",
            "iw",
            "wpa_cli",
            "dropbear",
            "avahi-daemon",
        ] {
            let _ = writeln!(s, "{t}: {}", which(t));
        }
        let _ = writeln!(
            s,
            "wlan0 operstate: {}",
            read("/sys/class/net/wlan0/operstate").trim()
        );
        let _ = writeln!(s, "/data: {}", ls("/data"));
        s
    });
    section("misc", {
        let mut s = String::new();
        let _ = writeln!(s, "kernel: {}", read("/proc/version").trim());
        let _ = writeln!(
            s,
            "cpuinfo: {}",
            grep(
                &read("/proc/cpuinfo"),
                &["model name", "CPU part", "Hardware"]
            )
        );
        let _ = writeln!(
            s,
            "meminfo: {}",
            grep(&read("/proc/meminfo"), &["MemTotal", "MemAvailable"])
        );
        let _ = writeln!(s, "baseos bins: {}", ls("/usr/sbin"));
        s
    });

    let _ = std::fs::write(root.join("System").join("slot2-diag.txt"), &out);
    out
}

#[allow(dead_code)]
fn read(p: &str) -> String {
    std::fs::read_to_string(p).unwrap_or_else(|e| format!("<{e}>"))
}

#[allow(dead_code)]
fn ls(p: &str) -> String {
    match std::fs::read_dir(p) {
        Ok(rd) => {
            let mut v: Vec<String> = rd
                .filter_map(|e| e.ok())
                .map(|e| e.file_name().to_string_lossy().into_owned())
                .collect();
            v.sort();
            v.join(" ")
        }
        Err(e) => format!("<{e}>"),
    }
}

#[allow(dead_code)]
fn grep(text: &str, needles: &[&str]) -> String {
    text.lines()
        .filter(|l| needles.iter().any(|n| l.contains(n)))
        .collect::<Vec<_>>()
        .join("\n")
}

#[allow(dead_code)]
fn run(cmd: &str, args: &[&str]) -> String {
    match Command::new(cmd).args(args).output() {
        Ok(o) => {
            let s = String::from_utf8_lossy(&o.stdout);
            let e = String::from_utf8_lossy(&o.stderr);
            s.lines()
                .chain(e.lines())
                .take(3)
                .collect::<Vec<_>>()
                .join("\n")
        }
        Err(e) => format!("<{e}>"),
    }
}

#[allow(dead_code)]
fn which(tool: &str) -> String {
    for d in ["/usr/sbin", "/usr/bin", "/sbin", "/bin", "/usr/local/bin"] {
        let p = format!("{d}/{tool}");
        if Path::new(&p).exists() {
            return p;
        }
    }
    "-".into()
}

#[allow(dead_code)]
fn find_lib(name: &str) -> String {
    for d in [
        "/usr/lib",
        "/lib",
        "/usr/lib/aarch64-linux-gnu",
        "/lib/aarch64-linux-gnu",
        "/usr/lib/mali",
        "/vendor/lib",
    ] {
        let p = format!("{d}/{name}");
        if Path::new(&p).exists() {
            return p;
        }
    }
    "-".into()
}
