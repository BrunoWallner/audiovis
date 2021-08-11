# audiovis
I tried to create a high quality classic audio visualiser with [cpal](https://github.com/RustAudio/cpal) as audio backend and [wgpu](https://github.com/gfx-rs/wgpu) as accelerated video frontend

![Demo Video](media/demo.mov)

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

