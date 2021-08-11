# audiovis
I tried to create a high quality classic audio visualiser with [cpal](https://github.com/RustAudio/cpal) as audio backend and [wgpu](https://github.com/gfx-rs/wgpu) as accelerated video frontend

![Demo Video](media/demo.gif)

### Supported Platforms
I primarily work on Linux so there will be the best support, but I try to keep portability in mind

Feature                 |   Windows 10  |   Linux   |   macOS   |
----------------------- | ------------- | --------- | --------- |
DefaultAudioInputStream | Supported     | Supported | ?         |
DesktopAudioInputStream | -             | WiP       | -         |
BarVisualisation        | Supported     | Supported | Supported |
StringVisualisation     | WiP           | WiP       | WiP       |

As you can see I could not figure out how to capture the Desktop Audio as Default Input-stream on any Platform,

You currently have to manually redirect your Desktop audio to your default input device, [guide for linux](https://www.kirsle.net/redirect-audio-out-to-mic-in-linux)

but I am trying to get it working on Linux.

And I am too poor and lazy to test it on macOS but it should work fine.

It should also be possible without much work to port it over to the web but I never did something like this.

## Features
##### already implemented
* 2-color bar coloring
* configurable buffering and smoothing via config.toml
* configurable scaling of lower frequencies for a better look
* selectable amount of frequencies that should be displayed (0 - 20.000)

##### WiP
* automatically selecting Desktop output-audio-stream as input-stream (not sure if it is even possible)
* displaying with strings instead of bars
* modifiable config via userinput during runtime
* selecting sampling rate of audio-stream
