<script lang="ts">
  // Light, dark or follow the system.
  import { Monitor, Moon, Sun } from '@lucide/svelte';
  import { getAppearance, updateAppearance } from '$lib/appearance.svelte';
  import type { Appearance, Icon } from '$lib/types';

  const tabs: { value: Appearance; icon: Icon; label: string }[] = [
    { value: 'light', icon: Sun, label: 'Light' },
    { value: 'dark', icon: Moon, label: 'Dark' },
    { value: 'system', icon: Monitor, label: 'System' },
  ];
</script>

<div class="inline-flex gap-1 rounded-lg bg-neutral-100 p-1 dark:bg-neutral-800">
  {#each tabs as tab (tab.value)}
    <button
      type="button"
      onclick={() => updateAppearance(tab.value)}
      aria-pressed={getAppearance() === tab.value}
      class="flex items-center rounded-md px-3.5 py-1.5 transition-colors {getAppearance() ===
      tab.value
        ? 'bg-white shadow-xs dark:bg-neutral-700 dark:text-neutral-100'
        : 'text-neutral-500 hover:bg-neutral-200/60 hover:text-black dark:text-neutral-400 dark:hover:bg-neutral-700/60'}"
    >
      <tab.icon class="-ml-1 h-4 w-4" />
      <span class="ml-1.5 text-sm">{tab.label}</span>
    </button>
  {/each}
</div>
