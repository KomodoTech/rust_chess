Chess
=====================
A simple web app for timed chess games


Playing from source
----------------------------

Depedencies:

The main depdency: the rust compiler.   
To get it, follow [rustup.rs](https://rustup.rs/) instructions.

On web, windows and mac os no other external depdendecies are required.
On linux followed libs may be required: 
```
apt install libx11-dev libxi-dev libgl1-mesa-dev libasound2-dev
```

### Running the game:

### Native PC build: 

```
cargo run --release
```
from this repo root.

### Building HTML5 build in chess_client project:
First, install cargo-make if you don't already have it:
```
cargo install cargo-make
```

Next, navigate to `chess_client` and run `cargo make update`. This will build the wasm file and move it into the `web` folder.
Alternatively, you can manually build and copy it yourself with the commands:

```
cargo build --target wasm32-unknown-unknown --release
cp target/wasm32-unknown-unknown/release/chess_client.wasm chess_client/web/chess_client.wasm
```

To serve the web build some web server will be required. One of the options: [devserver](https://github.com/kettle11/devserver) 

```
cargo install devserver
cd web
devserver .
```

And then open `localhost:8080`
