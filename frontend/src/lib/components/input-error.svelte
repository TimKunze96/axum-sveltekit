<script lang="ts">
  // The inline message under a form field, from the API's 422 errors.
  let { message }: { message?: string } = $props();
</script>

{#if message}
  <p class="text-sm text-destructive" role="alert">{message}</p>
{/if}
