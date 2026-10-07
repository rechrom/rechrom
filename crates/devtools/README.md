# Rechrom DevTools

This crate provides the local Performance protocol and UI used by the Rechrom desktop app. Press F12 in the browser to open it.

The crate depends only on tracing data and protocol types. It does not depend on `Page`, `rechrom`, or `rechrom_app`, so engine code can publish measurements without depending on the UI.

The default local endpoint is `http://127.0.0.1:9223/`. It exposes recording controls and trace export for the built-in frontend and automation clients.
