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
            },
    },
    preview: {
      host: "127.0.0.1",
      port: 4173,
      strictPort: true,
    },
  };
});