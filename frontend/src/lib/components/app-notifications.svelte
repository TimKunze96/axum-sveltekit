<script lang="ts">
  // The toaster in the current theme, fed by the flash notification a
  // redirect carried.
  import { toast } from 'svelte-sonner';
  import { getResolvedAppearance } from '$lib/appearance.svelte';
  import type { Notification } from '$lib/types';
  import { Toaster } from '$lib/components/ui/sonner';

  let { notification = null }: { notification?: Notification | null } = $props();

  $effect(() => {
    if (!notification) return;
    const show = notification.type === 'error' ? toast.error : toast.success;
    show(notification.title, { description: notification.body ?? undefined });
  });
</script>

<Toaster theme={getResolvedAppearance()} position="bottom-right" richColors />
