import shadersWGSL from './shaders.wgsl?raw';

const CANVAS_WIDTH = 640;
const CANVAS_HEIGHT = 512;

export async function initRenderer(
  canvas: HTMLCanvasElement,
): Promise<() => void> {
  canvas.width = CANVAS_WIDTH;
  canvas.height = CANVAS_HEIGHT;

  const device = await getGPUDevice();

  const context = getGPUContext(canvas, device);

  const pipeline = createPipeline(device);

  return function renderFrame() {
    draw(device, context, pipeline);
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

function draw(
  device: GPUDevice,
  context: GPUCanvasContext,
  pipeline: GPURenderPipeline,
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
  passEncoder.draw(3);
  passEncoder.end();

  device.queue.submit([commandEncoder.finish()]);
}
