# monitor-input

A command line tool to change display monitors' input sources via DDC/CI.

## Installation

### From github

```shell-session
cargo install --git https://github.com/kojiishi/monitor-input-rs
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
monitor-input U2723QE=dp1 P3223QE=hdmi1
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
monitor-input U2723QE=15 P3223QE=17
```
