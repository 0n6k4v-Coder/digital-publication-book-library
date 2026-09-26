import { defineConfig, loadEnv } from "vite";
import react from "@vitejs/plugin-react";

export default defineConfig(({ mode }) => {
  const env = loadEnv(mode, ".", "");
  const backendProxyTarget = env.BACKEND_PROXY_TARGET?.trim() ?? "";

  const apiProxy =
    backendProxyTarget.length > 0
      ? {
          target: backendProxyTarget,
          changeOrigin: true,
          secure: false,
        }
      : undefined;

  const adminApiProxy =
    apiProxy === undefined
      ? undefined
      : {
          ...apiProxy,
          bypass: (request: {
            method?: string;
            headers: Record<string, string | string[] | undefined>;
            url?: string;
          }) => {
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

  return {
    plugins: [react()],
    server: {
      host: "127.0.0.1",
      port: 5173,
      strictPort: true,
      proxy:
        apiProxy === undefined
          ? undefined
          : {
              "/auth": apiProxy,
              "/admin": adminApiProxy,
            },
    },
    preview: {
      host: "127.0.0.1",
      port: 4173,
      strictPort: true,
    },
  };
});
