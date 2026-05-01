import { onCleanup, onMount } from "solid-js";
import { invoke } from "@tauri-apps/api";

type InputCommand =
  | { type: "MouseMove"; dx: number; dy: number }
  | { type: "MouseButton"; button: string; pressed: boolean }
  | { type: "Scroll"; delta: number }
  | { type: "Key"; key: string; pressed: boolean };

declare global {
  interface Window {
    __bevy_wasm_send?: (json: string) => void;
  }
}

function sendCommand(cmd: InputCommand) {
  const json = JSON.stringify(cmd).replace(/"type"/, '"variant"');
  // prefer wasm direct call if available
  if (window.__bevy_wasm_send) {
    window.__bevy_wasm_send(JSON.stringify(cmd));
    return;
  }
  // fallback to Tauri invoke which calls into native Bevy
  invoke("bevy_send", { cmd_json: JSON.stringify(cmd) }).catch((e) => {
    console.warn("bevy_send failed:", e);
  });
}

export default function BevyViewer() {
  let canvas: HTMLDivElement | undefined;

  onMount(() => {
    if (!canvas) return;

    let lastX = 0;
    let lastY = 0;

    function onMouseMove(e: MouseEvent) {
      const dx = e.movementX || e.clientX - lastX;
      const dy = e.movementY || e.clientY - lastY;
      lastX = e.clientX;
      lastY = e.clientY;
      sendCommand({ type: "MouseMove", dx, dy });
    }

    function onMouseDown(e: MouseEvent) {
      const btn = e.button === 0 ? "Left" : e.button === 1 ? "Middle" : "Right";
      sendCommand({ type: "MouseButton", button: btn, pressed: true });
    }

    function onMouseUp(e: MouseEvent) {
      const btn = e.button === 0 ? "Left" : e.button === 1 ? "Middle" : "Right";
      sendCommand({ type: "MouseButton", button: btn, pressed: false });
    }

    function onWheel(e: WheelEvent) {
      sendCommand({ type: "Scroll", delta: e.deltaY > 0 ? 1 : -1 });
    }

    function onKeyDown(e: KeyboardEvent) {
      sendCommand({ type: "Key", key: e.key, pressed: true });
    }

    function onKeyUp(e: KeyboardEvent) {
      sendCommand({ type: "Key", key: e.key, pressed: false });
    }

    canvas.addEventListener("mousemove", onMouseMove);
    canvas.addEventListener("mousedown", onMouseDown);
    window.addEventListener("mouseup", onMouseUp);
    canvas.addEventListener("wheel", onWheel, { passive: true });
    window.addEventListener("keydown", onKeyDown);
    window.addEventListener("keyup", onKeyUp);

    onCleanup(() => {
      canvas?.removeEventListener("mousemove", onMouseMove);
      canvas?.removeEventListener("mousedown", onMouseDown);
      window.removeEventListener("mouseup", onMouseUp);
      canvas?.removeEventListener("wheel", onWheel);
      window.removeEventListener("keydown", onKeyDown);
      window.removeEventListener("keyup", onKeyUp);
    });
  });

  return (
    <div
      ref={canvas}
      style={{ width: "100%", height: "480px", border: "1px solid #444" }}
      tabindex={0}
    >
      {/* Canvas/WASM-bevy will attach to this element or native bevy will open separate window */}
      <div style={{ padding: "8px", color: "#ccc" }}>
        Bevy viewer (canvas for web, native window for desktop)
      </div>
    </div>
  );
}
