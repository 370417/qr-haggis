# Roadmap

Engine steps:

- [x] Define the game state
- [ ] Test a game state for correctness
- [ ] Implement the rules (playing cards, passing, etc)
- [ ] Compress the game state to the size of a QR code (goal: 272 bits)
- [ ] Reverse the compression
- [ ] Test roundtrip compression

Client steps:

- [ ] Convert between binary compressed game state and QR code
- [ ] Create QR code input and output
- [ ] Communicate between client and engine
- [ ] Display game state (terminal? webassembly?)
- [ ] Accept player input
