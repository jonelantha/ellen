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
  - Canvas rendering (hires only)
- Sound:
  - Records the writes made to a sound chip's register bus (write enable and 8 bit data) for each field
  - Rendering the chip is left to the caller

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
import initCh22, { System } from './ch22-core/pkg';

const { memory: wasmMemory } = await initCh22();

const ch22System = System.new();

/**
 * set one of the paged Roms
 * - bank: bank to populate, 0-16, 16 = OS Rom
 * - pagedRom: 16k Uint8Array
 */
ch22System.load_rom(bank, pagedRom);

/**
 * register callbacks for a device
 * - addresses: UInt16Array of addresses to register device for
 * - read: (address: number, cycles: bigint) => bigint
 *   - returns: read value, next cycle sync and interrupt encoded as bigint
 * - write: (address: number, value: number, cycles: bigint) => bigint
 *   - returns: next cycle sync and interrupt encoded as bigint
 * - handleTrigger: (cycles: bigint) => bigint
 *   - callback if sync is required
 *   - returns: next cycle sync and interrupt encoded as bigint
 * - flags:
 *   - 0x01 = 1mhz device
 *   - 0x02 = interrupt treated as NMI
 *   - 0x04 = interrupt treated as IRQ
 *   - 0x10 = device writes in clock phase 2
 */
const deviceId = ch22System.add_js_device(
  addresses,
  read,
  write,
  handleTrigger,
  flags,
);

/**
 * register the system VIA, which is wired to the sound chip and the video
 * address latch (IC32). Takes the same callbacks as `add_js_device`, with
 * these differences:
 * - no flags
 * - write: (address: number, value: number, ic32: number, cycles: bigint) => bigint
 *   - ic32: the latch value
 * - onVsyncChange: (vsync: boolean) => bigint
 *   - called when vsync changes
 *   - returns: next cycle sync and interrupt encoded as bigint
 */
const sysViaDeviceId = ch22System.add_sys_via_stub(
  addresses,
  read,
  write,
  onVsyncChange,
  handleTrigger,
);

/**
 * manually set the interrupt of a device
 * - deviceId: id returned from `add_js_device` call
 * - interrupt: whether interrupt is set
 */
ch22System.set_device_interrupt(deviceId, interrupt);

/**
 * register a device which returns a fixed value
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
 * executes instructions until until the next field is ready for render
 * returns number of cycles
 */
const cycleCount = ch22System.run_one_field();
```

### Snapshotting Video memory into a buffer

```js
/**
 * get buffer of snapshotted scanline data
 * each line is 116 bytes:
 * - 1 byte     - flags: 0x01 => line displayed, 0x02 => has bytes, 0x04 => invalid crtc range, 0x08 => interlace video and sync, 0x10 => cursor displayed (even field), 0x20 => cursor displayed (odd field)
 * - 1 byte     - ula control register
 * - 1 byte     - total chars (R1)
 * - 1 byte     - back porch chars
 * - 1 byte     - cursor char
 * - 3 bytes    - padding
 * - 8 bytes    - ula palette
 * - 100 bytes  - snapshot of up to 100 bytes of video memory for the scanline
 */
const memory = new Uint8Array(
  wasmMemory.buffer,
  ch22System.video_field_start(),
  ch22System.video_field_size(),
);
```

### Reading sound register writes

```js
/**
 * get buffer of the sound chip register writes made during the last field
 * all values are little endian
 * - 8 bytes    - base cycle count (the cycle the field started on)
 * - 4 bytes    - number of entries
 * - 3 bytes per entry, up to 500:
 *   - 2 bytes  - cycle offset from the base cycle count
 *   - 1 byte   - data
 * the buffer is emptied at the start of each field
 */
const memory = new Uint8Array(
  wasmMemory.buffer,
  ch22System.sound_register_writes_start(),
  ch22System.sound_register_writes_size(),
);
```

### Rendering

#### Field data renderer

Render directly from field data buffer (hires modes only)

```js
import { initCanvas, getGPUContext, createFieldDataRenderer } from './render';
import initCh22, { System } from './ch22-core/pkg';

const { memory: wasmMemory } = await initCh22();

const ch22System = System.new();

// ...

const canvas = document.getElementById('canvas');

initCanvas(canvas);

const gpuContext = getGPUContext(canvas);

const fieldDataBuffer = {
  buffer: wasmMemory.buffer,
  byteOffset: ch22System.video_field_start(),
  byteLength: ch22System.video_field_size(),
};

const renderFieldData = createFieldDataRenderer(gpuContext, fieldDataBuffer);

// ...

// render from current contents of field data
renderFieldData();
```

#### Direct renderer

Render directly from 4 bit screen data buffer (for rendering non hires modes)

```js
import { initCanvas, getGPUContext, createDirectRenderer } from './render';

const canvas = document.getElementById('canvas');

initCanvas(canvas);

const gpuContext = getGPUContext(canvas);

const directBuffer = new Uint8Array(640 * 512);

const renderDirect = createDirectRenderer(gpuContext, directBuffer);

// ...

// render from directBuffer
renderDirect();
```

## 🧪 Running tests

```bash
npm test
```

## 🔮 Future Development

Hopefully 🤞
