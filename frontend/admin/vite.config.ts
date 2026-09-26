import { defineConfig, loadEnv } from "vite";
import type { ProxyOptions } from "vite";
import { readFileSync } from "node:fs";
import react from "@vitejs/plugin-react";

export default defineConfig(({ mode }) => {
  const env = loadEnv(mode, ".", "");
  const backendProxyTarget = env.BACKEND_PROXY_TARGET?.trim() ?? "";

  const devHttpsKeyPath = env.DEV_HTTPS_KEY_PATH?.trim() ?? "";
  const devHttpsCertPath = env.DEV_HTTPS_CERT_PATH?.trim() ?? "";

  if ((devHttpsKeyPath.length === 0) !== (devHttpsCertPath.length === 0)) {
    throw new Error(
      "DEV_HTTPS_KEY_PATH and DEV_HTTPS_CERT_PATH must be provided together.",
    );
  }

  const devHttps =
    devHttpsKeyPath.length === 0
      ? undefined
      : {
          key: readFileSync(devHttpsKeyPath),
          cert: readFileSync(devHttpsCertPath),
        };

  const apiProxy: ProxyOptions | undefined =
    backendProxyTarget.length > 0
      ? {
          target: backendProxyTarget,
          changeOrigin: true,
          secure: false,
        }
      : undefined;

  const adminApiProxy: ProxyOptions | undefined =
    apiProxy === undefined
      ? undefined
      : {
          ...apiProxy,
          bypass: (request) => {
            const accept = request.headers.accept ?? "";

            // /admin/* is both:
            //   1. an SPA navigation path, and
            //   2. the backend Admin API path.
            //
            // Browser navigations request HTML, so let Vite handle
            // the SPA fallback. Fetch/XHR requests from the application
            // request JSON and are proxied to the backend.
            if (request.method === "GET" && accept.includes("text/html")) {
              return request.url;
            }

            return undefined;
          },
        };

  const proxy =
    apiProxy === undefined
      ? undefined
      : {
          "/auth": apiProxy,
          ...(adminApiProxy === undefined ? {} : { "/admin": adminApiProxy }),
        };

  return {
    plugins: [react()],
    server: {
      host: "127.0.0.1",
      port: 5173,
      strictPort: true,
      ...(devHttps === undefined ? {} : { https: devHttps }),
      proxy,
    },
    preview: {
      host: "127.0.0.1",
      port: 4173,
      strictPort: true,
    },
  };
});
