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
- Video:
  - ULA, CRTC and 'IC32' register addressing
  - Video memory and state snapshotting

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
 * set one of the paged Roms
 * - bank: bank to populate, 0-16, 16 = OS Rom
 * - pagedRom: 16k Uint8Array
 */
ch22System.load_rom(bank, pagedRom);

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
 *   - returns: next cycle sync and interrupt and optionally ic32_latch encoded as bigint
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

### Getting current state

```js
/**
 * get video ula control byte
 */
const ula_control = ch22System.get_ula_control();
```

```js
/**
 * get crtc register
 */
const crtc_register = ch22System.get_crtc_registers(register_index);
```

### Snapshotting Video memory into a buffer

```js
/**
 * get buffer of snapshotted scanline data
 * each line is 122 bytes:
 * - 1 byte     - 0 => out of scan, 1 => line visible, 2 => blank, 3 => invalid crtc range
 * - 100 bytes  - snapshot of up to 100 bytes of video memory for the scanline
 * - 2 bytes    - crtcMemoryAddress of snapshot
 * - 1 bytes    - crtcRasterAddress of snapshot (even field)
 * - 9 bytes    - crtc registers: R0, R1, R2, R3, R8, R10, R11, R14, R15
 * - 1 byte     - ula control register
 * - 8 bytes    - ula palette (16 nibbles)
 */
const memory = new Uint8Array(
  wasmMemory.buffer,
  ch22System.video_field_start(),
  ch22System.video_field_size(),
);

/**
 * clear the buffer
 */
ch22System.video_field_clear();

/**
 * add a snapshot of the current video memory and registers
 * - lineIndex: line in buffer for snapshot
 * - crtcMemoryAddress: crtc address for snapshot
 * - crtcRasterAddress: line index relative to current character row for the even field
 */
ch22System.snapshot_scanline(
  lineIndex,
  crtcMemoryAddress,
  crtcRasterAddressEvenField,
);
```

## 🧪 Running tests

```bash
npm test
```

## 🔮 Future Development

Hopefully 🤞
