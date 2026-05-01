import { onCleanup, onMount } from "solid-js";

export default function BevyWasmViewer() {
  let container: HTMLDivElement | undefined;

  onMount(() => {
    if (!container) return;

    // dynamically import the wasm pkg that you will build (adjust path after build)
    let wasmModule: any = null;
    let mounted = true;

    async function loadWasm() {
      try {
        // After building the wasm crate (wasm-pack or wasm-bindgen), replace this
        // path with the generated package path, e.g. '../bevy_wasm_pkg' or '/pkg'
        const pkg = await import("../bevy_wasm_pkg");
        if (!mounted) return;
        wasmModule = pkg;
        if (wasmModule.init) {
          await wasmModule.init("bevy-canvas");
        }
        // expose a global send function for compatibility with BevyViewer
        (window as any).__bevy_wasm_send = (json: string) => {
          if (wasmModule && wasmModule.handle_input) wasmModule.handle_input(json);
        };
        console.log("bevy-wasm loaded");
      } catch (e) {
        console.warn("failed to load bevy wasm (placeholder). Build the wasm crate and adjust import path.", e);
      }
    }

    loadWasm();

    // Let Bevy handle input on the canvas. Request pointer lock and focus
    // on user click so Bevy receives raw movement and keyboard events.
    function requestPointerLock() {
      const canvas = document.getElementById("bevy-canvas") as HTMLCanvasElement | null;
      if (canvas) {
        canvas.tabIndex = 0;
        canvas.focus();
        if ((canvas as any).requestPointerLock) (canvas as any).requestPointerLock();
      }
    }

    function onCanvasClick() {
      requestPointerLock();
    }

    container.addEventListener("click", onCanvasClick);

    onCleanup(() => {
      mounted = false;
      container?.removeEventListener("click", onCanvasClick);
    });
  });

  return (
    <div style={{ width: "100%", height: "480px", border: "1px solid #444" }}>
      <canvas id="bevy-canvas" style={{ width: "100%", height: "100%" }}></canvas>
    </div>
  );
}
