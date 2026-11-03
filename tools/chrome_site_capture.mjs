#!/usr/bin/env node

import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { spawn } from "node:child_process";

const [manifestPath, outputDir, widthText, heightText, dprText] = process.argv.slice(2);
if (!manifestPath || !outputDir) {
  throw new Error("usage: chrome_site_capture.mjs MANIFEST OUTPUT WIDTH HEIGHT DPR");
}
const width = Number(widthText || 1280);
const height = Number(heightText || 720);
const dpr = Number(dprText || 2);
const sites = JSON.parse(fs.readFileSync(manifestPath, "utf8"));
const extraChromeArgs = JSON.parse(process.env.CHROME_EXTRA_ARGS || "[]");
if (!Array.isArray(extraChromeArgs) || extraChromeArgs.some(value => typeof value !== "string")) {
  throw new Error("CHROME_EXTRA_ARGS must be a JSON array of strings");
}
fs.mkdirSync(outputDir, { recursive: true });
const resultsPath = path.join(outputDir, "results.json");

const profile = fs.mkdtempSync(path.join(os.tmpdir(), "rechrom-chrome-profile-"));
const chromePath = "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome";
const chrome = spawn(chromePath, [
  "--headless=new",
  "--remote-debugging-port=0",
  "--remote-allow-origins=*",
  `--user-data-dir=${profile}`,
  `--window-size=${width},${height}`,
  `--force-device-scale-factor=${dpr}`,
  "--force-color-profile=srgb",
  "--disable-background-networking",
  "--disable-component-update",
  "--disable-default-apps",
  "--disable-extensions",
  "--no-first-run",
  "--no-default-browser-check",
  ...extraChromeArgs,
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
      for (let index = 0; index < this.waiters.length; index++) {
        const waiter = this.waiters[index];
        if (waiter.method === message.method && waiter.sessionId === message.sessionId) {
          this.waiters.splice(index, 1);
          clearTimeout(waiter.timer);
          waiter.resolve(message.params || {});
          break;
        }
      }
    });
  }
  request(method, params = {}, sessionId = undefined, timeoutMs = 20000) {
    const id = this.nextId++;
    return new Promise((resolve, reject) => {
      const timer = setTimeout(() => {
        if (!this.pending.delete(id)) return;
        reject(new Error(`${method} timeout`));
      }, timeoutMs);
      this.pending.set(id, {
        resolve: value => {
          clearTimeout(timer);
          resolve(value);
        },
        reject: error => {
          clearTimeout(timer);
          reject(error);
        },
      });
      this.ws.send(JSON.stringify({ id, method, params, ...(sessionId ? { sessionId } : {}) }));
    });
  }
  wait(method, sessionId, timeoutMs = 20000) {
    return new Promise((resolve, reject) => {
      const waiter = { method, sessionId, resolve, reject };
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
const results = fs.existsSync(resultsPath)
  ? JSON.parse(fs.readFileSync(resultsPath, "utf8"))
  : [];
const completed = new Set(results.map(row => row.id));
let cdp;
try {
  cdp = new CDP(await browserWebSocket());
  await cdp.connect();
  const { targetId } = await cdp.request("Target.createTarget", { url: "about:blank" });
  const { sessionId } = await cdp.request("Target.attachToTarget", { targetId, flatten: true });
  await cdp.request("Page.enable", {}, sessionId);
  await cdp.request("Runtime.enable", {}, sessionId);
  await cdp.request("Performance.enable", {}, sessionId);
  await cdp.request("Network.enable", {}, sessionId);
  await cdp.request("DOM.enable", {}, sessionId);
  await cdp.request("CSS.enable", {}, sessionId);
  await cdp.request("Network.setUserAgentOverride", {
    userAgent: "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/154.0.0.0 Safari/537.36",
    acceptLanguage: "en-US,en;q=0.9",
    platform: "MacIntel",
  }, sessionId);
  await cdp.request("Emulation.setDeviceMetricsOverride", {
    width, height, deviceScaleFactor: dpr, mobile: false,
  }, sessionId);
  await cdp.request("Emulation.setEmulatedMedia", {
    media: "screen",
    features: [{ name: "prefers-color-scheme", value: "light" }],
  }, sessionId);

  for (const site of sites) {
    if (completed.has(site.id)) continue;
    const row = { id: site.id, url: site.url, screenshots: {}, error: null };
    try {
      const domReady = cdp.wait("Page.domContentEventFired", sessionId, 15000)
        .then(() => true, () => false);
      const navigationRequest = cdp.request("Page.navigate", { url: site.url }, sessionId)
        .then(value => ({ value }), error => ({ error }));
      const navigation = await Promise.race([
        navigationRequest,
        sleep(15000).then(() => null),
      ]);
      if (!navigation) {
        await cdp.request("Page.stopLoading", {}, sessionId);
      } else if (navigation.error) {
        throw navigation.error;
      } else if (navigation.value.errorText) {
        throw new Error(navigation.value.errorText);
      }
      if (!(await domReady)) {
        await cdp.request("Page.stopLoading", {}, sessionId);
      }
      await sleep(1500);
      await cdp.request("Runtime.evaluate", { expression: `(() => {
        let s=document.getElementById('__rechrom_compare_style');
        if(!s){s=document.createElement('style');s.id='__rechrom_compare_style';document.documentElement.appendChild(s)}
        s.textContent='*,*::before,*::after{animation:none!important;transition:none!important;caret-color:transparent!important;scroll-behavior:auto!important}';
      })()` }, sessionId);
      await sleep(250);
      const timingResponse = await cdp.request("Runtime.evaluate", {
        expression: `JSON.stringify((() => { const n=performance.getEntriesByType('navigation')[0]; const p=performance.getEntriesByType('paint'); const first=document.body?.firstElementChild; const rect=e=>e?Object.fromEntries(['x','y','width','height','top','right','bottom','left'].map(k=>[k,e.getBoundingClientRect()[k]])):null; const style=document.body?getComputedStyle(document.body):null; const headings=Array.from(document.querySelectorAll('h2')); const metric=e=>{const r=e.getBoundingClientRect(),s=getComputedStyle(e);return {tag:e.tagName,id:e.id,class:e.className,text:e.textContent.trim().slice(0,80),top:r.top+scrollY,height:r.height,display:s.display,position:s.position,grid_template_columns:s.gridTemplateColumns,grid_template_rows:s.gridTemplateRows}}; const gridHeading=headings.find(e=>e.textContent.trim()==='Grid layout in action'); return {href:location.href,title:document.title,scroll_height:document.documentElement.scrollHeight,headings:headings.map(e=>({text:e.textContent.trim(),top:e.getBoundingClientRect().top+scrollY,height:e.getBoundingClientRect().height})),grid_section:gridHeading?{section:metric(gridHeading.parentElement),children:Array.from(gridHeading.parentElement.children).map(metric)}:null,nav:n?{response_end:n.responseEnd,dom_content_loaded:n.domContentLoadedEventEnd,load:n.loadEventEnd}:null,paints:Object.fromEntries(p.map(x=>[x.name,x.startTime])),geometry:{document:rect(document.documentElement),body:rect(document.body),first:rect(first)},body_style:style?{font:style.font,font_family:style.fontFamily,font_size:style.fontSize,line_height:style.lineHeight,padding_top:style.paddingTop}:null} })())`,
        returnByValue: true,
      }, sessionId);
      row.timing = JSON.parse(timingResponse.result?.value || "{}");
      const documentNode = await cdp.request("DOM.getDocument", { depth: 0 }, sessionId);
      const bodyNode = await cdp.request("DOM.querySelector", {
        nodeId: documentNode.root.nodeId,
        selector: "body",
      }, sessionId);
      const firstNode = await cdp.request("DOM.querySelector", {
        nodeId: documentNode.root.nodeId,
        selector: "body > *",
      }, sessionId);
      row.platform_fonts = {};
      for (const [name, nodeId] of [["body", bodyNode.nodeId], ["first", firstNode.nodeId]]) {
        if (!nodeId) continue;
        row.platform_fonts[name] = (await cdp.request(
          "CSS.getPlatformFontsForNode", { nodeId }, sessionId
        )).fonts;
      }
      row.scroll_positions = {};
      for (const y of site.scrolls) {
        await cdp.request("Runtime.evaluate", { expression: `window.scrollTo(0,${Number(y)});` }, sessionId);
        await sleep(200);
        const position = await cdp.request("Runtime.evaluate", {
          expression: `JSON.stringify({actual_y:scrollY,scroll_height:document.documentElement.scrollHeight,client_height:document.documentElement.clientHeight})`,
          returnByValue: true,
        }, sessionId);
        row.scroll_positions[String(y)] = {
          requested_y: Number(y),
          ...JSON.parse(position.result?.value || "{}"),
        };
        const shot = await cdp.request("Page.captureScreenshot", {
          format: "png", fromSurface: true, captureBeyondViewport: false,
        }, sessionId);
        const filename = `${site.id}__y${y}.png`;
        fs.writeFileSync(path.join(outputDir, filename), Buffer.from(shot.data, "base64"));
        row.screenshots[String(y)] = filename;
      }
    } catch (error) {
      row.error = String(error?.stack || error);
    }
    results.push(row);
    fs.writeFileSync(resultsPath, JSON.stringify(results, null, 2));
  }
} finally {
  try { cdp?.ws?.close(); } catch {}
  try { chrome.kill("SIGTERM"); } catch {}
  await new Promise(resolve => {
    if (chrome.exitCode !== null) return resolve();
    const timer = setTimeout(resolve, 5000);
    chrome.once("exit", () => {
      clearTimeout(timer);
      resolve();
    });
  });
  for (let attempt = 0; attempt < 5; attempt++) {
    try {
      fs.rmSync(profile, { recursive: true, force: true, maxRetries: 3, retryDelay: 100 });
      break;
    } catch (error) {
      if (attempt === 4) throw error;
      await sleep(200);
    }
  }
}
