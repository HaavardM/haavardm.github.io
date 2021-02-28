<template>
  <div class="bg-light" id="app">
    <div class="border-bottom shadow-sm bg-white" id="nav">
      <router-link to="/" class="mx-2">GaussianProcess</router-link>
      <router-link to="/anyonethere" class="mx-2">AnyoneThere</router-link>
    </div>
    <transition name="fade" mode="out-in">
      <router-view class="container-fluid my-4" />
    </transition>
  </div>
</template>

<style lang="scss">
$enable-shadows: true;
@import url("https://fonts.googleapis.com/css2?family=Abhaya+Libre:wght@500&display=swap");
@import "~bootstrap";
@import "~bootstrap-vue";
#app {
  min-height: 100vh;
  font-family: "Abhaya Libre", Helvetica, Arial, sans-serif;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  text-align: center;
  color: #2c3e50;
}

.logo {
  font-size: 2rem;
}

#nav {
  padding: 30px;

  a {
    font-weight: bold;
    font-size: 1.3rem;
    color: #42b983;

    &.router-link-exact-active {
      color: #2c3e50;
    }
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
