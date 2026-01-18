import shadersWGSL from './shaders.wgsl?raw';
import { BufferParams, getGPUDevice, getGPUContext } from './utils';
import { createFieldDataRenderer } from './field-data-renderer';
import { createDirectRenderer } from './direct-renderer';
import { CANVAS_HEIGHT, CANVAS_WIDTH } from './constants';

export async function initRenderers(
  canvas: HTMLCanvasElement,
  sourceFieldDataBuffer: BufferParams,
  sourceDirectBuffer: BufferParams,
) {
  canvas.width = CANVAS_WIDTH;
  canvas.height = CANVAS_HEIGHT;

  const device = await getGPUDevice();
  const context = getGPUContext(canvas, device);
  const shaderModule = device.createShaderModule({ code: shadersWGSL });

  const renderFieldData = createFieldDataRenderer(
    device,
    context,
    shaderModule,
    sourceFieldDataBuffer,
  );

  const renderDirect = createDirectRenderer(
    device,
    context,
    shaderModule,
    sourceDirectBuffer,
  );

  return { renderFieldData, renderDirect };
}
