import {
  CANVAS_HEIGHT,
  CANVAS_WIDTH,
  DIRECT_BIND_GROUP_INDEX,
  DIRECT_BUFFER_BINDING,
  DIRECT_FRAGMENT_ENTRY,
  DIRECT_VERTEX_ENTRY,
} from './constants';
import { BufferParams, writeToGPUBuffer, createGPUBuffer } from './utils';

const SOURCE_SIZE = CANVAS_WIDTH * CANVAS_HEIGHT;

export function createDirectRenderer(
  device: GPUDevice,
  context: GPUCanvasContext,
  shaderModule: GPUShaderModule,
  sourceBuffer: BufferParams,
): () => void {
  if (sourceBuffer.byteLength !== SOURCE_SIZE) {
    throw new Error(`Incorrect source size: ${sourceBuffer.byteLength}`);
  }

  const gpuSourceBuffer = createGPUBuffer(device, SOURCE_SIZE, true);

  const renderPipeline = createRenderPipeline(device, shaderModule);

  const bindGroup = createBindGroup(device, renderPipeline, gpuSourceBuffer);

  return () => {
    writeToGPUBuffer(device, gpuSourceBuffer, sourceBuffer);

    render(device, context, renderPipeline, bindGroup);
  };
}

function createRenderPipeline(
  device: GPUDevice,
  shaderModule: GPUShaderModule,
) {
  return device.createRenderPipeline({
    layout: 'auto',
    vertex: {
      module: shaderModule,
      entryPoint: DIRECT_VERTEX_ENTRY,
    },
    fragment: {
      module: shaderModule,
      entryPoint: DIRECT_FRAGMENT_ENTRY,
      targets: [{ format: navigator.gpu.getPreferredCanvasFormat() }],
    },
    primitive: {
      topology: 'triangle-list',
    },
  });
}

function createBindGroup(
  device: GPUDevice,
  pipeline: GPURenderPipeline,
  gpuBuffer: GPUBuffer,
) {
  return device.createBindGroup({
    layout: pipeline.getBindGroupLayout(DIRECT_BIND_GROUP_INDEX),
    entries: [
      {
        binding: DIRECT_BUFFER_BINDING,
        resource: { buffer: gpuBuffer },
      },
    ],
  });
}

function render(
  device: GPUDevice,
  context: GPUCanvasContext,
  renderPipeline: GPURenderPipeline,
  bindGroup: GPUBindGroup,
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
  passEncoder.setPipeline(renderPipeline);
  passEncoder.setBindGroup(DIRECT_BIND_GROUP_INDEX, bindGroup);
  passEncoder.draw(3);
  passEncoder.end();

  device.queue.submit([commandEncoder.finish()]);
}
