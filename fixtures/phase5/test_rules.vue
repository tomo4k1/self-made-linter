<template>
  <div v-for="item in items"> <!-- Missing :key -->
    {{ item.name }} <!-- Missing whitespace -->
  </div>
  <div>
    {{ spaced }} <!-- Extra whitespace -->
    {{ good }} <!-- Good -->
  </div>
  <div v-html="dangerous"></div> <!-- no-v-html -->
</template>

<script>
if (import.meta.client) {
  console.log('Client side');
}
if (import.meta.server) {
  console.log('Server side');
}
</script>
