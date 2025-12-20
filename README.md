# LSP Inspector

## TODOs

- [ ] Add a Tauri webview that uses Leptos.
- [ ] When app is run, set up Tauri app, then only start STDIN and STDOUT threads when the app is ready. This could be in the form of a tauri hook in core, or perhaps even a message from the frontend on initial mount when the components are ready to receive data.
- [ ] Use an MPSC channel to receive messages transmitted via the LSP from the STDIN and STDOUT threads, then use a tauri channel to stream those messages to the webview.
