<script lang="ts">
  // The settings section: the heading and the side navigation next to
  // the page's form.
  import type { Snippet } from 'svelte';
  import { page } from '$app/state';
  import { isCurrentOrParentUrl } from '$lib/current-url';
  import type { NavItem } from '$lib/types';
  import { buttonVariants } from '$lib/components/ui/button';
  import { Separator } from '$lib/components/ui/separator';
  import Heading from './heading.svelte';

  let { children }: { children: Snippet } = $props();

  const sidebarNavItems: NavItem[] = [
    { title: 'Profile', href: '/settings/profile' },
    { title: 'Password', href: '/settings/password' },
    { title: 'Appearance', href: '/settings/appearance' },
  ];
</script>

<div class="px-4 py-6">
  <Heading title="Settings" description="Manage your account settings" />

  <div class="flex flex-col lg:flex-row lg:space-x-12">
    <aside class="w-full max-w-xl lg:w-48">
      <nav class="flex flex-col space-y-1 space-x-0" aria-label="Settings">
        {#each sidebarNavItems as item (item.href)}
          <a
            href={item.href}
            class="{buttonVariants({
              variant: 'ghost',
            })} w-full justify-start {isCurrentOrParentUrl(item.href, page.url.pathname)
              ? 'bg-muted'
              : ''}"
          >
            {item.title}
          </a>
        {/each}
      </nav>
    </aside>

    <Separator class="my-6 lg:hidden" />

    <div class="flex-1 md:max-w-2xl">
      <section class="max-w-xl space-y-12">
        {@render children()}
      </section>
    </div>
  </div>
</div>
