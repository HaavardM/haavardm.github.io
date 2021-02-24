<template>
  <div class="container-fluid">
    <AnyoneThere />
  </div>
</template>

<script>
// @ is an alias to /src
import AnyoneThere from "@/components/AnyoneThere.vue";

export default {
  name: "AnyoneThereView",
  components: {
    AnyoneThere,
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
