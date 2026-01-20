export interface BufferParams {
  buffer: ArrayBufferLike;
  byteOffset: number;
  byteLength: number;
}

export async function getGPUDevice() {
  const adapter = await navigator.gpu?.requestAdapter({
    featureLevel: 'compatibility',
  });

  if (!adapter) throw new Error('WebGPU adapter not available.');

  return await adapter.requestDevice();
}

export function getGPUContext(canvas: HTMLCanvasElement, device: GPUDevice) {
  const context = canvas.getContext('webgpu');

  if (!context) throw new Error('WebGPU context not available.');

  context.configure({
    device,
    format: navigator.gpu.getPreferredCanvasFormat(),
  });

  return context;
}

export function createGPUBuffer(
  device: GPUDevice,
  bufferLength: number,
  copyDst: boolean,
) {
  if (bufferLength % 4 !== 0) {
    throw new Error(`Not multiple of 4: ${bufferLength}`);
  }

  return device.createBuffer({
    size: bufferLength,
    usage: GPUBufferUsage.STORAGE | (copyDst ? GPUBufferUsage.COPY_DST : 0),
  });
}

export function writeToGPUBuffer(
  device: GPUDevice,
  dest: GPUBuffer,
  source: BufferParams,
) {
  device.queue.writeBuffer(
    dest,
    0,
    source.buffer,
    source.byteOffset,
    source.byteLength,
  );
}
