function supportsWebgl(): boolean {
  try {
    const canvas = document.createElement("canvas");
    return !!window.WebGLRenderingContext && !!(canvas.getContext("webgl") || canvas.getContext("experimental-webgl"));
  } catch (e) {
    console.error("cannot construct webglcontext (needed to render the map) because", e)
    return false;
  }
}

export const webglSupport: boolean = supportsWebgl();
