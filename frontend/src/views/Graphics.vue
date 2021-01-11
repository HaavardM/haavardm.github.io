<template>
  <div class="container-fluid">
    <div class="row">
      <div class="col card m-2 p-4"></div>
    </div>
    <div class="row">
      <div class="col-2 card mx-2">
        <div class="form-group">
          <label for="n_rectangles">#Rectangles</label>
          <input
            id="n_rectangles"
            class="form-control"
            type="number"
            v-model="count"
          />
          <div class="my-2 btn-group" id="graphics_control">
            <button type="button" class="btn btn-primary" @click="count++">
              Add
            </button>
            <button type="button" class="btn btn-primary" @click="count--">
              Remove
            </button>
          </div>
        </div>
        <div class="form-group">
          <label for="speed">Speed</label>
          <input
            id="speed"
            class="form-control"
            type="number"
            v-model="speed"
          />
          <div class="my-2 btn-group" id="graphics_control">
            <button type="button" class="btn btn-primary" @click="speed++">
              Add
            </button>
            <button type="button" class="btn btn-primary" @click="speed--">
              Remove
            </button>
          </div>
        </div>
        <div class="form-group col">
          <label for="speed">Direction</label>
          <div class="my-2 btn-group" id="graphics_control">
            <button
              type="button"
              class="btn btn-primary"
              @click="direction = 'left'"
            >
              Left
            </button>
            <button
              type="button"
              class="btn btn-primary"
              @click="direction = 'right'"
            >
              Right
            </button>
          </div>
        </div>
      </div>
      <div class="col card mx-2">
        <Graphics :n_rectangles="count" :speed="speed" :direction="direction" />
      </div>
    </div>
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
  data: function () {
    return {
      count: 10,
      speed: 100,
      direction: "right",
    };
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
