import shadersWGSL from './shaders.wgsl?raw';

const CANVAS_WIDTH = 640;
const CANVAS_HEIGHT = 512;

interface BufferParams {
  buffer: ArrayBuffer;
  start: number;
  length: number;
}

const FIELD_BUFFER_BIND_GROUP_INDEX = 0;
const FIELD_BUFFER_BINDING = 0;

const BYTES_PER_ROW = 122;
const MAX_ROWS = 320;
const BUFFER_SIZE = MAX_ROWS * BYTES_PER_ROW;

export async function initRenderer(
  canvas: HTMLCanvasElement,
  sourceFieldBuffer: BufferParams,
): Promise<() => void> {
  canvas.width = CANVAS_WIDTH;
  canvas.height = CANVAS_HEIGHT;

  const device = await getGPUDevice();

  const context = getGPUContext(canvas, device);

  const pipeline = createPipeline(device);

  if (sourceFieldBuffer.length !== BUFFER_SIZE) {
    throw new Error(`Unexpected field buffer size ${sourceFieldBuffer.length}`);
  }

  const gpuFieldBuffer = createGPUBuffer(device, sourceFieldBuffer.length);

  const fieldBufferBindGroup = createFieldBufferBindGroup(
    device,
    pipeline,
    gpuFieldBuffer,
  );

  return function renderFrame() {
    writeToGPUBuffer(device, gpuFieldBuffer, sourceFieldBuffer);

    draw(device, context, pipeline, fieldBufferBindGroup);
  };
}

async function getGPUDevice() {
  const adapter = await navigator.gpu?.requestAdapter({
    featureLevel: 'compatibility',
  });

  if (!adapter) throw new Error('WebGPU adapter not available.');

  return await adapter.requestDevice();
}

function getGPUContext(canvas: HTMLCanvasElement, device: GPUDevice) {
  const context = canvas.getContext('webgpu');

  if (!context) throw new Error('WebGPU context not available.');

  context.configure({
    device,
    format: navigator.gpu.getPreferredCanvasFormat(),
  });

  return context;
}

function createPipeline(device: GPUDevice) {
  const shaderModule = device.createShaderModule({ code: shadersWGSL });

  return device.createRenderPipeline({
    layout: 'auto',
    vertex: {
      module: shaderModule,
      entryPoint: 'vertex_main',
    },
    fragment: {
      module: shaderModule,
      entryPoint: 'fragment_main',
      targets: [{ format: navigator.gpu.getPreferredCanvasFormat() }],
    },
    primitive: {
      topology: 'triangle-list',
    },
  });
}

function createGPUBuffer(device: GPUDevice, bufferLength: number) {
  return device.createBuffer({
    size: alignTo(bufferLength, 4),
    usage: GPUBufferUsage.STORAGE | GPUBufferUsage.COPY_DST,
  });
}

function writeToGPUBuffer(
  device: GPUDevice,
  dest: GPUBuffer,
  source: BufferParams,
) {
  device.queue.writeBuffer(dest, 0, source.buffer, source.start, source.length);
}

function createFieldBufferBindGroup(
  device: GPUDevice,
  pipeline: GPURenderPipeline,
  gpuFieldBuffer: GPUBuffer,
) {
  return device.createBindGroup({
    layout: pipeline.getBindGroupLayout(FIELD_BUFFER_BIND_GROUP_INDEX),
    entries: [
      {
        binding: FIELD_BUFFER_BINDING,
        resource: { buffer: gpuFieldBuffer },
      },
    ],
  });
}

function draw(
  device: GPUDevice,
  context: GPUCanvasContext,
  pipeline: GPURenderPipeline,
  fieldBufferBindGroup: GPUBindGroup,
) {
  const commandEncoder = device.createCommandEncoder();

  const passEncoder = commandEncoder.beginRenderPass({
    colorAttachments: [
      {
        view: context.getCurrentTexture().createView(),
        clearValue: [0, 0, 0, 0],
        loadOp: 'clear',
        storeOp: 'store',
      },
    ],
  });
  passEncoder.setPipeline(pipeline);
  passEncoder.setBindGroup(FIELD_BUFFER_BIND_GROUP_INDEX, fieldBufferBindGroup);
  passEncoder.draw(3);
  passEncoder.end();

  device.queue.submit([commandEncoder.finish()]);
}

function alignTo(value: number, alignment: number): number {
  return Math.ceil(value / alignment) * alignment;
}
