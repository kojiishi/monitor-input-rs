# monitor-input

A command line tool to change display monitors' input sources via DDC/CI.

## Installation

Please [install Rust](https://rustup.rs/) if you haven't done so.

### From [github](https://github.com/kojiishi/monitor-input-rs)

```shell-session
cargo install --git https://github.com/kojiishi/monitor-input-rs
```

### From [`crates.io`](https://crates.io/crates/monitor-input)

```shell-session
cargo install monitor-input
```

### From local checkout

```shell-session
cd monitor-input-rs
cargo install --path .
```

## Usages

### List all display monitors
You can get the list of displays by running the command without arguments.
```shell-session
monitor-input
```
The output should look like below.
```shell-session
0: Dell P2415Q
    Input Source: DisplayPort2
1: Generic PnP Monitor
    Input Source: 0
2: Dell U2723QE
    Input Source: DisplayPort1
3: Dell P3223QE
    Input Source: Hdmi1
```

Note that a display monitor may be listed twice.
This happens when there are multiple ways to find display monitors,
such as by the OS API and by the display driver APIs.

### Set the input source by name
```shell-session
monitor-input U2723=dp1 P3223=hdmi1
```

All display monitors that have the specified name are affected.
The following example sets the input sources of all displays
whose name have "Dell" to `DisplayPort1`.
```shell-session
monitor-input Dell=dp1
```
You can test which display monitors match
by omitting the input source.
The following example lists all display monitors
whose name have "Dell" without changing their input sources.
```shell-session
monitor-input Dell
```

### Set the input source by the display monitor index
```shell-session
monitor-input 2=usbc2 3=usbc2
```

###  Vendor-specific input sources
The input source can be a number.
This is useful when the display has vendor-specific input sources.
```shell-session
monitor-input U2723=15 P3223=17
```

### Toggle the input sources

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

#### Toggle multiple display monitors

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

#### Cycle between more than two input sources

Cycling between more than two input sources is also possible,
in the same way as toggling between two input sources.
```shell-session
monitor-input P3223=hdmi1,usbc2,dp1
```
If the current input source is `Hdmi1`, it will be `UsbC2`.
If it's `UsbC2`, it will be `DisplayPort1`.
Otherwise it will be `Hdmi1`.
