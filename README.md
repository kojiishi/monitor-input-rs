## Installation

### From github

```shell-session
cargo install --git https://github.com/kojiishi/set-monitor-input-rs
```

### From local checkout

```shell-session
cd set-monitor-input-rs
cargo install --path .
```

## Usages

### List displays
You can get a list of displays by running the command without arguments.
```shell-session
set-monitor-input
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

### Set the input source by the display name
```shell-session
set-monitor-input U2723QE=dp1 P3223QE=hdmi1
```

### Set the input source by the display index
```shell-session
set-monitor-input 1=usbc1 2=27
```
