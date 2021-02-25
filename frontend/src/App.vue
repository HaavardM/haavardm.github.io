<template>
  <div class="container-fluid" id="app">
    <div class="container-fluid" id="nav">
      <router-link to="/" class="mx-2">Gaussian Process</router-link>
      <router-link to="/anyonethere" class="mx-2">AnyoneThere</router-link>
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

#nav {
  padding: 30px;

  a {
    font-weight: bold;
    color: #42b983;

    &.router-link-exact-active {
      color: #2c3e50;
    }
  }

  router-link {
    margin: 0, 2;
  }
}
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.4s;
}
.fade-enter, .fade-leave-to /* .fade-leave-active below version 2.1.8 */ {
  opacity: 0;
}
</style>
