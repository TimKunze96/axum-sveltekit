<script lang="ts">
  import type { PageProps } from './$types';
  import Heading from '$lib/components/heading.svelte';
  import { pageTitle } from '$lib/title';

  let { data }: PageProps = $props();
</script>

<svelte:head>
  <title>{pageTitle('Dashboard')}</title>
</svelte:head>

<div class="flex flex-1 flex-col gap-6 p-4">
  <Heading title="Welcome, {data.user.name}." description="You are signed in. Build from here." />
</div>
