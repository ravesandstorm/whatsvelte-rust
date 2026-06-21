<script lang="ts">
  import { onMount } from "svelte";
  import { session } from "./lib/stores/session.svelte";
  import { startEventBridge } from "./lib/stores/events";
  import PairingScreen from "./components/pairing/PairingScreen.svelte";
  import MainLayout from "./components/chat/MainLayout.svelte";

  onMount(() => {
    void startEventBridge();
  });
</script>

{#if session.loggedIn}
  <MainLayout />
{:else}
  <PairingScreen />
{/if}
