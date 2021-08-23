# audiovis
I tried to create a high resolution and classic audio visualiser with [cpal](https://github.com/RustAudio/cpal) as audio backend and [wgpu](https://github.com/gfx-rs/wgpu) as accelerated video frontend

## demo
bar visualisation          |  string visualisation
:-------------------------:|:-------------------------:
![](/media/demo_bars.gif)  |  ![](/media/demo_strings.gif)

### Supported Platforms
I primarily work on Linux so there will be the best support, but I try to keep portability in mind

Feature                 |   Windows 10  |   Linux   |   macOS   |
----------------------- | ------------- | --------- | --------- |
DefaultAudioInputStream | Supported     | Supported | ?         |
DesktopAudioInputStream | Supported     | Supported | ?         |
BarVisualisation        | Supported     | Supported | Supported |
StringVisualisation     | Supported     | Supported | Supported |

I am unable to test audiovis on macOS but it should just work fine.

It should also be possible without much work to port it over to the web but I never did something like this.

## Features
##### already implemented
* volume control
* rich configuration via config file
* string and bar visualisation
* configurable buffering and smoothing via config.toml
* configurable higher scaling of lower frequencies for a better look
* selectable amount of frequencies that should be displayed (0 - 20.000)
* automatically selecting Desktop output-audio-stream as input-stream (not sure if it is even possible)
* even runs on raspberrypi 4

##### WiP
* modifiable config via userinput during runtime
* selecting sampling rate of audio-stream

## How to configure
1. use `audiovis -g` to generate the default config as `default.config.toml` to your current directory
2. modify said configuration, documentation should be included in the file
3. use `audiovis -c <configuration>` to launch audiovis with your modified config
