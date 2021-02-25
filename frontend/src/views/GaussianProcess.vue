<template>
  <div class="container-fluid">
    <GaussianProcess />

    <div class="container card my-2 text-box shadow">
      <h1>Bayesian Methods: The Gaussian Process</h1>

      <h2>Work in progress</h2>
    </div>
  </div>
</template>

<script>
// @ is an alias to /src
import GaussianProcess from "@/components/GaussianProcess.vue";

export default {
  name: "GaussianProcessView",
  components: {
    GaussianProcess,
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
