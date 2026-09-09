<script lang="ts">
  // The inset header: the sidebar toggle and the breadcrumbs.
  import type { BreadcrumbItem } from '$lib/types';
  import * as Sidebar from '$lib/components/ui/sidebar';
  import Breadcrumbs from './breadcrumbs.svelte';

  let { breadcrumbs = [] }: { breadcrumbs?: BreadcrumbItem[] } = $props();
</script>

<header
  class="flex h-16 shrink-0 items-center gap-2 border-b border-sidebar-border/70 px-6 transition-[width,height] ease-linear group-has-data-[collapsible=icon]/sidebar-wrapper:h-12 md:px-4"
>
  <div class="flex items-center gap-2">
    <Sidebar.Trigger class="-ml-1" />
    {#if breadcrumbs.length > 0}
      <Breadcrumbs {breadcrumbs} />
    {/if}
  </div>
</header>
