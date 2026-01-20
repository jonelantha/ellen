/// <reference types="vite/client" />

export {};

declare global {
  interface Window {
    testRender(fieldData: Uint8Array): Promise<void>;
  }
}
