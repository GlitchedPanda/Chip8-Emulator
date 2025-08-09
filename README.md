# Chip-8 Emulator in Rust

![Preview](https://i.imgur.com/vGiC32H.png)

A small CHIP-8 emulator written in Rust, developed as a learning project to deepen my understanding of emulators, the Rust programming language, and computer architecture.

## Installation

1. Clone the repository:
   ```bash
   git clone https://github.com/GlitchedPanda/Chip8-Emulator.git
   cd Chip8-Emulator
   ```
2. Build the project:
    ```bash
    cargo build --release
    ```
3. Run the emulator:
    ```bash
    cargo run [pathToGame]
    ```
    You can find public-domain games [here](https://www.zophar.net/pdroms/chip8/chip-8-games-pack.html). 

## To Do
* [ ] Input
* [x] Propper opcode loop (InstructionsPerFrame)
* [ ] Sound

## Acknowledgments
These were the most useful CHIP-8 references I used when developing this project. 

* [CHIP8 - Wikipedia](https://en.wikipedia.org/wiki/CHIP-8)
* [Building a CHIP-8 Emulator - AUSTIN MORLAN](https://austinmorlan.com/posts/chip8_emulator/)
* [How to write an emulator - Multigesture](https://multigesture.net/articles/how-to-write-an-emulator-chip-8-interpreter/)

## License

This project is licensed under the GNU General Public License v3.0 - see the [LICENSE](LICENSE) file for details.
