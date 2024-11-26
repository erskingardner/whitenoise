# White Noise

A secure, private, and decentralized chat app built on Nostr, using the MLS protocol under the hood.

## Overview

White Noise aims to be the most secure private chat app on Nostr, with a focus on privacy and security. Under the hood, it uses the [Messaging Layer Security](https://www.rfc-editor.org/rfc/rfc9420.html) (MLS) protocol to manage group communications in a highly secure way. Nostr is used as the transport protocol and as the framework for the ongoing conversation in each chat.

## The Spec

White Noise is an implemenetation of the [NIP-104](https://github.com/nostr-protocol/nips/pull/1427) spec. This is still a draft spec, so it may change before it is finalized.

## Running White Noise

White Noise is a Tauri app, so it can be run natively on Windows, MacOS, Linux, iOS, and Android. The app is still under heavily development so not all of these platforms are supported yet. Currently the easist (most reliable) way to run the app is on MacOS desktop via the `bun tauri dev`.

## Contributing

White Noise is built with Tauri 2 and SvelteKit. To get started contributing you'll need to have the Rust toolchain installed and the Bun JavaScript package manager.

1. Clone the repo.
2. Run `bun install` to install the dependencies.
3. Wait for Rust to install dependencies. 
4. Run `bun tauri dev` to start the app. If you want to see more comprehensive logging, run `RUST_LOG=debug bun tauri dev`.

## License

White Noise is free and open source software, released under the Gnu AGPL v3 license. See the [LICENSE](LICENSE) file for details.
