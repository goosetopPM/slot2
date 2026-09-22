//! Turning the box off and on. BaseOS provides `baseos-poweroff` / `baseos-reboot`, which
//! sync the card and go through the vendor's shutdown path; plain `poweroff` is the
//! fallback. On the host both just end the process, since there is nothing to power off.

use std::process::Command;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PowerAction {
    PowerOff,
    Reboot,
}

impl PowerAction {
    fn commands(self) -> [&'static str; 2] {
        match self {
            PowerAction::PowerOff => ["/usr/sbin/baseos-poweroff", "/sbin/poweroff"],
            PowerAction::Reboot => ["/usr/sbin/baseos-reboot", "/sbin/reboot"],
        }
    }

    /// Perform the action. On a device this hands off to BaseOS and normally never returns;
    /// if every command fails it returns `false`. Off the device (`on_device == false`) it
    /// logs what it would have done and returns `true` so the caller can exit.
    pub fn perform(self, on_device: bool) -> bool {
        if !on_device {
            eprintln!("slot2: {self:?} requested (host: exiting instead)");
            return true;
        }
        for cmd in self.commands() {
            if !std::path::Path::new(cmd).exists() {
                continue;
            }
            eprintln!("slot2: {self:?} via {cmd}");
            match Command::new(cmd).status() {
                Ok(s) if s.success() => return true,
                Ok(s) => eprintln!("slot2: {cmd} exited with {s}"),
                Err(e) => eprintln!("slot2: {cmd}: {e}"),
            }
        }
        false
    }
}
