### Use the rust nightly compiler to compile the kernel.
-> Run "rustup override set nightly"

### Install "bootimage" tool
-> cargo install bootimage

### Add other tools like llvm-tools
-> rustup component add llvm-tools

### Ensure qemu is installed
-> for MacOS: brew install qemu [for other linux distributions, use corresponding package managers]
-> for windows: sorry
