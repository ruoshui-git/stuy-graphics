# MDL

![creepy donut](./creepy-donut.gif)

Uses nom (a parser combinator) to parse mdl file. Each line is parsed individually.


# Changes

- Add dependency on `nom` and `thiserror` crates
- Refactor lights module
    - rename to "light"
    - separates original config into Light and LightProps (light properties of an object)
    - implement point lights