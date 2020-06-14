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
    target: string
    data: {
        temperature?: {
            value: number
            updateTime: string
        }
    }
}

export function listProjects(username: string, password: string): Promise<Project[]> {
    return axios.get("/projects", {
        auth: {
            username,
            password
        }
    }).then(resp => resp.data.projects);
}

export function listDevices(username: string, password: string, project: string, types: string[] = []): Promise<Device[]> {
    const typeQuery = "device_types=" + types.join(",")
    return axios.get(`${project}/devices?${typeQuery}`, {
        auth: {
            username,
            password
        }
    }).then(resp => resp.data.devices);
}

export function fetchEvents(username: string, password: string, device: string, eventTypes: string[] = []): Promise<Event[]> {
    return axios.get(`${device}/events`, {
        auth: {
            username,
            password
        }
    }).then(resp => resp.data.events);
}



