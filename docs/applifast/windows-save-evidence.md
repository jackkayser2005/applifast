# Windows saves with an open reader

Baseline: `e5015645cf4a50d2c58e2b4963c8d9b8963438de`. Tested on this
Windows 11 x64 machine with the repository's pinned Rust 1.98.0 toolchain.

## Reproduction

A disposable, non-secret file was opened for reading, with read/write/delete
sharing enabled. The previous `MoveFileExW(REPLACE_EXISTING | WRITE_THROUGH)`
implementation failed with Windows error 5 (access denied). A native
`SetFileInformationByHandle(FileRenameInfoEx)` replacement with the replace and
POSIX flags succeeded. New opens read the new file; the original handle remains
attached to the old contents.

Repeating the probe with a reader that does not permit delete sharing produced
error 32 from the POSIX operation and preserved the old file. Such a lock is
still an actual save failure, not something this change bypasses.

The pinned Rust standard library already provides the POSIX operation as its
Windows `std::fs::rename` fallback after access denied. Reuse that implementation
instead of adding another variable-length Win32 structure or retry loop. The
normal write-through move remains the first attempt. A read-only destination
remains protected, and errors leave the original file and temporary replacement
intact for the caller to handle. No sleep, background retry, storage format,
credential contents, network access or interface layout is added.

Relevant primary references:

- [Rust 1.98.0 Windows rename implementation](https://github.com/rust-lang/rust/blob/1.98.0/library/std/src/sys/fs/windows.rs)
- [Windows rename information](https://learn.microsoft.com/en-us/windows/win32/api/winbase/ns-winbase-file_rename_info)
- [POSIX replacement semantics](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-fscc/4217551b-d2c0-42cb-9dc1-69a716cf6d0c)
- [MoveFileExW flags and access rules](https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-movefileexw)

## Scope and remaining evidence

The shared helper serves settings, window/session state, Apple library/queue
checkpoints, playlist caches/manifests and non-secret credential revocation
markers. Apple authorization tokens remain in Windows Credential Manager.
Other platforms retain their existing rename implementation. This change does
not claim power-loss durability or successful saves through exclusive locks.

The earlier intermittent cache/session test failures left a new temporary file
beside an older destination. Their original operating-system error was not
captured. The open-reader reproduction establishes one concrete defect, not
proof that it caused those earlier failures.

## Validation on October 10, 2026

The new open-reader regression failed against the baseline with error 5 before
the implementation changed. It passes with the fallback, including a Unicode
filename, new contents through the path, old contents through the retained
reader, and removal of the temporary name. A second regression covers a missing
source, a read-only destination, a reader denying delete sharing, preservation
of both files on failure, and success after that reader closes.

The existing session and Apple checkpoint tests now retain an old reader while
saving their replacements. They verify the older page/position through that
reader and the newer session/queue through a fresh load, including duplicate
queue occurrences, favorite state, upload identity and account isolation.

Formatting, strict default/demo all-target Clippy, 1,008 default and 1,042 demo
library tests (four opt-in checks ignored in each), binary/integration suites,
default doctests, strict demo Rustdoc, the demo build, gettext and Node
bridge/token checks pass. The ignored native Windows credential-store round
trip also passes using isolated dummy grants and removes those entries.
Portable-package fixture checks pass with dummy JWTs, including the immutable
source link to this report. README/tester-guide links and issue-form parsing pass.

Real-account restart, network interruption, output-device changes and gaming
soak remain separate release gates. Optional projectM/vcpkg, Ruby/Bundler site
checks, Nix and non-Windows compilation are unverified on this host. The
interface is unchanged; the existing heart and sign-in comparisons still apply.
