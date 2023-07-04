# Waves and Particles

Auto export gif for wave and particles.

Example:
![](output.gif)
```text
Import image and output waves and particles gif

Usage: waves_particles.exe [OPTIONS] [SPEED_GIF]

Arguments:
  [SPEED_GIF]  The speed for gif. Should be [1, 30] [default: 10]

Options:
  -i, --input <INPUT>                  The input image path [default: img.png]
  -o, --output <OUTPUT>                The output image path [default: output.gif]
      --width <WIDTH>                  The output gif width [default: 512]
      --height <HEIGHT>                The output gif height [default: 512]
      --bullet-width <BULLET_WIDTH>    The bullet width [default: 15]
      --bullet-height <BULLET_HEIGHT>  The bullet height [default: 15]
      --center-width <CENTER_WIDTH>    The center image width [default: 50]
      --center-height <CENTER_HEIGHT>  The center image height [default: 50]
  -w, --ways <WAYS>                    The bullet ways [default: 8]
  -f, --fps <FPS>                      [default: 10]
      --angle <ANGLE>                  The init angle [default: 0]
      --delta <DELTA>                  The speed of angle delta increase [default: 0.5]
      --speed <SPEED>                  The speed for bullet move per second [default: 96]
      --skip <SKIP>                    The frames to skip at beginning [default: 40]
      --frames <FRAMES>                The frames to record [default: 100]
  -r, --red <RED>                      The background color of red [default: 1]
  -g, --green <GREEN>                  The background color of green [default: 1]
  -b, --blue <BLUE>                    The background color of blue [default: 1]
  -a, --alpha <ALPHA>                  The background color of alpha [default: 1]
  -h, --help                           Print help
  -V, --version                        Print version

```