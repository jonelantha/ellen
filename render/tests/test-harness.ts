import {
  initCanvas,
  getGPUContext,
  createFieldDataRenderer,
} from '../index.js';

export async function testRender(fieldData: Uint8Array) {
  const canvas = document.getElementById('test-canvas') as HTMLCanvasElement;

  initCanvas(canvas);

  const gpuContext = await getGPUContext(canvas);

  const renderFieldData = createFieldDataRenderer(gpuContext, fieldData);

  renderFieldData();

  await new Promise(requestAnimationFrame);
}

window.testRender = testRender;
