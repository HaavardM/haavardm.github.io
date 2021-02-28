<template>
  <div>
    <transition name="fade" mode="out-in">
      <div
        :key="currentState.value"
        class="card shadow"
        :class="getStateClass(currentState.value)"
      >
        <div class="row my-5 py-2">
          <div class="col state-text">
            <h1 v-if="currentState">
              {{ prettyPrintState(currentState.value) }}
            </h1>
          </div>
        </div>
      </div>
    </transition>
    <div class="card shadow my-4 py-4">
      <transition-group name="fade" mode="out-in">
        <div class="row" v-for="s in history" :key="s.timestamp">
          <hr />
          <div class="col state-text">
            <p>
              {{ prettyPrintState(s.value) }}
            </p>
          </div>
          <div class="col state-timestamp px-4">
            <p>{{ prettyPrintDate(s.timestamp) }}</p>
          </div>
          <hr />
        </div>
      </transition-group>
    </div>
  </div>
</template>

<script lang="ts">
import { Component, Vue } from "vue-property-decorator";
import Axios from "axios";

interface State {
  value: string;
  timestamp: string;
}

interface Response {
  current: State;
  history: State[];
}

@Component
export default class AnyoneThere extends Vue {
  state: Response = {
    current: { value: "UNKNOWN", timestamp: "" },
    history: [],
  };
  interval: number | undefined;
  update(): void {
    Axios.get("https://anyonethere.shapingideas.fyi").then((r) => {
      this.state = r.data;
    });
  }

  created(): void {
    this.update();
    this.interval = setInterval(this.update, 500);
  }

  destroyed(): void {
    clearInterval(this.interval);
  }

  getStateClass(state: string): string {
    switch (state) {
      case "PRESENT":
        return "state-present";
      case "NOT_PRESENT":
        return "state-not-present";
      default:
        return "";
    }
  }

  prettyPrintState(state: string): string {
    switch (state) {
      case "PRESENT":
        return "🙉";
      case "NOT_PRESENT":
        return "🙈";
      default:
        return "🙊";
    }
  }

  prettyPrintDate(d: string): string {
    return new Intl.DateTimeFormat("no", {
      year: "numeric",
      month: "numeric",
      day: "numeric",
      hour: "numeric",
      minute: "numeric",
      second: "numeric",
    }).format(Date.parse(d));
  }

  get currentState(): State {
    return this.state.current;
  }

  get history(): State[] {
    return this.state.history.reverse();
  }
}
</script>

<!-- Add "scoped" attribute to limit CSS to this component only -->
<style scoped lang="scss">
$color-present: rgba(150, 255, 109, 0.705);
$color-not-present: rgba(233, 33, 59, 0.7);

h3 {
  margin: 40px 0 0;
}
ul {
  list-style-type: none;
  padding: 0;
}
li {
  display: inline-block;
  margin: 0 10px;
}
a {
  color: #42b983;
}

.bg {
  background-color: blue;
}

.state-present {
  background-color: $color-present;
}

.state-not-present {
  background-color: $color-not-present;
}

.history-state-present {
  color: $color-present;
}

.history-state-not-present {
  color: $color-not-present;
}

.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.5s;
}
.fade-enter, .fade-leave-to /* .fade-leave-active below version 2.1.8 */ {
  opacity: 0;
}

.state-text {
  p {
    font-size: 3rem;
  }

  h1 {
    font-size: 8rem;
  }
}

.state-timestamp {
  font-size: 1.5rem;
}
</style>
