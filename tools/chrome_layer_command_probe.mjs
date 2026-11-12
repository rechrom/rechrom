#!/usr/bin/env node

import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { spawn } from "node:child_process";

const [url, outputDir] = process.argv.slice(2);
if (!url || !outputDir) {
  throw new Error("usage: chrome_layer_command_probe.mjs URL OUTPUT_DIR");
}

const profile = fs.mkdtempSync(path.join(os.tmpdir(), "rechrom-chrome-layer-probe-"));
const chromePath = process.env.CHROME_PATH ||
  "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome";
const chrome = spawn(chromePath, [
  "--headless=new",
  "--remote-debugging-port=0",
  "--remote-allow-origins=*",
  `--user-data-dir=${profile}`,
  "--window-size=1280,800",
  "--force-device-scale-factor=1",
  "--force-color-profile=srgb",
  "--disable-gpu",
  "--disable-background-networking",
  "--disable-component-update",
  "--disable-default-apps",
  "--disable-extensions",
  "--no-first-run",
  "--no-default-browser-check",
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
      if (match) {
        clearTimeout(timer);
        resolve(match[1]);
      }
    });
    chrome.once("exit", code => {
      clearTimeout(timer);
      reject(new Error(`Chrome exited before DevTools was ready (${code})`));
    });
  });
}

class CDP {
  constructor(url) {
    this.url = url;
    this.nextId = 1;
    this.pending = new Map();
    this.waiters = [];
    this.latestEvents = new Map();
  }

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
      this.latestEvents.set(message.method, message.params || {});
      for (let index = 0; index < this.waiters.length; index++) {
        const waiter = this.waiters[index];
        if (waiter.method !== message.method) continue;
        this.waiters.splice(index, 1);
        clearTimeout(waiter.timer);
        waiter.resolve(message.params || {});
        break;
      }
    });
  }

  request(method, params = {}, timeoutMs = 20000) {
    const id = this.nextId++;
    return new Promise((resolve, reject) => {
      const timer = setTimeout(() => {
        if (this.pending.delete(id)) reject(new Error(`${method} timeout`));
      }, timeoutMs);
      this.pending.set(id, {
        resolve: value => { clearTimeout(timer); resolve(value); },
        reject: error => { clearTimeout(timer); reject(error); },
      });
      this.ws.send(JSON.stringify({ id, method, params }));
    });
  }

  wait(method, timeoutMs = 20000) {
    return new Promise((resolve, reject) => {
      const waiter = { method, resolve, reject };
      waiter.timer = setTimeout(() => {
        const index = this.waiters.indexOf(waiter);
        if (index >= 0) this.waiters.splice(index, 1);
        reject(new Error(`${method} timeout`));
      }, timeoutMs);
      this.waiters.push(waiter);
    });
  }
}

const sleep = ms => new Promise(resolve => setTimeout(resolve, ms));
let browser;
let page;
try {
  const browserUrl = await browserWebSocket();
  browser = new CDP(browserUrl);
  await browser.connect();
  const { targetId } = await browser.request("Target.createTarget", { url: "about:blank" });
  const endpoint = new URL(browserUrl);
  page = new CDP(`ws://${endpoint.host}/devtools/page/${targetId}`);
  await page.connect();
  await page.request("Page.enable");
  await page.request("Runtime.enable");
  await page.request("Emulation.setDeviceMetricsOverride", {
    width: 1280,
    height: 800,
    deviceScaleFactor: 1,
    mobile: false,
  });
  const loaded = page.wait("Page.loadEventFired", 30000);
  await page.request("Page.navigate", { url }, 30000);
  await loaded;
  await sleep(2000);
  await page.request("Page.captureScreenshot", { format: "png" });
  await page.request("LayerTree.enable");
  const treeChanged = page.wait("LayerTree.layerTreeDidChange", 10000);
  await page.request("Runtime.evaluate", {
    expression: `(async () => {
      document.body.style.willChange = 'transform';
      await new Promise(resolve => requestAnimationFrame(() => requestAnimationFrame(resolve)));
    })()`,
    awaitPromise: true,
  });
  const tree = await treeChanged;
  fs.mkdirSync(outputDir, { recursive: true });
  fs.writeFileSync(path.join(outputDir, "tree.json"), JSON.stringify(tree, null, 2));
  for (const layer of tree.layers || []) {
    if (!layer.drawsContent || !layer.width || !layer.height) continue;
    let snapshotId;
    try {
      ({ snapshotId } = await page.request("LayerTree.makeSnapshot", {
        layerId: layer.layerId,
      }));
      const { commandLog = [] } = await page.request("LayerTree.snapshotCommandLog", {
        snapshotId,
      });
      fs.writeFileSync(
        path.join(outputDir, `layer-${layer.layerId}.json`),
        JSON.stringify({ layer, commandLog }, null, 2),
      );
    } catch (error) {
      fs.writeFileSync(
        path.join(outputDir, `layer-${layer.layerId}.error.txt`),
        `${error}\n`,
      );
    } finally {
      if (snapshotId) {
        await page.request("LayerTree.releaseSnapshot", { snapshotId });
      }
    }
  }
} finally {
  page?.ws?.close();
  browser?.ws?.close();
  chrome.kill("SIGTERM");
}
