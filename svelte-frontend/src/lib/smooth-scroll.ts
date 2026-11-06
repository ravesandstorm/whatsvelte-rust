// Svelte action: eased mouse-wheel scrolling for a vertical scroll container.
//
// Design constraints (the message list is updated from both ends):
//  - Only discrete mouse-wheel notches are intercepted. Trackpads / precision
//    touchpads (small or fractional deltas) keep their native, already-smooth
//    scrolling, so they pay nothing.
//  - We never fight other code writing `scrollTop` (prepending older pages,
//    stick-to-bottom, scroll-anchor compensation). Each frame we compare the
//    real `scrollTop` with where we last put it; any difference is an external
//    adjustment, and we shift the glide target by the same amount so the glide
//    continues in the right place instead of snapping back.
//  - No work happens while idle: the rAF loop only runs during a glide, and it
//    does not touch layout other than one scrollTop read/write per frame.
//  - Honors prefers-reduced-motion.

const TAU_MS = 85; // easing time constant: lower = snappier
const LINE_PX = 40; // deltaMode=LINE → pixels
const MIN_WHOLE_NOTCH_PX = 40;
const EPS = 0.5;

export function smoothWheel(node: HTMLElement) {
  if (typeof matchMedia === "function" && matchMedia("(prefers-reduced-motion: reduce)").matches) {
    return {};
  }

  let raf = 0;
  let pos = 0; // where we last set scrollTop (float)
  let target = 0;
  let last = 0;
  let remainder = 0; // sub-pixel distance left over from the previous glide

  const max = () => node.scrollHeight - node.clientHeight;
  const clamp = (v: number) => Math.max(0, Math.min(max(), v));

  function frame(now: number) {
    // External adjustment since our last write → carry the target along.
    const actual = node.scrollTop;
    if (Math.abs(actual - pos) > 1.5) {
      const diff = actual - pos;
      pos = actual;
      target += diff;
    }
    target = clamp(target);

    // The first frame's timestamp can predate the wheel event; a negative dt
    // would step backwards.
    const dt = Math.max(8, Math.min(now - last, 64));
    last = now;
    const next = pos + (target - pos) * (1 - Math.exp(-dt / TAU_MS));
    if (Math.abs(target - next) < EPS) {
      // scrollTop is whole pixels in WKWebView. Carry the fraction into the next
      // glide, or slow notches (4px each) would lose up to a quarter of their
      // distance compared with native scrolling.
      const landed = Math.round(target);
      node.scrollTop = landed;
      remainder = target - landed;
      raf = 0;
      return;
    }
    pos = next;
    node.scrollTop = next;
    raf = requestAnimationFrame(frame);
  }

  function onWheel(e: WheelEvent) {
    if (e.ctrlKey || e.shiftKey || e.altKey || e.metaKey || e.deltaX !== 0 || e.deltaY === 0) return;
    const lines = e.deltaMode === 1;
    // A mouse wheel, as opposed to a trackpad's small whole-pixel stream, is:
    //  - line or page mode, or
    //  - a fractional delta: WKWebView reports an accelerated macOS mouse notch
    //    as e.g. 3.9996 or 11.9995, and display scaling gives WebView2 e.g. 68.57, or
    //  - a big whole number: 100 per notch on WebView2 without scaling.
    // Anything else keeps the platform's own (already smooth) scrolling.
    const isMouseWheel =
      e.deltaMode !== 0 || !Number.isInteger(e.deltaY) || Math.abs(e.deltaY) >= MIN_WHOLE_NOTCH_PX;
    if (!isMouseWheel) return;
    if (max() <= 0) return;

    const dy = lines ? e.deltaY * LINE_PX : e.deltaMode === 2 ? e.deltaY * node.clientHeight : e.deltaY;
    e.preventDefault();

    if (!raf) {
      pos = node.scrollTop;
      target = pos + remainder;
      remainder = 0;
      last = performance.now();
      raf = requestAnimationFrame(frame);
    }
    target = clamp(target + dy);
  }

  node.addEventListener("wheel", onWheel, { passive: false });
  return {
    destroy() {
      node.removeEventListener("wheel", onWheel);
      if (raf) cancelAnimationFrame(raf);
    },
  };
}
