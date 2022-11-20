# Wasm webgl game of life

## About

[Conway's game of life](https://en.wikipedia.org/wiki/Conway%27s_Game_of_Life) is a simple simulation that this project calculates and visualizes using [rust wasm](https://rustwasm.github.io/book/) and [webgl](https://get.webgl.org/).

This project was extended from the [rust wasm tutorial](https://rustwasm.github.io/book/game-of-life/introduction.html). 

The project structure was created from [`cargo generate`](https://github.com/ashleygwilliams/cargo-generate) project template [https://github.com/rustwasm/wasm-pack-template](https://github.com/rustwasm/wasm-pack-template) and npm init create [https://github.com/rustwasm/create-wasm-app](https://github.com/rustwasm/create-wasm-app)

## Usage


## Dev Setup
Install 
* [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/) 
* [node](https://nodejs.org/en/)

To run in development locally on localhost:8080
```
git clone <this repo>
cd <this repo>

npm install

wasm-pack build

npm run serve
```

### Build production into dist folder

```
wasm-pack build

npm run build
```

### Test in Headless Browsers with `wasm-pack test`

```
wasm-pack test --headless --firefox
```

## Included

* [`wasm-bindgen`](https://github.com/rustwasm/wasm-bindgen) for communicating
  between WebAssembly and JavaScript.

## License

* MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)