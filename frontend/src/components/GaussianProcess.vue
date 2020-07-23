<template>
  <div>
    <div class="container card">
      <b-modal id="userinputform" title="Disruptive Technologies API">
        <div class="container">
          <form v-if="projects">
            <div class="form-group">
              <label for="usernameInput">Username</label>
              <input
                class="form-control"
                id="usernameInput"
                @change="onUserInfo()"
                type="text"
                v-model="username"
                placeholder="Enter Username"
              />
              <label for="passwordInput">Password</label>
              <input
                id="passwordInput"
                class="form-control"
                @change="onUserInfo()"
                type="password"
                v-model="password"
                placeholder="Enter Password"
              />
            </div>
            <div class="form-group">
              <label for="projectSelect">Project</label>
              <select
                class="form-control"
                id="projectSelect"
                v-model="selectedProject"
                @change="onProjectSelection()"
                placeholder="Select project"
              >
                <option
                  v-for="project in projects"
                  :key="project.name"
                  :value="project"
                >{{ project.displayName }}</option>
              </select>
              <label for="deviceSelect">Device</label>
              <select
                id="deviceSelect"
                v-model="selectedDevice"
                @change="onDeviceSelection()"
                class="form-control"
              >
                <option
                  :key="device.name"
                  v-for="device in devices"
                  :value="device"
                >{{ device.labels.name }}</option>
              </select>
            </div>
          </form>
        </div>
      </b-modal>
      <div class="mx-md-2">
        <div class="row my-4">
          <div class="col">
            <label for="measurementNoise">Measurement Noise</label>
            <input
              id="measurementNoise"
              class="form-control"
              @change="doGP"
              step="0.00005"
              type="number"
              v-model="noiseY"
            />
          </div>
          <div class="col">
            <label for="lengthScale">Length Scale</label>
            <input
              id="lengthScale"
              class="form-control"
              @change="doGP"
              step="0.00005"
              type="number"
              v-model="lengthScale"
            />
          </div>
          <div class="col">
            <label for="lengthScalePeriodic">Length Scale Periodic</label>
            <input
              id="lengthScalePeriodic"
              class="form-control"
              @change="doGP"
              step="0.00005"
              type="number"
              v-model="lengthScalePeriodic"
            />
          </div>
          <div class="col">
            <label for="amplitude">Amplitude</label>
            <input
              id="amplitude"
              class="form-control"
              @change="doGP"
              step="0.00005"
              type="number"
              v-model="amplitude"
            />
          </div>
          <div class="col">
            <label for="period">Period</label>
            <input
              id="period"
              class="form-control"
              @change="doGP"
              step="0.00005"
              type="number"
              v-model="period"
            />
          </div>
        </div>
        <div class="my-4" style="position: relative; height:100%; width:100%">
          <canvas ref="canvas" id="cv"></canvas>
        </div>
      </div>
    </div>
    <div class="my-4 container card">
      <b-button class="m-2" v-b-modal.userinputform>DT</b-button>
    </div>
  </div>
</template>

<script lang="ts">
import { Component, Vue, Ref } from "vue-property-decorator";
import Chart from "chart.js";
import "chartjs-plugin-zoom";
import { x as dummyX, y as dummyY } from "@/modules/dummydata";
import {
  listProjects,
  Project,
  Device,
  listDevices,
  fetchEvents
} from "@/modules/DTAPI";
const rust = import("../../../dtgaussprocess/pkg/dtgaussprocess");

interface DataPoint {
  timestamp: number;
  value: number;
}

interface Point {
  x: number;
  y: number;
}

@Component
export default class GaussianProcess extends Vue {
  @Ref("canvas") canvas!: HTMLCanvasElement;

  lengthScale = 1;
  lengthScalePeriodic = 1;
  amplitude = 1;
  period = 1;
  username = "";
  password = "";

  projects: Project[] = [];

  devices: Device[] = [];

  selectedProject: Project | null = null;
  selectedDevice: Device | null = null;

  x: number[] = dummyX;
  y: number[] = dummyY;
  noiseY = Math.round(Math.pow(0.4 / 2, 2) * 100000) / 100000;

  chart: Chart | undefined;

  get sampleX(): number[] {
    const N = Math.ceil(-1 * this.x[0]);
    return [
      ...[...Array(N).keys()].map(i => -1 * i).reverse(),
      ...[...Array(Math.ceil((this.lengthScale * 60) / 2 / 15)).keys()].map(
        i => i + 1
      )
    ].flatMap(x => [...Array(10).keys()].reverse().map(i => x - i / 10));
  }

  mounted() {
    this.username = sessionStorage.getItem("username") || "";
    this.password = sessionStorage.getItem("password") || "";
    this.onUserInfo();
    this.doGP();
    const ctx = this.canvas.getContext("2d");
    if (ctx) {
      this.chart = new Chart(ctx, {
        type: "line",
        options: {
          aspectRatio: 40 / 38,
          scales: {
            xAxes: [
              {
                type: "linear",
                ticks: {
                  suggestedMax: 0,
                  suggestedMin: -20,
                  autoSkip: false
                }
              }
            ]
          },
          legend: {
            display: false
          },
          maintainAspectRatio: false,
          plugins: {
            zoom: {
              pan: {
                enabled: true,
                mode: "xy"
              },
              zoom: {
                enabled: true,
                mode: "xy"
              }
            }
          },
          onClick: this.onClick
        }
      });
    }
  }

  onClick(
    event: MouseEvent,
    activeElements: { _datasetIndex: number; _index: number }[]
  ) {
    for (const element of activeElements) {
      if (element._datasetIndex === 0) {
        this.x = this.x.filter((_, i) => i !== element._index);
        this.y = this.y.filter((_, i) => i !== element._index);
        this.doGP();
        break;
      }
    }
  }

  onUserInfo() {
    if (this.username && this.password) {
      listProjects(this.username, this.password)
        .then(p => {
          this.projects = p;
          if (this.projects.length > 0) {
            this.selectedProject = this.projects[0];
            this.onProjectSelection();
          }
        })
        .then(() => {
          sessionStorage.setItem("username", this.username);
          sessionStorage.setItem("password", this.password);
        });
    }
  }

  onProjectSelection() {
    if (this.selectedProject) {
      listDevices(this.username, this.password, this.selectedProject.name, [
        "temperature"
      ]).then(d => {
        this.devices = d;
        if (this.devices.length > 0) {
          this.selectedDevice = this.devices[0];
          this.onDeviceSelection();
        }
      });
    }
  }

  onDeviceSelection() {
    if (this.selectedDevice) {
      fetchEvents(this.username, this.password, this.selectedDevice.name, [
        "temperature"
      ])
        .then(events =>
          events.length
            ? events
            : Promise.reject(
                `No events found for ${this.selectedDevice?.name} `
              )
        )
        .then(events =>
          events.map(e => {
            if (!e.data.temperature) {
              return null;
            }
            return {
              timestamp:
                new Date(e.data.temperature.updateTime).getTime() / 1000,
              value: e.data.temperature.value
            };
          })
        )
        .then(events => {
          const filtered = events.filter(e => e) as DataPoint[];
          this.y = filtered.map(e => e.value);
          const N = this.y.length;
          const idx = [...Array(N).keys()];
          this.x = idx.map(i => -1 * i).reverse();
          this.doGP();
        })
        .catch(console.error);
    }
  }

  doGP() {
    rust.then(rust => {
      const gp = rust.GaussianProcess.new(
        Float64Array.from(this.x),
        Float64Array.from(this.y),
        (this.lengthScale * 60) / 15,
        (this.lengthScalePeriodic * 60) / 15,
        this.amplitude,
        (this.period * 60) / 15,
        this.noiseY
      );
      const post = gp.posterior(Float64Array.from(this.sampleX));
      const time: number[] = this.sampleX.map(x => (x * 15) / 60);
      const createPoint = (y: number, i: number) => {
        return { y, x: time[i] };
      };
      const mean: Point[] = Array.from(post.mean()).map(createPoint);
      const ciLow: Point[] = Array.from(post.ci_low()).map(createPoint);
      const ciHigh: Point[] = Array.from(post.ci_high()).map(createPoint);
      const measurements: Point[] = this.y.map((y, i) => {
        return { x: (this.x[i] * 15) / 60, y };
      });

      if (this.chart) {
        if (this.chart.data.datasets?.length === 4) {
          this.chart.data.datasets[0].data = measurements;
          this.chart.data.datasets[1].data = mean;
          this.chart.data.datasets[2].data = ciLow;
          this.chart.data.datasets[3].data = ciHigh;
        } else {
          this.chart.data.datasets = [
            {
              data: measurements,
              pointBackgroundColor: "rgba(0,0,0,0.3)",
              pointBorderColor: "#000000",
              pointBorderWidth: 2,
              showLine: false,
              pointRadius: 5,
              fill: false
            },
            {
              data: mean,
              label: "Posterior Mean",
              fill: false,
              borderColor: "#F0522C",
              pointRadius: 0
            },
            {
              data: ciLow,
              fill: false,
              borderColor: "rgba(0, 0, 255, 0.5)",
              pointRadius: 0
            },
            {
              data: ciHigh,
              fill: "-1",
              borderColor: "rgba(0, 0, 255, 0.5)",
              backgroundColor: "rgba(0, 0, 255, 0.1)",
              pointRadius: 0
            }
          ];
        }
        let max = ciHigh.map(l => l.y).reduce((f, s) => Math.max(f, s));
        let min = ciLow.map(l => l.y).reduce((f, s) => Math.min(f, s));
        const diff = max - min;
        max = max + 0.1 * diff;
        min = min - 0.1 * diff;
        if (this.chart.options.plugins?.zoom?.pan) {
          this.chart.options.plugins.zoom.pan = {
            ...this.chart.options.plugins.zoom.pan,
            rangeMin: {
              x: time[0],
              y: min
            },
            rangeMax: {
              x: time[time.length - 1],
              y: max
            }
          };
        }
        if (this.chart.options.plugins?.zoom?.zoom) {
          this.chart.options.plugins.zoom.zoom = {
            ...this.chart.options.plugins.zoom.zoom,
            rangeMin: {
              x: time[0],
              y: min
            },
            rangeMax: {
              x: time[time.length - 1],
              y: max
            }
          };
        }
        console.log(this.chart.options.plugins?.zoom.zoom);
        this.chart.update();
      }
    });
  }
}
</script>

<!-- Add "scoped" attribute to limit CSS to this component only -->
<style scoped lang="scss">
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
</style>
