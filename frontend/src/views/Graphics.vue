<template>
  <div class="container card">
    <Graphics />
  </div>
</template>

<script>
// @ is an alias to /src
import Graphics from "@/components/Graphics.vue";

export default {
  name: "GraphicsView",
  components: {
    Graphics,
  },
};
</script>

<style scoped lang="scss">
.text-box {
  h1 {
    margin: 2rem;
  }
  p {
    text-align: left;
  }
}
</style>
