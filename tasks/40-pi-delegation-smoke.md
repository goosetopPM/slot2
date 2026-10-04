# Task 40 - Verify Gajacode-to-Raspberry-Pi test delegation

Work directly from `C:\SLOT2`, but do not modify project source or configuration. This is an
operations smoke test: prove that the delegated worker can invoke the existing Raspberry Pi
test path and receive a real ARM test result without Codex accessing the Pi directly.

## Exact task

1. Read only this file and `C:\SLOT2\build\pi-test.ps1`.
2. Run exactly:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File C:\SLOT2\build\pi-test.ps1 -Crates slot2-store
```

3. The script's default target is `goosetop@192.168.68.166`. Do not substitute another host.
4. If and only if the first run fails because `/src` or required remote assets were never set
   up, run this once and then repeat the command in step 2 once:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File C:\SLOT2\build\pi-test.ps1 -Setup -Crates slot2-store
```

5. Do not edit files to make the test pass. Do not use adb, Samba, an SD card, RG SP, or any
   host other than the named Raspberry Pi. Do not run arbitrary remote cleanup or inspect
   unrelated remote files. Do not commit or push.
6. Success requires the delegated script to cross-compile at least one aarch64 test binary,
   copy it to the Pi, execute it there, and finish with exit code 0 and `all green on aarch64`.
   A successful SSH-only probe is not enough.

## Result report

Write `C:\SLOT2\tasks\40-pi-delegation-smoke.worker-result.md` containing no more than 20 lines:

- success or failure and cumulative attempt number out of two;
- whether the normal run worked or `-Setup` was required;
- number of test binaries and concise passed/failed totals shown by the remote run;
- final script status line and process exit code;
- any connection, authentication, Docker, cross-build, copy, or remote-execution failure in one
  concise line;
- approximate elapsed time.

Do not paste full logs. Do not delegate or create another task. Wait up to five minutes for the
first model response and up to forty-five minutes overall. A quiet buffered log is not failure.
