<script lang="ts">
  import { onMount } from "svelte";
  import { session } from "./lib/stores/session.svelte";
  import { chats } from "./lib/stores/chats.svelte";
  import { startEventBridge } from "./lib/stores/events";
  import { initTheme } from "./lib/stores/theme.svelte";
  import PairingScreen from "./components/pairing/PairingScreen.svelte";
  import LoadingScreen from "./components/pairing/LoadingScreen.svelte";
  import MainLayout from "./components/chat/MainLayout.svelte";

  // Apply the saved theme before first paint so the UI doesn't flash.
  initTheme();

  onMount(() => {
    void startEventBridge();
  });

  const hasChats = $derived(chats.size > 0);
</script>

<!-- Source of truth is the handshake type: registered (IK) = logged in,
     unregistered (XX) = logged out. Show cached chats immediately (offline-ok);
     show the loading screen while a fresh, cache-less session connects, and while
     the post-handshake offline sync drains a backlog with nothing to show yet
     (so the user sees a progress bar instead of an empty list). -->
{#if !session.registered}
  <PairingScreen />
{:else if session.syncActive && !hasChats}
  <LoadingScreen />
{:else if session.connected || hasChats}
  <MainLayout />
{:else}
  <LoadingScreen />
{/if}
