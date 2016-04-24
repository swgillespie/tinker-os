This repo is a small toy OS project that I'm tinkering with. It's written
entirely in rust and assembly, with zero C.

I've only tested compiling this on Linux. If you have qemu installed,
you can run `make run` to run the kernel in QEMU. You can also use
`make debug` to make QEMU wait for a debugger to attach, but my `gdb` 
can't handle the switch to long mode and dies. Your mileage my vary.
