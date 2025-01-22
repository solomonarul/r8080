# Intel 8080 Emulator

## How to use.

Write your own struct that implements the Bus8080 trait and attach it to the CPU using cpu.set_bus().

This will make sure that all reads / writes are redirected to your own devices.

You can also force a jump to set up the starting PC using cpu.force_jump(address).

For examples seee the tests/ folder.

### TODOs:

- graceful error handling.
- pass all tests.
- actual interrupt handling.
- try to get rid of nightly in tests or wait for trait upcasting to be a core feature of Rust.
- multithreading support.

---

<a href="https://brainmade.org/">
    Made with love by a human.
    <img src="https://brainmade.org/88x31-dark.png" align="right">
</a>