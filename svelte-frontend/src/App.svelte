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
     unregistered (XX) = logged out. While we rehydrate the cached chats from
     IndexedDB show the loading screen (with a progress bar); then show the chats
     (offline-ok), or a connecting screen for a fresh cache-less session. -->
{#if !session.registered}
  <PairingScreen />
{:else if session.hydrating}
  <LoadingScreen />
{:else if session.connected || hasChats}
  <MainLayout />
{:else}
  <LoadingScreen />
{/if}
