<script lang="ts">
  // SvelteKit routes every failure through this one component. The
  // API's message arrives as page.error.message (apiGet turns non-OK
  // responses into error(status, message)).
  import { page } from '$app/state';
  import { APP_NAME } from '$lib/app';

  const status = $derived(page.status);
  const message = $derived(page.error?.message ?? 'Something went wrong.');
</script>

<svelte:head>
  <title>{status} | {APP_NAME}</title>
</svelte:head>

<section class="flex min-h-[60vh] flex-col items-center justify-center gap-2 text-center">
  <p class="font-mono text-6xl font-semibold">{status}</p>
  <h1 class="text-xl">{message}</h1>
  <a href="/" class="text-sm text-muted-foreground underline">Back to {APP_NAME}</a>
</section>
