function supportsWebgl(): boolean {
  try {
    const canvas = document.createElement("canvas");
    return !!window.WebGLRenderingContext && !!(canvas.getContext("webgl") || canvas.getContext("experimental-webgl"));
  } catch (_e) {
    return false;
  }
}

export const webglSupport: boolean = supportsWebgl();
