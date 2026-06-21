<script lang="ts">
  import { initials } from "../../lib/util/jid";
  import { contacts, ensureContact } from "../../lib/stores/contacts.svelte";

  let {
    label,
    jid = null,
    group = false,
    size = 44,
  }: { label: string; jid?: string | null; group?: boolean; size?: number } = $props();

  // Lazily fetch name + picture for this JID (deduped in the store).
  $effect(() => {
    if (jid) void ensureContact(jid);
  });

  const url = $derived(jid ? (contacts.get(jid)?.pictureUrl ?? null) : null);
</script>

{#if url}
  <img class="avatar" src={url} alt={label} style="width:{size}px;height:{size}px" />
{:else}
  <div class="avatar fallback" style="width:{size}px;height:{size}px;font-size:{size * 0.38}px">
    {group ? "#" : initials(label)}
  </div>
{/if}

<style>
  .avatar {
    border-radius: 50%;
    flex-shrink: 0;
    object-fit: cover;
  }
  .fallback {
    display: grid;
    place-items: center;
    background: #6a7175;
    color: #e9edef;
    font-weight: 600;
    user-select: none;
  }
</style>
