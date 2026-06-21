<script lang="ts">
  import { api } from "../../lib/ipc";

  let phone = $state("");
  let customCode = $state("");
  let code = $state<string | null>(null);
  let error = $state<string | null>(null);
  let loading = $state(false);

  async function submit() {
    error = null;
    code = null;
    loading = true;
    try {
      const r = await api.authStartPairCode(phone.trim(), customCode.trim() || null);
      code = r.code;
    } catch (e: unknown) {
      const err = e as { message?: string; code?: string };
      error = err?.message ?? err?.code ?? String(e);
    } finally {
      loading = false;
    }
  }
</script>

<div class="pair">
  <p class="hint">Or link with your phone number</p>
  <div class="row">
    <input placeholder="Phone with country code, e.g. 15551234567" bind:value={phone} />
  </div>
  <div class="row">
    <input placeholder="Custom code (optional)" bind:value={customCode} maxlength="8" />
    <button onclick={submit} disabled={loading || !phone.trim()}>
      {loading ? "…" : "Get code"}
    </button>
  </div>

  {#if code}
    <div class="code">{code}</div>
    <p class="hint">Enter this in WhatsApp → Linked Devices → Link with phone number</p>
  {/if}
  {#if error}
    <div class="error">{error}</div>
  {/if}
</div>

<style>
  .pair {
    width: 100%;
    max-width: 320px;
  }
  .hint {
    color: var(--wa-text-muted);
    font-size: 13px;
    text-align: center;
  }
  .row {
    display: flex;
    gap: 8px;
    margin: 6px 0;
  }
  input {
    flex: 1;
    min-width: 0;
    padding: 9px 12px;
    border-radius: 8px;
    border: 1px solid var(--wa-border);
    background: var(--wa-panel-2);
  }
  button {
    padding: 9px 14px;
    border: none;
    border-radius: 8px;
    background: var(--wa-green);
    color: #04221c;
    font-weight: 600;
  }
  button:disabled {
    opacity: 0.5;
  }
  .code {
    text-align: center;
    font-size: 28px;
    letter-spacing: 6px;
    font-weight: 700;
    color: var(--wa-green);
    margin: 10px 0 4px;
  }
  .error {
    color: #f15c6d;
    font-size: 13px;
    text-align: center;
    margin-top: 6px;
  }
</style>
