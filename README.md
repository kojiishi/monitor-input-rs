[![CI-badge]][CI]
[![crate-badge]][crate]
[![docs-badge]][docs]

[CI-badge]: https://github.com/kojiishi/monitor-input-rs/actions/workflows/rust-ci.yml/badge.svg
[CI]: https://github.com/kojiishi/monitor-input-rs/actions/workflows/rust-ci.yml
[crate-badge]: https://img.shields.io/crates/v/monitor-input.svg
[crate]: https://crates.io/crates/monitor-input
[docs-badge]: https://docs.rs/monitor-input/badge.svg
[docs]: https://docs.rs/monitor-input/

# monitor-input

A command line tool to change input sources of display monitors with [DDC/CI].

* Supports Windows/Mac/Linux.
* Also exposed [as library](#as-library).
* Please see [release notes] for the change history.

[DDC/CI]: https://en.wikipedia.org/wiki/Display_Data_Channel
[release notes]: https://github.com/kojiishi/monitor-input-rs/releases

# Install

## Prerequisites

* [Install Rust] if it's not installed yet.
* On Windows, please also see [Windows App](#windows-app).
* On Linux, `libudev` is required. See [libudev-sys].

[libudev-sys]: https://github.com/dcuddeback/libudev-sys

[install Rust]: https://rustup.rs/

## From [`crates.io`][crate]

```shell-session
cargo install monitor-input
```

## From [github](https://github.com/kojiishi/monitor-input-rs)

```shell-session
cargo install --git https://github.com/kojiishi/monitor-input-rs
```

## From local checkout

After changing the current directory to the checkout directory:
```shell-session
cargo install --path .
```

## As library

```shell-session
cargo add monitor-input
```
Please see the [API documentation at docs.rs][docs].

## Windows App

On Windows, there are two types of applications:
console applications and Windows applications.
The `monitor-input` is a console application.
This means that,
if you run it from non-console applications,
a console window pops up.

To run it from non-console applications
without seeing the console window popping up,
a Windows subsystem version is available as an optional feature.
Please add `-F winapp` to the `cargo install` command.
```shell-session
cargo install monitor-input -F winapp
```
The `-F winapp` option installs
the `monitor-inputw.exe` (note the `w` suffix)
in addition to the `monitor-input.exe`.
The executable with the `w` suffix is a Windows subsystem application.

# Usages

## List display monitors

### List all display monitors

Run the command without arguments
to get the list of all display monitors.
```shell-session
monitor-input
```
The output should look like below.
```shell-session
0: Dell P2415Q
    Input Source: DisplayPort2
    Backend: winapi
1: Generic PnP Monitor
    Input Source: 0
    Backend: winapi
2: Dell U2723QE
    Input Source: DisplayPort1
    Backend: winapi
3: Dell P3223QE
    Input Source: Hdmi1
    Backend: winapi
```

Note that a display monitor may be listed twice.
This happens when there are multiple ways to find display monitors,
such as by the OS API and by the display driver APIs.
The `Backend` field indicates how it was found.
The `-b` option can filter display monitors
by the backend name.

### Search display monitors by the name

You can search display monitors
by specifying part of their names.
The following example lists all display monitors
whose name have "Dell".
```shell-session
monitor-input Dell
```

### Search by the display monitor index

Searching by the display monitor index is also possible
by using a number.
```shell-session
monitor-input 2
```
This command line lists the display monitors of index 2.
In the example above,
it's "Dell U2723QE".

## Set the input source

To change the input sources of display monitors,
append `=` and the input source name.

### Set the input source by name
```shell-session
monitor-input U2723=dp1 P3223=hdmi1
```

When the name matches multiple display monitors,
all matched display monitors are affected.
The following example sets the input sources of all display monitors
whose name have "Dell" to `DisplayPort1`.
```shell-session
monitor-input Dell=dp1
```

### Set the input source by the display monitor index

The name can be a number,
which specifies the display monitor index.
```shell-session
monitor-input 2=usbc2 3=usbc2
```

###  Vendor-specific input sources
The input source can be a number.
This is useful when the display has non-standard, vendor-specific input sources.
```shell-session
monitor-input U2723=15 P3223=17
```

## Toggle the input sources

You can toggle between two input sources.
To do this, specify the input sources to toggle
with a `,` (comma) as the separator.
```shell-session
monitor-input P3223=hdmi1,usbc2
```
The example above toggles the input source between `Hdmi1` and `UsbC2`.

If the current input source is not in the list,
the input source is set to the first input source in the list.
In the example above,
if the current input source is `Hdmi1`, it will be `UsbC2`.
Otherwise it will be `Hdmi1`.

### Toggle multiple display monitors

When toggling input sources of multiple display monitors at once,
the first display monitor is used to determine the current input source.
```shell-session
monitor-input U2723=dp1,usbc2 P3223=hdmi1,usbc2
```
In this example, `U2723` is the first display monitor.
If its input source is `DisplayPort1`,
its input source is changed to `UsbC2` as explained before.

Because the first display monitor is changed
to its second input source in the list,
following display monitors will be changed
to their second input sources in their lists.
In this example, `P3223` has `hdmi1,usbc2`.
Its input source is set to the second entry, `UsbC2` in this case.

This rule makes the input sources always consistent,
even when the current input sources are not consistent across display monitors.

### Cycle between more than two input sources

Cycling between more than two input sources is also possible,
in the same way as toggling between two input sources.
```shell-session
monitor-input P3223=hdmi1,usbc2,dp1
```
If the current input source is `Hdmi1`, it will be `UsbC2`.
If it's `UsbC2`, it will be `DisplayPort1`.
Otherwise it will be `Hdmi1`.
