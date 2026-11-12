#!/usr/bin/env node
// Pure-offscreen normal-GPU Chrome video golden and macOS conversion probe.

import fs from "node:fs";
import http from "node:http";
import os from "node:os";
import path from "node:path";
import process from "node:process";
import zlib from "node:zlib";
import { spawn, spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const defaults = {
  fixture: path.resolve(ROOT, "../src/media/test/data/four-colors.mp4"),
  output: path.resolve(ROOT, "work/video-gpu-golden.json"),
  chrome: "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
  candidate: undefined,
  requireExact: false,
  keepArtifacts: false,
};

function usage() {
  console.error(`usage: node tools/video_gpu_golden.mjs [options]
  --fixture PATH       MP4 fixture (default: Chromium four-colors.mp4)
  --output PATH        JSON report (default: work/video-gpu-golden.json)
  --chrome PATH        Chrome executable
  --candidate PATH     optional Rechrom PNG to compare to the Chrome golden
  --require-exact      fail unless --candidate is byte-exact
  --keep-artifacts     keep the temporary Chrome PNG and RGBA candidates`);
}

function parseArgs(argv) {
  const options = { ...defaults };
  for (let index = 0; index < argv.length; ++index) {
    const argument = argv[index];
    if (argument === "--require-exact") options.requireExact = true;
    else if (argument === "--keep-artifacts") options.keepArtifacts = true;
    else if (["--fixture", "--output", "--chrome", "--candidate"].includes(argument)) {
      if (++index >= argv.length) throw new Error(`${argument} requires a value`);
      const key = argument.slice(2);
      options[key] = path.resolve(argv[index]);
    } else if (argument === "--help" || argument === "-h") {
      usage();
      process.exit(0);
    } else {
      throw new Error(`unknown argument: ${argument}`);
    }
  }
  if (options.requireExact && !options.candidate)
    throw new Error("--require-exact requires --candidate");
  return options;
}

const sleep = milliseconds => new Promise(resolve => setTimeout(resolve, milliseconds));

class CDP {
  constructor(url) {
    this.url = url;
    this.nextId = 1;
    this.pending = new Map();
  }
  async connect() {
    this.ws = new WebSocket(this.url);
    await new Promise((resolve, reject) => {
      this.ws.addEventListener("open", resolve, { once: true });
      this.ws.addEventListener("error", reject, { once: true });
    });
    this.ws.addEventListener("message", event => {
      const message = JSON.parse(event.data);
      if (!message.id) return;
      const pending = this.pending.get(message.id);
      if (!pending) return;
      this.pending.delete(message.id);
      if (message.error) pending.reject(new Error(JSON.stringify(message.error)));
      else pending.resolve(message.result || {});
    });
  }
  request(method, params = {}, sessionId = undefined, timeoutMs = 20000) {
    const id = this.nextId++;
    return new Promise((resolve, reject) => {
      const timer = setTimeout(() => {
        this.pending.delete(id);
        reject(new Error(`${method} timeout`));
      }, timeoutMs);
      this.pending.set(id, {
        resolve: value => { clearTimeout(timer); resolve(value); },
        reject: error => { clearTimeout(timer); reject(error); },
      });
      this.ws.send(JSON.stringify({ id, method, params, ...(sessionId ? { sessionId } : {}) }));
    });
  }
}

function chromeWebSocket(chrome) {
  return new Promise((resolve, reject) => {
    let stderr = "";
    const timer = setTimeout(() => {
      reject(new Error(`Chrome DevTools endpoint timeout: ${stderr.slice(-2000)}`));
    }, 20000);
    chrome.stderr.setEncoding("utf8");
    chrome.stderr.on("data", chunk => {
      stderr += chunk;
      const match = stderr.match(/DevTools listening on (ws:\/\/[^\s]+)/);
      if (match) {
        clearTimeout(timer);
        resolve(match[1]);
      }
    });
    chrome.once("exit", code => {
      clearTimeout(timer);
      reject(new Error(`Chrome exited before DevTools was ready (${code}): ${stderr.slice(-2000)}`));
    });
  });
}

function fixtureServer(fixture) {
  const bytes = fs.readFileSync(fixture);
  const html = `<!doctype html>
<meta charset="utf-8">
<style>*{margin:0;padding:0}html,body,video{display:block;width:960px;height:540px;overflow:hidden;background:#000}</style>
<video id="video" muted autoplay playsinline preload="auto" src="/fixture.mp4"></video>`;
  return http.createServer((request, response) => {
    if (request.url !== "/fixture.mp4") {
      response.writeHead(200, { "Content-Type": "text/html", "Content-Length": Buffer.byteLength(html) });
      response.end(html);
      return;
    }
    const match = /bytes=(\d*)-(\d*)/.exec(request.headers.range || "");
    if (!match) {
      response.writeHead(200, {
        "Content-Type": "video/mp4", "Content-Length": bytes.length, "Accept-Ranges": "bytes",
      });
      response.end(bytes);
      return;
    }
    const start = match[1] ? Number(match[1]) : 0;
    const end = match[2] ? Number(match[2]) : bytes.length - 1;
    response.writeHead(206, {
      "Content-Type": "video/mp4",
      "Content-Length": end - start + 1,
      "Content-Range": `bytes ${start}-${end}/${bytes.length}`,
      "Accept-Ranges": "bytes",
    });
    response.end(bytes.subarray(start, end + 1));
  });
}

async function captureChrome(options, temporary) {
  const server = fixtureServer(options.fixture);
  await new Promise((resolve, reject) => {
    server.once("error", reject);
    server.listen(0, "127.0.0.1", resolve);
  });
  const profile = path.join(temporary, "chrome-profile");
  fs.mkdirSync(profile);
  const chrome = spawn(options.chrome, [
    "--headless=new", "--remote-debugging-port=0", "--remote-allow-origins=*",
    `--user-data-dir=${profile}`, "--window-size=960,540", "--force-device-scale-factor=1",
    "--force-color-profile=srgb", "--autoplay-policy=no-user-gesture-required",
    "--hide-scrollbars", "--disable-background-networking", "--disable-component-update",
    "--disable-default-apps", "--disable-extensions", "--no-first-run",
    "--no-default-browser-check", "about:blank",
  ], { stdio: ["ignore", "ignore", "pipe"] });
  let cdp;
  try {
    cdp = new CDP(await chromeWebSocket(chrome));
    await cdp.connect();
    const version = await cdp.request("Browser.getVersion");
    const system = await cdp.request("SystemInfo.getInfo");
    const { targetId } = await cdp.request("Target.createTarget", { url: "about:blank" });
    const { sessionId } = await cdp.request("Target.attachToTarget", { targetId, flatten: true });
    await cdp.request("Page.enable", {}, sessionId);
    await cdp.request("Runtime.enable", {}, sessionId);
    await cdp.request("Emulation.setDeviceMetricsOverride", {
      width: 960, height: 540, deviceScaleFactor: 1, mobile: false,
    }, sessionId);
    const port = server.address().port;
    await cdp.request("Page.navigate", { url: `http://127.0.0.1:${port}/` }, sessionId);
    const ready = await cdp.request("Runtime.evaluate", {
      expression: `(async()=>{
        const video=document.querySelector('video');
        if(video.readyState<2) await new Promise((resolve,reject)=>{
          video.addEventListener('loadeddata',resolve,{once:true});
          video.addEventListener('error',()=>reject(video.error),{once:true});
        });
        video.pause(); video.currentTime=0;
        await new Promise(resolve=>{
          const painted=()=>requestAnimationFrame(()=>requestAnimationFrame(resolve));
          video.requestVideoFrameCallback ? video.requestVideoFrameCallback(painted) : painted();
        });
        return {readyState:video.readyState,currentTime:video.currentTime,
                videoWidth:video.videoWidth,videoHeight:video.videoHeight};
      })()`,
      awaitPromise: true,
      returnByValue: true,
    }, sessionId, 30000);
    const copiedFrames = await cdp.request("Runtime.evaluate", {
      expression: `(async()=>{
        const video=document.querySelector('video');
        const encode=bytes=>{
          let value='';
          for(let start=0;start<bytes.length;start+=0x8000)
            value+=String.fromCharCode(...bytes.subarray(start,start+0x8000));
          return btoa(value);
        };
        const copy=async(frame,source)=>{
          try {
            const bytes=new Uint8Array(frame.allocationSize());
            const layouts=await frame.copyTo(bytes);
            let rgbaCopy;
            try {
              const rgbaBytes=new Uint8Array(frame.allocationSize({format:'RGBA'}));
              const rgbaLayouts=await frame.copyTo(rgbaBytes,{format:'RGBA'});
              rgbaCopy={format:'RGBA',layouts:rgbaLayouts,data:encode(rgbaBytes)};
            } catch(error) {
              rgbaCopy={format:'RGBA',error:String(error && (error.stack||error))};
            }
            const colorSpace=frame.colorSpace ? {
              primaries:frame.colorSpace.primaries,
              transfer:frame.colorSpace.transfer,
              matrix:frame.colorSpace.matrix,
              fullRange:frame.colorSpace.fullRange,
            } : null;
            return {source,format:frame.format,codedWidth:frame.codedWidth,
                    codedHeight:frame.codedHeight,displayWidth:frame.displayWidth,
                    displayHeight:frame.displayHeight,visibleRect:frame.visibleRect,
                    colorSpace,layouts,data:encode(bytes),rgbaCopy};
          } catch(error) {
            return {source,error:String(error && (error.stack||error))};
          } finally { frame.close(); }
        };
        const frames=[];
        try { frames.push(await copy(new VideoFrame(video),'html-video-element')); }
        catch(error) { frames.push({source:'html-video-element',error:String(error)}); }
        try {
          if(!video.captureStream) throw new Error('HTMLVideoElement.captureStream unavailable');
          if(!globalThis.MediaStreamTrackProcessor)
            throw new Error('MediaStreamTrackProcessor unavailable');
          video.currentTime=0;
          const stream=video.captureStream();
          const track=stream.getVideoTracks()[0];
          if(!track) throw new Error('captureStream exposed no video track');
          const processor=new MediaStreamTrackProcessor({track});
          const reader=processor.readable.getReader();
          await video.play();
          const result=await Promise.race([
            reader.read(),
            new Promise((_,reject)=>setTimeout(()=>reject(new Error('track frame timeout')),5000)),
          ]);
          video.pause();
          if(result.done||!result.value) throw new Error('track ended before a frame');
          frames.push(await copy(result.value,'media-stream-track-processor'));
          await reader.cancel();
          track.stop();
        } catch(error) {
          frames.push({source:'media-stream-track-processor',error:String(error && (error.stack||error))});
        }
        return frames;
      })()`,
      awaitPromise: true,
      returnByValue: true,
    }, sessionId, 30000);
    const screenshot = await cdp.request("Page.captureScreenshot", {
      format: "png", fromSurface: true, captureBeyondViewport: false,
    }, sessionId, 30000);
    const screenshotPath = path.join(temporary, "chrome-normal-gpu.png");
    fs.writeFileSync(screenshotPath, Buffer.from(screenshot.data, "base64"));
    const videoFrames = (copiedFrames.result?.value || []).map((frame, index) => {
      const data = frame.data ? Buffer.from(frame.data, "base64") : undefined;
      const rgbaData = frame.rgbaCopy?.data
        ? Buffer.from(frame.rgbaCopy.data, "base64")
        : undefined;
      const metadata = { ...frame };
      delete metadata.data;
      if (metadata.rgbaCopy) delete metadata.rgbaCopy.data;
      if (data) fs.writeFileSync(path.join(temporary, `chrome-frame-${index}.raw`), data);
      if (rgbaData) fs.writeFileSync(path.join(temporary, `chrome-frame-${index}-rgba.raw`), rgbaData);
      return { metadata, data, rgbaData };
    });
    return {
      screenshotPath,
      version,
      gpu: {
        devices: system.gpu?.devices || [],
        feature_status: system.gpu?.featureStatus || {},
        driver_bug_workarounds: system.gpu?.driverBugWorkarounds || [],
      },
      video: ready.result?.value,
      videoFrames,
    };
  } finally {
    try { cdp?.ws?.close(); } catch {}
    try { chrome.kill("SIGTERM"); } catch {}
    server.close();
    await sleep(500);
  }
}

function paeth(a, b, c) {
  const p = a + b - c;
  const pa = Math.abs(p - a), pb = Math.abs(p - b), pc = Math.abs(p - c);
  return pa <= pb && pa <= pc ? a : pb <= pc ? b : c;
}

function decodePng(png) {
  const signature = Buffer.from([137, 80, 78, 71, 13, 10, 26, 10]);
  if (!png.subarray(0, 8).equals(signature)) throw new Error("candidate is not a PNG");
  let offset = 8, width, height, bitDepth, colorType, interlace;
  const idat = [];
  while (offset < png.length) {
    const length = png.readUInt32BE(offset);
    const type = png.toString("ascii", offset + 4, offset + 8);
    const data = png.subarray(offset + 8, offset + 8 + length);
    offset += 12 + length;
    if (type === "IHDR") {
      width = data.readUInt32BE(0); height = data.readUInt32BE(4);
      bitDepth = data[8]; colorType = data[9]; interlace = data[12];
    } else if (type === "IDAT") idat.push(data);
    else if (type === "IEND") break;
  }
  if (bitDepth !== 8 || ![2, 6].includes(colorType) || interlace !== 0)
    throw new Error(`unsupported PNG layout: depth=${bitDepth} color=${colorType} interlace=${interlace}`);
  const channels = colorType === 6 ? 4 : 3;
  const stride = width * channels;
  const packed = zlib.inflateSync(Buffer.concat(idat));
  const decoded = Buffer.alloc(stride * height);
  let source = 0;
  for (let y = 0; y < height; ++y) {
    const filter = packed[source++];
    for (let x = 0; x < stride; ++x) {
      const raw = packed[source++];
      const left = x >= channels ? decoded[y * stride + x - channels] : 0;
      const above = y ? decoded[(y - 1) * stride + x] : 0;
      const upperLeft = y && x >= channels ? decoded[(y - 1) * stride + x - channels] : 0;
      const predictor = filter === 0 ? 0 : filter === 1 ? left : filter === 2 ? above
        : filter === 3 ? Math.floor((left + above) / 2) : filter === 4
          ? paeth(left, above, upperLeft) : NaN;
      if (Number.isNaN(predictor)) throw new Error(`unsupported PNG filter ${filter}`);
      decoded[y * stride + x] = (raw + predictor) & 255;
    }
  }
  if (channels === 4) return { width, height, rgba: decoded };
  const rgba = Buffer.alloc(width * height * 4);
  for (let pixel = 0; pixel < width * height; ++pixel) {
    decoded.copy(rgba, pixel * 4, pixel * 3, pixel * 3 + 3);
    rgba[pixel * 4 + 3] = 255;
  }
  return { width, height, rgba };
}

function metrics(reference, candidate, width, height, mask = undefined) {
  if (reference.length !== candidate.length) throw new Error("RGBA byte lengths differ");
  const sums = [0, 0, 0, 0], maxima = [0, 0, 0, 0];
  let changed = 0, pixels = 0;
  for (let y = 0; y < height; ++y) {
    for (let x = 0; x < width; ++x) {
      if (mask && !mask(x, y)) continue;
      ++pixels;
      const offset = (y * width + x) * 4;
      let pixelChanged = false;
      for (let channel = 0; channel < 4; ++channel) {
        const difference = Math.abs(reference[offset + channel] - candidate[offset + channel]);
        sums[channel] += difference;
        maxima[channel] = Math.max(maxima[channel], difference);
        pixelChanged ||= difference !== 0;
      }
      changed += Number(pixelChanged);
    }
  }
  return {
    exact: changed === 0,
    pixels,
    changed_pixels: changed,
    changed_ratio: pixels ? changed / pixels : 0,
    mae_rgba: sums.map(value => pixels ? value / pixels : 0),
    max_rgba: maxima,
  };
}

function compileAndRunProbe(options, temporary) {
  const source = path.join(ROOT, "tools/macos_video_gpu_golden_probe.m");
  const binary = path.join(temporary, "macos-video-gpu-golden-probe");
  const compile = spawnSync("xcrun", [
    "--sdk", "macosx", "clang", "-fobjc-arc", "-O2", source, "-o", binary,
    "-framework", "Foundation", "-framework", "AVFoundation",
    "-framework", "CoreMedia", "-framework", "CoreVideo",
    "-framework", "CoreImage", "-framework", "CoreGraphics",
  ], { encoding: "utf8" });
  if (compile.status !== 0) throw new Error(`probe compile failed: ${compile.stderr}`);
  const run = spawnSync(binary, [options.fixture, temporary], { encoding: "utf8" });
  if (run.status !== 0) throw new Error(`probe failed: ${run.stderr}`);
  return JSON.parse(run.stdout);
}

function readRaw(temporary, name, expectedLength) {
  const value = fs.readFileSync(path.join(temporary, name));
  if (value.length !== expectedLength)
    throw new Error(`${name} has ${value.length} bytes, expected ${expectedLength}`);
  return value;
}

function packedPlane(bytes, layout, rowBytes, rows) {
  if (!layout || !Number.isInteger(layout.offset) || !Number.isInteger(layout.stride))
    throw new Error("VideoFrame.copyTo did not return a usable plane layout");
  const output = Buffer.alloc(rowBytes * rows);
  for (let row = 0; row < rows; ++row) {
    const start = layout.offset + row * layout.stride;
    if (start < 0 || start + rowBytes > bytes.length)
      throw new Error("VideoFrame plane layout exceeds copied bytes");
    bytes.copy(output, row * rowBytes, start, start + rowBytes);
  }
  return output;
}

function byteMetrics(reference, candidate, rowBytes) {
  if (reference.length !== candidate.length)
    return { comparable: false, reference_bytes: reference.length, candidate_bytes: candidate.length };
  let changed = 0, absolute = 0, maximum = 0;
  const deltas = new Map();
  const firstDifferences = [];
  for (let index = 0; index < reference.length; ++index) {
    const delta = candidate[index] - reference[index];
    if (!delta) continue;
    ++changed;
    absolute += Math.abs(delta);
    maximum = Math.max(maximum, Math.abs(delta));
    deltas.set(delta, (deltas.get(delta) || 0) + 1);
    if (firstDifferences.length < 24) {
      firstDifferences.push({
        x: index % rowBytes,
        y: Math.floor(index / rowBytes),
        avfoundation: reference[index],
        chrome: candidate[index],
        delta,
      });
    }
  }
  return {
    comparable: true,
    exact: changed === 0,
    bytes: reference.length,
    changed_bytes: changed,
    changed_ratio: changed / reference.length,
    mae: absolute / reference.length,
    max_difference: maximum,
    delta_histogram: Object.fromEntries([...deltas].sort((a, b) => a[0] - b[0])),
    first_differences: firstDifferences,
  };
}

function compareCopiedFrame(frame, avY, avUv, width, height) {
  const metadata = frame.metadata;
  if (metadata.error) return { copy_boundary: "unavailable", error: metadata.error };
  const format = metadata.format;
  if (["RGBA", "RGBX", "BGRA", "BGRX"].includes(format)) {
    return {
      copy_boundary: "rgb-color-converted",
      explanation: "This VideoFrame is already RGB; copyTo cannot expose decoder YUV planes.",
    };
  }
  if (!frame.data) return { copy_boundary: "unavailable", error: "copyTo returned no bytes" };
  if (metadata.codedWidth !== width || metadata.codedHeight !== height) {
    return {
      copy_boundary: "native-yuv-size-mismatch",
      expected: [width, height],
      actual: [metadata.codedWidth, metadata.codedHeight],
    };
  }
  const chromaWidth = Math.ceil(width / 2);
  const chromaHeight = Math.ceil(height / 2);
  const y = packedPlane(frame.data, metadata.layouts?.[0], width, height);
  const result = {
    copy_boundary: "native-yuv",
    y: byteMetrics(avY, y, width),
  };
  if (format === "NV12") {
    const uv = packedPlane(frame.data, metadata.layouts?.[1], chromaWidth * 2, chromaHeight);
    result.uv = byteMetrics(avUv, uv, chromaWidth * 2);
    return result;
  }
  if (format === "I420" || format === "I420A") {
    const u = packedPlane(frame.data, metadata.layouts?.[1], chromaWidth, chromaHeight);
    const v = packedPlane(frame.data, metadata.layouts?.[2], chromaWidth, chromaHeight);
    const avU = Buffer.alloc(chromaWidth * chromaHeight);
    const avV = Buffer.alloc(chromaWidth * chromaHeight);
    for (let index = 0; index < avU.length; ++index) {
      avU[index] = avUv[index * 2];
      avV[index] = avUv[index * 2 + 1];
    }
    result.u = byteMetrics(avU, u, chromaWidth);
    result.v = byteMetrics(avV, v, chromaWidth);
    return result;
  }
  return {
    copy_boundary: "unsupported-yuv-format",
    format,
    y: result.y,
  };
}

async function main() {
  const options = parseArgs(process.argv.slice(2));
  if (process.platform !== "darwin") throw new Error("the AVFoundation probe requires macOS");
  if (!fs.existsSync(options.fixture)) throw new Error(`fixture does not exist: ${options.fixture}`);
  const temporary = fs.mkdtempSync(path.join(os.tmpdir(), "rechrom-video-gpu-golden-"));
  try {
    const platform = compileAndRunProbe(options, temporary);
    const chrome = await captureChrome(options, temporary);
    const golden = decodePng(fs.readFileSync(chrome.screenshotPath));
    if (golden.width !== platform.width || golden.height !== platform.height)
      throw new Error(`capture ${golden.width}x${golden.height} != decoded ${platform.width}x${platform.height}`);
    const expectedLength = golden.width * golden.height * 4;
    const names = [
      "ci_raw.rgba", "ci_srgb.rgba", "center_raw_fp32_round.rgba",
      "center_gamma_fp32_round.rgba", "center_gamma_fp32_floor.rgba",
      "center_raw_fp16_round.rgba", "center_gamma_fp16_round.rgba",
      "nearest_gamma_fp32_round.rgba",
    ];
    const candidates = Object.fromEntries(names.map(name => [name, readRaw(temporary, name, expectedLength)]));
    const avY = readRaw(temporary, "av_y.raw", platform.width * platform.height);
    const avUv = readRaw(
      temporary,
      "av_uv.raw",
      Math.ceil(platform.width / 2) * Math.ceil(platform.height / 2) * 2,
    );
    const copiedFrameComparisons = chrome.videoFrames.map(frame => ({
      ...frame.metadata,
      comparison: compareCopiedFrame(frame, avY, avUv, platform.width, platform.height),
      rgba_copy_comparison: frame.rgbaData
        ? metrics(golden.rgba, frame.rgbaData, golden.width, golden.height)
        : undefined,
      rgba_copy_vs_centered_raw: frame.rgbaData
        ? metrics(
            candidates["center_raw_fp32_round.rgba"],
            frame.rgbaData,
            golden.width,
            golden.height,
          )
        : undefined,
      rgba_copy_vs_core_image_raw: frame.rgbaData
        ? metrics(candidates["ci_raw.rgba"], frame.rgbaData, golden.width, golden.height)
        : undefined,
    }));
    const nativeCopiedFrames = copiedFrameComparisons.filter(
      frame => frame.comparison.copy_boundary === "native-yuv",
    );
    const copiedPlanesMatchAvFoundation = nativeCopiedFrames.length > 0
      && nativeCopiedFrames.every(frame =>
        Object.entries(frame.comparison)
          .filter(([name, value]) => name !== "copy_boundary" && value && "exact" in value)
          .every(([, value]) => value.exact));
    const rgbaCopiesMatchRawMatrix = copiedFrameComparisons
      .filter(frame => frame.rgba_copy_vs_centered_raw)
      .every(frame => frame.rgba_copy_vs_centered_raw.exact);
    const comparisons = Object.fromEntries(
      Object.entries(candidates).map(([name, bytes]) => [name.replace(".rgba", ""), metrics(golden.rgba, bytes, golden.width, golden.height)]),
    );
    const edgeMask = (x, y) => Math.abs(x - golden.width / 2) <= 8 || Math.abs(y - golden.height / 2) <= 8;
    const stages = {
      centered_sampling_identity: metrics(
        candidates["ci_raw.rgba"], candidates["center_raw_fp32_round.rgba"], golden.width, golden.height),
      bt709_apple_to_srgb: metrics(
        candidates["center_raw_fp32_round.rgba"], candidates["center_gamma_fp32_round.rgba"], golden.width, golden.height),
      chroma_nearest_to_centered: metrics(
        candidates["nearest_gamma_fp32_round.rgba"], candidates["center_gamma_fp32_round.rgba"], golden.width, golden.height),
      chroma_nearest_to_centered_edge_band: metrics(
        candidates["nearest_gamma_fp32_round.rgba"], candidates["center_gamma_fp32_round.rgba"], golden.width, golden.height, edgeMask),
      fp16_stage_rounding: metrics(
        candidates["center_gamma_fp32_round.rgba"], candidates["center_gamma_fp16_round.rgba"], golden.width, golden.height),
      unorm_floor_instead_of_round: metrics(
        candidates["center_gamma_fp32_round.rgba"], candidates["center_gamma_fp32_floor.rgba"], golden.width, golden.height),
    };
    if (!stages.centered_sampling_identity.exact)
      throw new Error("controlled centered BT.709 calculation no longer matches CI raw");
    let suppliedCandidate;
    if (options.candidate) {
      const decoded = decodePng(fs.readFileSync(options.candidate));
      if (decoded.width !== golden.width || decoded.height !== golden.height)
        throw new Error(`candidate ${decoded.width}x${decoded.height} != golden ${golden.width}x${golden.height}`);
      suppliedCandidate = metrics(golden.rgba, decoded.rgba, golden.width, golden.height);
    }
    const gpuDevice = chrome.gpu.devices[0]?.deviceString || "";
    const normalGpu = chrome.gpu.feature_status.gpu_compositing === "enabled"
      && !/swiftshader|software/i.test(gpuDevice);
    const errorMagnitude = value => value.mae_rgba.reduce((sum, channel) => sum + channel, 0);
    const stagedFp32 = comparisons.center_gamma_fp32_round;
    const report = {
      fixture: options.fixture,
      capture: {
        mode: "headless-normal-gpu-foreign-layer",
        viewport: [golden.width, golden.height],
        device_scale_factor: 1,
        forced_color_profile: "srgb",
        browser: chrome.version,
        gpu: chrome.gpu,
        video: chrome.video,
        copied_video_frames: copiedFrameComparisons,
      },
      platform_frame: platform,
      chrome_golden_comparisons: comparisons,
      controlled_stage_contributions: stages,
      supplied_candidate: suppliedCandidate,
      acceptance: {
        normal_gpu: normalGpu,
        chrome_native_yuv_matches_avfoundation: copiedPlanesMatchAvFoundation,
        chrome_rgba_copy_matches_unmanaged_yuv_matrix: rgbaCopiesMatchRawMatrix,
        centered_sampling_matches_ci_raw: stages.centered_sampling_identity.exact,
        bt709_apple_gamma_required:
          errorMagnitude(stagedFp32) < errorMagnitude(comparisons.center_raw_fp32_round),
        fp32_beats_stage_quantized_fp16:
          errorMagnitude(stagedFp32) < errorMagnitude(comparisons.center_gamma_fp16_round),
        unorm_round_beats_floor:
          errorMagnitude(stagedFp32) < errorMagnitude(comparisons.center_gamma_fp32_floor),
        centered_beats_nearest:
          errorMagnitude(stagedFp32) < errorMagnitude(comparisons.nearest_gamma_fp32_round),
        supplied_candidate_exact: suppliedCandidate?.exact,
      },
      resize_video_damage_contract: {
        source_resize_events: ["WindowEvent::Resized", "Emulation.setDeviceMetricsOverride"],
        coalescing: ["unique viewport samples", "BeginFrame count", "layout/paint/commit count per sample"],
        surface_identity: ["LocalSurfaceId allocation count", "Surface::CommitFrame", "Surface::ActivateFrame", "Display::SetSurfaceId"],
        damage: ["video child-surface damage rect", "root aggregated damage rect", "damage_pixels / viewport_pixels", "EarlyOut_NoDamage"],
        video_resources: ["decoded frame revision", "NV12 upload count", "texture allocation/reuse", "raster/readback count"],
        presentation: ["Display::Resize", "Display::DrawAndSwap", "SwapBuffers", "offscreen published frame size", "request-to-matching-frame latency"],
      },
    };
    fs.mkdirSync(path.dirname(options.output), { recursive: true });
    fs.writeFileSync(options.output, `${JSON.stringify(report, null, 2)}\n`);
    console.log(JSON.stringify(report, null, 2));
    if (!normalGpu) throw new Error(`capture did not use ordinary hardware GPU: ${gpuDevice}`);
    if (options.keepArtifacts) {
      const artifactDirectory = `${options.output}.artifacts`;
      fs.mkdirSync(artifactDirectory, { recursive: true });
      fs.copyFileSync(chrome.screenshotPath, path.join(artifactDirectory, "chrome-normal-gpu.png"));
      for (const name of names) fs.copyFileSync(path.join(temporary, name), path.join(artifactDirectory, name));
      fs.copyFileSync(path.join(temporary, "av_y.raw"), path.join(artifactDirectory, "av_y.raw"));
      fs.copyFileSync(path.join(temporary, "av_uv.raw"), path.join(artifactDirectory, "av_uv.raw"));
      chrome.videoFrames.forEach((frame, index) => {
        if (frame.data)
          fs.writeFileSync(path.join(artifactDirectory, `chrome-frame-${index}.raw`), frame.data);
        if (frame.rgbaData)
          fs.writeFileSync(path.join(artifactDirectory, `chrome-frame-${index}-rgba.raw`), frame.rgbaData);
      });
    }
    if (options.requireExact && !suppliedCandidate?.exact) process.exitCode = 1;
  } finally {
    fs.rmSync(temporary, { recursive: true, force: true, maxRetries: 3, retryDelay: 100 });
  }
}

main().catch(error => {
  console.error(error.stack || String(error));
  process.exitCode = 2;
});
