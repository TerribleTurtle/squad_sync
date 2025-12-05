---
description: Deploy the Signaling Server, Web App, and Build Desktop
---

# Deployment & Distribution Guide

This project consists of three parts:

1. **Signaling Server** (PartyKit)
2. **Web App** (Next.js)
3. **Desktop App** (Tauri)

## 1. Signaling Server

The signaling server is hosted on PartyKit.

**To deploy manually:**

```bash
cd apps/signaling
pnpm deploy
```

_Note: You must be logged in to PartyKit (`npx partykit login`)._

## 2. Web App

The web application is a Next.js app.

**To deploy:**

- **Vercel/Netlify:** Simply commit and push your changes to the `main` branch. The CI/CD pipeline should handle the rest.
- **Manual Build:**

```bash
cd apps/web
pnpm build
```

## 3. Desktop App

The desktop application must be built locally or via CI (e.g., GitHub Actions) for each OS.

**To build for Windows (Installer):**
// turbo

```bash
pnpm build:desktop
```

_This will:_

1. Build the `shared` package.
2. Build the desktop frontend (Vite).
3. Compile the Rust backend and package it with NSIS.

**Output Location:**
The installer `.exe` will be found in:
`apps/desktop/src-tauri/target/release/bundle/nsis/`

## 4. Git Push

To save your changes and trigger any connected CI/CD pipelines:

```bash
git add .
git commit -m "Update deployment"
git push
```
