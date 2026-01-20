import { initRenderers } from '../index.js';

export async function testRender(fieldData: Uint8Array) {
  const renderers = await initRenderers(
    document.getElementById('test-canvas') as HTMLCanvasElement,
    fieldData,
    new Uint8Array(640 * 512),
  );

  renderers.renderFieldData();

  await new Promise(requestAnimationFrame);
}

window.testRender = testRender;
