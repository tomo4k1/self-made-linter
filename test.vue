<template>
  <div v-html="rawContent">
    <h1>Hello World</h1>
  </div>
</template>

<script lang="ts">
/* console.log */
const a: number = 1;
const env = import.meta.env.NODE_ENV;
</script>

<style>
h1 { color: red; }
</style>
