<template>
  <div class="container-fluid" id="app">
    <div class="container-fluid" id="nav">
      <router-link class="mx-2" to="/">Gaussian Process</router-link>
      <router-link class="mx-2" to="/graphics">Graphics</router-link>
    </div>
    <transition name="fade" mode="out-in">
      <router-view class="container-fluid" />
    </transition>
  </div>
</template>

<style lang="scss">
@import url("https://fonts.googleapis.com/css2?family=Abhaya+Libre:wght@500&display=swap");
@import "~bootstrap";
@import "~bootstrap-vue";
#app {
  font-family: "Abhaya Libre", Helvetica, Arial, sans-serif;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  text-align: center;
  color: #2c3e50;
}

.fade-enter-active,
.fade-leave-active {
  transition-duration: 0.2s;
  transition-property: opacity;
  transition-timing-function: ease;
}

.fade-enter,
.fade-leave-active {
  opacity: 0;
}

#nav {
  padding: 30px;

  a {
    font-weight: bold;
    color: #42b983;

    &.router-link-exact-active {
      color: #2c3e50;
    }
  }
}
</style>
