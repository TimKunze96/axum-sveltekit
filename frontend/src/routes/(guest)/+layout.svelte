<script lang="ts">
  // The guest shell: a centered card with the logo above it.
  import type { Snippet } from 'svelte';
  import { APP_NAME } from '$lib/app';
  import AppLogoIcon from '$lib/components/app-logo-icon.svelte';

  let { children }: { children: Snippet } = $props();
</script>

<div class="flex min-h-svh flex-col items-center justify-center gap-6 bg-background p-6 md:p-10">
  <div class="w-full max-w-sm">
    <div class="flex flex-col gap-8">
      <div class="flex flex-col items-center gap-4">
        <a href="/" class="flex flex-col items-center gap-2 font-medium">
          <AppLogoIcon class="size-10 rounded-lg" />
          <span class="sr-only">{APP_NAME}</span>
        </a>
      </div>
      {@render children()}
    </div>
  </div>
</div>
