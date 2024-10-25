# dump-diff

Crude diff tools for assembly binary blobs

## usage

```console
Usage: dump-diff [OPTIONS] --target <TARGET> --current <CURRENT> --addr <ADDR> --arch <ARCH> --mode <MODE>

Options:
  -t, --target <TARGET>          Filename + optional offset
  -c, --current <CURRENT>        Filename + optional offset
  -x, --addr <ADDR>              Virtual address
  -l, --length <LENGTH>          Number of bytes to compare (default: all)
  -a, --arch <ARCH>              Architecture
  -m, --mode <MODE>              Mode
  -e, --endianness <ENDIANNESS>  Endianness (default: little)
  -h, --help                     Print help
```

- TARGET: the target binary, optionally followed by a colon and an offset. `-t file.bin:0x42`.
- CURRENT: your current compiled code, same as above.
- ADDR: the (virtual) memory address where the code is located at.
- ARCH: the [architecture](https://docs.rs/capstone/latest/capstone/enum.Arch.html)
- MODE: the [mode](https://docs.rs/capstone/latest/capstone/enum.Mode.html)
- ENDIANNESS: big or little

⚠️ WARNING ⚠️ The tool doesn't check the arch/mode tuple, so you can ask for ARM64 + RiscV64 (and it will panic).

## Example

```console
$ dump-diff -t target.bin -c current.bin -x 0x821843e8 -a ppc -m mode32 -e big
mflr r12                      mflr r12
stw r12, -8(r1)               stw r12, -8(r1)
std r30, -0x18(r1)            std r30, -0x18(r1)
std r31, -0x10(r1)            std r31, -0x10(r1)
stfd f31, -0x20(r1)           stfd f31, -0x20(r1)
stwu r1, -0x80(r1)            stwu r1, -0x80(r1)
lfs f31, 0x54(r3)             mr r31, r3                    <===========
mr r31, r3                    mr r30, r4                    <===========
mr r30, r4                    lfs f31, 0x54(r31)            <===========
bl 0x8210fb68                 bl 0x8210fb68
extsh r11, r30                extsh r11, r30
lis r10, -0x7dff              lis r10, -0x7dff
std r11, 0x50(r1)             std r11, 0x50(r1)
lfd f0, 0x50(r1)              lfd f0, 0x50(r1)
fcfid f0, f0                  fcfid f0, f0
lis r9, -0x7e00               lis r9, -0x7e00
frsp f12, f0                  frsp f12, f0
lfd f0, -0x3fb0(r10)          lfd f0, -0x3fb0(r10)
lfs f13, 0x1580(r9)           lfs f13, 0x1580(r9)
fmuls f12, f1, f12            fmuls f12, f1, f12
fnmsub f0, f12, f0, f31       fnmsub f0, f12, f0, f31
frsp f0, f0                   frsp f0, f0
fcmpu cr6, f0, f13            fcmpu cr6, f0, f13
blt cr6, 0x82184450           blt cr6, 0x82184450
fsubs f0, f0, f13             fsubs f0, f0, f13
b 0x82184464                  b 0x82184464
lis r11, -0x7e00              lis r11, -0x7e00
lfs f12, 0x7a0(r11)           lfs f12, 0x7a0(r11)
fcmpu cr6, f0, f12            fcmpu cr6, f0, f12
bge cr6, 0x82184464           bge cr6, 0x82184464
fadds f0, f0, f13             fadds f0, f0, f13
stfs f0, 0x54(r31)            stfs f0, 0x54(r31)
addi r1, r1, 0x80             addi r1, r1, 0x80
lwz r12, -8(r1)               lwz r12, -8(r1)
mtlr r12                      mtlr r12
lfd f31, -0x20(r1)            lfd f31, -0x20(r1)
ld r30, -0x18(r1)             ld r30, -0x18(r1)
ld r31, -0x10(r1)             ld r31, -0x10(r1)
blr                           blr 
```