<script lang="ts">
  import { onMount } from "svelte";
  import { session } from "./lib/stores/session.svelte";
  import { startEventBridge } from "./lib/stores/events";
  import PairingScreen from "./components/pairing/PairingScreen.svelte";
  import LoadingScreen from "./components/pairing/LoadingScreen.svelte";
  import MainLayout from "./components/chat/MainLayout.svelte";

  onMount(() => {
    void startEventBridge();
  });
</script>

{#if session.loggedIn}
  <MainLayout />
{:else if session.registered}
  <!-- Already paired, just reconnecting — don't flash the QR. -->
  <LoadingScreen />
{:else}
  <PairingScreen />
{/if}
