<template>
  <div class="my-4" ref="canvas"></div>
</template>

<script lang="ts">
/* eslint @typescript-eslint/no-var-requires: "off" */
import { Component, Vue, Ref } from "vue-property-decorator";
import * as PIXI from "pixi.js";

@Component
export default class Graphics extends Vue {
  @Ref("canvas") canvas?: HTMLDivElement;
  app = new PIXI.Application({
    transparent: true,
    resolution: 1,
    width: 512,
    height: 512,
  });
  mounted(): void {
    this.canvas?.appendChild(this.app.view);
    this.app.loader
      .add("map", require("@/components/owl-icon.png"))
      .load(
        (
          loader: PIXI.Loader,
          resources: Partial<Record<string, PIXI.LoaderResource>>
        ) => {
          if (resources.map) {
            const map = new PIXI.Sprite(resources.map.texture);
            this.app.stage.addChild(map);
            map.anchor.set(0.5);
            map.x = this.app.screen.width / 2;
            map.y = this.app.screen.height / 2;
            this.app.ticker.add((delta) => {
              map.rotation -= 0.01 * delta;
            });
          }
        }
      );
    this.app.resizeTo = this.canvas as HTMLElement;
    this.app.render();
  }
}
</script>

<!-- Add "scoped" attribute to limit CSS to this component only -->
<style scoped lang="scss"></style>
