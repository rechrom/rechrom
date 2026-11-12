#!/usr/bin/env node
// Offscreen Chrome baseline for tools/resize_offscreen_probe.py.

import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { spawn } from "node:child_process";

const [url = "http://127.0.0.1:9877/python-docs/index.html", output = "work/chrome-resize-offscreen-probe.json"] = process.argv.slice(2);
const sleep = ms => new Promise(resolve => setTimeout(resolve, ms));
const profile = fs.mkdtempSync(path.join(os.tmpdir(), "rechrom-chrome-resize-"));
const chrome = spawn("/Applications/Google Chrome.app/Contents/MacOS/Google Chrome", [
  "--headless=new", "--remote-debugging-port=0", "--remote-allow-origins=*",
  `--user-data-dir=${profile}`, "--window-size=1280,720", "--force-device-scale-factor=1",
  "--force-color-profile=srgb", "--disable-background-networking", "--disable-component-update",
  "--disable-default-apps", "--disable-extensions", "--no-first-run", "--no-default-browser-check",
  "about:blank",
], { stdio: ["ignore", "ignore", "pipe"] });

function browserWebSocket() {
  return new Promise((resolve, reject) => {
    let text = "";
    const timer = setTimeout(() => reject(new Error("Chrome DevTools endpoint timeout")), 15000);
    chrome.stderr.setEncoding("utf8");
    chrome.stderr.on("data", chunk => {
      text += chunk;
      const match = text.match(/DevTools listening on (ws:\/\/[^\s]+)/);
      if (match) { clearTimeout(timer); resolve(match[1]); }
    });
    chrome.once("exit", code => reject(new Error(`Chrome exited before DevTools was ready (${code})`)));
  });
}

class CDP {
  constructor(url) { this.url = url; this.nextId = 1; this.pending = new Map(); this.listeners = new Map(); }
  async connect() {
    this.ws = new WebSocket(this.url);
    await new Promise((resolve, reject) => {
      this.ws.addEventListener("open", resolve, { once: true });
      this.ws.addEventListener("error", reject, { once: true });
    });
    this.ws.addEventListener("message", event => {
      const message = JSON.parse(event.data);
      if (message.id) {
        const pending = this.pending.get(message.id);
        if (!pending) return;
        this.pending.delete(message.id);
        if (message.error) pending.reject(new Error(JSON.stringify(message.error)));
        else pending.resolve(message.result || {});
        return;
      }
      for (const listener of this.listeners.get(message.method) || []) listener(message.params || {}, message.sessionId);
    });
  }
  on(method, listener) {
    const listeners = this.listeners.get(method) || [];
    listeners.push(listener);
    this.listeners.set(method, listeners);
  }
  request(method, params = {}, sessionId = undefined, timeoutMs = 20000) {
    const id = this.nextId++;
    return new Promise((resolve, reject) => {
      const timer = setTimeout(() => { this.pending.delete(id); reject(new Error(`${method} timeout`)); }, timeoutMs);
      this.pending.set(id, {
        resolve: value => { clearTimeout(timer); resolve(value); },
        reject: error => { clearTimeout(timer); reject(error); },
      });
      this.ws.send(JSON.stringify({ id, method, params, ...(sessionId ? { sessionId } : {}) }));
    });
  }
}

function percentile(values, fraction) {
  if (!values.length) return 0;
  const ordered = [...values].sort((a, b) => a - b);
  return ordered[Math.min(ordered.length - 1, Math.round((ordered.length - 1) * fraction))];
}

const sequence = [];
for (let width = 1280; width >= 600; width -= 20) sequence.push(width);
for (let width = 620; width <= 1280; width += 20) sequence.push(width);
const requests = [];
const frames = [];
let cdp;
try {
  cdp = new CDP(await browserWebSocket());
  await cdp.connect();
  const { targetId } = await cdp.request("Target.createTarget", { url: "about:blank" });
  const { sessionId } = await cdp.request("Target.attachToTarget", { targetId, flatten: true });
  await cdp.request("Page.enable", {}, sessionId);
  await cdp.request("Emulation.setDeviceMetricsOverride", {
    width: 1280, height: 720, deviceScaleFactor: 1, mobile: false,
  }, sessionId);
  await cdp.request("Page.navigate", { url }, sessionId);
  await sleep(2000);
  cdp.on("Page.screencastFrame", (params, eventSessionId) => {
    if (eventSessionId !== sessionId) return;
    const receivedMs = performance.now();
    frames.push({
      received_ms: receivedMs,
      width: params.metadata?.deviceWidth,
      height: params.metadata?.deviceHeight,
      timestamp: params.metadata?.timestamp,
      data_bytes: Buffer.from(params.data, "base64").length,
    });
    cdp.request("Page.screencastFrameAck", { sessionId: params.sessionId }, sessionId).catch(() => {});
  });
  await cdp.request("Page.startScreencast", { format: "png", everyNthFrame: 1 }, sessionId);
  await sleep(100);
  for (const width of sequence) {
    const sentMs = performance.now();
    await cdp.request("Emulation.setDeviceMetricsOverride", {
      width, height: 720, deviceScaleFactor: 1, mobile: false,
    }, sessionId);
    requests.push({ sent_ms: sentMs, width, height: 720 });
    await sleep(8);
  }
  await sleep(500);
  await cdp.request("Page.stopScreencast", {}, sessionId);
  const shotStarted = performance.now();
  const shot = await cdp.request("Page.captureScreenshot", {
    format: "png", fromSurface: true, captureBeyondViewport: false,
  }, sessionId);
  const shotMs = performance.now() - shotStarted;
  const matchedLatencies = [];
  for (const frame of frames) {
    const candidates = requests.filter(request => request.width === frame.width && request.sent_ms <= frame.received_ms);
    if (candidates.length) matchedLatencies.push(frame.received_ms - candidates.at(-1).sent_ms);
  }
  const intervals = frames.slice(1).map((frame, index) => frame.received_ms - frames[index].received_ms);
  const result = {
    configuration: { url, sequence: sequence.map(width => [width, 720]), interval_ms: 8 },
    requests: requests.length,
    screencast_frames: frames.length,
    unique_frame_sizes: [...new Set(frames.map(frame => `${frame.width}x${frame.height}`))].sort(),
    empty_frames: frames.filter(frame => !frame.data_bytes).length,
    request_to_matching_frame_ms: {
      p50: percentile(matchedLatencies, 0.50), p95: percentile(matchedLatencies, 0.95),
      max: Math.max(0, ...matchedLatencies), values: matchedLatencies,
    },
    frame_intervals_ms: {
      p50: percentile(intervals, 0.50), p95: percentile(intervals, 0.95),
      max: Math.max(0, ...intervals), over_16_7_ms: intervals.filter(value => value > 16.7).length,
      values: intervals,
    },
    final_capture: { width: 1280, height: 720, bytes: Buffer.from(shot.data, "base64").length, capture_ms: shotMs },
  };
  fs.mkdirSync(path.dirname(output), { recursive: true });
  fs.writeFileSync(output, JSON.stringify(result, null, 2));
  console.log(JSON.stringify(result, null, 2));
} finally {
  try { cdp?.ws?.close(); } catch {}
  try { chrome.kill("SIGTERM"); } catch {}
  await sleep(500);
  fs.rmSync(profile, { recursive: true, force: true, maxRetries: 3, retryDelay: 100 });
}
