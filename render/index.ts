import shadersWGSL from './shaders.wgsl?raw';

const CANVAS_WIDTH = 640;
const CANVAS_HEIGHT = 512;

interface BufferParams {
  buffer: ArrayBufferLike;
  start: number;
  length: number;
}

const FIELD_BIND_GROUP_INDEX = 0;
const FIELD_BUFFER_BINDING = 0;
const FIELD_FRAME_METRICS_BUFFER_BINDING = 1;
const FIELD_BUFFER_BYTES_PER_LINE = 116;
const FIELD_FRAME_METRICS_BUFFER_SIZE = 3 * 4; // three 32-bit values

const DIRECT_BIND_GROUP_INDEX = 1;
const DIRECT_BUFFER_BINDING = 0;
const DIRECT_BUFFER_SIZE = CANVAS_WIDTH * CANVAS_HEIGHT; // 327,680 bytes

export async function initRenderer(
  canvas: HTMLCanvasElement,
  sourceFieldBuffer: BufferParams,
  sourceDirectBuffer: BufferParams,
): Promise<{ renderField: () => void; renderDirect: () => void }> {
  canvas.width = CANVAS_WIDTH;
  canvas.height = CANVAS_HEIGHT;

  const device = await getGPUDevice();

  const context = getGPUContext(canvas, device);

  const shaderModule = device.createShaderModule({ code: shadersWGSL });

  if (sourceFieldBuffer.length % FIELD_BUFFER_BYTES_PER_LINE !== 0) {
    throw new Error(`Not multiple of line size: ${sourceFieldBuffer.length}`);
  }

  const gpuFieldBuffer = createGPUBuffer(
    device,
    sourceFieldBuffer.length,
    true,
  );
  const gpuFrameMetricsBuffer = createGPUBuffer(
    device,
    FIELD_FRAME_METRICS_BUFFER_SIZE,
    false,
  );
  const gpuDirectBuffer = createGPUBuffer(device, DIRECT_BUFFER_SIZE, true);

  const computePipeline = createComputePipeline(device, shaderModule);
  const computeBindGroup = createBindGroup(
    device,
    computePipeline,
    gpuFieldBuffer,
    gpuFrameMetricsBuffer,
  );

  const renderPipeline = createRenderPipeline(device, shaderModule);
  const renderBindGroup = createBindGroup(
    device,
    renderPipeline,
    gpuFieldBuffer,
    gpuFrameMetricsBuffer,
  );

  const directRenderPipeline = createDirectRenderPipeline(device, shaderModule);
  const directBindGroup = createDirectBindGroup(
    device,
    directRenderPipeline,
    gpuDirectBuffer,
  );

  return {
    renderField() {
      writeToGPUBuffer(device, gpuFieldBuffer, sourceFieldBuffer);

      drawFieldData(
        device,
        context,
        computePipeline,
        computeBindGroup,
        renderPipeline,
        renderBindGroup,
      );
    },

    renderDirect() {
      writeToGPUBuffer(device, gpuDirectBuffer, sourceDirectBuffer);

      drawDirect(device, context, directRenderPipeline, directBindGroup);
    },
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

function createRenderPipeline(
  device: GPUDevice,
  shaderModule: GPUShaderModule,
) {
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

function createDirectRenderPipeline(
  device: GPUDevice,
  shaderModule: GPUShaderModule,
) {
  return device.createRenderPipeline({
    layout: 'auto',
    vertex: {
      module: shaderModule,
      entryPoint: 'vertex_main',
    },
    fragment: {
      module: shaderModule,
      entryPoint: 'direct_fragment_main',
      targets: [{ format: navigator.gpu.getPreferredCanvasFormat() }],
    },
    primitive: {
      topology: 'triangle-list',
    },
  });
}

function createComputePipeline(
  device: GPUDevice,
  shaderModule: GPUShaderModule,
) {
  return device.createComputePipeline({
    layout: 'auto',
    compute: {
      module: shaderModule,
      entryPoint: 'metrics_main',
    },
  });
}

function createGPUBuffer(
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

function writeToGPUBuffer(
  device: GPUDevice,
  dest: GPUBuffer,
  source: BufferParams,
) {
  device.queue.writeBuffer(dest, 0, source.buffer, source.start, source.length);
}

function createBindGroup(
  device: GPUDevice,
  pipeline: GPURenderPipeline | GPUComputePipeline,
  gpuFieldBuffer: GPUBuffer,
  gpuFrameMetricsBuffer: GPUBuffer,
) {
  return device.createBindGroup({
    layout: pipeline.getBindGroupLayout(FIELD_BIND_GROUP_INDEX),
    entries: [
      {
        binding: FIELD_BUFFER_BINDING,
        resource: { buffer: gpuFieldBuffer },
      },
      {
        binding: FIELD_FRAME_METRICS_BUFFER_BINDING,
        resource: { buffer: gpuFrameMetricsBuffer },
      },
    ],
  });
}

function createDirectBindGroup(
  device: GPUDevice,
  pipeline: GPURenderPipeline,
  gpuDirectBuffer: GPUBuffer,
) {
  return device.createBindGroup({
    layout: pipeline.getBindGroupLayout(DIRECT_BIND_GROUP_INDEX),
    entries: [
      {
        binding: DIRECT_BUFFER_BINDING,
        resource: { buffer: gpuDirectBuffer },
      },
    ],
  });
}

function drawFieldData(
  device: GPUDevice,
  context: GPUCanvasContext,
  computePipeline: GPUComputePipeline,
  computeBindGroup: GPUBindGroup,
  renderPipeline: GPURenderPipeline,
  renderBindGroup: GPUBindGroup,
) {
  const commandEncoder = device.createCommandEncoder();

  // Compute pass to calculate metrics
  const computePassEncoder = commandEncoder.beginComputePass();
  computePassEncoder.setPipeline(computePipeline);
  computePassEncoder.setBindGroup(FIELD_BIND_GROUP_INDEX, computeBindGroup);
  computePassEncoder.dispatchWorkgroups(1);
  computePassEncoder.end();

  // Render pass
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
  passEncoder.setPipeline(renderPipeline);
  passEncoder.setBindGroup(FIELD_BIND_GROUP_INDEX, renderBindGroup);
  passEncoder.draw(3);
  passEncoder.end();

  device.queue.submit([commandEncoder.finish()]);
}

function drawDirect(
  device: GPUDevice,
  context: GPUCanvasContext,
  directRenderPipeline: GPURenderPipeline,
  directBindGroup: GPUBindGroup,
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
  passEncoder.setPipeline(directRenderPipeline);
  passEncoder.setBindGroup(DIRECT_BIND_GROUP_INDEX, directBindGroup);
  passEncoder.draw(3);
  passEncoder.end();

  device.queue.submit([commandEncoder.finish()]);
}
