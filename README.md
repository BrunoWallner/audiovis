# audiovis
I tried to create a high resolution and classic audio visualiser with [cpal](https://github.com/RustAudio/cpal) as audio backend and [wgpu](https://github.com/gfx-rs/wgpu) as accelerated video frontend

It is not intended to be scientifically used.

## demo
![](/media/demo.gif)

### Supported Platforms
I primarily work on Linux so there will be the best support, but I try to keep portability in mind

Feature                 |   Windows 10  |   Linux       |   macOS       |
----------------------- | ------------- | ------------- | ------------- |
DefaultAudioInputStream | Supported     | Supported     | ?             |
DesktopAudioInputStream | Supported     | Supported     | ?             |
3DVisualisation         | Supported     | Supported     | Supported     |

2D String- and BarVisualisation are still available in the legacy branch

I am unable to test audiovis on macOS but it should just work fine.

It should also be possible without much work to port it over to the web but I never did something like this.

## Features
##### already implemented
* volume control
* 3D visualisation
* rich configuration via config file
* configurable buffering and smoothing via config.toml
* configurable higher scaling of lower frequencies for a better look
* selectable amount of frequencies that should be displayed (0 - 20.000)
* automatically selecting Desktop output-audio-stream as input-stream
* bar texturing
* selecting sampling rate of audio-stream
+ multiple types of visualisation
* even runs on raspberrypi 4

##### WiP
* modifiable config via userinput during runtime
* more visualisation types
* beat visualisation

## How to configure
1. use `audiovis -g` to generate the default config as `default.config.toml` to your current directory
2. modify said configuration, documentation should be included in the file
3. use `audiovis -c <configuration>` to launch audiovis with your modified config

## How to customize bar-texture
1. create a new picture with any height and width
2. the left pixels correspond the the front bars and the right to the bars further away
3. create a color transition in any image manipulation tool, I use krita, which works perfectly for this

## How to use microphone as input
* just type `audiovis -i`
