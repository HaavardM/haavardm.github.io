import Axios from "axios";

const baseURL = "https://api.disruptive-technologies.com/v2";
const axios = Axios.create({ baseURL });

export interface Device {
  name: string;
  labels: object;
  type: string;
}

export interface Project {
  name: string;
  displayName: string;
}

export interface Event {
  target: string;
  data: {
    temperature?: {
      value: number;
      updateTime: string;
    };
  };
}

export function listProjects(
  username: string,
  password: string
): Promise<Project[]> {
  return axios
    .get("/projects", {
      auth: {
        username,
        password,
      },
    })
    .then((resp) => resp.data.projects);
}

export function listDevices(
  username: string,
  password: string,
  project: string,
  types: string[] = []
): Promise<Device[]> {
  const typeQuery = "device_types=" + types.join(",");
  return axios
    .get(`${project}/devices?${typeQuery}`, {
      auth: {
        username,
        password,
      },
    })
    .then((resp) => resp.data.devices);
}

export function fetchEvents(
  username: string,
  password: string,
  device: string,
  eventTypes: string[] = [],
  startTime: string | undefined = undefined
): Promise<Event[]> {
  if (!startTime) {
    const now = Date.now();
    const backTwoDays = now - 2 * 24 * 60 * 60 * 1000;

    startTime = new Date(backTwoDays).toISOString();
    console.log(startTime);
  }
  return axios
    .get(
      `${device}/events?start_time=${startTime}&event_types=` +
        eventTypes.join(","),
      {
        auth: {
          username,
          password,
        },
      }
    )
    .then((resp) => resp.data.events);
}
