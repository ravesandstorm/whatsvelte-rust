<script lang="ts">
  import { initials } from "../../lib/util/jid";
  import { contacts, ensureContact } from "../../lib/stores/contacts.svelte";
  import { canonicalJid } from "../../lib/stores/lid";
  import { inView } from "../../lib/in-view";

  let {
    label,
    jid = null,
    group = false,
    size = 44,
  }: { label: string; jid?: string | null; group?: boolean; size?: number } = $props();

  // Only fetch name + picture (two server lookups) and show the image once the
  // avatar is near the viewport. The chat list isn't virtualized, so eagerly
  // fetching every row meant a lookup burst and hundreds of decoded images held
  // in memory for rows nobody scrolled to. Deduped in the store.
  let seen = $state(false);
  $effect(() => {
    if (jid && seen) void ensureContact(jid);
  });

  const url = $derived(
    jid && seen ? (contacts.get(canonicalJid(jid))?.pictureUrl ?? null) : null,
  );
</script>

{#if url}
  <img
    class="avatar"
    src={url}
    alt={label}
    decoding="async"
    style="width:{size}px;height:{size}px"
  />
{:else}
  <div
    class="avatar fallback"
    use:inView={() => (seen = true)}
    style="width:{size}px;height:{size}px;font-size:{size * 0.38}px">
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
