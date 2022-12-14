# Wasm webgl game of life

Work in progress

## About

[Conway's game of life](https://en.wikipedia.org/wiki/Conway%27s_Game_of_Life) is a simple simulation that this project calculates and visualizes using [rust wasm](https://rustwasm.github.io/book/) and [webgl](https://get.webgl.org/).

This project was extended from the [rust wasm tutorial](https://rustwasm.github.io/book/game-of-life/introduction.html). 

The project structure was created from the [cargo generate](https://github.com/ashleygwilliams/cargo-generate) project template [https://github.com/rustwasm/wasm-pack-template](https://github.com/rustwasm/wasm-pack-template) and npm init create [https://github.com/rustwasm/create-wasm-app](https://github.com/rustwasm/create-wasm-app)

## Usage
TBD

## Dev Setup
Install 
* [rust](https://www.rust-lang.org/tools/install) 
* [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/) 
* [node](https://nodejs.org/en/)

To run in development locally on localhost:8080
```
git clone https://github.com/nickrubenstein/game-of-life-wasm-webgl.git

cd game-of-life-wasm-webgl

npm install

wasm-pack build

npm run serve
```

### Build production into a dist folder

```
wasm-pack build

npm run build
```

### Test in Headless Browsers

```
wasm-pack test --headless --firefox
```

## Included

* [wasm-bindgen](https://github.com/rustwasm/wasm-bindgen) for communicating
  between WebAssembly and JavaScript.

## License

* MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)