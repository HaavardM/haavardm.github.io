<template>
  <div class="my-4" ref="canvas"></div>
</template>

<script lang="ts">
/* eslint @typescript-eslint/no-var-requires: "off" */
import { Component, Vue, Ref, Prop, Watch } from "vue-property-decorator";
import * as PIXI from "pixi.js";

PIXI.Loader.shared.add("map", require("@/components/owl-icon.png"));

function getRandomColor(): number {
  return Math.floor(Math.random() * 0xffffff);
}

@Component
export default class Graphics extends Vue {
  @Ref("canvas") readonly canvas?: HTMLDivElement;
  @Prop({ default: 10 }) readonly n_rectangles!: number;
  @Prop({ default: 100 }) readonly speed!: number;
  @Prop({ default: "right" }) readonly direction!: "right" | "left";

  time = 0;
  app = new PIXI.Application({
    transparent: true,
    resolution: 1,
    sharedLoader: true,
  });

  @Watch("n_rectangles", {
    immediate: true,
  })
  on_n_rectangles_change(new_value: number): void {
    if (new_value == this.rectangles.length || new_value < 0) {
      console.log("equal", new_value, this.rectangles.length);
      return;
    }
    while (new_value > this.rectangles.length) {
      console.log("adding", new_value, this.rectangles.length);
      this.add_rectangle(getRandomColor());
      this.on_n_rectangles_change(new_value);
    }
    while (new_value < this.rectangles.length) {
      console.log("removing", new_value, this.rectangles.length);
      const rect = this.rectangles.pop();
      if (rect) {
        this.rectangles_container.removeChild(rect);
        rect.destroy();
      }
    }
  }

  rectangles: PIXI.Graphics[] = [];
  rectangles_container = new PIXI.Container();
  mounted(): void {
    this.canvas?.appendChild(this.app.view);
    this.app.resizeTo = this.canvas as HTMLElement;
    this.app.stage.addChild(this.rectangles_container);
    this.app.loader.load(
      (_, resources: Partial<Record<string, PIXI.LoaderResource>>) =>
        this.add_owl(resources)
    );
    this.app.ticker.add((delta) => {
      this.time += delta;
    });

    this.app.ticker.add(() => {
      console.log(this.direction_multiplier);
      const N = this.rectangles.length;
      const width = this.width / N;
      this.rectangles.forEach((rectangle, i) => {
        rectangle.width = width;
        rectangle.x = i * width;
        rectangle.height =
          this.height *
          (1 -
            Math.pow(
              Math.sin(
                Math.PI *
                  ((this.direction_multiplier * this.time * this.speed) / 1e4 +
                    i / N)
              ),
              2
            ));
        rectangle.y = this.height - rectangle.height;
        rectangle.visible = true;
      });
    });
  }

  get height(): number {
    return this.app.screen.height;
  }
  get width(): number {
    return this.app.screen.width;
  }

  get direction_multiplier() {
    return this.direction === "right" ? -1 : 1;
  }

  destroyed(): void {
    this.app.destroy();
    console.log("destroyed");
  }

  add_owl(resources: Partial<Record<string, PIXI.LoaderResource>>): void {
    if (resources.map) {
      const map = new PIXI.Sprite(resources.map.texture);
      map.zIndex = 1;
      this.app.stage.addChild(map);
      map.anchor.set(0.5);
      map.x = this.width / 2;
      map.y = this.height / 2;
      this.app.ticker.add((delta) => {
        map.rotation -= (Math.PI / 100) * delta;
        map.x =
          this.width / 2 +
          (this.width / 3) * Math.cos((6 * Math.PI * this.time) / 1000);
        map.y =
          this.height / 2 +
          (this.height / 3) * Math.sin((6 * Math.PI * this.time) / 1000);
      });
    }
  }

  add_rectangle(color = 0xcc66ff, offset = 0): void {
    const y = this.app.screen.height / 2;
    const rect = new PIXI.Graphics();
    let N = this.rectangles.push(rect);
    rect.beginFill(color);
    rect.drawRect(0, 0, this.width / N, y);
    rect.visible = false;
    this.rectangles_container.addChild(rect);
  }
}
</script>

<!-- Add "scoped" attribute to limit CSS to this component only -->
<style scoped lang="scss"></style>
