<script lang="ts">
  // The "Platform" group of the sidebar.
  import { page } from '$app/state';
  import { isCurrentUrl } from '$lib/current-url';
  import type { NavItem } from '$lib/types';
  import * as Sidebar from '$lib/components/ui/sidebar';

  let { items }: { items: NavItem[] } = $props();
</script>

<Sidebar.Group class="px-2 py-0">
  <Sidebar.GroupLabel>Platform</Sidebar.GroupLabel>
  <Sidebar.Menu>
    {#each items as item (item.title)}
      <Sidebar.MenuItem>
        <Sidebar.MenuButton
          isActive={isCurrentUrl(item.href, page.url.pathname)}
          tooltipContent={item.title}
        >
          {#snippet child({ props })}
            <a {...props} href={item.href}>
              {#if item.icon}
                <item.icon />
              {/if}
              <span>{item.title}</span>
            </a>
          {/snippet}
        </Sidebar.MenuButton>
      </Sidebar.MenuItem>
    {/each}
  </Sidebar.Menu>
</Sidebar.Group>
