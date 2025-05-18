# Ellen / Ch22

A [MOS 6502](https://en.wikipedia.org/wiki/MOS_Technology_6502) emulator written in Rust, targeting web assembly

> The MOS Technology 6502 was an 8-bit microprocessor commonly found in the video consoles and home computers of the 1980s

## 🌟 Features

- Implementation of all 'legal' instructions and some 'illegal' instructions
- Passes the [SingleStepTests](https://github.com/SingleStepTests/65x02) (including full read/write cycles)

## ✔️ Requirements

- [node v22](https://nodejs.org/en) or later
- [Rust toolchain](https://www.rust-lang.org)

## 🏗️ Build

```bash
npm run build-release
# or `build-dev` to include panic! stack traces
```

## 🛠️ Usage from JavaScript (TypeScript)

### Setting up memory

```js
import initCh22, { Ch22Memory } from "./ch22-core/pkg";

const { memory: wasmMemory } = await initCh22();

function readMem(address: number): number {
  /**
   * custom callback for memory reads in these spaces:
   * 0x8000 - 0xc000
   * 0xfc00 - 0xff00
   */
  return 0;
}

function writeMem(address: number, value: number): boolean {
  /**
   * custom callback for memory writes in this space:
   * 0x8000 - 0xc000
   *
   * return true if write requires a clock phase 2 operation
   */
  return false;
}

const ch22Memory = Ch22Memory.new(readMem, writeMem);

/**
 * full 64k machine memory space stored in webassembly
 * use to initialise roms etc or for rendering video
 */
const memory = new Uint8Array(
  wasmMemory.buffer,
  ch22Memory.ram_start(),
  ch22Memory.ram_size(),
);
```

### Executing instructions / handling interrupts

```js
import { Ch22Cpu } from "./ch22-core/pkg";

function checkInterrupt(instructionCpuCycles: number): number {
  /**
   * Custom callback for checking interrupts
   * parameters:
   *   `instructionCpuCycles` - number of cpu cycles since the start of the instruction
   * returns:
   *   bitfield: 0x01 = IRQ, 0x02 = NMI
   */
}

function doPhase2(instructionCpuCycles: number) {
  /**
   * Custom callback for executing clock phase 2 operations
   * parameters:
   *   `instructionCpuCycles` - number of cpu cycles since the start of the instruction
   */
}

const cpu = Ch22Cpu.new(checkInterrupt, doPhase2);

/**
 * reset cpu (requires memory access to read reset vector)
 */
cpu.reset(ch22Memory);

/**
 * execute the next instruction or interrupt
 * returns number of cpu cycles executed for the instruction
 */
const cycleCount = cpu.handle_next_instruction(ch22Memory);
```

## 🧪 Running tests

```bash
npm test
```

## 🔮 Future Development

Hopefully 🤞
