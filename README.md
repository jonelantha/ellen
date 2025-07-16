# Ellen / Ch22

A Rust library to emulate an 8-bit microcomputer featuring [MOS 6502](https://en.wikipedia.org/wiki/MOS_Technology_6502) emulation

Targeting web assembly in the browser

> The MOS Technology 6502 was an 8-bit microprocessor commonly found in the video consoles and home computers of the 1980s

## 🌟 Features

- 6502 emulation:
  - Implementation of all 'legal' instructions and some 'illegal' instructions
  - Passes the [SingleStepTests](https://github.com/SingleStepTests/65x02) (including full read/write cycles)
- Memory layout:
  - 32k ram
  - a bank of upto 16 paged roms
  - a fixed rom
  - a dedicated IO space mapped to devices
- Cycle management:
  - inserts additional cycles for reads/writes to slower devices
  - supports devices with actions occuring on clock phase 2
- Device support:
  - IO devices with addresses which map to the IO space
  - Timer devices which require a callback after a certain number of cycles

## ✔️ Requirements

- [node v22](https://nodejs.org/en) or later
- [Rust toolchain](https://www.rust-lang.org)

## 🏗️ Build

```bash
npm run build-release
# or `build-dev` to include panic! stack traces
```

## 🛠️ Usage from JavaScript (TypeScript)

### Setting up

```js
import initCh22, { System } from "./ch22-core/pkg";

const { memory: wasmMemory } = await initCh22();

const ch22System = System.new();

/**
 * set the OS Rom
 * - osRom: 16k Uint6Array
 */
ch22System.load_os_rom(osRom);

/**
 * set one of the paged Roms
 * - bank: bank to populate, 0-15
 * - pagedRom: 16k Uint8Array
 */
ch22System.load_paged_rom(bank, pagedRom);

/**
 * full 64k machine memory space stored in webassembly
 * use to initialise roms etc or for rendering video
 */
const memory = new Uint8Array(
  wasmMemory.buffer,
  ch22Memory.ram_start(),
  ch22Memory.ram_size(),
);

/**
 * register a callback to be called at certain cycles
 * - handleTrigger: (cycles: bigint): bigint
 *   - cycles: machine cycles at time of callback
 *   - returns: the desired next value of cycles to be called encoded as a bigint
 */
const deviceId = ch22System.add_js_timer_device(handleTrigger);

/**
 * manually set the desired next value of cycles for a registered callback
 * - deviceId: id returned from `add_js_timer_device` call
 * - cycles: bigint
 */
ch22System.set_device_trigger(deviceId, cycles);

/**
 * register callbacks for an IO device
 * - addresses: UInt16Array of addresses to register device for
 * - read: (address: number, cycles: bigint): bigint
 *   - returns: read value, next cycle sync and interrupt encoded as bigint
 * - write: (address: number, value: number, cycles: bigint): bigint
 *   - returns: next cycle sync and interrupt encoded as bigint
 * - handleTrigger: (address: number, value: number, cycles: bigint): bigint
 *   - callback if sync is required
 *   - returns: next cycle sync and interrupt encoded as bigint
 * - flags:
 *   - 0x01 = 1mhz device
 *   - 0x02 = interrupt treated as NMI
 *   - 0x04 = interrupt treated as IRQ
 *   - 0x10 = device writes in clock phase 2
 */
const deviceId = ch22System.add_js_io_device(
  addresses,
  read,
  write,
  handleTrigger,
  flags,
);

/**
 * manually set the interrupt of a device
 * - deviceId: id returned from `add_js_io_device` call
 * - interrupt: whether interrupt is set
 */
ch22System.set_device_interrupt(deviceId, interrupt);

/**
 * register an io device which returns a fixed value
 * - addresses: UInt16Array of address to register device for
 * - readValue: 8 bit value to return for all reads
 * - oneMhz: bool for one mhz reads
 * - panicOnWrite: rust should panic if write attempted
 */
ch22System.add_static_device(addresses, readValue, oneMhz, panicOnWrite);
```

### Executing instructions

```js
import { Ch22Cpu } from "./ch22-core/pkg";

/**
 * reset cpu
 */
ch22System.reset();

/**
 * executes instructions until targetCycles is reached
 * returns number of cycles
 */
const cycleCount = ch22System.run(targetCycles);
```

## 🧪 Running tests

```bash
npm test
```

## 🔮 Future Development

Hopefully 🤞
