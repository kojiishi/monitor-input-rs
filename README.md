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

You can get a list of displays by running the command without arguments.
```shell-session
set-monitor-input
```
The output should look like below.
```shell-session
0: Dell P2415Q
    Input Source: Ok(16)
1: Generic PnP Monitor
    Input Source: Ok(0)
2: Dell U2723QE
    Input Source: Ok(15)
3: Dell P3223QE
    Input Source: Ok(17)
```
You can set the input source by the display name as shown below.
```shell-session
set-monitor-input U2723QE=27 P3223QE=27
```
You can also specify the display by the index.
```shell-session
set-monitor-input 1=27 2=27
```
