import { onCleanup, onMount } from "solid-js";

export default function BevyWasmViewer() {
  let container: HTMLDivElement | undefined;
  let canvas: HTMLCanvasElement | undefined;

  onMount(() => {
    if (!container) return;
    if (!canvas) return;

    let mounted = true;

    async function loadWasm() {
      try {
        const pkg = await import("../bevy_wasm_pkg/tracks_renderer");

        const initializer = pkg.default;
        await initializer();

        if (!mounted) return;


        if (pkg.init) {
          await pkg.init(`#${canvas?.id || "bevy-canvas"}`);
        }
        console.log("Bevy WASM initialized successfully.");
      } catch (e) {
        console.warn("Failed to load Bevy WASM.", e);
      }
    }

    loadWasm();

    // 1. Focus handler to pull user input safely into Bevy
    function focusCanvas() {
      if (canvas) {
        canvas.tabIndex = 0;
        canvas.focus();
      }
    }

    // 2. Escape handler to back out of the simulation
    function handleKeyDown(e: KeyboardEvent) {
      if (e.key === "Escape") {
        if (canvas && document.activeElement === canvas) {
          // Removes active focus from the canvas element
          canvas.blur();
          // Optional: Force focus to body or container so inputs immediately return to the web page
          document.body.focus();
          console.log("Exited Bevy canvas focus via Escape Key.");
        }
      }
    }

    container.addEventListener("click", focusCanvas);
    window.addEventListener("keydown", handleKeyDown);

    onCleanup(() => {
      mounted = false;
      container?.removeEventListener("click", focusCanvas);
      window.removeEventListener("keydown", handleKeyDown);
    });
  });

  return (
    <div
      ref={container}
      style={{
        width: "100%",
        height: "480px",
        border: "1px solid #444",
        position: "relative",
        outline: "none"
      }}
    >
      <canvas
        ref={canvas}
        id="bevy-canvas"
        style={{
          width: "100%",
          height: "100%",
          display: "block",
          outline: "none"
        }}
      ></canvas>
    </div>
  );
}