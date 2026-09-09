<script lang="ts">
  import type { Snippet } from 'svelte';
  import { page } from '$app/state';
  import AppLayout from '$lib/components/app-layout.svelte';
  import type { LayoutData } from './$types';

  let { data, children }: { data: LayoutData; children: Snippet } = $props();
</script>

<AppLayout user={data.user} sidebarOpen={data.sidebarOpen} breadcrumbs={page.data.breadcrumbs}>
  {@render children()}
</AppLayout>
