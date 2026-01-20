import {
  FIELD_DATA_SOURCE_BYTES_PER_LINE,
  FIELD_DATA_FRAME_METRICS_BUFFER_SIZE,
  FIELD_DATA_BIND_GROUP_INDEX,
  FIELD_DATA_SOURCE_BUFFER_BINDING,
  FIELD_DATA_FRAME_METRICS_BUFFER_BINDING,
  FIELD_DATA_VERTEX_ENTRY,
  FIELD_DATA_FRAGMENT_ENTRY,
  FIELD_DATA_METRICS_ENTRY,
} from './constants';
import { BufferParams, createGPUBuffer, writeToGPUBuffer } from './utils';

export function createFieldDataRenderer(
  device: GPUDevice,
  context: GPUCanvasContext,
  shaderModule: GPUShaderModule,
  sourceBuffer: BufferParams,
): () => void {
  if (sourceBuffer.byteLength % FIELD_DATA_SOURCE_BYTES_PER_LINE !== 0) {
    throw new Error(`Not multiple of line size: ${sourceBuffer.byteLength}`);
  }

  const gpuSourceBuffer = createGPUBuffer(
    device,
    sourceBuffer.byteLength,
    true,
  );
  const gpuFrameMetricsBuffer = createGPUBuffer(
    device,
    FIELD_DATA_FRAME_METRICS_BUFFER_SIZE,
    false,
  );

  const computePipeline = createComputePipeline(device, shaderModule);
  const renderPipeline = createRenderPipeline(device, shaderModule);

  const computeBindGroup = createBindGroup(
    device,
    computePipeline,
    gpuSourceBuffer,
    gpuFrameMetricsBuffer,
  );

  const renderBindGroup = createBindGroup(
    device,
    renderPipeline,
    gpuSourceBuffer,
    gpuFrameMetricsBuffer,
  );

  return () => {
    writeToGPUBuffer(device, gpuSourceBuffer, sourceBuffer);

    render(
      device,
      context,
      computePipeline,
      computeBindGroup,
      renderPipeline,
      renderBindGroup,
    );
  };
}

function createComputePipeline(
  device: GPUDevice,
  shaderModule: GPUShaderModule,
) {
  return device.createComputePipeline({
    layout: 'auto',
    compute: {
      module: shaderModule,
      entryPoint: FIELD_DATA_METRICS_ENTRY,
    },
  });
}

function createRenderPipeline(
  device: GPUDevice,
  shaderModule: GPUShaderModule,
) {
  return device.createRenderPipeline({
    layout: 'auto',
    vertex: {
      module: shaderModule,
      entryPoint: FIELD_DATA_VERTEX_ENTRY,
    },
    fragment: {
      module: shaderModule,
      entryPoint: FIELD_DATA_FRAGMENT_ENTRY,
      targets: [{ format: navigator.gpu.getPreferredCanvasFormat() }],
    },
    primitive: {
      topology: 'triangle-list',
    },
  });
}

function createBindGroup(
  device: GPUDevice,
  pipeline: GPURenderPipeline | GPUComputePipeline,
  gpuSourceBuffer: GPUBuffer,
  gpuFrameMetricsBuffer: GPUBuffer,
) {
  return device.createBindGroup({
    layout: pipeline.getBindGroupLayout(FIELD_DATA_BIND_GROUP_INDEX),
    entries: [
      {
        binding: FIELD_DATA_SOURCE_BUFFER_BINDING,
        resource: { buffer: gpuSourceBuffer },
      },
      {
        binding: FIELD_DATA_FRAME_METRICS_BUFFER_BINDING,
        resource: { buffer: gpuFrameMetricsBuffer },
      },
    ],
  });
}

function render(
  device: GPUDevice,
  context: GPUCanvasContext,
  computePipeline: GPUComputePipeline,
  computeBindGroup: GPUBindGroup,
  renderPipeline: GPURenderPipeline,
  renderBindGroup: GPUBindGroup,
) {
  const commandEncoder = device.createCommandEncoder();

  // calculate offsets
  const computePassEncoder = commandEncoder.beginComputePass();
  computePassEncoder.setPipeline(computePipeline);
  computePassEncoder.setBindGroup(
    FIELD_DATA_BIND_GROUP_INDEX,
    computeBindGroup,
  );
  computePassEncoder.dispatchWorkgroups(1);
  computePassEncoder.end();

  // render
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
  passEncoder.setBindGroup(FIELD_DATA_BIND_GROUP_INDEX, renderBindGroup);
  passEncoder.draw(3);
  passEncoder.end();

  device.queue.submit([commandEncoder.finish()]);
}
