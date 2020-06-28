<template>
  <div class="container-fluid" id="app">
    <div class="container-fluid" id="nav">
      <router-link to="/">Gaussian Process</router-link>
    </div>
    <router-view class="container-fluid" />
  </div>
</template>

<style lang="scss">
@import url("https://fonts.googleapis.com/css2?family=Cardo&display=swap");
#app {
  font-family: "Cardo", Helvetica, Arial, sans-serif;
  font-size: 24px;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  text-align: center;
  color: #2c3e50;
}

#nav {
  padding: 30px;

  a {
    font-weight: bold;
    color: #2c3e50;

    &.router-link-exact-active {
      color: #42b983;
    }
  }
}
</style>
