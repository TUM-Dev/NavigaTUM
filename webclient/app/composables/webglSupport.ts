function supportsWebgl(): boolean {
  if (import.meta.server) return false;
  try {
    const canvas = document.createElement("canvas");
    return (
      Boolean(window.WebGLRenderingContext) &&
      Boolean(canvas.getContext("webgl") || canvas.getContext("experimental-webgl"))
    );
  } catch (e) {
    console.error("cannot construct webglcontext (needed to render the map) because", e);
    return false;
  }
}

export const webglSupport: boolean = supportsWebgl();
