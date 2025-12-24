# LSP Inspector

An application that wraps the language server to capture and display communication in \
realtime for easier debugging of the LSP.

## Origin

I had a bug in [Biome](https://biomejs.dev/) that I wanted to investigate, and the first \
step was to figure out if it was a client or server issue. I think there are some ways to \
get a log of messages being sent between the client and server, but it would likely be \
mixed in with all the regular messages that get sent during regular use.

One day, I came across [Langoustine](https://neandertech.github.io/langoustine/tracer.html) \
that has a GUI which allows the user to similarly monitor the LSP messages being sent \
between client and server. It was mentioned in a talk called \
[Adventures in the Land of Language Servers](https://www.youtube.com/watch?v=HF0xVrBZqtI). \
This triggered the idea to stick a program between the client and server. I could simply \
ensure all messages go through as per normal, but my program would extract the messages \
before they get sent through so that we can do stuff with them.

Why not use Langoustine? Frankly, I was too lazy to figure out how because it seemed like \
it would require installing a whole other language toolchain. So I decided to build \
`lsp-inspector` instead.
