# Client implementation for hamlib's `rigctld`

This library makes use of `rigctld`, which is part of the [hamlib](https://hamlib.github.io/). The deamon provides a network interface to communicate with connected ham radio rigs. Check out the link above on how to get the hamlib on your device.

The given [client](src/rig.rs) implements the extended response protocol. Furthermore, to not have to start the daemon each time by hand, an [abstraction](src/daemon.rs) to start and stop `rigctld` is implemented. 

As for now, only the functions to get/set the frequency and mode are implemented within the client. The code already provides the necessary building blocks to implement the other available commands of the extended response protocol too. If you are missing a function feel free to implement it yourself or open an issue. The same applies for the daemon. If your use case requires an additional command line switch, it should be relatively straightforward to add it. Make sure to checkout `rigctld --help` to get an overview of the available command line switches and their parameters. For now, invalid parameters are not detected. This may result in communication timeouts between the client and `rigctld`. It is therefore recommended to manually start `rigctld` with the required command line switches beforehand to check wether all options are set correctly.

## Example

Within the [basic example](examples/basic.rs), the usage of the library is shown.

## Tests

A few basic tests are provided for the deamon and the client. All enabled tests will use the dummy device interface.

Before running all tests at once it is recommended to first check if `rigctld` is available on the system. To check for that run `cargo test daemon::tests::rigctld_exists`. It will try to start a binary available within the systems `PATH` by the name `rigctld`. To execute all other tests run `cargo test -- --test-threads 1`. This makes sure that the tests run sequentially which is required since all started deamons will listen on the same port.
