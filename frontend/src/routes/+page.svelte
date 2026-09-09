<script lang="ts">
  import type { PageProps } from './$types';
  import { APP_NAME } from '$lib/app';
  import Welcome from '$lib/components/welcome.svelte';

  let { data }: PageProps = $props();
</script>

<svelte:head>
  <title>{APP_NAME}</title>
  <meta name="description" content="An Axum and SvelteKit application." />
</svelte:head>

<Welcome signedIn={data.user !== null} />
