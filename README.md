# audiovis
I tried to create a high resolution and yet classic audio visualiser with [cpal](https://github.com/RustAudio/cpal) as audio backend and [wgpu](https://github.com/gfx-rs/wgpu) as accelerated video frontend

## demo
![](/media/demo.gif)

## 3D
check the `experimental_3D` for experimental 3D support, keep in mind that the codebase is older, so it is not as modular and still depends on wgpu version 0.9.0 instead of 0.10.0

### Supported Platforms
I primarily work on Linux so there will be the best support, but I try to keep portability in mind so it should work in Linux, Windows and MacOS

I am unable to test audiovis on MacOS but it should just work fine.

It should also be possible without much work to port it over to the web but I never did something like this.

## Features
##### already implemented
* volume control
* configurable buffering and smoothing via config.toml
* configurable higher scaling of lower frequencies for a better look
* selectable amount of frequencies that should be displayed (0 - 20.000)
* even runs on raspberrypi 4 with latest vulkan drivers installed

##### WiP
* reimplementation of config
* better working audio-capturing
* automatically pausing processing after some time, when no sound is playing
* modifiable config via userinput during runtime
* selecting sampling rate of audio-stream

## How to configure (currently not working, but should be reimplemented in a few commits)
1. use `audiovis -g` to generate the default config as `default.config.toml` to your current directory
2. modify said configuration, documentation should be included in the file
3. use `audiovis -c <configuration>` to launch audiovis with your modified config
