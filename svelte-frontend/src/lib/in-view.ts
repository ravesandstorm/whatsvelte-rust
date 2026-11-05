// Svelte action: call `onVisible` once, the first time the node comes within
// `ROOT_MARGIN` of the viewport. One shared IntersectionObserver serves every
// node, so a long list (e.g. hundreds of chat rows) costs a single observer.

const ROOT_MARGIN = "200px";
const callbacks = new WeakMap<Element, () => void>();
let observer: IntersectionObserver | null = null;

function getObserver(): IntersectionObserver {
  if (observer) return observer;
  observer = new IntersectionObserver(
    (entries) => {
      for (const e of entries) {
        if (!e.isIntersecting) continue;
        const cb = callbacks.get(e.target);
        observer!.unobserve(e.target);
        callbacks.delete(e.target);
        cb?.();
      }
    },
    { rootMargin: ROOT_MARGIN },
  );
  return observer;
}

export function inView(node: Element, onVisible: () => void) {
  if (typeof IntersectionObserver === "undefined") {
    onVisible();
    return {};
  }
  callbacks.set(node, onVisible);
  getObserver().observe(node);
  return {
    destroy() {
      callbacks.delete(node);
      observer?.unobserve(node);
    },
  };
}
