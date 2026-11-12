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
const preserveMotion = process.env.CHROME_PRESERVE_MOTION === "1";
const settleMs = Number(process.env.CHROME_SETTLE_MS || 1500);
const visibleImageTimeoutMs = Number(process.env.CHROME_VISIBLE_IMAGE_TIMEOUT_MS || 15000);
const navigationTimeoutMs = Number(process.env.CHROME_NAVIGATION_TIMEOUT_MS || 60000);
const afterSettleScript = process.env.CHROME_AFTER_SETTLE_SCRIPT || "";
const domSnapshotDir = process.env.CHROME_DOM_SNAPSHOT_DIR || "";
const transparentBackground = process.env.CHROME_TRANSPARENT_BACKGROUND === "1";
const dumpLayerCommands = process.env.CHROME_DUMP_LAYER_COMMANDS === "1";
const dumpLayerSnapshotsDir = process.env.CHROME_DUMP_LAYER_SNAPSHOTS_DIR || "";
const dumpAllLayers = process.env.CHROME_DUMP_ALL_LAYERS === "1";
const layerPrefixSteps = JSON.parse(process.env.CHROME_LAYER_PREFIX_STEPS || "[]");
const traceOutput = process.env.CHROME_TRACE_OUTPUT || "";
const traceAfterSettle = process.env.CHROME_TRACE_AFTER_SETTLE === "1";
const forceFullRepaintAfterScroll =
  process.env.SITE_COMPARE_FORCE_FULL_REPAINT_AFTER_SCROLL === "1";
const forceRepaintSettleMs = Number(
  process.env.SITE_COMPARE_FORCE_REPAINT_SETTLE_MS || 1000
);
const staticNoPageScript = process.env.CHROME_STATIC_NO_PAGE_SCRIPT === "1";
if (!Array.isArray(extraChromeArgs) || extraChromeArgs.some(value => typeof value !== "string")) {
  throw new Error("CHROME_EXTRA_ARGS must be a JSON array of strings");
}
if (!Array.isArray(layerPrefixSteps) || layerPrefixSteps.some(value => !Number.isInteger(value) || value <= 0)) {
  throw new Error("CHROME_LAYER_PREFIX_STEPS must be a JSON array of positive integers");
}
fs.mkdirSync(outputDir, { recursive: true });
const resultsPath = path.join(outputDir, "results.json");

const profile = fs.mkdtempSync(path.join(os.tmpdir(), "rechrom-chrome-profile-"));
const chromePath = process.env.CHROME_PATH ||
  "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome";
const chrome = spawn(chromePath, [
  "--headless=new",
  "--remote-debugging-port=0",
  "--remote-allow-origins=*",
  `--user-data-dir=${profile}`,
  `--window-size=${width},${height}`,
  `--force-device-scale-factor=${dpr}`,
  "--force-color-profile=srgb",
  // Rechrom currently has only the SoftwareRenderer backend. Keep Chromium on
  // the corresponding backend so a strict pixel comparison does not randomly
  // alternate between Chromium's GPU and software image caches.
  "--disable-gpu",
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
      this.latestEvents.set(`${message.sessionId || ''}:${message.method}`, message.params || {});
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
async function saveTraceStream(cdp, handle, output) {
  const chunks = [];
  for (;;) {
    const response = await cdp.request("IO.read", { handle }, undefined, 60000);
    chunks.push(response.base64Encoded
      ? Buffer.from(response.data || "", "base64")
      : Buffer.from(response.data || "", "utf8"));
    if (response.eof) break;
  }
  await cdp.request("IO.close", { handle });
  fs.mkdirSync(path.dirname(path.resolve(output)), { recursive: true });
  fs.writeFileSync(path.resolve(output), Buffer.concat(chunks));
}
async function waitForFontsAndFrames(cdp, sessionId, timeoutMs = 15000) {
  const response = await cdp.request("Runtime.evaluate", {
    expression: `(async () => {
      const timeout = new Promise(resolve => setTimeout(() => resolve('timeout'), ${timeoutMs}));
      const fonts = document.fonts ? document.fonts.ready.then(() => document.fonts.status) : Promise.resolve('unsupported');
      const fontStatus = await Promise.race([fonts, timeout]);
      await new Promise(resolve => requestAnimationFrame(() => requestAnimationFrame(() => requestAnimationFrame(resolve))));
      return {fontStatus};
    })()`,
    awaitPromise: true,
    returnByValue: true,
  }, sessionId, timeoutMs + 5000);
  return response.result?.value || {};
}

async function captureStableScreenshot(cdp, sessionId, timeoutMs = 5000) {
  const deadline = Date.now() + timeoutMs;
  let previous = null;
  let repeats = 0;
  let latest = null;
  let attempts = 0;
  while (Date.now() < deadline) {
    latest = await cdp.request("Page.captureScreenshot", {
      format: "png", fromSurface: true, captureBeyondViewport: false,
    }, sessionId);
    attempts += 1;
    if (latest.data === previous) {
      repeats += 1;
      if (repeats >= 2) return {shot: latest, stable: true, attempts};
    } else {
      previous = latest.data;
      repeats = 0;
    }
    await sleep(250);
  }
  return {shot: latest, stable: false, attempts};
}

async function waitForVisibleImages(cdp, sessionId, timeoutMs = 15000) {
  const response = await cdp.request("Runtime.evaluate", {
    expression: `(async () => {
      const visible = image => {
        const r=image.getBoundingClientRect();
        return r.width > 0 && r.height > 0 && r.bottom > 0 && r.right > 0 && r.top < innerHeight && r.left < innerWidth;
      };
      const pending = Array.from(document.images).filter(image => visible(image) && !image.complete);
      await Promise.all(pending.map(image => new Promise(resolve => {
        const done=()=>resolve();
        image.addEventListener('load', done, {once:true});
        image.addEventListener('error', done, {once:true});
        setTimeout(done, ${timeoutMs});
      })));
      await new Promise(resolve => requestAnimationFrame(() => requestAnimationFrame(resolve)));
      const current = Array.from(document.images).filter(visible);
      return {
        visible: current.length,
        complete: current.filter(image => image.complete).length,
        decoded: current.filter(image => image.complete && image.naturalWidth > 0).length,
        images: current.map(image => {
          const r=image.getBoundingClientRect();
          return {
            src:image.currentSrc || image.src,
            complete:image.complete,
            naturalWidth:image.naturalWidth,
            naturalHeight:image.naturalHeight,
            rect:[r.x,r.y,r.width,r.height],
          };
        }),
      };
    })()`,
    awaitPromise: true,
    returnByValue: true,
  }, sessionId, timeoutMs + 5000);
  return response.result?.value || {};
}

const results = fs.existsSync(resultsPath)
  ? JSON.parse(fs.readFileSync(resultsPath, "utf8"))
  : [];
const completed = new Set(results.map(row => row.id));
let cdp;
let traceStarted = false;
try {
  cdp = new CDP(await browserWebSocket());
  await cdp.connect();
  if (traceOutput && !traceAfterSettle) {
    await cdp.request("Tracing.start", {
      categories: "disabled-by-default-cc.debug,disabled-by-default-devtools.timeline.layers,cc",
      transferMode: "ReturnAsStream",
    });
    traceStarted = true;
  }
  const { targetId } = await cdp.request("Target.createTarget", { url: "about:blank" });
  const { sessionId } = await cdp.request("Target.attachToTarget", { targetId, flatten: true });
  await cdp.request("Page.enable", {}, sessionId);
  await cdp.request("Runtime.enable", {}, sessionId);
  await cdp.request("Performance.enable", {}, sessionId);
  await cdp.request("Network.enable", {}, sessionId);
  await cdp.request("DOM.enable", {}, sessionId);
  await cdp.request("CSS.enable", {}, sessionId);
  if (dumpLayerCommands) {
    await cdp.request("LayerTree.enable", {}, sessionId);
  }
  await cdp.request("Network.setUserAgentOverride", {
    userAgent: "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/154.0.0.0 Safari/537.36",
    acceptLanguage: "en-US,en;q=0.9",
    platform: "MacIntel",
  }, sessionId);
  await cdp.request("Emulation.setDeviceMetricsOverride", {
    width, height, deviceScaleFactor: dpr, mobile: false,
  }, sessionId);
  if (transparentBackground) {
    await cdp.request("Emulation.setDefaultBackgroundColorOverride", {
      color: { r: 0, g: 0, b: 0, a: 0 },
    }, sessionId);
  }
  await cdp.request("Emulation.setEmulatedMedia", {
    media: "screen",
    features: [
      { name: "prefers-color-scheme", value: "light" },
      { name: "prefers-reduced-motion", value: "reduce" },
    ],
  }, sessionId);

  for (const site of sites) {
    if (completed.has(site.id)) continue;
    const row = { id: site.id, url: site.url, screenshots: {}, error: null };
    try {
      if (staticNoPageScript) {
        await cdp.request("Emulation.setScriptExecutionDisabled", { value: true }, sessionId);
      }
      const domReady = cdp.wait("Page.domContentEventFired", sessionId, navigationTimeoutMs)
        .then(() => true, () => false);
      const navigationRequest = cdp.request("Page.navigate", { url: site.url }, sessionId)
        .then(value => ({ value }), error => ({ error }));
      const navigation = await Promise.race([
        navigationRequest,
        sleep(navigationTimeoutMs).then(() => null),
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
      await sleep(settleMs);
      if (staticNoPageScript) {
        row.scroll_positions = {};
        let previousY = 0;
        for (const y of site.scrolls) {
          const requestedY = Number(y);
          const deltaY = requestedY - previousY;
          if (deltaY) {
            await cdp.request("Input.dispatchMouseEvent", {
              type: "mouseWheel",
              x: width / 2,
              y: height / 2,
              deltaX: 0,
              deltaY,
            }, sessionId);
            await sleep(200);
          }
          const stableCapture = await captureStableScreenshot(cdp, sessionId);
          const layoutMetrics = await cdp.request("Page.getLayoutMetrics", {}, sessionId);
          const filename = `${site.id}__y${y}.png`;
          fs.writeFileSync(
            path.join(outputDir, filename),
            Buffer.from(stableCapture.shot.data, "base64"),
          );
          row.screenshots[String(y)] = filename;
          row.scroll_positions[String(y)] = {
            requested_y: requestedY,
            actual_y: layoutMetrics.cssVisualViewport?.pageY ??
              layoutMetrics.visualViewport?.pageY,
            content_height: layoutMetrics.cssContentSize?.height ??
              layoutMetrics.contentSize?.height,
            client_height: layoutMetrics.cssLayoutViewport?.clientHeight ??
              layoutMetrics.layoutViewport?.clientHeight,
            stable_frame: stableCapture.stable,
            stability_attempts: stableCapture.attempts,
          };
          previousY = requestedY;
        }
        results.push(row);
        fs.writeFileSync(resultsPath, JSON.stringify(results, null, 2));
        continue;
      }
      row.render_readiness = await waitForFontsAndFrames(cdp, sessionId);
      if (domSnapshotDir) {
        const snapshot = await cdp.request("Runtime.evaluate", {
          expression: `(() => {
            const clone=document.documentElement.cloneNode(true);
            clone.querySelectorAll('script,noscript,template[data-target="react-app.embeddedData"]')
              .forEach(node=>node.remove());
            clone.querySelectorAll('link[rel="modulepreload"],link[rel="preload"][as="script"]')
              .forEach(node=>node.remove());
            clone.querySelectorAll('meta[http-equiv="Content-Security-Policy"]')
              .forEach(node=>node.remove());
            clone.querySelectorAll('img').forEach(image=>{
              image.setAttribute('loading','eager');
              image.setAttribute('decoding','sync');
              image.setAttribute('fetchpriority','high');
            });
            const base=document.createElement('base');
            base.href=location.href;
            const freeze=document.createElement('style');
            freeze.id='__rechrom_frozen_snapshot_style';
            freeze.textContent='*,*::before,*::after{animation:none!important;transition:none!important;caret-color:transparent!important;scroll-behavior:auto!important}';
            (clone.querySelector('head') || clone).prepend(base,freeze);
            return '<!doctype html>\\n' + clone.outerHTML;
          })()`,
          returnByValue: true,
        }, sessionId, 60000);
        const safeId = String(site.id).replace(/[^A-Za-z0-9_.-]+/g, "_");
        const snapshotPath = path.resolve(domSnapshotDir, `${safeId}.html`);
        fs.mkdirSync(path.dirname(snapshotPath), { recursive: true });
        fs.writeFileSync(snapshotPath, snapshot.result?.value || "", "utf8");
        row.dom_snapshot = snapshotPath;
      }
      if (!preserveMotion) await cdp.request("Runtime.evaluate", { expression: `(() => {
        let s=document.getElementById('__rechrom_compare_style');
        if(!s){s=document.createElement('style');s.id='__rechrom_compare_style';document.documentElement.appendChild(s)}
        s.textContent='*,*::before,*::after{animation:none!important;transition:none!important;caret-color:transparent!important;scroll-behavior:auto!important}';
      })()` }, sessionId);
      if (afterSettleScript) {
        await cdp.request(
          "Runtime.evaluate",
          { expression: afterSettleScript, awaitPromise: true },
          sessionId,
        );
        const probe = await cdp.request(
          "Runtime.evaluate",
          {
            expression: "document.documentElement.dataset.rechromCompareProbe || null",
            returnByValue: true,
          },
          sessionId,
        );
        if (probe.result?.value != null) row.after_settle_probe = probe.result.value;
      }
      await sleep(250);
      row.render_readiness.after_style = await waitForFontsAndFrames(cdp, sessionId);
      if (traceOutput && traceAfterSettle && !traceStarted) {
        await cdp.request("Tracing.start", {
          categories: "disabled-by-default-cc.debug,disabled-by-default-devtools.timeline.layers,cc",
          transferMode: "ReturnAsStream",
        });
        traceStarted = true;
      }
      const timingResponse = await cdp.request("Runtime.evaluate", {
        expression: `JSON.stringify((() => { const n=performance.getEntriesByType('navigation')[0]; const p=performance.getEntriesByType('paint'); const first=document.body?.firstElementChild; const rect=e=>e?Object.fromEntries(['x','y','width','height','top','right','bottom','left'].map(k=>[k,e.getBoundingClientRect()[k]])):null; const style=document.body?getComputedStyle(document.body):null; const headings=Array.from(document.querySelectorAll('h2')); const metric=e=>{const r=e.getBoundingClientRect(),s=getComputedStyle(e);return {tag:e.tagName,id:e.id,class:e.className,text:e.textContent.trim().slice(0,80),top:r.top+scrollY,height:r.height,display:s.display,position:s.position,grid_template_columns:s.gridTemplateColumns,grid_template_rows:s.gridTemplateRows}}; const gridHeading=headings.find(e=>e.textContent.trim()==='Grid layout in action'); return {href:location.href,title:document.title,scroll_height:document.documentElement.scrollHeight,headings:headings.map(e=>({text:e.textContent.trim(),top:e.getBoundingClientRect().top+scrollY,height:e.getBoundingClientRect().height})),grid_section:gridHeading?{section:metric(gridHeading.parentElement),children:Array.from(gridHeading.parentElement.children).map(metric)}:null,phase_geometry:typeof collectPhaseGeometry==='function'?collectPhaseGeometry():null,nav:n?{response_end:n.responseEnd,dom_content_loaded:n.domContentLoadedEventEnd,load:n.loadEventEnd}:null,paints:Object.fromEntries(p.map(x=>[x.name,x.startTime])),geometry:{document:rect(document.documentElement),body:rect(document.body),first:rect(first)},body_style:style?{font:style.font,font_family:style.fontFamily,font_size:style.fontSize,line_height:style.lineHeight,padding_top:style.paddingTop}:null} })())`,
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
      for (const [name, nodeId] of [
        ["body", bodyNode.nodeId],
        ["first", firstNode.nodeId],
      ]) {
        if (!nodeId) continue;
        row.platform_fonts[name] = (await cdp.request(
          "CSS.getPlatformFontsForNode", { nodeId }, sessionId
        )).fonts;
      }
      const search = await cdp.request("DOM.performSearch", {
        query: "header a, header button, nav a, nav button, main h1, main h2, main p",
        includeUserAgentShadowDOM: false,
      }, sessionId);
      if (search.resultCount) {
        const candidates = await cdp.request("DOM.getSearchResults", {
          searchId: search.searchId,
          fromIndex: 0,
          toIndex: Math.min(search.resultCount, 32),
        }, sessionId);
        for (const nodeId of candidates.nodeIds || []) {
          const fonts = (await cdp.request(
            "CSS.getPlatformFontsForNode", { nodeId }, sessionId
          )).fonts;
          if (fonts.length) {
            row.platform_fonts.representative_text = fonts;
            break;
          }
        }
      }
      await cdp.request("DOM.discardSearchResults", { searchId: search.searchId }, sessionId);
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
        row.scroll_positions[String(y)].visible_images = await waitForVisibleImages(
          cdp, sessionId, visibleImageTimeoutMs
        );
        if (forceFullRepaintAfterScroll) {
          await cdp.request("Runtime.evaluate", {
            expression: `(async () => {
              const root=document.documentElement;
              const previous=root.style.visibility;
              root.style.visibility='hidden';
              await new Promise(resolve => requestAnimationFrame(() => requestAnimationFrame(resolve)));
              root.style.visibility=previous;
              await new Promise(resolve => requestAnimationFrame(() => requestAnimationFrame(resolve)));
            })()`,
            awaitPromise: true,
          }, sessionId);
          if (forceRepaintSettleMs > 0) {
            await sleep(forceRepaintSettleMs);
          }
        }
        if (dumpLayerCommands) {
          const treeKey = `${sessionId}:LayerTree.layerTreeDidChange`;
          // Use the most recently reported live tree. Disabling LayerTree
          // invalidates every layer id before makeSnapshot can consume it.
          const tree = cdp.latestEvents.get(treeKey) ||
            [...cdp.latestEvents.entries()].find(([key]) =>
              key.endsWith(":LayerTree.layerTreeDidChange")
            )?.[1] || null;
          if (dumpLayerSnapshotsDir && tree?.layers) {
            const directory = path.resolve(dumpLayerSnapshotsDir);
            fs.mkdirSync(directory, { recursive: true });
            const safeId = String(site.id).replace(/[^A-Za-z0-9_.-]+/g, "_");
            fs.writeFileSync(
              path.join(directory, `${safeId}__y${y}__tree.json`),
              JSON.stringify(tree, null, 2),
            );
          }
          for (const layer of tree?.layers || []) {
            if (!layer.drawsContent) continue;
            let snapshotId;
            try {
              ({ snapshotId } = await cdp.request(
                "LayerTree.makeSnapshot", { layerId: layer.layerId }, sessionId
              ));
              const { commandLog = [] } = await cdp.request(
                "LayerTree.snapshotCommandLog", { snapshotId }, sessionId
              );
              const matches = commandLog.filter(command => {
                const text = JSON.stringify(command);
                return text.includes("drawRRect");
              });
              if (matches.length || dumpAllLayers) {
                process.stderr.write(`chrome-layer-probe\t${JSON.stringify({ layer, matches })}\n`);
                if (dumpLayerSnapshotsDir) {
                  const replay = await cdp.request(
                    "LayerTree.replaySnapshot",
                    { snapshotId, scale: 1 },
                    sessionId,
                  );
                  const encoded = String(replay.dataURL || "").replace(
                    /^data:image\/png;base64,/,
                    "",
                  );
                  const directory = path.resolve(dumpLayerSnapshotsDir);
                  fs.mkdirSync(directory, { recursive: true });
                  const stem = `${String(site.id).replace(/[^A-Za-z0-9_.-]+/g, "_")}__y${y}__layer-${layer.layerId}`;
                  fs.writeFileSync(path.join(directory, `${stem}.png`), Buffer.from(encoded, "base64"));
                  fs.writeFileSync(
                    path.join(directory, `${stem}.json`),
                    JSON.stringify({ layer, matches, commandLog }, null, 2),
                  );
                  for (const toStep of layerPrefixSteps) {
                    if (toStep > commandLog.length) continue;
                    const prefixReplay = await cdp.request(
                      "LayerTree.replaySnapshot",
                      { snapshotId, fromStep: 0, toStep, scale: 1 },
                      sessionId,
                    );
                    const prefixEncoded = String(prefixReplay.dataURL || "").replace(
                      /^data:image\/png;base64,/,
                      "",
                    );
                    fs.writeFileSync(
                      path.join(directory, `${stem}__to-${toStep}.png`),
                      Buffer.from(prefixEncoded, "base64"),
                    );
                  }
                }
              }
            } catch (error) {
              process.stderr.write(`chrome-layer-probe-error\t${layer.layerId}\t${error}\n`);
            } finally {
              if (snapshotId) {
                await cdp.request("LayerTree.releaseSnapshot", { snapshotId }, sessionId);
              }
            }
          }
        }
        const stableCapture = await captureStableScreenshot(cdp, sessionId);
        const shot = stableCapture.shot;
        row.scroll_positions[String(y)].stable_frame = stableCapture.stable;
        row.scroll_positions[String(y)].stability_attempts = stableCapture.attempts;
        const probe = await cdp.request("Runtime.evaluate", {
          expression: "document.documentElement.dataset.rechromCompareProbe || null",
          returnByValue: true,
        }, sessionId);
        if (probe.result?.value != null) {
          row.scroll_positions[String(y)].probe = probe.result.value;
        }
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
  if (traceStarted && cdp?.ws?.readyState === WebSocket.OPEN) {
    try {
      const complete = cdp.wait("Tracing.tracingComplete", undefined, 60000);
      await cdp.request("Tracing.end", {}, undefined, 60000);
      const { stream } = await complete;
      if (stream) await saveTraceStream(cdp, stream, traceOutput);
    } catch (error) {
      process.stderr.write(`chrome-trace-error\t${error}\n`);
    }
  }
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
