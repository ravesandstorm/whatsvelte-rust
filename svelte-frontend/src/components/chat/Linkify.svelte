<script lang="ts">
  import { openUrl } from "@tauri-apps/plugin-opener";

  let { text }: { text: string } = $props();

  type Part = { t: string; href: string | null };
  // http(s):// URLs, or bare www.* with at least one dot after it.
  const URL_RE = /(https?:\/\/[^\s<]+|www\.[^\s<]+\.[^\s<]+)/gi;

  const parts = $derived.by(() => {
    const out: Part[] = [];
    let last = 0;
    const re = new RegExp(URL_RE);
    let m: RegExpExecArray | null;
    while ((m = re.exec(text)) !== null) {
      if (m.index > last) out.push({ t: text.slice(last, m.index), href: null });
      const raw = m[0];
      // Trailing punctuation usually belongs to the sentence, not the URL.
      const url = raw.replace(/[.,;:!?)\]}>'"]+$/, "");
      out.push({ t: url, href: url.startsWith("http") ? url : `https://${url}` });
      if (url.length < raw.length) out.push({ t: raw.slice(url.length), href: null });
      last = m.index + raw.length;
    }
    if (last < text.length) out.push({ t: text.slice(last), href: null });
    return out;
  });

  // Open in the OS default browser, not the webview itself.
  function open(e: MouseEvent, href: string) {
    e.preventDefault();
    void openUrl(href);
  }
</script>

{#each parts as p}{#if p.href}<a
      class="link"
      href={p.href}
      onclick={(e) => open(e, p.href!)}>{p.t}</a>{:else}{p.t}{/if}{/each}

<style>
  .link {
    color: #53bdeb;
    text-decoration: underline;
    cursor: pointer;
    word-break: break-all;
  }
</style>
