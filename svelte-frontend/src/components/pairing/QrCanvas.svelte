<script lang="ts">
  import QRCode from "qrcode";

  let { code }: { code: string } = $props();
  let canvas: HTMLCanvasElement | undefined = $state();

  // Re-render whenever the backend pushes a fresh ref (~every 20s).
  $effect(() => {
    if (canvas && code) {
      QRCode.toCanvas(canvas, code, { width: 264, margin: 1 }, (err) => {
        if (err) console.error("QR render failed", err);
      });
    }
  });
</script>

<canvas bind:this={canvas} width="264" height="264"></canvas>

<style>
  canvas {
    background: #fff;
    border-radius: 8px;
    padding: 8px;
  }
</style>
