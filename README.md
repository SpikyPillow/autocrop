<p align="center">
    <img width="300" alt="Autocrop Logo" src="https://raw.githubusercontent.com/SpikyPillow/autocrop/master/image/autocrop.png">
</p>


<h1 align="center">Rui's super cool auto crop tool</h1>

<p align="center">
  <img width="600"
       src="https://raw.githubusercontent.com/SpikyPillow/autocrop/master/image/compare.png">
</p>

## About
This is the repo for rui's super cool auto crop tool! (R)autocrop for short!

Autocrop is a tool used to "crop out" the differences in similar images, the intent is it to be used for compression and data saving.

Presently, Autocrop is in alpha stages of development, the version number isn't taken into consideration on code changes, 
and the tool still has several bugs and features to be worked out. It is presently functional, just not yet ideal.

### Testing locally 

Right now if you want to run the program you can compile from source:

You will need rust installed, for instructions on how to do that see [here](https://doc.rust-lang.org/book/ch01-01-installation.html).

### Compile Constructions:
- On Linux you may need to first run `sudo apt-get install libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev`.
- Clone the repo (`git clone https://github.com/SpikyPillow/autocrop`)
- `cd autocrop`
- run `cargo run --release`
